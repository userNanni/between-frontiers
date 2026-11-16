//! Economy state: saved, hashed, replayed. Everything here is integer or [`Fixed`].

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::prof::P;

/// People of one profession in one state (dependents included).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pop {
    pub size: i64,
    /// Workers in jobs (≤ size × participation).
    pub employed: i64,
    /// Money income of the last week, after tax.
    pub income: Fixed,
    pub savings: Fixed,
    /// Share of each need tier met last week, 0–1.
    pub satisfaction: [Fixed; 4],
}

/// Levels of one building kind in a state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plant {
    /// Index into the catalogue's buildings.
    pub kind: u16,
    pub levels: i64,
    pub method: u8,
    /// Share of the jobs filled, 0–1.
    pub staffing: Fixed,
    /// Share of the inputs obtained last week, 0–1.
    pub input_ratio: Fixed,
    /// Share of its capacity the plant runs, following what it sold: a plant that sold 70%
    /// cuts production instead of piling up unsold goods.
    #[serde(default = "one")]
    pub utilization: Fixed,
    /// Value added minus wages of the last week (the owners' share).
    pub profit: Fixed,
    /// Weeks in a row with a loss; a level closes after a quarter of losses.
    pub losing_weeks: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// Market (country index) the state trades in.
    pub market: u16,
    pub colony: bool,
    /// One entry per profession (`prof`).
    pub pops: Vec<Pop>,
    pub plants: Vec<Plant>,
    pub literacy: Fixed,
    /// Deposits: resource index, discovery year, ceiling in levels of the base method.
    pub deposits: Vec<(u8, i32, i64)>,
    /// Land: farm index, ceiling in levels.
    pub land: Vec<(u8, i64)>,
    /// Staple food the unemployed grow for themselves, per 1,000 people per week, in tonnes.
    pub subsistence: Fixed,
    /// Labour productivity: scales a plant's inputs and outputs per worker. Calibrated in
    /// 1925 from GDP per worker; research moves it from phase 2 on.
    pub tfp: Fixed,
    /// Productivity at the start; growth since then reaches services only in part.
    #[serde(default)]
    pub tfp_start: Fixed,
    /// Id of the map state this region is (`StateDef::id`).
    #[serde(default)]
    pub state: u16,
    /// Neighbouring regions of the same market (indices), from the map's land borders.
    #[serde(default)]
    pub neighbours: Vec<u32>,
    /// Goods that clear here (`Catalogue::local` order). Empty until the first week.
    #[serde(default)]
    pub local: Vec<LocalGood>,
}

/// A good made and used in the state: transport, electricity before the national grid.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalGood {
    pub price: Fixed,
    /// Share of the state's demand met last week, from its own plants and its neighbours.
    pub buy: Fixed,
    /// Share of the state's supply sold last week.
    pub sell: Fixed,
    /// Demand last week.
    pub demand: Fixed,
}

impl HashState for LocalGood {
    fn hash_state(&self, h: &mut StateHasher) {
        self.price.hash_state(h);
        self.buy.hash_state(h);
        self.sell.hash_state(h);
        self.demand.hash_state(h);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Market {
    pub price: Vec<Fixed>,
    pub supply: Vec<Fixed>,
    pub demand: Vec<Fixed>,
    /// Share of the demand met last week.
    pub buy_ratio: Vec<Fixed>,
    /// Share of the supply sold last week.
    pub sell_ratio: Vec<Fixed>,
    /// Imports (positive) or exports (negative) last week.
    pub net_import: Vec<Fixed>,
    /// Savings waiting to be invested.
    pub pool: Fixed,
    pub treasury: Fixed,
    /// Productivity factor calibrated in 1925 against the country's GDP.
    pub tfp: Fixed,
    /// Construction goods still to be bought, by good.
    pub construction: Vec<Fixed>,
    /// GDP at 1925 base prices accumulated this year, and the last full year.
    pub gdp_running: Fixed,
    pub gdp_last_year: Fixed,
    /// Productivity from technology by building group, relative to 1925; empty means 1.
    #[serde(default)]
    pub tech_tfp: Vec<Fixed>,
    /// Workers on the state payroll (civil servants, clergy, soldiers), counted monthly.
    pub gov_workers: i64,
    /// Works waiting for construction points, oldest first.
    #[serde(default)]
    pub queue: Vec<Project>,
    /// Taxes, spending and public investment of the country.
    #[serde(default)]
    pub budget: Budget,
    #[serde(default)]
    pub laws: Laws,
}

/// The country's laws that the economy reads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Laws {
    pub migration: super::migration::MigrationLaw,
}

impl Market {
    /// A market for `goods` goods at zero prices, with no money: for tests and tools.
    pub fn empty(goods: usize) -> Self {
        let zeros = vec![Fixed::ZERO; goods];
        Self {
            price: zeros.clone(),
            supply: zeros.clone(),
            demand: zeros.clone(),
            buy_ratio: zeros.clone(),
            sell_ratio: zeros.clone(),
            net_import: zeros.clone(),
            pool: Fixed::ZERO,
            treasury: Fixed::ZERO,
            tfp: Fixed::ONE,
            construction: zeros,
            gdp_running: Fixed::ZERO,
            gdp_last_year: Fixed::ZERO,
            tech_tfp: Vec::new(),
            gov_workers: 0,
            queue: Vec::new(),
            budget: Budget::default(),
            laws: Laws::default(),
        }
    }
}

/// A level under construction: it runs only when its points are all delivered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub region: u32,
    pub kind: u16,
    pub method: u8,
    /// Paid by the treasury (true) or by the investment pool.
    pub public: bool,
    /// Construction points the work costs, and those still missing.
    pub points: Fixed,
    pub left: Fixed,
}

