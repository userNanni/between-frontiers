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
/// Schooling a profession asks of who enters it, with schooling by cohort: 0 primary, 1
/// secondary, 2 tertiary; `None` for professions that ask none.
pub const SCHOOLING_ASKED: [Option<usize>; P] = {
    let mut a = [None; P];
    a[prof::SKILLED] = Some(0);
    a[prof::CLERK] = Some(0);
    a[prof::CIVIL_SERVANT] = Some(0);
    a[prof::MANAGER] = Some(1);
    a[prof::ENGINEER] = Some(2);
    a[prof::PROFESSIONAL] = Some(2);
    a
};

/// Workers of a region with at least each level of schooling who do not yet work in a
/// profession that asks for it, given `qualified`, the share of those 15–64 with at least
/// primary, secondary and tertiary (`demography::education`).
pub fn graduates_free(pops: &[(i64, i64)], qualified: [Fixed; 3]) -> [i64; 3] {
    let total: i64 = pops.iter().map(|p| p.0).sum();
    std::array::from_fn(|l| {
        let pool = (Fixed::from_int(workers(total)) * qualified[l]).to_int_floor();
        let taken: i64 = pops
            .iter()
            .enumerate()
            .filter(|(pi, _)| SCHOOLING_ASKED[*pi].is_some_and(|a| a >= l))
            .map(|(_, p)| workers(p.0))
            .sum();
        (pool - taken).max(0)
    })
}

/// Open jobs of each profession that wait for graduates: jobs the plants offer beyond the
/// workers the profession has, in professions whose level of schooling has no free graduates
/// left in the region. What the screen shows as "missing graduates".
pub fn graduates_missing(cat: &Catalogue, r: &Region, qualified: [Fixed; 3]) -> [i64; P] {
    let mut required = [0i64; P];
    for p in r.plants.iter().filter(|p| Some(p.kind) != cat.informal) {
        let m = &cat.buildings[p.kind as usize].methods[p.method as usize];
        for (pi, n) in &m.jobs {
            required[*pi as usize] += *n * p.levels;
        }
    }
    let pops: Vec<(i64, i64)> = r.pops.iter().map(|p| (p.size, 0)).collect();
    let free = graduates_free(&pops, qualified);
    std::array::from_fn(|pi| match SCHOOLING_ASKED[pi] {
        Some(l) if free[l] == 0 => (required[pi] - workers(pops[pi].0)).max(0),
        _ => 0,
    })
}

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
    /// Jobs follow use (`game::Switches::jobs_follow_use`): a plant that sells less than it
    /// could make lays off part of its crew each month, toward the jobs of its levels ×
    /// utilization, instead of paying a full crew out of what it sells.
    pub jobs_follow_use: bool,
}

/// Share of the gap between the crew a plant has and the crew its use needs that it lays off
/// in a month: 15 %, a half-life of about four months. Firms hoard labour when orders fall
/// (training, firing costs, hope of recovery) and adjust employment to output with a lag of a
/// few months in manufacturing (Hamermesh, *Labor Demand*, 1993, on dynamic labour demand; Fay and Medoff, AER
/// 75(4), 1985, on labour hoarding in US plants). A design value within that range.
pub const LAYOFF_PER_MONTH: Fixed = milli(150);

/// Jobs a plant fills at its use: the jobs of its levels × utilization, rounded up.
pub(crate) fn jobs_in_use(jobs: i64, utilization: Fixed) -> i64 {
    -(Fixed::from_int(-jobs) * utilization.min(Fixed::ONE)).to_int_floor()
}

