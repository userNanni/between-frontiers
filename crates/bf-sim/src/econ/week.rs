//! The weekly market: orders, clearing with world trade, settlement.

use std::sync::Arc;

use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};
use laplace_schelling::{Fixed, Weekday};
use rayon::prelude::*;

use super::catalogue::{Catalogue, GoodIx};
use super::prof::{self, P};
use super::state::{Economy, Market, Region};
use super::{SAVE_RATE, TAX_RATE, WAGE_WEIGHT, fill, labor_share, milli, workers};
use crate::game::GameWorld;

pub const ECON: ResourceId = ResourceId(10);

/// Price moves toward base × (1 + 0.75 × (D − S) / min(D, S)), held within 25%–400% of base.
const PRICE_SPEED: Fixed = milli(250);
const PRICE_FLOOR: Fixed = milli(250);
const PRICE_CEILING: Fixed = milli(4000);
/// Share of a market's surplus offered abroad, and of its shortage sought abroad, each week:
/// below 1 so that local prices still carry part of the signal (distance, ports, habits).
const TRADE_REACH: Fixed = milli(850);
/// The state spends this share of its treasury each week: 60% on servants, 40% on goods.
pub(crate) const STATE_SPENDING: Fixed = milli(100);
/// What the state buys with its goods budget.
pub(crate) const STATE_BASKET: [(&str, i64); 9] = [
    ("services", 300),
    ("transportation", 100),
    ("paper", 100),
    ("medicines", 100),
    ("small_arms", 100),
    ("ammunition", 100),
    ("artillery", 50),
    ("warships", 100),
    ("military_aircraft", 50),
];
/// The money autopilot aims at plants running at 95% and prices within 10% of base.
const MONEY_TARGET_UTILIZATION: Fixed = milli(950);
const MONEY_PRICE_CEILING: Fixed = milli(1100);
const MONEY_EASING: Fixed = milli(500);
const MONEY_TIGHTENING: Fixed = milli(300);

/// Share of the investment pool spent each week on upkeep and housing.
pub(crate) const UPKEEP_SHARE: Fixed = milli(30);
/// Investment buys mostly the work of building (services) and some materials and machines.
pub(crate) const UPKEEP_BASKET: [(&str, i64); 6] = [
    ("services", 600),
    ("timber", 120),
    ("cement", 30),
    ("steel", 30),
    ("machinery", 150),
    ("tools", 70),
];

/// What the pool buys for upkeep this week, computed from the market before clearing.
fn upkeep_orders(cat: &Catalogue, m: &Market) -> Vec<(GoodIx, Fixed)> {
    let money = m.pool.max(Fixed::ZERO) * UPKEEP_SHARE;
    UPKEEP_BASKET
        .iter()
        .filter_map(|(k, share)| {
            let g = cat.good(k)?;
            Some((g, money.mul_ratio(*share, 1000) / m.price[g as usize]))
        })
        .collect()
}

/// Construction goods are bought over about two months.
const CONSTRUCTION_WEEKS: i64 = 8;

pub struct Week {
    pub cat: Arc<Catalogue>,
}

/// What a pop wants to buy this week.
pub(crate) struct PopPlan {
    pub(crate) orders: Vec<(GoodIx, Fixed)>,
    /// Money the full need of each tier would cost, and what the plan covers.
    tier_cost: [Fixed; 4],
    tier_planned: [Fixed; 4],
    /// Share of tier 1 already covered by the pop's own food.
    in_kind: Fixed,
}

struct RegionOrders {
    demand: Vec<Fixed>,
    supply: Vec<Fixed>,
    pops: Vec<Option<PopPlan>>,
}

pub struct RegionSettlement {
    pub pops: Vec<(Fixed, Fixed, [Fixed; 4])>,
    pub plants: Vec<(Fixed, Fixed, Fixed, Fixed)>,
    pub taxes: Fixed,
    pub pool: Fixed,
    pub gdp: Fixed,
}

pub struct WeekChanges {
    markets: Vec<Market>,
    regions: Vec<RegionSettlement>,
    world_price: Vec<Fixed>,
}

