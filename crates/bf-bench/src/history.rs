//! `--history-out`: yearly series of every country, metropole and colonies apart, for the
//! comparison with history (`tools/history/`, `docs/history.pt-BR.md`).
//!
//! Read only: it looks at the world after each tick and never touches it, so the state hash
//! is the same with and without it. Flows (GDP, production, taxes) are summed over the calendar
//! year; stocks (population, literacy, sector employment) are the mean of 1 January of the year
//! and of the next, close to the mid-year figures the historical series give.

use bf_sim::data::GameData;
use bf_sim::demography::TerritoryId;
use bf_sim::econ::{Market, Region, week, workers};
use bf_sim::economy::BUILDING_GROUPS;
use bf_sim::game::{GameWorld, Session};
use laplace_schelling::{Fixed, Weekday};
use serde::Serialize;

/// Goods with a usable historical series, and the catalogue keys summed into each.
const GOODS: &[(&str, &[&str])] = &[
    ("steel", &["steel"]),
    ("coal", &["coal"]),
    ("crude_oil", &["crude_oil"]),
    ("natural_gas", &["natural_gas"]),
    ("electricity", &["electricity"]),
    ("cereals", &["grain", "maize", "rice"]),
    ("cement", &["cement"]),
    ("automobiles", &["automobiles"]),
];

/// Sectors of employment: farms; mines, energy and industry; everything else, the state's
/// payroll included.
const SECTORS: [&str; 3] = ["agriculture", "industry", "services"];

fn sector_of(group: &str) -> usize {
    match group {
        "agriculture" => 0,
        "extraction" | "heavy_industry" | "light_industry" | "energy" | "military_industry" => 1,
        _ => 2,
    }
}

/// Series of one territory: a tag's metropole or its colonies.
#[derive(Serialize, Default)]
pub struct Territory {
    pub tag: String,
    /// `metropole` or `colonies`.
    pub part: &'static str,
    pub population: Vec<i64>,
    /// GDP at 1925 base prices (2011 dollars), summed over the calendar year. The country's GDP
    /// is the simulation's; the split
    /// between metropole and colonies follows the value added estimated here week by week.
    pub gdp: Vec<f64>,
    pub gdp_per_capita: Vec<f64>,
    /// Share of the population outside farming (the simulation's `urban_share`).
    pub non_farm: Vec<f64>,
    pub literacy: Vec<f64>,
    /// Workers by sector (`SECTORS`), in jobs filled; agriculture counts the jobless who live off
    /// their own plot.
    pub employment: Vec<[f64; 3]>,
    /// Informal workers, as the ILO counts them: urban self-employment
    /// (`BuildingDef::informal`) and the smallholders who work their own plot.
    pub informal: Vec<f64>,
    /// Pasture and forest turned into cropland since 1925, km², on 1 January.
    pub cleared_km2: Vec<[f64; 2]>,
    /// Yearly production by good (`GOODS`), in the catalogue's units.
    pub production: Vec<Vec<f64>>,
    /// Births, deaths and net migration per thousand people, life expectancy at birth and
    /// children per woman: the year's monthly rates, weighted by population.
    pub births: Vec<f64>,
    pub deaths: Vec<f64>,
    pub migration: Vec<f64>,
    pub life_expectancy: Vec<f64>,
    pub fertility: Vec<f64>,
    /// Values of `demography::FERTILITY_TERMS` (in that order), weighted by population.
    pub fertility_terms: Vec<Vec<f64>>,
    /// By profession (`econ::prof`): share of the people (mean of the months) and children per
    /// woman.
    pub class_share: Vec<Vec<f64>>,
    pub class_fertility: Vec<Vec<f64>>,
}

/// Public accounts and prices of one country (metropole and colonies together).
#[derive(Serialize, Default)]
pub struct Country {
    pub tag: String,
    /// Taxes, spending (payroll, goods, services, works, interest) and new central-bank money
    /// lent to the treasury over the year; debt at the year's end. Shares of GDP.
    pub taxes: Vec<f64>,
    pub spending: Vec<f64>,
    pub debt: Vec<f64>,
    /// Consumer price inflation over the year, percent.
    pub inflation: Vec<f64>,
}

