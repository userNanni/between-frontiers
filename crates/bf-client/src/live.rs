//! Country figures read from the simulation for the HUD and the country panel, once a month.
//!
//! Metropole and colonies stay apart, as on the map. Population, literacy and urbanization are
//! exact sums over the regions. GDP exists only per market (the colonies trade in the
//! metropole's market), so the market's figure is split between the two: by the scenario while
//! the 1925 calibration stands (it is the scenario's GDP), then by the income the pops earned
//! last week.

use bf_sim::game::GameWorld;
use laplace_schelling::calendar::Date;

/// Metropole or colonies of one country.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Part {
    pub population: f64,
    /// Dollars at the simulation's base prices, last full year.
    pub gdp: f64,
    /// 0–1, weighted by population; `None` without people.
    pub literacy: Option<f64>,
    pub urbanization: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveCountry {
    pub tag: String,
    pub metropole: Part,
    pub colonies: Part,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveFigures {
    /// Month of the game the figures were read in.
    pub month: (i32, u8),
    /// Year the GDP belongs to; `None` while the first year of the run is under way and the
    /// markets still carry the GDP they were calibrated to in 1925.
    pub gdp_year: Option<i32>,
    /// In the simulation's country order.
    pub countries: Vec<LiveCountry>,
    /// Each country's closed years (`econ.history`), oldest first, in the same order: the whole
    /// market, colonies included.
    pub history: Vec<Vec<YearPoint>>,
}

/// One closed year of a country, for the figures' charts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YearPoint {
    pub year: i32,
    pub population: f64,
    pub gdp: f64,
    pub literacy: f64,
    pub urban: f64,
}

#[derive(Default)]
struct Sums {
    population: i64,
    literate: f64,
    urban: f64,
    income: f64,
}

impl Sums {
    fn part(&self, gdp: f64) -> Part {
        let pop = self.population as f64;
        let share = |v: f64| (self.population > 0).then(|| v / pop);
        Part {
            population: pop,
            gdp,
            literacy: share(self.literate),
            urbanization: share(self.urban),
        }
    }
}

