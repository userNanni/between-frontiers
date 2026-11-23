//! The density of the target (item 5 of the war's missing functions; the orchestrator's choice
//! (b) of 2026-11-19). Each home region's townsfolk stand for one city whose density falls from
//! the centre, `D(r) = D0 · e^(−r/r0)`; its people fix `r0 = √(U / 2πD0)`. The warheads that fall
//! on it add their lethal areas into one disc of radius `R`, and the dead are those inside it:
//! `U · [1 − (1 + x) · e^(−x)]`, with `x = R / r0 = √(2 · A · D0 / U)`. The first warhead on a
//! city kills at the density of its core; the next ones reach the suburbs, so the return falls
//! by the shape, with no target number.
//! - `D0`, the core density (`core_density`), is the density within 0.6 mile of ground zero in
//!   Hiroshima and Nagasaki (Glasstone and Dolan, *The Effects of Nuclear Weapons*, 1977, p. 544,
//!   table 12.09): Japanese cities of 1945, of wood and without warning, so a ceiling for other
//!   times; the spread by country is a gap.
//! - The lethal area of an equivalent megaton (`lethal_km2_per_emt`) is my computation from the
//!   same table: the dead of each ring over its density.
//! - The rural people are outside the target by construction (fallout reaches them through N8).
//!
//! All integer arithmetic (parts per billion), so the result is the same on every machine.

use crate::game::GameWorld;

/// Parts per billion: the scale of the shares here.
const PPB: u128 = 1_000_000_000;
/// ln 2 in parts per billion.
const LN2: u128 = 693_147_181;

/// Integer square root, rounded down.
fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = 1u128 << (n.ilog2() / 2 + 1);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// `e^(−x)`, `x` and the result in parts per billion.
fn exp_neg(x: u128) -> u128 {
    let k = x / LN2;
    if k >= 64 {
        return 0;
    }
    let f = x - k * LN2;
    // e^(−f) for f in [0, ln 2) by its series; the terms fall fast.
    let (mut term, mut sum, mut n) = (PPB as i128, PPB as i128, 1i128);
    while term.abs() > 0 && n < 30 {
        term = -term * f as i128 / (PPB as i128 * n);
        sum += term;
        n += 1;
    }
    (sum.max(0) as u128) >> k
}

/// Share of a city's `urban` people inside a lethal area of `area` (thousandths of a km²), with
/// core density `d0` (people a km²), in parts per billion: `1 − (1 + x) · e^(−x)`. Rises with the
/// area and never passes one.
pub fn share(urban: i64, d0: u32, area: u64) -> u128 {
    if urban <= 0 || area == 0 || d0 == 0 {
        return 0;
    }
    // x² = 2 · A · D0 / U, A in km² (area / 1000); x in parts per billion.
    let x2 = 2 * u128::from(area) * u128::from(d0) * PPB * PPB / 1000 / urban as u128;
    let x = isqrt(x2);
    let left = (PPB + x).saturating_mul(exp_neg(x)) / PPB;
    PPB.saturating_sub(left)
}

/// The dead of a city of `urban` people when the area that fell on it grows from `from` to `to`.
pub fn more_dead(urban: i64, d0: u32, from: u64, to: u64) -> i64 {
    let s = |a: u64| (urban.max(0) as u128 * share(urban, d0, a) / PPB) as i64;
    (s(to) - s(from)).max(0)
}

/// The target's cities: each home region with townsfolk, `(region, townsfolk)`, the most
/// populous first.
pub fn cities(world: &GameWorld, m: u16) -> Vec<(u32, i64)> {
    let mut out: Vec<(u32, i64)> = world
        .econ
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| r.market == m && !r.colony)
        .map(|(i, r)| {
            let pop = r.population();
            let urban = (i128::from(r.urban_share().raw()) * i128::from(pop)
                / i128::from(laplace_schelling::Fixed::SCALE)) as i64;
            (i as u32, urban.min(pop))
        })
        .filter(|(_, u)| *u > 0)
        .collect();
    out.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    out
}

/// Where each warhead falls and whom it kills: each, in order, on the city where it kills most,
/// its area added to what already fell there. `areas` in thousandths of a km². Returns, by
/// warhead, the index into `cities` and the dead (none once no city is left).
pub fn plan(cities: &[(u32, i64)], d0: u32, areas: &[u64]) -> Vec<(usize, i64)> {
    let mut fallen = vec![0u64; cities.len()];
    let mut out = Vec::with_capacity(areas.len());
    for &a in areas {
        let best = (0..cities.len())
            .map(|i| {
                let u = cities[i].1;
                (more_dead(u, d0, fallen[i], fallen[i] + a), i)
            })
            .max_by(|x, y| x.0.cmp(&y.0).then(y.1.cmp(&x.1)));
        let Some((dead, i)) = best else {
            break;
        };
        fallen[i] += a;
        out.push((i, dead));
    }
    out
}

/// Warheads of `area` each (thousandths of a km²) that kill `dead` among `cities`; `None` when
/// even `limit` of them do not.
pub fn warheads_to_kill(
    cities: &[(u32, i64)],
    d0: u32,
    area: u64,
    dead: i64,
    limit: u64,
) -> Option<u64> {
    if dead <= 0 {
        return Some(0);
    }
    if cities.iter().map(|c| c.1).sum::<i64>() < dead {
        return None;
    }
    let mut fallen = vec![0u64; cities.len()];
    let mut sum = 0i64;
    for k in 1..=limit {
        let (gain, i) = (0..cities.len())
            .map(|i| (more_dead(cities[i].1, d0, fallen[i], fallen[i] + area), i))
            .max_by(|x, y| x.0.cmp(&y.0).then(y.1.cmp(&x.1)))?;
        if gain == 0 {
            return None;
        }
        fallen[i] += area;
        sum += gain;
        if sum >= dead {
            return Some(k);
        }
    }
    None
}

/// The lethal area of a warhead of `emt_mille` equivalent megatons per mille, in thousandths of
/// a km².
pub fn area_of(km2_per_emt: u32, emt_mille: u64) -> u64 {
    u64::from(km2_per_emt) * emt_mille
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_and_root() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(99), 9);
        assert_eq!(isqrt(100), 10);
        assert_eq!(exp_neg(0), PPB);
        // e^-1 = 0.367879441
        assert!((exp_neg(PPB) as i128 - 367_879_441).abs() < 5);
        // e^-5 = 0.006737947
        assert!((exp_neg(5 * PPB) as i128 - 6_737_947).abs() < 5);
    }

    #[test]
    fn the_share_saturates() {
        let u = 1_000_000;
        let mut last = 0;
        for a in [
            1_000u64,
            10_000,
            100_000,
            1_000_000,
            10_000_000,
            100_000_000,
        ] {
            let s = share(u, 9_900, a);
            assert!(s >= last && s <= PPB, "{a}: {s}");
            last = s;
        }
        assert!(last > PPB * 999 / 1000);
    }

    #[test]
    fn many_warheads_on_one_city_never_kill_more_than_its_townsfolk() {
        let cities = [(0u32, 2_000_000i64)];
        let p = plan(&cities, 9_900, &[132_000; 500]);
        let dead: i64 = p.iter().map(|x| x.1).sum();
        assert!(dead <= 2_000_000, "{dead}");
        assert!(dead > 1_990_000, "{dead}");
        // Each next warhead kills no more than the one before (to the rounding of one person).
        assert!(p.windows(2).all(|w| w[1].1 <= w[0].1 + 1));
    }
}
