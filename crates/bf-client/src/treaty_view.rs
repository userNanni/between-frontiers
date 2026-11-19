//! The negotiation table (`docs/treaties.pt-BR.md`, section 13) for the journal: the proposals
//! that involve the player, open and closed, and the weighing of a draft on the table, by the
//! same reading the simulation's answer uses (`treaty_table::account` and `respond`), the
//! cooldown of a refused proposal (`blocked`), the clauses that sleep (`dormant`), the
//! parameters outside the catalogue's limits (`treaty::limit_violations`) and the clauses that
//! recognition forbids (`treaty::recognition_conflicts`). Read off the tick, in the simulation's
//! thread, when the table or the draft changes.

use std::collections::BTreeMap;

use bf_sim::game::GameWorld;
use bf_sim::treaty::{ClauseCatalogue, Front, Proposal, Valuation, ValueTerm};
use bf_sim::treaty_table::{self, Outcome, ProposalAnswer};
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// One party's account of a proposal, in real dollars (the reference units of the treaties).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Account {
    pub total: f64,
    /// Each term that is not zero, in the order of `ValueTerm::ALL`.
    pub terms: Vec<(ValueTerm, f64)>,
    /// The value of each clause for the party, by clause index.
    pub clauses: Vec<f64>,
}

impl Account {
    fn of(v: &Valuation, clauses: usize) -> Self {
        let f = |x: Fixed| x.to_f64_lossy();
        let mut by_clause = vec![0.0; clauses];
        for l in &v.lines {
            if let Some(c) = by_clause.get_mut(l.clause) {
                *c += f(l.value);
            }
        }
        Account {
            total: f(v.total),
            terms: ValueTerm::ALL
                .iter()
                .map(|t| (*t, f(v.term(*t))))
                .filter(|(_, x)| *x != 0.0)
                .collect(),
            clauses: by_clause,
        }
    }
}

/// What the other side would answer, by the account it will make.
#[derive(Clone, Debug, PartialEq)]
pub enum Likely {
    Accept,
    /// It would counter with this proposal (in the orientation it was sent).
    Counter(Proposal),
    /// It would refuse, for the term that weighed most.
    Refuse(Option<ValueTerm>),
}

/// What the table says of one clause of a draft.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClauseNote {
    /// The front it waits for: signed, shown, without effect.
    pub dormant: Option<Front>,
    /// Its parameters outside the catalogue's limits.
    pub limits: Vec<&'static str>,
    /// Recognition forbids it: the giver and the receiver do not recognise each other.
    pub unrecognised: bool,
}

/// A proposal weighed for the player: the draft on the table, or one the player received.
#[derive(Clone, Debug, PartialEq)]
pub struct Weighed {
    /// The other party.
    pub other: String,
    pub proposal: Proposal,
    pub me: Account,
    pub other_account: Account,
    /// The other side's likely answer (for a draft the player sends).
    pub likely: Option<Likely>,
    /// An identical proposal was refused or lapsed: when, and when it may come back.
    pub blocked: Option<(Date, Date)>,
    pub notes: Vec<ClauseNote>,
}

impl Weighed {
    /// Why the draft cannot go: a clause outside its limits or forbidden by recognition, or the
    /// cooldown of an identical refusal (the key in `treaties.ftl`).
    pub fn refusal(&self) -> Option<&'static str> {
        if self.proposal.clauses.is_empty() {
            Some("table-refusal-empty")
        } else if self.notes.iter().any(|n| n.unrecognised) {
            Some("table-refusal-recognition")
        } else if self.notes.iter().any(|n| !n.limits.is_empty()) {
            Some("table-refusal-limits")
        } else if self.blocked.is_some() {
            Some("table-refusal-cooldown")
        } else {
            None
        }
    }
}

fn index_of(world: &GameWorld, tag: &str) -> Option<u16> {
    world
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .map(|i| i as u16)
}