/// Tax classes: lower (peasants, workers, soldiers, clergy), middle (skilled, clerks,
/// engineers, professionals, servants), upper (managers, capitalists, landowners).
pub const TAX_CLASSES: usize = 3;
/// Public investment categories: infrastructure, education, health, urban housing.
pub const PUBLIC_CATEGORIES: usize = 4;

/// The country's treasury rules and last week's accounts. The balance itself is
/// `Market::treasury`, which may go negative: a deficit accumulates there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    /// Income tax by class (`TAX_CLASSES`), 0–1.
    pub tax_rates: [Fixed; TAX_CLASSES],
    /// Rates follow the historical rule until the player sets them.
    pub tax_auto: bool,
    /// Fiscal correction of the rule: rises after deficits, falls after surpluses.
    pub adjust: Fixed,
    /// Weight of each public investment category (`PUBLIC_CATEGORIES`).
    pub priority: [u8; PUBLIC_CATEGORIES],
    /// Last week: taxes, money issued by the monetary stand-in (negative when withdrawn),
    /// payroll, goods, public services and public works paid.
    pub revenue: Fixed,
    pub issued: Fixed,
    pub payroll: Fixed,
    pub goods: Fixed,
    pub services: Fixed,
    pub works: Fixed,
    /// Weekly revenue smoothed over about a quarter.
    pub revenue_avg: Fixed,
    /// Public debt. Always zero in this slice: a deficit only takes the treasury below zero.
    /// The next slice (currencies, central bank, bonds) turns the negative balance into bonds
    /// with interest.
    pub debt: Fixed,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            tax_rates: super::public::historical_rates(1925),
            tax_auto: true,
            adjust: Fixed::ONE,
            priority: super::public::DEFAULT_PRIORITY,
            revenue: Fixed::ZERO,
            issued: Fixed::ZERO,
            payroll: Fixed::ZERO,
            goods: Fixed::ZERO,
            services: Fixed::ZERO,
            works: Fixed::ZERO,
            revenue_avg: Fixed::ZERO,
            debt: Fixed::ZERO,
        }
    }
}

impl HashState for Project {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_i64(i64::from(self.region));
        h.write_u16(self.kind);
        h.write_u8(self.method);
        self.public.hash_state(h);
        self.points.hash_state(h);
        self.left.hash_state(h);
    }
}

impl HashState for Budget {
    fn hash_state(&self, h: &mut StateHasher) {
        for r in &self.tax_rates {
            r.hash_state(h);
        }
        self.tax_auto.hash_state(h);
        self.adjust.hash_state(h);
        for p in &self.priority {
            h.write_u8(*p);
        }
        for v in [
            self.revenue,
            self.issued,
            self.payroll,
            self.goods,
            self.services,
            self.works,
            self.revenue_avg,
            self.debt,
        ] {
            v.hash_state(h);
        }
    }
}

/// One row of the yearly statistics of a market.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct YearRecord {
    pub year: i32,
    pub market: u16,
    pub population: i64,
    pub gdp: Fixed,
    pub literacy: Fixed,
    pub urban: Fixed,
    /// Average satisfaction of tiers 1 and 2, 0–1.
    pub basic_needs: Fixed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Economy {
    pub regions: Vec<Region>,
    pub markets: Vec<Market>,
    pub world_price: Vec<Fixed>,
    pub history: Vec<YearRecord>,
}