pub struct LabourChange {
    pops: Vec<(i64, i64)>,
    staffing: Vec<Fixed>,
    /// Levels of urban self-employment the town's jobless labourers fill this month.
    informal: Option<i64>,
    /// Pioneers: land cleared toward the next level, and the levels they open this month.
    pioneers: (Fixed, Option<(u8, i64)>),
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

/// Jobs follow use (`game::Switches::jobs_follow_use`): by profession, the plants hire at once
/// up to the jobs their use needs and lay off a share of the excess a month
/// (`LAYOFF_PER_MONTH`), never above the jobs of their levels.
fn jobs_after_layoffs(cat: &Catalogue, r: &Region, required: &mut [i64; P]) {
    let mut in_use = [0i64; P];
    let mut informal = [0i64; P];
    for p in &r.plants {
        let m = &cat.buildings[p.kind as usize].methods[p.method as usize];
        let own = Some(p.kind) == cat.informal;
        for (pi, n) in &m.jobs {
            if own {
                informal[*pi as usize] += *n * p.levels;
            } else {
                in_use[*pi as usize] += jobs_in_use(*n * p.levels, p.utilization);
            }
        }
    }
    for pi in 0..P {
        if required[pi] == 0 {
            continue;
        }
        // The crew the plants held last month, without the self-employed.
        let held = (r.pops[pi].employed - informal[pi]).clamp(0, required[pi]);
        required[pi] = crew_next_month(held, in_use[pi]);
    }
}

/// The crew a plant keeps next month: up to what its use needs at once, down by a share of the
/// excess (`LAYOFF_PER_MONTH`, rounded up so that the last worker goes too).
fn crew_next_month(held: i64, target: i64) -> i64 {
    if target >= held {
        target
    } else {
        held - jobs_in_use(held - target, LAYOFF_PER_MONTH)
    }
}

fn labour_region(
    cat: &Catalogue,
    r: &Region,
    moves: &super::migration::Moves,
    soldiers: Option<i64>,
    qualified: Option<[Fixed; 3]>,
    jobs_follow_use: bool,
) -> LabourChange {
    let mut required = [0i64; P];
    // Self-employment is sized below, from the labourers the plants leave idle.
    for p in r.plants.iter().filter(|p| Some(p.kind) != cat.informal) {
        let m = &cat.buildings[p.kind as usize].methods[p.method as usize];
        for (pi, n) in &m.jobs {
            required[*pi as usize] += *n * p.levels;
        }
    }
    if jobs_follow_use {
        jobs_after_layoffs(cat, r, &mut required);
    }
    let mut pops: Vec<(i64, i64)> = r.pops.iter().map(|p| (p.size, 0)).collect();
    // Migration first: idle peasants who left for town, and newcomers from other states.
    super::migration::apply(&mut pops, moves);
    let literacy = r.literacy;
    // The state hires toward its share of the population.
    let total: i64 = pops.iter().map(|p| p.0).sum();
    for (pi, per_mille) in STATE_SHARE {
        let target = total * per_mille / 1000;
        // With forces, the soldiers are their men, hired and sent home (`war::place_soldiers`).
        let target = match (pi, soldiers) {
            (prof::SOLDIER, Some(men)) => {
                (Fixed::from_int(men) / super::WORKER_SHARE).to_int_round()
            }
            _ => target,
        };
        let gap = target - pops[pi].0;
        if gap < 0 && pi == prof::SOLDIER && soldiers.is_some() {
            let n = (-gap / 10).max(1).min(pops[pi].0);
            pops[pi].0 -= n;
            pops[prof::PEASANT].0 += n;
        }
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
        match (qualified, SCHOOLING_ASKED[to]) {
            // With schooling by cohort: no more than the graduates of the level still free.
            (Some(q), Some(l)) => n = n.min(graduates_free(&pops, q)[l]),
            (Some(_), None) => {}
            (None, _) => {
                if needs_letters {
                    n = (Fixed::from_int(n) * literacy).to_int_floor();
                }
            }
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
        pioneers: (r.pioneer_km2, None),
    }
}

/// Land a pioneer clears with his own labour in a year, km² per idle peasant worker: 1.5 ha.
/// Colonist families on the Transamazon cleared about 10 ha every other year in their first
/// eight years (Walker, Homma, Scatena et al., Revista de Economia e Sociologia Rural 35(2),
/// pp. 121-122); settlers in northern British Columbia, clearing by hand, 2 to 3.7 acres a year
/// per farm (1943-45). Two to three workers a family.
const PIONEER_KM2_PER_WORKER: Fixed = milli(15);

/// Peasants without work or free land clear the state's frontier for their own plot, with
/// their labour and no capital: as far as the land law lets them keep it (`frontier_access`)
/// and as far as transport reaches the place. Returns the land cleared toward the next level
/// and the levels opened, of the staple farm with the most frontier.
pub fn pioneers(cat: &Catalogue, r: &Region, m: &super::Market) -> (Fixed, Option<(u8, i64)>) {
    let Some(farm) = r
        .frontier
        .iter()
        .filter(|f| f.1 > 0 && cat.staple_farms.iter().any(|s| s.0 == f.0))
        .max_by_key(|f| (f.1, std::cmp::Reverse(f.0)))
        .map(|f| f.0)
    else {
        return (r.pioneer_km2, None);
    };
    let km2 = cat.farm_km2.get(usize::from(farm)).copied().unwrap_or(0);
    if km2 <= 0 || !super::week::short_of_land(cat, r, m) {
        return (r.pioneer_km2, None);
    }
    let p = &r.pops[prof::PEASANT];
    let idle = (workers(p.size) - p.employed).max(0);
    let reach = cat
        .freight
        .and_then(|g| cat.local.iter().position(|&l| l == g))
        .and_then(|li| r.local.get(li))
        .map_or(Fixed::ONE, |l| l.buy);
    let cleared = PIONEER_KM2_PER_WORKER.mul_ratio(idle, 12) * m.laws.of(r).frontier_access * reach;
    let total = r.pioneer_km2 + cleared;
    let open = r.frontier.iter().find(|f| f.0 == farm).map_or(0, |f| f.1);
    let levels = (total / km2).to_int_floor().min(open);
    (
        total - Fixed::from_int(levels * km2),
        Some((farm, levels)).filter(|x| x.1 > 0),
    )
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
        Access::new()
            .read(ECON)
            .read(crate::war::WAR)
            .write(ECON)
            .read(crate::demography::DEMOGRAPHY)
    }
    fn compute(&self, _ctx: &TickCtx, world: &GameWorld) -> Vec<LabourChange> {
        let moves = super::migration::plan(&self.cat, &world.econ);
        world
            .econ
            .regions
            .par_iter()
            .zip(moves.par_iter())
            .enumerate()
            .map(|(ri, (r, mv))| {
                let qualified = crate::demography::education::qualified(world, ri);
                let mut c = labour_region(
                    &self.cat,
                    r,
                    mv,
                    world.war.soldiers(ri),
                    qualified,
                    self.jobs_follow_use,
                );
                c.pioneers = pioneers(&self.cat, r, &world.econ.markets[r.market as usize]);
                c
            })
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
            r.pioneer_km2 = c.pioneers.0;
            if let Some((farm, levels)) = c.pioneers.1 {
                for _ in 0..levels {
                    clear_land(&self.cat, r, farm);
                }
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
    /// `game::Switches::small_market_wage_floor`.
    pub wage_floor: bool,
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

/// Capital a level of `method` costs to build or to switch to. Where the building declares
/// capital by method (`Method::capital`), the first method's level times the method's share:
/// a nuclear plant costs more than a gas turbine for the same work. Elsewhere, as before:
/// `level_capital` of the method.
pub fn method_capital(cat: &Catalogue, kind: usize, method: usize, tfp: Fixed) -> Fixed {
    match cat.buildings[kind].methods[method].capital {
        Some(share) => level_capital(cat, kind, 0, tfp) * share,
        None => level_capital(cat, kind, method, tfp),
    }
}

/// What the choice of method weighs: the profit of a level over the capital it ties up, where
/// the building declares capital by method; the profit itself elsewhere.
pub fn method_score(m: &super::catalogue::Method, profit: Fixed) -> Fixed {
    match m.capital {
        Some(share) => profit / share,
        None => profit,
    }
}

/// Average weekly pay of a worker in each market: what a method that needs fewer hands saves.
/// With `floor` (`game::Switches::small_market_wage_floor`), a market too small to have a wage
/// of its own reads the lowest of the others.
pub fn market_wages(econ: &crate::econ::Economy, floor: bool) -> Vec<Fixed> {
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
    let mut wages: Vec<Fixed> = pay
        .iter()
        .zip(&hands)
        .map(|(p, h)| if *h > 0 { *p / *h } else { Fixed::ZERO })
        .collect();
    // A market with fewer workers than a few plant levels (San Marino, Andorra, Liechtenstein
    // in 1925) has no wage of its own to read: its two or three plants run at the floor of use
    // with full crews and pay almost nothing. Its people could work across the border, so it
    // reads the lowest wage among the markets with a reading when its own is lower.
    if !floor {
        return wages;
    }
    let floor = wages
        .iter()
        .zip(&hands)
        .filter(|(w, h)| **h >= MIN_HANDS_FOR_A_WAGE && **w > Fixed::ZERO)
        .map(|(w, _)| *w)
        .min();
    // In full below half the threshold, fading linearly to nothing at it: a market that grows
    // past it from one month to the next sees no jump in its wage.
    if let Some(floor) = floor {
        for (w, h) in wages.iter_mut().zip(&hands) {
            *w = small_market_wage(*w, floor, *h);
        }
    }
    wages
}

/// The wage a market with `hands` plant workers reads, its own pay being `own`: the floor in
/// full below half of `MIN_HANDS_FOR_A_WAGE`, fading linearly to its own pay at it.
fn small_market_wage(own: Fixed, floor: Fixed, hands: i64) -> Fixed {
    if own >= floor || hands >= MIN_HANDS_FOR_A_WAGE {
        return own;
    }
    let half = MIN_HANDS_FOR_A_WAGE / 2;
    let weight = if hands <= half {
        Fixed::ONE
    } else {
        Fixed::from_ratio(MIN_HANDS_FOR_A_WAGE - hands, half)
    };
    own + (floor - own) * weight
}

/// Workers in plants below which a market's average pay is not read in full as its wage
/// (`market_wages`): about ten plant levels.
pub const MIN_HANDS_FOR_A_WAGE: i64 = 10_000;

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
    let mut best: Option<(u8, Fixed, Fixed)> = None;
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
        let score = method_score(m, profit);
        if best.is_none_or(|(_, _, s)| score > s) {
            best = Some((mi as u8, profit, score));
        }
    }
    best.map(|(mi, profit, _)| (mi, profit))
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
        Access::new()
            .read(ECON)
            .write(ECON)
            .read(crate::demography::DEMOGRAPHY)
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
        let wages = market_wages(econ, self.wage_floor);
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
                    // A new level is built in the building's first method's terms, at the
                    // capital of the method it will run.
                    let level = match b.methods[method as usize].capital {
                        Some(share) => level_capital(cat, kind, 0, plant_tfp(r, m, b)) * share,
                        None => level_capital(cat, kind, 0, plant_tfp(r, m, b)),
                    };
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
        let schools: Option<Vec<[i64; 3]>> = (world.education.regions.len() == econ.regions.len())
            .then(|| {
                (0..econ.regions.len())
                    .map(|ri| crate::demography::education::places_needed(world, ri))
                    .collect()
            });
        let public =
            super::public::public_works(cat, econ, &unlocks, schools.as_deref(), self.wage_floor);
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

#[cfg(test)]
mod capital_tests {
    use super::*;
    use crate::econ::catalogue::Method;

    #[test]
    fn without_declared_capital_the_choice_is_the_profit_itself() {
        let m = Method::default();
        let profit = Fixed::from_int(1234);
        assert_eq!(method_score(&m, profit), profit);
        let unit = Method {
            capital: Some(Fixed::ONE),
            ..Method::default()
        };
        assert_eq!(method_score(&unit, profit), profit);
    }

    #[test]
    fn a_method_that_ties_up_more_capital_needs_more_profit() {
        // A gas turbine at a third of the capital beats a reactor earning twice as much.
        let gas = Method {
            capital: Some(milli(385)),
            ..Method::default()
        };
        let nuclear = Method {
            capital: Some(milli(3960)),
            ..Method::default()
        };
        assert!(
            method_score(&gas, Fixed::from_int(25)) > method_score(&nuclear, Fixed::from_int(74))
        );
    }
}

#[cfg(test)]
mod layoff_tests {
    use super::*;

    #[test]
    fn a_plant_at_the_floor_of_use_lays_off_gradually() {
        // A level of 1,000 jobs at 5 % use needs 50: the crew falls 15 % of the excess a month.
        let target = jobs_in_use(1000, milli(50));
        assert_eq!(target, 50);
        let mut crew = 1000;
        let mut months = 0;
        while crew > target + 100 {
            crew = crew_next_month(crew, target);
            months += 1;
        }
        // About four months of half-life: within 100 of the need in a year or so, not in a week.
        assert!((10..=14).contains(&months), "{months} months");
        assert_eq!(crew_next_month(1000, 1000), 1000);
        assert_eq!(crew_next_month(51, 50), 50, "the last worker goes too");
    }

    #[test]
    fn the_wage_floor_of_a_small_market_fades_without_a_jump() {
        let (own, floor) = (milli(1_000), Fixed::from_int(20));
        assert_eq!(small_market_wage(own, floor, 2_000), floor);
        assert_eq!(small_market_wage(own, floor, 5_000), floor);
        let mid = small_market_wage(own, floor, 7_500);
        assert!(mid > own && mid < floor);
        // One worker either side of the threshold: no jump.
        let below = small_market_wage(own, floor, MIN_HANDS_FOR_A_WAGE - 1);
        assert!(below - own < milli(10), "{}", below.to_f64_lossy());
        assert_eq!(small_market_wage(own, floor, MIN_HANDS_FOR_A_WAGE), own);
        assert_eq!(
            small_market_wage(Fixed::from_int(30), floor, 100),
            Fixed::from_int(30)
        );
    }

    #[test]
    fn hiring_up_to_the_use_is_at_once() {
        assert_eq!(crew_next_month(200, 900), 900);
        assert_eq!(jobs_in_use(1000, Fixed::ONE), 1000);
        assert_eq!(jobs_in_use(999, milli(500)), 500, "rounded up");
    }
}
