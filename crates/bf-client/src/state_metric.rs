//! The metrics of a state the map can colour and the states' table can sort by, read from the
//! monthly `state_view` (and the state's area, from the map): people, density, household income
//! per person, literacy, towns, the share at work, and works under way.

use bf_i18n::I18n;

use crate::state_view::StateFigures;

/// Keys (`lens-state-<key>`, `states-col-<key>`), in the order the picker lists them.
pub const METRICS: [&str; 7] = [
    "population",
    "density",
    "income",
    "literacy",
    "urban",
    "employed",
    "works",
];

/// How a metric spreads: by orders of magnitude (people, density, income), as a share 0–1, or
/// as a count from zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Log,
    Share,
    Count,
}

pub fn scale(metric: usize) -> Scale {
    match metric {
        0..=2 => Scale::Log,
        3..=5 => Scale::Share,
        _ => Scale::Count,
    }
}

/// The metric's value for one state; `None` where the simulation does not run it or has
/// nothing to measure (no people).
pub fn value(f: &StateFigures, area_km2: f64, metric: usize) -> Option<f64> {
    if !f.known {
        return None;
    }
    match metric {
        0 => (f.population > 0).then_some(f.population as f64),
        1 => (area_km2 > 0.0 && f.population > 0).then(|| f.population as f64 / area_km2),
        2 => f.income_per_head(),
        3 => (f.population > 0).then_some(f.literacy),
        4 => (f.population > 0).then_some(f.urban),
        5 => f.employed_share(),
        _ => Some((f.works[0] + f.works[1]) as f64),
    }
}

/// The metric's value in words, with its unit.
pub fn text(i18n: &I18n, metric: usize, v: f64) -> String {
    match metric {
        0 => i18n.compact(v),
        1 => i18n.tr_args(
            "states-density",
            &[("n", bf_i18n::Arg::String(i18n.decimal(v, 1)))],
        ),
        2 => crate::text::money(i18n, v),
        3..=5 => crate::screens::budget::pct(i18n, v * 100.0, 0),
        _ => i18n.int(v as i64),
    }
}

/// Where `v` falls between `lo` and `hi` on the metric's scale, 0–1.
pub fn position(metric: usize, v: f64, lo: f64, hi: f64) -> f32 {
    let t = match scale(metric) {
        Scale::Log => {
            let (a, b, x) = (lo.max(1e-9).ln(), hi.max(1e-9).ln(), v.max(1e-9).ln());
            if b > a { (x - a) / (b - a) } else { 1.0 }
        }
        Scale::Share | Scale::Count => {
            if hi > lo {
                (v - lo) / (hi - lo)
            } else {
                1.0
            }
        }
    };
    t.clamp(0.0, 1.0) as f32
}

/// The range the lens spreads over: the states' lowest and highest (shares from 0 to the
/// highest, counts from zero).
pub fn range(metric: usize, values: impl Iterator<Item = f64>) -> Option<(f64, f64)> {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let log = scale(metric) == Scale::Log;
    for v in values {
        // On a log scale the floor is the lowest state with something: nothing has no logarithm.
        if !log || v > 0.0 {
            lo = lo.min(v);
        }
        hi = hi.max(v);
    }
    if !lo.is_finite() {
        return None;
    }
    Some(match scale(metric) {
        Scale::Log => (lo, hi),
        Scale::Share | Scale::Count => (0.0, hi.max(1e-9)),
    })
}
