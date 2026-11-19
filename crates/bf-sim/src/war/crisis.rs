//! Crisis and declaration (phase 5, slice B, draft): a war starts from a demand refused, never
//! from a bare declaration. The demand is a treaty proposal; accepted, it is carried out as a
//! treaty (`treaty_exec`); refused or left to lapse, the side that demanded declares war or backs
//! down. The AI only answers: as the target it weighs the force against the share of the country
//! asked for (owner's decision W3), and as the side that demanded it backs down. Design in
//! `docs/war.pt-BR.md`, "Crise e declaração".

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use super::catalogue::{Domain, Military};
use super::state::{WarState, men};
use crate::game::GameWorld;
use crate::treaty::{ClauseCatalogue, ClauseKind, Proposal, Side};

/// `weeks_left` of a demand that named no deadline: the forces system sets the rules' one.
pub const RULES_DEADLINE: u16 = u16::MAX;

/// Years of tribute a demand is weighed over.
pub const TRIBUTE_YEARS: i64 = 10;

/// What the forces system needs to run crises.
#[derive(Clone, Debug)]
pub struct CrisisSetup {
    pub rules: CrisisRules,
    /// Carries out a demand accepted; without it an accepted demand only closes the crisis.
    pub clauses: Option<ClauseCatalogue>,
    /// Map state key → id (`Region::state`).
    pub state_ids: BTreeMap<String, u16>,
}

/// How crises run (`data/military/crisis.ron`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrisisRules {
    /// Weeks the target has to answer when the demand names none (W2: 4).
    pub deadline_weeks: u16,
    /// Force over the target's, per mille, at which a peripheral demand is yielded (W3: 3,000).
    pub yield_ratio: u32,
    /// Share of the country, per mille, from which a demand is existential and never yielded.
    pub existential_share: u32,
    /// Weeks a third party called to war by an agreement has to answer (W4).
    #[serde(default = "default_call_weeks")]
    pub call_weeks: u16,
    /// A call left to lapse is honoured (owner's decision P2) or betrayed.
    #[serde(default = "default_true")]
    pub lapsed_call_honours: bool,
    /// N4: against a demander that can coerce (a secure second strike the target lacks), the
    /// ratio it takes to yield a peripheral demand is multiplied by this, per mille. The
    /// existential never yields.
    #[serde(default = "default_coercion")]
    pub nuclear_coercion: u32,
}

fn default_coercion() -> u32 {
    1000
}

fn default_call_weeks() -> u16 {
    2
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// The deadline runs.
    Waiting,
    /// Refused, or the deadline lapsed: the side that demanded declares or backs down.
    Refused,
    /// Accepted: carried out as a treaty in the next week, then closed.
    Accepted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Crisis {
    pub id: u32,
    /// Market index of the side that demands, and of the target.
    pub by: u16,
    pub target: u16,
    /// The demand: `side_a` the one who demands, `side_b` the target.
    pub demand: Proposal,
    pub weeks_left: u16,
    pub stage: Stage,
    /// Whether the side that demands holds a cause against the target. A demand without one is
    /// allowed and recorded (W1). No causes exist before the treaties' breaches enter the tick.
    pub casus_belli: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct War {
    pub id: u32,
    /// Market indices: the side that declared (its leader first) and the side declared on.
    pub side_a: Vec<u16>,
    pub side_b: Vec<u16>,
    /// The crisis it came from, and its demand, still at stake.
    pub crisis: u32,
    pub demand: Proposal,
    /// (year, month, day) of the declaration.
    pub since: (i32, u8, u8),
}

/// Crises open and wars under way (in `WarState`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflicts {
    pub crises: Vec<Crisis>,
    pub wars: Vec<War>,
    /// Id of the next crisis or war.
    pub next_id: u32,
    /// Calls to war the player has to answer (W4, `third`).
    pub calls: Vec<super::third::Call>,
}

impl Conflicts {
    pub fn is_empty(&self) -> bool {
        self.crises.is_empty() && self.wars.is_empty() && self.next_id == 0 && self.calls.is_empty()
    }

    pub fn at_war(&self, a: u16, b: u16) -> bool {
        self.wars.iter().any(|w| {
            (w.side_a.contains(&a) && w.side_b.contains(&b))
                || (w.side_a.contains(&b) && w.side_b.contains(&a))
        })
    }
}

/// What the target weighs (W3), term by term, for the crisis screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weighing {
    /// Land force of the side that demands over the target's, per mille.
    pub ratio: i64,
    /// Share of the target asked for, per mille: its people in the states it would give, or the
    /// money over its GDP of last year, whichever is larger.
    pub share: i64,
    /// The ratio it takes to yield at this share; `None` when the demand is existential.
    pub needed: Option<i64>,
    pub yields: bool,
    /// N4: the demander can coerce (a secure second strike the target lacks): the ratio needed
    /// fell by `nuclear_coercion`.
    pub coerced: bool,
}

