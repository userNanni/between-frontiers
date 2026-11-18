//! The negotiation table (`docs/treaties.pt-BR.md`, section 13, "O fluxo, passo a passo"): a
//! proposal sent by a country waits in `TreatyState::proposals` until the other side answers,
//! within [`DEADLINE_WEEKS`]. A country no player governs answers in the first week by the
//! account of [`crate::treaty::evaluate`]: it accepts a positive balance, counters by touching
//! as little as it can, or refuses with the term that weighed most. An accepted proposal is
//! carried out by the bridge (`treaty_exec::settle` and `queue`). Behind
//! `Switches::negotiation`: without it the weekly step does not run.
//!
//! Not yet: ratification inside the country (section 8) passes always; the parameter limits of
//! the catalogue (`clauses.ron`) are the bounds of the counterproposal once they exist.

use std::collections::BTreeMap;

use laplace_schelling::Date;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Weekday;
use laplace_schelling::schedule::{Access, Cadence, System, TickCtx};
use serde::{Deserialize, Serialize};

use crate::game::GameWorld;
use crate::scenario::Tag;
use crate::treaty::{
    ClauseCatalogue, ClauseKind, EvalInputs, Front, Proposal, Valuation, ValueTerm, evaluate,
};

/// Weeks the other side has to answer (design choice, as the demands of independence).
pub const DEADLINE_WEEKS: u16 = 4;
/// Counter rounds per proposal (section 8).
pub const MAX_ROUNDS: u8 = 3;
/// Changes a counterproposal makes at most.
pub const MAX_CHANGES: usize = 3;
/// Months an identical refused proposal stays refused (section 8, design choice).
pub const COOLDOWN_MONTHS: i32 = 6;
/// DESIGN CHOICE: the yearly rate the table discounts promised flows at.
pub fn discount() -> Fixed {
    Fixed::from_ratio(5, 100)
}

/// One proposal on the table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    pub id: u32,
    /// Who sent it (side A of the proposal) and who answers (side B).
    pub from: Tag,
    pub to: Tag,
    pub proposal: Proposal,
    pub sent: Date,
    pub deadline: Date,
    /// 1 for a proposal, 2 and 3 for counterproposals.
    pub round: u8,
}

/// How a proposal ended (kept for the cooldown and the record).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closed {
    pub id: u32,
    pub from: Tag,
    pub to: Tag,
    pub proposal: Proposal,
    pub date: Date,
    pub outcome: Outcome,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Signed: the agreement's id.
    Signed(u32),
    /// Refused, with the term that weighed most (`None`: refused by a player).
    Refused(Option<ValueTerm>),
    /// Answered by counterproposal `id`.
    Countered(u32),
    /// The deadline passed without an answer.
    Lapsed,
    /// The bridge could not carry it out (the reason).
    Failed(String),
}

/// The answer of a side to a proposal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalAnswer {
    Accept,
    Refuse,
    Counter(Proposal),
}

/// The table's state: in `TreatyState::table`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub open: Vec<Pending>,
    /// Closed in the last [`COOLDOWN_MONTHS`], oldest first.
    pub closed: Vec<Closed>,
    pub next_id: u32,
    /// Answers of players waiting for the weekly step: (proposal id, answering tag, answer).
    pub answers: Vec<(u32, Tag, ProposalAnswer)>,
    /// Denunciations waiting for the weekly step: (agreement, tag).
    pub denounce: Vec<(u32, Tag)>,
}

impl Table {
    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
            && self.closed.is_empty()
            && self.next_id == 0
            && self.answers.is_empty()
            && self.denounce.is_empty()
    }

    /// Puts a proposal on the table; `None` when an identical one was refused in the cooldown.
    pub fn propose(&mut self, from: &str, to: &str, proposal: Proposal, date: Date) -> Option<u32> {
        if self.refused_recently(from, to, &proposal) {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.open.push(Pending {
            id,
            from: from.to_owned(),
            to: to.to_owned(),
            proposal,
            sent: date,
            deadline: Date::from_days(date.to_days() + i64::from(DEADLINE_WEEKS) * 7),
            round: 1,
        });
        Some(id)
    }

    fn refused_recently(&self, from: &str, to: &str, p: &Proposal) -> bool {
        self.closed.iter().any(|c| {
            c.from == from
                && c.to == to
                && c.proposal == *p
                && matches!(c.outcome, Outcome::Refused(_) | Outcome::Lapsed)
        })
    }
}

/// What the account reads of the world for `evaluate`.
pub struct Inputs {
    pub gdp: BTreeMap<Tag, Fixed>,
    pub live: Vec<Front>,
}

impl Inputs {
    pub fn of(world: &GameWorld) -> Self {
        let gdp = world
            .countries
            .iter()
            .zip(&world.econ.markets)
            .map(|((_, c), m)| (c.tag.clone(), m.gdp_last_year))
            .collect();
        Inputs {
            gdp,
            live: vec![
                Front::Borders,
                Front::Ledger,
                Front::Recognition,
                Front::Politics,
            ],
        }
    }
}

