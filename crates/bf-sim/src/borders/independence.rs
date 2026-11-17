//! Part C: independence by movement (`docs/borders.pt-BR.md`, section 11; decisions I1–I5).
//!
//! The independence movement of the politics is measured in every colonial unit (a candidate of
//! the catalogue whose states are, by most of their people, colonies of one owner). When the
//! unit's mobilization passes the mark of the demand, the metropole answers: self-rule (the law
//! `trusteeship`), a dominion, independence by agreement, or a refusal. After a refusal, past the
//! mark of the revolt, the unit breaks away without an agreement. The metropole without a player
//! answers by a measured account: the value of the colony against the cost of holding it
//! (legitimacy, the international cost, the pressure of the anticolonial countries' influence).
//! A player metropole answers by [`crate::game::GameCommand::AnswerDemand`].
//!
//! Everything is integer fixed point and free of randomness. Runs every January 2nd, after the
//! politics has moved the movements of the country on the 1st.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};
use laplace_schelling::{Date, Fixed};
use serde::{Deserialize, Serialize};

use crate::borders::op::{BorderRules, Operation, Terms};
use crate::econ::catalogue::Catalogue;
use crate::econ::prof::P;
use crate::game::GameWorld;
use crate::politics::{Measure, Politics, Scope, margin};
use crate::polity::MovementState;
use crate::polity::measure::{self, Facts};
use crate::treaty::{Clause, ClauseCatalogue, ClauseKind, Proposal, Side};

/// The state of the independence movements by unit.
pub const INDEPENDENCE: ResourceId = ResourceId(18);

/// Rules and design values (`data/borders/independence.ron`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependenceRules {
    /// On only with `game::Switches::independence` (written by `GameData::rules_with`, never
    /// read from the data): a new rule ships off.
    #[serde(default)]
    pub enabled: bool,
    /// Mobilization (per mille) at which a unit demands an answer.
    pub demand: u16,
    /// Mobilization (per mille) at which a refused unit breaks away.
    pub revolt: u16,
    /// Months a player metropole has to answer.
    pub answer_months: u8,
    /// Per mille of the metropole's GDP a year that one per mille of its legitimacy is worth.
    pub legitimacy_price: u32,
    pub born_independent: u16,
    pub no_colonies: u16,
    /// Goodwill withdrawn a year, per mille of the anticolonial country's GDP, at full stance
    /// and full mobilization.
    pub reputation_rate: u32,
    /// Cap of that withdrawal a year, per mille of its GDP.
    pub reputation_cap: u32,
    /// Weight of the anticolonial countries' influence over the metropole, per mille.
    pub material_weight: u32,
}

/// What the metropole answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Answer {
    /// A player metropole has not answered yet.
    Pending,
    /// Self-rule: the law `trusteeship` in the colonies.
    SelfRule,
    /// The unit becomes a subject (bundle `dominion`) by agreement.
    Dominion,
    /// The unit becomes independent by agreement.
    Independence,
    Refused,
}

/// The account behind an answer, each term a share of the metropole's GDP a year.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// What the colony sends home (its value added times the colonial tribute).
    pub value: Fixed,
    /// Legitimacy lost to the unit's movement, at its price.
    pub legitimacy: Fixed,
    /// Goodwill the anticolonial countries withdraw, at their size.
    pub international: Fixed,
    /// Influence the anticolonial countries hold over the metropole (issue Money).
    pub material: Fixed,
    /// The countries paying most of the international cost (country, share of the metropole's
    /// GDP), largest first, at most five.
    pub top: Vec<(u16, Fixed)>,
}

