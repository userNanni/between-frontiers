//! Border operations in the tick (part B of `docs/borders.pt-BR.md`): move states between
//! countries, create a country (independent or a subject), annex a whole country, change a
//! subject's status. A command only queues the operation ([`BordersState::pending`]); the
//! `borders` system, which holds the map and the catalogues ([`BorderRules`]), runs it at the end
//! of the day. Every operation is checked before anything moves: a refused one changes nothing but
//! the record of refusals.
//!
//! The country index is stable: a new country goes to the end of every table, a country that ends
//! keeps its place with no states (a tombstone) and may be born again under the same tag. Money and
//! people are neither created nor destroyed ([`super::Totals`]), except the debt an agreement
//! creates, which comes with a claim of the same size.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use laplace_schelling::Date;
use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};
use serde::{Deserialize, Serialize};

use super::CandidatesFile;
use crate::econ::Market;
use crate::game::{CountryState, GameWorld};
use crate::research::Stage;
use crate::scenario::{CountriesFile, MapFile, Status, SubjectKind, Tag};

/// The border state of the world.
pub const BORDERS: ResourceId = ResourceId(14);

/// Public debt that changes debtor by agreement (decision 1 of `docs/borders.pt-BR.md`): `to`
/// takes over `amount` (reference units) of the public debt of `from`, from every holder in
/// proportion ([`DebtTerm::Market`]). Debt an agreement creates between states (a reparation,
/// the price of a recognition, the debt a new country is born with) goes to the named ledger of
/// the treaties ([`DebtTerm::Agreement`], `docs/treaties.pt-BR.md`), resolved when the operation
/// runs, so that a country born by the same operation can owe or be owed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebtTerm {
    Market {
        from: Tag,
        to: Tag,
        amount: Fixed,
    },
    /// A new line of the ledger (`Economy::ledger`): `debtor` owes `creditor` under `treaty`.
    Agreement {
        debtor: Tag,
        creditor: Tag,
        treaty: String,
        terms: crate::treaty::DebtTerms,
    },
}

/// A technique carried by people and papers (decision 6): the winner of a peace taking scientists
/// and projects, a spy. `to` reaches `stage`, never above the stage of `from`; `people` (0–1) is
/// the share of the progress toward its next stage `from` loses with them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechniqueTerm {
    pub tech: u16,
    pub from: Tag,
    pub to: Tag,
    pub stage: Stage,
    pub people: Fixed,
}

/// Terms any operation may carry; written by a treaty (phase 4) or a peace (phase 5).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terms {
    #[serde(default)]
    pub debt: Vec<DebtTerm>,
    #[serde(default)]
    pub techniques: Vec<TechniqueTerm>,
}

impl Terms {
    pub fn is_empty(&self) -> bool {
        self.debt.is_empty() && self.techniques.is_empty()
    }
}

/// One border operation. States are map state keys (`StateDef::key`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// States move between two countries that hold land. `colony`: what they become for the
    /// receiver; `None` keeps what they were.
    Transfer {
        from: Tag,
        to: Tag,
        states: Vec<String>,
        colony: Option<bool>,
        terms: Terms,
    },
    /// States become country `tag` (new, or born again from a tombstone), independent or a subject
    /// of `overlord`. `signatories`: who signs the creation, and so recognizes the new country
    /// (the owner's decision of 2026-11-17, `docs/treaties.pt-BR.md`); the treaties keep the
    /// pairs that do not recognize each other.
    Secede {
        from: Tag,
        states: Vec<String>,
        tag: Tag,
        overlord: Option<Tag>,
        #[serde(default)]
        signatories: Vec<Tag>,
        terms: Terms,
    },
    /// Country `from` ends inside `into`; its states become metropole or colonies of `into`.
    Annex {
        from: Tag,
        into: Tag,
        colony: bool,
        terms: Terms,
    },
    /// `tag` becomes a subject of `overlord`, or sovereign with `None`.
    Status { tag: Tag, overlord: Option<Tag> },
    /// Only the terms (an agreement with no territory: a reparation, a technique handed over).
    Terms { terms: Terms },
    /// Country `from` ends, all of its states split among new (or reborn) countries: the
    /// Czechoslovakia of 1993, Yugoslavia, the Mali Federation. Every state of `from` goes to
    /// exactly one heir. The treasury, the fund, the savers' claims abroad and the public debt
    /// split among the heirs by the income of their states (**w**).
    Dissolve {
        from: Tag,
        heirs: Vec<Heir>,
        terms: Terms,
    },
}

/// The terms an operation carries, if it carries any.
pub fn terms_mut(op: &mut Operation) -> Option<&mut Terms> {
    match op {
        Operation::Transfer { terms, .. }
        | Operation::Secede { terms, .. }
        | Operation::Annex { terms, .. }
        | Operation::Terms { terms }
        | Operation::Dissolve { terms, .. } => Some(terms),
        Operation::Status { .. } => None,
    }
}

/// One country born of a dissolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heir {
    pub tag: Tag,
    pub states: Vec<String>,
    #[serde(default)]
    pub overlord: Option<Tag>,
}

/// A subject's tie to its overlord, read by the diplomacy to decide when it breaks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub overlord: u16,
    pub kind: SubjectKind,
    pub since: Date,
}

/// An operation that ran.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub date: Date,
    pub op: Operation,
    /// Index of the country the operation created or brought back.
    pub born: Option<u16>,
    /// Index of the country that ended.
    pub ended: Option<u16>,
    /// People abroad (and of them, arrivals since the start) whose origin and destination became
    /// one territory: no longer abroad.
    #[serde(default)]
    pub repatriated: (i64, i64),
}

/// An operation that was refused, with why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refusal {
    pub date: Date,
    pub op: Operation,
    pub reason: String,
}

/// Saved and hashed only once something happened: a world where no operation ever ran keeps the
/// hash it had before the border system existed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BordersState {
    /// Operations queued by commands, run at the end of the day, oldest first.
    pub pending: Vec<Operation>,
    pub changes: Vec<ChangeRecord>,
    pub refused: Vec<Refusal>,
    /// Overlord of every country, by country index; filled from the scenario at the first
    /// operation (until then [`BorderRules::subjects`] holds it).
    pub subjects: Vec<Option<Subject>>,
}

impl BordersState {
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.changes.is_empty() && self.refused.is_empty()
    }
}

impl HashState for Subject {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.overlord);
        h.write_u8(self.kind.clone() as u8);
        self.since.hash_state(h);
    }
}

