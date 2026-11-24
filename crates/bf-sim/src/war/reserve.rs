//! The trained reserve by cohort (item 9 of the war's missing functions; the orchestrator's
//! design of 2026-11-24), behind the switch `reserve_cohorts` (off). Each January the class that
//! ends its service joins the reserve with the training it had; each cohort loses training year
//! by year out of service and leaves the reserve after the years the law keeps it there.
//! Mobilization calls the youngest first, each with its own training. The men of the reserve
//! work in the economy until they are called, as before. The length of service is the country's
//! (the Armaments Year-Book of 1924 where it has a page), and the player may lengthen or shorten
//! it (`GameCommand::SetServiceMonths`).

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

/// `data/military/peace.ron`, `reserve_cohorts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveRule {
    /// Training a cohort loses a year out of service, per mille. DESIGN VALUE, without a source
    /// (the refresher courses of the 1920s laws point to a loss, not to its size).
    pub training_loss: u32,
    /// Years a cohort stays in the reserve where the country has no figure. DESIGN VALUE.
    pub reserve_years: u32,
    /// Months of service by the `conscription` effect in force where the country has no figure:
    /// (least effect, months), the highest reached applies. DESIGN VALUES.
    pub law_months: Vec<(i32, u16)>,
    /// The bounds of `SetServiceMonths`.
    pub min_months: u16,
    pub max_months: u16,
    /// (tag, months of service, years in the reserve; 0: `reserve_years`) with a source, in
    /// 1925.
    pub countries: Vec<(String, u16, u32)>,
}

impl ReserveRule {
    /// Months of service of `tag` in 1925, or by the law (`conscription` effect, per mille).
    pub fn months_1925(&self, tag: &str, conscription: i32) -> u16 {
        if let Some(&(_, m, _)) = self.countries.iter().find(|(t, _, _)| t == tag) {
            return m;
        }
        self.law_months
            .iter()
            .filter(|(least, _)| conscription >= *least)
            .map(|(_, m)| *m)
            .max()
            .unwrap_or(0)
    }

    /// Years a cohort of `tag` stays in the reserve.
    pub fn years(&self, tag: &str) -> u32 {
        self.countries
            .iter()
            .find(|(t, _, _)| t == tag)
            .map(|(_, _, y)| *y)
            .filter(|y| *y > 0)
            .unwrap_or(self.reserve_years)
            .max(1)
    }
}

/// The men a year's class left in the reserve, and their training now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveCohort {
    /// The year it left service.
    pub year: i32,
    pub men: u64,
    /// 0 to 1.
    pub training: Fixed,
}

/// A country's reserve by cohort, youngest first, and its length of service.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryReserve {
    pub country: u16,
    /// The length of service by law, months; 0, no conscription.
    pub service_months: u16,
    /// The months that size the army in peace now: a longer service reaches it a year at a
    /// time, as the classes it keeps stay on (12 months a year); a shorter one at once.
    pub serving_months: u16,
    pub cohorts: Vec<ReserveCohort>,
}

impl CountryReserve {
    pub fn men(&self) -> u64 {
        self.cohorts.iter().map(|c| c.men).sum()
    }
}

/// The reserves by cohort, by country; empty with the switch off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveState {
    pub countries: Vec<CountryReserve>,
}

impl ReserveState {
    pub fn is_empty(&self) -> bool {
        self.countries.is_empty()
    }

    pub fn of(&self, country: u16) -> Option<&CountryReserve> {
        self.countries
            .binary_search_by_key(&country, |c| c.country)
            .ok()
            .map(|i| &self.countries[i])
    }
}

impl HashState for ReserveState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.countries.len());
        for c in &self.countries {
            h.write_u16(c.country);
            h.write_u16(c.service_months);
            h.write_u16(c.serving_months);
            h.write_len(c.cohorts.len());
            for k in &c.cohorts {
                h.write_i64(i64::from(k.year));
                h.write_u64(k.men);
                k.training.hash_state(h);
            }
        }
    }
}

/// The 1925 reserve of `men` split into `years` cohorts that left service in the years before
/// `year`, the same number each (the remainder to the youngest), the training of each
/// `training` less `loss` per mille a year out. Sums to `men`.
pub fn initial(men: u64, years: u32, year: i32, training: Fixed, loss: u32) -> Vec<ReserveCohort> {
    let years = u64::from(years.max(1));
    let each = men / years;
    let rest = men - each * years;
    (0..years)
        .map(|k| ReserveCohort {
            year: year - 1 - k as i32,
            men: each + if k == 0 { rest } else { 0 },
            training: decayed(training, loss, k as u32 + 1),
        })
        .filter(|c| c.men > 0)
        .collect()
}

/// `training` after `years` out of service at `loss` per mille a year.
pub fn decayed(training: Fixed, loss: u32, years: u32) -> Fixed {
    let mut t = training;
    for _ in 0..years {
        t = t.mul_ratio(1000 - i64::from(loss.min(1000)), 1000);
    }
    t
}