impl Account {
    pub fn cost(&self) -> Fixed {
        self.legitimacy + self.international + self.material
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Demand {
    pub since: Date,
    pub answer: Answer,
    pub account: Account,
}

/// One colonial unit and its movement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    /// Tag of the catalogue's candidate.
    pub key: String,
    pub owner: u16,
    /// Region indices of the unit held as colonies by the owner.
    pub regions: Vec<u32>,
    pub movement: MovementState,
    /// Per mille of the unit's people mobilized.
    pub mobilized: u16,
    pub demand: Option<Demand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum How {
    Agreement,
    Dominion,
    Revolt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub date: Date,
    pub unit: String,
    pub owner: u16,
    pub how: How,
}

/// `GameWorld::independence`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependenceState {
    pub units: Vec<Unit>,
    pub history: Vec<Record>,
    /// Answers given by command and not yet acted on: (unit, answer).
    #[serde(default)]
    pub answers: Vec<(String, Answer)>,
}

impl IndependenceState {
    pub fn is_empty(&self) -> bool {
        self.units.is_empty() && self.history.is_empty() && self.answers.is_empty()
    }
}

fn hash_fixed(h: &mut laplace_schelling::hash::StateHasher, x: Fixed) {
    use laplace_schelling::hash::HashState;
    x.hash_state(h);
}

impl laplace_schelling::hash::HashState for IndependenceState {
    fn hash_state(&self, h: &mut laplace_schelling::hash::StateHasher) {
        h.write_len(self.units.len());
        for u in &self.units {
            h.write_str(&u.key);
            h.write_u16(u.owner);
            h.write_len(u.regions.len());
            for r in &u.regions {
                h.write_u32(*r);
            }
            u.movement.hash_state(h);
            h.write_u16(u.mobilized);
            match &u.demand {
                None => h.write_u8(0),
                Some(d) => {
                    h.write_u8(1);
                    d.since.hash_state(h);
                    h.write_u8(d.answer as u8);
                    for x in [
                        d.account.value,
                        d.account.legitimacy,
                        d.account.international,
                        d.account.material,
                    ] {
                        hash_fixed(h, x);
                    }
                }
            }
        }
        h.write_len(self.history.len());
        for r in &self.history {
            r.date.hash_state(h);
            h.write_str(&r.unit);
            h.write_u16(r.owner);
            h.write_u8(r.how as u8);
        }
        h.write_len(self.answers.len());
        for (u, a) in &self.answers {
            h.write_str(u);
            h.write_u8(*a as u8);
        }
    }
}

/// What the system reads besides the world.
#[derive(Clone)]
pub struct IndependenceSetup {
    pub rules: IndependenceRules,
    pub borders: Arc<BorderRules>,
    pub politics: Arc<Politics>,
    pub cat: Arc<Catalogue>,
    pub clauses: Option<Arc<ClauseCatalogue>>,
    pub influence: Option<Arc<crate::influence::InfluenceSetup>>,
    /// The law `trusteeship` (self-rule), resolved at load.
    pub trusteeship: Option<u16>,
    /// The buildings `school` and `hospital`, resolved at load.
    pub services: (Option<usize>, Option<usize>),
}

/// The colonial units of the world as it stands: every candidate without an end, with states,
/// whose present states are, by most of their people, colonies of one owner; a state in one unit
/// only (file order). Units keep their movement from `previous` while the owner is the same.
pub fn units_of(setup: &IndependenceSetup, world: &GameWorld, previous: &[Unit]) -> Vec<Unit> {
    let b = &setup.borders;
    let by_state: BTreeMap<u16, usize> = world
        .econ
        .regions
        .iter()
        .enumerate()
        .map(|(i, r)| (r.state, i))
        .collect();
    let alive: BTreeSet<&str> = world
        .countries
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            world
                .econ
                .regions
                .iter()
                .any(|r| usize::from(r.market) == *i)
        })
        .map(|(_, (_, c))| c.tag.as_str())
        .collect();
    let mut used = BTreeSet::new();
    let mut out = Vec::new();
    for cand in &b.candidates.countries {
        if cand.ended.is_some() || cand.rename_of.is_some() || cand.states.is_empty() {
            continue;
        }
        if alive.contains(cand.tag.as_str()) {
            continue;
        }
        let regions: Vec<usize> = cand
            .states
            .iter()
            .filter_map(|s| b.state_ids.get(s).and_then(|id| by_state.get(id)).copied())
            .filter(|ri| !used.contains(ri))
            .collect();
        let total: i64 = regions
            .iter()
            .map(|&ri| world.econ.regions[ri].population())
            .sum();
        if total <= 0 {
            continue;
        }
        let mut by_owner: BTreeMap<u16, i64> = BTreeMap::new();
        for &ri in &regions {
            let r = &world.econ.regions[ri];
            if r.colony {
                *by_owner.entry(r.market).or_default() += r.population();
            }
        }
        let Some((&owner, &people)) = by_owner
            .iter()
            .max_by(|x, y| x.1.cmp(y.1).then(y.0.cmp(x.0)))
        else {
            continue;
        };
        if people * 2 <= total {
            continue;
        }
        let mine: Vec<u32> = regions
            .iter()
            .copied()
            .filter(|&ri| {
                let r = &world.econ.regions[ri];
                r.colony && r.market == owner
            })
            .map(|ri| ri as u32)
            .collect();
        for &ri in &mine {
            used.insert(ri as usize);
        }
        let old = previous
            .iter()
            .find(|u| u.key == cand.tag && u.owner == owner);
        out.push(Unit {
            key: cand.tag.clone(),
            owner,
            regions: mine,
            movement: old.map(|u| u.movement).unwrap_or_default(),
            mobilized: old.map_or(0, |u| u.mobilized),
            demand: old.and_then(|u| u.demand.clone()),
        });
    }
    out
}