/// The account of `party` for `p` in `world`.
pub fn account(world: &GameWorld, cat: &ClauseCatalogue, p: &Proposal, party: &str) -> Valuation {
    let i = Inputs::of(world);
    evaluate(
        cat,
        &EvalInputs {
            discount: discount(),
            gdp: &i.gdp,
            reputation: &world.treaties.reputation,
            live: &i.live,
        },
        p,
        party,
    )
}

/// Value of each clause for `party`, by clause index.
fn by_clause(v: &Valuation, n: usize) -> Vec<Fixed> {
    let mut out = vec![Fixed::ZERO; n];
    for l in &v.lines {
        if l.clause < n {
            out[l.clause] += l.value;
        }
    }
    out
}

/// The term that weighed most against `party`.
fn worst_term(v: &Valuation) -> Option<ValueTerm> {
    ValueTerm::ALL
        .iter()
        .copied()
        .filter(|t| v.term(*t) < Fixed::ZERO)
        .min_by_key(|t| v.term(*t))
}

/// A numeric amount of the clause the counterproposal can lower, if any.
fn amount(kind: &mut ClauseKind) -> Option<&mut Fixed> {
    match kind {
        ClauseKind::Payment { amount } => Some(amount),
        ClauseKind::Tribute { yearly } => Some(yearly),
        ClauseKind::Debt { terms: Some(t) } => Some(&mut t.principal),
        _ => None,
    }
}

/// How `party` answers `p`: accept when its balance is positive; else the smallest change
/// that closes it (lower the amount of its worst clause, or drop the clause; at most
/// [`MAX_CHANGES`], in the order of the worst value first, ties by clause order); else refuse.
/// Deterministic: the same proposal in the same world gets the same answer.
pub fn respond(
    world: &GameWorld,
    cat: &ClauseCatalogue,
    p: &Proposal,
    party: &str,
    may_counter: bool,
) -> (ProposalAnswer, Option<ValueTerm>) {
    let v = account(world, cat, p, party);
    if v.total > Fixed::ZERO {
        return (ProposalAnswer::Accept, None);
    }
    let refuse = (ProposalAnswer::Refuse, worst_term(&v));
    if !may_counter {
        return refuse;
    }
    let mut q = p.clone();
    for _ in 0..MAX_CHANGES {
        let v = account(world, cat, &q, party);
        if v.total > Fixed::ZERO {
            break;
        }
        let values = by_clause(&v, q.clauses.len());
        let Some((worst, &value)) = values
            .iter()
            .enumerate()
            .filter(|(_, x)| **x < Fixed::ZERO)
            .min_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(&b.0)))
        else {
            break;
        };
        // Lower the amount by the share that closes the balance; drop it if that is all of it.
        let deficit = -v.total;
        let lowered = amount(&mut q.clauses[worst].kind).map(|a| {
            let cut = if -value > Fixed::ZERO {
                (*a * deficit / -value).min(*a)
            } else {
                *a
            };
            // A hair more than the deficit, so the balance turns positive.
            *a = (*a - cut - Fixed::from_raw(1)).max(Fixed::ZERO);
            *a > Fixed::ZERO
        });
        if lowered != Some(true) {
            q.clauses.remove(worst);
        }
        if q.clauses.is_empty() {
            return refuse;
        }
    }
    if account(world, cat, &q, party).total > Fixed::ZERO && q != *p {
        (ProposalAnswer::Counter(q), None)
    } else {
        refuse
    }
}

/// Whether `tag` is governed by a player (`GameWorld::player`).
fn is_player(world: &GameWorld, tag: &str) -> bool {
    world
        .player
        .and_then(|id| world.countries.get(id))
        .is_some_and(|c| c.tag == tag)
}

/// One week of the table (the apply half): answers, deadlines, signatures, denunciations.
pub fn week(world: &mut GameWorld, cat: &ClauseCatalogue, date: Date) {
    // Players' answers first, in the order given.
    let answers = std::mem::take(&mut world.treaties.table.answers);
    for (id, tag, answer) in answers {
        if let Some(i) = world
            .treaties
            .table
            .open
            .iter()
            .position(|p| p.id == id && p.to == tag)
        {
            let p = world.treaties.table.open.remove(i);
            settle_answer(world, cat, p, answer, None, date);
        }
    }
    // The AI answers what is addressed to it; deadlines lapse.
    let open = std::mem::take(&mut world.treaties.table.open);
    let mut keep = Vec::new();
    for p in open {
        if !is_player(world, &p.to) {
            let (answer, why) = respond(world, cat, &p.proposal, &p.to, p.round < MAX_ROUNDS);
            settle_answer(world, cat, p, answer, why, date);
        } else if date >= p.deadline {
            close(world, &p, Outcome::Lapsed, date);
        } else {
            keep.push(p);
        }
    }
    let mut open = keep;
    open.append(&mut world.treaties.table.open);
    world.treaties.table.open = open;
    // Denunciations.
    let denounce = std::mem::take(&mut world.treaties.table.denounce);
    for (agreement, tag) in denounce {
        denounce_one(world, agreement, &tag, date);
    }
    // The cooldown forgets what is older than its months (30 days to the month).
    let limit = Date::from_days(date.to_days() - i64::from(COOLDOWN_MONTHS) * 30);
    world.treaties.table.closed.retain(|c| c.date >= limit);
}