/// A year passes for a country's reserve (January): the cohorts lose training, those older than
/// `years` leave, and `class` men join with `training`, as the cohort of `year - 1`. Returns
/// the men who left by age.
pub fn year_passes(
    r: &mut CountryReserve,
    rule: &ReserveRule,
    years: u32,
    year: i32,
    class: u64,
    training: Fixed,
) -> u64 {
    for c in &mut r.cohorts {
        c.training = decayed(c.training, rule.training_loss, 1);
    }
    let limit = year - 1 - years as i32;
    let aged: u64 = r
        .cohorts
        .iter()
        .filter(|c| c.year <= limit)
        .map(|c| c.men)
        .sum();
    r.cohorts.retain(|c| c.year > limit);
    if class > 0 {
        r.cohorts.insert(
            0,
            ReserveCohort {
                year: year - 1,
                men: class,
                training,
            },
        );
    }
    aged
}

/// Calls up to `n` men, youngest cohorts first: (men called, their mean training).
pub fn call(r: &mut CountryReserve, n: u64) -> (u64, Fixed) {
    let (mut left, mut got, mut weighted) = (n, 0u64, Fixed::ZERO);
    for c in &mut r.cohorts {
        if left == 0 {
            break;
        }
        let take = c.men.min(left);
        c.men -= take;
        left -= take;
        got += take;
        weighted += c.training * take as i64;
    }
    r.cohorts.retain(|c| c.men > 0);
    let training = if got > 0 {
        weighted / Fixed::from_int(got as i64)
    } else {
        Fixed::ZERO
    };
    (got, training)
}

/// January: the months in effect move toward the law's, 12 a year up, at once down. Returns
/// the months they grew this year (the share of a class kept on).
pub fn serving_moves(r: &mut CountryReserve) -> u16 {
    let before = r.serving_months;
    r.serving_months = if r.service_months > before {
        r.service_months.min(before.saturating_add(12))
    } else {
        r.service_months
    };
    r.serving_months.saturating_sub(before)
}

/// Men coming back from service (demobilized) join the youngest cohort, or a new one of
/// `year - 1`, at `training`.
pub fn back(r: &mut CountryReserve, men: u64, year: i32, training: Fixed) {
    if men == 0 {
        return;
    }
    match r.cohorts.first_mut() {
        Some(c) if c.year == year - 1 => {
            let all = c.men + men;
            c.training =
                (c.training * c.men as i64 + training * men as i64) / Fixed::from_int(all as i64);
            c.men = all;
        }
        _ => r.cohorts.insert(
            0,
            ReserveCohort {
                year: year - 1,
                men,
                training,
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> ReserveRule {
        ReserveRule {
            training_loss: 50,
            reserve_years: 15,
            law_months: vec![(0, 0), (300, 12), (800, 24)],
            min_months: 6,
            max_months: 36,
            countries: vec![("FRA".into(), 18, 28)],
        }
    }

    #[test]
    fn the_1925_reserve_splits_into_cohorts_that_sum_to_it() {
        let c = initial(1_000_003, 28, 1925, Fixed::ONE, 50);
        assert_eq!(c.iter().map(|x| x.men).sum::<u64>(), 1_000_003);
        assert_eq!(c.len(), 28);
        assert_eq!(c[0].year, 1924);
        assert!(c.windows(2).all(|w| w[0].training >= w[1].training));
    }

    #[test]
    fn men_in_and_out_balance() {
        let r = rule();
        let mut c = CountryReserve {
            country: 0,
            service_months: 18,
            serving_months: 18,
            cohorts: initial(600_000, 3, 1925, Fixed::ONE, 50),
        };
        let mut total = c.men();
        for year in 1926..1940 {
            let class = 100_000;
            let aged = year_passes(&mut c, &r, 3, year, class, Fixed::ONE);
            total = total + class - aged;
            assert_eq!(c.men(), total, "{year}");
            let (got, t) = call(&mut c, 50_000);
            total -= got;
            assert!(t <= Fixed::ONE);
            back(&mut c, got, year, t);
            total += got;
            assert_eq!(c.men(), total, "{year} after the call and the return");
        }
        assert_eq!(r.months_1925("FRA", 0), 18);
        assert_eq!(r.months_1925("XXX", 800), 24);
        assert_eq!(r.months_1925("XXX", 20), 0);
        assert_eq!(r.years("XXX"), 15);
        c.service_months = 36;
        assert_eq!((serving_moves(&mut c), c.serving_months), (12, 30));
        assert_eq!((serving_moves(&mut c), c.serving_months), (6, 36));
        c.service_months = 12;
        assert_eq!((serving_moves(&mut c), c.serving_months), (0, 12));
    }
}
