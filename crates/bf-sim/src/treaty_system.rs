//! The treaty system (phase 4): answers the proposals made by command, once a day, with the
//! same account for every party (`treaty::evaluate`), the recognition gate and the ratification
//! by the interest groups at home; executes the one-off clauses the live fronts allow and keeps
//! the continuous ones as an agreement in force. Design: `docs/treaties.pt-BR.md`, sections 8
//! to 10.
//!
//! No proposal, no work: the system returns at once when the queue is empty, so a run without
//! diplomacy costs nothing and keeps its hash.

use std::sync::Arc;

use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};
use laplace_schelling::{Date, Fixed};

use crate::game::GameWorld;
use crate::politics::{Politics, Scope};
use crate::treaty::{
    ANSWERS_KEPT, Agreement, Answer, ClauseCatalogue, ClauseKind, EvalInputs, InterstateDebt,
    LIVE_FRONTS, Outcome, PendingProposal, Proposal, Side, evaluate, recognition_conflicts,
};

/// The treaty state (`GameWorld::treaties`).
pub const TREATIES: ResourceId = ResourceId(15);

pub struct TreatySystem {
    pub cat: Arc<ClauseCatalogue>,
    pub politics: Option<Arc<Politics>>,
}

/// What a day's answers change.
#[derive(Default)]
pub struct Changes {
    answers: Vec<Answer>,
    agreements: Vec<Agreement>,
    debts: Vec<InterstateDebt>,
    /// (from, to, amount): payments between treasuries.
    payments: Vec<(u16, u16, Fixed)>,
    /// (from, to): recognitions granted.
    recognitions: Vec<(u16, u16)>,
    /// (creditor, treaty key, terms): revisions of ledger lines.
    revisions: Vec<(u16, String, crate::treaty::DebtTerms)>,
    /// (country, legitimacy paid).
    legitimacy: Vec<(u16, u16)>,
    next_id: u32,
}

/// Support for a proposal among `country`'s interest groups, and the bar it must reach, per
/// mille of clout (equal shares before the first monthly measure). Each clause weighs on each group through the concerns of the catalogue: an
/// effect the groups already weigh (`politics::EFFECTS`) by the group's own interest; the two
/// treaty concerns (`sovereignty`, `burden`) the same for every group (design choice until the
/// groups weigh them in `interest_groups.ron`). A giver feels the concern as written, a receiver
/// the opposite. The bar is `ratify_share` of the clout times the franchise of the laws in force.
pub fn ratification(
    world: &GameWorld,
    politics: Option<&Politics>,
    cat: &ClauseCatalogue,
    proposal: &Proposal,
    country: u16,
    tag: &str,
) -> (u16, u16) {
    let Some(p) = politics else {
        return (1000, 0);
    };
    let c = usize::from(country);
    let (Some(laws), Some(clout)) = (world.politics.laws.get(c), world.politics.clout.get(c))
    else {
        return (1000, 0);
    };
    let effects = p.effects(laws, Scope::Metropole);
    let franchise = crate::politics::effect_index("franchise")
        .and_then(|i| effects.get(i).copied())
        .unwrap_or(0)
        .clamp(0, 1000);
    let required = (cat.weights.ratify_share * Fixed::from_int(i64::from(franchise)))
        .to_int_round()
        .clamp(0, 1000) as u16;
    // Clout is measured monthly; before the first measure every group weighs the same.
    let total: i64 = clout.iter().map(|&c| i64::from(c)).sum();
    let weight = |g: usize| {
        if total > 0 {
            i64::from(clout.get(g).copied().unwrap_or(0))
        } else {
            1
        }
    };
    let all: i64 = if total > 0 {
        total
    } else {
        p.interest_groups.len() as i64
    };
    let mut support: i64 = 0;
    for (g, group) in p.interest_groups.iter().enumerate() {
        let mut posture: i64 = 0;
        for cl in &proposal.clauses {
            let gives = members(&cl.giver, proposal).any(|t| t == tag);
            let receives = members(&cl.receiver, proposal).any(|t| t == tag);
            let sign = match (gives, receives) {
                (true, false) => 1,
                (false, true) => -1,
                _ => continue,
            };
            let Some(def) = cat.get(cl.kind.key()) else {
                continue;
            };
            for (concern, w) in &def.concerns {
                let interest = match crate::politics::effect_index(concern) {
                    Some(e) => group
                        .interests
                        .iter()
                        .find(|(x, _)| usize::from(*x) == e)
                        .map_or(0, |(_, v)| i64::from(*v)),
                    None => 1,
                };
                posture += sign * i64::from(*w) * interest;
            }
        }
        if posture >= 0 {
            support += weight(g);
        }
    }
    let share = if all > 0 { support * 1000 / all } else { 1000 };
    (share.clamp(0, 1000) as u16, required)
}

