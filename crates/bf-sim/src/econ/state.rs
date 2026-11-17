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
    /// Levels worn out but not yet retired: depreciation accrues in fractions of a level and a
    /// level goes when a whole one has worn out (`yearly::DEPRECIATION`).
    #[serde(default)]
    pub wear: Fixed,
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
    /// Land: farm index, ceiling in levels. For crops, the land open to the plough.
    pub land: Vec<(u8, i64)>,
    /// Crop levels that pasture and forest suited to the crop could still give, once cleared:
    /// farm index, levels (`monthly::Investment` opens them as works).
    #[serde(default)]
    pub frontier: Vec<(u8, i64)>,
    /// Pasture and forest left, km².
    #[serde(default)]
    pub wild_km2: [i64; 2],
    /// Pasture and forest turned into cropland since 1925, km².
    #[serde(default)]
    pub cleared_km2: [i64; 2],
    /// Land the pioneers have cleared toward the next farm level, km².
    #[serde(default)]
    pub pioneer_km2: Fixed,
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
    /// Currency, central bank and public debt.
    #[serde(default)]
    pub money: super::money::Money,
}

/// The country's laws that the economy reads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Laws {
    pub migration: super::migration::MigrationLaw,
    /// What the laws in force do to the economy, for the metropole (0) and the colonial regions
    /// (1); written by the politics system (`polity`), neutral without it.
    #[serde(default)]
    pub effects: [LawEffects; 2],
}

impl Laws {
    /// The effects that reach `r`.
    pub fn of(&self, r: &Region) -> &LawEffects {
        &self.effects[usize::from(r.colony)]
    }
}

/// The effects of the laws in force that the economy reads (`politics::EFFECTS` with a place
/// to fall). Every default is the rule the economy ran before politics: no law, no change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawEffects {
    /// Share of the open migration flows let through, within a state and between states.
    pub migration_reach: [Fixed; 2],
    /// Highest income tax rate of each class.
    pub tax_caps: [Fixed; TAX_CLASSES],
    /// Share of the assessed tax that reaches the treasury; the rest stays with the taxpayer.
    pub tax_collection: Fixed,
    /// (public good, share of it the treasury pays); the users pay the rest out of their
    /// savings. A good not listed is paid in full by the treasury.
    pub public_share: Vec<(u16, Fixed)>,
    /// Highest share of the school-age children schooling counts for.
    pub enrolment_target: Fixed,
    /// Landowners' take of farm profit relative to free tenancy (1 = the owners' payout).
    pub landowner_rent: Fixed,
    /// Share of industry the state owns: of what industry pays its owners, this share goes to
    /// the state instead of the capitalists.
    #[serde(default)]
    pub state_ownership: Fixed,
    /// Share of the state's take that reaches the treasury as revenue; the rest is reinvested.
    #[serde(default)]
    pub state_profit_to_treasury: Fixed,
    /// Ad valorem tariff on imports (`tariff` of the trade law); zero without politics.
    #[serde(default)]
    pub tariff: Fixed,
    /// Share of the pioneers' clearing the land law allows (`frontier_access`); 1 without one.
    #[serde(default = "one")]
    pub frontier_access: Fixed,
    /// Pull of foreign capital (`foreign_investment`, combined over the laws: 1 is open, 0.1
    /// closed); one without politics.
    #[serde(default = "one")]
    pub foreign_investment: Fixed,
    /// Colonies only: share of colonial revenue sent to the metropole (`colonial_tribute`) and
    /// share of what the colonial state spends on schools and hospitals that serves the
    /// colonizers instead of the natives (`discrimination` of the colonial rule).
    #[serde(default)]
    pub colonial_tribute: Fixed,
    #[serde(default)]
    pub discrimination: Fixed,
    /// Public money and legal room of the established religion, 0–1 (`church_privilege`;
    /// negative values count as zero): how far religious schools grow outside the Treasury.
    #[serde(default)]
    pub church_privilege: Fixed,
}

impl Default for LawEffects {
    fn default() -> Self {
        Self {
            migration_reach: [Fixed::ONE; 2],
            tax_caps: [Fixed::ONE; TAX_CLASSES],
            tax_collection: Fixed::ONE,
            public_share: Vec::new(),
            enrolment_target: Fixed::ONE,
            landowner_rent: Fixed::ONE,
            state_ownership: Fixed::ZERO,
            state_profit_to_treasury: Fixed::ZERO,
            tariff: Fixed::ZERO,
            frontier_access: Fixed::ONE,
            foreign_investment: Fixed::ONE,
            colonial_tribute: Fixed::ZERO,
            discrimination: Fixed::ZERO,
            church_privilege: Fixed::ZERO,
        }
    }
}

impl LawEffects {
    pub fn is_neutral(&self) -> bool {
        *self == Self::default()
    }

