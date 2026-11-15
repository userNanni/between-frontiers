//! Seeded pseudo-random numbers with deterministic stream derivation.
//!
//! # Algorithm
//!
//! The generator is **xoshiro256\*\* 1.0** by David Blackman and Sebastiano Vigna
//! ("Scrambled linear pseudorandom number generators", ACM TOMS 47(4), 2021; reference code at
//! <https://prng.di.unimi.it/xoshiro256starstar.c>): 256 bits of state, period 2²⁵⁶ − 1, passes
//! BigCrush and PractRand, a handful of integer instructions per output. Seeds are expanded with
//! **SplitMix64** (Steele, Lea and Flood, OOPSLA 2014; <https://prng.di.unimi.it/splitmix64.c>),
//! as the authors recommend.
//!
//! # Streams
//!
//! A shared generator would make results depend on the order in which threads draw from it.
//! Instead every consumer derives its own generator from `(world seed, tick, stream, entity)`
//! with [`Rng::stream`]: the same key always yields the same sequence, whatever thread runs it
//! and whatever else ran before. A system uses its stream id (from its name) and the entity it
//! is working on; see [`crate::schedule::TickCtx::rng`].

use serde::{Deserialize, Serialize};

use crate::fixed::Fixed;

const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

/// The SplitMix64 output function applied to `state + GOLDEN_GAMMA`: a bijective 64-bit mixer.
const fn mix64(state: u64) -> u64 {
    let mut z = state.wrapping_add(GOLDEN_GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// SplitMix64, used to expand a 64-bit seed into xoshiro state.
#[derive(Clone, Debug)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        let out = mix64(self.0);
        self.0 = self.0.wrapping_add(GOLDEN_GAMMA);
        out
    }
}

/// xoshiro256\*\* generator. Cloning it forks the sequence.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    /// Expands `seed` with SplitMix64.
    pub fn from_seed(seed: u64) -> Self {
        let mut expander = SplitMix64(seed);
        let s = [
            expander.next_u64(),
            expander.next_u64(),
            expander.next_u64(),
            expander.next_u64(),
        ];
        // Four consecutive SplitMix64 outputs are never all zero (the mixer is a bijection and
        // the states differ), so the state is valid.
        Self { s }
    }

    /// From raw state, as in the reference implementation.
    ///
    /// # Panics
    /// When the state is all zeros (xoshiro's only invalid state).
    pub fn from_state(s: [u64; 4]) -> Self {
        assert!(s != [0; 4], "xoshiro256** state must not be all zeros");
        Self { s }
    }

    /// Generator for one `(world seed, tick, stream, entity)` key.
    ///
    /// The key is folded through the SplitMix64 mixer one field at a time (so swapping two
    /// fields changes the result) and the result seeds the generator.
    pub fn stream(seed: u64, tick: u64, stream: u64, entity: u64) -> Self {
        let mut key = mix64(seed);
        key = mix64(key ^ tick);
        key = mix64(key ^ stream);
        key = mix64(key ^ entity);
        Self::from_seed(key)
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Next 32 random bits (the high half, the better-mixed one).
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform integer in `[0, bound)`, without modulo bias (Lemire, "Fast random integer
    /// generation in an interval", ACM TOMACS 2019).
    ///
    /// # Panics
    /// When `bound` is zero.
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "Rng::below with a zero bound");
        let mut product = u128::from(self.next_u64()) * u128::from(bound);
        let mut low = product as u64;
        if low < bound {
            let threshold = bound.wrapping_neg() % bound;
            while low < threshold {
                product = u128::from(self.next_u64()) * u128::from(bound);
                low = product as u64;
            }
        }
        (product >> 64) as u64
    }

    /// Uniform integer in `[low, high)`.
    ///
    /// # Panics
    /// When the range is empty.
    pub fn range(&mut self, low: i64, high: i64) -> i64 {
        assert!(low < high, "Rng::range with an empty range");
        let span = high.wrapping_sub(low) as u64;
        low.wrapping_add(self.below(span) as i64)
    }

    /// Uniform [`Fixed`] in `[0, 1)` at full resolution.
    pub fn unit(&mut self) -> Fixed {
        Fixed::from_raw(self.below(Fixed::SCALE as u64) as i64)
    }

    /// Uniform [`Fixed`] in `[low, high)` at full resolution.
    ///
    /// # Panics
    /// When the range is empty.
    pub fn fixed_range(&mut self, low: Fixed, high: Fixed) -> Fixed {
        Fixed::from_raw(self.range(low.raw(), high.raw()))
    }

    /// `true` with probability `p` (clamped to `[0, 1]`), at 0.001 resolution.
    pub fn chance(&mut self, p: Fixed) -> bool {
        (self.below(Fixed::SCALE as u64) as i64) < p.raw()
    }
}

