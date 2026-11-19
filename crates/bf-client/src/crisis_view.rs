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
    /// Who would join if the side that demanded declared, by the agreements in force (W4);
    /// `None` with no agreement in force at all (nothing to read).
    pub joiners: Option<Vec<Joiner>>,
    /// The nuclear balance with the other side, as the player reads it; `None` when neither
    /// holds a warhead.
    pub nuclear: Option<crate::nuclear_view::Panel>,
}

/// A third party an agreement in force would call, or bar, at the declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct Joiner {
    pub party: String,
    pub kind: bf_sim::treaty::DutyKind,
    /// The agreement's key (`treaty-<key>`) and year of signature.
    pub agreement: String,
    pub signed: i32,
    /// The country of the crisis it is tied to by that clause.
    pub partner: String,
    pub suspended: bool,
    pub barred: bool,
}

/// A call on the player to honour an agreement in a war (W4).
#[derive(Clone, Debug, PartialEq)]
pub struct CallLine {
    pub war: u32,
    pub side: bf_sim::war::third::CallSide,
    /// Who the player would fight beside, and against.
    pub ally: String,
    pub enemy: String,
    pub agreement: String,
    pub signed: i32,
    pub weeks_left: u16,
    pub call_weeks: u16,
    /// What not honouring costs: a year of the forces' cost, real dollars.
    pub price: f64,
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
    /// The country the crises were read for (the one played now).
    pub me: String,
    pub crises: Vec<CrisisLine>,
    pub wars: Vec<WarLine>,
    /// Calls on the player to join a war by an agreement.
    pub calls: Vec<CallLine>,
    /// The weeks a demand gives by the rules (W2).
    pub deadline_weeks: u16,
    /// What the conflicts looked like when read: a change reads them again.
    pub signature: Vec<(u32, u8, u16)>,
}

/// What the conflicts look like, cheaply: each crisis's id, stage and weeks, each war, each call.
pub fn signature(world: &GameWorld) -> Vec<(u32, u8, u16)> {
    let c = &world.war.conflicts;
    c.crises
        .iter()
        .map(|k| (k.id, k.stage as u8, k.weeks_left))
        .chain(c.wars.iter().map(|w| (w.id, 9, 0)))
        .chain(c.calls.iter().map(|k| (k.war, 10, k.weeks_left)))
        .collect()
}

fn line(
    world: &GameWorld,
    me_tag: &str,
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
        joiners: joiners(world, &tag(k.by), &tag(k.target)),
        demand: k.demand.clone(),
        nuclear: {
            let me = world
                .countries
                .iter()
                .position(|(_, c)| c.tag == me_tag)
                .map(|i| i as u16);
            me.and_then(|me| {
                let other = if me == k.by { k.target } else { k.by };
                crate::nuclear_view::read(world, mil, me, other)
            })
        },
    }
}

/// The agreement of id `id`: its key and year of signature.
fn agreement(world: &GameWorld, id: u32) -> (String, i32) {
    world
        .treaties
        .agreements
        .iter()
        .find(|a| a.id == id)
        .map_or_else(|| (String::new(), 0), |a| (a.key.clone(), a.signed))
}

/// Who would join if `by` declared on `target`; `None` with no agreement in force at all.
fn joiners(world: &GameWorld, by: &str, target: &str) -> Option<Vec<Joiner>> {
    use bf_sim::treaty::DutyKind;
    if world.treaties.agreements.is_empty() {
        return None;
    }
    Some(
        bf_sim::war::third::who_would_join(world, by, target)
            .into_iter()
            .map(|e| {
                let (key, signed) = agreement(world, e.agreement);
                let partner = match e.kind {
                    DutyKind::JoinAttacker => by,
                    _ => target,
                };
                Joiner {
                    partner: partner.to_owned(),
                    party: e.party,
                    kind: e.kind,
                    agreement: key,
                    signed,
                    suspended: e.suspended,
                    barred: e.barred,
                }
            })
            .collect(),
    )
}

/// The player's crises and wars; `None` without a player or with the crisis switch off.
pub fn read(
    world: &GameWorld,
    tag: &str,
    setup: Option<&CrisisSetup>,
    mil: Option<&Military>,
) -> Option<CrisisView> {
    let (setup, mil) = (setup?, mil?);
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
            Some(line(world, tag, setup, mil, k, role))
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
    let first = |v: &[u16]| tags(&v[..v.len().min(1)]).pop().unwrap_or_default();
    let calls = c
        .calls
        .iter()
        .filter(|k| k.party == me)
        .filter_map(|k| {
            let w = c.wars.iter().find(|w| w.id == k.war)?;
            let (beside, against) = match k.side {
                bf_sim::war::third::CallSide::Defender => (&w.side_b, &w.side_a),
                bf_sim::war::third::CallSide::Attacker => (&w.side_a, &w.side_b),
            };
            let (key, signed) = agreement(world, k.agreement);
            Some(CallLine {
                war: k.war,
                side: k.side,
                ally: first(beside),
                enemy: first(against),
                agreement: key,
                signed,
                weeks_left: k.weeks_left,
                call_weeks: setup.rules.call_weeks,
                price: bf_sim::war::third::forces_cost_year(world, me).to_f64_lossy(),
            })
        })
        .collect();
    Some(CrisisView {
        me: tag.to_owned(),
        crises,
        wars,
        calls,
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
    tag: &str,
    setup: Option<&CrisisSetup>,
    mil: Option<&Military>,
    target: &str,
    demand: &Proposal,
) -> Option<DraftWeighing> {
    let (setup, mil) = (setup?, mil?);
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
        line: line(world, tag, setup, mil, &k, Role::Demander),
        blocked,
        target_gdp: world
            .econ
            .markets
            .get(usize::from(to))
            .map_or(0.0, |m| m.gdp_last_year.to_f64_lossy()),
    })
}
