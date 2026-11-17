//! The 1925 economy built from the scenario: regions, pops, plants and markets.
//!
//! Plants are sized by a world input-output balance: each block (a country's metropole, or its
//! colonies) has a final demand at its 1925 income (households, the state, investment);
//! manufacturing and services meet their own block's demand, while farm and mine goods meet the
//! world's demand, shared among blocks by their land and deposits (farm goods feed their own
//! block first). That is what makes Malaya grow rubber for Detroit and Australia shear wool for
//! Yorkshire on day one.
//!
//! The spending is the simulation's own: a few rounds alternate between plants for a spending
//! and the spending those plants pay for by the weekly rules (wages, owners' payout, taxes,
//! savings, what households cannot spend), scaled so that the plants' value added, the state's
//! payroll and the peasants' food make the scenario's GDP. The block's productivity is the one
//! that staffs those plants with the workers it has. Pops, the pool and the treasury start at
//! those flows, so the first week buys what the plants make (docs/economy.pt-BR.md, "1925").

use laplace_schelling::Fixed;

use super::catalogue::Catalogue;
use super::prof::{self, P};
use super::state::{Economy, Market, Plant, Region, empty_pops};
use super::{WORKER_SHARE, milli, workers};
use crate::initial::{IndustryFile, PopulationFile};
use crate::resources::{DepositsFile, FARMS, LandFile, RESOURCES};
use crate::scenario::{CountryDef, MapFile};

pub struct Inputs<'a> {
    pub countries: &'a [CountryDef],
    pub map: &'a MapFile,
    pub deposits: Option<&'a DepositsFile>,
    pub land: Option<&'a LandFile>,
    pub population: Option<&'a PopulationFile>,
    pub industry: Option<&'a IndustryFile>,
}

/// Deposit units of `deposits.ron` to the catalogue's good units.
fn deposit_factor(resource: &str) -> f64 {
    match resource {
        "coal" | "iron_ore" | "oil" | "natural_gas" => 1.0e6,
        "tungsten" => 1.5,
        "uranium" => 1179.0,
        _ => 1.0e3,
    }
}

fn land_per_level(farm: &str) -> f64 {
    crate::resources::land_per_level(farm) as f64
}

/// The first guess of a block's spending, before its own accounts replace it (`ROUNDS`): shares
/// of GDP for the state's purchases and for investment, and of household consumption by fifth.
const STATE_GOODS_SHARE: f64 = 0.026;
const INVESTMENT_SHARE: f64 = 0.10;
const CONSUMPTION_BY_FIFTH: [f64; 5] = [0.06, 0.10, 0.15, 0.22, 0.47];

fn fx(v: f64) -> Fixed {
    Fixed::from_f64_lossy(v)
}

/// Largest-remainder split of `total` by weights.
fn split(total: i64, weights: &[f64]) -> Vec<i64> {
    let sum: f64 = weights.iter().sum();
    if total <= 0 || sum <= 0.0 {
        return vec![0; weights.len()];
    }
    let raw: Vec<f64> = weights.iter().map(|w| total as f64 * w / sum).collect();
    let mut out: Vec<i64> = raw.iter().map(|r| r.floor() as i64).collect();
    let mut rest = total - out.iter().sum::<i64>();
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by(|&a, &b| {
        (raw[b] - raw[b].floor())
            .total_cmp(&(raw[a] - raw[a].floor()))
            .then(a.cmp(&b))
    });
    for i in order {
        if rest == 0 {
            break;
        }
        out[i] += 1;
        rest -= 1;
    }
    out
}

fn yearly_output(cat: &Catalogue, kind: usize, good: usize) -> f64 {
    cat.buildings[kind].methods[0]
        .outputs
        .iter()
        .find(|(g, _)| *g as usize == good)
        .map_or(0.0, |(_, q)| q.to_f64_lossy() * 52.0)
}

fn best_yearly_output(cat: &Catalogue, kind: usize) -> f64 {
    cat.buildings[kind]
        .methods
        .iter()
        .filter_map(|m| m.outputs.first().map(|(_, q)| q.to_f64_lossy() * 52.0))
        .fold(0.0, f64::max)
}

struct Block {
    market: usize,
    regions: Vec<usize>,
    sizes: Vec<i64>,
    urban: Vec<f64>,
    population: u64,
    gdp_per_capita: f64,
    literacy: f64,
    /// Output per worker relative to the catalogue's 1925 baseline.
    tfp: f64,
    agr: f64,
    market_farming: f64,
    /// Weekly final demand by good.
    final_demand: Vec<f64>,
    /// Farm and mine capacity in levels, by building; mines also their 1925 levels.
    capacity: Vec<f64>,
    mines_1925: Vec<f64>,
    /// Workers the 1925 plants may employ: outside subsistence farming and the state's
    /// service, and within what the states' towns hold.
    market_workers: f64,
    /// Staples the jobless grow for themselves, tonnes a week.
    in_kind: f64,
    /// The scenario's GDP, a week.
    target: f64,
    grain_price: f64,
    /// Household money the needs cannot absorb, a week, invested through the pool.
    invested: f64,
}

impl Block {
    /// Factor that takes the accounts of the solved plants to the scenario's GDP: plants' value
    /// added, the state's payroll and the peasants' food (eaten and sold) make up the GDP the
    /// market measures.
    fn scale(&self, a: &Accounts) -> f64 {
        let food = self.in_kind * (1.0 + PEASANT_SURPLUS) * self.grain_price;
        let plants = a.va + a.payroll;
        if plants > 0.0 {
            ((self.target - food) / plants).max(0.0)
        } else {
            0.0
        }
    }
}

/// A block's weekly accounts at base prices, from its plants: value added, the jobs and the
/// people of each profession, and where the money goes by the simulation's own rules (wages
/// by `labor_share` and `WAGE_WEIGHT`, half the profit to owners and half to the pool, the
/// state's pay, taxes by class, savings by profession).
#[derive(Default)]
struct Accounts {
    va: f64,
    workers: f64,
    people: [f64; P],
    /// Income after tax, and what is left to spend after saving, by profession.
    disposable: [f64; P],
    consumption: [f64; P],
    pool: f64,
    taxes: f64,
    payroll: f64,
}

/// Grain smallholders sell, as a share of the staples they grow (`week::peasant_surplus`).
const PEASANT_SURPLUS: f64 = 0.16;

