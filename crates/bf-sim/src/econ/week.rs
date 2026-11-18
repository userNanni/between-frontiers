//! The weekly market: orders, clearing with world trade, settlement.

use std::sync::Arc;

use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};
use laplace_schelling::{Fixed, Weekday};
use rayon::prelude::*;

use super::catalogue::{Building, Catalogue, GoodIx, Method};
use super::local::View;
use super::prof::{self, P};
use super::public::{self, CLASS_OF};
use super::state::{Economy, Market, Plant, Region};
use super::{PAYOUT, PriceBand, SAVE_RATE, Unlocks, WAGE_WEIGHT, fill, milli, plant_tfp, workers};
use crate::game::GameWorld;

pub const ECON: ResourceId = ResourceId(10);

/// Market prices: tension (D − S) / min(D, S), held within 25%–400% of base.
const PRICES: PriceBand = PriceBand {
    floor: milli(250),
    ceiling: milli(4000),
};
/// Share of a market's surplus offered abroad, and of its shortage sought abroad, each week:
/// below 1 so that local prices still carry part of the signal (distance, ports, habits). The
/// world pool's (trade by pair switched off); trade by pair reads `PurchasesFile::trade`.
const TRADE_REACH: Fixed = milli(850);
/// Share of the investment pool spent each week on upkeep and housing (`Catalogue::upkeep_basket`).
pub(crate) const UPKEEP_SHARE: Fixed = milli(30);

/// What `money` buys of each good of a basket, shared by its weights per mille.
fn basket_orders<'a>(
    basket: impl Iterator<Item = &'a (GoodIx, i64)>,
    money: Fixed,
    price: &[Fixed],
) -> Vec<(GoodIx, Fixed)> {
    basket
        .map(|&(g, per_mille)| (g, money.mul_ratio(per_mille, 1000) / price[g as usize]))
        .collect()
}

/// What the pool orders for upkeep this week, from last week's pool and prices. The same
/// quantities join the demand and are paid for, so the pool pays for what it got.
fn upkeep_orders(cat: &Catalogue, m: &Market) -> Vec<(GoodIx, Fixed)> {
    let money = m.pool.max(Fixed::ZERO) * UPKEEP_SHARE;
    basket_orders(cat.upkeep_basket.iter(), money, &m.price)
}

/// What the state buys this week with its goods budget. With forces (`military`), their orders
/// come first and the rest buys the civilian part of the basket, its weights renormalized
/// (`docs/war.pt-BR.md`, "Forças em paz").
fn state_orders(
    cat: &Catalogue,
    m: &Market,
    exists: &[bool],
    military: Option<&[(GoodIx, Fixed)]>,
) -> Vec<(GoodIx, Fixed)> {
    let goods = cat.state_basket.iter().filter(|(g, _)| exists[*g as usize]);
    let Some(military) = military else {
        return basket_orders(goods, public::goods_budget(m), &m.price);
    };
    crate::war::state_purchases(cat, goods, military, public::goods_budget(m), &m.price)
}

/// Construction goods are bought over about two months: an eighth of what is left each week,
/// and at least one unit.
fn construction_this_week(left: Fixed) -> Fixed {
    left.mul_ratio(1, 8).max(left.min(Fixed::ONE))
}

pub struct Week {
    pub cat: Arc<Catalogue>,
    pub tree: Option<Arc<crate::research::TechTree>>,
    /// Distances for the freight of trade (`econ::trade::Geography`).
    pub geo: Option<Arc<super::trade::Geography>>,
    /// Trade by pair (`game::Switches::trade_by_pair`); off, the world pool shares trade pro
    /// rata and the trade state stays empty.
    pub by_pair: bool,
    /// `game::Switches::small_market_wage_floor`.
    pub wage_floor: bool,
    /// Capital crosses borders (`game::Switches::foreign_flow`); off, the week runs as before
    /// it: no foreign owner is read, no profit is remitted, no work becomes a stake.
    pub foreign_flow: bool,
}

