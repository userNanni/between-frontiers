//! Monthly systems: labour (jobs and changes of profession), demography, investment.

use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::schedule::{Access, Cadence, System, TickCtx};
use rayon::prelude::*;

use super::catalogue::Catalogue;
use super::prof::{self, P};
use super::state::Region;
use super::week::ECON;
use super::{Unlocks, WORKER_SHARE, fill, milli, plant_tfp, workers};
use crate::game::GameWorld;

// ---------------------------------------------------------------------------------------------
// Labour

/// People move between professions toward open jobs. Each line: target, source, monthly share
/// of the shortage that can move, and whether literacy limits it.
const PROMOTIONS: [(usize, usize, i64, bool); 12] = [
    (prof::FARM_LABORER, prof::PEASANT, 250, false),
    (prof::LABORER, prof::PEASANT, 200, false),
    (prof::SKILLED, prof::LABORER, 120, true),
    (prof::SKILLED, prof::FARM_LABORER, 60, true),
    (prof::CLERK, prof::LABORER, 100, true),
    (prof::ENGINEER, prof::SKILLED, 60, true),
    (prof::ENGINEER, prof::CLERK, 40, true),
    (prof::PROFESSIONAL, prof::CLERK, 50, true),
    (prof::MANAGER, prof::CLERK, 60, true),
    (prof::MANAGER, prof::SKILLED, 40, true),
    (prof::CAPITALIST, prof::MANAGER, 20, false),
    (prof::LANDOWNER, prof::PEASANT, 10, false),
];
/// Workers in urban self-employment in market `market` (`BuildingDef::informal`).
pub fn informal_jobs(cat: &Catalogue, econ: &super::Economy, market: usize) -> i64 {
    let Some(k) = cat.informal else { return 0 };
    let per_level = cat.buildings[k as usize].methods[0].workers;
    econ.regions
        .iter()
        .filter(|r| r.market as usize == market)
        .flat_map(|r| r.plants.iter())
        .filter(|p| p.kind == k)
        .map(|p| p.levels * per_level)
        .sum()
}

/// Share of the population the state employs, by profession.
pub(crate) const STATE_SHARE: [(usize, i64); 3] = [
    (prof::CIVIL_SERVANT, 12),
    (prof::SOLDIER, 10),
    (prof::CLERGY, 5),
];

pub struct Labour {
    pub cat: Arc<Catalogue>,
}

pub struct LabourChange {
    pops: Vec<(i64, i64)>,
    staffing: Vec<Fixed>,
    /// Levels of urban self-employment the town's jobless labourers fill this month.
    informal: Option<i64>,
}

fn move_people(pops: &mut [(i64, i64)], from: usize, to: usize, workers_moved: i64) {
    if workers_moved <= 0 {
        return;
    }
    let people = (Fixed::from_int(workers_moved) / WORKER_SHARE)
        .to_int_floor()
        .min(pops[from].0);
    pops[from].0 -= people;
    pops[to].0 += people;
}

