//! The works' queue of the player's country (D8, `docs/state.pt-BR.md`), for the government's
//! works tab: each public work in the order the points go to it, with its progress, the weeks
//! left at last week's pace and the money still to pay (`bf_sim::econ::levers::works`); the
//! private works of the same queue, read only (owner's decision O2: the government has no hand
//! on them, but they take points from the same builders); what holds the whole queue back; and
//! what the state could put in the queue, building by building and region by region, with its
//! cost and the levels the region has already (`levers::buildable`). Read once a month and when
//! the queue or its order changes.

use bf_sim::econ::Catalogue;
use bf_sim::econ::construction::{MIN_WEEKS, want};
use bf_sim::econ::levers::{Bottleneck, WorkKey};
use bf_sim::game::GameWorld;
use bf_sim::research::TechTree;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// One public work of the queue.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkRow {
    pub key: WorkKey,
    /// Building key (`building-<key>` in the Fluent files).
    pub building: String,
    /// Map state id of the region (1-based, `World::states[id - 1]`).
    pub state: u16,
    /// Public category (`econ::public::INFRASTRUCTURE`…), when the state builds this building.
    pub category: Option<usize>,
    pub progress: f64,
    pub weeks: i64,
    /// Weeks left once the bottleneck clears, at the full weekly share.
    pub weeks_full: i64,
    /// What the points still missing cost at today's price of construction.
    pub cost_left: f64,
    /// Placed by the government's order rather than by arrival.
    pub fixed: bool,
    /// The region is held by an enemy (switch `occupied_works`): the work waits, taking no
    /// points, until it comes back.
    pub stopped: bool,
}

/// A private work of the queue: an investor's, shown and not touched.
#[derive(Clone, Debug, PartialEq)]
pub struct PrivateRow {
    pub building: String,
    pub state: u16,
    pub progress: f64,
    pub weeks: i64,
    /// As [`WorkRow::stopped`].
    pub stopped: bool,
}

/// A building the state may ask for in one region.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildOption {
    pub region: u32,
    pub state: u16,
    pub cost: f64,
    /// Levels of the building the region has already.
    pub levels: i64,
    /// Levels of it already in the queue there.
    pub queued: i64,
}

/// A building the state may ask for, with the regions that may take it, cheapest first.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildKind {
    pub kind: u16,
    pub building: String,
    pub category: Option<usize>,
    pub regions: Vec<BuildOption>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorksView {
    /// Month read in.
    pub month: (i32, u8),
    pub rows: Vec<WorkRow>,
    pub private: Vec<PrivateRow>,
    /// What holds the whole queue back this week, how tight (0–1, `WorkLine::met`), and with
    /// `Cash` the money the treasury lacks to pay in full again.
    pub bottleneck: Bottleneck,
    pub met: f64,
    pub short: Option<f64>,
    /// Share of the points the queue asked for that it got last week, 0–1.
    pub pace: f64,
    /// Weeks a work asked for now would take at last week's pace.
    pub new_weeks: i64,
    /// Buildings the state could ask for, by name of the building key.
    pub buildable: Vec<BuildKind>,
    /// The government fixed the order (D8).
    pub ordered: bool,
    /// What the view was read from: queue length and order length.
    pub signature: (usize, usize, usize),
}

impl WorksView {
    /// Money the public works still need, at today's price of construction.
    pub fn cost_left(&self) -> f64 {
        self.rows.iter().map(|r| r.cost_left).sum()
    }
}

/// The queue and order lengths: a change reads the view again within the month.
pub fn signature(world: &GameWorld, ci: usize) -> (usize, usize, usize) {
    (
        world.econ.markets.get(ci).map_or(0, |m| m.queue.len()),
        world.levers.order(ci).map_or(0, <[WorkKey]>::len),
        world.war.fighting.control.len(),
    )
}

/// Weeks to finish `left` of `points` at `pace` of the full weekly share.
fn weeks_at(points: Fixed, left: Fixed, pace: f64) -> i64 {
    let full = points.to_f64_lossy() / MIN_WEEKS as f64;
    if full <= 0.0 {
        return 0;
    }
    (left.to_f64_lossy() / (full * pace.max(0.01)))
        .ceil()
        .max(1.0) as i64
}

