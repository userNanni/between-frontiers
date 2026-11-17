//! Saves from before research that costs (`docs/research.pt-BR.md`, "Pesquisa que custa"): the
//! budget had four public works categories and the law effects no research priority. A snapshot
//! of that layout decodes into these mirrors and converts into the economy of today, with the
//! research category at its default weight and no research priority, which hash as before.

use laplace_schelling::Fixed;
use serde::{Deserialize, Serialize};

use super::state::{
    Budget, Economy, LawEffects, Laws, Market, Project, Region, TAX_CLASSES, YearRecord,
};

/// The economy as the ledger of debts between states left it, with the markets of before.
#[derive(Deserialize)]
pub(crate) struct EconomyBeforeResearch {
    regions: Vec<Region>,
    markets: Vec<MarketBeforeResearch>,
    world_price: Vec<Fixed>,
    history: Vec<YearRecord>,
    ledger: crate::treaty::Ledger,
}

/// A market with four public works categories and no research priority.
#[derive(Deserialize, Serialize)]
pub(crate) struct MarketBeforeResearch {
    price: Vec<Fixed>,
    supply: Vec<Fixed>,
    demand: Vec<Fixed>,
    buy_ratio: Vec<Fixed>,
    sell_ratio: Vec<Fixed>,
    net_import: Vec<Fixed>,
    pool: Fixed,
    treasury: Fixed,
    tfp: Fixed,
    construction: Vec<Fixed>,
    gdp_running: Fixed,
    gdp_last_year: Fixed,
    tech_tfp: Vec<Fixed>,
    gov_workers: i64,
    queue: Vec<Project>,
    budget: BudgetBeforeResearch,
    laws: LawsBeforeResearch,
    money: super::money::Money,
}

#[derive(Deserialize, Serialize)]
struct BudgetBeforeResearch {
    tax_rates: [Fixed; TAX_CLASSES],
    tax_auto: bool,
    adjust: Fixed,
    priority: [u8; 4],
    revenue: Fixed,
    issued: Fixed,
    payroll: Fixed,
    goods: Fixed,
    services: Fixed,
    works: Fixed,
    revenue_avg: Fixed,
    debt: Fixed,
    interest: Fixed,
    beyond_rule: Fixed,
    indirect_rate: Fixed,
    revenue_indirect: Fixed,
    revenue_firms: Fixed,
}

#[derive(Deserialize, Serialize)]
struct LawsBeforeResearch {
    migration: super::migration::MigrationLaw,
    effects: [LawEffectsBeforeResearch; 2],
}

#[derive(Deserialize, Serialize)]
struct LawEffectsBeforeResearch {
    migration_reach: [Fixed; 2],
    tax_caps: [Fixed; TAX_CLASSES],
    tax_collection: Fixed,
    public_share: Vec<(u16, Fixed)>,
    enrolment_target: Fixed,
    landowner_rent: Fixed,
    state_ownership: Fixed,
    state_profit_to_treasury: Fixed,
    tariff: Fixed,
    frontier_access: Fixed,
    foreign_investment: Fixed,
    colonial_tribute: Fixed,
    discrimination: Fixed,
    church_privilege: Fixed,
}

impl From<EconomyBeforeResearch> for Economy {
    fn from(e: EconomyBeforeResearch) -> Self {
        Self {
            regions: e.regions,
            markets: e.markets.into_iter().map(Market::from).collect(),
            world_price: e.world_price,
            history: e.history,
            ledger: e.ledger,
            trade: Default::default(),
        }
    }
}

impl From<MarketBeforeResearch> for Market {
    fn from(m: MarketBeforeResearch) -> Self {
        Self {
            price: m.price,
            supply: m.supply,
            demand: m.demand,
            buy_ratio: m.buy_ratio,
            sell_ratio: m.sell_ratio,
            net_import: m.net_import,
            pool: m.pool,
            treasury: m.treasury,
            tfp: m.tfp,
            construction: m.construction,
            gdp_running: m.gdp_running,
            gdp_last_year: m.gdp_last_year,
            tech_tfp: m.tech_tfp,
            gov_workers: m.gov_workers,
            queue: m.queue,
            budget: m.budget.into(),
            laws: Laws {
                migration: m.laws.migration,
                effects: m.laws.effects.map(LawEffects::from),
            },
            money: m.money,
        }
    }
}

impl From<BudgetBeforeResearch> for Budget {
    fn from(b: BudgetBeforeResearch) -> Self {
        let mut priority = super::public::DEFAULT_PRIORITY;
        priority[..4].copy_from_slice(&b.priority);
        Self {
            tax_rates: b.tax_rates,
            tax_auto: b.tax_auto,
            adjust: b.adjust,
            priority,
            revenue: b.revenue,
            issued: b.issued,
            payroll: b.payroll,
            goods: b.goods,
            services: b.services,
            works: b.works,
            revenue_avg: b.revenue_avg,
            debt: b.debt,
            interest: b.interest,
            beyond_rule: b.beyond_rule,
            indirect_rate: b.indirect_rate,
            revenue_indirect: b.revenue_indirect,
            revenue_firms: b.revenue_firms,
        }
    }
}

