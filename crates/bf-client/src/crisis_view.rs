//! Crises and wars of the player's country (phase 5, slice B), for the journal: the demand with
//! the share of the country it takes, the deadline, the answer the target is expected to give
//! (`bf_sim::war::weigh_crisis`, term by term) and the wars declared. Read with the snapshot when
//! the conflicts change, and the weighing of a demand being drafted when the draft changes.

use bf_sim::game::GameWorld;
use bf_sim::treaty::Proposal;
use bf_sim::war::crisis::TRIBUTE_YEARS;
use bf_sim::war::{Crisis, CrisisSetup, Military, Stage, Weighing};

/// The player's part in a crisis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The player demanded: after a refusal, declares or backs down.
    Demander,
    /// The demand is on the player: accepts or refuses.
    Target,
}

/// One crisis as the journal shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct CrisisLine {
    pub id: u32,
    pub by: String,
    pub target: String,
    pub role: Role,
    pub stage: Stage,
    pub weeks_left: u16,
    pub deadline_weeks: u16,
    /// False: a demand without a ground, kept on the record (W1).
    pub casus_belli: bool,
    pub demand: Proposal,
    /// The states it takes (map keys) and whether it absorbs the target.
    pub states: Vec<String>,
    pub union: bool,
    /// Money it takes over the horizon the weighing uses, real dollars.
    pub money: f64,
    pub weighing: Weighing,
    /// Men under arms on land of each side.
    pub land_by: i64,
    pub land_target: i64,
}

/// A war the player is in.
#[derive(Clone, Debug, PartialEq)]
pub struct WarLine {
    pub id: u32,
    pub side_a: Vec<String>,
    pub side_b: Vec<String>,
    pub since: (i32, u8, u8),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrisisView {
    pub crises: Vec<CrisisLine>,
    pub wars: Vec<WarLine>,
    /// The weeks a demand gives by the rules (W2).
    pub deadline_weeks: u16,
    /// What the conflicts looked like when read: a change reads them again.
    pub signature: Vec<(u32, u8, u16)>,
}

/// What the conflicts look like, cheaply: each crisis's id, stage and weeks, then each war.
pub fn signature(world: &GameWorld) -> Vec<(u32, u8, u16)> {
    let c = &world.war.conflicts;
    c.crises
        .iter()
        .map(|k| (k.id, k.stage as u8, k.weeks_left))
        .chain(c.wars.iter().map(|w| (w.id, 9, 0)))
        .collect()
}

fn line(
    world: &GameWorld,
    setup: &CrisisSetup,
    mil: &Military,
    k: &Crisis,
    role: Role,
) -> CrisisLine {
    let tag = |i: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(i))
            .map_or_else(String::new, |(_, c)| c.tag.clone())
    };
    let (states, union) = bf_sim::war::crisis::states_asked(&k.demand);
    CrisisLine {
        id: k.id,
        by: tag(k.by),
        target: tag(k.target),
        role,
        stage: k.stage,
        weeks_left: k.weeks_left,
        deadline_weeks: setup.rules.deadline_weeks,
        casus_belli: k.casus_belli,
        states: states.iter().map(|s| (*s).to_owned()).collect(),
        union,
        money: bf_sim::war::crisis::money_asked(&k.demand, TRIBUTE_YEARS).to_f64_lossy(),
        weighing: bf_sim::war::weigh_crisis(setup, mil, world, k),
        land_by: bf_sim::war::crisis::land_men(mil, &world.war, usize::from(k.by)),
        land_target: bf_sim::war::crisis::land_men(mil, &world.war, usize::from(k.target)),
        demand: k.demand.clone(),
    }
}

/// The player's crises and wars; `None` without a player or with the crisis switch off.
pub fn read(
    world: &GameWorld,
    setup: Option<&CrisisSetup>,
    mil: Option<&Military>,
) -> Option<CrisisView> {
    let (setup, mil) = (setup?, mil?);
    let tag = world.player_tag()?;
    let me = world.countries.iter().position(|(_, c)| c.tag == tag)? as u16;
    let c = &world.war.conflicts;
    let crises = c
        .crises
        .iter()
        .filter_map(|k| {
            let role = if k.by == me {
                Role::Demander
            } else if k.target == me {
                Role::Target
            } else {
                return None;
            };
            Some(line(world, setup, mil, k, role))
        })
        .collect();
    let tags = |v: &[u16]| -> Vec<String> {
        v.iter()
            .filter_map(|i| {
                world
                    .countries
                    .iter()
                    .nth(usize::from(*i))
                    .map(|(_, c)| c.tag.clone())
            })
            .collect()
    };
    let wars = c
        .wars
        .iter()
        .filter(|w| w.side_a.contains(&me) || w.side_b.contains(&me))
        .map(|w| WarLine {
            id: w.id,
            side_a: tags(&w.side_a),
            side_b: tags(&w.side_b),
            since: w.since,
        })
        .collect();
    Some(CrisisView {
        crises,
        wars,
        deadline_weeks: setup.rules.deadline_weeks,
        signature: signature(world),
    })
}

/// A demand being drafted, weighed as the target would weigh it.
#[derive(Clone, Debug, PartialEq)]
pub struct DraftWeighing {
    pub target: String,
    pub demand: Proposal,
    pub line: CrisisLine,
    /// Why it cannot be made now: at war with the target, or a crisis already open between them.
    pub blocked: Option<&'static str>,
    /// The target's GDP of the last year, real dollars: the base of a payment asked as a share.
    pub target_gdp: f64,
}

/// Weighs the player's draft `demand` on `target` without touching the world.
pub fn weigh_draft(
    world: &GameWorld,
    setup: Option<&CrisisSetup>,
    mil: Option<&Military>,
    target: &str,
    demand: &Proposal,
) -> Option<DraftWeighing> {
    let (setup, mil) = (setup?, mil?);
    let tag = world.player_tag()?;
    let index = |t: &str| world.countries.iter().position(|(_, c)| c.tag == t);
    let (by, to) = (index(tag)? as u16, index(target)? as u16);
    let c = &world.war.conflicts;
    let blocked = if c.at_war(by, to) {
        Some("crisis-blocked-war")
    } else if c
        .crises
        .iter()
        .any(|k| (k.by == by && k.target == to) || (k.by == to && k.target == by))
    {
        Some("crisis-blocked-open")
    } else {
        None
    };
    let k = Crisis {
        id: 0,
        by,
        target: to,
        demand: demand.clone(),
        weeks_left: setup.rules.deadline_weeks,
        stage: Stage::Waiting,
        casus_belli: false,
    };
    Some(DraftWeighing {
        target: target.to_owned(),
        demand: demand.clone(),
        line: line(world, setup, mil, &k, Role::Demander),
        blocked,
        target_gdp: world
            .econ
            .markets
            .get(usize::from(to))
            .map_or(0.0, |m| m.gdp_last_year.to_f64_lossy()),
    })
}