fn members<'a>(side: &'a Side, p: &'a Proposal) -> impl Iterator<Item = &'a str> + 'a {
    let list: Vec<&'a str> = match side {
        Side::Senior => p.side_a.iter().map(String::as_str).collect(),
        Side::Junior => p.side_b.iter().map(String::as_str).collect(),
        Side::All => p
            .side_a
            .iter()
            .chain(&p.side_b)
            .map(String::as_str)
            .collect(),
        Side::Parties(t) => t.iter().map(String::as_str).collect(),
    };
    list.into_iter()
}

impl TreatySystem {
    fn answer(
        &self,
        world: &GameWorld,
        date: Date,
        pending: &PendingProposal,
        id: u32,
        out: &mut Changes,
    ) -> Outcome {
        let p = &pending.proposal;
        let index = |t: &str| {
            world
                .countries
                .iter()
                .position(|(_, c)| c.tag == t)
                .map(|i| i as u16)
        };
        let parties: Vec<&String> = p.side_a.iter().chain(&p.side_b).collect();
        if p.side_a.is_empty() || p.side_b.is_empty() || parties.iter().any(|t| index(t).is_none())
        {
            return Outcome::Malformed;
        }
        if let Some((clause, giver, receiver)) =
            recognition_conflicts(p, &world.treaties.recognition, &self.cat, index)
                .into_iter()
                .next()
        {
            return Outcome::NotRecognised {
                clause,
                giver,
                receiver,
            };
        }
        // The answering side weighs the account; the proposer made its own when it proposed.
        let gdp = world
            .countries
            .iter()
            .enumerate()
            .map(|(i, (_, c))| {
                let g = world
                    .econ
                    .markets
                    .get(i)
                    .map_or(Fixed::ZERO, |m| m.gdp_last_year);
                (c.tag.clone(), g)
            })
            .collect();
        let reputation = world
            .treaties
            .reputation
            .iter()
            .filter_map(|(c, r)| {
                world
                    .countries
                    .iter()
                    .nth(usize::from(*c))
                    .map(|(_, x)| (x.tag.clone(), *r))
            })
            .collect();
        let inputs = EvalInputs {
            discount: self.cat.weights.discount_rate,
            gdp: &gdp,
            reputation: &reputation,
            live: LIVE_FRONTS,
        };
        for party in &p.side_b {
            let v = evaluate(&self.cat, &inputs, p, party);
            if v.total < Fixed::ZERO {
                return Outcome::Refused {
                    party: party.clone(),
                    total: v.total,
                };
            }
        }
        // Ratification at home: the answering side by the bar; the proposer by the bar unless it
        // chose to pay for ratifying below it.
        let politics = self.politics.as_deref();
        let mut paid = 0u16;
        for party in parties {
            let c = index(party).unwrap_or_default();
            let (support, required) = ratification(world, politics, &self.cat, p, c, party);
            if support >= required {
                continue;
            }
            let proposer = p.side_a.contains(party);
            if proposer && pending.ratify_anyway {
                let missing = Fixed::from_int(i64::from(required - support));
                let cost = (missing * self.cat.weights.legitimacy_per_missing)
                    .to_int_round()
                    .clamp(0, 1000) as u16;
                out.legitimacy.push((c, cost));
                paid = paid.saturating_add(cost);
                continue;
            }
            return Outcome::NotRatified {
                party: party.clone(),
                support,
                required,
            };
        }
        // Signed: the one-off clauses act now (when their front is live), the others stay.
        let mut kept = Vec::new();
        for cl in &p.clauses {
            let givers: Vec<u16> = members(&cl.giver, p).filter_map(index).collect();
            let receivers: Vec<u16> = members(&cl.receiver, p).filter_map(index).collect();
            let pairs: Vec<(u16, u16)> = givers
                .iter()
                .flat_map(|g| receivers.iter().map(move |r| (*g, *r)))
                .filter(|(g, r)| g != r)
                .collect();
            let n = pairs.len().max(1) as i64;
            match &cl.kind {
                ClauseKind::Debt { terms: Some(t) } => {
                    for (g, r) in &pairs {
                        out.debts.push(InterstateDebt {
                            id: 0,
                            debtor: *g,
                            creditor: *r,
                            treaty: format!("agreement-{id}"),
                            currency: t.currency,
                            principal: t.principal.mul_ratio(1, n),
                            rate: t.rate,
                            annuity: t.annuity.mul_ratio(1, n),
                            years_left: t.years,
                            deferrable: t.deferrable,
                            transfer_cap: t.transfer_cap,
                            pledge: t.pledge,
                            contingent: t.contingent.mul_ratio(1, n),
                            arrears: Fixed::ZERO,
                            suspended_until: if t.grace_years > 0 {
                                date.year + i32::from(t.grace_years) - 1
                            } else {
                                0
                            },
                        });
                    }
                }
                ClauseKind::Payment { amount } => {
                    for (g, r) in &pairs {
                        out.payments.push((*g, *r, amount.mul_ratio(1, n)));
                    }
                }
                ClauseKind::Recognition { .. } => {
                    out.recognitions.extend(pairs.iter().copied());
                    kept.push(cl.clone());
                }
                ClauseKind::ReviseDebt { treaty, terms } => {
                    for g in &givers {
                        out.revisions.push((*g, treaty.clone(), terms.clone()));
                    }
                }
                // One-off clauses whose front is not live yet: signed, shown, without effect.
                _ if self.cat.get(cl.kind.key()).is_some_and(|d| d.once) => kept.push(cl.clone()),
                _ => kept.push(cl.clone()),
            }
        }
        if !kept.is_empty() {
            out.agreements.push(Agreement {
                id,
                side_a: p.side_a.iter().filter_map(|t| index(t)).collect(),
                side_b: p.side_b.iter().filter_map(|t| index(t)).collect(),
                clauses: kept,
                signed: date,
                term_years: p.term_years,
                notice_years: p.notice_years,
            });
        }
        Outcome::Signed {
            legitimacy_paid: paid,
        }
    }
}

