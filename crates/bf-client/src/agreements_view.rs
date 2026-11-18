//! The agreements of the player's country (`docs/treaties.pt-BR.md`, sections 9 and 13): those
//! in force with their clauses, term and notice; those ended; the breaches that touch it and the
//! causes it holds or faces; and the two reputations (general and security) of every country,
//! for the journal's agreements screen, where the player also denounces. Read in the
//! simulation's thread when the agreements change.

use bf_sim::game::GameWorld;
use bf_sim::treaty::{Agreement, AgreementStatus, BreachCause};

/// One clause an agreement keeps.
#[derive(Clone, Debug, PartialEq)]
pub struct ClauseLine {
    pub key: &'static str,
    pub givers: Vec<String>,
    pub receivers: Vec<String>,
    /// The player's country gives it.
    pub mine: bool,
    /// The date it ends before the agreement (ISO), if any.
    pub until: Option<String>,
}

/// One agreement.
#[derive(Clone, Debug, PartialEq)]
pub struct AgreementLine {
    pub id: u32,
    /// `treaty-<key>` in the Fluent files; agreements signed in play are `agreement-<id>`.
    pub key: String,
    pub side_a: Vec<String>,
    pub side_b: Vec<String>,
    pub signed: i32,
    pub term_years: Option<u16>,
    pub notice_years: u16,
    pub clauses: Vec<ClauseLine>,
    /// The year it ended (ended ones) or will end by its term (in force), if any.
    pub ends: Option<i32>,
}

impl AgreementLine {
    fn of(a: &Agreement, me: &str) -> Self {
        let tags = |v: Vec<&str>| v.into_iter().map(str::to_owned).collect::<Vec<_>>();
        AgreementLine {
            id: a.id,
            key: a.key.clone(),
            side_a: a.side_a.clone(),
            side_b: a.side_b.clone(),
            signed: a.signed,
            term_years: a.term_years,
            notice_years: a.notice_years,
            clauses: a
                .kept
                .iter()
                .map(|c| {
                    let givers = tags(a.side(&c.giver));
                    ClauseLine {
                        key: c.kind.key(),
                        mine: givers.iter().any(|g| g == me),
                        givers,
                        receivers: tags(a.side(&c.receiver)),
                        until: c.until.clone(),
                    }
                })
                .collect(),
            ends: match a.status {
                AgreementStatus::Ended { year } => Some(year),
                AgreementStatus::InForce => a.term_years.map(|t| a.signed + i32::from(t)),
            },
        }
    }

    /// The clauses the player's country gives: what denouncing without notice breaks.
    pub fn given(&self) -> impl Iterator<Item = &ClauseLine> {
        self.clauses.iter().filter(|c| c.mine)
    }
}

/// One breach that touches the player's country.
#[derive(Clone, Debug, PartialEq)]
pub struct BreachLine {
    pub date: laplace_schelling::calendar::Date,
    pub agreement: String,
    pub clause: Option<&'static str>,
    pub by: String,
    pub harmed: Vec<String>,
    /// What the harmed lose, real dollars.
    pub value: f64,
    pub cause: BreachCause,
}