pub fn read(
    world: &GameWorld,
    tag: &str,
    cat: &Catalogue,
    tree: Option<&TechTree>,
    date: Date,
    occupied_works: bool,
) -> Option<WorksView> {
    let stopped = |region: u32| occupied_works && !bf_sim::econ::construction::held(world, region);
    let ci = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let m = world.econ.markets.get(ci)?;
    let order = world.levers.order(ci);
    let building = |kind: u16| {
        cat.buildings
            .get(usize::from(kind))
            .map_or_else(String::new, |b| b.key.clone())
    };
    let state_of = |region: u32| {
        world
            .econ
            .regions
            .get(region as usize)
            .map_or(0, |r| r.state)
    };
    let construction = cat.construction.map(usize::from);
    let price = construction
        .and_then(|g| m.price.get(g))
        .map_or(0.0, |p| p.to_f64_lossy());
    let lines = bf_sim::econ::levers::works(cat, m, order);
    let first = lines.first().copied();
    let pace = first.map_or_else(
        || {
            construction
                .and_then(|g| m.buy_ratio.get(g))
                .map_or(1.0, |r| r.to_f64_lossy())
        },
        |w| w.pace,
    );
    let left_of = |key: &WorkKey| {
        m.queue
            .iter()
            .find(|p| p.public && WorkKey::of(p) == *key)
            .map_or(Fixed::ZERO, |p| p.left)
    };
    let rows = lines
        .into_iter()
        .map(|w| WorkRow {
            key: w.key,
            building: building(w.key.kind),
            state: state_of(w.key.region),
            category: w.category,
            progress: w.progress,
            weeks: w.weeks,
            weeks_full: w.weeks_full,
            cost_left: left_of(&w.key).to_f64_lossy() * price,
            fixed: w.fixed,
            stopped: stopped(w.key.region),
        })
        .collect();
    let private = m
        .queue
        .iter()
        .filter(|p| !p.public)
        .map(|p| PrivateRow {
            building: building(p.kind),
            state: state_of(p.region),
            progress: if p.points > Fixed::ZERO {
                1.0 - p.left.to_f64_lossy() / p.points.to_f64_lossy()
            } else {
                1.0
            },
            weeks: if want(p) > Fixed::ZERO {
                weeks_at(p.points, p.left, pace)
            } else {
                0
            },
            stopped: stopped(p.region),
        })
        .collect();
    let queued = |region: u32, kind: u16| {
        m.queue
            .iter()
            .filter(|p| p.region == region && p.kind == kind)
            .count() as i64
    };
    let levels = |region: u32, kind: u16| {
        world
            .econ
            .regions
            .get(region as usize)
            .and_then(|r| r.plants.iter().find(|p| p.kind == kind))
            .map_or(0, |p| p.levels)
    };
    let kinds_public = bf_sim::econ::public::PublicKinds::new(cat);
    let mut buildable: Vec<BuildKind> = Vec::new();
    for (region, kinds) in bf_sim::econ::levers::buildable(cat, world, tree, date.year, ci) {
        for (kind, cost) in kinds {
            let option = BuildOption {
                region,
                state: state_of(region),
                cost: cost.to_f64_lossy(),
                levels: levels(region, kind),
                queued: queued(region, kind),
            };
            match buildable.iter_mut().find(|b| b.kind == kind) {
                Some(b) => b.regions.push(option),
                None => buildable.push(BuildKind {
                    kind,
                    building: building(kind),
                    category: kinds_public.category_of(usize::from(kind)),
                    regions: vec![option],
                }),
            }
        }
    }
    for b in &mut buildable {
        b.regions.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.region.cmp(&b.region))
        });
    }
    buildable.sort_by_key(|b| (b.category.unwrap_or(usize::MAX), b.kind));
    Some(WorksView {
        month: (date.year, date.month),
        rows,
        private,
        bottleneck: first.map_or(Bottleneck::None, |w| w.bottleneck),
        met: first.map_or(1.0, |w| w.met),
        short: first.and_then(|w| w.short),
        pace,
        new_weeks: (MIN_WEEKS as f64 / pace.max(0.01)).ceil() as i64,
        buildable,
        ordered: order.is_some(),
        signature: signature(world, ci),
    })
}