#[derive(Serialize)]
pub struct HistoryReport {
    pub first_year: i32,
    pub years: Vec<i32>,
    pub goods: Vec<&'static str>,
    pub sectors: [&'static str; 3],
    pub territories: Vec<Territory>,
    pub countries: Vec<Country>,
    /// Population and GDP of every state of the map, for aggregating by modern country.
    pub states: Vec<StateSeries>,
    /// Arrivals by year, origin and destination (`TAG/metropole`, `TAG/colonies`), for every
    /// road that carried 1,000 people or more over the run.
    pub migration_pairs: Vec<PairSeries>,
    /// Mass migrations: ended and still open at the last year.
    pub waves: Vec<WaveSeries>,
}

#[derive(Serialize, Default)]
pub struct PairSeries {
    pub origin: String,
    pub dest: String,
    pub arrivals: Vec<i64>,
}

#[derive(Serialize)]
pub struct WaveSeries {
    pub origin: String,
    pub dest: String,
    pub from: [i32; 2],
    /// `None` while open.
    pub until: Option<[i32; 2]>,
    pub cause: String,
    pub end: Option<String>,
    pub people: i64,
}

#[derive(Serialize, Default)]
pub struct StateSeries {
    /// `StateDef::key`, whose first three letters are the Natural Earth `adm0_a3`.
    pub key: String,
    pub tag: String,
    pub colony: bool,
    pub population: Vec<i64>,
    pub gdp: Vec<f64>,
    /// Births, deaths and net migration per thousand, life expectancy and children per woman:
    /// the year's monthly rates weighted by population.
    pub births: Vec<f64>,
    pub deaths: Vec<f64>,
    pub migration: Vec<f64>,
    pub life_expectancy: Vec<f64>,
    pub fertility: Vec<f64>,
}

/// Stocks seen on 1 January, by territory.
#[derive(Clone, Default)]
struct Stocks {
    population: f64,
    literate: f64,
    non_farm: f64,
    employment: [f64; 3],
    informal: f64,
    cleared: [f64; 2],
}

/// The year's running sums, by territory (`market * 2 + colony`) and by market.
pub struct History {
    goods: Vec<Vec<usize>>,
    year: i32,
    jan: Vec<Stocks>,
    /// Population of every region on 1 January, and its value added summed over the year.
    region_jan: Vec<f64>,
    value_added: Vec<f64>,
    production: Vec<Vec<f64>>,
    revenue: Vec<f64>,
    spending: Vec<f64>,
    price_jan: Vec<(f64, u16)>,
    /// By territory: people summed over the months, and births, deaths, migration, life
    /// expectancy and fertility times people.
    vital: Vec<[f64; 6]>,
    /// By territory: the fertility terms times people.
    terms: Vec<[f64; 5]>,
    /// By territory and profession: people summed over the months, and children per woman
    /// times people.
    class: Vec<[[f64; 2]; bf_sim::econ::prof::P]>,
    /// The same by region.
    region_vital: Vec<[f64; 6]>,
    /// Arrivals since the start by (origin, destination) at the last 1 January, and their
    /// yearly series.
    arrived: std::collections::BTreeMap<(TerritoryId, TerritoryId), i64>,
    pairs: std::collections::BTreeMap<(TerritoryId, TerritoryId), Vec<i64>>,
    report: HistoryReport,
}

impl History {
    pub fn new(data: &GameData, s: &Session) -> Self {
        let cat = &data.catalogue;
        let goods = GOODS
            .iter()
            .map(|(_, keys)| {
                keys.iter()
                    .filter_map(|k| cat.good(k).map(usize::from))
                    .collect()
            })
            .collect();
        let w = s.world();
        let markets = w.econ.markets.len();
        let tags: Vec<String> = w.countries.iter().map(|(_, c)| c.tag.clone()).collect();
        let mut territories = Vec::with_capacity(TerritoryId::count(markets));
        for tag in &tags {
            for part in ["metropole", "colonies"] {
                territories.push(Territory {
                    tag: tag.clone(),
                    part,
                    ..Territory::default()
                });
            }
        }
        let year = s.clock().date.year;
        let key_of = |id: u16| {
            data.map
                .states
                .iter()
                .find(|d| d.id == id)
                .map_or_else(String::new, |d| d.key.clone())
        };
        let states = w
            .econ
            .regions
            .iter()
            .map(|r| StateSeries {
                key: key_of(r.state),
                tag: tags[r.market as usize].clone(),
                colony: r.colony,
                ..StateSeries::default()
            })
            .collect();
        Self {
            goods,
            year,
            jan: stocks(data, w),
            region_jan: w
                .econ
                .regions
                .iter()
                .map(|r| r.population() as f64)
                .collect(),
            value_added: vec![0.0; w.econ.regions.len()],
            production: vec![vec![0.0; GOODS.len()]; TerritoryId::count(markets)],
            revenue: vec![0.0; markets],
            spending: vec![0.0; markets],
            price_jan: prices(w),
            vital: vec![[0.0; 6]; TerritoryId::count(markets)],
            terms: vec![[0.0; 5]; TerritoryId::count(markets)],
            class: vec![[[0.0; 2]; bf_sim::econ::prof::P]; TerritoryId::count(markets)],
            region_vital: vec![[0.0; 6]; w.econ.regions.len()],
            arrived: arrived(w),
            pairs: Default::default(),
            report: HistoryReport {
                first_year: year,
                years: Vec::new(),
                goods: GOODS.iter().map(|(k, _)| *k).collect(),
                sectors: SECTORS,
                territories,
                states,
                migration_pairs: Vec::new(),
                waves: Vec::new(),
                countries: tags
                    .into_iter()
                    .map(|tag| Country {
                        tag,
                        ..Country::default()
                    })
                    .collect(),
            },
        }
    }

