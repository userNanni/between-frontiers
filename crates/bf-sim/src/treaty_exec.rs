//! From clauses to effects (phase 4): the bridge between a signed treaty and the operations that
//! carry it out. Whoever signs (the negotiation of the treaties, the peace of the war) calls
//! [`settle`] to translate the clauses and [`queue`] to put them in motion. Design:
//! `docs/treaties.pt-BR.md`, sections 4 and 11.
//!
//! - Territory goes through the border operations (`borders::op`), queued for the `borders`
//!   system at the end of the day, in the treaty's order: cession → `Transfer`, a new country →
//!   `Secede` (independent, or a subject of the receiving side when the clause names a bundle),
//!   union → `Annex`.
//! - Debt between governments goes to the ledger through a `DebtTerm::Agreement`, in one
//!   `Operation::Terms` queued after the territory, so that a country born by the same treaty can
//!   owe or be owed (the independence that is born owing).
//! - A payment moves treasuries and a revision changes the creditor's lines at once.
//! - Every other clause is kept: it holds while the agreement is in force.

use laplace_schelling::Fixed;

use crate::borders::op::{DebtTerm, Operation, Terms};
use crate::game::GameWorld;
use crate::scenario::Tag;
use crate::treaty::{
    Agreement, AgreementStatus, Clause, ClauseCatalogue, ClauseKind, DebtTerms, Proposal, Side,
};

/// What a signed treaty does, before it is put in motion.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settlement {
    /// Border operations, in order; the last carries the treaty's debts.
    pub ops: Vec<Operation>,
    /// (from, to, amount): payments between treasuries, reference units.
    pub payments: Vec<(Tag, Tag, Fixed)>,
    /// (creditor, treaty key, terms): revisions of the creditor's lines of that treaty.
    pub revisions: Vec<(Tag, String, DebtTerms)>,
    /// The clauses that hold while the agreement is in force.
    pub kept: Vec<Clause>,
    /// The record of the agreement (`TreatyState::agreements`), written by [`queue`].
    pub agreement: Option<Agreement>,
}

/// The parties a side names. A side may name a country the same treaty creates
/// (`ReleaseCountry`): it can owe, be owed, receive or give from the day it is born.
fn side<'a>(s: &'a Side, p: &'a Proposal) -> Vec<&'a str> {
    match s {
        Side::Senior => p.side_a.iter().map(String::as_str).collect(),
        Side::Junior => p.side_b.iter().map(String::as_str).collect(),
        Side::All => p
            .side_a
            .iter()
            .chain(&p.side_b)
            .map(String::as_str)
            .collect(),
        Side::Parties(t) => t.iter().map(String::as_str).collect(),
    }
}

fn one<'a>(list: &[&'a str], what: &str, clause: &str) -> Result<&'a str, String> {
    match list {
        [x] => Ok(x),
        _ => Err(format!("{clause}: the {what} must be one country")),
    }
}

