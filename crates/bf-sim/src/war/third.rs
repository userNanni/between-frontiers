//! Third parties in a war (phase 5, slice B, W4): who the agreements in force call in when a war
//! is declared, by which clause, and what not answering costs. The duties come from the treaties
//! (`TreatyState::duties`); a breach goes through their record (`record_breach`, section 9 of
//! `docs/treaties.pt-BR.md`), unless the clause stands suspended by an earlier breach of the
//! partner. Design in `docs/war.pt-BR.md`, "Terceiros: quem entra, por qual cláusula".

use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::{Date, Fixed};
use serde::{Deserialize, Serialize};

use crate::game::GameWorld;
use crate::treaty::{BreachCause, DutyKind};

/// The side of a war a third party is called to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallSide {
    /// Beside the one that declared (an offence clause).
    Attacker,
    /// Beside the one declared on (defence, border guarantee).
    Defender,
}

/// A call the player has to answer: honour (go to war) or not (a breach).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    pub war: u32,
    /// Market index of the party called.
    pub party: u16,
    pub side: CallSide,
    pub agreement: u32,
    /// Index of the clause in `Agreement::kept`.
    pub clause: u16,
    pub weeks_left: u16,
}

/// One line of "who would join": what an agreement in force asks of a third party if `attacker`
/// declares on `attacked`, for the crisis screen before the declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub party: String,
    pub kind: DutyKind,
    pub agreement: u32,
    pub clause: u16,
    /// The side it would be called to; `None` for neutrality and non-aggression.
    pub side: Option<CallSide>,
    /// The partner broke the agreement first: the clause stands suspended, no call and no breach.
    pub suspended: bool,
    /// A neutrality of the same party bars this side: it is not called.
    pub barred: bool,
}

/// Who the agreements in force would call in, and who they would hold back, if `attacker`
/// declared war on `attacked` (tags). A party called to both sides answers the defender's call
/// only: defence comes before offence. A neutrality toward one side bars joining against it.
pub fn who_would_join(world: &GameWorld, attacker: &str, attacked: &str) -> Vec<Entry> {
    let t = &world.treaties;
    let duties = t.duties(attacker, attacked);
    // Neutralities: the party may not join against the partner (the receiver of the clause).
    let neutral_toward = |party: &str| -> (bool, bool) {
        let mut toward = (false, false); // (attacker, attacked)
        for d in duties
            .iter()
            .filter(|d| d.kind == DutyKind::StayOut && d.party == party)
        {
            if let Some(a) = t.agreements.iter().find(|a| a.id == d.agreement)
                && let Some(c) = a.kept.get(d.clause as usize)
            {
                let receivers = a.side(&c.receiver);
                toward.0 |= receivers.contains(&attacker);
                toward.1 |= receivers.contains(&attacked);
            }
        }
        toward
    };
    duties
        .iter()
        .map(|d| {
            let side = match d.kind {
                DutyKind::JoinDefender => Some(CallSide::Defender),
                DutyKind::JoinAttacker => Some(CallSide::Attacker),
                DutyKind::StayOut | DutyKind::NotAttack => None,
            };
            let (to_attacker, to_attacked) = neutral_toward(&d.party);
            let defends = duties
                .iter()
                .any(|e| e.party == d.party && e.kind == DutyKind::JoinDefender);
            let barred = match side {
                // Joining the defender is fighting the attacker.
                Some(CallSide::Defender) => to_attacker,
                // Joining the attacker is fighting the attacked; and defence comes first.
                Some(CallSide::Attacker) => to_attacked || defends,
                None => false,
            };
            Entry {
                party: d.party.clone(),
                kind: d.kind,
                agreement: d.agreement,
                clause: d.clause,
                side,
                suspended: t.suspended(d.agreement, d.clause as usize, &d.party),
                barred,
            }
        })
        .collect()
}

fn tag(world: &GameWorld, market: u16) -> Option<String> {
    world
        .countries
        .iter()
        .nth(market as usize)
        .map(|(_, c)| c.tag.clone())
}

fn market(world: &GameWorld, tag: &str) -> Option<u16> {
    world
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .map(|i| i as u16)
}