    /// Call after every tick.
    pub fn tick(&mut self, data: &GameData, s: &Session) {
        let date = s.clock().date;
        // The weekly market runs on Mondays: on Tuesday the world holds that week's outcome.
        if date.weekday() == Weekday::Tuesday {
            self.week(data, s.world());
        }
        // The demography runs on the 15th: on the 16th the world holds the month's rates.
        if date.day == 16 {
            self.month(s.world());
        }
        if date.month == 1 && date.day == 1 && date.year > self.year {
            self.close_year(data, s.world());
            self.year = date.year;
        }
    }

    pub fn finish(self) -> HistoryReport {
        self.report
    }

    fn week(&mut self, data: &GameData, w: &GameWorld) {
        let cat = &data.catalogue;
        let grain = cat.good("grain").map(usize::from);
        for (ri, r) in w.econ.regions.iter().enumerate() {
            let mi = r.market as usize;
            let m = &w.econ.markets[mi];
            let t = TerritoryId::of(r).index();
            let mut out = vec![0.0f64; cat.goods.len()];
            let mut va = 0.0;
            for p in &r.plants {
                let b = &cat.buildings[p.kind as usize];
                let method = &b.methods[p.method as usize];
                let scale = (Fixed::from_int(p.levels)
                    * p.staffing
                    * bf_sim::econ::plant_tfp(r, m, b)
                    * p.utilization)
                    .to_f64_lossy();
                let out_scale = scale * p.input_ratio.to_f64_lossy();
                for (gi, q) in &method.outputs {
                    let gi = *gi as usize;
                    let units = q.to_f64_lossy() * out_scale;
                    out[gi] += units;
                    va += units
                        * m.sell_ratio[gi].to_f64_lossy()
                        * cat.goods[gi].base_price.to_f64_lossy();
                }
                for (gi, q) in &method.inputs {
                    let gi = *gi as usize;
                    va -= q.to_f64_lossy()
                        * scale
                        * m.buy_ratio[gi].to_f64_lossy()
                        * cat.goods[gi].base_price.to_f64_lossy();
                }
            }
            if let Some(g) = grain {
                let own = week::subsistence_food(&data.catalogue, r, m);
                let grown = (own + week::peasant_surplus(m, own)).to_f64_lossy();
                out[g] += grown;
                va += grown * cat.goods[g].base_price.to_f64_lossy();
            }
            self.value_added[ri] += va;
            for (k, keys) in self.goods.iter().enumerate() {
                self.production[t][k] += keys.iter().map(|g| out[*g]).sum::<f64>();
            }
        }
        for (mi, m) in w.econ.markets.iter().enumerate() {
            let b = &m.budget;
            self.revenue[mi] += b.revenue.to_f64_lossy();
            self.spending[mi] +=
                (b.payroll + b.goods + b.services + b.works + b.interest).to_f64_lossy();
        }
    }