/// What a pop wants to buy this week.
/// Most a pop spends on its upper needs (tiers 3 and 4), in multiples of their weighted cost.
/// The richest of 1925 (owners at some $60,000 a head) spent most of their income on servants,
/// travel and luxury; what is beyond this is invested (`PopPlan::unplaced`).
const SPARE_CEILING: Fixed = Fixed::from_int(120);

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct PopPlan {
    pub(crate) orders: Vec<(GoodIx, Fixed)>,
    /// Money the full need of each tier would cost, and what the plan covers.
    tier_cost: [Fixed; 4],
    tier_planned: [Fixed; 4],
    /// Share of tier 1 already covered by the pop's own food.
    in_kind: Fixed,
    /// Budget the plan found nothing to spend on: the needs of the rich have a ceiling, and
    /// what they cannot consume they invest (it goes to the pool, not to the buffer).
    pub(crate) unplaced: Fixed,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct RegionOrders {
    demand: Vec<Fixed>,
    supply: Vec<Fixed>,
    pops: Vec<Option<PopPlan>>,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct RegionSettlement {
    pub pops: Vec<(Fixed, Fixed, [Fixed; 4])>,
    pub plants: Vec<(Fixed, Fixed, Fixed, Fixed)>,
    pub taxes: Fixed,
    /// Inside `taxes`: the tax on consumption and the state firms' profit to the treasury.
    pub indirect: Fixed,
    pub firms: Fixed,
    /// Pay of the servants, soldiers and clergy here.
    pub payroll: Fixed,
    pub pool: Fixed,
    pub gdp: Fixed,
    /// The foreign owners' part of the profit, by owner market: what leaves for their funds,
    /// and what they keep here to invest (already in `pool`).
    pub remit: Vec<(u16, Fixed, Fixed)>,
}

pub struct WeekChanges {
    markets: Vec<Market>,
    /// What each state ordered with its goods budget.
    state_orders: Vec<Vec<(GoodIx, Fixed)>>,
    regions: Vec<RegionSettlement>,
    world_price: Vec<Fixed>,
    /// Local goods of each region this week.
    local: Vec<Vec<super::state::LocalGood>>,
    /// What each market's pool ordered for upkeep.
    upkeep: Vec<Vec<(GoodIx, Fixed)>>,
    /// Exports by pair this week at base prices, seller × markets + buyer.
    trade_value: Vec<Fixed>,
    /// The preferences of trade after a month of inertia, in the first week of a month.
    trade_pref: Option<Vec<Vec<(u16, u16)>>>,
}

/// Cost-weighted choice among the goods that meet a need: shares fall with the square of the
/// price per unit of need, so the cheapest good dominates without a cliff.
fn need_mix(
    cat: &Catalogue,
    need: usize,
    prices: &[Fixed],
    exists: &[bool],
) -> (Vec<(GoodIx, Fixed)>, Fixed) {
    let n = &cat.needs[need];
    let options: Vec<(GoodIx, Fixed, Fixed)> = n
        .goods
        .iter()
        .filter(|(g, w)| exists[*g as usize] && *w > Fixed::ZERO)
        .map(|(g, w)| (*g, *w, prices[*g as usize] / *w))
        .collect();
    let Some(cheapest) = options.iter().map(|o| o.2).min() else {
        return (Vec::new(), Fixed::ZERO);
    };
    let weights: Vec<Fixed> = options
        .iter()
        .map(|o| {
            let r = fill(cheapest, o.2);
            r * r
        })
        .collect();
    let total: Fixed = weights.iter().copied().sum();
    let mut cost = Fixed::ZERO;
    let mix = options
        .iter()
        .zip(&weights)
        .map(|(o, w)| {
            let share = *w / total;
            cost += share * o.2;
            // Units of the good per unit of need.
            (o.0, share / o.1)
        })
        .collect();
    (mix, cost)
}

/// The goods each need is met with at a set of prices, and what a unit of the need costs. The
/// same for every pop that sees those prices, so it is worked out once per state and week.
pub(crate) struct NeedMixes {
    mixes: Vec<(Vec<(GoodIx, Fixed)>, Fixed)>,
    /// Goods across all mixes: the most orders a pop can place, so its list is allocated once.
    goods: usize,
}

impl NeedMixes {
    pub(crate) fn new(cat: &Catalogue, prices: &[Fixed], exists: &[bool]) -> Self {
        let mixes: Vec<_> = (0..cat.needs.len())
            .map(|i| need_mix(cat, i, prices, exists))
            .collect();
        let goods = mixes.iter().map(|(mix, _)| mix.len()).sum();
        Self { mixes, goods }
    }
}

pub(crate) fn plan_pop(
    cat: &Catalogue,
    mixes: &NeedMixes,
    size: i64,
    budget: Fixed,
    staple_in_kind: Fixed,
    urban: Option<bool>,
) -> PopPlan {
    let mut tier_cost = [Fixed::ZERO; 4];
    let mut need_cost = Vec::with_capacity(cat.needs.len());
    let mut in_kind = Fixed::ZERO;
    for (i, n) in cat.needs.iter().enumerate() {
        // City rent is for townspeople; the countryside builds its own shelter.
        let mut qty = match (n.urban, urban) {
            (Some(need), Some(pop)) if need != pop => Fixed::ZERO,
            _ => n.per_capita.mul_ratio(size, 1_000_000),
        };
        if Some(i) == cat.staple_need && qty > Fixed::ZERO {
            in_kind = fill(staple_in_kind, qty);
            qty = (qty - staple_in_kind).max(Fixed::ZERO);
        }
        let (mix, unit_cost) = &mixes.mixes[i];
        let cost = qty * *unit_cost;
        tier_cost[(n.tier - 1) as usize] += cost;
        need_cost.push((qty, mix, cost));
    }
    // Tiers in order; whatever is left buys more of tiers 3 and 4.
    let mut left = budget;
    let mut tier_share = [Fixed::ZERO; 4];
    for t in 0..4 {
        if tier_cost[t] <= Fixed::ZERO {
            tier_share[t] = Fixed::ONE;
            continue;
        }
        let share = fill(left, tier_cost[t]);
        tier_share[t] = share;
        left -= tier_cost[t] * share;
    }
    let upper = tier_cost[2] + tier_cost[3];
    let extra = if left > Fixed::ZERO && upper > Fixed::ZERO {
        (left / upper).min(Fixed::from_int(20))
    } else {
        Fixed::ZERO
    };
    let weighted_upper: Fixed = need_cost
        .iter()
        .enumerate()
        .filter(|(i, _)| cat.needs[*i].tier >= 3)
        .map(|(i, (_, _, c))| *c * cat.needs[i].spare_weight)
        .sum();
    let extra = if extra > Fixed::ZERO && weighted_upper > Fixed::ZERO {
        // Same money, redistributed by weight, up to `SPARE_CEILING` times the upper needs.
        (left / weighted_upper).min(SPARE_CEILING)
    } else {
        Fixed::ZERO
    };
    let mut orders: Vec<(GoodIx, Fixed)> = Vec::with_capacity(mixes.goods);
    let mut tier_planned = [Fixed::ZERO; 4];
    for (i, (qty, mix, cost)) in need_cost.iter().enumerate() {
        let t = (cat.needs[i].tier - 1) as usize;
        let mut share = tier_share[t];
        if t >= 2 {
            share += extra * cat.needs[i].spare_weight;
        }
        tier_planned[t] += *cost * share;
        let q = *qty * share;
        for &(g, per_unit) in mix.iter() {
            let units = q * per_unit;
            if units > Fixed::ZERO {
                orders.push((g, units));
            }
        }
    }
    let placed: Fixed = tier_planned.iter().copied().sum();
    PopPlan {
        orders,
        tier_cost,
        tier_planned,
        in_kind,
        unplaced: (budget - placed).max(Fixed::ZERO),
    }
}

/// What a plant could run at full use: levels × staffing × productivity.
pub(crate) fn plant_capacity(cat: &Catalogue, r: &Region, m: &Market, p: &Plant) -> Fixed {
    Fixed::from_int(p.levels) * p.staffing * plant_tfp(r, m, &cat.buildings[p.kind as usize])
}

/// A plant's building and method, and the scale it runs at this week: its capacity ×
/// utilization.
pub(crate) fn plant_run<'c>(
    cat: &'c Catalogue,
    r: &Region,
    m: &Market,
    p: &Plant,
) -> (&'c Building, &'c Method, Fixed) {
    let b = &cat.buildings[p.kind as usize];
    let scale = plant_capacity(cat, r, m, p) * p.utilization;
    (b, &b.methods[p.method as usize], scale)
}

/// Grain smallholders bring to market: a sixth of what they eat, rising to half as farming
/// technology (seed, fertilizer, machines) improves. The region's own catch-up in productivity
/// (institutions, schooling) does not make a smallholder's plot yield more.
pub fn peasant_surplus(m: &Market, staple: Fixed) -> Fixed {
    let agri = m.tech_tfp.first().copied().unwrap_or(Fixed::ONE);
    let gain = (agri - Fixed::ONE).clamp(Fixed::ZERO, Fixed::ONE);
    staple * (milli(160) + gain * milli(340))
}

/// Staples the jobless grow for themselves. Smallholders get half the gain of farm technology
/// (better seed, fertilizer): the Green Revolution reached them too. They till only the
/// cropland the farms leave free: where farms hold the land, the jobless grow less and look for
/// work in town instead of crowding the farms out of their market.
pub fn subsistence_food(cat: &Catalogue, r: &Region, m: &Market) -> Fixed {
    let (wanted, land) = subsistence_and_land(cat, r, m);
    wanted.min(land)
}

/// Whether the jobless here want more staples than the open land the farms leave free can
/// grow: they are short of land, and clear it if they can (`monthly::pioneers`).
pub fn short_of_land(cat: &Catalogue, r: &Region, m: &Market) -> bool {
    let (wanted, land) = subsistence_and_land(cat, r, m);
    wanted > land
}

/// Staples the jobless would grow with land enough, and what the free open land can grow.
fn subsistence_and_land(cat: &Catalogue, r: &Region, m: &Market) -> (Fixed, Fixed) {
    let agri = m.tech_tfp.first().copied().unwrap_or(Fixed::ONE);
    let gain = Fixed::ONE + (agri - Fixed::ONE).max(Fixed::ZERO) * milli(500);
    let wanted = r.subsistence.mul_ratio(unemployed_people(r), 1000) * gain;
    if cat.staple_farms.is_empty() {
        return (wanted, Fixed::MAX);
    }
    let mut free = Fixed::ZERO;
    for &(farm, kind, per_level) in &cat.staple_farms {
        let land: i64 = r.land.iter().filter(|l| l.0 == farm).map(|l| l.1).sum();
        let used = r
            .plants
            .iter()
            .find(|p| p.kind == kind)
            .map_or(0, |p| p.levels);
        free += per_level * (land - used).max(0);
    }
    (wanted, free * gain)
}

fn unemployed_people(r: &Region) -> i64 {
    // Dependents of the jobless count too: they eat from the same plot.
    r.pops
        .iter()
        .map(|p| {
            let w = workers(p.size);
            if w == 0 {
                0
            } else {
                p.size.mul_div_floor(w - p.employed.min(w), w)
            }
        })
        .sum()
}

trait MulDivFloor {
    fn mul_div_floor(self, num: i64, den: i64) -> i64;
}

impl MulDivFloor for i64 {
    fn mul_div_floor(self, num: i64, den: i64) -> i64 {
        ((i128::from(self) * i128::from(num)) / i128::from(den)) as i64
    }
}

/// With `SKIP_IDLE`, a plant running at scale zero (no levels left) is passed over: everything
/// it would add is zero. The tests run both ways.
fn region_orders<const SKIP_IDLE: bool>(
    cat: &Catalogue,
    econ: &Economy,
    r: &Region,
    exists: &[bool],
    national: &[bool],
) -> RegionOrders {
    let g = cat.goods.len();
    let m = &econ.markets[r.market as usize];
    let view = View::last_week(cat, m, r, national);
    let mut demand = vec![Fixed::ZERO; g];
    let mut supply = vec![Fixed::ZERO; g];
    for p in &r.plants {
        let (_, method, scale) = plant_run(cat, r, m, p);
        if SKIP_IDLE && scale == Fixed::ZERO {
            continue;
        }
        for (gi, q) in &method.inputs {
            demand[*gi as usize] += *q * scale;
        }
        let out_scale = scale * p.input_ratio;
        for (gi, q) in &method.outputs {
            supply[*gi as usize] += *q * out_scale;
        }
    }
    let staple = subsistence_food(cat, r, m);
    if let Some(grain) = cat.grain {
        supply[grain as usize] += peasant_surplus(m, staple);
    }
    let total = r.population().max(1);
    let mixes = NeedMixes::new(cat, &view.price, exists);
    let pops = r
        .pops
        .iter()
        .enumerate()
        .map(|(pi, pop)| {
            if pop.size <= 0 {
                return None;
            }
            // What people do not save they mean to spend; a little of the buffer helps when
            // income falls.
            let budget = (pop.income * (Fixed::ONE - SAVE_RATE[pi])
                + pop.savings.mul_ratio(1, 100))
            .min(pop.savings);
            let own_food = staple.mul_ratio(pop.size, total);
            let plan = plan_pop(
                cat,
                &mixes,
                pop.size,
                budget.max(Fixed::ZERO),
                own_food,
                Some(super::migration::is_urban(pi)),
            );
            for (gi, q) in &plan.orders {
                demand[*gi as usize] += *q;
            }
            Some(plan)
        })
        .collect();
    RegionOrders {
        demand,
        supply,
        pops,
    }
}

#[allow(clippy::too_many_arguments)]
fn clear_market(
    cat: &Catalogue,
    mut m: Market,
    demand: &[Fixed],
    supply: &[Fixed],
    trade: &[Fixed],
    exists: &[bool],
    upkeep: &[(GoodIx, Fixed)],
    military: Option<&[(GoodIx, Fixed)]>,
) -> (Market, Vec<(GoodIx, Fixed)>) {
    let g = cat.goods.len();
    // The state's own purchases and construction join the demand.
    let mut demand = demand.to_vec();
    let orders = state_orders(cat, &m, exists, military);
    for (gi, q) in &orders {
        demand[*gi as usize] += *q;
    }
    // The state buys everything its schools and hospitals make.
    for &gi in &cat.public_goods {
        demand[gi as usize] += supply[gi as usize];
    }
    if let Some(cg) = cat.construction {
        demand[cg as usize] += super::construction::queue_want(&m);
    }
    for (gi, &left) in m.construction.iter().enumerate() {
        if left > Fixed::ZERO {
            demand[gi] += construction_this_week(left);
        }
    }
    // Capital that found no project still turns over: maintenance, housing, roads.
    for &(gi, q) in upkeep {
        demand[gi as usize] += q;
    }
    for gi in 0..g {
        let mut s = supply[gi];
        let mut d = demand[gi];
        // Imports (positive) or exports (negative) from the world pool.
        let net = trade[gi];
        if net > Fixed::ZERO {
            s += net;
        } else {
            d -= net;
        }
        m.net_import[gi] = net;
        m.supply[gi] = s;
        m.demand[gi] = d;
        let traded = s.min(d);
        m.buy_ratio[gi] = fill(traded, d);
        m.sell_ratio[gi] = fill(traded, s);
        m.price[gi] = PRICES.next(m.price[gi], cat.goods[gi].base_price, s, d, || {
            (d - s) / s.min(d).max(Fixed::EPSILON)
        });
    }
    (m, orders)
}

/// With `SKIP_IDLE`, a plant at scale zero skips the sums it would add zero to (cost, revenue,
/// output, pay); its input ratio and utilization still move as for any plant, so it restarts
/// where it would have. The tests run both ways. `abroad`: the foreign owners of the region's
/// plants, as (kind, owner market, share of the plant's levels; `ForeignState::owners_in`).
fn settle_region<const SKIP_IDLE: bool, const ABROAD: bool>(
    cat: &Catalogue,
    m: &Market,
    r: &Region,
    orders: &RegionOrders,
    wage: Fixed,
    v: &View,
    abroad: &[(u16, u16, Fixed)],
) -> RegionSettlement {
    let laws = m.laws.of(r);
    let mut wages = [Fixed::ZERO; P];
    let mut owners_industry = Fixed::ZERO;
    let mut owners_land = Fixed::ZERO;
    // Farm profit the land tenure law hands from the landowners to the tenants.
    let mut tenants = Fixed::ZERO;
    let mut pool = Fixed::ZERO;
    let mut gdp = Fixed::ZERO;
    let mut remit: Vec<(u16, Fixed, Fixed)> = Vec::new();
    let mut plants = Vec::with_capacity(r.plants.len());
    for p in &r.plants {
        let (b, method, scale) = plant_run(cat, r, m, p);
        let idle = SKIP_IDLE && scale == Fixed::ZERO;
        let mut cost = Fixed::ZERO;
        let mut cost_base = Fixed::ZERO;
        // Missing inputs cut output by their share in the value of the output (a linear
        // Cobb–Douglas): a shortage of explosives in a mine costs it a tenth, a shortage of ore
        // in a steel mill stops it.
        let output_value = method.output_value;
        let mut loss = Fixed::ZERO;
        for (gi, q) in &method.inputs {
            let base = cat.goods[*gi as usize].base_price;
            if !idle {
                let bought = *q * scale * v.buy[*gi as usize];
                cost += bought * v.price[*gi as usize];
                cost_base += bought * base;
            }
            if output_value > Fixed::ZERO {
                let mut share = fill(*q * base, output_value);
                // Freight: what the plant cannot ship it does not sell.
                if Some(*gi) == cat.freight && method.freight > Fixed::ZERO {
                    share = share.max(super::local::ACCESS_WEIGHT);
                }
                loss += share * (Fixed::ONE - v.buy[*gi as usize]);
            }
        }
        let input_ratio = (Fixed::ONE - loss).max(Fixed::ZERO);
        let out_scale = scale * p.input_ratio;
        let mut revenue = Fixed::ZERO;
        let mut revenue_base = Fixed::ZERO;
        for (gi, q) in method.outputs.iter().filter(|_| !idle) {
            let sold = *q * out_scale * v.sell[*gi as usize];
            revenue += sold * v.price[*gi as usize];
            revenue_base += sold * cat.goods[*gi as usize].base_price;
        }
        gdp += revenue_base - cost_base;
        let value_added = revenue - cost;
        // Public services have no owners: all their value added is pay.
        let wage_bill = if b.public || b.informal {
            value_added.max(Fixed::ZERO)
        } else {
            value_added.max(Fixed::ZERO) * b.labor_share
        };
        let profit = value_added - wage_bill;
        let weight_total = method.job_weight;
        if !idle && weight_total > Fixed::ZERO {
            for (pi, n) in &method.jobs {
                wages[*pi as usize] += wage_bill * (WAGE_WEIGHT[*pi as usize] * *n) / weight_total;
            }
        }
        if profit > Fixed::ZERO {
            // The foreign owners' share leaves for their funds, less what they reinvest here,
            // which stays in this country's fund as their claim; the rest splits as at home
            // (`docs/foreign-capital.pt-BR.md`, section 8).
            let mut kept = profit;
            for &(_, owner, share) in abroad.iter().filter(|e| ABROAD && e.0 == p.kind) {
                let theirs = crate::foreign_capital::remittance(profit, share, Fixed::ZERO);
                let out = crate::foreign_capital::remittance(
                    profit,
                    share,
                    crate::foreign_capital::REINVESTED,
                );
                kept -= theirs;
                pool += theirs - out;
                match remit.iter_mut().find(|e| e.0 == owner) {
                    Some(e) => {
                        e.1 += out;
                        e.2 += theirs - out;
                    }
                    None => remit.push((owner, out, theirs - out)),
                }
            }
            let profit = kept;
            let to_owners = profit * PAYOUT;
            if b.agriculture {
                // Free tenancy gives the landowners the owners' payout; harsher tenure takes more
                // (out of what would be saved), reform hands the cut to the tenants.
                let rent = (to_owners * laws.landowner_rent).min(profit);
                owners_land += rent;
                tenants += (to_owners - rent).max(Fixed::ZERO);
                pool += profit - to_owners.max(rent);
            } else {
                owners_industry += to_owners;
                pool += profit - to_owners;
            }
        } else {
            pool += profit;
        }
        // Plants follow demand for their main output: they ramp up while everything sells and
        // ease off in step with what goes unsold.
        let sold_share = method
            .outputs
            .first()
            .map_or(Fixed::ONE, |(gi, _)| v.sell[*gi as usize]);
        let utilization = if sold_share >= milli(990) {
            (p.utilization + milli(50)).min(Fixed::ONE)
        } else {
            (p.utilization * (sold_share + milli(30))).clamp(milli(50), Fixed::ONE)
        };
        plants.push((input_ratio, profit, value_added, utilization));
    }
    // Owners absent from the state: their dividends go to the country's savings.
    // State firms: the ownership law's share of the owners' payout goes to the state, which
    // books part of it as revenue and reinvests the rest.
    let state_take = owners_industry * laws.state_ownership;
    let state_revenue = state_take * laws.state_profit_to_treasury;
    owners_industry -= state_take;
    pool += state_take - state_revenue;
    if r.pops[prof::CAPITALIST].size > 0 {
        wages[prof::CAPITALIST] += owners_industry;
    } else {
        pool += owners_industry;
    }
    if r.pops[prof::LANDOWNER].size > 0 {
        wages[prof::LANDOWNER] += owners_land;
    } else {
        pool += owners_land;
    }
    if r.pops[prof::PEASANT].size > 0 {
        wages[prof::PEASANT] += tenants;
    } else {
        pool += tenants;
    }
    // Smallholders sell what they grow beyond their own table.
    let staple = subsistence_food(cat, r, m);
    if let Some(grain) = cat.grain {
        let surplus = peasant_surplus(m, staple);
        let sold = surplus * v.sell[grain as usize] * v.price[grain as usize];
        wages[prof::PEASANT] += sold;
        gdp += sold;
    }
    // The state pays its servants from the treasury, at the country's going wage.
    // A treasury that could not borrow pays in arrears (`public::pay_share`).
    let mut payroll = Fixed::ZERO;
    let paid = super::public::pay_share(m);
    // Public pay over the private wage: the rule's, or the one the government fixed (D2).
    let public_pay = m.levers.public_pay();
    for pi in [prof::CIVIL_SERVANT, prof::CLERGY, prof::SOLDIER] {
        let pay = wage * WAGE_WEIGHT[pi] * public_pay * workers(r.pops[pi].size) * paid;
        wages[pi] += pay;
        payroll += pay;
    }
    // The state's own output is counted at cost, as national accounts do: what it pays its
    // servants, soldiers and clergy.
    gdp += payroll;
    let mut taxes = state_revenue;
    let mut indirect = Fixed::ZERO;
    let mut pops = Vec::with_capacity(P);
    for (pi, pop) in r.pops.iter().enumerate() {
        let gross = wages[pi];
        let tax = gross * m.budget.tax_rates[CLASS_OF[pi]] * laws.tax_collection;
        taxes += tax;
        let income = gross - tax;
        let unplaced = orders.pops[pi].as_ref().map_or(Fixed::ZERO, |p| p.unplaced);
        let (spent, satisfaction) = match &orders.pops[pi] {
            Some(plan) => {
                let mut spent = Fixed::ZERO;
                let mut fulfilled = Fixed::ZERO;
                let mut wanted = Fixed::ZERO;
                for (gi, q) in &plan.orders {
                    let money = *q * v.price[*gi as usize];
                    wanted += money;
                    let got = money * v.buy[*gi as usize];
                    spent += got;
                    fulfilled += got;
                }
                let ratio = fill(fulfilled, wanted);
                let mut sat = [Fixed::ZERO; 4];
                for t in 0..4 {
                    sat[t] = if plan.tier_cost[t] <= Fixed::ZERO {
                        Fixed::ONE
                    } else {
                        fill(plan.tier_planned[t] * ratio, plan.tier_cost[t])
                    };
                }
                sat[0] = (sat[0] + plan.in_kind).min(Fixed::ONE);
                (spent.min(pop.savings.max(Fixed::ZERO)), sat)
            }
            None => (Fixed::ZERO, [Fixed::ZERO; 4]),
        };
        // Savings go to the country's investment pool; the buffer keeps what was meant for
        // consumption but not spent.
        let saved = income * SAVE_RATE[pi];
        // Tax on consumption, out of what is left after buying and saving.
        let before_tax = pop.savings - spent + income - saved;
        let consumption_tax = (spent * m.budget.indirect_rate).min(before_tax.max(Fixed::ZERO));
        indirect += consumption_tax;
        let buffer = before_tax - consumption_tax;
        let invested = unplaced.min(buffer).max(Fixed::ZERO);
        pool += saved + invested;
        pops.push((income, buffer - invested, satisfaction));
    }
    // Food the jobless grow for themselves counts in GDP at the base price of grain.
    if let Some(grain) = cat.grain {
        gdp += staple * cat.goods[grain as usize].base_price;
    }
    RegionSettlement {
        pops,
        plants,
        taxes: taxes + indirect,
        indirect,
        firms: state_revenue,
        payroll,
        pool,
        gdp,
        remit,
    }
}

/// The world pool (trade by pair switched off): each market offers part of its surplus and
/// seeks part of its shortage; what is traded is the smaller side, shared pro rata.
fn world_pool(cat: &Catalogue, supply: &[Vec<Fixed>], demand: &[Vec<Fixed>]) -> Vec<Vec<Fixed>> {
    let g = cat.goods.len();
    let nm = supply.len();
    let mut trade = vec![vec![Fixed::ZERO; g]; nm];
    for gi in 0..g {
        if !cat.goods[gi].tradable {
            continue;
        }
        let mut offered = Fixed::ZERO;
        let mut sought = Fixed::ZERO;
        for mi in 0..nm {
            let gap = supply[mi][gi] - demand[mi][gi];
            if gap > Fixed::ZERO {
                offered += gap * TRADE_REACH;
            } else {
                sought -= gap * TRADE_REACH;
            }
        }
        let traded = offered.min(sought);
        if traded <= Fixed::ZERO {
            continue;
        }
        for mi in 0..nm {
            let gap = supply[mi][gi] - demand[mi][gi];
            trade[mi][gi] = if gap > Fixed::ZERO {
                -(gap * TRADE_REACH * traded / offered)
            } else {
                -gap * TRADE_REACH * traded / sought
            };
        }
    }
    trade
}

/// What trade by pair hands the week: the trade of each market by good, the exports by pair
/// at base prices (seller × markets + buyer) and the month's preferences in its first week.
type PairTrade = (Vec<Vec<Fixed>>, Vec<Fixed>, Option<Vec<Vec<(u16, u16)>>>);

impl Week {
    /// Trade by pair (`econ::trade`, docs/trade.pt-BR.md): each market offers part of its
    /// surplus and seeks part of its shortage, and every buyer splits what it seeks among the
    /// sellers by what they offer and how much it leans toward each.
    fn trade_by_pair(
        &self,
        ctx: &TickCtx,
        world: &GameWorld,
        supply: &[Vec<Fixed>],
        demand: &[Vec<Fixed>],
    ) -> PairTrade {
        let cat = &*self.cat;
        let econ = &world.econ;
        let g = cat.goods.len();
        let nm = econ.markets.len();
        let reach = Fixed::from_f64_lossy(f64::from(cat.trade.reach));
        let mut trade = vec![vec![Fixed::ZERO; g]; nm];
        let mut trade_value = vec![Fixed::ZERO; nm * nm];
        // Preferences move once a month, in its first week, part of the way toward the cost
        // of arrival (`econ::trade::step`); the clearing uses the month's.
        let geo = self.geo.as_deref();
        let groups = crate::economy::BUILDING_GROUPS.len();
        let date = ctx.date.to_string();
        let trade_pref = (ctx.date.day <= 7 || econ.trade.pref.is_empty()).then(|| {
            let trade_ix = crate::research::EFFECTS
                .iter()
                .position(|e| *e == "trade")
                .unwrap_or(0);
            let effect: Vec<Fixed> = (0..nm)
                .map(|c| {
                    world
                        .research
                        .effects
                        .get(c)
                        .map_or(Fixed::ZERO, |e| e[trade_ix])
                })
                .collect();
            let gdp: Vec<Fixed> = econ.markets.iter().map(|m| m.gdp_last_year).collect();
            let transport: Vec<Fixed> = econ
                .markets
                .iter()
                .map(|m| {
                    cat.freight.map_or(Fixed::ONE, |f| {
                        m.price[f as usize] / cat.goods[f as usize].base_price
                    })
                })
                .collect();
            let factors = super::trade::FreightFactors::of(&effect, &gdp, &transport);
            let eq = super::trade::equilibrium(cat, geo, &econ.markets, &factors, &date);
            let lambda = Fixed::from_f64_lossy(f64::from(cat.trade.monthly_step));
            super::trade::step(&econ.trade.pref, &eq, lambda)
        });
        let pref_table = super::trade::dense(trade_pref.as_deref().unwrap_or(&econ.trade.pref), nm);
        for gi in 0..g {
            if !cat.goods[gi].tradable {
                continue;
            }
            let (mut sellers, mut offered, mut buyers, mut sought) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for mi in 0..nm {
                let gap = supply[mi][gi] - demand[mi][gi];
                if gap > Fixed::ZERO {
                    sellers.push(mi);
                    offered.push(gap * reach);
                } else if gap < Fixed::ZERO {
                    buyers.push(mi);
                    sought.push(-gap * reach);
                }
            }
            if sellers.is_empty() || buyers.is_empty() {
                continue;
            }
            // How much each buyer leans toward each seller: the month's preference in the
            // good's group; a buyer with none toward any seller of the good leans equally.
            let base = cat.goods[gi].base_price;
            let k = cat.good_group[gi].map(usize::from);
            let mut pref: Vec<Vec<Fixed>> = sellers
                .iter()
                .map(|&sm| {
                    buyers
                        .iter()
                        .map(|&bm| {
                            // A closed pair trades nothing from the week it closes.
                            if geo.is_some_and(|g| g.closed(sm, bm, &date)) {
                                return Fixed::ZERO;
                            }
                            k.and_then(|k| pref_table.get(bm * groups + k))
                                .map_or(Fixed::ONE, |row| row[sm])
                        })
                        .collect()
                })
                .collect();
            for b in 0..buyers.len() {
                if pref.iter().all(|row| row[b] <= Fixed::ZERO) {
                    for (s, row) in pref.iter_mut().enumerate() {
                        if !geo.is_some_and(|g| g.closed(sellers[s], buyers[b], &date)) {
                            row[b] = Fixed::ONE;
                        }
                    }
                }
            }
            super::trade::normalize(&mut pref);
            let flow = super::trade::clear_good(&offered, &sought, |s, b| pref[s][b]);
            let (exports, imports) = super::trade::totals(&flow, buyers.len());
            for (s, row) in flow.iter().enumerate() {
                for (b, q) in row.iter().enumerate() {
                    if *q > Fixed::ZERO {
                        trade_value[sellers[s] * nm + buyers[b]] += *q * base;
                    }
                }
            }
            for (k, &mi) in sellers.iter().enumerate() {
                trade[mi][gi] = -exports[k];
            }
            for (k, &mi) in buyers.iter().enumerate() {
                trade[mi][gi] = imports[k];
            }
        }
        (trade, trade_value, trade_pref)
    }
}

impl System<GameWorld> for Week {
    type Changes = WeekChanges;

    fn name(&self) -> &'static str {
        "economy-week"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Weekly(Weekday::Monday)
    }

    fn access(&self) -> Access {
        Access::new()
            .read(ECON)
            .read(crate::war::WAR)
            .write(ECON)
            .read(crate::research_system::RESEARCH)
            .read(crate::foreign_stakes::FOREIGN)
            .write(crate::foreign_stakes::FOREIGN)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> WeekChanges {
        let cat = &*self.cat;
        let econ = &world.econ;
        let unlocks = Unlocks::new(cat, ctx.date.year, self.tree.as_deref(), &world.research);
        let exists = &unlocks.goods;
        let national = super::local::national(
            cat,
            self.tree.as_deref(),
            &world.research,
            econ.markets.len(),
        );
        let orders: Vec<RegionOrders> = econ
            .regions
            .par_iter()
            .map(|r| region_orders::<true>(cat, econ, r, exists, &national[r.market as usize]))
            .collect();
        let g = cat.goods.len();
        let mut demand = vec![vec![Fixed::ZERO; g]; econ.markets.len()];
        let mut supply = vec![vec![Fixed::ZERO; g]; econ.markets.len()];
        for (r, o) in econ.regions.iter().zip(&orders) {
            let mi = r.market as usize;
            for gi in 0..g {
                demand[mi][gi] += o.demand[gi];
                supply[mi][gi] += o.supply[gi];
            }
        }
        // Local goods clear state by state first; the country-wide buyers get what is left.
        let region_demand: Vec<&[Fixed]> = orders.iter().map(|o| &o.demand[..]).collect();
        let region_supply: Vec<&[Fixed]> = orders.iter().map(|o| &o.supply[..]).collect();
        let mut cleared = super::local::clear(cat, econ, &region_demand, &region_supply, &national);
        for (ri, (r, o)) in econ.regions.iter().zip(&orders).enumerate() {
            let mi = r.market as usize;
            for (li, &lg) in cat.local.iter().enumerate() {
                if national[mi][li] {
                    continue;
                }
                let lg = lg as usize;
                demand[mi][lg] -= o.demand[lg];
                supply[mi][lg] += cleared.leftover[ri][li] - o.supply[lg];
            }
        }
        // Trade between markets: by pair when switched on, else the world pool.
        let (trade, trade_value, trade_pref) = if self.by_pair {
            self.trade_by_pair(ctx, world, &supply, &demand)
        } else {
            (world_pool(cat, &supply, &demand), Vec::new(), None)
        };
        let upkeep: Vec<_> = econ.markets.iter().map(|m| upkeep_orders(cat, m)).collect();
        let (mut markets, state_orders): (Vec<Market>, Vec<_>) = econ
            .markets
            .iter()
            .enumerate()
            .map(|(mi, m)| {
                let (d, s, t) = (&demand[mi], &supply[mi], &trade[mi]);
                let military = world.war.orders(mi);
                clear_market(cat, m.clone(), d, s, t, exists, &upkeep[mi], military)
            })
            .unzip();
        super::local::finish(
            cat,
            econ,
            &mut markets,
            &mut cleared,
            &region_demand,
            &region_supply,
            &national,
        );
        let local = cleared.regions;
        let wages = super::monthly::market_wages(econ, self.wage_floor);
        // Capital across borders is tested once: off, this is the loop of before it.
        let regions: Vec<RegionSettlement> = if self.foreign_flow {
            let (foreign, year) = (&world.foreign, ctx.date.year);
            econ.regions
                .par_iter()
                .enumerate()
                .zip(orders.par_iter())
                .zip(local.par_iter())
                .map(|(((ri, r), o), l)| {
                    let mi = r.market as usize;
                    let m = &markets[mi];
                    // The foreign owners of the state's plants, in the state's own task: empty
                    // (no allocation) where nobody abroad owns a plant.
                    let a = foreign.owners_in(ri as u32, year, |k| {
                        r.plants
                            .iter()
                            .find(|p| p.kind == k)
                            .map_or(0, |p| p.levels)
                    });
                    let v = View::new(cat, m, l, &[]);
                    settle_region::<true, true>(cat, m, r, o, wages[mi], &v, &a)
                })
                .collect()
        } else {
            econ.regions
                .par_iter()
                .zip(orders.par_iter())
                .zip(local.par_iter())
                .map(|((r, o), l)| {
                    let mi = r.market as usize;
                    let m = &markets[mi];
                    let v = View::new(cat, m, l, &[]);
                    settle_region::<true, false>(cat, m, r, o, wages[mi], &v, &[])
                })
                .collect()
        };
        // World price: average of the market prices weighted by volume.
        let world_price = (0..g)
            .map(|gi| {
                let mut num = Fixed::ZERO;
                let mut den = Fixed::ZERO;
                for m in &markets {
                    let v = m.supply[gi].min(m.demand[gi]);
                    num += m.price[gi] * v;
                    den += v;
                }
                if den > Fixed::ZERO {
                    num / den
                } else {
                    cat.goods[gi].base_price
                }
            })
            .collect();
        WeekChanges {
            markets,
            state_orders,
            regions,
            world_price,
            local,
            upkeep,
            trade_value,
            trade_pref,
        }
    }

    fn apply(&self, ctx: &TickCtx, changes: WeekChanges, world: &mut GameWorld) {
        let cat = &*self.cat;
        let econ = &mut world.econ;
        // Off, the trade state stays empty and out of the world's hash.
        if self.by_pair {
            let nm = econ.markets.len();
            econ.trade.fit(nm);
            if let Some(p) = changes.trade_pref {
                econ.trade.pref = p;
            }
            for (k, v) in changes.trade_value.iter().enumerate() {
                if *v > Fixed::ZERO {
                    econ.trade.add(k / nm, k % nm, *v);
                }
            }
        }
        let mut markets = changes.markets;
        // Trade margins: importers collected more than exporters were paid. Each market keeps
        // its balance, minus its share of the world total, so no money is created.
        let mut balance = vec![Fixed::ZERO; markets.len()];
        let mut volume = vec![Fixed::ZERO; markets.len()];
        let construction_good = cat.construction;
        let mut done = Vec::new();
        for (mi, m) in markets.iter().enumerate() {
            for gi in 0..cat.goods.len() {
                let n = m.net_import[gi];
                if n > Fixed::ZERO {
                    balance[mi] += n * m.sell_ratio[gi] * m.price[gi];
                    volume[mi] += n * m.price[gi];
                } else if n < Fixed::ZERO {
                    balance[mi] += n * m.buy_ratio[gi] * m.price[gi];
                    volume[mi] -= n * m.price[gi];
                }
            }
        }
        let total_balance: Fixed = balance.iter().copied().sum();
        let total_volume: Fixed = volume.iter().copied().sum();
        for (mi, m) in markets.iter_mut().enumerate() {
            let share = if total_volume > Fixed::ZERO {
                total_balance * volume[mi] / total_volume
            } else {
                Fixed::ZERO
            };
            m.pool += balance[mi] - share;
            // The state pays for the goods it got and for what its schools and hospitals made.
            let goods: Fixed = changes.state_orders[mi]
                .iter()
                .map(|(g, q)| *q * m.buy_ratio[*g as usize] * m.price[*g as usize])
                .sum();
            // The health and education laws say how much of it the treasury pays; the users pay
            // the rest out of their savings.
            let shares = &m.laws.effects[0];
            let can = super::public::pay_share(m);
            let (services, fees) = cat
                .public_goods
                .iter()
                .map(|&g| g as usize)
                .map(|g| {
                    let cost = m.supply[g] * m.sell_ratio[g] * m.price[g];
                    let paid = cost * shares.public_share(g) * can;
                    (paid, cost - paid)
                })
                .fold((Fixed::ZERO, Fixed::ZERO), |(a, b), (p, f)| (a + p, b + f));
            m.treasury -= goods + services;
            m.pool -= fees;
            m.budget.goods = goods;
            m.budget.services = services;
            m.budget.revenue = Fixed::ZERO;
            m.budget.revenue_indirect = Fixed::ZERO;
            m.budget.revenue_firms = Fixed::ZERO;
            m.budget.payroll = Fixed::ZERO;
            // The building trade delivers points to the queue of works.
            m.budget.works = Fixed::ZERO;
            if let Some(cg) = construction_good {
                let cg = cg as usize;
                let d = super::construction::deliver(m, m.buy_ratio[cg], m.price[cg]);
                m.pool -= d.private_cost;
                m.treasury -= d.public_cost;
                m.budget.works = d.public_cost;
                done.extend(d.done);
            }
            for gi in 0..cat.goods.len() {
                let left = m.construction[gi];
                if left > Fixed::ZERO {
                    let bought = construction_this_week(left) * m.buy_ratio[gi];
                    m.construction[gi] = (left - bought).max(Fixed::ZERO);
                    m.pool -= bought * m.price[gi];
                }
            }
            for &(gi, q) in &changes.upkeep[mi] {
                m.pool -= q * m.buy_ratio[gi as usize] * m.price[gi as usize];
            }
        }
        // How busy each market was, for its central bank (`money::week`).
        let activity = super::money::activity(cat, &econ.regions, &markets);
        for (r, l) in econ.regions.iter_mut().zip(changes.local) {
            r.local = l;
        }
        let mut remitted: Vec<(u16, u16, Fixed, Fixed)> = Vec::new();
        for (r, s) in econ.regions.iter_mut().zip(changes.regions) {
            let m = &mut markets[r.market as usize];
            m.treasury += s.taxes - s.payroll;
            m.budget.revenue += s.taxes;
            m.budget.revenue_indirect += s.indirect;
            m.budget.revenue_firms += s.firms;
            m.budget.payroll += s.payroll;
            m.pool += s.pool;
            m.gdp_running += s.gdp;
            if self.foreign_flow {
                remitted.extend(
                    s.remit
                        .into_iter()
                        .map(|(o, out, kept)| (o, r.market, out, kept)),
                );
            }
            for (pop, (income, savings, sat)) in r.pops.iter_mut().zip(s.pops) {
                pop.income = income;
                pop.savings = savings;
                pop.satisfaction = sat;
            }
            for (plant, (input_ratio, profit, _va, utilization)) in
                r.plants.iter_mut().zip(s.plants)
            {
                plant.input_ratio = input_ratio;
                plant.utilization = utilization;
                plant.profit = profit;
                plant.losing_weeks = if profit < Fixed::ZERO {
                    plant.losing_weeks.saturating_add(1)
                } else {
                    0
                };
            }
        }
        for (owner, host, out, kept) in remitted {
            markets[owner as usize].pool += out;
            if kept > Fixed::ZERO {
                world.foreign.retain(owner, host, kept);
            }
        }
        for m in &mut markets {
            let b = &mut m.budget;
            b.revenue_avg += (b.revenue - b.revenue_avg).mul_ratio(1, public::REVENUE_SMOOTHING);
        }
        econ.markets = markets;
        econ.world_price = changes.world_price;
        // Prices, central banks and bonds: the treasury's gap becomes debt.
        super::money::week(econ, &activity, ctx.date.year);
        // Finished works start running; one paid from abroad becomes its owner's stake.
        for p in done {
            let r = &mut econ.regions[p.region as usize];
            let plant = r.plant_mut(p.kind);
            if plant.levels == 0 {
                plant.method = p.method;
            }
            plant.levels += 1;
            if self.foreign_flow && !p.public {
                world.foreign.finished(p.region, p.kind, ctx.date.year);
            }
        }
    }
}

/// The goods a pop of `size` would order with `budget` (for calibration tools).
pub fn plan_pop_public(
    cat: &Catalogue,
    prices: &[Fixed],
    year: i32,
    size: i64,
    budget: Fixed,
    in_kind: Fixed,
) -> Vec<(GoodIx, Fixed)> {
    let mixes = NeedMixes::new(cat, prices, &Unlocks::by_year(cat, year).goods);
    plan_pop(cat, &mixes, size, budget, in_kind, None).orders
}

/// The goods a pop of `size` living in town (`urban`) or in the countryside would order.
pub fn plan_pop_where(
    cat: &Catalogue,
    prices: &[Fixed],
    year: i32,
    size: i64,
    budget: Fixed,
    in_kind: Fixed,
    urban: bool,
) -> Vec<(GoodIx, Fixed)> {
    let mixes = NeedMixes::new(cat, prices, &Unlocks::by_year(cat, year).goods);
    plan_pop(cat, &mixes, size, budget, in_kind, Some(urban)).orders
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Skipping plants at scale zero changes nothing: on the 1925 world, with one plant per state
    /// idled (no levels), then brought back to one level after a week idle, the orders and the
    /// settlement of every state are the same with and without the skip.
    #[test]
    fn idle_plants_skip_only_zeros_and_restart_the_same() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let data = crate::data::GameData::load(&root).unwrap();
        let cat = &*data.catalogue;
        let mut world = data.new_world();
        let unlocks = Unlocks::new(cat, 1925, data.tree.as_deref(), &world.research);
        let national = super::super::local::national(
            cat,
            data.tree.as_deref(),
            &world.research,
            world.econ.markets.len(),
        );
        let compare = |econ: &Economy| {
            let mut idle = 0;
            let mut settled = Vec::new();
            for (ri, r) in econ.regions.iter().enumerate() {
                let nat = &national[r.market as usize];
                let m = &econ.markets[r.market as usize];
                let skip = region_orders::<true>(cat, econ, r, &unlocks.goods, nat);
                let full = region_orders::<false>(cat, econ, r, &unlocks.goods, nat);
                assert_eq!(skip, full, "orders of state {ri}");
                // Short of every good, so that an input's shortfall moves the input ratio.
                let mut v = View::last_week(cat, m, r, nat);
                v.buy.iter_mut().for_each(|b| *b = milli(700));
                let wage = Fixed::from_int(3);
                let s = settle_region::<true, false>(cat, m, r, &skip, wage, &v, &[]);
                let f = settle_region::<false, false>(cat, m, r, &full, wage, &v, &[]);
                assert_eq!(s, f, "settlement of state {ri}");
                idle += r
                    .plants
                    .iter()
                    .filter(|p| plant_run(cat, r, m, p).2 == Fixed::ZERO)
                    .count();
                settled.push(s);
            }
            (idle, settled)
        };
        // A week idle: the first plant of every state loses its levels, at half use (so the
        // utilization the idle week moves is visible).
        for r in &mut world.econ.regions {
            if let Some(p) = r.plants.first_mut() {
                p.levels = 0;
                p.utilization = milli(500);
            }
        }
        let (idle, settled) = compare(&world.econ);
        assert!(idle >= world.econ.regions.len() / 2, "{idle} idle plants");
        // The week's input ratio and utilization go back to the plants, as the week's apply does,
        // and the idled plants get a level again.
        for (r, s) in world.econ.regions.iter_mut().zip(&settled) {
            for (p, &(input_ratio, _, _, utilization)) in r.plants.iter_mut().zip(&s.plants) {
                p.input_ratio = input_ratio;
                p.utilization = utilization;
            }
            if let Some(p) = r.plants.first_mut() {
                p.levels = 1;
            }
        }
        compare(&world.econ);
    }
}