impl HashState for BordersState {
    fn hash_state(&self, h: &mut StateHasher) {
        // Operations are hashed through their serialized form: one rule for every variant.
        let text = |op: &Operation| ron::to_string(op).unwrap_or_default();
        h.write_len(self.pending.len());
        for op in &self.pending {
            h.write_str(&text(op));
        }
        h.write_len(self.changes.len());
        for c in &self.changes {
            c.date.hash_state(h);
            h.write_str(&text(&c.op));
            c.born.map(u32::from).hash_state(h);
            c.ended.map(u32::from).hash_state(h);
            h.write_i64(c.repatriated.0);
            h.write_i64(c.repatriated.1);
        }
        h.write_len(self.refused.len());
        for r in &self.refused {
            r.date.hash_state(h);
            h.write_str(&text(&r.op));
            h.write_str(&r.reason);
        }
        h.write_len(self.subjects.len());
        for s in &self.subjects {
            s.hash_state(h);
        }
    }
}

/// What the operations read besides the world: the map, the catalogues and the scenario.
#[derive(Clone)]
pub struct BorderRules {
    /// Land neighbours of every map state, by state id.
    pub neighbours: BTreeMap<u16, Vec<u16>>,
    /// State id of every state key.
    pub state_ids: BTreeMap<String, u16>,
    pub map: Arc<MapFile>,
    pub communities: Option<Arc<crate::communities::Communities>>,
    pub candidates: CandidatesFile,
    /// Overlords on the start date, by country index.
    pub subjects: Vec<Option<Subject>>,
    pub tree: Option<Arc<crate::research::TechTree>>,
    /// Index of the movement whose goal is independence.
    pub independence: Option<usize>,
    /// Law groups that hold only for colonies.
    pub colonial_groups: Vec<bool>,
    /// How a new country is recognised (`treaty::Policy::recognition_default`); `None` without
    /// the treaty data: recognition is not kept.
    pub recognition: Option<crate::treaty::RecognitionDefault>,
    /// Building kinds whose private levels in a colony stay the old metropole's at independence
    /// (`foreign_stakes::colonial_stakes`): neither farms, nor public, nor informal. Empty (no
    /// catalogue, or the rule switched off): none.
    pub colonial_kinds: Vec<bool>,
}

impl BorderRules {
    pub fn new(
        map: Arc<MapFile>,
        communities: Option<Arc<crate::communities::Communities>>,
        candidates: CandidatesFile,
        countries: &CountriesFile,
        tree: Option<Arc<crate::research::TechTree>>,
        politics: Option<&crate::politics::Politics>,
        start: Date,
    ) -> Self {
        let index_of = |t: &str| countries.countries.iter().position(|c| c.tag == t);
        let subjects = countries
            .countries
            .iter()
            .map(|c| match &c.status {
                Status::Subject { overlord, kind } => index_of(overlord).map(|o| Subject {
                    overlord: o as u16,
                    kind: kind.clone(),
                    since: start,
                }),
                Status::Sovereign => None,
            })
            .collect();
        Self {
            neighbours: map
                .states
                .iter()
                .map(|s| (s.id, s.neighbours.clone()))
                .collect(),
            state_ids: map.states.iter().map(|s| (s.key.clone(), s.id)).collect(),
            map,
            communities,
            candidates,
            subjects,
            tree,
            independence: politics.and_then(|p| {
                p.movements
                    .iter()
                    .position(|m| matches!(m.goal, crate::politics::GoalRef::Independence))
            }),
            colonial_groups: politics
                .map(|p| p.groups.iter().map(|g| g.colonial).collect())
                .unwrap_or_default(),
            recognition: None,
            colonial_kinds: Vec::new(),
        }
    }

    /// The same rules with the colony's plants staying the old metropole's at independence
    /// (`Switches::foreign_contract`).
    pub fn with_colonial_plants(&self, cat: &crate::econ::Catalogue) -> Self {
        let mut out = self.clone();
        out.colonial_kinds = cat
            .buildings
            .iter()
            .map(|b| !b.agriculture && !b.public && !b.informal)
            .collect();
        out
    }

    /// The overlord of country `c`, if it is a subject.
    pub fn overlord_of(&self, world: &GameWorld, c: usize) -> Option<usize> {
        let subjects = if world.borders.subjects.is_empty() {
            &self.subjects
        } else {
            &world.borders.subjects
        };
        subjects
            .get(c)
            .and_then(|s| s.as_ref())
            .map(|s| usize::from(s.overlord))
    }
}

fn tag_ix(world: &GameWorld, tag: &str) -> Option<usize> {
    world.countries.iter().position(|(_, c)| c.tag == tag)
}

/// Regions each country holds.
fn held(world: &GameWorld) -> Vec<usize> {
    let mut n = vec![0usize; world.countries.len()];
    for r in &world.econ.regions {
        if let Some(x) = n.get_mut(usize::from(r.market)) {
            *x += 1;
        }
    }
    n
}

