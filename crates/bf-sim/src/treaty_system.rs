//! The monthly step of the agreements in force (`docs/treaties.pt-BR.md`, sections 3 and 9):
//! on the 2nd of each month the continuous clauses that can be measured today are carried out
//! and tested, and in January the agreements whose fixed term ran out end. Behind
//! `Switches::clause_tests` (off): without it nothing here runs and the world hashes as before.
//!
//! Tested today: `tribute` (the giver pays a twelfth of the yearly sum to each receiver; a
//! treasury that cannot pay it is a breach, `ServiceUnpaid`, at most one a year per clause and
//! giver). The other continuous clauses wait for their fronts (section 14): their test is a
//! breach only once there is something to measure.
//!
//! With `Switches::reputation` (9.4, owner's decisions P1–P3 of 2026-11-18) the same step makes
//! the reputation forget a tenth a year in January, and halves the security reputation of a
//! country whose form of government changed since the last step.

use laplace_schelling::Fixed;
use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};

use crate::game::GameWorld;
use crate::treaty::{BreachCause, ClauseKind};

/// The treaty state (`GameWorld::treaties`).
pub const TREATIES: ResourceId = ResourceId(19);

/// What the monthly step reads besides the world.
#[derive(Clone, Debug, Default)]
pub struct TreatySetup {
    /// Yearly loss of the reputation, a fraction (`Weights::reputation_decay`).
    pub decay: Fixed,
    /// The laws that are forms of government (`politics::GOVERNMENT_GROUP`): a change to one is
    /// a change of regime.
    pub government: Vec<u16>,
}

/// What one month of the agreements does.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Month {
    /// The reputation forgets this share (January, with the reputation kept).
    pub decay: Option<Fixed>,
    /// Countries whose form of government changed since the last step.
    pub new_regimes: Vec<String>,
    /// Agreements whose term ran out end this year (January only).
    pub expire: Option<i32>,
    /// (from, to, amount): payments between treasuries, market indices, reference units.
    pub payments: Vec<(u16, u16, Fixed)>,
    /// (agreement, clause, by, value): payments a treasury could not make.
    pub unpaid: Vec<(u32, usize, String, Fixed)>,
}

pub struct TreatyMonth {
    pub setup: std::sync::Arc<TreatySetup>,
    /// `Switches::clause_tests`: tribute and terms. Without it only the reputation runs.
    pub tests: bool,
}

/// The month of the agreements on `date`, read from the world (the compute half of the system).
pub fn month(
    world: &GameWorld,
    setup: &TreatySetup,
    tests: bool,
    date: laplace_schelling::Date,
) -> Month {
    let (year, month_of_year) = (date.year, date.month);
    let mut out = Month::default();
    if world.treaties.reputation_on {
        if month_of_year == 1 {
            out.decay = Some(setup.decay);
        }
        let since = if month_of_year == 1 {
            laplace_schelling::Date::new(year - 1, 12, date.day)
        } else {
            laplace_schelling::Date::new(year, month_of_year - 1, date.day)
        };
        let mut seen = Vec::new();
        for r in &world.politics.changes {
            if r.date >= since
                && r.date < date
                && setup.government.contains(&r.to)
                && !seen.contains(&r.country)
            {
                seen.push(r.country);
                if let Some((_, c)) = world.countries.iter().nth(usize::from(r.country)) {
                    out.new_regimes.push(c.tag.clone());
                }
            }
        }
    }
    if !tests {
        return out;
    }
    out.expire = (month_of_year == 1).then_some(year);
    let index = |tag: &str| {
        world
            .countries
            .iter()
            .position(|(_, c)| c.tag == tag)
            .filter(|&i| i < world.econ.markets.len())
    };
    // What each treasury has left this month, as the payments go out in order.
    let mut left: Vec<Fixed> = world.econ.markets.iter().map(|m| m.treasury).collect();
    for a in &world.treaties.agreements {
        for (ci, c) in a.kept.iter().enumerate() {
            let ClauseKind::Tribute { yearly } = &c.kind else {
                continue;
            };
            let givers = a.side(&c.giver);
            let receivers = a.side(&c.receiver);
            let pairs: Vec<(&str, &str)> = givers
                .iter()
                .flat_map(|g| receivers.iter().map(move |r| (*g, *r)))
                .filter(|(g, r)| g != r)
                .collect();
            if pairs.is_empty() {
                continue;
            }
            let due = yearly.mul_ratio(1, 12 * pairs.len() as i64);
            for (g, r) in pairs {
                let (Some(gi), Some(ri)) = (index(g), index(r)) else {
                    continue;
                };
                if left[gi] >= due {
                    left[gi] -= due;
                    left[ri] += due;
                    out.payments.push((gi as u16, ri as u16, due));
                } else if !world.treaties.breaches.iter().any(|b| {
                    b.agreement == a.id
                        && usize::from(b.clause) == ci
                        && b.by == g
                        && b.date.year == year
                }) && !out
                    .unpaid
                    .iter()
                    .any(|u| u.0 == a.id && u.1 == ci && u.2 == g)
                {
                    out.unpaid.push((a.id, ci, g.to_owned(), due));
                }
            }
        }
    }
    out
}

/// Carries out a month (the apply half): money moves between treasuries only, so the world's
/// sum does not change.
pub fn apply_month(world: &mut GameWorld, m: Month, date: laplace_schelling::Date) {
    for (from, to, amount) in m.payments {
        world.econ.markets[usize::from(from)].treasury -= amount;
        world.econ.markets[usize::from(to)].treasury += amount;
    }
    for (agreement, clause, by, value) in m.unpaid {
        world.treaties.record_breach(
            agreement,
            clause,
            &by,
            value,
            date,
            BreachCause::ServiceUnpaid,
        );
    }
    if let Some(year) = m.expire {
        world.treaties.expire(year);
    }
    if let Some(d) = m.decay {
        world.treaties.decay_reputation(d);
    }
    for tag in &m.new_regimes {
        world.treaties.new_regime(tag);
    }
}

impl System<GameWorld> for TreatyMonth {
    type Changes = Month;

    fn name(&self) -> &'static str {
        "treaties"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Monthly { day: 2 }
    }

    fn access(&self) -> Access {
        Access::new()
            .read(crate::polity::POLITY)
            .write(crate::econ::week::ECON)
            .write(TREATIES)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> Month {
        if world.treaties.agreements.is_empty() && world.treaties.reputation.is_empty() {
            return Month::default();
        }
        month(world, &self.setup, self.tests, ctx.date)
    }

    fn apply(&self, ctx: &TickCtx, changes: Month, world: &mut GameWorld) {
        apply_month(world, changes, ctx.date);
    }
}
