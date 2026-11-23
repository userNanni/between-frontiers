//! Yearly series for the screen's charts of a country's evolution (`GameWorld::series`): what
//! the yearly record (`Economy::history`) does not keep. A record only: nothing in the
//! simulation reads it and the world's hash leaves it out, so a world runs the same with it.
//!
//! - By year and market ([`YearSeries`]): life expectancy, the public debt, the year's
//!   inflation, the treasury, the central bank's rate, and the metropole's own population, GDP,
//!   literacy, urban share and life expectancy (the yearly record mixes metropole and colonies).
//! - By year and market ([`YearPrices`]): the mean price over the year of each good, over its
//!   base price, in thousandths, over the weeks the good had a market there; zero for a good
//!   with none that year (not yet invented, a mineral the country neither mines nor buys).
//!   Dense by good: a list of (good, price) pairs came out larger in the save, since most goods
//!   have a market almost everywhere.
//!
//! The week adds each market's prices and the metropole's GDP to the running sums
//! ([`SeriesState::add_week`]); the first of January closes the year ([`close_year`]).

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use super::{Economy, Market};
use crate::demography::DemographyState;

/// One market's series for one year.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct YearSeries {
    pub year: i32,
    pub market: u16,
    /// Life expectancy at birth, years: the whole country and the metropole.
    pub life: Fixed,
    pub life_metro: Fixed,
    /// Public debt (`Money::total_debt`) and the treasury's cash on 1 January, local real
    /// units; the year's inflation (consumer prices against the last 1 January) and the central
    /// bank's rate on 1 January, percent.
    pub debt: Fixed,
    pub treasury: Fixed,
    pub inflation: Fixed,
    pub policy_rate: Fixed,
    /// The metropole alone (regions that are not colonies): population, GDP of the year at 1925
    /// base prices, literacy and urban share (0–1).
    pub population_metro: i64,
    pub gdp_metro: Fixed,
    pub literacy_metro: Fixed,
    pub urban_metro: Fixed,
}

/// One market's mean prices for one year.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct YearPrices {
    pub year: i32,
    pub market: u16,
    /// By good (catalogue order): the mean of the weekly prices over the base price, in
    /// thousandths (1000 = the base price), over the weeks the good had supply or demand in
    /// the market; zero for a good with no market there that year.
    pub per_mille: Vec<u32>,
}

/// `GameWorld::series`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesState {
    /// The year's running sums: each market's weekly prices over base, by good, and the weeks
    /// the good had a market there; the metropole's GDP; the weeks counted.
    pub price_sum: Vec<Vec<Fixed>>,
    pub price_weeks: Vec<Vec<u16>>,
    pub gdp_metro: Vec<Fixed>,
    pub weeks: u16,
    pub years: Vec<YearSeries>,
    pub prices: Vec<YearPrices>,
}

impl SeriesState {
    /// Adds one economy week: each market's prices over the base price, and the GDP each
    /// metropole made (`metro_gdp`, by market).
    pub fn add_week(&mut self, base: &[Fixed], markets: &[Market], metro_gdp: &[Fixed]) {
        let n = markets.len();
        if self.price_sum.len() < n {
            self.price_sum.resize(n, Vec::new());
            self.price_weeks.resize(n, Vec::new());
            self.gdp_metro.resize(n, Fixed::ZERO);
        }
        for (mi, m) in markets.iter().enumerate() {
            let (sum, weeks) = (&mut self.price_sum[mi], &mut self.price_weeks[mi]);
            if sum.len() < base.len() {
                sum.resize(base.len(), Fixed::ZERO);
                weeks.resize(base.len(), 0);
            }
            for (g, &b) in base.iter().enumerate() {
                let traded = m.supply.get(g).is_some_and(|x| *x > Fixed::ZERO)
                    || m.demand.get(g).is_some_and(|x| *x > Fixed::ZERO);
                if b > Fixed::ZERO && traded {
                    sum[g] += m.price[g] / b;
                    weeks[g] = weeks[g].saturating_add(1);
                }
            }
            self.gdp_metro[mi] += metro_gdp.get(mi).copied().unwrap_or(Fixed::ZERO);
        }
        self.weeks = self.weeks.saturating_add(1);
    }