impl From<LawEffectsBeforeResearch> for LawEffects {
    fn from(e: LawEffectsBeforeResearch) -> Self {
        Self {
            migration_reach: e.migration_reach,
            tax_caps: e.tax_caps,
            tax_collection: e.tax_collection,
            public_share: e.public_share,
            enrolment_target: e.enrolment_target,
            landowner_rent: e.landowner_rent,
            state_ownership: e.state_ownership,
            state_profit_to_treasury: e.state_profit_to_treasury,
            tariff: e.tariff,
            frontier_access: e.frontier_access,
            foreign_investment: e.foreign_investment,
            colonial_tribute: e.colonial_tribute,
            discrimination: e.discrimination,
            church_privilege: e.church_privilege,
            research_priority: Fixed::ZERO,
        }
    }
}

/// Goods the catalogue added after a save was made (`research` came with research that costs):
/// every per-good list of the economy grows to the catalogue's length, the new goods at their
/// base price and with nothing bought or sold yet. Runs after the snapshot's hash was checked,
/// so the hash stays the one the save recorded.
pub(crate) fn fit_goods(econ: &mut Economy, cat: &super::Catalogue) {
    let n = cat.goods.len();
    let base = |g: usize| cat.goods[g].base_price;
    for m in &mut econ.markets {
        for g in m.price.len()..n {
            m.price.push(base(g));
        }
        for v in [
            &mut m.supply,
            &mut m.demand,
            &mut m.buy_ratio,
            &mut m.sell_ratio,
            &mut m.net_import,
            &mut m.construction,
        ] {
            v.resize(n.max(v.len()), Fixed::ZERO);
        }
    }
    for g in econ.world_price.len()..n {
        econ.world_price.push(base(g));
    }
}

#[cfg(test)]
impl MarketBeforeResearch {
    /// A market of today written as before research that costs, for the tests of old saves.
    /// Only for markets at the default research weight and without a research priority.
    pub(crate) fn of(m: &Market) -> Self {
        let b = &m.budget;
        let effect = |e: &LawEffects| LawEffectsBeforeResearch {
            migration_reach: e.migration_reach,
            tax_caps: e.tax_caps,
            tax_collection: e.tax_collection,
            public_share: e.public_share.clone(),
            enrolment_target: e.enrolment_target,
            landowner_rent: e.landowner_rent,
            state_ownership: e.state_ownership,
            state_profit_to_treasury: e.state_profit_to_treasury,
            tariff: e.tariff,
            frontier_access: e.frontier_access,
            foreign_investment: e.foreign_investment,
            colonial_tribute: e.colonial_tribute,
            discrimination: e.discrimination,
            church_privilege: e.church_privilege,
        };
        Self {
            price: m.price.clone(),
            supply: m.supply.clone(),
            demand: m.demand.clone(),
            buy_ratio: m.buy_ratio.clone(),
            sell_ratio: m.sell_ratio.clone(),
            net_import: m.net_import.clone(),
            pool: m.pool,
            treasury: m.treasury,
            tfp: m.tfp,
            construction: m.construction.clone(),
            gdp_running: m.gdp_running,
            gdp_last_year: m.gdp_last_year,
            tech_tfp: m.tech_tfp.clone(),
            gov_workers: m.gov_workers,
            queue: m.queue.clone(),
            budget: BudgetBeforeResearch {
                tax_rates: b.tax_rates,
                tax_auto: b.tax_auto,
                adjust: b.adjust,
                priority: [b.priority[0], b.priority[1], b.priority[2], b.priority[3]],
                revenue: b.revenue,
                issued: b.issued,
                payroll: b.payroll,
                goods: b.goods,
                services: b.services,
                works: b.works,
                revenue_avg: b.revenue_avg,
                debt: b.debt,
                interest: b.interest,
                beyond_rule: b.beyond_rule,
                indirect_rate: b.indirect_rate,
                revenue_indirect: b.revenue_indirect,
                revenue_firms: b.revenue_firms,
            },
            laws: LawsBeforeResearch {
                migration: m.laws.migration,
                effects: [effect(&m.laws.effects[0]), effect(&m.laws.effects[1])],
            },
            money: m.money.clone(),
        }
    }
}

/// The economy as saves wrote it before trade by pair (2026-11-17): the same fields without the
/// trade state, which loads empty and fills in the first week.
#[derive(Deserialize)]
pub(crate) struct EconomyBeforeTrade {
    regions: Vec<Region>,
    markets: Vec<Market>,
    world_price: Vec<Fixed>,
    history: Vec<YearRecord>,
    ledger: crate::treaty::Ledger,
}

impl From<EconomyBeforeTrade> for Economy {
    fn from(e: EconomyBeforeTrade) -> Self {
        Self {
            regions: e.regions,
            markets: e.markets,
            world_price: e.world_price,
            history: e.history,
            ledger: e.ledger,
            trade: Default::default(),
        }
    }
}

impl From<Economy> for EconomyBeforeTrade {
    fn from(e: Economy) -> Self {
        Self {
            regions: e.regions,
            markets: e.markets,
            world_price: e.world_price,
            history: e.history,
            ledger: e.ledger,
        }
    }
}

/// The steps from before research that costs and from before the ledger land on the economy of
/// version 5 and 6, which had no trade by pair.
impl From<EconomyBeforeResearch> for EconomyBeforeTrade {
    fn from(e: EconomyBeforeResearch) -> Self {
        Economy::from(e).into()
    }
}

impl From<super::state::EconomyBeforeLedger> for EconomyBeforeTrade {
    fn from(e: super::state::EconomyBeforeLedger) -> Self {
        Economy::from(e).into()
    }
}