/// Reads the figures; `None` when the world runs without an economy.
pub fn read(world: &GameWorld, date: Date) -> Option<LiveFigures> {
    let econ = &world.econ;
    if econ.regions.is_empty() {
        return None;
    }
    let n = world.countries.len();
    let mut sums: Vec<[Sums; 2]> = (0..n).map(|_| Default::default()).collect();
    for r in &econ.regions {
        let Some(slot) = sums.get_mut(usize::from(r.market)) else {
            continue;
        };
        let s = &mut slot[usize::from(r.colony)];
        let pop = r.population();
        s.population += pop;
        s.literate += r.literacy.to_f64_lossy() * pop as f64;
        s.urban += r.urban_share().to_f64_lossy() * pop as f64;
        s.income += r
            .pops
            .iter()
            .map(|p| p.income.to_f64_lossy().max(0.0))
            .sum::<f64>();
    }
    let gdp_year = econ.history.iter().map(|h| h.year).max();
    let countries = world
        .countries
        .iter()
        .zip(sums)
        .enumerate()
        .map(|(i, ((_, c), [metro, colonies]))| {
            let gdp = econ
                .markets
                .get(i)
                .map_or(0.0, |m| m.gdp_last_year.to_f64_lossy());
            // The calibration year splits as the scenario does; later years by income, or by
            // population when nobody earned anything.
            let (a, b) = if gdp_year.is_none() {
                (
                    c.population as f64 * c.gdp_per_capita as f64,
                    c.colonial_population as f64 * c.colonial_gdp_per_capita as f64,
                )
            } else if metro.income + colonies.income > 0.0 {
                (metro.income, colonies.income)
            } else {
                (metro.population as f64, colonies.population as f64)
            };
            let metro_share = if a + b > 0.0 { a / (a + b) } else { 1.0 };
            LiveCountry {
                tag: c.tag.clone(),
                metropole: metro.part(gdp * metro_share),
                colonies: colonies.part(gdp * (1.0 - metro_share)),
            }
        })
        .collect();
    let mut history: Vec<Vec<YearPoint>> = vec![Vec::new(); n];
    for h in &econ.history {
        if let Some(v) = history.get_mut(usize::from(h.market)) {
            v.push(YearPoint {
                year: h.year,
                population: h.population as f64,
                gdp: h.gdp.to_f64_lossy(),
                literacy: h.literacy.to_f64_lossy(),
                urban: h.urban.to_f64_lossy(),
            });
        }
    }
    for v in &mut history {
        v.sort_by_key(|p| p.year);
    }
    Some(LiveFigures {
        month: (date.year, date.month),
        gdp_year,
        countries,
        history,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_sim::econ::state::{Market, Region, YearRecord, empty_pops};
    use bf_sim::scenario::{ColonialStats, CountryDef, Status};
    use laplace_schelling::Fixed;

    fn region(market: u16, colony: bool, people: i64, income: i64) -> Region {
        let mut pops = empty_pops();
        pops[0].size = people;
        pops[0].income = Fixed::from_int(income);
        Region {
            market,
            colony,
            pops,
            plants: Vec::new(),
            literacy: Fixed::from_ratio(1, 2),
            deposits: Vec::new(),
            land: Vec::new(),
            frontier: Vec::new(),
            wild_km2: [0; 2],
            cleared_km2: [0; 2],
            pioneer_km2: Fixed::ZERO,
            subsistence: Fixed::ZERO,
            tfp: Fixed::ONE,
            tfp_start: Fixed::ONE,
            state: 1,
            neighbours: Vec::new(),
            local: Vec::new(),
        }
    }

    fn world(colonial_population: u64, colonial_gdp_per_capita: u32) -> GameWorld {
        let def = CountryDef {
            tag: "AAA".into(),
            color: (0, 0, 0),
            status: Status::Sovereign,
            government: "presidential_republic".into(),
            capital_lon_lat: (0.0, 0.0),
            population: 300,
            gdp_per_capita: 1_000,
            source: String::new(),
            literacy: None,
            life_expectancy: None,
            urbanization: None,
            social_source: String::new(),
            colonies: Some(ColonialStats {
                population: colonial_population,
                gdp_per_capita: colonial_gdp_per_capita,
                literacy: None,
                life_expectancy: None,
                urbanization: None,
            }),
            flag: None,
        };
        let mut world = GameWorld::from_scenario("1925", [&def]);
        let mut market = Market::empty(0);
        market.gdp_last_year = Fixed::from_int(1_000);
        world.econ.markets.push(market);
        world.econ.regions = vec![region(0, false, 300, 30), region(0, true, 700, 10)];
        world
    }

    #[test]
    fn the_calibration_year_splits_gdp_as_the_scenario() {
        // Scenario: 300 × 1,000 in the metropole, 700 × 1,000 in the colonies.
        let f = read(&world(700, 1_000), Date::new(1925, 5, 1)).unwrap();
        assert_eq!(f.month, (1925, 5));
        assert_eq!(f.gdp_year, None);
        let c = &f.countries[0];
        assert_eq!(c.metropole.population, 300.0);
        assert_eq!(c.colonies.population, 700.0);
        assert!(
            (c.metropole.gdp - 300.0).abs() < 1e-6,
            "{}",
            c.metropole.gdp
        );
        assert!((c.colonies.gdp - 700.0).abs() < 1e-6);
        assert!((c.metropole.literacy.unwrap() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn later_years_split_gdp_by_income() {
        let mut w = world(700, 1_000);
        w.econ.history.push(YearRecord {
            year: 1925,
            market: 0,
            population: 1_000,
            gdp: Fixed::from_int(1_000),
            literacy: Fixed::ZERO,
            urban: Fixed::ZERO,
            basic_needs: Fixed::ZERO,
        });
        let f = read(&w, Date::new(1926, 2, 1)).unwrap();
        assert_eq!(f.gdp_year, Some(1925));
        // Income 30 against 10.
        let c = &f.countries[0];
        assert!(
            (c.metropole.gdp - 750.0).abs() < 1e-6,
            "{}",
            c.metropole.gdp
        );
        assert!((c.colonies.gdp - 250.0).abs() < 1e-6);
    }

    #[test]
    fn no_economy_means_no_live_figures() {
        let world = GameWorld::from_scenario("1925", []);
        assert!(read(&world, Date::new(1925, 1, 1)).is_none());
    }
}