/// One country's reputation: what its broken promises weigh, by kind.
#[derive(Clone, Debug, PartialEq)]
pub struct ReputationLine {
    pub tag: String,
    pub general: f64,
    pub security: f64,
    /// Its GDP of the last year: a promise is discounted by the reputation over it.
    pub gdp: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgreementsView {
    pub me: String,
    pub in_force: Vec<AgreementLine>,
    pub ended: Vec<AgreementLine>,
    pub breaches: Vec<BreachLine>,
    /// Causes the player's country holds (`true`) or faces: (other party, value, held).
    pub causes: Vec<(String, f64, bool)>,
    /// The reputation is kept (`Switches::reputation`).
    pub reputation_on: bool,
    /// Every country with a reputation, the heaviest first; the player's always.
    pub reputation: Vec<ReputationLine>,
    /// Denunciations sent and not yet carried out (the table's next Monday).
    pub pending: Vec<u32>,
    /// The negotiation table runs (`Switches::negotiation`): it carries denunciations out.
    pub can_denounce: bool,
    pub signature: Signature,
}

pub type Signature = (usize, usize, usize, usize, usize, i32);

pub fn signature(world: &GameWorld, year: i32) -> Signature {
    let t = &world.treaties;
    (
        t.agreements.len(),
        t.ended.len(),
        t.breaches.len(),
        t.table.denounce.len(),
        t.reputation.len(),
        year,
    )
}

/// The player's agreements; `None` without a player or without any agreement on record.
pub fn read(world: &GameWorld, year: i32, can_denounce: bool) -> Option<AgreementsView> {
    let me = world.player_tag()?;
    let t = &world.treaties;
    if t.agreements.is_empty() && t.ended.is_empty() {
        return None;
    }
    let mine = |a: &&Agreement| a.side_a.iter().chain(&a.side_b).any(|x| x == me);
    let in_force: Vec<AgreementLine> = t
        .agreements
        .iter()
        .filter(mine)
        .map(|a| AgreementLine::of(a, me))
        .collect();
    let ended: Vec<AgreementLine> = t
        .ended
        .iter()
        .rev()
        .filter(mine)
        .map(|a| AgreementLine::of(a, me))
        .collect();
    let find = |id: u32| t.agreements.iter().chain(&t.ended).find(|a| a.id == id);
    let breaches = t
        .breaches
        .iter()
        .rev()
        .filter(|b| b.by == me || b.harmed.iter().any(|h| h == me))
        .map(|b| {
            let a = find(b.agreement);
            BreachLine {
                date: b.date,
                agreement: a.map_or_else(String::new, |a| a.key.clone()),
                clause: a
                    .and_then(|a| a.kept.get(usize::from(b.clause)))
                    .map(|c| c.kind.key()),
                by: b.by.clone(),
                harmed: b.harmed.clone(),
                value: b.value.to_f64_lossy(),
                cause: b.cause,
            }
        })
        .collect();
    let causes = t
        .causes
        .iter()
        .filter_map(|c| {
            if c.holder == me {
                Some((c.against.clone(), c.value.to_f64_lossy(), true))
            } else if c.against == me {
                Some((c.holder.clone(), c.value.to_f64_lossy(), false))
            } else {
                None
            }
        })
        .collect();
    let gdp = |tag: &str| {
        world
            .countries
            .iter()
            .position(|(_, c)| c.tag == tag)
            .and_then(|i| world.econ.markets.get(i))
            .map_or(0.0, |m| m.gdp_last_year.to_f64_lossy())
    };
    let mut reputation: Vec<ReputationLine> = t
        .reputation
        .iter()
        .filter(|(tag, r)| {
            tag.as_str() == me || r.general.is_positive() || r.security.is_positive()
        })
        .map(|(tag, r)| ReputationLine {
            tag: tag.clone(),
            general: r.general.to_f64_lossy(),
            security: r.security.to_f64_lossy(),
            gdp: gdp(tag),
        })
        .collect();
    if !reputation.iter().any(|r| r.tag == me) {
        reputation.push(ReputationLine {
            tag: me.to_owned(),
            general: 0.0,
            security: 0.0,
            gdp: gdp(me),
        });
    }
    reputation.sort_by(|a, b| {
        (b.general + b.security)
            .total_cmp(&(a.general + a.security))
            .then(a.tag.cmp(&b.tag))
    });
    Some(AgreementsView {
        me: me.to_owned(),
        in_force,
        ended,
        breaches,
        causes,
        reputation_on: t.reputation_on,
        reputation,
        pending: t
            .table
            .denounce
            .iter()
            .filter(|(_, tag)| tag == me)
            .map(|(id, _)| *id)
            .collect(),
        can_denounce,
        signature: signature(world, year),
    })
}