/// Recruits of a movement kind, per mille of each profession.
fn recruits(kind: &crate::politics::MovementKind) -> [u16; P] {
    let mut r = [if kind.professions.is_empty() { 1000 } else { 0 }; P];
    for &(p, s) in &kind.professions {
        r[usize::from(p)] = s;
    }
    r
}

/// One year of the unit's movement: the same rule as the politics' (`polity::system`), measured
/// on the unit's people under the owner's colonial laws. Returns the mobilization, per mille.
fn advance(
    kind: &crate::politics::MovementKind,
    st: &mut MovementState,
    f: &Facts,
    row: &[u16; P],
    effects: &[i32],
    legitimacy: i32,
) -> u16 {
    let people = f.population();
    let recruits: i64 = (0..P)
        .map(|q| (i128::from(f.size[q]) * i128::from(row[q]) / 1000) as i64)
        .sum();
    let mut weakest = 0u8;
    let mut pressure = i32::MAX;
    for (i, &(m, b)) in kind.cause.iter().enumerate() {
        let value = match m {
            Measure::Indicator(x) if usize::from(x) == measure::LEGITIMACY => Some(legitimacy),
            Measure::Indicator(x) => f.indicator(usize::from(x), row),
            Measure::Effect(e) => effects.get(usize::from(e)).copied(),
        };
        let mg = value.map_or(0, |v| margin(b, v, m.range()));
        if mg < pressure {
            pressure = mg;
            weakest = i as u8;
        }
    }
    if people == 0 || recruits == 0 {
        pressure = 0;
    }
    let pressure = pressure.clamp(0, 1000);
    let mut support = i32::from(st.support);
    if pressure > 0 {
        support += (i32::from(kind.growth) * pressure / 1000).max(1);
    } else if support > 0 {
        support -= (support * 693 / (1000 * i32::from(kind.persistence))).max(1);
    }
    st.support = support.clamp(0, 1000) as u16;
    st.pressure = pressure as u16;
    st.weakest = weakest;
    if people > 0 {
        (i64::from(st.support) * recruits / people).clamp(0, 1000) as u16
    } else {
        0
    }
}

fn milli(n: u32) -> Fixed {
    Fixed::from_raw(i64::from(n))
}

