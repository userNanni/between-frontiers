//! The public works' queue of the player's country (D8, `docs/state.pt-BR.md`), for the
//! government's works tab: each work in the order the points go to it, with its progress, the
//! weeks left at last week's pace and what holds it back (`bf_sim::econ::levers::works`), and
//! what the state could put in the queue, region by region, with its cost
//! (`levers::buildable`). Only public works: the government has no hand on the private ones
//! (owner's decision O2). Read once a month and when the queue or its order changes.

use bf_sim::econ::Catalogue;
use bf_sim::econ::levers::{Bottleneck, WorkKey};
use bf_sim::game::GameWorld;
use bf_sim::research::TechTree;
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
    pub bottleneck: Bottleneck,
    /// Placed by the government's order rather than by arrival.
    pub fixed: bool,
}

/// What the state could put in the queue in one region.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildRegion {
    pub region: u32,
    pub state: u16,
    /// Building index, key, and cost at this week's construction price, real dollars.
    pub kinds: Vec<(u16, String, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorksView {
    pub month: (i32, u8),
    pub rows: Vec<WorkRow>,
    pub buildable: Vec<BuildRegion>,
    /// The government fixed an order (`SetWorksOrder` with `Some`).
    pub ordered: bool,
    /// What the queue looked like when read: its length and the order fixed.
    pub signature: (usize, usize),
}

/// The queue's length and the length of the fixed order: a change reads the works again.
pub fn signature(world: &GameWorld, ci: usize) -> (usize, usize) {
    (
        world.econ.markets.get(ci).map_or(0, |m| m.queue.len()),
        world.levers.order(ci).map_or(0, <[WorkKey]>::len),
    )
}

/// The player's public works, or `None` without a player.
pub fn read(
    world: &GameWorld,
    cat: &Catalogue,
    tree: Option<&TechTree>,
    date: Date,
) -> Option<WorksView> {
    let tag = world.player_tag()?;
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
    let rows = bf_sim::econ::levers::works(cat, m, order)
        .into_iter()
        .map(|w| WorkRow {
            key: w.key,
            building: building(w.key.kind),
            state: state_of(w.key.region),
            category: w.category,
            progress: w.progress,
            weeks: w.weeks,
            bottleneck: w.bottleneck,
            fixed: w.fixed,
        })
        .collect();
    let buildable = bf_sim::econ::levers::buildable(cat, world, tree, date.year, ci)
        .into_iter()
        .filter(|(_, kinds)| !kinds.is_empty())
        .map(|(region, kinds)| BuildRegion {
            region,
            state: state_of(region),
            kinds: kinds
                .into_iter()
                .map(|(k, cost)| (k, building(k), cost.to_f64_lossy()))
                .collect(),
        })
        .collect();
    Some(WorksView {
        month: (date.year, date.month),
        rows,
        buildable,
        ordered: order.is_some(),
        signature: signature(world, ci),
    })
}
