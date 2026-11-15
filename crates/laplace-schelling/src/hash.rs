//! Canonical state hash.
//!
//! Two runs agree when their hashes agree, so the hash must not depend on the platform or on
//! memory layout: every value is written in an explicit little-endian encoding, `usize` is
//! widened to `u64`, sequences carry their length, tables are written in id order, and floats
//! have no encoding at all. The bytes go through BLAKE3 in derive-key mode, so a state hash
//! never collides with a BLAKE3 hash of some other byte string.
//!
//! The world type implements [`HashState`] by naming its sections and handing each table to
//! the hasher:
//!
//! ```
//! use laplace_schelling::hash::{HashState, StateHasher};
//! use laplace_schelling::ids::Table;
//! use laplace_schelling::fixed::Fixed;
//!
//! struct Region;
//! struct World { price: Table<Region, Fixed>, people: Table<Region, i64> }
//!
//! impl HashState for World {
//!     fn hash_state(&self, h: &mut StateHasher) {
//!         h.section("price");
//!         self.price.hash_state(h);
//!         h.section("people");
//!         self.people.hash_state(h);
//!     }
//! }
//! ```

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::calendar::Date;
use crate::fixed::Fixed;
use crate::ids::{Id, Table};

/// BLAKE3 derive-key context; changing the canonical encoding means changing this string.
const CONTEXT: &str = "laplace-schelling 2026-11 canonical state hash v1";

/// Bytes buffered before they are handed to BLAKE3, which is fastest on large inputs.
const BUFFER: usize = 64 * 1024;

/// A 256-bit state hash. Displays as lowercase hex.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct StateHash(pub [u8; 32]);

impl fmt::Display for StateHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for StateHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StateHash({self})")
    }
}

/// Error parsing a [`StateHash`] from hex.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseHashError;

impl fmt::Display for ParseHashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a state hash is 64 hex digits")
    }
}

impl std::error::Error for ParseHashError {}

impl FromStr for StateHash {
    type Err = ParseHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 64 {
            return Err(ParseHashError);
        }
        let mut out = [0u8; 32];
        for (i, pair) in bytes.as_chunks::<2>().0.iter().enumerate() {
            let text = core::str::from_utf8(pair).map_err(|_| ParseHashError)?;
            out[i] = u8::from_str_radix(text, 16).map_err(|_| ParseHashError)?;
        }
        Ok(Self(out))
    }
}

impl Serialize for StateHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for StateHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let text = String::deserialize(deserializer)?;
            text.parse().map_err(serde::de::Error::custom)
        } else {
            <[u8; 32]>::deserialize(deserializer).map(Self)
        }
    }
}

/// Canonical byte sink feeding BLAKE3.
pub struct StateHasher {
    inner: blake3::Hasher,
    buffer: Vec<u8>,
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHasher {
    /// A fresh hasher.
    pub fn new() -> Self {
        Self {
            inner: blake3::Hasher::new_derive_key(CONTEXT),
            buffer: Vec::with_capacity(BUFFER),
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        if self.buffer.len() + bytes.len() > BUFFER {
            self.inner.update(&self.buffer);
            self.buffer.clear();
            if bytes.len() > BUFFER {
                self.inner.update(bytes);
                return;
            }
        }
        self.buffer.extend_from_slice(bytes);
    }

    /// Starts a named section. Sections make two different layouts that happen to share bytes
    /// hash differently, and make a divergence easy to locate by hashing sections apart.
    pub fn section(&mut self, name: &str) {
        self.write(&[0xff]);
        self.write_str(name);
    }

    /// Writes one byte.
    pub fn write_u8(&mut self, value: u8) {
        self.write(&[value]);
    }

    /// Writes a `u16`, little-endian.
    pub fn write_u16(&mut self, value: u16) {
        self.write(&value.to_le_bytes());
    }

    /// Writes a `u32`, little-endian.
    pub fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    /// Writes a `u64`, little-endian.
    pub fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    /// Writes an `i64`, little-endian two's complement.
    pub fn write_i64(&mut self, value: i64) {
        self.write(&value.to_le_bytes());
    }

    /// Writes a length (as `u64`, whatever the platform's `usize`).
    pub fn write_len(&mut self, len: usize) {
        self.write_u64(len as u64);
    }

    /// Writes length-prefixed bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_len(bytes.len());
        self.write(bytes);
    }

    /// Writes a length-prefixed UTF-8 string.
    pub fn write_str(&mut self, value: &str) {
        self.write_bytes(value.as_bytes());
    }

    /// Writes a slice of `i64` (length first), the common case for a column.
    pub fn write_i64_slice(&mut self, values: &[i64]) {
        values.hash_state(self);
    }

    /// The hash of everything written so far.
    pub fn finish(&self) -> StateHash {
        let mut inner = self.inner.clone();
        inner.update(&self.buffer);
        StateHash(*inner.finalize().as_bytes())
    }
}

/// A value with a canonical encoding for the state hash.
pub trait HashState {
    /// Writes the canonical encoding of `self`.
    fn hash_state(&self, hasher: &mut StateHasher);