fn gdp(world: &GameWorld, c: usize) -> Fixed {
    world.econ.markets.get(c).map_or(Fixed::ZERO, |m| {
        if m.gdp_last_year > Fixed::ZERO {
            m.gdp_last_year
        } else {
            m.gdp_running
        }
    })
}

/// Anticolonial stance of every country, per mille: born of an independence in the game, or
/// holding no colonies; a holder of colonies has none.
pub fn stances(rules: &IndependenceRules, world: &GameWorld) -> Vec<u16> {
    let n = world.econ.markets.len();
    let mut colonies = vec![false; n];
    let mut alive = vec![false; n];
    for r in &world.econ.regions {
        let m = usize::from(r.market);
        if m < n {
            alive[m] = true;
            colonies[m] |= r.colony;
        }
    }
    let born: BTreeSet<&str> = world
        .independence
        .history
        .iter()
        .map(|r| r.unit.as_str())
        .collect();
    (0..n)
        .map(|c| {
            if !alive[c] || colonies[c] {
                return 0;
            }
            let tag = world
                .countries
                .iter()
                .nth(c)
                .map_or("", |(_, x)| x.tag.as_str());
            if born.contains(tag) {
                rules.born_independent
            } else {
                rules.no_colonies
            }
        })
        .collect()
}

/// The pressure of the anticolonial countries' influence over `owner` (issue Money, weighted by
/// stance), share of its GDP: the material cost, the same for all its units.
pub fn material(
    setup: &IndependenceSetup,
    world: &GameWorld,
    owner: usize,
    stance: &[u16],
    year: i32,
) -> Fixed {
    let mut out = Fixed::ZERO;
    if let Some(inf) = &setup.influence {
        for (x, v) in crate::influence::influence_on(world, inf, owner, year) {
            let s = stance.get(usize::from(x)).copied().unwrap_or(0);
            if s > 0 {
                out += v[0].max(Fixed::ZERO)
                    * Fixed::from_ratio(i64::from(s), 1000)
                    * milli(setup.rules.material_weight);
            }
        }
    }
    out
}

/// The account of holding `unit`: value against legitimacy, international and material cost
/// (`material`, from [`material`]), with the goodwill each anticolonial country withdraws this
/// year (country, share of its GDP).
pub fn account(
    setup: &IndependenceSetup,
    world: &GameWorld,
    unit: &Unit,
    stance: &[u16],
    material: Fixed,
) -> (Account, Vec<(u16, Fixed)>) {
    let r = &setup.rules;
    let o = usize::from(unit.owner);
    let gdp_o = gdp(world, o);
    if gdp_o <= Fixed::ZERO {
        return (Account::default(), Vec::new());
    }
    let cat = &setup.cat;
    let m = &world.econ.markets[o];
    let tribute = m.laws.effects[1].colonial_tribute;
    let mut va = Fixed::ZERO;
    for &ri in &unit.regions {
        for p in &world.econ.regions[ri as usize].plants {
            if p.levels <= 0 {
                continue;
            }
            va += crate::econ::monthly::base_value_added(cat, p.kind as usize, p.method as usize)
                * p.levels
                * p.staffing
                * 52;
        }
    }
    let value = va * tribute / gdp_o;
    // The unit's part of the country's colonial mobilization, at the politics' loss of 3 per
    // mille of legitimacy per mille mobilized.
    let colonial: i64 = world
        .econ
        .regions
        .iter()
        .filter(|x| x.colony && usize::from(x.market) == o)
        .map(|x| x.population())
        .sum();
    let unit_people: i64 = unit
        .regions
        .iter()
        .map(|&ri| world.econ.regions[ri as usize].population())
        .sum();
    let mob = Fixed::from_ratio(i64::from(unit.mobilized), 1000);
    let share = if colonial > 0 {
        Fixed::from_ratio(unit_people, colonial)
    } else {
        Fixed::ZERO
    };
    let legitimacy = mob * share * 3 * milli(r.legitimacy_price);
    let mut withdrawn = Vec::new();
    let mut international = Fixed::ZERO;
    if unit.movement.support > 0 {
        for (x, &s) in stance.iter().enumerate() {
            if s == 0 || x == o {
                continue;
            }
            let loss = (milli(r.reputation_rate) * Fixed::from_ratio(i64::from(s), 1000) * mob)
                .min(milli(r.reputation_cap));
            if loss <= Fixed::ZERO {
                continue;
            }
            let cost = loss * gdp(world, x) / gdp_o;
            international += cost;
            withdrawn.push((x as u16, loss, cost));
        }
    }
    let mut top: Vec<(u16, Fixed)> = withdrawn.iter().map(|(x, _, c)| (*x, *c)).collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(5);
    (
        Account {
            value,
            legitimacy,
            international,
            material,
            top,
        },
        withdrawn.into_iter().map(|(x, l, _)| (x, l)).collect(),
    )
}