/// The target yields when the demand is not existential and the force against it reaches the
/// ratio, which rises with the share asked for: `yield_ratio / (1 − share / existential)`.
pub fn weigh(rules: &CrisisRules, ratio: i64, share: i64, union: bool) -> Weighing {
    let existential = i64::from(rules.existential_share.max(1));
    if union || share >= existential {
        return Weighing {
            ratio,
            share,
            needed: None,
            yields: false,
            coerced: false,
        };
    }
    let needed = i64::from(rules.yield_ratio) * existential / (existential - share);
    Weighing {
        ratio,
        share,
        needed: Some(needed),
        yields: ratio >= needed,
        coerced: false,
    }
}

/// Whether the clause has the target (the junior side) give it.
fn target_gives(side: &Side) -> bool {
    matches!(side, Side::Junior | Side::All)
}

/// The states (map keys) a demand takes from the target, and whether it absorbs the target.
pub fn states_asked(demand: &Proposal) -> (Vec<&str>, bool) {
    let mut states = Vec::new();
    let mut union = false;
    for c in &demand.clauses {
        if !target_gives(&c.giver) {
            continue;
        }
        match &c.kind {
            ClauseKind::CedeStates { states: s }
            | ClauseKind::ReleaseCountry { states: s, .. }
            | ClauseKind::Occupation { states: s } => {
                states.extend(s.iter().map(String::as_str));
            }
            ClauseKind::Union => union = true,
            _ => {}
        }
    }
    (states, union)
}

/// Money a demand takes from the target over the horizon: payments, and tribute for `years`.
pub fn money_asked(demand: &Proposal, years: i64) -> Fixed {
    demand
        .clauses
        .iter()
        .filter(|c| target_gives(&c.giver))
        .map(|c| match &c.kind {
            ClauseKind::Payment { amount } => *amount,
            ClauseKind::Tribute { yearly } => *yearly * years,
            _ => Fixed::ZERO,
        })
        .sum()
}

/// Men under arms on land of a country.
pub fn land_men(mil: &Military, war: &WarState, country: usize) -> i64 {
    war.countries.get(country).map_or(0, |c| {
        c.formations
            .iter()
            .filter(|f| mil.types[f.unit as usize].domain == Domain::Land)
            .map(|f| men(mil, f))
            .sum()
    })
}

/// How the target of `k` weighs it: the land force against it and the share it is asked for.
pub fn weigh_crisis(
    setup: &CrisisSetup,
    mil: &Military,
    world: &GameWorld,
    k: &Crisis,
) -> Weighing {
    let theirs = land_men(mil, &world.war, k.by as usize);
    let mine = land_men(mil, &world.war, k.target as usize).max(1);
    let ratio = theirs * 1000 / mine;
    let (states, union) = states_asked(&k.demand);
    let ids: Vec<u16> = states
        .iter()
        .filter_map(|s| setup.state_ids.get(*s).copied())
        .collect();
    let (mut asked, mut all) = (0i64, 0i64);
    for r in world.econ.regions.iter().filter(|r| r.market == k.target) {
        all += r.population();
        if ids.contains(&r.state) {
            asked += r.population();
        }
    }
    let people = if all > 0 { asked * 1000 / all } else { 0 };
    let gdp = world
        .econ
        .markets
        .get(k.target as usize)
        .map_or(Fixed::ZERO, |m| m.gdp_last_year);
    let money = money_asked(&k.demand, TRIBUTE_YEARS);
    let money = if gdp.is_positive() {
        (money.mul_ratio(1000, 1) / gdp).to_int_floor()
    } else {
        0
    };
    let mut w = weigh(&setup.rules, ratio, people.max(money), union);
    // N4: the target reads the demander's arsenal (opaque if never tested); one-sided, it
    // coerces on the periphery.
    if let Some(needed) = w.needed
        && let Some(b) = super::balance::balance(mil, world, k.by, k.target, Some(k.target))
        && b.deterrence.first_strike_incentive[0]
    {
        let n = needed * i64::from(setup.rules.nuclear_coercion) / 1000;
        w.needed = Some(n);
        w.yields = w.ratio >= n;
        w.coerced = true;
    }
    w
}