/// Three or four characters: a capital letter, then capitals or digits (`X00` for a country
/// history did not make).
fn valid_tag(t: &str) -> bool {
    let b = t.as_bytes();
    (3..=4).contains(&b.len())
        && b[0].is_ascii_uppercase()
        && b[1..]
            .iter()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// The region indices of `states`, all held by `from`; or why not.
fn regions_of(
    rules: &BorderRules,
    world: &GameWorld,
    from: usize,
    states: &[String],
) -> Result<Vec<usize>, String> {
    if states.is_empty() {
        return Err("no state listed".into());
    }
    let by_state: BTreeMap<u16, usize> = world
        .econ
        .regions
        .iter()
        .enumerate()
        .map(|(i, r)| (r.state, i))
        .collect();
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for key in states {
        if !seen.insert(key) {
            return Err(format!("state {key} listed twice"));
        }
        let Some(&ri) = rules.state_ids.get(key).and_then(|id| by_state.get(id)) else {
            return Err(format!("unknown state {key}"));
        };
        if usize::from(world.econ.regions[ri].market) != from {
            return Err(format!(
                "state {key} is not held by {}",
                world
                    .countries
                    .iter()
                    .nth(from)
                    .map_or("?", |(_, c)| c.tag.as_str())
            ));
        }
        out.push(ri);
    }
    out.sort_unstable();
    Ok(out)
}

fn check_terms(world: &GameWorld, rules: &BorderRules, terms: &Terms) -> Result<(), String> {
    let known = |t: &str| tag_ix(world, t).ok_or_else(|| format!("unknown tag {t}"));
    for d in &terms.debt {
        match d {
            DebtTerm::Market {
                from: a,
                to: b,
                amount,
            } => {
                if known(a)? == known(b)? {
                    return Err("a debt term needs two countries".into());
                }
                if *amount <= Fixed::ZERO {
                    return Err("a debt term needs a positive amount".into());
                }
            }
            // The parties may be born by the operation that carries the term: checked when it
            // runs, where an unknown tag drops the line (nothing is created on one side only).
            DebtTerm::Agreement {
                debtor,
                creditor,
                terms,
                ..
            } => {
                if debtor == creditor {
                    return Err("a debt term needs two countries".into());
                }
                if terms.principal <= Fixed::ZERO || terms.years == 0 {
                    return Err("a debt term needs a principal and a term".into());
                }
            }
        }
    }
    let techs = rules.tree.as_ref().map_or(0, |t| t.techs.len());
    for t in &terms.techniques {
        if known(&t.from)? == known(&t.to)? {
            return Err("a technique term needs two countries".into());
        }
        if usize::from(t.tech) >= techs {
            return Err(format!("unknown technology {}", t.tech));
        }
        if t.people < Fixed::ZERO || t.people > Fixed::ONE {
            return Err("people must be 0 to 1".into());
        }
    }
    Ok(())
}

/// Checks `op` against the world; nothing moves.
pub fn check(world: &GameWorld, rules: &BorderRules, op: &Operation) -> Result<(), String> {
    if world.econ.regions.is_empty() {
        return Err("no economy".into());
    }
    let holds = held(world);
    let alive = |t: &str| -> Result<usize, String> {
        let c = tag_ix(world, t).ok_or_else(|| format!("unknown tag {t}"))?;
        if holds[c] == 0 {
            return Err(format!("{t} holds no state"));
        }
        Ok(c)
    };
    match op {
        Operation::Transfer {
            from,
            to,
            states,
            terms,
            ..
        } => {
            let (f, t) = (alive(from)?, alive(to)?);
            if f == t {
                return Err("from and to are the same".into());
            }
            let rs = regions_of(rules, world, f, states)?;
            if rs.len() == holds[f] {
                return Err(format!("{from} would give every state: annex it instead"));
            }
            check_terms(world, rules, terms)
        }
        Operation::Secede {
            from,
            states,
            tag,
            overlord,
            signatories,
            terms,
        } => {
            let f = alive(from)?;
            if !valid_tag(tag) {
                return Err(format!("bad tag {tag}"));
            }
            if tag_ix(world, tag).is_some_and(|c| holds[c] > 0) {
                return Err(format!("{tag} already holds states"));
            }
            if let Some(o) = overlord {
                alive(o)?;
                if o == tag {
                    return Err("a country is not its own overlord".into());
                }
            }
            for s in signatories {
                tag_ix(world, s).ok_or_else(|| format!("unknown signatory {s}"))?;
            }
            let rs = regions_of(rules, world, f, states)?;
            if rs.len() == holds[f] {
                return Err(format!("{from} would free every state it holds"));
            }
            check_terms(world, rules, terms)
        }
        Operation::Annex {
            from, into, terms, ..
        } => {
            let (f, t) = (alive(from)?, alive(into)?);
            if f == t {
                return Err("a country does not annex itself".into());
            }
            check_terms(world, rules, terms)
        }
        Operation::Status { tag, overlord } => {
            let c = alive(tag)?;
            if let Some(o) = overlord {
                let o = alive(o)?;
                if o == c {
                    return Err("a country is not its own overlord".into());
                }
            }
            Ok(())
        }
        Operation::Terms { terms } => {
            if terms.is_empty() {
                return Err("no terms".into());
            }
            check_terms(world, rules, terms)
        }
        Operation::Dissolve { from, heirs, terms } => {
            let f = alive(from)?;
            if heirs.is_empty() {
                return Err("a dissolution needs heirs".into());
            }
            let mut tags = BTreeSet::new();
            let mut all = Vec::new();
            for h in heirs {
                if !valid_tag(&h.tag) {
                    return Err(format!("bad tag {}", h.tag));
                }
                if h.tag == *from || !tags.insert(h.tag.as_str()) {
                    return Err(format!("heir {} repeated or the dissolved country", h.tag));
                }
                if tag_ix(world, &h.tag).is_some_and(|c| holds[c] > 0) {
                    return Err(format!("{} already holds states", h.tag));
                }
                if let Some(o) = &h.overlord {
                    alive(o)?;
                }
                all.extend(h.states.iter().cloned());
            }
            let rs = regions_of(rules, world, f, &all)?;
            if rs.len() != holds[f] {
                return Err(format!(
                    "the heirs take {} of the {} states of {from}",
                    rs.len(),
                    holds[f]
                ));
            }
            check_terms(world, rules, terms)
        }
    }
}

/// Share of the country's measured income held in `regions` (the **w** of the design: what the
/// pops of those states earned last week, wages and profits), and of its people (**p**).
fn weights(world: &GameWorld, market: usize, regions: &[usize]) -> (Fixed, Fixed) {
    let (mut inc, mut inc_all, mut pop, mut pop_all) = (Fixed::ZERO, Fixed::ZERO, 0i64, 0i64);
    let moved: BTreeSet<usize> = regions.iter().copied().collect();
    for (i, r) in world.econ.regions.iter().enumerate() {
        if usize::from(r.market) != market {
            continue;
        }
        let ri: Fixed = r.pops.iter().map(|p| p.income.max(Fixed::ZERO)).sum();
        let rp = r.population();
        inc_all += ri;
        pop_all += rp;
        if moved.contains(&i) {
            inc += ri;
            pop += rp;
        }
    }
    let p = if pop_all > 0 {
        Fixed::from_ratio(pop, pop_all)
    } else {
        Fixed::ZERO
    };
    let w = if inc_all > Fixed::ZERO {
        (inc / inc_all).clamp(Fixed::ZERO, Fixed::ONE)
    } else {
        p
    };
    (w, p)
}

/// Moves `share` of `*from` to `*to`; the rounding stays with `from`.
fn shift(from: &mut Fixed, to: &mut Fixed, share: Fixed) {
    let x = *from * share;
    *from -= x;
    *to += x;
}

fn two<T>(v: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
    assert_ne!(a, b);
    if a < b {
        let (l, r) = v.split_at_mut(b);
        (&mut l[a], &mut r[0])
    } else {
        let (l, r) = v.split_at_mut(a);
        (&mut r[0], &mut l[b])
    }
}

/// The market of a country born out of `parent`: its prices, productivity, currency and rules,
/// with no money, no debt, no works.
fn new_market(parent: &Market) -> Market {
    let mut m = parent.clone();
    let zeros = vec![Fixed::ZERO; m.construction.len()];
    m.pool = Fixed::ZERO;
    m.treasury = Fixed::ZERO;
    m.construction = zeros;
    m.gdp_running = Fixed::ZERO;
    m.gdp_last_year = Fixed::ZERO;
    m.gov_workers = 0;
    m.queue.clear();
    let b = &mut m.budget;
    for v in [
        &mut b.revenue,
        &mut b.issued,
        &mut b.payroll,
        &mut b.goods,
        &mut b.services,
        &mut b.works,
        &mut b.revenue_avg,
        &mut b.debt,
        &mut b.interest,
        &mut b.beyond_rule,
        &mut b.revenue_indirect,
        &mut b.revenue_firms,
    ] {
        *v = Fixed::ZERO;
    }
    let y = &mut m.money;
    for v in [
        &mut y.debt_home,
        &mut y.debt_bank,
        &mut y.debt_abroad,
        &mut y.debt_fx,
        &mut y.foreign_assets,
        &mut y.default_history,
        &mut y.monetized,
        &mut y.credit,
    ] {
        *v = Fixed::ZERO;
    }
    y.defaults = 0;
    y.excluded_until = 0;
    y.redenominations = 0;
    y.rate_command = None;
    y.bank = crate::econ::money::CentralBank::Historical;
    m
}

/// Index of country `tag`: an existing one (a tombstone is born again), or a new row at the end of
/// every table, made from `parent`.
fn ensure_country(world: &mut GameWorld, rules: &BorderRules, tag: &str, parent: usize) -> usize {
    if let Some(c) = tag_ix(world, tag) {
        // Born again: the parent's market and rules, no money.
        world.econ.markets[c] = new_market(&world.econ.markets[parent]);
        copy_rows(world, rules, parent, c);
        return c;
    }
    let c = world.countries.len();
    world.countries.push(CountryState {
        tag: tag.to_owned(),
        population: 0,
        gdp_per_capita: 0,
        colonial_population: 0,
        colonial_gdp_per_capita: 0,
    });
    let m = new_market(&world.econ.markets[parent]);
    world.econ.markets.push(m);
    let r = &mut world.research;
    if !r.is_empty() {
        r.stage.push(r.stage[parent].clone());
        r.progress.push(r.progress[parent].clone());
        r.focus.push(None);
        r.points.push(Fixed::ZERO);
        if let Some(e) = r.effects.get(parent).copied() {
            r.effects.push(e);
        }
    }
    let p = &mut world.politics;
    if !p.is_empty() {
        p.laws.push(p.laws[parent].clone());
        p.targets.push(Vec::new());
        p.clout.push(p.clout[parent].clone());
        if !p.leaders.is_empty() {
            p.leaders.push(vec![None; p.leaders[parent].len()]);
        }
        p.movements.push(p.movements[parent].clone());
        p.legitimacy.push(1000);
        if let Some(t) = p.taxes.get(parent).copied() {
            p.taxes.push(t);
        }
    }
    world.borders.subjects.push(None);
    copy_rows(world, rules, parent, c);
    c
}

/// The research and politics a new country takes from `parent` (decision 6, and the laws of
/// `docs/borders.pt-BR.md`). The techniques its plants use are applied by
/// [`keep_plant_techniques`] once its states are in.
fn copy_rows(world: &mut GameWorld, rules: &BorderRules, parent: usize, c: usize) {
    let r = &mut world.research;
    if !r.is_empty() {
        let stage: Vec<Stage> = r.stage[parent]
            .iter()
            .map(|&s| s.min(Stage::Prototype))
            .collect();
        let progress = r.stage[parent]
            .iter()
            .zip(&r.progress[parent])
            .map(|(&s, &p)| if s > Stage::Prototype { Fixed::ZERO } else { p })
            .collect();
        r.stage[c] = stage;
        r.progress[c] = progress;
        r.focus[c] = None;
        r.points[c] = Fixed::ZERO;
    }
    let p = &mut world.politics;
    if !p.is_empty() {
        let mut laws = p.laws[parent].clone();
        for (g, l) in laws.iter_mut().enumerate() {
            if rules.colonial_groups.get(g).copied().unwrap_or(false) {
                *l = None;
            }
        }
        p.laws[c] = laws;
        p.targets[c].clear();
        p.clout[c] = p.clout[parent].clone();
        let mut mv = p.movements[parent].clone();
        if let Some(i) = rules.independence
            && let Some(m) = mv.get_mut(i)
        {
            *m = Default::default();
        }
        p.movements[c] = mv;
        p.legitimacy[c] = 1000;
    }
}

/// A new country keeps, at its parent's stage, the techniques its own plants use (decision 6:
/// the technique goes with the plant and the school, not with the flag).
fn keep_plant_techniques(
    world: &mut GameWorld,
    rules: &BorderRules,
    parent: usize,
    c: usize,
    regions: &[usize],
) {
    let Some(tree) = &rules.tree else { return };
    if world.research.is_empty() {
        return;
    }
    let mut used = BTreeSet::new();
    for &ri in regions {
        for p in &world.econ.regions[ri].plants {
            if p.levels <= 0 {
                continue;
            }
            if let Some(Some(t)) = tree
                .method_tech
                .get(usize::from(p.kind))
                .and_then(|m| m.get(usize::from(p.method)))
            {
                used.insert(usize::from(*t));
            }
        }
    }
    let r = &mut world.research;
    for t in used {
        r.stage[c][t] = r.stage[parent][t];
        r.progress[c][t] = r.progress[parent][t];
    }
}

/// Moves regions to market `to`, with what goes with the states: the works queued on them and a
/// share of the goods those works still have to buy. Neighbour lists are rebuilt.
fn move_regions(
    world: &mut GameWorld,
    rules: &BorderRules,
    from: usize,
    to: usize,
    regions: &[usize],
    colony: Option<bool>,
) {
    let moved: BTreeSet<u32> = regions.iter().map(|&r| r as u32).collect();
    for &ri in regions {
        let r = &mut world.econ.regions[ri];
        r.market = to as u16;
        if let Some(c) = colony {
            r.colony = c;
        }
    }
    // A state back with the country that owns plants in it: those are no longer foreign.
    let regions_now = &world.econ.regions;
    world
        .foreign
        .border_changed(|r| regions_now[r as usize].market);
    let (a, b) = two(&mut world.econ.markets, from, to);
    let left_all: Fixed = a.queue.iter().map(|p| p.left).sum();
    let left_moved: Fixed = a
        .queue
        .iter()
        .filter(|p| moved.contains(&p.region))
        .map(|p| p.left)
        .sum();
    let (go, stay): (Vec<_>, Vec<_>) = a.queue.drain(..).partition(|p| moved.contains(&p.region));
    a.queue = stay;
    b.queue.extend(go);
    if left_all > Fixed::ZERO {
        let share = (left_moved / left_all).clamp(Fixed::ZERO, Fixed::ONE);
        for (x, y) in a.construction.iter_mut().zip(b.construction.iter_mut()) {
            shift(x, y, share);
        }
    }
    rebuild_neighbours(world, rules);
}

/// `Region::neighbours`: the regions of the same market across a land border.
fn rebuild_neighbours(world: &mut GameWorld, rules: &BorderRules) {
    let regions = &world.econ.regions;
    let by_state: BTreeMap<u16, u32> = regions
        .iter()
        .enumerate()
        .map(|(i, r)| (r.state, i as u32))
        .collect();
    let lists: Vec<Vec<u32>> = regions
        .iter()
        .map(|r| {
            let mut v: Vec<u32> = rules
                .neighbours
                .get(&r.state)
                .into_iter()
                .flatten()
                .filter_map(|s| by_state.get(s).copied())
                .filter(|&j| regions[j as usize].market == r.market)
                .collect();
            v.sort_unstable();
            v.dedup();
            v
        })
        .collect();
    for (r, l) in world.econ.regions.iter_mut().zip(lists) {
        r.neighbours = l;
    }
}

/// The colonial laws `to` needs once it holds colonies: those of `from`, which held them.
fn colonial_laws(world: &mut GameWorld, rules: &BorderRules, from: usize, to: usize) {
    let p = &mut world.politics;
    if p.is_empty() {
        return;
    }
    for (g, &col) in rules.colonial_groups.iter().enumerate() {
        if col && p.laws[to][g].is_none() {
            p.laws[to][g] = p.laws[from][g];
        }
    }
}

/// The demography of the territories that changed: their facts again, the people abroad split by
/// the people of the states that moved, the migration waves of those territories closed.
fn demography(
    world: &mut GameWorld,
    rules: &BorderRules,
    old: &[(usize, crate::demography::TerritoryId)],
    date: Date,
) -> (i64, i64) {
    use crate::demography::TerritoryId;
    use crate::demography::{WaveEnd, WaveRecord};
    if world.demography.is_empty() {
        return (0, 0);
    }
    // Moved people (all, and outside the farm professions) by (old territory, new territory).
    let mut moved: BTreeMap<(TerritoryId, TerritoryId), (i64, i64)> = BTreeMap::new();
    let mut total: BTreeMap<TerritoryId, (i64, i64)> = BTreeMap::new();
    let olds: BTreeSet<TerritoryId> = old.iter().map(|&(_, t)| t).collect();
    let was: BTreeMap<usize, TerritoryId> = old.iter().copied().collect();
    for (i, r) in world.econ.regions.iter().enumerate() {
        let urban = (Fixed::from_int(r.population()) * r.urban_share()).to_int_floor();
        let t_old = was.get(&i).copied().unwrap_or_else(|| TerritoryId::of(r));
        if olds.contains(&t_old) {
            let e = total.entry(t_old).or_default();
            e.0 += r.population();
            e.1 += urban;
        }
        if let Some(&t) = was.get(&i) {
            let e = moved.entry((t, TerritoryId::of(r))).or_default();
            e.0 += r.population();
            e.1 += urban;
        }
    }
    let d = &mut world.demography;
    let mut out = Vec::with_capacity(d.diaspora.len());
    for x in d.diaspora.drain(..) {
        let mut rest = x.clone();
        for (&(t_old, t_new), &(people, urban)) in &moved {
            let (all_p, all_u) = total[&t_old];
            // Abroad by origin: the share of the origin's people; at home by destination: the
            // share of its towns, where the immigrants live.
            let (num, den) = if x.origin == t_old {
                (people, all_p)
            } else if x.dest == t_old {
                (urban, all_u)
            } else {
                continue;
            };
            if den <= 0 || num <= 0 {
                continue;
            }
            let part = (i128::from(x.people) * i128::from(num) / i128::from(den)) as i64;
            let arrived = (i128::from(x.arrived) * i128::from(num) / i128::from(den)) as i64;
            if part == 0 && arrived == 0 {
                continue;
            }
            rest.people -= part;
            rest.arrived -= arrived;
            let mut piece = x.clone();
            piece.people = part;
            piece.arrived = arrived;
            if x.origin == t_old {
                piece.origin = t_new;
            } else {
                piece.dest = t_new;
            }
            out.push(piece);
        }
        out.push(rest);
    }
    let home = out
        .iter()
        .filter(|x| x.origin == x.dest)
        .fold((0, 0), |(p, a), x| (p + x.people, a + x.arrived));
    out.retain(|x| x.origin != x.dest && (x.people != 0 || x.arrived != 0));
    out.sort_by_key(|x| (x.origin, x.dest, x.culture));
    let mut merged: Vec<crate::demography::Diaspora> = Vec::with_capacity(out.len());
    for x in out {
        match merged.last_mut() {
            Some(m) if (m.origin, m.dest, m.culture) == (x.origin, x.dest, x.culture) => {
                m.people += x.people;
                m.arrived += x.arrived;
            }
            _ => merged.push(x),
        }
    }
    d.diaspora = merged;
    let month = date.year * 12 + i32::from(date.month) - 1;
    let (closed, open): (Vec<_>, Vec<_>) = d
        .waves
        .drain(..)
        .partition(|w| olds.contains(&w.origin) || olds.contains(&w.dest));
    d.waves = open;
    d.waves_ended
        .extend(closed.into_iter().map(|wave| WaveRecord {
            wave,
            until: month,
            end: WaveEnd::Border,
        }));
    d.territories =
        crate::demography::territories(&world.econ, &rules.map, rules.communities.as_deref());
    home
}

/// Runs the debt and technique terms.
fn apply_terms(world: &mut GameWorld, terms: &Terms) {
    for d in &terms.debt {
        let (from, to, amount) = match d {
            DebtTerm::Market { from, to, amount } => (from, to, amount),
            DebtTerm::Agreement {
                debtor,
                creditor,
                treaty,
                terms,
            } => {
                if let (Some(a), Some(b)) = (tag_ix(world, debtor), tag_ix(world, creditor))
                    && a != b
                {
                    world.econ.ledger.create(crate::treaty::InterstateDebt::new(
                        a as u16,
                        b as u16,
                        treaty.clone(),
                        terms,
                    ));
                }
                continue;
            }
        };
        let (Some(a), Some(b)) = (tag_ix(world, from), tag_ix(world, to)) else {
            continue;
        };
        let (ma, mb) = two(&mut world.econ.markets, a, b);
        let (x, y) = (&mut ma.money, &mut mb.money);
        let total = x.debt_home + x.debt_bank + x.debt_abroad + x.debt_fx;
        if total <= Fixed::ZERO {
            continue;
        }
        let amount = (*amount).min(total);
        for (a, b) in [
            (&mut x.debt_home, &mut y.debt_home),
            (&mut x.debt_bank, &mut y.debt_bank),
            (&mut x.debt_abroad, &mut y.debt_abroad),
            (&mut x.debt_fx, &mut y.debt_fx),
        ] {
            let part = a.mul_div(amount, total);
            *a -= part;
            *b += part;
        }
    }
    for t in &terms.techniques {
        let (Some(a), Some(b)) = (tag_ix(world, &t.from), tag_ix(world, &t.to)) else {
            continue;
        };
        let r = &mut world.research;
        if r.is_empty() {
            continue;
        }
        let i = usize::from(t.tech);
        let ceiling = r.stage[a][i].min(t.stage);
        if r.stage[b][i] < ceiling {
            r.stage[b][i] = ceiling;
            r.progress[b][i] = Fixed::ZERO;
        }
        let lost = r.progress[a][i] * t.people;
        r.progress[a][i] -= lost;
    }
}

/// Runs a checked operation. Returns the countries born and ended.
fn execute(
    world: &mut GameWorld,
    rules: &BorderRules,
    op: &Operation,
    date: Date,
) -> (Option<u16>, Option<u16>, (i64, i64)) {
    use crate::demography::TerritoryId;
    if world.borders.subjects.is_empty() {
        let mut s = rules.subjects.clone();
        s.resize(world.countries.len(), None);
        world.borders.subjects = s;
    }
    let ix = |w: &GameWorld, t: &str| tag_ix(w, t).expect("checked");
    let mut born = None;
    let mut ended = None;
    let mut repatriated = (0, 0);
    match op {
        Operation::Transfer {
            from,
            to,
            states,
            colony,
            terms,
        } => {
            let (f, t) = (ix(world, from), ix(world, to));
            let rs = regions_of(rules, world, f, states).expect("checked");
            let old: Vec<_> = rs
                .iter()
                .map(|&r| (r, TerritoryId::of(&world.econ.regions[r])))
                .collect();
            let (w, p) = weights(world, f, &rs);
            {
                let (a, b) = two(&mut world.econ.markets, f, t);
                shift(&mut a.pool, &mut b.pool, w);
                shift(&mut a.money.foreign_assets, &mut b.money.foreign_assets, p);
            }
            move_regions(world, rules, f, t, &rs, *colony);
            if rs.iter().any(|&r| world.econ.regions[r].colony) {
                colonial_laws(world, rules, f, t);
            }
            apply_terms(world, terms);
            repatriated = demography(world, rules, &old, date);
        }
        Operation::Secede {
            from,
            states,
            tag,
            overlord,
            signatories,
            terms,
        } => {
            let f = ix(world, from);
            let rs = regions_of(rules, world, f, states).expect("checked");
            let old: Vec<_> = rs
                .iter()
                .map(|&r| (r, TerritoryId::of(&world.econ.regions[r])))
                .collect();
            let (w, p) = weights(world, f, &rs);
            // Debt goes with the metropole states only (decision 1): a colony is born without.
            let metro: Vec<usize> = rs
                .iter()
                .copied()
                .filter(|&r| !world.econ.regions[r].colony)
                .collect();
            let (w_metro, _) = weights(world, f, &metro);
            let c = ensure_country(world, rules, tag, f);
            {
                let (a, b) = two(&mut world.econ.markets, f, c);
                shift(&mut a.pool, &mut b.pool, w);
                shift(&mut a.money.foreign_assets, &mut b.money.foreign_assets, p);
                shift(&mut a.gdp_running, &mut b.gdp_running, w);
                shift(&mut a.gdp_last_year, &mut b.gdp_last_year, w);
                shift(&mut a.budget.revenue_avg, &mut b.budget.revenue_avg, w);
                if w_metro > Fixed::ZERO {
                    let (x, y) = (&mut a.money, &mut b.money);
                    shift(&mut x.debt_home, &mut y.debt_home, w_metro);
                    shift(&mut x.debt_bank, &mut y.debt_bank, w_metro);
                    shift(&mut x.debt_abroad, &mut y.debt_abroad, w_metro);
                    shift(&mut x.debt_fx, &mut y.debt_fx, w_metro);
                }
            }
            // The plants of the colony stay the old metropole's, less what local owners held
            // (C6a, C6c), from the metropole's colonial laws of the day before.
            let capitalists = |r: &crate::econ::Region| {
                let n: i64 = r.pops.iter().map(|p| p.size).sum();
                (r.pops[crate::econ::prof::CAPITALIST].size, n)
            };
            let home = world
                .econ
                .regions
                .iter()
                .filter(|r| usize::from(r.market) == f && !r.colony)
                .map(capitalists)
                .fold((0i64, 0i64), |a, b| (a.0 + b.0, a.1 + b.1));
            for &ri in &rs {
                let r = &world.econ.regions[ri];
                if !r.colony {
                    continue;
                }
                let private = crate::foreign_stakes::colonial_private_share(
                    world.econ.markets[f].laws.of(r),
                    crate::foreign_stakes::local_owners(capitalists(r), home),
                );
                let plants: Vec<(u16, i64)> = r.plants.iter().map(|p| (p.kind, p.levels)).collect();
                crate::foreign_stakes::colonial_stakes(
                    &mut world.foreign,
                    ri as u32,
                    f as u16,
                    date.year,
                    private,
                    &rules.colonial_kinds,
                    plants.into_iter(),
                );
            }
            move_regions(world, rules, f, c, &rs, Some(false));
            keep_plant_techniques(world, rules, f, c, &rs);
            world.borders.subjects[c] = overlord.as_ref().map(|o| Subject {
                overlord: ix(world, o) as u16,
                kind: SubjectKind::Protectorate,
                since: date,
            });
            // Recognised only by the signatories of its creation (owner's decision 4 of
            // `docs/treaties.pt-BR.md`).
            if let Some(policy) = rules.recognition {
                let signed: Vec<u16> = signatories
                    .iter()
                    .filter_map(|s| tag_ix(world, s).map(|i| i as u16))
                    .collect();
                let n = world.countries.len() as u16;
                world
                    .treaties
                    .recognition
                    .on_new_country(c as u16, n, &signed, policy);
            }
            apply_terms(world, terms);
            repatriated = demography(world, rules, &old, date);
            born = Some(c as u16);
        }
        Operation::Annex {
            from,
            into,
            colony,
            terms,
        } => {
            let (f, t) = (ix(world, from), ix(world, into));
            let rs: Vec<usize> = world
                .econ
                .regions
                .iter()
                .enumerate()
                .filter(|(_, r)| usize::from(r.market) == f)
                .map(|(i, _)| i)
                .collect();
            let old: Vec<_> = rs
                .iter()
                .map(|&r| (r, TerritoryId::of(&world.econ.regions[r])))
                .collect();
            {
                let (a, b) = two(&mut world.econ.markets, f, t);
                for (x, y) in [
                    (&mut a.pool, &mut b.pool),
                    (&mut a.treasury, &mut b.treasury),
                    (&mut a.gdp_running, &mut b.gdp_running),
                    (&mut a.gdp_last_year, &mut b.gdp_last_year),
                    (&mut a.budget.revenue_avg, &mut b.budget.revenue_avg),
                    (&mut a.money.debt_home, &mut b.money.debt_home),
                    (&mut a.money.debt_bank, &mut b.money.debt_bank),
                    (&mut a.money.debt_abroad, &mut b.money.debt_abroad),
                    (&mut a.money.debt_fx, &mut b.money.debt_fx),
                    (&mut a.money.foreign_assets, &mut b.money.foreign_assets),
                ] {
                    shift(x, y, Fixed::ONE);
                }
            }
            move_regions(world, rules, f, t, &rs, Some(*colony));
            if *colony {
                colonial_laws(world, rules, f, t);
            }
            // Subjects of the annexed country now answer to the annexer.
            for s in world.borders.subjects.iter_mut().flatten() {
                if usize::from(s.overlord) == f {
                    s.overlord = t as u16;
                }
            }
            world.borders.subjects[f] = None;
            // Debts between governments: the annexer owes and is owed what the annexed was; a
            // line between the two disappears from both sides (`treaty::Ledger::on_annex`).
            world.econ.ledger.on_annex(f as u16, t as u16);
            apply_terms(world, terms);
            repatriated = demography(world, rules, &old, date);
            world.treaties.recognition.on_annex(f as u16);
            ended = Some(f as u16);
        }
        Operation::Status { tag, overlord } => {
            let c = ix(world, tag);
            world.borders.subjects[c] = overlord.as_ref().map(|o| Subject {
                overlord: ix(world, o) as u16,
                kind: SubjectKind::Protectorate,
                since: date,
            });
        }
        Operation::Terms { terms } => apply_terms(world, terms),
        Operation::Dissolve { from, heirs, terms } => {
            let f = ix(world, from);
            let all: Vec<usize> = world
                .econ
                .regions
                .iter()
                .enumerate()
                .filter(|(_, r)| usize::from(r.market) == f)
                .map(|(i, _)| i)
                .collect();
            let old: Vec<_> = all
                .iter()
                .map(|&r| (r, TerritoryId::of(&world.econ.regions[r])))
                .collect();
            // The heir with the largest income continues the dissolved country in the ledger of
            // debts between governments: its lines pass whole (a line split among heirs would be
            // a new line, which the treaties write).
            let main_heir = heirs
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    let rs = regions_of(rules, world, f, &h.states).expect("checked");
                    (weights(world, f, &rs).0, std::cmp::Reverse(i))
                })
                .max()
                .map_or(0, |(_, std::cmp::Reverse(i))| i);
            let mut main_ix = None;
            // Each heir in turn takes its share of what is left: w over the country as it stands
            // then. The last takes all that remains, so nothing is lost to rounding.
            let n = heirs.len();
            for (i, h) in heirs.iter().enumerate() {
                let rs = regions_of(rules, world, f, &h.states).expect("checked");
                let (w, p) = if i + 1 == n {
                    (Fixed::ONE, Fixed::ONE)
                } else {
                    weights(world, f, &rs)
                };
                let c = ensure_country(world, rules, &h.tag, f);
                if i == main_heir {
                    main_ix = Some(c);
                }
                {
                    let (a, b) = two(&mut world.econ.markets, f, c);
                    for (x, y) in [
                        (&mut a.pool, &mut b.pool),
                        (&mut a.treasury, &mut b.treasury),
                        (&mut a.gdp_running, &mut b.gdp_running),
                        (&mut a.gdp_last_year, &mut b.gdp_last_year),
                        (&mut a.budget.revenue_avg, &mut b.budget.revenue_avg),
                        (&mut a.money.debt_home, &mut b.money.debt_home),
                        (&mut a.money.debt_bank, &mut b.money.debt_bank),
                        (&mut a.money.debt_abroad, &mut b.money.debt_abroad),
                        (&mut a.money.debt_fx, &mut b.money.debt_fx),
                    ] {
                        shift(x, y, w);
                    }
                    shift(&mut a.money.foreign_assets, &mut b.money.foreign_assets, p);
                }
                move_regions(world, rules, f, c, &rs, Some(false));
                keep_plant_techniques(world, rules, f, c, &rs);
                world.borders.subjects[c] = h.overlord.as_ref().map(|o| Subject {
                    overlord: ix(world, o) as u16,
                    kind: SubjectKind::Protectorate,
                    since: date,
                });
            }
            if let Some(m) = main_ix {
                world.econ.ledger.on_annex(f as u16, m as u16);
            }
            // Subjects of the dissolved country are free.
            for s in world.borders.subjects.iter_mut() {
                if s.as_ref().is_some_and(|s| usize::from(s.overlord) == f) {
                    *s = None;
                }
            }
            world.borders.subjects[f] = None;
            apply_terms(world, terms);
            repatriated = demography(world, rules, &old, date);
            world.treaties.recognition.on_annex(f as u16);
            ended = Some(f as u16);
        }
    }
    (born, ended, repatriated)
}