/// What a party's forces cost a year at this week's prices: the value of the help a call not
/// answered withholds (`record_breach`'s value, reference units).
pub fn forces_cost_year(world: &GameWorld, market: u16) -> Fixed {
    let (Some(c), Some(m)) = (
        world.war.countries.get(market as usize),
        world.econ.markets.get(market as usize),
    ) else {
        return Fixed::ZERO;
    };
    let week: Fixed = c
        .orders
        .iter()
        .map(|(g, q)| *q * m.price.get(*g as usize).copied().unwrap_or(Fixed::ZERO))
        .sum();
    week * 52
}

/// `party` enters war `war` on `side`, and calls up its reserve; a party already in it stays.
pub fn join(world: &mut GameWorld, war: u32, party: u16, side: CallSide) {
    let Some(w) = world.war.conflicts.wars.iter_mut().find(|w| w.id == war) else {
        return;
    };
    if w.side_a.contains(&party) || w.side_b.contains(&party) {
        return;
    }
    match side {
        CallSide::Attacker => w.side_a.push(party),
        CallSide::Defender => w.side_b.push(party),
    }
    if let Some(c) = world.war.countries.get_mut(party as usize) {
        c.mobilized = true;
    }
}

/// `by` does not answer the call: a breach of the clause, unless it stands suspended.
pub fn betray(world: &mut GameWorld, call: &Call, date: Date) -> Option<u32> {
    let by = tag(world, call.party)?;
    if world
        .treaties
        .suspended(call.agreement, call.clause as usize, &by)
    {
        return None;
    }
    let value = forces_cost_year(world, call.party);
    world.treaties.record_breach(
        call.agreement,
        call.clause as usize,
        &by,
        value,
        date,
        BreachCause::CallUnanswered { war: call.war },
    )
}

/// At the declaration of war `war`: the attacker's own non-aggression is broken; the AI called
/// in joins at once; the player gets a call with `weeks` to answer.
pub fn on_declaration(world: &mut GameWorld, war: u32, weeks: u16, date: Date) {
    let Some(w) = world.war.conflicts.wars.iter().find(|w| w.id == war) else {
        return;
    };
    let (Some(attacker), Some(attacked)) = (
        w.side_a.first().and_then(|&m| tag(world, m)),
        w.side_b.first().and_then(|&m| tag(world, m)),
    ) else {
        return;
    };
    let player = world
        .countries
        .iter()
        .position(|(id, _)| Some(id) == world.player)
        .map(|i| i as u16);
    let entries = who_would_join(world, &attacker, &attacked);
    for e in entries {
        if e.kind == DutyKind::NotAttack {
            if !e.suspended {
                let by = market(world, &e.party);
                let value = by.map_or(Fixed::ZERO, |m| forces_cost_year(world, m));
                world.treaties.record_breach(
                    e.agreement,
                    e.clause as usize,
                    &e.party,
                    value,
                    date,
                    BreachCause::Attacked { war },
                );
            }
            continue;
        }
        let (Some(side), false, false) = (e.side, e.suspended, e.barred) else {
            continue;
        };
        let Some(party) = market(world, &e.party) else {
            continue;
        };
        if Some(party) == player {
            let c = &mut world.war.conflicts;
            if !c.calls.iter().any(|k| k.war == war && k.party == party) {
                c.calls.push(Call {
                    war,
                    party,
                    side,
                    agreement: e.agreement,
                    clause: e.clause,
                    weeks_left: weeks,
                });
            }
        } else {
            // The AI honours its agreements (W4); its choice comes with the strategic AI.
            join(world, war, party, side);
        }
    }
}

/// The player answers the call of war `war`: honour (go to war) or not (a breach).
pub fn answer(world: &mut GameWorld, war: u32, party: u16, honour: bool, date: Date) {
    let Some(i) = world
        .war
        .conflicts
        .calls
        .iter()
        .position(|k| k.war == war && k.party == party)
    else {
        return;
    };
    let call = world.war.conflicts.calls.remove(i);
    if honour {
        join(world, call.war, call.party, call.side);
    } else {
        betray(world, &call, date);
    }
}

impl HashState for Call {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u32(self.war);
        h.write_u16(self.party);
        h.write_u8(self.side as u8);
        h.write_u32(self.agreement);
        h.write_u16(self.clause);
        h.write_u16(self.weeks_left);
    }
}