fn block_accounts(cat: &Catalogue, b: &Block, levels: &[f64]) -> Accounts {
    let f = |x: Fixed| x.to_f64_lossy();
    let mut a = Accounts::default();
    let mut wages = [0.0f64; P];
    let mut jobs = [0.0f64; P];
    for (ki, &n) in levels.iter().enumerate() {
        if n <= 0.0 {
            continue;
        }
        let bd = &cat.buildings[ki];
        let m = &bd.methods[0];
        let va = f(m.value_added(|g| cat.goods[g].base_price)) * b.tfp * n;
        a.va += va;
        a.workers += m.workers as f64 * n;
        let bill = if bd.public || bd.informal {
            va.max(0.0)
        } else {
            va.max(0.0) * f(bd.labor_share)
        };
        let weight = f(m.job_weight);
        for (pi, nj) in &m.jobs {
            let pi = *pi as usize;
            jobs[pi] += *nj as f64 * n;
            if weight > 0.0 {
                wages[pi] += bill * f(super::WAGE_WEIGHT[pi]) * *nj as f64 / weight;
            }
        }
        let profit = va - bill;
        if profit > 0.0 {
            let owner = if bd.agriculture {
                prof::LANDOWNER
            } else {
                prof::CAPITALIST
            };
            let payout = profit * f(super::PAYOUT);
            wages[owner] += payout;
            a.pool += profit - payout;
        } else {
            a.pool += profit;
        }
    }
    wages[prof::PEASANT] += b.in_kind * PEASANT_SURPLUS * b.grain_price;
    let pop = b.population as f64;
    let share = f(WORKER_SHARE);
    for pi in 0..P {
        a.people[pi] = jobs[pi] / share;
    }
    for (pi, per_mille) in STATE_SHARE {
        a.people[pi] += pop * per_mille as f64 / 1000.0;
    }
    let others: f64 = (1..P).map(|pi| a.people[pi]).sum();
    a.people[prof::PEASANT] = (pop - others).max(a.people[prof::PEASANT]);
    let rates = super::public::historical_rates(1925);
    let rate = |pi: usize| f(rates[super::public::CLASS_OF[pi]]);
    // The going wage the state pays its servants by (`monthly::market_wages`).
    let private = |pi: usize| {
        !matches!(
            pi,
            prof::CAPITALIST | prof::LANDOWNER | prof::CIVIL_SERVANT | prof::CLERGY | prof::SOLDIER
        )
    };
    let (pay, hands) = (0..P)
        .filter(|&pi| private(pi))
        .fold((0.0, 0.0), |acc, pi| {
            (acc.0 + wages[pi] * (1.0 - rate(pi)), acc.1 + jobs[pi])
        });
    let wage = if hands > 0.0 { pay / hands } else { 0.0 };
    for (pi, _) in STATE_SHARE {
        let p =
            wage * f(super::WAGE_WEIGHT[pi]) * f(super::public::PUBLIC_PAY) * a.people[pi] * share;
        wages[pi] += p;
        a.payroll += p;
    }
    for pi in 0..P {
        let tax = wages[pi] * rate(pi);
        a.taxes += tax;
        let disposable = wages[pi] - tax;
        let saved = disposable * f(super::SAVE_RATE[pi]);
        a.pool += saved;
        a.disposable[pi] = disposable;
        a.consumption[pi] = disposable - saved;
    }
    a
}

/// A block's weekly final demand when its accounts are scaled by `k`: households by
/// profession, the pool's upkeep, the state's goods and its schools and hospitals.
fn spending(
    cat: &Catalogue,
    base: &[Fixed],
    base_f: &[f64],
    b: &Block,
    a: &Accounts,
    k: f64,
) -> (Vec<f64>, f64) {
    let mut fd = vec![0.0f64; cat.goods.len()];
    // What households cannot place on their needs goes to the pool (`week::PopPlan::unplaced`).
    let mut invested = 0.0;
    let pop = (b.population as f64).max(1.0);
    for pi in 0..P {
        if a.people[pi] < 1.0 || a.consumption[pi] <= 0.0 {
            continue;
        }
        let plan = super::week::plan_pop_where(
            cat,
            base,
            1925,
            a.people[pi] as i64,
            fx(a.consumption[pi] * k),
            fx(b.in_kind * a.people[pi] / pop),
            super::migration::is_urban(pi),
        );
        let mut placed = 0.0;
        for (g, q) in plan {
            fd[g as usize] += q.to_f64_lossy();
            placed += q.to_f64_lossy() * base_f[g as usize];
        }
        invested += (a.consumption[pi] * k - placed).max(0.0);
    }
    if let Some(g) = cat.grain {
        fd[g as usize] = (fd[g as usize] - b.in_kind * PEASANT_SURPLUS).max(0.0);
    }
    let services = public_services(cat, b, &mut fd);
    let services_value: f64 = services;
    for &(g, per_mille) in &cat.upkeep_basket {
        fd[g as usize] +=
            (a.pool * k + invested) * (per_mille as f64 / 1000.0) / base_f[g as usize];
    }
    let goods = (a.payroll * k * super::public::GOODS_PER_PAYROLL.to_f64_lossy())
        .max((a.taxes - a.payroll) * k - services_value);
    for &(g, per_mille) in &cat.state_basket {
        if cat.goods[g as usize].available_from <= 1925 {
            fd[g as usize] += goods * per_mille as f64 / 1000.0 / base_f[g as usize];
        }
    }
    (fd, invested)
}

/// Research a block bought in a week of 1925, in lots: the firms' part on the economy outside
/// the farms, times how close to the frontier the block is (its income against the richest
/// in 1925, about the United States' $10,000 a head at 2011 prices: before the first month the
/// research state is not there to count technologies, `public::frontier`); the state's part
/// on its revenue (the goods it buys are two fifths of it).
/// `RESEARCH_OFF_FARM_1925` is design: total R&D was about 0.6 % of national income in the
/// United States of the late 1930s and a tenth of a percent in Britain (Bernal 1939,
/// `docs/research.pt-BR.md`), almost all of it industrial.
const RESEARCH_OFF_FARM_1925: f64 = 0.003;

fn research_1925(gdp_week: f64, agr: f64, gdp_per_capita: f64) -> f64 {
    let frontier = (gdp_per_capita / 10_000.0).clamp(0.05, 1.0);
    let state =
        gdp_week * STATE_GOODS_SHARE / 0.4 * super::public::STATE_RESEARCH_SHARE.to_f64_lossy();
    (gdp_week * (1.0 - agr) * RESEARCH_OFF_FARM_1925 * frontier + state) / 1000.0
}

/// Schools, hospitals and laboratories the state runs, added to `fd`: enrolment follows
/// literacy, hospital care income, research the economy outside the farms. Returns their value
/// at base prices.
fn public_services(cat: &Catalogue, b: &Block, fd: &mut [f64]) -> f64 {
    let mut value = 0.0;
    if let Some(g) = cat.research {
        let q = research_1925(
            b.population as f64 * b.gdp_per_capita / 52.0,
            b.agr,
            b.gdp_per_capita,
        );
        fd[g as usize] += q;
        value += q * cat.goods[g as usize].base_price.to_f64_lossy();
    }
    for (key, per_person, cover) in [
        (
            "schooling",
            super::public::PUPILS_PER_PERSON,
            b.literacy.clamp(0.05, 0.95),
        ),
        (
            "hospital_care",
            super::public::CARE_PER_PERSON,
            (b.gdp_per_capita / 15_000.0).clamp(0.02, 0.5),
        ),
    ] {
        if let Some(g) = cat.good(key) {
            let q = b.population as f64 * per_person.to_f64_lossy() * cover / 52.0;
            fd[g as usize] += q;
            value += q * cat.goods[g as usize].base_price.to_f64_lossy();
        }
    }
    value
}