fn labour_region(cat: &Catalogue, r: &Region, moves: &super::migration::Moves) -> LabourChange {
    let mut required = [0i64; P];
    // Self-employment is sized below, from the labourers the plants leave idle.
    for p in r.plants.iter().filter(|p| Some(p.kind) != cat.informal) {
        let m = &cat.buildings[p.kind as usize].methods[p.method as usize];
        for (pi, n) in &m.jobs {
            required[*pi as usize] += *n * p.levels;
        }
    }
    let mut pops: Vec<(i64, i64)> = r.pops.iter().map(|p| (p.size, 0)).collect();
    // Migration first: idle peasants who left for town, and newcomers from other states.
    super::migration::apply(&mut pops, moves);
    let literacy = r.literacy;
    // The state hires toward its share of the population.
    let total: i64 = pops.iter().map(|p| p.0).sum();
    for (pi, per_mille) in STATE_SHARE {
        let target = total * per_mille / 1000;
        let gap = target - pops[pi].0;
        if gap > 0 {
            let src = if pops[prof::PEASANT].0 > gap {
                prof::PEASANT
            } else {
                prof::LABORER
            };
            let n = (gap / 10).max(1).min(pops[src].0);
            pops[src].0 -= n;
            pops[pi].0 += n;
        }
    }
    // Promotions toward open jobs.
    for (to, from, per_mille, needs_letters) in PROMOTIONS {
        let have = workers(pops[to].0);
        let gap = required[to] - have;
        if gap <= 0 {
            continue;
        }
        let mut n = gap * per_mille / 1000;
        if needs_letters {
            n = (Fixed::from_int(n) * literacy).to_int_floor();
        }
        let available = workers(pops[from].0) - required[from].min(workers(pops[from].0));
        move_people(&mut pops, from, to, n.min(available).max(0));
    }
    // Labourers no plant employs work on their own (street trade, domestic service), where
    // the town pays better than the land and has room for them; plants hire them first.
    let informal = cat.informal.map(|k| {
        let per_level = cat.buildings[k as usize].methods[0].workers.max(1);
        if moves.push_back {
            return 0;
        }
        let idle = (workers(pops[prof::LABORER].0) - required[prof::LABORER]).max(0);
        let levels = idle / per_level;
        for (pi, n) in &cat.buildings[k as usize].methods[0].jobs {
            required[*pi as usize] += *n * levels;
        }
        levels
    });
    // Idle workers drift back: to labourers from skilled jobs, to the land from the rest. Jobless
    // labourers stay in town while it pays better than the land and has room for them.
    for pi in [
        prof::FARM_LABORER,
        prof::LABORER,
        prof::SKILLED,
        prof::CLERK,
        prof::ENGINEER,
        prof::PROFESSIONAL,
        prof::MANAGER,
    ] {
        let mut idle = workers(pops[pi].0) - required[pi];
        // A town that pays and has room keeps its jobless.
        if pi == prof::LABORER && !moves.push_back {
            idle = 0;
        }
        if idle > 0 {
            let back = if matches!(
                pi,
                prof::SKILLED | prof::CLERK | prof::ENGINEER | prof::PROFESSIONAL | prof::MANAGER
            ) {
                prof::LABORER
            } else {
                prof::PEASANT
            };
            move_people(&mut pops, pi, back, idle / 10);
        }
    }
    // Employment and staffing.
    let mut fill_rate = [Fixed::ONE; P];
    for pi in 0..P {
        let w = workers(pops[pi].0);
        let employed = if matches!(pi, prof::CIVIL_SERVANT | prof::SOLDIER | prof::CLERGY) {
            w
        } else {
            w.min(required[pi])
        };
        pops[pi].1 = employed;
        fill_rate[pi] = fill(Fixed::from_int(employed), Fixed::from_int(required[pi]));
    }
    let staffing = r
        .plants
        .iter()
        .map(|p| {
            let m = &cat.buildings[p.kind as usize].methods[p.method as usize];
            if m.workers == 0 || p.levels == 0 {
                return Fixed::ZERO;
            }
            let filled: Fixed = m
                .jobs
                .iter()
                .map(|(pi, n)| fill_rate[*pi as usize] * *n)
                .sum();
            filled / m.workers
        })
        .collect();
    LabourChange {
        pops,
        staffing,
        informal,
    }
}