impl System<GameWorld> for TreatySystem {
    type Changes = Changes;

    fn name(&self) -> &'static str {
        "treaties"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }

    fn access(&self) -> Access {
        Access::new()
            .read(TREATIES)
            .write(TREATIES)
            .read(crate::econ::week::ECON)
            .write(crate::econ::week::ECON)
            .read(crate::polity::POLITY)
            .write(crate::polity::POLITY)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> Changes {
        let mut out = Changes::default();
        let mut id = world.treaties.next_id;
        for pending in &world.treaties.proposals {
            let outcome = self.answer(world, ctx.date, pending, id, &mut out);
            out.answers.push(Answer {
                id,
                date: ctx.date,
                side_a: pending.proposal.side_a.clone(),
                side_b: pending.proposal.side_b.clone(),
                outcome,
            });
            id += 1;
        }
        out.next_id = id;
        out
    }

    fn apply(&self, ctx: &TickCtx, ch: Changes, world: &mut GameWorld) {
        if ch.answers.is_empty() {
            return;
        }
        let t = &mut world.treaties;
        t.proposals.clear();
        t.next_id = ch.next_id;
        t.agreements.extend(ch.agreements);
        for (g, r) in ch.recognitions {
            t.recognition.grant(g, r);
        }
        t.answers.extend(ch.answers);
        let excess = t.answers.len().saturating_sub(ANSWERS_KEPT);
        t.answers.drain(..excess);
        for d in ch.debts {
            world.econ.ledger.create(d);
        }
        for (creditor, key, terms) in ch.revisions {
            for d in world
                .econ
                .ledger
                .debts
                .iter_mut()
                .filter(|d| d.creditor == creditor && d.treaty == key)
            {
                d.rate = terms.rate;
                d.annuity = terms.annuity;
                d.years_left = terms.years;
                if terms.grace_years > 0 {
                    d.suspended_until = ctx.date.year + i32::from(terms.grace_years) - 1;
                }
                if terms.principal >= Fixed::ZERO && terms.principal < d.principal {
                    d.principal = terms.principal;
                }
            }
        }
        for (g, r, amount) in ch.payments {
            let (g, r) = (usize::from(g), usize::from(r));
            if g < world.econ.markets.len() && r < world.econ.markets.len() {
                world.econ.markets[g].treasury -= amount;
                world.econ.markets[r].treasury += amount;
            }
        }
        for (c, cost) in ch.legitimacy {
            if let Some(l) = world.politics.legitimacy.get_mut(usize::from(c)) {
                *l = l.saturating_sub(cost);
            }
        }
    }
}