/// Share of the population the state employs, per mille (`monthly::STATE_SHARE`).
use super::monthly::STATE_SHARE;

/// (block, building, levels) a raw good is split into.
type Allocation = Vec<(usize, usize, f64)>;

pub fn build(cat: &Catalogue, inp: &Inputs) -> Economy {
    let ng = cat.goods.len();
    let nb = cat.buildings.len();
    let country_index = |tag: &str| inp.countries.iter().position(|c| c.tag == tag);
    let land_of = |key: &str| {
        inp.land
            .and_then(|l| l.states.iter().find(|s| s.state == key))
    };
    let deposits_of = |key: &str| {
        inp.deposits
            .and_then(|d| d.states.iter().find(|s| s.state == key))
    };
    let pop_of = |key: &str| {
        inp.population
            .and_then(|p| p.states.iter().find(|s| s.state == key))
    };
    let base: Vec<Fixed> = cat.goods.iter().map(|g| g.base_price).collect();
    let base_f: Vec<f64> = base.iter().map(|p| p.to_f64_lossy()).collect();
    let mine_of = |res: usize| {
        cat.buildings
            .iter()
            .position(|b| b.resource == Some(res as u8))
    };
    let farm_of = |farm: usize| {
        cat.buildings
            .iter()
            .position(|b| b.farm == Some(farm as u8))
    };

    // 1. Regions in map order, with their deposits and land in levels.
    let mut regions: Vec<Region> = Vec::new();
    let mut keys: Vec<(String, f64)> = Vec::new();
    let mut region_of_state: std::collections::BTreeMap<u16, u32> =
        std::collections::BTreeMap::new();
    for s in &inp.map.states {
        let Some(ci) = country_index(&s.owner) else {
            continue;
        };
        let mut deposits = Vec::new();
        for dep in deposits_of(&s.key).map_or(&[][..], |d| &d.deposits[..]) {
            let Some(res) = RESOURCES.iter().position(|r| *r == dep.resource) else {
                continue;
            };
            let Some(kind) = mine_of(res) else { continue };
            let per_level = best_yearly_output(cat, kind);
            let ceiling = if per_level > 0.0 {
                (f64::from(dep.potential) * deposit_factor(&dep.resource) / per_level).ceil() as i64
            } else {
                0
            };
            deposits.push((res as u8, dep.discovered, ceiling.max(1)));
        }
        let mut land = Vec::new();
        let mut frontier = Vec::new();
        let mut wild_km2 = [0i64; 2];
        if let Some(l) = land_of(&s.key) {
            wild_km2 = [l.pasture_km2 as i64, l.forest_km2 as i64];
            for (farm, suit) in &l.farms {
                let Some(fi) = FARMS.iter().position(|f| f == farm) else {
                    continue;
                };
                let ceiling = match farm.as_str() {
                    "fishery" => {
                        if l.coast {
                            (40.0 * f64::from(*suit)).ceil()
                        } else {
                            0.0
                        }
                    }
                    "livestock_ranch" => {
                        f64::from(l.pasture_km2) * f64::from(*suit) / land_per_level(farm)
                    }
                    "forestry" => f64::from(l.forest_km2) * f64::from(*suit) / land_per_level(farm),
                    _ => f64::from(l.cropland_km2) * f64::from(*suit) / land_per_level(farm),
                } as i64;
                if ceiling > 0 {
                    land.push((fi as u8, ceiling));
                }
                // Pasture and forest suited to the crop can be cleared for it.
                if crate::resources::on_cropland(farm) {
                    let wild = f64::from(l.pasture_km2) + f64::from(l.forest_km2);
                    let more = (wild * f64::from(*suit) / land_per_level(farm)) as i64;
                    if more > 0 {
                        frontier.push((fi as u8, more));
                    }
                }
            }
        }
        regions.push(Region {
            market: ci as u16,
            colony: s.colony,
            pops: empty_pops(),
            plants: Vec::new(),
            literacy: Fixed::ZERO,
            deposits,
            land,
            frontier,
            wild_km2,
            cleared_km2: [0; 2],
            pioneer_km2: Fixed::ZERO,
            subsistence: Fixed::ZERO,
            tfp: Fixed::ONE,
            tfp_start: Fixed::ONE,
            state: s.id,
            neighbours: Vec::new(),
            local: Vec::new(),
        });
        keys.push((s.key.clone(), f64::from(s.area_km2)));
        region_of_state.insert(s.id, regions.len() as u32 - 1);
    }
    // Land borders within one market: what a railway or a power line can cross.
    for s in &inp.map.states {
        let Some(&ri) = region_of_state.get(&s.id) else {
            continue;
        };
        let market = regions[ri as usize].market;
        let list: Vec<u32> = s
            .neighbours
            .iter()
            .filter_map(|n| region_of_state.get(n).copied())
            .filter(|&n| regions[n as usize].market == market)
            .collect();
        regions[ri as usize].neighbours = list;
    }

    // Staple food per 1,000 people per week, in tonnes.
    let staple = cat
        .needs
        .iter()
        .find(|n| n.key == "staple_food")
        .map_or(0.0, |n| n.per_capita.to_f64_lossy() / 1000.0);

    // 2. Blocks: population, labour structure, final demand, capacity.
    let mut blocks: Vec<Block> = Vec::new();
    for (ci, def) in inp.countries.iter().enumerate() {
        for colony in [false, true] {
            let idx: Vec<usize> = (0..regions.len())
                .filter(|&i| regions[i].market as usize == ci && regions[i].colony == colony)
                .collect();
            if idx.is_empty() {
                continue;
            }
            let (population, gdppc, lit, urb) = if colony {
                match &def.colonies {
                    Some(c) => (
                        c.population,
                        f64::from(c.gdp_per_capita),
                        c.literacy.unwrap_or(0.1),
                        c.urbanization.unwrap_or(0.1),
                    ),
                    None => continue,
                }
            } else {
                (
                    def.population,
                    f64::from(def.gdp_per_capita),
                    def.literacy.unwrap_or(0.3),
                    def.urbanization.unwrap_or(0.2),
                )
            };
            let has_census = idx.iter().all(|&i| pop_of(&keys[i].0).is_some());
            let sizes: Vec<i64> = if has_census {
                idx.iter()
                    .map(|&i| pop_of(&keys[i].0).map_or(0, |p| p.population as i64))
                    .collect()
            } else {
                let w: Vec<f64> = idx
                    .iter()
                    .map(|&i| {
                        land_of(&keys[i].0).map_or(0.0, |l| f64::from(l.cropland_km2))
                            + keys[i].1 * 0.02
                            + 1.0
                    })
                    .collect();
                split(population as i64, &w)
            };
            let urban: Vec<f64> = idx
                .iter()
                .map(|&i| pop_of(&keys[i].0).map_or(f64::from(urb), |p| f64::from(p.urban)))
                .collect();
            let workforce = inp.population.and_then(|p| {
                p.workforce
                    .iter()
                    .find(|w| w.country == def.tag && w.colonies == colony)
            });
            let agr = workforce.map_or_else(
                || (0.85 - gdppc / 12_000.0).clamp(0.08, 0.85),
                |w| f64::from(w.agriculture),
            );
            let market_farming = (gdppc / 6_000.0).clamp(0.2, 0.9);

            let gdp_week = population as f64 * gdppc / 52.0;
            let mut final_demand = vec![0.0f64; ng];
            let in_kind = staple * population as f64 / 1000.0 * agr * (1.0 - market_farming);
            // Households in five income fifths: the poor buy staples, the rich the rest. Each
            // fifth splits into town and country by the census share, for rent and shelter.
            let town = f64::from(urb).clamp(0.0, 1.0);
            for share in CONSUMPTION_BY_FIFTH {
                for (urban, part) in [(true, town), (false, 1.0 - town)] {
                    let plan = super::week::plan_pop_where(
                        cat,
                        &base,
                        1925,
                        (population as f64 / 5.0 * part) as i64,
                        fx(gdp_week * 0.70 * share * part),
                        fx(in_kind / 5.0 * part),
                        urban,
                    );
                    for (g, q) in plan {
                        final_demand[g as usize] += q.to_f64_lossy();
                    }
                }
            }
            // The state buys goods with 40% of its tax take; the rest pays its servants.
            for &(g, per_mille) in &cat.state_basket {
                if cat.goods[g as usize].available_from <= 1925 {
                    final_demand[g as usize] += gdp_week * STATE_GOODS_SHARE * per_mille as f64
                        / 1000.0
                        / base_f[g as usize];
                }
            }
            // Schools and hospitals: enrolment follows literacy, hospital care income.
            for (key, per_person, cover) in [
                (
                    "schooling",
                    super::public::PUPILS_PER_PERSON,
                    f64::from(lit).clamp(0.05, 0.95),
                ),
                (
                    "hospital_care",
                    super::public::CARE_PER_PERSON,
                    (gdppc / 15_000.0).clamp(0.02, 0.5),
                ),
            ] {
                if let Some(g) = cat.good(key) {
                    final_demand[g as usize] +=
                        population as f64 * per_person.to_f64_lossy() * cover / 52.0;
                }
            }
            if let Some(g) = cat.research {
                final_demand[g as usize] += research_1925(gdp_week, agr, gdppc);
            }
            for &(g, per_mille) in &cat.upkeep_basket {
                final_demand[g as usize] +=
                    gdp_week * INVESTMENT_SHARE * (per_mille as f64 / 1000.0) / base_f[g as usize];
            }
            let mut capacity = vec![0.0f64; nb];
            let mut mines_1925 = vec![0.0f64; nb];
            for &ri in &idx {
                for (farm, ceiling) in &regions[ri].land {
                    if let Some(ki) = farm_of(*farm as usize) {
                        capacity[ki] += *ceiling as f64;
                    }
                }
                for (res, discovered, ceiling) in &regions[ri].deposits {
                    if *discovered <= 1925
                        && let Some(ki) = mine_of(*res as usize)
                    {
                        // Ceilings count levels of the best method; 1925 mines use the first.
                        let first = cat.buildings[ki].methods[0]
                            .outputs
                            .first()
                            .map_or(0, |(g, _)| *g as usize);
                        let ratio =
                            best_yearly_output(cat, ki) / yearly_output(cat, ki, first).max(1.0);
                        capacity[ki] += *ceiling as f64 * ratio;
                    }
                }
                for dep in deposits_of(&keys[ri].0).map_or(&[][..], |d| &d.deposits[..]) {
                    if dep.discovered > 1925 || dep.output_1925 <= 0.0 {
                        continue;
                    }
                    let Some(res) = RESOURCES.iter().position(|x| *x == dep.resource) else {
                        continue;
                    };
                    let Some(ki) = mine_of(res) else { continue };
                    let first = cat.buildings[ki].methods[0]
                        .outputs
                        .first()
                        .map_or(0, |(g, _)| *g as usize);
                    let per_level = yearly_output(cat, ki, first).max(1.0);
                    mines_1925[ki] +=
                        f64::from(dep.output_1925) * deposit_factor(&dep.resource) / per_level;
                    let _ = gdppc;
                }
            }
            for ki in 0..nb {
                capacity[ki] = capacity[ki].max(mines_1925[ki]);
            }
            // The catalogue's methods make about $6,000 a worker. The GDP not grown by peasants
            // for themselves is made by the market's workers: that sets the gap.
            let grain_price = cat.good("grain").map_or(700.0, |g| base_f[g as usize]);
            let subsistence = staple / 1000.0 * 52.0 * agr * (1.0 - market_farming) * grain_price;
            let market_workers = WORKER_SHARE.to_f64_lossy() * (1.0 - agr * (1.0 - market_farming));
            let tfp = ((gdppc - subsistence).max(gdppc * 0.3) / (market_workers * 6_000.0))
                .clamp(0.35, 8.0);
            let total_workers = sizes.iter().map(|&s| workers(s)).sum::<i64>() as f64;
            let jobless = agr * (1.0 - market_farming);
            // The state's servants, soldiers and clergy are not on the plants' payrolls, and no
            // state fills more than `TOWN_ROOM` of its people with plant workers.
            let state_workers =
                total_workers * STATE_SHARE.iter().map(|s| s.1 as f64).sum::<f64>() / 1000.0;
            blocks.push(Block {
                market_workers: (total_workers * (1.0 - jobless) * WORKER_ROOM)
                    .min(total_workers * TOWN_ROOM - state_workers),
                // As `week::subsistence_food` grows it: nine tenths of a full ration.
                in_kind: staple * population as f64 / 1000.0 * jobless * 0.9,
                target: gdp_week,
                grain_price,
                invested: 0.0,
                tfp,
                market: ci,
                regions: idx,
                sizes,
                urban,
                population,
                gdp_per_capita: gdppc,
                literacy: f64::from(lit),
                agr,
                market_farming,
                final_demand,
                capacity,
                mines_1925,
            });
        }
    }

    // 3. Who makes each good: raw goods (farms, mines) world-wide by capacity; the rest locally,
    //    by the first plant whose first method has it as its main output.
    let raw_producers: Vec<Vec<usize>> = (0..ng)
        .map(|g| {
            cat.buildings
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    (b.farm.is_some() || b.resource.is_some())
                        && b.methods[0].outputs.iter().any(|(o, _)| *o as usize == g)
                })
                .map(|(k, _)| k)
                .collect()
        })
        .collect();
    let local_producer: Vec<Option<usize>> = (0..ng)
        .map(|g| {
            if !raw_producers[g].is_empty() {
                return None;
            }
            cat.buildings.iter().position(|b| {
                b.farm.is_none()
                    && b.resource.is_none()
                    && b.methods[0]
                        .outputs
                        .first()
                        .is_some_and(|(o, _)| *o as usize == g)
            })
        })
        .collect();

    // 4. World input-output balance, then the blocks' own accounts: what the simulation's
    //    rules pay out of the plants' value added is what the block spends, and its
    //    productivity is what makes its workers produce the scenario's GDP.
    let mut lv = solve_levels(cat, &blocks, &raw_producers, &local_producer);
    let mut accounts: Vec<Accounts> = Vec::new();
    for round in 0..ROUNDS {
        accounts = blocks
            .iter()
            .enumerate()
            .map(|(bi, b)| block_accounts(cat, b, &lv[bi]))
            .collect();
        if round + 1 == ROUNDS {
            break;
        }
        for (b, a) in blocks.iter_mut().zip(&accounts) {
            // Workers the plants need against the market workers the block has: the same
            // value added from fewer or more hands is a higher or lower productivity.
            let room = b.market_workers;
            if a.workers > 0.0 && room > 0.0 {
                b.tfp = (b.tfp * a.workers / room).clamp(0.2, 12.0);
            }
            (b.final_demand, b.invested) = spending(cat, &base, &base_f, b, a, b.scale(a));
        }
        lv = solve_levels(cat, &blocks, &raw_producers, &local_producer);
    }

    if std::env::var_os("BF_SETUP_DEBUG").is_some() {
        let mut demand: Vec<f64> = vec![0.0; ng];
        for (bi, b) in blocks.iter().enumerate() {
            for g in 0..ng {
                demand[g] += b.final_demand[g];
            }
            for (ki, n) in lv[bi].iter().enumerate() {
                for (g, q) in &cat.buildings[ki].methods[0].inputs {
                    demand[*g as usize] += n * q.to_f64_lossy() * b.tfp;
                }
            }
        }
        for key in [
            "natural_rubber",
            "cotton",
            "iron_ore",
            "coal",
            "steel",
            "lead_zinc_ore",
            "grain",
        ] {
            let Some(g) = cat.good(key) else { continue };
            let g = g as usize;
            let mut supply = 0.0;
            let mut cap = 0.0;
            for (bi, b) in blocks.iter().enumerate() {
                for (ki, n) in lv[bi].iter().enumerate() {
                    let per = yearly_output(cat, ki, g) / 52.0 * b.tfp;
                    supply += n * per;
                    cap += b.capacity[ki] * yearly_output(cat, ki, g) / 52.0;
                }
            }
            eprintln!(
                "solve {key}: demand/wk {:.0}, planned supply/wk {:.0}, capacity/wk {:.0}",
                demand[g], supply, cap
            );
        }
    }

    // 5. Fit each block into its workforce and lay the plants and people on its regions.
    let housing_kind = cat.building("urban_housing");
    for (bi, b) in blocks.iter().enumerate() {
        let total_workers: i64 = b.sizes.iter().map(|&s| workers(s)).sum();
        let reserved = (total_workers as f64 * b.agr * (1.0 - b.market_farming)) as i64;
        let room = (total_workers - reserved).max(0) as f64;
        // Farms and mines keep their size; manufacturing and services give way if people lack.
        let is_raw =
            |ki: usize| cat.buildings[ki].farm.is_some() || cat.buildings[ki].resource.is_some();
        let need_raw: f64 = lv[bi]
            .iter()
            .enumerate()
            .filter(|(ki, _)| is_raw(*ki))
            .map(|(ki, n)| n * cat.buildings[ki].methods[0].workers as f64)
            .sum();
        let need_other: f64 = lv[bi]
            .iter()
            .enumerate()
            .filter(|(ki, _)| !is_raw(*ki))
            .map(|(ki, n)| n * cat.buildings[ki].methods[0].workers as f64)
            .sum();
        let left = (room * 0.95 - need_raw).max(0.0);
        let k_other = if need_other > left {
            left / need_other
        } else {
            1.0
        };
        if std::env::var_os("BF_SETUP_DEBUG").is_some() {
            eprintln!(
                "block {} colony {}: workers {:.1} M, reserved {:.1} M, raw need {:.1} M, other need {:.1} M, k_other {:.2}, tfp {:.2}",
                inp.countries[b.market].tag,
                regions[b.regions[0]].colony,
                total_workers as f64 / 1e6,
                reserved as f64 / 1e6,
                need_raw / 1e6,
                need_other / 1e6,
                k_other,
                b.tfp
            );
        }
        let urban_weight: Vec<f64> = b
            .sizes
            .iter()
            .zip(&b.urban)
            .map(|(s, u)| *s as f64 * u.max(0.02))
            .collect();
        // Transport and power go last, where the plants and people placed before them need
        // them: a railway is laid to the mine, not to the capital.
        // Farms and mines first, on their land; then the rest of the plants, in the towns but
        // only as far as each state has people for them; transport and power last.
        let local = |ki: usize| super::local::local_output(cat, ki).is_some();
        let order: Vec<usize> = (0..nb)
            .filter(|&ki| is_raw(ki) && !local(ki))
            .chain((0..nb).filter(|&ki| !is_raw(ki) && !local(ki)))
            .chain((0..nb).filter(|&ki| local(ki)))
            .collect();
        let people: i64 = b.sizes.iter().sum::<i64>().max(1);
        let mut town: Option<Vec<f64>> = None;
        let mut electrified = false;
        for ki in order {
            // Before transport and power are laid: the share of the plants that ran on
            // electricity in 1925, as far as the country made it (`industry.ron`).
            if local(ki) && !electrified {
                electrified = true;
                let colony = regions[b.regions[0]].colony;
                let target = electricity_1925(inp, cat, &inp.countries[b.market].tag, b, colony);
                electrify(cat, b, &mut regions, target);
            }
            if !is_raw(ki) && !local(ki) && town.is_none() {
                let need: f64 = (0..nb)
                    .filter(|&k| !is_raw(k) && !local(k))
                    .map(|k| lv[bi][k] * k_other * cat.buildings[k].methods[0].workers as f64)
                    .sum::<f64>()
                    / WORKER_SHARE.to_f64_lossy();
                let room: Vec<f64> = b
                    .regions
                    .iter()
                    .zip(&b.sizes)
                    .map(|(&ri, &size)| {
                        let raw: i64 = regions[ri]
                            .plants
                            .iter()
                            .map(|p| cat.buildings[p.kind as usize].methods[0].workers * p.levels)
                            .sum();
                        (size as f64 * TOWN_ROOM - raw as f64 / WORKER_SHARE.to_f64_lossy())
                            .max(0.0)
                    })
                    .collect();
                town = Some(fill_by_weight(need, &urban_weight, &room));
            }
            let n = &lv[bi][ki];
            let k = if is_raw(ki) { 1.0 } else { k_other };
            let levels = (n * k).round() as i64;
            let bd = &cat.buildings[ki];
            let local_good = super::local::local_output(cat, ki).map(|li| cat.local[li] as usize);
            if levels <= 0 && (local_good.is_none() || n * k <= 0.0) {
                continue;
            }
            let w: Vec<f64> = b
                .regions
                .iter()
                .zip(&b.sizes)
                .zip(&urban_weight)
                .enumerate()
                .map(|(pos, ((&ri, &size), w))| {
                    if let Some(g) = local_good {
                        // Demand here: the plants' inputs and a share of the people's and
                        // the state's purchases by population.
                        let plants: f64 = regions[ri]
                            .plants
                            .iter()
                            .map(|p| {
                                cat.buildings[p.kind as usize].methods[p.method as usize]
                                    .inputs
                                    .iter()
                                    .filter(|(gi, _)| *gi as usize == g)
                                    .map(|(_, q)| q.to_f64_lossy())
                                    .sum::<f64>()
                                    * p.levels as f64
                                    * b.tfp
                            })
                            .sum();
                        let coast_ok =
                            bd.key != "port" || land_of(&keys[ri].0).is_some_and(|l| l.coast);
                        if coast_ok {
                            plants + b.final_demand[g] * size as f64 / people as f64
                        } else {
                            0.0
                        }
                    } else if let Some(farm) = bd.farm {
                        regions[ri]
                            .land
                            .iter()
                            .filter(|l| l.0 == farm)
                            .map(|l| l.1 as f64)
                            .sum()
                    } else if let Some(res) = bd.resource {
                        regions[ri]
                            .deposits
                            .iter()
                            .filter(|d| d.0 == res && d.1 <= 1925)
                            .map(|d| d.2 as f64)
                            .sum()
                    } else if bd.key == "port" && !land_of(&keys[ri].0).is_some_and(|l| l.coast) {
                        0.0
                    } else {
                        town.as_ref().map_or(*w, |t| t[pos])
                    }
                })
                .collect();
            // Farms and mines go to their land as far as the state has people to work them;
            // the land of an empty state waits for settlers.
            let w = if is_raw(ki) && !local(ki) {
                let per_level = bd.methods[0].workers.max(1) as f64;
                let room: Vec<f64> = b
                    .regions
                    .iter()
                    .zip(&b.sizes)
                    .zip(&w)
                    .map(|((&ri, &size), land)| {
                        let used: i64 = regions[ri]
                            .plants
                            .iter()
                            .map(|p| cat.buildings[p.kind as usize].methods[0].workers * p.levels)
                            .sum();
                        let hands =
                            size as f64 * TOWN_ROOM * WORKER_SHARE.to_f64_lossy() - used as f64;
                        land.min((hands / per_level).max(0.0))
                    })
                    .collect();
                let placed = fill_by_weight(n * k, &w, &room);
                if placed.iter().sum::<f64>() > 0.0 {
                    placed
                } else {
                    w
                }
            } else {
                w
            };
            // Transport and power are rounded up state by state: rounded for the whole block,
            // most states with a little demand got no power plant at all and waited for a
            // neighbour's surplus.
            let placed = if let Some(g) = local_good {
                let total: f64 = w.iter().sum();
                // As many levels as the demand laid so far asks for: the plants moved to
                // electric methods use more power than the balance planned for.
                let per_level = yearly_output(cat, ki, g) / 52.0 * b.tfp;
                let wanted = if per_level > 0.0 {
                    (n * k).max(total / per_level)
                } else {
                    n * k
                };
                w.iter()
                    .map(|x| {
                        if total > 0.0 {
                            (wanted * x / total - 1e-6).ceil().max(0.0) as i64
                        } else {
                            0
                        }
                    })
                    .collect()
            } else {
                split(levels, &w)
            };
            for (&ri, n) in b.regions.iter().zip(placed) {
                if n > 0 {
                    let p: &mut Plant = regions[ri].plant_mut(ki as u16);
                    p.levels += n;
                }
            }
        }
        for (pos, &ri) in b.regions.iter().enumerate() {
            let r = &mut regions[ri];
            let size = b.sizes[pos];
            r.literacy = fx(b.literacy);
            r.tfp = fx(b.tfp);
            r.tfp_start = r.tfp;
            let cropland = land_of(&keys[ri].0).map_or(0.0, |l| f64::from(l.cropland_km2));
            let per_head_ha = if size > 0 {
                cropland * 100.0 / size as f64
            } else {
                0.0
            };
            r.subsistence = fx(staple * (per_head_ha / 0.3).clamp(0.4, 1.0) * 0.9);
            let mut jobs = [0i64; P];
            for p in &r.plants {
                for (pi, n) in &cat.buildings[p.kind as usize].methods[p.method as usize].jobs {
                    jobs[*pi as usize] += *n * p.levels;
                }
            }
            let mut people: Vec<i64> = jobs
                .iter()
                .map(|w| (Fixed::from_int(*w) / WORKER_SHARE).to_int_floor())
                .collect();
            let used: i64 = people.iter().sum();
            if used > size * 9 / 10 && used > 0 {
                // More jobs than people here: thin every plant to fit.
                let k = Fixed::from_ratio(size * 9 / 10, used);
                for p in &mut r.plants {
                    p.levels = (Fixed::from_int(p.levels) * k).to_int_floor();
                }
                r.plants.retain(|p| p.levels > 0);
                jobs = [0; P];
                for p in &r.plants {
                    for (pi, n) in &cat.buildings[p.kind as usize].methods[p.method as usize].jobs {
                        jobs[*pi as usize] += *n * p.levels;
                    }
                }
                people = jobs
                    .iter()
                    .map(|w| (Fixed::from_int(*w) / WORKER_SHARE).to_int_floor())
                    .collect();
            }
            for (pi, share) in STATE_SHARE {
                people[pi] += size * share / 1000;
            }
            let used: i64 = people.iter().sum();
            people[prof::PEASANT] += (size - used).max(0);
            let a = &accounts[bi];
            let k = b.scale(a);
            for (pi, pop) in r.pops.iter_mut().enumerate() {
                pop.size = people[pi];
                pop.employed = workers(people[pi]).min(jobs[pi]);
                pop.satisfaction = [milli(800); 4];
                // What the block's accounts pay each person of the profession, after tax.
                let income = if a.people[pi] >= 1.0 {
                    a.disposable[pi] * k * pop.size as f64 / a.people[pi]
                } else {
                    0.0
                };
                pop.income = fx(income);
                pop.savings = fx(income * 6.0);
            }
            // City housing for every town dweller (by profession), whatever the census said.
            if let Some(hk) = housing_kind {
                let per = super::migration::housing_per_level(cat, r, hk);
                if per > Fixed::ZERO {
                    let urban = Fixed::from_int(super::migration::urban_population(r));
                    let levels = (urban / per).to_int_floor() + 1;
                    r.plant_mut(hk as u16).levels = levels;
                }
            }
            for p in &mut r.plants {
                p.staffing = Fixed::ONE;
            }
        }
    }

    // 6. Markets and the money people start with.
    let mut markets = Vec::new();
    for ci in 0..inp.countries.len() {
        let target_gdp: f64 = blocks
            .iter()
            .filter(|b| b.market == ci)
            .map(|b| b.population as f64 * b.gdp_per_capita)
            .sum();
        // The pool starts where its upkeep spends what flows in, the treasury at its reserve:
        // both at the block accounts' rates, so the first week spends what the plants pay.
        let (mut inflow, mut taxes, mut payroll, mut services) = (0.0, 0.0, 0.0, 0.0);
        let mut scratch = vec![0.0; ng];
        for (b, a) in blocks.iter().zip(&accounts).filter(|(b, _)| b.market == ci) {
            let k = b.scale(a);
            inflow += a.pool * k + b.invested;
            taxes += a.taxes * k;
            payroll += a.payroll * k;
            services += public_services(cat, b, &mut scratch);
        }
        let revenue = fx(taxes);
        markets.push(Market {
            price: base.clone(),
            supply: vec![Fixed::ZERO; ng],
            demand: vec![Fixed::ZERO; ng],
            buy_ratio: vec![Fixed::ONE; ng],
            sell_ratio: vec![Fixed::ONE; ng],
            net_import: vec![Fixed::ZERO; ng],
            // A pool that spends its weekly upkeep share at the rate investment flows in.
            pool: fx(inflow.max(0.0) / super::week::UPKEEP_SHARE.to_f64_lossy()),
            treasury: revenue * super::public::RESERVE_WEEKS,
            tfp: Fixed::ONE,
            construction: vec![Fixed::ZERO; ng],
            gdp_running: Fixed::ZERO,
            gdp_last_year: fx(target_gdp),
            tech_tfp: Vec::new(),
            gov_workers: 0,
            queue: Vec::new(),
            laws: Default::default(),
            money: super::money::Money::start(fx(target_gdp)),
            budget: super::state::Budget {
                revenue_avg: revenue,
                payroll: fx(payroll),
                services: fx(services),
                ..Default::default()
            },
        });
    }
    Economy {
        regions,
        markets,
        world_price: base,
        history: Vec::new(),
        ledger: Default::default(),
    }
}