    fn month(&mut self, w: &GameWorld) {
        for (ri, (r, v)) in w.econ.regions.iter().zip(&w.demography.vital).enumerate() {
            let t = TerritoryId::of(r).index();
            let n = r.population() as f64;
            for (k, x) in v.fertility_terms.iter().enumerate() {
                self.terms[t][k] += x.to_f64_lossy() * n;
            }
            for (acc, (p, x)) in self.class[t].iter_mut().zip(r.pops.iter().zip(v.tfr_by_class)) {
                let n = p.size as f64;
                acc[0] += n;
                acc[1] += x.to_f64_lossy() * n;
            }
            for acc in [&mut self.vital[t], &mut self.region_vital[ri]] {
                acc[0] += n;
                for (k, x) in [v.births, v.deaths, v.migration, v.life, v.tfr]
                    .into_iter()
                    .enumerate()
                {
                    acc[k + 1] += x.to_f64_lossy() * n;
                }
            }
        }
    }

    fn close_year(&mut self, data: &GameData, w: &GameWorld) {
        let dec = stocks(data, w);
        let now = prices(w);
        let rep = &mut self.report;
        rep.years.push(self.year);
        // GDP of the year just ended: `gdp_running`, read before the yearly system of 1 January
        // moves it to `gdp_last_year` (which at this point still holds the year before).
        // The country's GDP is shared among its states by the value added estimated here, or by
        // population where none was positive.
        let regions = &w.econ.regions;
        let nm = w.econ.markets.len();
        let (mut va_sum, mut pop_sum) = (vec![0.0f64; nm], vec![0.0f64; nm]);
        let pops: Vec<f64> = regions
            .iter()
            .zip(&self.region_jan)
            .map(|(r, a)| (a + r.population() as f64) / 2.0)
            .collect();
        for (ri, r) in regions.iter().enumerate() {
            va_sum[r.market as usize] += self.value_added[ri].max(0.0);
            pop_sum[r.market as usize] += pops[ri];
        }
        let mut territory_gdp = vec![0.0f64; TerritoryId::count(nm)];
        for (ri, r) in regions.iter().enumerate() {
            let mi = r.market as usize;
            let gdp = w.econ.markets[mi].gdp_running.to_f64_lossy();
            let share = if va_sum[mi] > 0.0 {
                self.value_added[ri].max(0.0) / va_sum[mi]
            } else if pop_sum[mi] > 0.0 {
                pops[ri] / pop_sum[mi]
            } else {
                0.0
            };
            territory_gdp[TerritoryId::of(r).index()] += gdp * share;
            let st = &mut rep.states[ri];
            st.population.push(pops[ri].round() as i64);
            st.gdp.push(gdp * share);
            let v = &self.region_vital[ri];
            let mean = |k: usize| if v[0] > 0.0 { v[k] / v[0] } else { 0.0 };
            st.births.push(mean(1));
            st.deaths.push(mean(2));
            st.migration.push(mean(3));
            st.life_expectancy.push(mean(4));
            st.fertility.push(mean(5));
        }
        for (mi, m) in w.econ.markets.iter().enumerate() {
            let gdp = m.gdp_running.to_f64_lossy();
            for part in 0..2 {
                let t = TerritoryId::new(mi as u16, part == 1).index();
                let (a, b) = (&self.jan[t], &dec[t]);
                let pop = (a.population + b.population) / 2.0;
                let g = territory_gdp[t];
                let per_head = |x: f64, y: f64| {
                    let n = a.population + b.population;
                    if n > 0.0 { (x + y) / n } else { 0.0 }
                };
                let ter = &mut rep.territories[t];
                ter.population.push(pop.round() as i64);
                ter.gdp.push(g);
                ter.gdp_per_capita
                    .push(if pop > 0.0 { g / pop } else { 0.0 });
                ter.non_farm.push(per_head(a.non_farm, b.non_farm));
                ter.literacy.push(per_head(a.literate, b.literate));
                ter.employment.push(std::array::from_fn(|k| {
                    (a.employment[k] + b.employment[k]) / 2.0
                }));
                ter.informal.push((a.informal + b.informal) / 2.0);
                ter.cleared_km2.push(a.cleared);
                ter.production.push(self.production[t].clone());
                let v = &self.vital[t];
                let mean = |k: usize| if v[0] > 0.0 { v[k] / v[0] } else { 0.0 };
                ter.births.push(mean(1));
                ter.deaths.push(mean(2));
                ter.migration.push(mean(3));
                ter.life_expectancy.push(mean(4));
                ter.fertility.push(mean(5));
                let tm = &self.terms[t];
                ter.fertility_terms.push(
                    tm.iter()
                        .map(|x| if v[0] > 0.0 { x / v[0] } else { 0.0 })
                        .collect(),
                );
                ter.class_share.push(
                    self.class[t]
                        .iter()
                        .map(|c| if v[0] > 0.0 { c[0] / v[0] } else { 0.0 })
                        .collect(),
                );
                ter.class_fertility.push(
                    self.class[t]
                        .iter()
                        .map(|c| if c[0] > 0.0 { c[1] / c[0] } else { 0.0 })
                        .collect(),
                );
            }
            let c = &mut rep.countries[mi];
            let g = gdp.max(1.0);
            c.taxes.push(self.revenue[mi] / g);
            c.spending.push(self.spending[mi] / g);
            c.debt.push(m.money.total_debt().to_f64_lossy() / g);
            let (p0, r0) = self.price_jan[mi];
            let (p1, r1) = now[mi];
            let cut = 1000f64.powi(i32::from(r1) - i32::from(r0));
            c.inflation.push((p1 * cut / p0.max(1e-12) - 1.0) * 100.0);
        }
        self.jan = dec;
        self.region_jan = regions.iter().map(|r| r.population() as f64).collect();
        self.price_jan = now;
        self.value_added.iter_mut().for_each(|x| *x = 0.0);
        self.production
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|x| *x = 0.0));
        self.revenue.iter_mut().for_each(|x| *x = 0.0);
        self.spending.iter_mut().for_each(|x| *x = 0.0);
        self.vital.iter_mut().for_each(|v| *v = [0.0; 6]);
        self.terms.iter_mut().for_each(|v| *v = [0.0; 5]);
        self.class
            .iter_mut()
            .for_each(|v| *v = [[0.0; 2]; bf_sim::econ::prof::P]);
        // Arrivals of the year by road; the series and the waves are rewritten every year.
        let now = arrived(w);
        let n = self.report.years.len();
        for (&k, &v) in &now {
            let year = v - self.arrived.get(&k).copied().unwrap_or(0);
            let row = self.pairs.entry(k).or_insert_with(|| vec![0; n - 1]);
            row.push(year);
        }
        for row in self.pairs.values_mut() {
            row.resize(n, 0);
        }
        self.arrived = now;
        let names = territory_names(w);
        let name = |t: TerritoryId| territory_name(&names, t);
        self.report.migration_pairs = self
            .pairs
            .iter()
            .filter(|(_, v)| v.iter().sum::<i64>() >= 1_000)
            .map(|(&(o, d), v)| PairSeries {
                origin: name(o),
                dest: name(d),
                arrivals: v.clone(),
            })
            .collect();
        let month = |m: i32| [m.div_euclid(12), m.rem_euclid(12) + 1];
        let dem = &w.demography;
        self.report.waves = dem
            .waves_ended
            .iter()
            .map(|r| (&r.wave, Some((r.until, format!("{:?}", r.end)))))
            .chain(dem.waves.iter().map(|w| (w, None)))
            .map(|(wave, end)| WaveSeries {
                origin: name(wave.origin),
                dest: name(wave.dest),
                from: month(wave.since),
                until: end.as_ref().map(|(u, _)| month(*u)),
                cause: format!("{:?}", wave.cause),
                end: end.map(|(_, e)| e),
                people: wave.people,
            })
            .collect();
        self.region_vital.iter_mut().for_each(|v| *v = [0.0; 6]);
    }
}