    /// A year closed ([`close_year`]): its series are kept and the running sums start again.
    pub fn close(&mut self, closed: (Vec<YearSeries>, Vec<YearPrices>)) {
        self.years.extend(closed.0);
        self.prices.extend(closed.1);
        for s in &mut self.price_sum {
            s.iter_mut().for_each(|x| *x = Fixed::ZERO);
        }
        for s in &mut self.price_weeks {
            s.iter_mut().for_each(|x| *x = 0);
        }
        self.gdp_metro.iter_mut().for_each(|x| *x = Fixed::ZERO);
        self.weeks = 0;
    }
}

/// What the first of January writes for the year `year` that ended (computed from the world
/// before the year's records are cleared).
pub fn close_year(
    year: i32,
    series: &SeriesState,
    econ: &Economy,
    demography: &DemographyState,
) -> (Vec<YearSeries>, Vec<YearPrices>) {
    let mut years = Vec::new();
    let mut prices = Vec::new();
    for (mi, m) in econ.markets.iter().enumerate() {
        let market = mi as u16;
        let (mut people, mut people_metro) = (0i64, 0i64);
        let (mut lit, mut urb) = (Fixed::ZERO, Fixed::ZERO);
        for r in econ.regions.iter().filter(|r| r.market == market) {
            let p = r.population();
            people += p;
            if !r.colony {
                people_metro += p;
                lit += r.literacy * p;
                urb += r.urban_share() * p;
            }
        }
        if people == 0 {
            continue;
        }
        let rates = |colony: bool| {
            demography
                .territory_rates(econ, market, colony)
                .map(|v| (v.life, econ_people(econ, market, colony)))
        };
        let metro = rates(false);
        let colonies = rates(true);
        let life = match (metro, colonies) {
            (Some((a, na)), Some((b, nb))) if na + nb > 0 => (a * na + b * nb) / (na + nb),
            (Some((a, _)), _) | (None, Some((a, _))) => a,
            (None, None) => Fixed::ZERO,
        };
        let per = |x: Fixed| {
            if people_metro > 0 {
                x / people_metro
            } else {
                Fixed::ZERO
            }
        };
        years.push(YearSeries {
            year,
            market,
            life,
            life_metro: metro.map_or(Fixed::ZERO, |m| m.0),
            debt: m.money.total_debt(),
            treasury: m.treasury,
            inflation: if m.money.price_level_jan > Fixed::ZERO {
                (m.money.price_level / m.money.price_level_jan - Fixed::ONE) * 100
            } else {
                Fixed::ZERO
            },
            policy_rate: m.money.policy_rate,
            population_metro: people_metro,
            gdp_metro: series.gdp_metro.get(mi).copied().unwrap_or(Fixed::ZERO),
            literacy_metro: per(lit),
            urban_metro: per(urb),
        });
        let per_mille = match (series.price_sum.get(mi), series.price_weeks.get(mi)) {
            (Some(sum), Some(counted)) => sum
                .iter()
                .zip(counted)
                .map(|(x, w)| {
                    if *w == 0 {
                        0
                    } else {
                        u32::try_from(x.raw() / i64::from(*w)).unwrap_or(0)
                    }
                })
                .collect(),
            _ => Vec::new(),
        };
        prices.push(YearPrices {
            year,
            market,
            per_mille,
        });
    }
    (years, prices)
}

fn econ_people(econ: &Economy, market: u16, colony: bool) -> i64 {
    econ.regions
        .iter()
        .filter(|r| r.market == market && r.colony == colony)
        .map(super::Region::population)
        .sum()
}