/// The answer of a metropole without a player: refuse while the colony is worth more than it
/// costs; else the cheapest step that removes the cause (self-rule while it is not in force),
/// then a dominion, and independence when the cost is more than twice the value.
pub fn answer(acc: &Account, self_rule_in_force: bool) -> Answer {
    let cost = acc.cost();
    if cost <= acc.value {
        Answer::Refused
    } else if !self_rule_in_force {
        Answer::SelfRule
    } else if cost <= acc.value * 2 {
        Answer::Dominion
    } else {
        Answer::Independence
    }
}

pub struct IndependenceSystem {
    pub setup: Arc<IndependenceSetup>,
}

#[derive(Default)]
pub struct Changes {
    state: Option<IndependenceState>,
    /// Laws to ask for: (country, law).
    laws: Vec<(u16, u16)>,
    ops: Vec<Operation>,
    settlements: Vec<crate::treaty_exec::Settlement>,
    /// Goodwill acts: ((holder, country), value).
    goodwill: Vec<((u16, u16), Fixed)>,
}

impl IndependenceSystem {
    fn tag(world: &GameWorld, c: usize) -> String {
        world
            .countries
            .iter()
            .nth(c)
            .map_or_else(String::new, |(_, x)| x.tag.clone())
    }

    fn states_of(&self, world: &GameWorld, u: &Unit) -> Vec<String> {
        let ids: BTreeMap<u16, &String> = self
            .setup
            .borders
            .state_ids
            .iter()
            .map(|(k, v)| (*v, k))
            .collect();
        u.regions
            .iter()
            .filter_map(|&ri| {
                ids.get(&world.econ.regions[ri as usize].state)
                    .map(|s| (*s).clone())
            })
            .collect()
    }

    /// Carry out an answer.
    #[allow(clippy::too_many_arguments)]
    fn act(
        &self,
        world: &GameWorld,
        date: Date,
        trusteeship: Option<u16>,
        u: &mut Unit,
        a: Answer,
        out: &mut Changes,
        history: &mut Vec<Record>,
        gone: &mut BTreeSet<String>,
    ) {
        match a {
            Answer::SelfRule => {
                if let Some(l) = trusteeship {
                    out.laws.push((u.owner, l));
                }
            }
            Answer::Dominion | Answer::Independence => {
                let how = if a == Answer::Dominion {
                    How::Dominion
                } else {
                    How::Agreement
                };
                self.release(world, u, how, out);
                history.push(Record {
                    date,
                    unit: u.key.clone(),
                    owner: u.owner,
                    how,
                });
                gone.insert(u.key.clone());
            }
            Answer::Refused | Answer::Pending => {}
        }
        if let Some(d) = &mut u.demand {
            d.answer = a;
        }
    }