/// What a week of crises does to the world, worked out from it in the forces system's compute.
#[derive(Debug, Default)]
pub struct CrisisWeek {
    pub conflicts: Conflicts,
    /// Demands accepted, to carry out as treaties.
    pub settled: Vec<crate::treaty_exec::Settlement>,
    /// Calls whose deadline ran out, honoured (`true`) or betrayed by the rule `lapsed_call_honours`.
    pub lapsed: Vec<(super::third::Call, bool)>,
}

/// A week of crises: deadlines run, the AI answers as the target (W3) and backs down when it
/// demanded and heard a refusal; accepted demands are settled and close.
pub fn crisis_week(setup: &CrisisSetup, mil: &Military, world: &GameWorld) -> CrisisWeek {
    let player = world
        .countries
        .iter()
        .position(|(id, _)| Some(id) == world.player)
        .map(|i| i as u16);
    let mut out = CrisisWeek {
        conflicts: world.war.conflicts.clone(),
        settled: Vec::new(),
        lapsed: Vec::new(),
    };
    // Calls to war: the deadline runs; lapsed, they are honoured or betrayed by the rules.
    let mut calls = Vec::new();
    for mut c in std::mem::take(&mut out.conflicts.calls) {
        if c.weeks_left == RULES_DEADLINE {
            c.weeks_left = setup.rules.call_weeks;
        }
        if c.weeks_left == 0 {
            out.lapsed.push((c, setup.rules.lapsed_call_honours));
        } else {
            c.weeks_left -= 1;
            calls.push(c);
        }
    }
    out.conflicts.calls = calls;
    let mut keep = Vec::new();
    for mut k in std::mem::take(&mut out.conflicts.crises) {
        if k.weeks_left == RULES_DEADLINE {
            k.weeks_left = setup.rules.deadline_weeks;
        }
        if k.stage == Stage::Waiting {
            if Some(k.target) != player {
                k.stage = if weigh_crisis(setup, mil, world, &k).yields {
                    Stage::Accepted
                } else {
                    Stage::Refused
                };
            } else if k.weeks_left == 0 {
                // Left to lapse: a refusal.
                k.stage = Stage::Refused;
            } else {
                k.weeks_left -= 1;
            }
        }
        match k.stage {
            Stage::Accepted => {
                if let Some(cat) = &setup.clauses
                    && let Ok(s) =
                        crate::treaty_exec::settle(cat, &k.demand, &format!("crisis-{}", k.id))
                {
                    out.settled.push(s);
                }
            }
            // The AI that demanded and heard a refusal backs down.
            Stage::Refused if Some(k.by) != player => {}
            _ => keep.push(k),
        }
    }
    out.conflicts.crises = keep;
    out
}

/// The side that demanded crisis `id` and heard a refusal declares war or backs down. Declared,
/// both sides call up their reserve; the demand stays at stake in the war; the agreements in
/// force call in third parties (W4, `third::on_declaration`).
pub fn crisis_decide(
    world: &mut GameWorld,
    id: u32,
    by: Option<usize>,
    declare: bool,
    date: laplace_schelling::Date,
) {
    let c = &mut world.war.conflicts;
    let Some(i) = c
        .crises
        .iter()
        .position(|k| k.id == id && Some(k.by as usize) == by && k.stage == Stage::Refused)
    else {
        return;
    };
    // The nuclear ceiling (war::balance): a declaration it blocks does not go; the crisis stays,
    // with its reason on the screen, until the demander backs down.
    if declare
        && world
            .war
            .nuclear
            .blocked(c.crises[i].by, c.crises[i].target)
    {
        return;
    }
    let k = c.crises.remove(i);
    if !declare {
        return;
    }
    open_war(
        world,
        vec![k.by],
        vec![k.target],
        k.id,
        k.demand,
        date,
        true,
    );
}