impl HashState for Pop {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_i64(self.size);
        h.write_i64(self.employed);
        self.income.hash_state(h);
        self.savings.hash_state(h);
        for s in &self.satisfaction {
            s.hash_state(h);
        }
    }
}

impl HashState for Plant {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.kind);
        h.write_i64(self.levels);
        h.write_u8(self.method);
        self.staffing.hash_state(h);
        self.input_ratio.hash_state(h);
        self.utilization.hash_state(h);
        self.profit.hash_state(h);
        h.write_u16(self.losing_weeks);
    }
}

impl HashState for Region {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.market);
        self.colony.hash_state(h);
        self.pops.hash_state(h);
        self.plants.hash_state(h);
        self.literacy.hash_state(h);
        h.write_len(self.deposits.len());
        for (r, y, c) in &self.deposits {
            h.write_u8(*r);
            h.write_i64(i64::from(*y));
            h.write_i64(*c);
        }
        h.write_len(self.land.len());
        for (f, c) in &self.land {
            h.write_u8(*f);
            h.write_i64(*c);
        }
        self.subsistence.hash_state(h);
        self.tfp.hash_state(h);
        self.tfp_start.hash_state(h);
        h.write_u16(self.state);
        h.write_len(self.neighbours.len());
        for n in &self.neighbours {
            h.write_i64(i64::from(*n));
        }
        self.local.hash_state(h);
    }
}

impl HashState for Market {
    fn hash_state(&self, h: &mut StateHasher) {
        for v in [
            &self.price,
            &self.supply,
            &self.demand,
            &self.buy_ratio,
            &self.sell_ratio,
            &self.net_import,
            &self.construction,
        ] {
            v.hash_state(h);
        }
        for v in [
            self.pool,
            self.treasury,
            self.tfp,
            self.gdp_running,
            self.gdp_last_year,
        ] {
            v.hash_state(h);
        }
        h.write_i64(self.gov_workers);
        self.tech_tfp.hash_state(h);
        self.queue.hash_state(h);
        self.budget.hash_state(h);
        h.write_u8(self.laws.migration as u8);
    }
}

impl HashState for YearRecord {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_i64(i64::from(self.year));
        h.write_u16(self.market);
        h.write_i64(self.population);
        for v in [self.gdp, self.literacy, self.urban, self.basic_needs] {
            v.hash_state(h);
        }
    }
}

impl HashState for Economy {
    fn hash_state(&self, h: &mut StateHasher) {
        h.section("regions");
        self.regions.hash_state(h);
        h.section("markets");
        self.markets.hash_state(h);
        h.section("world_price");
        self.world_price.hash_state(h);
        h.section("history");
        self.history.hash_state(h);
    }
}

/// Share of productivity growth that reaches services (per mille).
const STAGNANT_GROWTH: i64 = 300;

impl Region {
    /// Productivity a plant of this building sees here.
    pub fn tfp_for(&self, b: &super::catalogue::Building) -> Fixed {
        if !b.stagnant || self.tfp_start <= Fixed::ZERO || self.tfp <= self.tfp_start {
            return self.tfp;
        }
        self.tfp_start + (self.tfp - self.tfp_start) * super::milli(STAGNANT_GROWTH)
    }
    pub fn population(&self) -> i64 {
        self.pops.iter().map(|p| p.size).sum()
    }

    pub fn plant_mut(&mut self, kind: u16) -> &mut Plant {
        let at = match self.plants.binary_search_by_key(&kind, |p| p.kind) {
            Ok(i) => i,
            Err(i) => {
                self.plants.insert(
                    i,
                    Plant {
                        kind,
                        levels: 0,
                        method: 0,
                        staffing: Fixed::ZERO,
                        input_ratio: Fixed::ONE,
                        utilization: Fixed::ONE,
                        profit: Fixed::ZERO,
                        losing_weeks: 0,
                    },
                );
                i
            }
        };
        &mut self.plants[at]
    }

    pub fn urban_share(&self) -> Fixed {
        let total = self.population();
        if total == 0 {
            return Fixed::ZERO;
        }
        let rural: i64 = [
            super::prof::PEASANT,
            super::prof::FARM_LABORER,
            super::prof::LANDOWNER,
        ]
        .iter()
        .map(|&p| self.pops[p].size)
        .sum();
        Fixed::from_ratio(total - rural, total)
    }
}

fn one() -> Fixed {
    Fixed::ONE
}

pub fn empty_pops() -> Vec<Pop> {
    vec![Pop::default(); P]
}