/// Electricity a block's plants used in 1925, a week: what its country generated
/// (`industry.ron`, colonies included in the owner's figure, so all of it goes to the
/// metropole) less what its households and state buy.
fn electricity_1925(inp: &Inputs, cat: &Catalogue, tag: &str, b: &Block, colony: bool) -> f64 {
    let Some(g) = cat.good("electricity") else {
        return 0.0;
    };
    if colony {
        return 0.0;
    }
    let made = inp
        .industry
        .and_then(|f| f.countries.iter().find(|c| c.country == tag))
        .and_then(|c| c.output.iter().find(|(k, _)| k == "electricity"))
        .map_or(0.0, |(_, v)| f64::from(*v) * 1000.0 / 52.0);
    (made - b.final_demand[g as usize]).max(0.0)
}

/// Moves plants of a block to the electric method their building had in 1925, the most urban
/// states first, until the plants use `target` of electricity a week. A plant keeps its output:
/// it runs as many levels of the new method as make what the old ones made.
fn electrify(cat: &Catalogue, b: &Block, regions: &mut [Region], target: f64) {
    let Some(g) = cat.good("electricity") else {
        return;
    };
    let uses = |m: &super::catalogue::Method| -> f64 {
        m.inputs
            .iter()
            .filter(|(gi, _)| *gi == g)
            .map(|(_, q)| q.to_f64_lossy())
            .sum()
    };
    let mut used: f64 = b
        .regions
        .iter()
        .flat_map(|&ri| regions[ri].plants.iter())
        .map(|p| uses(&cat.buildings[p.kind as usize].methods[p.method as usize]) * p.levels as f64)
        .sum::<f64>()
        * b.tfp;
    let mut order: Vec<(usize, f64)> = b
        .regions
        .iter()
        .copied()
        .zip(b.urban.iter().copied())
        .collect();
    order.sort_by(|a, c| c.1.total_cmp(&a.1).then(a.0.cmp(&c.0)));
    for (ri, _) in order {
        for p in regions[ri].plants.iter_mut() {
            if used >= target {
                return;
            }
            let bd = &cat.buildings[p.kind as usize];
            if bd.public || p.method != 0 || p.levels <= 0 {
                continue;
            }
            let first = &bd.methods[0];
            let Some(&(out, q0)) = first.outputs.first() else {
                continue;
            };
            let Some((mi, m)) = bd.methods.iter().enumerate().skip(1).find(|(_, m)| {
                m.available_from <= 1925
                    && uses(m) > 0.0
                    && m.outputs.first().is_some_and(|(o, _)| *o == out)
            }) else {
                continue;
            };
            let q1 = m.outputs[0].1.to_f64_lossy();
            if q1 <= 0.0 {
                continue;
            }
            let levels = (p.levels as f64 * q0.to_f64_lossy() / q1).ceil().max(1.0) as i64;
            let extra = (uses(m) * levels as f64 - uses(first) * p.levels as f64) * b.tfp;
            // A switch that would overshoot by more than what is missing waits for a smaller one.
            if extra > 2.0 * (target - used) {
                continue;
            }
            used += extra;
            p.method = mi as u8;
            p.levels = levels;
        }
    }
}