#[cfg(test)]
mod tests {
    use super::{Rng, SplitMix64};
    use crate::fixed::Fixed;

    /// Reference outputs of `xoshiro256starstar.c` with state {1, 2, 3, 4}.
    #[test]
    fn matches_the_reference_xoshiro_vectors() {
        let mut rng = Rng::from_state([1, 2, 3, 4]);
        let expected: [u64; 10] = [
            11_520,
            0,
            1_509_978_240,
            1_215_971_899_390_074_240,
            1_216_172_134_540_287_360,
            607_988_272_756_665_600,
            16_172_922_978_634_559_625,
            8_476_171_486_693_032_832,
            10_595_114_339_597_558_777,
            2_904_607_092_377_533_576,
        ];
        for value in expected {
            assert_eq!(rng.next_u64(), value);
        }
    }

    /// Reference outputs of `splitmix64.c` seeded with 0, and xoshiro seeded through it with 42
    /// (both checked against the C reference code).
    #[test]
    fn matches_the_reference_splitmix_seeding() {
        let mut expander = SplitMix64(0);
        for value in [
            0xe220_a839_7b1d_cdaf,
            0x6e78_9e6a_a1b9_65f4,
            0x06c4_5d18_8009_454f,
            0xf88b_b8a8_724c_81ec,
        ] {
            assert_eq!(expander.next_u64(), value);
        }
        let mut rng = Rng::from_seed(42);
        for value in [
            0x1578_0b2e_0c2e_c716,
            0x6104_d986_6d11_3a7e,
            0xae17_5332_39e4_99a1,
            0xecb8_ad47_03b3_60a1,
        ] {
            assert_eq!(rng.next_u64(), value);
        }
    }

    #[test]
    fn streams_are_reproducible_and_distinct() {
        let draw = |seed, tick, stream, entity| Rng::stream(seed, tick, stream, entity).next_u64();
        assert_eq!(draw(1, 2, 3, 4), draw(1, 2, 3, 4));
        let base = draw(1, 2, 3, 4);
        for other in [
            draw(2, 2, 3, 4),
            draw(1, 3, 3, 4),
            draw(1, 2, 4, 4),
            draw(1, 2, 3, 5),
            draw(1, 3, 2, 4),
        ] {
            assert_ne!(base, other);
        }
        // Pinned (computed independently from the reference algorithms) so that any change to
        // the derivation, which would change every saved game, fails here.
        assert_eq!(draw(1925, 0, 0, 0), 0x60a6_b8b6_f5b3_c20b);
        assert_eq!(draw(1925, 36_524, 7, 899), 0xc969_bbc6_0c01_be77);
    }

    #[test]
    fn bounded_draws_stay_in_range_and_cover_it() {
        let mut rng = Rng::from_seed(3);
        let mut seen = [0u32; 7];
        for _ in 0..70_000 {
            let value = rng.below(7);
            seen[value as usize] += 1;
        }
        for count in seen {
            assert!((9_000..11_000).contains(&count), "{seen:?}");
        }
        for _ in 0..10_000 {
            let value = rng.range(-3, 4);
            assert!((-3..4).contains(&value));
            let unit = rng.unit();
            assert!(unit >= Fixed::ZERO && unit < Fixed::ONE);
        }
        assert!(!rng.chance(Fixed::ZERO));
        assert!(rng.chance(Fixed::ONE));
        assert_eq!(rng.range(i64::MIN, i64::MIN + 1), i64::MIN);
        let _ = rng.range(i64::MIN, i64::MAX);
    }
}