fn close(world: &mut GameWorld, p: &Pending, outcome: Outcome, date: Date) {
    world.treaties.table.closed.push(Closed {
        id: p.id,
        from: p.from.clone(),
        to: p.to.clone(),
        proposal: p.proposal.clone(),
        date,
        outcome,
    });
}

fn settle_answer(
    world: &mut GameWorld,
    cat: &ClauseCatalogue,
    p: Pending,
    answer: ProposalAnswer,
    why: Option<ValueTerm>,
    date: Date,
) {
    match answer {
        ProposalAnswer::Accept => {
            let key = format!("agreement-{}", p.id);
            match crate::treaty_exec::settle(cat, &p.proposal, &key) {
                Ok(s) => {
                    let id = crate::treaty_exec::queue(world, &s, date.year);
                    close(world, &p, Outcome::Signed(id.unwrap_or(u32::MAX)), date);
                }
                Err(e) => close(world, &p, Outcome::Failed(e), date),
            }
        }
        ProposalAnswer::Refuse => close(world, &p, Outcome::Refused(why), date),
        ProposalAnswer::Counter(mut q) if p.round < MAX_ROUNDS => {
            // The counterproposal goes back with the sides swapped: `to` now proposes.
            std::mem::swap(&mut q.side_a, &mut q.side_b);
            for c in &mut q.clauses {
                swap_side(&mut c.giver);
                swap_side(&mut c.receiver);
            }
            let id = world.treaties.table.next_id;
            world.treaties.table.next_id += 1;
            close(world, &p, Outcome::Countered(id), date);
            world.treaties.table.open.push(Pending {
                id,
                from: p.to.clone(),
                to: p.from.clone(),
                proposal: q,
                sent: date,
                deadline: Date::from_days(date.to_days() + i64::from(DEADLINE_WEEKS) * 7),
                round: p.round + 1,
            });
        }
        ProposalAnswer::Counter(_) => close(world, &p, Outcome::Refused(why), date),
    }
}

fn swap_side(s: &mut crate::treaty::Side) {
    use crate::treaty::Side;
    let swapped = match &*s {
        Side::Senior => Side::Junior,
        Side::Junior => Side::Senior,
        other => other.clone(),
    };
    *s = swapped;
}

/// `tag` denounces agreement `agreement` (section 13, "Denunciar"): with notice, the agreement
/// ends when the notice runs out, without a breach; without notice it ends now, a breach of
/// every clause `tag` gives (valued at zero until the table prices each clause for the harmed).
fn denounce_one(world: &mut GameWorld, agreement: u32, tag: &str, date: Date) {
    let Some(a) = world.treaties.in_force(agreement) else {
        return;
    };
    if !a.side_a.iter().chain(&a.side_b).any(|t| t == tag) {
        return;
    }
    let (signed, notice) = (a.signed, a.notice_years);
    if notice > 0 {
        let end = (date.year - signed + i32::from(notice)).clamp(0, i32::from(u16::MAX)) as u16;
        if let Some(a) = world
            .treaties
            .agreements
            .iter_mut()
            .find(|a| a.id == agreement)
        {
            a.term_years = Some(a.term_years.map_or(end, |t| t.min(end)));
        }
        return;
    }
    let given: Vec<usize> = a
        .kept
        .iter()
        .enumerate()
        .filter(|(_, c)| a.side(&c.giver).contains(&tag))
        .map(|(i, _)| i)
        .collect();
    for i in given {
        world.treaties.record_breach(
            agreement,
            i,
            tag,
            Fixed::ZERO,
            date,
            crate::treaty::BreachCause::Denounced,
        );
    }
    world.treaties.end(agreement, date.year);
}

/// The weekly step of the table, on Mondays.
pub struct TableWeek {
    pub cat: std::sync::Arc<ClauseCatalogue>,
}

impl System<GameWorld> for TableWeek {
    type Changes = ();

    fn name(&self) -> &'static str {
        "treaty-table"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Weekly(Weekday::Monday)
    }

    fn access(&self) -> Access {
        Access::new()
            .write(crate::treaty_system::TREATIES)
            .write(crate::econ::week::ECON)
            .write(crate::borders::op::BORDERS)
    }

    fn compute(&self, _ctx: &TickCtx, _world: &GameWorld) {}

    fn apply(&self, ctx: &TickCtx, _changes: (), world: &mut GameWorld) {
        let t = &world.treaties.table;
        if t.open.is_empty() && t.answers.is_empty() && t.denounce.is_empty() {
            return;
        }
        week(world, &self.cat, ctx.date);
    }
}