/// Checks and runs every pending operation, in order.
pub fn run_pending(world: &mut GameWorld, rules: &BorderRules, date: Date) {
    let pending = std::mem::take(&mut world.borders.pending);
    for mut op in pending {
        // A player freeing land names no tag: the catalogue's, or a generic one.
        if let Operation::Secede {
            from, states, tag, ..
        } = &mut op
            && tag.is_empty()
            && let Some(f) = tag_ix(world, from)
            && let Ok(rs) = regions_of(rules, world, f, states)
        {
            *tag = tag_for(world, rules, &rs);
        }
        match check(world, rules, &op) {
            Ok(()) => {
                let (born, ended, repatriated) = execute(world, rules, &op, date);
                world.borders.changes.push(ChangeRecord {
                    date,
                    op,
                    born,
                    ended,
                    repatriated,
                });
            }
            Err(reason) => world.borders.refused.push(Refusal { date, op, reason }),
        }
    }
}

/// The candidate whose core holds more than half of the people of `regions`, among those not
/// alive; else a generic tag `X00`..`X99` not in use. For a player freeing part of its land.
pub fn tag_for(world: &GameWorld, rules: &BorderRules, regions: &[usize]) -> Tag {
    let holds = held(world);
    let people: BTreeMap<u16, i64> = regions
        .iter()
        .map(|&r| {
            let r = &world.econ.regions[r];
            (r.state, r.population())
        })
        .collect();
    let total: i64 = people.values().sum();
    let key_of: BTreeMap<&str, u16> = rules
        .state_ids
        .iter()
        .map(|(k, &id)| (k.as_str(), id))
        .collect();
    for c in &rules.candidates.countries {
        if c.rename_of.is_some() || tag_ix(world, &c.tag).is_some_and(|i| holds[i] > 0) {
            continue;
        }
        let inside: i64 = c
            .states
            .iter()
            .filter_map(|k| key_of.get(k.as_str()))
            .filter_map(|id| people.get(id))
            .sum();
        if total > 0 && inside * 2 > total {
            return c.tag.clone();
        }
    }
    (0..100)
        .map(|i| format!("X{i:02}"))
        .find(|t| tag_ix(world, t).is_none_or(|i| holds[i] == 0))
        .unwrap_or_else(|| "X99".into())
}