/// Share of a state's people the plants of 1925 may employ (with their families) before the
/// rest of a block's industry and services goes to other states: the state also has its
/// servants, soldiers and clergy, and the transport and power laid last.
const TOWN_ROOM: f64 = 0.88;

/// Splits `total` by `weight`, no part above its `room`; what a full part cannot take goes to
/// the others by weight. With too little room in all, every part is full and the rest is lost.
fn fill_by_weight(total: f64, weight: &[f64], room: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0; weight.len()];
    let mut open: Vec<bool> = room.iter().map(|r| *r > 0.0).collect();
    let mut left = total;
    for _ in 0..weight.len() {
        let w: f64 = (0..weight.len())
            .filter(|&i| open[i])
            .map(|i| weight[i])
            .sum();
        if left <= 0.0 || w <= 0.0 {
            break;
        }
        let mut overflow = false;
        for i in 0..weight.len() {
            if open[i] && out[i] + left * weight[i] / w >= room[i] {
                overflow = true;
            }
        }
        if !overflow {
            for i in 0..weight.len() {
                if open[i] {
                    out[i] += left * weight[i] / w;
                }
            }
            break;
        }
        // Fill the parts that overflow, then share what is left among the rest.
        let share = left;
        for i in 0..weight.len() {
            if open[i] && out[i] + share * weight[i] / w >= room[i] {
                left -= room[i] - out[i];
                out[i] = room[i];
                open[i] = false;
            }
        }
    }
    out
}