/// Opens a war: `side_a` declared on `side_b` over `demand` (from crisis `crisis`, `u32::MAX`
/// without one). Both sides call up their reserve; with `calls`, the agreements in force call in
/// third parties (W4). Returns the war's id.
pub fn open_war(
    world: &mut GameWorld,
    side_a: Vec<u16>,
    side_b: Vec<u16>,
    crisis: u32,
    demand: Proposal,
    date: laplace_schelling::Date,
    calls: bool,
) -> u32 {
    let c = &mut world.war.conflicts;
    let wid = c.next_id;
    c.next_id += 1;
    for s in side_a.iter().chain(&side_b) {
        if let Some(cw) = world.war.countries.get_mut(*s as usize) {
            cw.mobilized = true;
        }
    }
    world.war.conflicts.wars.push(War {
        id: wid,
        side_a,
        side_b,
        crisis,
        demand,
        since: (date.year, date.month, date.day),
    });
    if calls {
        super::third::on_declaration(world, wid, RULES_DEADLINE, date);
    }
    wid
}

impl HashState for Proposal {
    fn hash_state(&self, h: &mut StateHasher) {
        // Proposals are small and rare: hashed through their serialized form.
        let bytes = laplace_schelling::save::encode_snapshot(self).unwrap_or_default();
        h.write_len(bytes.len());
        for b in bytes {
            h.write_u8(b);
        }
    }
}

impl HashState for Conflicts {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.crises.len());
        for c in &self.crises {
            h.write_u32(c.id);
            h.write_u16(c.by);
            h.write_u16(c.target);
            c.demand.hash_state(h);
            h.write_u16(c.weeks_left);
            h.write_u8(c.stage as u8);
            c.casus_belli.hash_state(h);
        }
        h.write_len(self.wars.len());
        for w in &self.wars {
            h.write_u32(w.id);
            h.write_len(w.side_a.len());
            for s in &w.side_a {
                h.write_u16(*s);
            }
            h.write_len(w.side_b.len());
            for s in &w.side_b {
                h.write_u16(*s);
            }
            h.write_u32(w.crisis);
            w.demand.hash_state(h);
            h.write_i64(i64::from(w.since.0));
            h.write_u8(w.since.1);
            h.write_u8(w.since.2);
        }
        h.write_u32(self.next_id);
        // Only when there are any: a crisis run without third parties hashes as before them.
        if !self.calls.is_empty() {
            h.write_len(self.calls.len());
            for c in &self.calls {
                c.hash_state(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> CrisisRules {
        CrisisRules {
            deadline_weeks: 4,
            yield_ratio: 3000,
            existential_share: 500,
            call_weeks: 2,
            lapsed_call_honours: true,
            nuclear_coercion: 500,
        }
    }

    /// Crimea, 2014: a small, peripheral share before a much larger force is yielded.
    #[test]
    fn a_peripheral_demand_by_a_threefold_force_is_yielded() {
        let w = weigh(&rules(), 3500, 50, false);
        assert!(w.yields, "{w:?}");
        assert!(!weigh(&rules(), 2500, 50, false).yields);
    }

    /// Israel, 1948: an existential share is refused whatever the odds.
    #[test]
    fn an_annexation_is_refused_whatever_the_odds() {
        assert!(!weigh(&rules(), 100_000, 0, true).yields);
        assert!(!weigh(&rules(), 100_000, 600, false).yields);
        assert_eq!(weigh(&rules(), 100_000, 600, false).needed, None);
    }

    #[test]
    fn the_ratio_it_takes_rises_with_the_share() {
        let a = weigh(&rules(), 0, 100, false).needed.unwrap();
        let b = weigh(&rules(), 0, 300, false).needed.unwrap();
        assert_eq!(weigh(&rules(), 0, 0, false).needed, Some(3000));
        assert!(a > 3000 && b > a, "{a} {b}");
    }
}