/// The system that runs the queued operations at the end of the day.
pub struct Borders {
    pub rules: Arc<BorderRules>,
}

impl System<GameWorld> for Borders {
    type Changes = ();

    fn name(&self) -> &'static str {
        "borders"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }

    fn access(&self) -> Access {
        Access::new()
            .write(BORDERS)
            .write(crate::econ::week::ECON)
            .write(crate::research_system::RESEARCH)
            .write(crate::polity::POLITY)
            .write(crate::demography::DEMOGRAPHY)
            .write(crate::foreign_stakes::FOREIGN)
            .write(ResourceId(0))
    }

    fn compute(&self, _ctx: &TickCtx, _world: &GameWorld) {}

    fn apply(&self, ctx: &TickCtx, _changes: (), world: &mut GameWorld) {
        if !world.borders.pending.is_empty() {
            run_pending(world, &self.rules, ctx.date);
        }
    }
}

/// Tables a world needs once countries were born: for tests, every per-country table has the same
/// length as the country table.
pub fn tables_agree(world: &GameWorld) -> bool {
    let n = world.countries.len();
    let r = &world.research;
    let p = &world.politics;
    world.econ.markets.len() == n
        && (r.is_empty() || (r.stage.len() == n && r.progress.len() == n && r.focus.len() == n))
        && (p.is_empty()
            || (p.laws.len() == n
                && p.clout.len() == n
                && p.movements.len() == n
                && p.legitimacy.len() == n))
        && (world.borders.subjects.is_empty() || world.borders.subjects.len() == n)
}