/// Cost-weighted choice among the goods that meet a need: shares fall with the square of the
/// price per unit of need, so the cheapest good dominates without a cliff.
fn need_mix(
    cat: &Catalogue,
    need: usize,
    prices: &[Fixed],
    year: i32,
) -> (Vec<(GoodIx, Fixed)>, Fixed) {
    let n = &cat.needs[need];
    let options: Vec<(GoodIx, Fixed, Fixed)> = n
        .goods
        .iter()
        .filter(|(g, w)| cat.goods[*g as usize].available_from <= year && *w > Fixed::ZERO)
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

pub(crate) fn plan_pop(
    cat: &Catalogue,
    prices: &[Fixed],
    year: i32,
    size: i64,
    budget: Fixed,
    staple_in_kind: Fixed,
) -> PopPlan {
    let mut tier_cost = [Fixed::ZERO; 4];
    let mut need_cost = Vec::with_capacity(cat.needs.len());
    let mut in_kind = Fixed::ZERO;
    for (i, n) in cat.needs.iter().enumerate() {
        let mut qty = n.per_capita.mul_ratio(size, 1_000_000);
        if n.key == "staple_food" && qty > Fixed::ZERO {
            in_kind = fill(staple_in_kind, qty);
            qty = (qty - staple_in_kind).max(Fixed::ZERO);
        }
        let (mix, unit_cost) = need_mix(cat, i, prices, year);
        let cost = qty * unit_cost;
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
    // Spare money goes mostly to services and less to goods: the rich of 1925 hired servants,
    // travelled and banked more than they bought steel.
    let weight_of = |key: &str| -> Fixed {
        match key {
            "services" | "savings" => Fixed::ONE,
            "mobility" => milli(800),
            "communication" => milli(600),
            "luxury" => milli(500),
            _ => milli(300),
        }
    };
    let weighted_upper: Fixed = need_cost
        .iter()
        .enumerate()
        .filter(|(i, _)| cat.needs[*i].tier >= 3)
        .map(|(i, (_, _, c))| *c * weight_of(&cat.needs[i].key))
        .sum();
    let extra = if extra > Fixed::ZERO && weighted_upper > Fixed::ZERO {
        // Same money, redistributed by weight.
        (left / weighted_upper).min(Fixed::from_int(40))
    } else {
        Fixed::ZERO
    };
    let mut orders: Vec<(GoodIx, Fixed)> = Vec::new();
    let mut tier_planned = [Fixed::ZERO; 4];
    for (i, (qty, mix, cost)) in need_cost.iter().enumerate() {
        let t = (cat.needs[i].tier - 1) as usize;
        let mut share = tier_share[t];
        if t >= 2 {
            share += extra * weight_of(&cat.needs[i].key);
        }
        tier_planned[t] += *cost * share;
        let q = *qty * share;
        for (g, per_unit) in mix {
            let units = q * *per_unit;
            if units > Fixed::ZERO {
                orders.push((*g, units));
            }
        }
    }
    PopPlan {
        orders,
        tier_cost,
        tier_planned,
        in_kind,
    }
}

/// Units a plant level produces this week from its method, staffing and inputs.
fn plant_scale(levels: i64, staffing: Fixed) -> Fixed {
    Fixed::from_int(levels) * staffing
}

/// Grain smallholders bring to market: a sixth of what they eat, rising to half as farming
/// methods improve.
fn peasant_surplus(r: &Region, staple: Fixed) -> Fixed {
    let gain = if r.tfp_start > Fixed::ZERO {
        (r.tfp / r.tfp_start - Fixed::ONE).clamp(Fixed::ZERO, Fixed::ONE)
    } else {
        Fixed::ZERO
    };
    staple * (milli(160) + gain * milli(340))
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

fn region_orders(cat: &Catalogue, econ: &Economy, r: &Region, year: i32) -> RegionOrders {
    let g = cat.goods.len();
    let m = &econ.markets[r.market as usize];
    let mut demand = vec![Fixed::ZERO; g];
    let mut supply = vec![Fixed::ZERO; g];
    for p in &r.plants {
        let b = &cat.buildings[p.kind as usize];
        let method = &b.methods[p.method as usize];
        let scale = plant_scale(p.levels, p.staffing) * r.tfp_for(b) * p.utilization;
        for (gi, q) in &method.inputs {
            demand[*gi as usize] += *q * scale;
        }
        let out_scale = scale * p.input_ratio;
        for (gi, q) in &method.outputs {
            supply[*gi as usize] += *q * out_scale;
        }
    }
    let staple = r.subsistence.mul_ratio(unemployed_people(r), 1000);
    if let Some(grain) = cat.good("grain") {
        supply[grain as usize] += peasant_surplus(r, staple);
    }
    let total = r.population().max(1);
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
                &m.price,
                year,
                pop.size,
                budget.max(Fixed::ZERO),
                own_food,
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

fn clear_market(
    cat: &Catalogue,
    mut m: Market,
    demand: &[Fixed],
    supply: &[Fixed],
    trade: &[Fixed],
    year: i32,
) -> Market {
    let g = cat.goods.len();
    // The state's own purchases and construction join the demand.
    let mut demand = demand.to_vec();
    let budget = m.treasury.max(Fixed::ZERO) * STATE_SPENDING * milli(400);
    for (key, per_mille) in STATE_BASKET {
        if let Some(gi) = cat.good(key) {
            let good = &cat.goods[gi as usize];
            if good.available_from <= year {
                demand[gi as usize] += budget.mul_ratio(per_mille, 1000) / m.price[gi as usize];
            }
        }
    }
    for (gi, left) in m.construction.iter().enumerate() {
        if *left > Fixed::ZERO {
            demand[gi] += (*left)
                .mul_ratio(1, CONSTRUCTION_WEEKS)
                .max((*left).min(Fixed::ONE));
        }
    }
    // Capital that found no project still turns over: maintenance, housing, roads.
    for (gi, q) in upkeep_orders(cat, &m) {
        demand[gi as usize] += q;
    }
    for gi in 0..g {
        let good = &cat.goods[gi];
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
        let base = good.base_price;
        let target = if s <= Fixed::ZERO && d <= Fixed::ZERO {
            base
        } else if s <= Fixed::ZERO {
            base * PRICE_CEILING
        } else if d <= Fixed::ZERO {
            base * PRICE_FLOOR
        } else {
            let tension = (d - s) / s.min(d).max(Fixed::EPSILON);
            let factor = (Fixed::ONE + tension * milli(750)).clamp(PRICE_FLOOR, PRICE_CEILING);
            base * factor
        };
        m.price[gi] = m.price[gi] + (target - m.price[gi]) * PRICE_SPEED;
    }
    m
}

fn settle_region(
    cat: &Catalogue,
    m: &Market,
    r: &Region,
    orders: &RegionOrders,
) -> RegionSettlement {
    let mut wages = [Fixed::ZERO; P];
    let mut owners_industry = Fixed::ZERO;
    let mut owners_land = Fixed::ZERO;
    let mut pool = Fixed::ZERO;
    let mut gdp = Fixed::ZERO;
    let mut plants = Vec::with_capacity(r.plants.len());
    for p in &r.plants {
        let b = &cat.buildings[p.kind as usize];
        let method = &b.methods[p.method as usize];
        let scale = plant_scale(p.levels, p.staffing) * r.tfp_for(b) * p.utilization;
        let mut cost = Fixed::ZERO;
        let mut cost_base = Fixed::ZERO;
        // Missing inputs cut output by their share in the value of the output (a linear
        // Cobb–Douglas): a shortage of explosives in a mine costs it a tenth, a shortage of ore
        // in a steel mill stops it.
        let output_value: Fixed = method
            .outputs
            .iter()
            .map(|(gi, q)| *q * cat.goods[*gi as usize].base_price)
            .sum();
        let mut loss = Fixed::ZERO;
        for (gi, q) in &method.inputs {
            let base = cat.goods[*gi as usize].base_price;
            let bought = *q * scale * m.buy_ratio[*gi as usize];
            cost += bought * m.price[*gi as usize];
            cost_base += bought * base;
            if output_value > Fixed::ZERO {
                let share = fill(*q * base, output_value);
                loss += share * (Fixed::ONE - m.buy_ratio[*gi as usize]);
            }
        }
        let input_ratio = (Fixed::ONE - loss).max(Fixed::ZERO);
        let out_scale = scale * p.input_ratio;
        let mut revenue = Fixed::ZERO;
        let mut revenue_base = Fixed::ZERO;
        for (gi, q) in &method.outputs {
            let sold = *q * out_scale * m.sell_ratio[*gi as usize];
            revenue += sold * m.price[*gi as usize];
            revenue_base += sold * cat.goods[*gi as usize].base_price;
        }
        gdp += revenue_base - cost_base;
        let value_added = revenue - cost;
        let wage_bill = value_added.max(Fixed::ZERO) * labor_share(&b.group);
        let profit = value_added - wage_bill;
        let weight_total: Fixed = method
            .jobs
            .iter()
            .map(|(pi, n)| WAGE_WEIGHT[*pi as usize] * *n)
            .sum();
        if weight_total > Fixed::ZERO {
            for (pi, n) in &method.jobs {
                wages[*pi as usize] += wage_bill * (WAGE_WEIGHT[*pi as usize] * *n) / weight_total;
            }
        }
        if profit > Fixed::ZERO {
            let to_owners = profit.mul_ratio(1, 2);
            if b.group == "agriculture" {
                owners_land += to_owners;
            } else {
                owners_industry += to_owners;
            }
            pool += profit - to_owners;
        } else {
            pool += profit;
        }
        // Next week's run follows this week's sales of the main output, recovering slowly when
        // everything sold.
        // Plants follow demand for their main output: they ramp up while everything sells and
        // ease off in step with what goes unsold.
        let sold_share = method
            .outputs
            .first()
            .map_or(Fixed::ONE, |(gi, _)| m.sell_ratio[*gi as usize]);
        let utilization = if sold_share >= milli(990) {
            (p.utilization + milli(50)).min(Fixed::ONE)
        } else {
            (p.utilization * (sold_share + milli(30))).clamp(milli(50), Fixed::ONE)
        };
        plants.push((input_ratio, profit, value_added, utilization));
    }
    // Owners absent from the state: their dividends go to the country's savings.
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
    // Smallholders sell what they grow beyond their own table.
    if let Some(grain) = cat.good("grain") {
        let surplus = peasant_surplus(r, r.subsistence.mul_ratio(unemployed_people(r), 1000));
        let sold = surplus * m.sell_ratio[grain as usize] * m.price[grain as usize];
        wages[prof::PEASANT] += sold;
        gdp += sold;
    }
    // The state pays its servants from the treasury.
    let gov_workers = m.gov_workers_cache();
    let payroll = m.treasury.max(Fixed::ZERO) * STATE_SPENDING * milli(600);
    for pi in [prof::CIVIL_SERVANT, prof::CLERGY, prof::SOLDIER] {
        let w = workers(r.pops[pi].size);
        if gov_workers > 0 && w > 0 {
            wages[pi] += payroll.mul_ratio(w, gov_workers);
        }
    }
    let mut taxes = Fixed::ZERO;
    let mut pops = Vec::with_capacity(P);
    for (pi, pop) in r.pops.iter().enumerate() {
        let gross = wages[pi];
        let tax = gross * TAX_RATE;
        taxes += tax;
        let income = gross - tax;
        let (spent, satisfaction) = match &orders.pops[pi] {
            Some(plan) => {
                let mut spent = Fixed::ZERO;
                let mut fulfilled = Fixed::ZERO;
                let mut wanted = Fixed::ZERO;
                for (gi, q) in &plan.orders {
                    let money = *q * m.price[*gi as usize];
                    wanted += money;
                    let got = money * m.buy_ratio[*gi as usize];
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
        pool += saved;
        pops.push((income, pop.savings - spent + income - saved, satisfaction));
    }
    // Food the jobless grow for themselves counts in GDP at the base price of grain.
    if let Some(grain) = cat.good("grain") {
        gdp += r.subsistence.mul_ratio(unemployed_people(r), 1000)
            * cat.goods[grain as usize].base_price;
    }
    RegionSettlement {
        pops,
        plants,
        taxes,
        pool,
        gdp,
    }
}

impl Market {
    /// Workers on the state payroll, cached weekly by the labour system.
    pub fn gov_workers_cache(&self) -> i64 {
        self.gov_workers
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
        Access::new().read(ECON).write(ECON)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> WeekChanges {
        let cat = &*self.cat;
        let econ = &world.econ;
        let year = ctx.date.year;
        let orders: Vec<RegionOrders> = econ
            .regions
            .par_iter()
            .map(|r| region_orders(cat, econ, r, year))
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
        // The world pool: each market offers part of its surplus and seeks part of its shortage;
        // what is traded is the smaller side, shared pro rata.
        let nm = econ.markets.len();
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
        let markets: Vec<Market> = econ
            .markets
            .iter()
            .enumerate()
            .map(|(mi, m)| clear_market(cat, m.clone(), &demand[mi], &supply[mi], &trade[mi], year))
            .collect();
        let regions: Vec<RegionSettlement> = econ
            .regions
            .par_iter()
            .zip(orders.par_iter())
            .map(|(r, o)| settle_region(cat, &markets[r.market as usize], r, o))
            .collect();
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
            regions,
            world_price,
        }
    }

    fn apply(&self, _ctx: &TickCtx, changes: WeekChanges, world: &mut GameWorld) {
        let cat = &*self.cat;
        let econ = &mut world.econ;
        let mut markets = changes.markets;
        // Trade margins: importers collected more than exporters were paid. Each market keeps
        // its balance, minus its share of the world total, so no money is created.
        let mut balance = vec![Fixed::ZERO; markets.len()];
        let mut volume = vec![Fixed::ZERO; markets.len()];
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
            // State spending and construction purchases leave the treasury and the pool.
            let gov_spent = m.treasury.max(Fixed::ZERO) * STATE_SPENDING;
            m.treasury -= gov_spent;
            for gi in 0..cat.goods.len() {
                let left = m.construction[gi];
                if left > Fixed::ZERO {
                    let bought = left
                        .mul_ratio(1, CONSTRUCTION_WEEKS)
                        .max(left.min(Fixed::ONE))
                        * m.buy_ratio[gi];
                    m.construction[gi] = (left - bought).max(Fixed::ZERO);
                    m.pool -= bought * m.price[gi];
                }
            }
            for (gi, q) in upkeep_orders(cat, m) {
                m.pool -= q * m.buy_ratio[gi as usize] * m.price[gi as usize];
            }
        }
        // Stand-in for a central bank until currencies exist: money is created when plants
        // stand idle and withdrawn when prices run hot, so nominal demand follows capacity.
        let nm = markets.len();
        let mut capacity_value = vec![Fixed::ZERO; nm];
        let mut used_value = vec![Fixed::ZERO; nm];
        for r in econ.regions.iter() {
            let mi = r.market as usize;
            for p in &r.plants {
                let method = &cat.buildings[p.kind as usize].methods[p.method as usize];
                let v: Fixed = method
                    .outputs
                    .iter()
                    .map(|(g, q)| *q * cat.goods[*g as usize].base_price)
                    .sum::<Fixed>()
                    * r.tfp_for(&cat.buildings[p.kind as usize])
                    * p.levels;
                capacity_value[mi] += v;
                used_value[mi] += v * p.utilization;
            }
        }
        for (mi, m) in markets.iter_mut().enumerate() {
            let utilization = fill(used_value[mi], capacity_value[mi]);
            let mut traded_base = Fixed::ZERO;
            let mut traded_now = Fixed::ZERO;
            for gi in 0..cat.goods.len() {
                let q = m.supply[gi].min(m.demand[gi]);
                traded_base += q * cat.goods[gi].base_price;
                traded_now += q * m.price[gi];
            }
            let index = if traded_base > Fixed::ZERO {
                traded_now / traded_base
            } else {
                Fixed::ONE
            };
            let gdp_week = m.gdp_last_year.mul_ratio(1, 52);
            let slack = (MONEY_TARGET_UTILIZATION - utilization).max(Fixed::ZERO);
            let heat = (index - MONEY_PRICE_CEILING).max(Fixed::ZERO);
            let change = gdp_week * (slack * MONEY_EASING - heat * MONEY_TIGHTENING);
            m.treasury = (m.treasury + change).max(Fixed::ZERO);
        }
        for (r, s) in econ.regions.iter_mut().zip(changes.regions) {
            let m = &mut markets[r.market as usize];
            m.treasury += s.taxes;
            m.pool += s.pool;
            m.gdp_running += s.gdp;
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
        econ.markets = markets;
        econ.world_price = changes.world_price;
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
    plan_pop(cat, prices, year, size, budget, in_kind).orders
}