    /// Writes every element of a slice (without its length). The default loops over
    /// [`hash_state`](HashState::hash_state); fixed-width numbers override it to encode the
    /// slice in bulk. The bytes must be the same either way.
    fn hash_slice(values: &[Self], hasher: &mut StateHasher)
    where
        Self: Sized,
    {
        for value in values {
            value.hash_state(hasher);
        }
    }
}

/// Encodes a slice of fixed-width values through a stack buffer, one `write` per 8 KiB.
fn write_le_slice<T, const N: usize>(
    values: &[T],
    hasher: &mut StateHasher,
    to_le: impl Fn(&T) -> [u8; N],
) {
    let mut buffer = [0u8; 8192];
    let per_chunk = buffer.len() / N;
    for chunk in values.chunks(per_chunk) {
        for (dst, value) in buffer.as_chunks_mut::<N>().0.iter_mut().zip(chunk) {
            *dst = to_le(value);
        }
        hasher.write(&buffer[..chunk.len() * N]);
    }
}

/// The state hash of one value.
pub fn hash_of<T: HashState + ?Sized>(value: &T) -> StateHash {
    let mut hasher = StateHasher::new();
    value.hash_state(&mut hasher);
    hasher.finish()
}

macro_rules! hash_le {
    ($($ty:ty),*) => {$(
        impl HashState for $ty {
            fn hash_state(&self, hasher: &mut StateHasher) {
                hasher.write(&self.to_le_bytes());
            }

            fn hash_slice(values: &[Self], hasher: &mut StateHasher) {
                write_le_slice(values, hasher, |v| v.to_le_bytes());
            }
        }
    )*};
}

hash_le!(u8, u16, u32, u64, i8, i16, i32, i64);

impl HashState for bool {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write_u8(u8::from(*self));
    }
}

impl HashState for str {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write_str(self);
    }
}

impl HashState for String {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write_str(self);
    }
}

impl HashState for Fixed {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write_i64(self.raw());
    }

    fn hash_slice(values: &[Self], hasher: &mut StateHasher) {
        write_le_slice(values, hasher, |v| v.raw().to_le_bytes());
    }
}

impl HashState for Date {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write(&self.year.to_le_bytes());
        hasher.write(&[self.month, self.day]);
    }
}

impl HashState for StateHash {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write(&self.0);
    }
}

impl<K> HashState for Id<K> {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write_u32(self.raw());
    }
}

impl<T: HashState> HashState for Option<T> {
    fn hash_state(&self, hasher: &mut StateHasher) {
        match self {
            None => hasher.write_u8(0),
            Some(value) => {
                hasher.write_u8(1);
                value.hash_state(hasher);
            }
        }
    }
}

impl<T: HashState> HashState for [T] {
    fn hash_state(&self, hasher: &mut StateHasher) {
        hasher.write_len(self.len());
        T::hash_slice(self, hasher);
    }
}

impl<T: HashState> HashState for Vec<T> {
    fn hash_state(&self, hasher: &mut StateHasher) {
        self.as_slice().hash_state(hasher);
    }
}

impl<A: HashState, B: HashState> HashState for (A, B) {
    fn hash_state(&self, hasher: &mut StateHasher) {
        self.0.hash_state(hasher);
        self.1.hash_state(hasher);
    }
}

/// Length, then every value in id order.
impl<K, V: HashState> HashState for Table<K, V> {
    fn hash_state(&self, hasher: &mut StateHasher) {
        self.as_slice().hash_state(hasher);
    }
}

#[cfg(test)]
mod tests {
    use super::{HashState, StateHash, StateHasher, hash_of};
    use crate::fixed::Fixed;
    use crate::ids::Table;

    struct Region;

    #[test]
    fn encoding_is_canonical_and_order_sensitive() {
        let a: Table<Region, Fixed> = Table::from_vec(vec![Fixed::ONE, Fixed::ZERO]);
        let b: Table<Region, Fixed> = Table::from_vec(vec![Fixed::ZERO, Fixed::ONE]);
        assert_eq!(hash_of(&a), hash_of(&a.clone()));
        assert_ne!(hash_of(&a), hash_of(&b));
        // Length prefixes keep ([1], [2, 3]) and ([1, 2], [3]) apart.
        assert_ne!(
            hash_of(&(vec![1u8], vec![2u8, 3])),
            hash_of(&(vec![1u8, 2], vec![3u8]))
        );
        let raw: Vec<i64> = a.as_slice().iter().map(|v| v.raw()).collect();
        let mut fast = StateHasher::new();
        fast.write_i64_slice(&raw);
        assert_eq!(fast.finish(), hash_of(&a));
    }

    #[test]
    fn buffering_does_not_change_the_hash() {
        let values: Vec<u64> = (0..100_000).collect();
        let whole = hash_of(&values);
        let mut by_parts = StateHasher::new();
        by_parts.write_len(values.len());
        for value in &values {
            value.hash_state(&mut by_parts);
        }
        assert_eq!(
            by_parts.finish(),
            whole,
            "bulk and per-element encodings agree"
        );
        let mut direct = blake3::Hasher::new_derive_key(super::CONTEXT);
        direct.update(&(values.len() as u64).to_le_bytes());
        for value in &values {
            direct.update(&value.to_le_bytes());
        }
        assert_eq!(StateHash(*direct.finalize().as_bytes()), whole);
    }

    #[test]
    fn hex_round_trip() {
        let hash = hash_of(&42u64);
        assert_eq!(hash.to_string().parse::<StateHash>(), Ok(hash));
        assert!("xyz".parse::<StateHash>().is_err());
    }
}