/// Weighs `proposal` between the player `me` and `other`. `sending`: a draft the player would
/// send (side A is the player), with the other side's likely answer and the cooldown.
pub fn weigh(
    world: &GameWorld,
    cat: &ClauseCatalogue,
    me: &str,
    other: &str,
    proposal: &Proposal,
    sending: bool,
) -> Weighed {
    let n = proposal.clauses.len();
    let treaty_table::Inputs { gdp, live, .. } = treaty_table::Inputs::of(world);
    let mut notes: Vec<ClauseNote> = proposal
        .clauses
        .iter()
        .map(|c| ClauseNote {
            dormant: cat.dormant(c.kind.key(), &live),
            ..ClauseNote::default()
        })
        .collect();
    for (ci, param) in bf_sim::treaty::limit_violations(cat, proposal, &gdp) {
        if let Some(note) = notes.get_mut(ci) {
            note.limits.push(param);
        }
    }
    for (ci, _, _) in
        bf_sim::treaty::recognition_conflicts(proposal, &world.treaties.recognition, cat, |t| {
            index_of(world, t)
        })
    {
        if let Some(note) = notes.get_mut(ci) {
            note.unrecognised = true;
        }
    }
    let likely = (sending && n > 0).then(|| {
        match treaty_table::respond(world, cat, proposal, other, true) {
            (ProposalAnswer::Accept, _) => Likely::Accept,
            (ProposalAnswer::Counter(q), _) => Likely::Counter(q),
            (ProposalAnswer::Refuse, why) => Likely::Refuse(why),
        }
    });
    Weighed {
        other: other.to_owned(),
        me: Account::of(&treaty_table::account(world, cat, proposal, me), n),
        other_account: Account::of(&treaty_table::account(world, cat, proposal, other), n),
        likely,
        blocked: if sending {
            treaty_table::blocked(world, me, other, proposal)
        } else {
            None
        },
        notes,
        proposal: proposal.clone(),
    }
}

/// A proposal waiting for an answer, sent by the player or to it.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenLine {
    pub id: u32,
    pub from: String,
    pub to: String,
    pub sent: Date,
    pub deadline: Date,
    /// 1 for a proposal, 2 and 3 for counterproposals.
    pub round: u8,
    /// Weighed for the player (no likely answer: the player answers, or the AI does next week).
    pub weighed: Weighed,
}

/// A proposal of the player's closed in the last months.
#[derive(Clone, Debug, PartialEq)]
pub struct ClosedLine {
    pub id: u32,
    pub from: String,
    pub to: String,
    pub date: Date,
    pub outcome: Outcome,
}

/// The table as the journal shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct TableView {
    /// The country the table was read for (the one played now).
    pub me: String,
    /// Proposals to the player: it accepts, refuses or counters before the deadline.
    pub incoming: Vec<OpenLine>,
    /// Proposals of the player waiting for the other side (it answers on Monday).
    pub outgoing: Vec<OpenLine>,
    /// The player's proposals closed in the last months, the newest first.
    pub closed: Vec<ClosedLine>,
    /// Each country's GDP of the last year, real dollars: the base of the parameters given as a
    /// share of GDP.
    pub gdp: BTreeMap<String, f64>,
    /// Weeks the other side has to answer.
    pub deadline_weeks: u16,
    pub signature: Signature,
}

/// What the table looked like when read: the week, the ids handed out, the open and the closed.
pub type Signature = (i64, u32, usize, usize);

pub fn signature(world: &GameWorld, date: Date) -> Signature {
    let t = &world.treaties.table;
    (
        date.to_days().div_euclid(7),
        t.next_id,
        t.open.len(),
        t.closed.len(),
    )
}

/// The table of the player's country; `None` without a player.
pub fn read(world: &GameWorld, me: &str, cat: &ClauseCatalogue, date: Date) -> Option<TableView> {
    let t = &world.treaties.table;
    let line = |p: &treaty_table::Pending, other: &str| OpenLine {
        id: p.id,
        from: p.from.clone(),
        to: p.to.clone(),
        sent: p.sent,
        deadline: p.deadline,
        round: p.round,
        weighed: weigh(world, cat, me, other, &p.proposal, false),
    };
    let incoming = t
        .open
        .iter()
        .filter(|p| p.to == me)
        .map(|p| line(p, &p.from))
        .collect();
    let outgoing = t
        .open
        .iter()
        .filter(|p| p.from == me)
        .map(|p| line(p, &p.to))
        .collect();
    let closed = t
        .closed
        .iter()
        .rev()
        .filter(|c| c.from == me || c.to == me)
        .map(|c| ClosedLine {
            id: c.id,
            from: c.from.clone(),
            to: c.to.clone(),
            date: c.date,
            outcome: c.outcome.clone(),
        })
        .collect();
    Some(TableView {
        me: me.to_owned(),
        incoming,
        outgoing,
        closed,
        gdp: treaty_table::Inputs::of(world)
            .gdp
            .into_iter()
            .map(|(t, g)| (t, g.to_f64_lossy()))
            .collect(),
        deadline_weeks: treaty_table::DEADLINE_WEEKS,
        signature: signature(world, date),
    })
}