    /// Carry out an answer that lets the unit go.
    fn release(&self, world: &GameWorld, u: &Unit, how: How, out: &mut Changes) {
        let owner = Self::tag(world, usize::from(u.owner));
        let states = self.states_of(world, u);
        if states.is_empty() {
            return;
        }
        match how {
            How::Revolt => out.ops.push(Operation::Secede {
                from: owner,
                states,
                tag: u.key.clone(),
                overlord: None,
                // Without an agreement nobody signs: the metropole does not recognise it.
                signatories: Vec::new(),
                terms: Terms::default(),
            }),
            How::Agreement | How::Dominion => {
                let dominion = how == How::Dominion;
                // The metropole gives and, for a dominion, receives the bond; the new country is
                // side B (the settlement does not list it as a signatory of its own creation).
                let proposal = Proposal {
                    side_a: vec![owner.clone()],
                    side_b: vec![u.key.clone()],
                    clauses: vec![Clause {
                        kind: ClauseKind::ReleaseCountry {
                            states: states.clone(),
                            tag: u.key.clone(),
                            bundle: dominion.then(|| "dominion".to_owned()),
                        },
                        giver: Side::Senior,
                        receiver: Side::Senior,
                        until: None,
                        note: None,
                    }],
                    term_years: None,
                    notice_years: 0,
                };
                let settled = self.setup.clauses.as_deref().and_then(|cat| {
                    crate::treaty_exec::settle(cat, &proposal, &format!("independence_{}", u.key))
                        .ok()
                });
                match settled {
                    Some(s) => out.settlements.push(s),
                    None => out.ops.push(Operation::Secede {
                        from: owner.clone(),
                        states,
                        tag: u.key.clone(),
                        overlord: dominion.then(|| owner.clone()),
                        signatories: vec![owner, u.key.clone()],
                        terms: Terms::default(),
                    }),
                }
            }
        }
    }
}

impl System<GameWorld> for IndependenceSystem {
    type Changes = Changes;