/// Rounds of the setup's balance: plants for a spending, then the spending those plants pay for.
const ROUNDS: usize = 6;
/// Share of a block's market workers the 1925 plants employ; the rest look for work.
const WORKER_ROOM: f64 = 0.95;

fn solve_levels(
    cat: &Catalogue,
    blocks: &[Block],
    raw_producers: &[Vec<usize>],
    local_producer: &[Option<usize>],
) -> Vec<Vec<f64>> {
    let ng = cat.goods.len();
    let nb = cat.buildings.len();
    let nbk = blocks.len();
    // Workers each block can put on commercial farms and in mines.
    let farm_cap: Vec<f64> = blocks
        .iter()
        .map(|b| b.sizes.iter().map(|&s| workers(s)).sum::<i64>() as f64 * b.agr * b.market_farming)
        .collect();
    let mine_cap: Vec<f64> = blocks
        .iter()
        .map(|b| b.sizes.iter().map(|&s| workers(s)).sum::<i64>() as f64 * 0.05)
        .collect();
    let mut lv = vec![vec![0.0f64; nb]; nbk];
    for _ in 0..16 {
        let mut demand: Vec<Vec<f64>> = blocks.iter().map(|b| b.final_demand.clone()).collect();
        for (bi, levels) in lv.iter().enumerate() {
            for (ki, n) in levels.iter().enumerate() {
                if *n > 0.0 {
                    for (g, q) in &cat.buildings[ki].methods[0].inputs {
                        demand[bi][*g as usize] += n * q.to_f64_lossy() * blocks[bi].tfp;
                    }
                }
            }
        }
        let mut next = vec![vec![0.0f64; nb]; nbk];
        let mut raw_alloc: Vec<(f64, Allocation)> = vec![(0.0, Vec::new()); ng];
        let mut residual = vec![0.0f64; ng];
        for g in 0..ng {
            if let Some(ki) = local_producer[g] {
                let per_level = yearly_output(cat, ki, g) / 52.0;
                if per_level > 0.0 {
                    for bi in 0..nbk {
                        next[bi][ki] =
                            f64::max(next[bi][ki], demand[bi][g] / (per_level * blocks[bi].tfp));
                    }
                }
                continue;
            }
            if raw_producers[g].is_empty() {
                continue;
            }
            let world: f64 = demand.iter().map(|d| d[g]).sum();
            // Shares by capacity, mines weighted toward what they already produced in 1925.
            let mut weights = Vec::new();
            for (bi, block) in blocks.iter().enumerate() {
                for &ki in &raw_producers[g] {
                    let cap = block.capacity[ki];
                    if cap > 0.0 {
                        let w = if cat.buildings[ki].resource.is_some() {
                            block.mines_1925[ki] + 0.1 * cap
                        } else {
                            cap
                        };
                        weights.push((bi, ki, w));
                    }
                }
            }
            raw_alloc[g] = (world, weights);
        }
        // Raw goods: shares by capacity; then each block's farms and mines are scaled to the
        // people it has for them, all goods alike; what that leaves unmet goes to blocks with
        // land and people to spare.
        let place = |next: &mut Vec<Vec<f64>>,
                     g: usize,
                     amount: f64,
                     weights: &[(usize, usize, f64)],
                     limit: &dyn Fn(usize, usize) -> f64|
         -> f64 {
            let total: f64 = weights.iter().map(|w| w.2).sum();
            if total <= 0.0 || amount <= 0.0 {
                return 0.0;
            }
            let mut placed = 0.0;
            for (bi, ki, w) in weights {
                let per_level = yearly_output(cat, *ki, g) / 52.0 * blocks[*bi].tfp;
                if per_level <= 0.0 {
                    continue;
                }
                let want = (amount * w / total / per_level).min(limit(*bi, *ki));
                if want > 0.0 {
                    next[*bi][*ki] += want;
                    placed += want * per_level;
                }
            }
            placed
        };
        for g in 0..ng {
            let (world, weights) = &raw_alloc[g];
            let limit = |bi: usize, ki: usize| {
                let floor = blocks[bi].mines_1925[ki] / blocks[bi].tfp;
                (blocks[bi].capacity[ki] / blocks[bi].tfp).max(floor)
            };
            // Farm goods feed their own block first, as far as its land goes: most food was
            // eaten in the country that grew it, and the weekly world market reaches only part
            // of a surplus (`week::TRADE_REACH`). Mines keep the world shares of 1925.
            let mut home = 0.0;
            if raw_producers[g]
                .iter()
                .all(|&ki| cat.buildings[ki].farm.is_some())
            {
                for (bi, d) in demand.iter().enumerate() {
                    let own: Vec<(usize, usize, f64)> =
                        weights.iter().copied().filter(|w| w.0 == bi).collect();
                    home += place(&mut next, g, d[g], &own, &limit);
                }
            }
            let snapshot = next.clone();
            let placed = place(&mut next, g, (world - home).max(0.0), weights, &|bi, ki| {
                (limit(bi, ki) - snapshot[bi][ki]).max(0.0)
            });
            residual[g] = (world - home - placed).max(0.0);
        }
        let used = |next: &Vec<Vec<f64>>, bi: usize, farms: bool| -> f64 {
            next[bi]
                .iter()
                .enumerate()
                .filter(|(ki, _)| {
                    if farms {
                        cat.buildings[*ki].farm.is_some()
                    } else {
                        cat.buildings[*ki].resource.is_some()
                    }
                })
                .map(|(ki, n)| n * cat.buildings[ki].methods[0].workers as f64)
                .sum()
        };
        for bi in 0..nbk {
            for farms in [true, false] {
                let cap = if farms { farm_cap[bi] } else { mine_cap[bi] };
                let u = used(&next, bi, farms);
                if u > cap && u > 0.0 {
                    let k = cap / u;
                    for ki in 0..nb {
                        let is = if farms {
                            cat.buildings[ki].farm.is_some()
                        } else {
                            cat.buildings[ki].resource.is_some()
                        };
                        if !is {
                            continue;
                        }
                        let floor = if farms {
                            0.0
                        } else {
                            blocks[bi].mines_1925[ki] / blocks[bi].tfp
                        };
                        let before = next[bi][ki];
                        let after = (before * k).max(floor.min(before));
                        for (g, q) in &cat.buildings[ki].methods[0].outputs {
                            residual[*g as usize] +=
                                (before - after) * q.to_f64_lossy() * blocks[bi].tfp;
                        }
                        next[bi][ki] = after;
                    }
                }
            }
        }
        for g in 0..ng {
            if residual[g] <= 0.0 {
                continue;
            }
            let (_, weights) = &raw_alloc[g];
            let spare: Vec<(usize, usize, f64)> = weights
                .iter()
                .map(|(bi, ki, w)| {
                    let farms = cat.buildings[*ki].farm.is_some();
                    let cap = if farms { farm_cap[*bi] } else { mine_cap[*bi] };
                    let room = (cap - used(&next, *bi, farms)).max(0.0);
                    (*bi, *ki, w * room)
                })
                .collect();
            let snapshot = next.clone();
            place(&mut next, g, residual[g], &spare, &|bi, ki| {
                let farms = cat.buildings[ki].farm.is_some();
                let cap = if farms { farm_cap[bi] } else { mine_cap[bi] };
                let room = (cap - used(&snapshot, bi, farms)).max(0.0)
                    / cat.buildings[ki].methods[0].workers.max(1) as f64;
                room.min((blocks[bi].capacity[ki] / blocks[bi].tfp - snapshot[bi][ki]).max(0.0))
            });
        }
        lv = next;
    }

    lv
}
