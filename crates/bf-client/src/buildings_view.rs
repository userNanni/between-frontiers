//! What the player's country has built: every building kind with its levels, workers, output,
//! use and profit, summed over the states and state by state (the economy window's buildings
//! tab). Read only, once a month and when the works queue changes.
//!
//! Per plant the simulation keeps its levels, one method, staffing, the share of inputs it got,
//! utilization and the owners' weekly profit (`econ::Plant`). Workers and output follow from
//! them as the week computes them (`week::plant_run`): workers are the method's jobs × levels ×
//! staffing; output is the method's outputs × levels × staffing × productivity × utilization ×
//! inputs obtained, valued at the market's prices.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// One building kind in one state.
#[derive(Clone, Debug, PartialEq)]
pub struct StateRow {
    /// Region index in the simulation.
    pub region: u32,
    /// Map state id.
    pub state: u16,
    pub levels: i64,
    /// Levels under construction here (public and private works).
    pub building: i64,
    /// Method key the plant runs.
    pub method: String,
    pub workers: f64,
    /// Value of a year's output at this month's prices.
    pub output: f64,
    /// Owners' profit over a year at last week's pace (negative: a loss).
    pub profit: f64,
    pub utilization: f64,
    pub staffing: f64,
    /// Share of its inputs the plant obtained.
    pub inputs: f64,
}

/// One building kind over the whole country.
#[derive(Clone, Debug, PartialEq)]
pub struct KindRow {
    pub kind: u16,
    pub key: String,
    /// Index into `economy::BUILDING_GROUPS`.
    pub group: u8,
    /// Built and paid for by the state (schools, hospitals, roads).
    pub public: bool,
    pub levels: i64,
    pub building: i64,
    pub workers: f64,
    pub output: f64,
    pub profit: f64,
    /// Means weighted by levels.
    pub utilization: f64,
    pub staffing: f64,
    /// Its states, most levels first.
    pub states: Vec<StateRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuildingsView {
    pub month: (i32, u8),
    /// Length of the works queue when read: a work that finished or started reads it again.
    pub queue: usize,
    pub kinds: Vec<KindRow>,
}

impl BuildingsView {
    /// Totals by group (`economy::BUILDING_GROUPS` order): levels, workers, output, profit.
    pub fn groups(&self) -> Vec<[f64; 4]> {
        let mut out = vec![[0.0; 4]; bf_sim::economy::BUILDING_GROUPS.len()];
        for k in &self.kinds {
            if let Some(g) = out.get_mut(usize::from(k.group)) {
                g[0] += k.levels as f64;
                g[1] += k.workers;
                g[2] += k.output;
                g[3] += k.profit;
            }
        }
        out
    }
}

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// The player's buildings, or `None` without a player or an economy.
pub fn read(world: &GameWorld, tag: &str, cat: &Catalogue, date: Date) -> Option<BuildingsView> {
    let ci = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let m = world.econ.markets.get(ci)?;
    let mut kinds: Vec<Option<KindRow>> = vec![None; cat.buildings.len()];
    let mut queued: std::collections::BTreeMap<(u32, u16), i64> = Default::default();
    for p in &m.queue {
        *queued.entry((p.region, p.kind)).or_default() += 1;
    }
    let price = |g: u16| m.price.get(usize::from(g)).copied().unwrap_or(Fixed::ZERO);
    let mut add = |ri: u32,
                   r: &bf_sim::econ::Region,
                   kind: u16,
                   plant: Option<&bf_sim::econ::Plant>| {
        let Some(b) = cat.buildings.get(usize::from(kind)) else {
            return;
        };
        let building = queued.get(&(ri, kind)).copied().unwrap_or(0);
        let (levels, row) = match plant {
            Some(p) if p.levels > 0 => {
                let Some(method) = b.methods.get(usize::from(p.method)) else {
                    return;
                };
                let tfp = bf_sim::econ::plant_tfp(r, m, b);
                let scale =
                    Fixed::from_int(p.levels) * p.staffing * tfp * p.utilization * p.input_ratio;
                let output: Fixed = method
                    .outputs
                    .iter()
                    .map(|(g, q)| *q * scale * price(*g))
                    .sum();
                let workers = method.workers as f64 * p.levels as f64 * f(p.staffing);
                (
                    p.levels,
                    StateRow {
                        region: ri,
                        state: r.state,
                        levels: p.levels,
                        building,
                        method: method.key.clone(),
                        workers,
                        output: f(output) * 52.0,
                        profit: f(p.profit) * 52.0,
                        utilization: f(p.utilization),
                        staffing: f(p.staffing),
                        inputs: f(p.input_ratio),
                    },
                )
            }
            _ if building > 0 => (
                0,
                StateRow {
                    region: ri,
                    state: r.state,
                    levels: 0,
                    building,
                    method: String::new(),
                    workers: 0.0,
                    output: 0.0,
                    profit: 0.0,
                    utilization: 0.0,
                    staffing: 0.0,
                    inputs: 0.0,
                },
            ),
            _ => return,
        };
        let k = kinds[usize::from(kind)].get_or_insert_with(|| KindRow {
            kind,
            key: b.key.clone(),
            group: b.group_ix,
            public: b.public,
            levels: 0,
            building: 0,
            workers: 0.0,
            output: 0.0,
            profit: 0.0,
            utilization: 0.0,
            staffing: 0.0,
            states: Vec::new(),
        });
        k.levels += levels;
        k.building += row.building;
        k.workers += row.workers;
        k.output += row.output;
        k.profit += row.profit;
        k.utilization += row.utilization * levels as f64;
        k.staffing += row.staffing * levels as f64;
        k.states.push(row);
    };
    for (ri, r) in world.econ.regions.iter().enumerate() {
        if usize::from(r.market) != ci {
            continue;
        }
        let ri = ri as u32;
        for p in r.plants.iter().filter(|p| p.levels > 0) {
            add(ri, r, p.kind, Some(p));
        }
        // Works for a building the state has none of yet.
        for (&(qr, kind), _) in queued.range((ri, 0)..=(ri, u16::MAX)) {
            debug_assert_eq!(qr, ri);
            if !r.plants.iter().any(|p| p.kind == kind && p.levels > 0) {
                add(ri, r, kind, None);
            }
        }
    }
    let mut kinds: Vec<KindRow> = kinds.into_iter().flatten().collect();
    for k in &mut kinds {
        if k.levels > 0 {
            k.utilization /= k.levels as f64;
            k.staffing /= k.levels as f64;
        }
        k.states
            .sort_by(|a, b| b.levels.cmp(&a.levels).then(b.building.cmp(&a.building)));
    }
    Some(BuildingsView {
        month: (date.year, date.month),
        queue: m.queue.len(),
        kinds,
    })
}