impl System<GameWorld> for Labour {
    type Changes = Vec<LabourChange>;
    fn name(&self) -> &'static str {
        "economy-labour"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 1 }
    }
    fn access(&self) -> Access {
        Access::new().read(ECON).write(ECON)
    }
    fn compute(&self, _ctx: &TickCtx, world: &GameWorld) -> Vec<LabourChange> {
        let moves = super::migration::plan(&self.cat, &world.econ);
        world
            .econ
            .regions
            .par_iter()
            .zip(moves.par_iter())
            .map(|(r, m)| labour_region(&self.cat, r, m))
            .collect()
    }
    fn apply(&self, _ctx: &TickCtx, changes: Vec<LabourChange>, world: &mut GameWorld) {
        let econ = &mut world.econ;
        for m in &mut econ.markets {
            m.gov_workers = 0;
        }
        for (r, c) in econ.regions.iter_mut().zip(changes) {
            for (pop, (size, employed)) in r.pops.iter_mut().zip(c.pops) {
                // Money follows the people who moved, pro rata.
                if pop.size > 0 && size != pop.size {
                    pop.savings = pop.savings.mul_ratio(size.max(0), pop.size);
                }
                pop.size = size;
                pop.employed = employed;
            }
            for (plant, s) in r.plants.iter_mut().zip(c.staffing) {
                plant.staffing = s;
            }
            if let (Some(k), Some(levels)) = (self.cat.informal, c.informal)
                && (levels > 0 || r.plants.iter().any(|p| p.kind == k))
            {
                let p = r.plant_mut(k);
                p.levels = levels;
                p.method = 0;
            }
            let gov: i64 = [prof::CIVIL_SERVANT, prof::SOLDIER, prof::CLERGY]
                .iter()
                .map(|&p| workers(r.pops[p].size))
                .sum();
            econ.markets[r.market as usize].gov_workers += gov;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Investment

/// A level costs two years of the value it adds at base prices.
pub(crate) const CAPITAL_YEARS: i64 = 3;
/// The least yearly return worth building for.
const MIN_RETURN: Fixed = milli(50);
/// Above the floor, a project must beat the real rate on the country's bonds by this, up to
/// the ceiling.
const RETURN_SPREAD: Fixed = milli(20);
const MAX_HURDLE: Fixed = milli(250);

/// The least return a project needs in market `m`: tight money and risky debt stop building.
pub fn hurdle(m: &super::Market) -> Fixed {
    let real = super::money::real_rate(m) / 100;
    (real + RETURN_SPREAD).clamp(MIN_RETURN, MAX_HURDLE)
}
/// Most new levels a market starts in a month.
const MAX_BUILDS: usize = 400;
/// A quarter of losses closes a level.
const LOSING_WEEKS_TO_CLOSE: u16 = 13;

pub struct Investment {
    pub cat: Arc<Catalogue>,
    pub tree: Option<Arc<crate::research::TechTree>>,
}

pub struct Build {
    region: usize,
    kind: u16,
    method: u8,
    /// Equipment bought directly, and construction points for the queue.
    goods: Vec<(u16, Fixed)>,
    points: Fixed,
    cost: Fixed,
    /// The farm (`FARMS` index) whose land this level clears from pasture or forest.
    clears: Option<u8>,
}

pub struct InvestmentChanges {
    builds: Vec<Build>,
    /// Money each market's pool has left this month after the works it started at home: the
    /// reading point for capital that crosses borders (not applied; the hash does not see it).
    pub leftover: Vec<Fixed>,
    closes: Vec<(usize, u16)>,
    public: Vec<super::state::Project>,
}

/// Value added per level per week at base prices of a method (used for capital cost).
pub(crate) fn base_value_added(cat: &Catalogue, kind: usize, method: usize) -> Fixed {
    cat.buildings[kind].methods[method]
        .value_added(|g| cat.goods[g].base_price)
        .max(Fixed::ZERO)
}

/// Capital a level of `method` costs: `CAPITAL_YEARS` of the value it adds at base prices and at
/// the productivity `tfp` it will run with. A level that makes more needs more machines: the
/// capital-output ratio, not the capital per level, is alike across countries.
pub fn level_capital(cat: &Catalogue, kind: usize, method: usize, tfp: Fixed) -> Fixed {
    base_value_added(cat, kind, method) * tfp * (52 * CAPITAL_YEARS)
}

/// Average weekly pay of a worker in each market: what a method that needs fewer hands saves.
pub fn market_wages(econ: &crate::econ::Economy) -> Vec<Fixed> {
    let mut pay = vec![Fixed::ZERO; econ.markets.len()];
    let mut hands = vec![0i64; econ.markets.len()];
    for r in &econ.regions {
        for (pi, p) in r.pops.iter().enumerate() {
            if matches!(
                pi,
                prof::CAPITALIST
                    | prof::LANDOWNER
                    | prof::CIVIL_SERVANT
                    | prof::CLERGY
                    | prof::SOLDIER
            ) || p.employed <= 0
            {
                continue;
            }
            pay[r.market as usize] += p.income;
            hands[r.market as usize] += p.employed;
        }
    }
    pay.iter()
        .zip(hands)
        .map(|(p, h)| if h > 0 { *p / h } else { Fixed::ZERO })
        .collect()
}

/// Best method a building can use at market prices, with its weekly owners' profit per level:
/// value added less the market wage of the workers it needs. Mechanizing pays where labour is
/// dear and not where it is cheap.
#[allow(clippy::too_many_arguments)]
pub fn best_method(
    cat: &Catalogue,
    kind: usize,
    unlocks: &Unlocks<'_>,
    country: usize,
    price: &[Fixed],
    buy: &[Fixed],
    tfp: Fixed,
    wage: Fixed,
) -> Option<(u8, Fixed)> {
    let b = &cat.buildings[kind];
    let mut best: Option<(u8, Fixed)> = None;
    for (mi, m) in b.methods.iter().enumerate() {
        if !unlocks.method(cat, country, kind, mi) {
            continue;
        }
        // A method whose inputs the market cannot supply is not an option.
        if m.inputs
            .iter()
            .any(|(g, _)| buy[*g as usize] < milli(500) || !unlocks.goods[*g as usize])
        {
            continue;
        }
        let profit = m.profit(price, tfp, wage);
        if best.is_none_or(|(_, p)| profit > p) {
            best = Some((mi as u8, profit));
        }
    }
    best
}

const BUSY_UTILIZATION: Fixed = milli(900);

/// Clearing for a crop costs as much again as the farm level that will work the land.
const CLEARING: Fixed = Fixed::ONE;

/// Opens a level of `farm` on pasture and forest, in proportion to what is left of each: the
/// level moves from the frontier to the open land, and the ranches and forestry of the state
/// keep the share of their land that is left.
fn clear_land(cat: &Catalogue, r: &mut Region, farm: u8) {
    let Some(f) = r.frontier.iter_mut().find(|f| f.0 == farm && f.1 > 0) else {
        return;
    };
    f.1 -= 1;
    match r.land.iter_mut().find(|l| l.0 == farm) {
        Some(l) => l.1 += 1,
        None => r.land.push((farm, 1)),
    }
    let km2 = cat.farm_km2.get(usize::from(farm)).copied().unwrap_or(0);
    let wild = r.wild_km2[0] + r.wild_km2[1];
    if wild <= 0 || km2 <= 0 {
        return;
    }
    let from_pasture = (km2 * r.wild_km2[0] / wild).min(r.wild_km2[0]);
    let from_forest = (km2 - from_pasture).min(r.wild_km2[1]);
    for (i, (taken, kind)) in [
        (from_pasture, cat.pasture_farm),
        (from_forest, cat.forest_farm),
    ]
    .into_iter()
    .enumerate()
    {
        let before = r.wild_km2[i];
        r.wild_km2[i] -= taken;
        r.cleared_km2[i] += taken;
        if let Some(k) = kind
            && let Some(l) = r.land.iter_mut().find(|l| l.0 == k)
            && before > 0
        {
            l.1 = l.1 * r.wild_km2[i] / before;
        }
    }
}

/// Money of the pool already promised: construction goods still to buy and the private works
/// in the queue.
pub fn committed(cat: &Catalogue, m: &super::Market) -> Fixed {
    m.construction
        .iter()
        .zip(&m.price)
        .map(|(q, p)| *q * *p)
        .sum::<Fixed>()
        + cat.construction.map_or(Fixed::ZERO, |cg| {
            m.queue
                .iter()
                .filter(|p| !p.public)
                .map(|p| p.left * m.price[cg as usize])
                .sum()
        })
}

impl System<GameWorld> for Investment {
    type Changes = InvestmentChanges;
    fn name(&self) -> &'static str {
        "economy-investment"
    }
    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 8 }
    }
    fn access(&self) -> Access {
        Access::new().read(ECON).write(ECON)
    }
    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> InvestmentChanges {
        let cat = &*self.cat;
        let econ = &world.econ;
        let year = ctx.date.year;
        let unlocks = Unlocks::new(cat, year, self.tree.as_deref(), &world.research);
        let national = super::local::national(
            cat,
            self.tree.as_deref(),
            &world.research,
            econ.markets.len(),
        );
        let wages = market_wages(econ);
        let mut closes = Vec::new();
        for (ri, r) in econ.regions.iter().enumerate() {
            for p in &r.plants {
                if p.levels > 0 && p.losing_weeks >= LOSING_WEEKS_TO_CLOSE {
                    closes.push((ri, p.kind));
                }
            }
        }
        // Utilization of each building across its market: nobody adds levels of a kind the
        // market already leaves idle. Levels under construction count as idle.
        let nk = cat.buildings.len();
        let mut used = vec![vec![Fixed::ZERO; nk]; econ.markets.len()];
        let mut built = vec![vec![Fixed::ZERO; nk]; econ.markets.len()];
        for r in &econ.regions {
            for p in &r.plants {
                let levels = Fixed::from_int(p.levels);
                used[r.market as usize][p.kind as usize] += levels * p.utilization;
                built[r.market as usize][p.kind as usize] += levels;
            }
        }
        let mut queued: Vec<Vec<(u16, i64)>> = vec![Vec::new(); econ.regions.len()];
        for (mi, m) in econ.markets.iter().enumerate() {
            for p in &m.queue {
                built[mi][p.kind as usize] += Fixed::ONE;
                let q = &mut queued[p.region as usize];
                match q.iter_mut().find(|(k, _)| *k == p.kind) {
                    Some(e) => e.1 += 1,
                    None => q.push((p.kind, 1)),
                }
            }
        }
        let construction = cat.construction;
        let busy: Vec<Vec<bool>> = used
            .iter()
            .zip(&built)
            .map(|(u, b)| {
                u.iter()
                    .zip(b)
                    .map(|(u, b)| *b <= Fixed::ZERO || *u >= *b * BUSY_UTILIZATION)
                    .collect()
            })
            .collect();

        // Candidates per market, ranked by yearly return.
        let mut by_market: Vec<Vec<(Fixed, Build)>> =
            (0..econ.markets.len()).map(|_| Vec::new()).collect();
        let candidates: Vec<Vec<(Fixed, Build)>> = econ
            .regions
            .par_iter()
            .enumerate()
            .map(|(ri, r)| {
                let m = &econ.markets[r.market as usize];
                let view = super::local::View::last_week(cat, m, r, &national[r.market as usize]);
                // Hands a new plant can hire here: idle peasants, jobless labourers and those
                // working on their own in town, who take a plant's wage over the street.
                let idle_of = |pi: usize| (workers(r.pops[pi].size) - r.pops[pi].employed).max(0);
                let informal: i64 = cat.informal.map_or(0, |k| {
                    r.plants
                        .iter()
                        .filter(|p| p.kind == k)
                        .map(|p| p.levels * cat.buildings[k as usize].methods[0].workers)
                        .sum()
                });
                let idle: i64 = idle_of(prof::PEASANT) + idle_of(prof::LABORER) + informal;
                let mut out = Vec::new();
                for (kind, b) in cat.buildings.iter().enumerate() {
                    if b.public || b.informal {
                        continue;
                    }
                    let current = r.plants.iter().find(|p| p.kind as usize == kind);
                    let existing = current.map_or(0, |p| p.levels)
                        + queued[ri]
                            .iter()
                            .find(|(k, _)| *k as usize == kind)
                            .map_or(0, |e| e.1);
                    // A good that clears state by state is judged by the state: a railway is
                    // built where trains are full, not where the country's are.
                    let local = super::local::local_output(cat, kind)
                        .filter(|&li| !national[r.market as usize][li]);
                    let busy_here = match local {
                        Some(_) => current
                            .is_none_or(|p| p.levels <= 0 || p.utilization >= BUSY_UTILIZATION),
                        None => busy[r.market as usize][kind],
                    };
                    if !busy_here {
                        continue;
                    }
                    let ceiling = if let Some(res) = b.resource {
                        r.deposits
                            .iter()
                            .filter(|d| d.0 == res && d.1 <= year)
                            .map(|d| d.2)
                            .sum::<i64>()
                    } else if let Some(farm) = b.farm {
                        r.land
                            .iter()
                            .filter(|l| l.0 == farm)
                            .map(|l| l.1)
                            .sum::<i64>()
                    } else {
                        i64::MAX
                    };
                    // A farm on all its open land can clear pasture or forest suited to its crop.
                    let clears = b.farm.filter(|&farm| {
                        existing >= ceiling && r.frontier.iter().any(|f| f.0 == farm && f.1 > 0)
                    });
                    if existing >= ceiling && clears.is_none() {
                        continue;
                    }
                    let Some((method, profit)) = best_method(
                        cat,
                        kind,
                        &unlocks,
                        r.market as usize,
                        &view.price,
                        &view.buy,
                        plant_tfp(r, m, b),
                        wages[r.market as usize],
                    ) else {
                        continue;
                    };
                    if cat.buildings[kind].methods[method as usize].workers > idle {
                        continue;
                    }
                    let level = level_capital(cat, kind, 0, plant_tfp(r, m, b));
                    if level <= Fixed::ZERO {
                        continue;
                    }
                    // Clearing is work: as much again as the farm level, all of it building
                    // work, in construction points.
                    let clearing = if clears.is_some() {
                        level * CLEARING
                    } else {
                        Fixed::ZERO
                    };
                    let cost = level + clearing;
                    let ret = profit * 52 / cost;
                    if ret < hurdle(m) {
                        continue;
                    }
                    // The building itself becomes points at the base price of construction;
                    // without a building trade in the catalogue it is bought as goods.
                    let mut points = Fixed::ZERO;
                    let goods = b
                        .construction
                        .iter()
                        .filter_map(|&(g, share, work)| {
                            if let Some(cg) = construction
                                && work
                            {
                                points += level.mul_ratio(share, 1000)
                                    / cat.goods[cg as usize].base_price;
                                return None;
                            }
                            Some((g, level.mul_ratio(share, 1000) / m.price[g as usize]))
                        })
                        .collect();
                    if let Some(cg) = construction {
                        points += clearing / cat.goods[cg as usize].base_price;
                    }
                    out.push((
                        ret,
                        Build {
                            region: ri,
                            kind: kind as u16,
                            method,
                            goods,
                            points,
                            cost,
                            clears,
                        },
                    ));
                }
                out
            })
            .collect();
        for (r, list) in econ.regions.iter().zip(candidates) {
            by_market[r.market as usize].extend(list);
        }
        let mut builds = Vec::new();
        let mut leftover = Vec::with_capacity(econ.markets.len());
        for (mi, mut list) in by_market.into_iter().enumerate() {
            let m = &econ.markets[mi];
            let mut budget = m.pool - committed(cat, m);
            // Highest return first; ties by region then kind, so the order never depends on threads.
            list.sort_by(|a, b| {
                b.0.cmp(&a.0)
                    .then(a.1.region.cmp(&b.1.region))
                    .then(a.1.kind.cmp(&b.1.kind))
            });
            let mut started = 0;
            for (_, build) in list.into_iter().take(MAX_BUILDS * 4) {
                if started >= MAX_BUILDS {
                    break;
                }
                if build.cost > budget {
                    continue;
                }
                budget -= build.cost;
                builds.push(build);
                started += 1;
            }
            leftover.push(budget);
        }
        let public = super::public::public_works(cat, econ, &unlocks);
        InvestmentChanges {
            builds,
            leftover,
            closes,
            public,
        }
    }
    fn apply(&self, _ctx: &TickCtx, changes: InvestmentChanges, world: &mut GameWorld) {
        let econ = &mut world.econ;
        for (ri, kind) in changes.closes {
            // A tenth of a losing plant closes each month, at least one level.
            let p = econ.regions[ri].plant_mut(kind);
            p.levels -= (p.levels / 10).max(1);
            p.losing_weeks = 0;
        }
        for b in changes.builds {
            if let Some(farm) = b.clears {
                clear_land(&self.cat, &mut econ.regions[b.region], farm);
            }
            let market = econ.regions[b.region].market as usize;
            for (g, q) in b.goods {
                econ.markets[market].construction[g as usize] += q;
            }
            let _ = b.cost;
            if b.points > Fixed::ZERO {
                // The level runs when the building trade has delivered its points.
                econ.markets[market].queue.push(super::state::Project {
                    region: b.region as u32,
                    kind: b.kind,
                    method: b.method,
                    public: false,
                    points: b.points,
                    left: b.points,
                });
            } else {
                let p = econ.regions[b.region].plant_mut(b.kind);
                if p.levels == 0 {
                    p.method = b.method;
                }
                p.levels += 1;
            }
        }
        for p in changes.public {
            let market = econ.regions[p.region as usize].market as usize;
            econ.markets[market].queue.push(p);
        }
    }
}