/// Who holds every map state, for the interface: read again when a border operation ran.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnersView {
    /// Operations run so far (`BordersState::changes`): the view's version.
    pub changes: usize,
    /// Country tags in the simulation's order.
    pub tags: Vec<String>,
    /// (owner index or `u16::MAX`, colony) by map state, index = state id − 1.
    pub owners: Vec<(u16, bool)>,
    /// Overlord of every country, by index.
    pub overlords: Vec<Option<u16>>,
    /// People of every map state, index = state id − 1 (to weigh what a new country inherits).
    pub people: Vec<i64>,
}

impl OwnersView {
    pub fn read(world: &GameWorld, rules: &BorderRules) -> Self {
        let n = rules
            .map
            .states
            .iter()
            .map(|s| usize::from(s.id))
            .max()
            .unwrap_or(0);
        let mut owners = vec![(u16::MAX, false); n];
        let mut people = vec![0i64; n];
        for r in &world.econ.regions {
            if let Some(i) = usize::from(r.state).checked_sub(1).filter(|&i| i < n) {
                owners[i] = (r.market, r.colony);
                people[i] = r.population();
            }
        }
        Self {
            changes: world.borders.changes.len(),
            tags: world.countries.iter().map(|(_, c)| c.tag.clone()).collect(),
            owners,
            overlords: (0..world.countries.len())
                .map(|c| rules.overlord_of(world, c).map(|o| o as u16))
                .collect(),
            people,
        }
    }
}