    /// Share of public good `g` the treasury pays.
    pub fn public_share(&self, g: usize) -> Fixed {
        self.public_share
            .iter()
            .find(|(x, _)| usize::from(*x) == g)
            .map_or(Fixed::ONE, |(_, s)| *s)
    }
}

impl HashState for LawEffects {
    fn hash_state(&self, h: &mut StateHasher) {
        self.migration_reach.hash_state(h);
        self.tax_caps.hash_state(h);
        self.tax_collection.hash_state(h);
        self.public_share.hash_state(h);
        self.enrolment_target.hash_state(h);
        self.landowner_rent.hash_state(h);
        self.state_ownership.hash_state(h);
        self.state_profit_to_treasury.hash_state(h);
        self.tariff.hash_state(h);
        self.frontier_access.hash_state(h);
        self.foreign_investment.hash_state(h);
        self.colonial_tribute.hash_state(h);
        self.discrimination.hash_state(h);
        self.church_privilege.hash_state(h);
    }
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
            money: super::money::Money::default(),
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
    /// Last week: taxes, new money the central bank lent the treasury (bonds it bought from
    /// the treasury), payroll, goods, public services and public works paid.
    pub revenue: Fixed,
    pub issued: Fixed,
    pub payroll: Fixed,
    pub goods: Fixed,
    pub services: Fixed,
    pub works: Fixed,
    /// Weekly revenue smoothed over about a quarter.
    pub revenue_avg: Fixed,
    /// Public debt, all holders, in real units (`money::Money::total_debt`).
    pub debt: Fixed,
    /// Interest paid last week (the central bank's is handed back and left out).
    #[serde(default)]
    pub interest: Fixed,
    /// Money the central bank created last week beyond its rule (a rate fixed below it).
    #[serde(default)]
    pub beyond_rule: Fixed,
    /// Tax on what the pops buy, 0–1: the income tax the taxation law's ceilings keep the
    /// rule from charging, collected on consumption instead (`public::cap_taxes`).
    #[serde(default)]
    pub indirect_rate: Fixed,
    /// Last week, inside `revenue`: the tax on consumption, and the profit of state firms
    /// paid to the treasury. The rest of `revenue` is income tax.
    #[serde(default)]
    pub revenue_indirect: Fixed,
    #[serde(default)]
    pub revenue_firms: Fixed,
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
            interest: Fixed::ZERO,
            beyond_rule: Fixed::ZERO,
            indirect_rate: Fixed::ZERO,
            revenue_indirect: Fixed::ZERO,
            revenue_firms: Fixed::ZERO,
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
            self.interest,
            self.beyond_rule,
        ] {
            v.hash_state(h);
        }
        // Budgets without politics keep their old hash.
        if self.indirect_rate != Fixed::ZERO
            || self.revenue_indirect != Fixed::ZERO
            || self.revenue_firms != Fixed::ZERO
        {
            self.indirect_rate.hash_state(h);
            self.revenue_indirect.hash_state(h);
            self.revenue_firms.hash_state(h);
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
    /// Debts between governments by treaty (`treaty.rs`): named debtor and creditor, served in
    /// the weekly money step. Empty in worlds without treaties.
    #[serde(default)]
    pub ledger: crate::treaty::Ledger,
}

/// The economy as saves wrote it before the ledger of debts between states (2026-11-17).
/// Postcard is not self-describing: loading an older save tries this layout
/// (`game::decode_world`).
#[derive(Deserialize)]
pub(crate) struct EconomyBeforeLedger {
    regions: Vec<Region>,
    markets: Vec<Market>,
    world_price: Vec<Fixed>,
    history: Vec<YearRecord>,
}

impl From<EconomyBeforeLedger> for Economy {
    fn from(e: EconomyBeforeLedger) -> Self {
        Self {
            regions: e.regions,
            markets: e.markets,
            world_price: e.world_price,
            history: e.history,
            ledger: Default::default(),
        }
    }
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
        self.wear.hash_state(h);
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
        // Worlds without a frontier keep their hash.
        if !self.frontier.is_empty() {
            h.write_len(self.frontier.len());
            for (f, c) in &self.frontier {
                h.write_u8(*f);
                h.write_i64(*c);
            }
            for v in self.wild_km2.iter().chain(&self.cleared_km2) {
                h.write_i64(*v);
            }
            self.pioneer_km2.hash_state(h);
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
        // Markets without politics keep their old hash, and old saves load.
        if self.laws.effects.iter().any(|e| !e.is_neutral()) {
            self.laws.effects.hash_state(h);
        }
        self.money.hash_state(h);
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
        // Worlds without debts between governments keep their hash.
        if !self.ledger.is_empty() {
            h.section("ledger");
            self.ledger.hash_state(h);
        }
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
                        wear: Fixed::ZERO,
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