/// Arrivals since the start by (origin, destination), all cultures together.
fn arrived(w: &GameWorld) -> std::collections::BTreeMap<(TerritoryId, TerritoryId), i64> {
    let mut out = std::collections::BTreeMap::new();
    for d in &w.demography.diaspora {
        *out.entry((d.origin, d.dest)).or_insert(0) += d.arrived;
    }
    out
}

/// Country tags by market.
fn territory_names(w: &GameWorld) -> Vec<String> {
    w.countries.iter().map(|(_, c)| c.tag.clone()).collect()
}

/// `TAG/metropole` or `TAG/colonies`.
fn territory_name(tags: &[String], t: TerritoryId) -> String {
    let tag = tags.get(t.market()).cloned().unwrap_or_default();
    let part = if t.colony() { "colonies" } else { "metropole" };
    format!("{tag}/{part}")
}

fn prices(w: &GameWorld) -> Vec<(f64, u16)> {
    w.econ
        .markets
        .iter()
        .map(|m: &Market| (m.money.price_level.to_f64_lossy(), m.money.redenominations))
        .collect()
}

/// Population, literate people, people outside farming and workers by sector, by territory.
fn stocks(data: &GameData, w: &GameWorld) -> Vec<Stocks> {
    let cat = &data.catalogue;
    let sector: Vec<usize> = BUILDING_GROUPS.iter().map(|g| sector_of(g)).collect();
    let mut out = vec![Stocks::default(); TerritoryId::count(w.econ.markets.len())];
    for r in &w.econ.regions {
        let s = &mut out[TerritoryId::of(r).index()];
        let pop = r.population() as f64;
        s.population += pop;
        s.literate += r.literacy.to_f64_lossy() * pop;
        s.non_farm += r.urban_share().to_f64_lossy() * pop;
        for (c, v) in s.cleared.iter_mut().zip(r.cleared_km2) {
            *c += v as f64;
        }
        for p in &r.plants {
            let b = &cat.buildings[p.kind as usize];
            let m = &b.methods[p.method as usize];
            let jobs = (Fixed::from_int(p.levels * m.workers) * p.staffing).to_f64_lossy();
            s.employment[sector[usize::from(b.group_ix)]] += jobs;
            if b.informal {
                s.informal += jobs;
            }
        }
        s.employment[2] += state_payroll(r) as f64;
        s.employment[0] += smallholders(r) as f64;
        s.informal += smallholders(r) as f64;
    }
    out
}

/// Employment shares of a metropole (agriculture, industry, services), as the scorecard counts
/// them; for the diagnostics.
pub(crate) fn sector_shares(data: &GameData, w: &GameWorld, market: usize) -> [f64; 3] {
    let s = &stocks(data, w)[market * 2];
    let total: f64 = s.employment.iter().sum::<f64>().max(1.0);
    s.employment.map(|e| e / total)
}

/// Civil servants, clergy and soldiers at work.
fn state_payroll(r: &Region) -> i64 {
    use bf_sim::econ::prof;
    [prof::CIVIL_SERVANT, prof::CLERGY, prof::SOLDIER]
        .iter()
        .map(|&p| workers(r.pops[p].size))
        .sum()
}

/// Peasants of working age without a job: they work their own plot (`subsistence_food`).
fn smallholders(r: &Region) -> i64 {
    let p = &r.pops[bf_sim::econ::prof::PEASANT];
    (workers(p.size) - p.employed).max(0)
}

/// Writes the series as JSON.
pub fn write(h: History, path: &str) -> Result<(), String> {
    let json = serde_json::to_string(&h.finish()).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| format!("{path}: {e}"))
}