/// What freeing `states` of country `from` would move (`docs/borders.pt-BR.md`, the decision
/// "Libertar território"): read by the interface before the player confirms.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Preview {
    pub people: i64,
    /// Plant levels in the states.
    pub levels: i64,
    /// Share of the country's investment fund that goes with them (**w**), and the amount.
    pub fund_share: Fixed,
    pub fund: Fixed,
    /// Public debt that leaves with the metropole states (a colony is born without debt).
    pub debt: Fixed,
    /// The country's people and plant levels now, for the shares.
    pub people_total: i64,
    pub levels_total: i64,
}

/// [`Preview`] of freeing `states` (map state keys) of `from`; `None` when the operation would
/// be refused (a state not held, all of them).
pub fn preview(
    world: &GameWorld,
    rules: &BorderRules,
    from: usize,
    states: &[String],
) -> Option<Preview> {
    let rs = regions_of(rules, world, from, states).ok()?;
    if rs.len() == held(world)[from] {
        return None;
    }
    let (w, _) = weights(world, from, &rs);
    let metro: Vec<usize> = rs
        .iter()
        .copied()
        .filter(|&r| !world.econ.regions[r].colony)
        .collect();
    let (w_metro, _) = weights(world, from, &metro);
    let m = &world.econ.markets[from];
    let y = &m.money;
    let mut p = Preview {
        fund_share: w,
        fund: m.pool * w,
        debt: (y.debt_home + y.debt_bank + y.debt_abroad + y.debt_fx) * w_metro,
        ..Preview::default()
    };
    for (i, r) in world.econ.regions.iter().enumerate() {
        if usize::from(r.market) != from {
            continue;
        }
        let levels: i64 = r.plants.iter().map(|p| p.levels).sum();
        p.people_total += r.population();
        p.levels_total += levels;
        if rs.binary_search(&i).is_ok() {
            p.people += r.population();
            p.levels += levels;
        }
    }
    Some(p)
}