/// Translates the clauses of a signed treaty. `key` names the treaty in the ledger and in the
/// record. Refuses what cannot be carried out: a cession or union with more than one giver or
/// receiver, a debt with no principal or no term.
pub fn settle(cat: &ClauseCatalogue, proposal: &Proposal, key: &str) -> Result<Settlement, String> {
    let mut out = Settlement::default();
    let mut debts: Vec<DebtTerm> = Vec::new();
    // The countries the treaty itself creates sign it, but are not signatories of their own
    // creation: they do not exist yet when the operation runs.
    let created: Vec<&Tag> = proposal
        .clauses
        .iter()
        .filter_map(|c| match &c.kind {
            ClauseKind::ReleaseCountry { tag, .. } => Some(tag),
            _ => None,
        })
        .collect();
    let signatories: Vec<Tag> = proposal
        .side_a
        .iter()
        .chain(&proposal.side_b)
        .filter(|t| !created.contains(t))
        .cloned()
        .collect();
    for cl in &proposal.clauses {
        let k = cl.kind.key();
        let givers = side(&cl.giver, proposal);
        let receivers = side(&cl.receiver, proposal);
        let pairs: Vec<(&str, &str)> = givers
            .iter()
            .flat_map(|g| receivers.iter().map(move |r| (*g, *r)))
            .filter(|(g, r)| g != r)
            .collect();
        let n = pairs.len().max(1) as i64;
        match &cl.kind {
            ClauseKind::CedeStates { states } => {
                out.ops.push(Operation::Transfer {
                    from: one(&givers, "giver", k)?.into(),
                    to: one(&receivers, "receiver", k)?.into(),
                    states: states.clone(),
                    colony: None,
                    terms: Terms::default(),
                });
            }
            ClauseKind::ReleaseCountry {
                states,
                tag,
                bundle,
            } => {
                let from = one(&givers, "giver", k)?;
                if let Some(b) = bundle
                    && cat.bundle(b).is_none()
                {
                    return Err(format!("{k}: unknown bundle {b}"));
                }
                let overlord = match bundle {
                    Some(_) => Some(one(&receivers, "overlord", k)?.to_string()),
                    None => None,
                };
                out.ops.push(Operation::Secede {
                    from: from.into(),
                    states: states.clone(),
                    tag: tag.clone(),
                    overlord,
                    signatories: signatories.clone(),
                    terms: Terms::default(),
                });
                if bundle.is_some() {
                    out.kept.push(cl.clone());
                }
            }
            ClauseKind::Union => {
                out.ops.push(Operation::Annex {
                    from: one(&givers, "giver", k)?.into(),
                    into: one(&receivers, "receiver", k)?.into(),
                    colony: false,
                    terms: Terms::default(),
                });
            }
            ClauseKind::Debt { terms: Some(t) } => {
                if t.principal <= Fixed::ZERO || t.years == 0 {
                    return Err(format!("{k}: a debt needs a principal and a term"));
                }
                for (g, r) in &pairs {
                    debts.push(DebtTerm::Agreement {
                        debtor: (*g).into(),
                        creditor: (*r).into(),
                        treaty: key.into(),
                        terms: DebtTerms {
                            principal: t.principal.mul_ratio(1, n),
                            annuity: t.annuity.mul_ratio(1, n),
                            contingent: t.contingent.mul_ratio(1, n),
                            ..t.clone()
                        },
                    });
                }
            }
            ClauseKind::Payment { amount } => {
                for (g, r) in &pairs {
                    out.payments
                        .push(((*g).into(), (*r).into(), amount.mul_ratio(1, n)));
                }
            }
            ClauseKind::ReviseDebt { treaty, terms } => {
                for g in &givers {
                    out.revisions
                        .push(((*g).into(), treaty.clone(), terms.clone()));
                }
            }
            _ => out.kept.push(cl.clone()),
        }
    }
    out.agreement = Some(Agreement {
        id: 0,
        key: key.into(),
        side_a: proposal.side_a.clone(),
        side_b: proposal.side_b.clone(),
        signed: 0,
        term_years: proposal.term_years,
        notice_years: proposal.notice_years,
        kept: out.kept.clone(),
        status: AgreementStatus::InForce,
    });
    if !debts.is_empty() {
        out.ops.push(Operation::Terms {
            terms: Terms {
                debt: debts,
                techniques: Vec::new(),
            },
        });
    }
    Ok(out)
}

/// Puts a settlement in motion: the operations join the border queue (run at the end of the
/// day, each checked before anything moves); payments and revisions act now. Money moves
/// between treasuries only, so the world's sum does not change. The agreement is recorded in
/// `GameWorld::treaties.agreements`, signed in `year`; returns its id.
pub fn queue(world: &mut GameWorld, s: &Settlement, year: i32) -> Option<u32> {
    world.borders.pending.extend(s.ops.iter().cloned());
    let index = |w: &GameWorld, t: &str| w.countries.iter().position(|(_, c)| c.tag == t);
    for (from, to, amount) in &s.payments {
        let (Some(a), Some(b)) = (index(world, from), index(world, to)) else {
            continue;
        };
        if a != b && a < world.econ.markets.len() && b < world.econ.markets.len() {
            world.econ.markets[a].treasury -= *amount;
            world.econ.markets[b].treasury += *amount;
        }
    }
    for (creditor, key, terms) in &s.revisions {
        let Some(c) = index(world, creditor) else {
            continue;
        };
        let record = world.econ.ledger.record;
        for d in world
            .econ
            .ledger
            .debts
            .iter_mut()
            .filter(|d| usize::from(d.creditor) == c && d.treaty == *key)
        {
            d.rate = terms.rate;
            d.annuity = terms.annuity;
            d.years_left = terms.years;
            if terms.grace_years > 0 {
                d.suspended_until = year + i32::from(terms.grace_years) - 1;
            }
            // A revision may forgive, never add: the principal only falls.
            if terms.principal < d.principal {
                let to = terms.principal.max(Fixed::ZERO);
                if record {
                    d.written_off += d.principal - to;
                }
                d.principal = to;
            }
        }
    }
    // The record, with what holds while in force.
    s.agreement.clone().map(|mut a| {
        a.signed = year;
        world.treaties.sign(a)
    })
}