    fn name(&self) -> &'static str {
        "independence"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }

    fn access(&self) -> Access {
        Access::new()
            .read(crate::econ::week::ECON)
            .read(crate::polity::POLITY)
            .write(crate::polity::POLITY)
            .read(crate::influence::INFLUENCE)
            .write(crate::influence::INFLUENCE)
            .read(crate::borders::op::BORDERS)
            .write(crate::borders::op::BORDERS)
            .read(INDEPENDENCE)
            .write(INDEPENDENCE)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> Changes {
        let s = &self.setup;
        let mut out = Changes::default();
        if !s.rules.enabled || world.econ.regions.is_empty() || world.politics.laws.is_empty() {
            return out;
        }
        let date = ctx.date;
        let state = &world.independence;
        // Answers by command, any day; the rest on January 2nd. Every other day ends here.
        let yearly = date.month == 1 && date.day == 2;
        if state.answers.is_empty() && !yearly {
            return out;
        }
        let player = world
            .player
            .and_then(|id| world.countries.iter().position(|(i, _)| i == id));
        let p = &*s.politics;
        let trusteeship = s.trusteeship;
        let self_rule =
            |c: usize| trusteeship.is_some_and(|l| world.politics.laws[c].contains(&Some(l)));
        let mut units = if yearly {
            units_of(s, world, &state.units)
        } else {
            state.units.clone()
        };
        let mut history = state.history.clone();
        let stance = stances(&s.rules, world);
        let mut gone = BTreeSet::new();
        for (key, a) in &state.answers {
            if let Some(u) = units.iter_mut().find(|u| u.key == *key)
                && u.demand
                    .as_ref()
                    .is_some_and(|d| d.answer == Answer::Pending)
                && *a != Answer::Pending
            {
                self.act(
                    world,
                    date,
                    trusteeship,
                    u,
                    *a,
                    &mut out,
                    &mut history,
                    &mut gone,
                );
            }
        }
        if yearly {
            let kind_ix = s.borders.independence;
            let services = s.services;
            let facts = measure::facts(&s.cat, services, &[], world);
            let mut materials: BTreeMap<u16, Fixed> = BTreeMap::new();
            for u in units.iter_mut() {
                if gone.contains(&u.key) {
                    continue;
                }
                let o = usize::from(u.owner);
                let Some(k) = kind_ix else { break };
                let kind = &p.movements[k];
                let mut f = Facts::default();
                for &ri in &u.regions {
                    f.add_region(&s.cat, services, &world.econ.regions[ri as usize]);
                }
                if let Some(of) = facts.get(o) {
                    f.tax_burden = of[1].tax_burden;
                    f.import_share = of[1].import_share;
                    f.fiscal_stress = of[1].fiscal_stress;
                    f.gdp_growth = of[1].gdp_growth;
                    f.immigrant_share = of[1].immigrant_share;
                }
                let effects = p.effects(&world.politics.laws[o], Scope::Colonies);
                let legitimacy =
                    i32::from(world.politics.legitimacy.get(o).copied().unwrap_or(1000));
                u.mobilized = advance(
                    kind,
                    &mut u.movement,
                    &f,
                    &recruits(kind),
                    &effects,
                    legitimacy,
                );
                let m = *materials
                    .entry(u.owner)
                    .or_insert_with(|| material(s, world, o, &stance, date.year));
                let (acc, withdrawn) = account(s, world, u, &stance, m);
                for (x, loss) in withdrawn {
                    out.goodwill.push(((u.owner, x), -loss));
                }
                let refused = u
                    .demand
                    .as_ref()
                    .is_some_and(|d| d.answer == Answer::Refused);
                if refused && u.mobilized >= s.rules.revolt {
                    self.release(world, u, How::Revolt, &mut out);
                    history.push(Record {
                        date,
                        unit: u.key.clone(),
                        owner: u.owner,
                        how: How::Revolt,
                    });
                    gone.insert(u.key.clone());
                    continue;
                }
                let pending_late = u.demand.as_ref().is_some_and(|d| {
                    d.answer == Answer::Pending
                        && (date.year - d.since.year) * 12 + i32::from(date.month)
                            - i32::from(d.since.month)
                            >= i32::from(s.rules.answer_months)
                });
                if pending_late {
                    if let Some(d) = &mut u.demand {
                        d.answer = Answer::Refused;
                    }
                    continue;
                }
                if u.mobilized < s.rules.demand {
                    continue;
                }
                if player == Some(o) {
                    if !u
                        .demand
                        .as_ref()
                        .is_some_and(|d| d.answer == Answer::Pending)
                    {
                        u.demand = Some(Demand {
                            since: date,
                            answer: Answer::Pending,
                            account: acc,
                        });
                    }
                    continue;
                }
                let a = answer(&acc, self_rule(o));
                u.demand = Some(Demand {
                    since: date,
                    answer: a,
                    account: acc,
                });
                self.act(
                    world,
                    date,
                    trusteeship,
                    u,
                    a,
                    &mut out,
                    &mut history,
                    &mut gone,
                );
            }
        }
        units.retain(|u| !gone.contains(&u.key));
        out.state = Some(IndependenceState {
            units,
            history,
            answers: Vec::new(),
        });
        out
    }

    fn apply(&self, ctx: &TickCtx, changes: Changes, world: &mut GameWorld) {
        if let Some(st) = changes.state {
            world.independence = st;
        }
        for (c, l) in changes.laws {
            world.politics.request(c, l);
        }
        world.borders.pending.extend(changes.ops);
        for s in &changes.settlements {
            crate::treaty_exec::queue(world, s, ctx.date.year);
        }
        for (key, v) in changes.goodwill {
            world.influence.goodwill.entry(key).or_default().add(v);
        }
    }
}

/// The Fluent key of the condition holding a movement back (`MovementState::weakest`):
/// `indicator-<key>` or `law-effect-<key>`.
pub fn weakest_key(kind: &crate::politics::MovementKind, weakest: u8) -> Option<String> {
    let (m, _) = kind.cause.get(usize::from(weakest))?;
    Some(match m {
        Measure::Indicator(i) => format!(
            "indicator-{}",
            crate::politics::INDICATORS.get(usize::from(*i))?.key
        ),
        Measure::Effect(e) => format!(
            "law-effect-{}",
            crate::politics::EFFECTS.get(usize::from(*e))?.key
        ),
    })
}