/// One way the player's country can free land: a country of the catalogue whose core it holds
/// (in part), or one state of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct LiberationOption {
    /// The catalogue's tag; `None` for a single state (the tag is chosen when it is freed).
    pub tag: Option<String>,
    pub states: Vec<String>,
    pub preview: Preview,
}

/// Every way `from` can free land, catalogue countries first (by people), then its states (by
/// key).
pub fn liberation_options(
    world: &GameWorld,
    rules: &BorderRules,
    from: usize,
) -> Vec<LiberationOption> {
    let holds = held(world);
    let key_of: BTreeMap<u16, &str> = rules
        .state_ids
        .iter()
        .map(|(k, &id)| (id, k.as_str()))
        .collect();
    let mine: BTreeSet<&str> = world
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == from)
        .filter_map(|r| key_of.get(&r.state).copied())
        .collect();
    let mut out: Vec<LiberationOption> = rules
        .candidates
        .countries
        .iter()
        .filter(|c| c.rename_of.is_none())
        .filter(|c| tag_ix(world, &c.tag).is_none_or(|i| holds[i] == 0))
        .filter_map(|c| {
            let states: Vec<String> = c
                .states
                .iter()
                .filter(|s| mine.contains(s.as_str()))
                .cloned()
                .collect();
            if states.is_empty() {
                return None;
            }
            let preview = preview(world, rules, from, &states)?;
            Some(LiberationOption {
                tag: Some(c.tag.clone()),
                states,
                preview,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.preview
            .people
            .cmp(&a.preview.people)
            .then_with(|| a.tag.cmp(&b.tag))
    });
    for s in &mine {
        let states = vec![(*s).to_owned()];
        if let Some(preview) = preview(world, rules, from, &states) {
            out.push(LiberationOption {
                tag: None,
                states,
                preview,
            });
        }
    }
    out
}