/// A demand as the player's screen shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemandView {
    /// The candidate's tag (`country-<tag>`).
    pub unit: String,
    /// The unit's states (`state-<key>`).
    pub states: Vec<String>,
    /// Mobilization and the two marks, per mille.
    pub mobilized: u16,
    pub demand_mark: u16,
    pub revolt_mark: u16,
    pub since: Date,
    /// Last day to answer; after it, silence counts as a refusal.
    pub deadline: Date,
    pub answer: Answer,
    /// The account when the demand was made.
    pub account: Account,
    /// `indicator-<key>` or `law-effect-<key>`: what holds the movement back.
    pub weakest: Option<String>,
}

fn add_months(d: Date, months: u8) -> Date {
    let m0 = i32::from(d.month) - 1 + i32::from(months);
    Date::new(
        d.year + m0.div_euclid(12),
        (m0.rem_euclid(12) + 1) as u8,
        d.day.min(28),
    )
}

/// The pending demands to `owner`, for the screen.
pub fn demands_of(setup: &IndependenceSetup, world: &GameWorld, owner: u16) -> Vec<DemandView> {
    let ids: BTreeMap<u16, &String> = setup
        .borders
        .state_ids
        .iter()
        .map(|(k, v)| (*v, k))
        .collect();
    let kind = setup
        .borders
        .independence
        .and_then(|k| setup.politics.movements.get(k));
    world
        .independence
        .units
        .iter()
        .filter(|u| u.owner == owner)
        .filter_map(|u| {
            let d = u.demand.as_ref().filter(|d| d.answer == Answer::Pending)?;
            Some(DemandView {
                unit: u.key.clone(),
                states: u
                    .regions
                    .iter()
                    .filter_map(|&ri| {
                        ids.get(&world.econ.regions[ri as usize].state)
                            .map(|s| (*s).clone())
                    })
                    .collect(),
                mobilized: u.mobilized,
                demand_mark: setup.rules.demand,
                revolt_mark: setup.rules.revolt,
                since: d.since,
                deadline: add_months(d.since, setup.rules.answer_months),
                answer: d.answer,
                account: d.account.clone(),
                weakest: kind.and_then(|k| weakest_key(k, u.movement.weakest)),
            })
        })
        .collect()
}

/// What an answer changes, a year after it, each a share of the metropole's GDP a year: the
/// account as it would stand, and for a refusal how near the revolt is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effects {
    pub after: Account,
    /// For a refusal: (mobilization, mark of the revolt), per mille.
    pub revolt: Option<(u16, u16)>,
}

/// The price of each answer to `unit`'s demand, computed by the simulation for the screen:
/// - self-rule: the cause stops holding (the law raises self-rule past the cause's bound), the
///   colony stays, and the costs stay this year (the movement only decays);
/// - dominion and independence: the colony leaves, with its value and every cost of holding it;
/// - refusal: the account as it is, and the revolt's distance.
pub fn answer_effects(
    setup: &IndependenceSetup,
    world: &GameWorld,
    unit: &str,
    answer: Answer,
) -> Option<Effects> {
    let u = world.independence.units.iter().find(|u| u.key == unit)?;
    let acc = u
        .demand
        .as_ref()
        .map(|d| d.account.clone())
        .unwrap_or_default();
    Some(match answer {
        Answer::Dominion | Answer::Independence => Effects {
            after: Account {
                material: acc.material,
                ..Default::default()
            },
            revolt: None,
        },
        Answer::SelfRule => Effects {
            after: acc,
            revolt: None,
        },
        Answer::Refused | Answer::Pending => Effects {
            after: acc,
            revolt: Some((u.mobilized, setup.rules.revolt)),
        },
    })
}
