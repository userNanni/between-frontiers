//! Phase 3d: the politics of each country as the simulation runs it. The catalogue (laws,
//! interest groups, movements) is in [`crate::politics`]; this module holds what changes: the
//! laws in force, the clout of each interest group, the support of each movement and the record
//! of every law change, plus the system that keeps them and the read API of the interface.
//!
//! Everything is indexed by country in scenario order (the same index as `econ.markets`), by law
//! group, by interest group and by movement kind in catalogue order. Rules in
//! `docs/politics.pt-BR.md`.

use std::sync::Arc;

use laplace_schelling::Date;
use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::schedule::{Access, Cadence, ResourceId, System, TickCtx};
use serde::{Deserialize, Serialize};

use crate::econ::prof::P;
use crate::econ::week::ECON;
use crate::game::GameWorld;
use crate::politics::{EFFECTS, LawsInForce, Politics, Scope};

/// The politics state of every country.
pub const POLITY: ResourceId = ResourceId(12);

/// Per mille: the unit of clout, support and legitimacy.
pub const MILLE: u16 = 1000;

/// One movement in one country.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementState {
    /// Per mille of the people it draws from who back it; 0 is no movement.
    pub support: u16,
    /// How far its cause holds now, per mille ([`crate::politics::MovementKind::pressure`]);
    /// 0 when any condition fails.
    pub pressure: u16,
    /// Index in the kind's `cause` of the condition with the smallest margin, the one the
    /// interface names as holding the movement back.
    pub weakest: u8,
}

impl HashState for MovementState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.support);
        h.write_u16(self.pressure);
        h.write_u8(self.weakest);
    }
}

/// Who changed a law.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enactor {
    /// A command of a player (or of a tool acting as one).
    Command,
    /// The country itself, under the pressure of this movement kind.
    Movement(u16),
    /// The country itself, because the interest groups that count backed the law.
    Groups,
}

impl HashState for Enactor {
    fn hash_state(&self, h: &mut StateHasher) {
        match self {
            Enactor::Command => h.write_u8(0),
            Enactor::Movement(m) => {
                h.write_u8(1);
                h.write_u16(*m);
            }
            Enactor::Groups => h.write_u8(2),
        }
    }
}

/// A law change that happened in the campaign.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawChangeRecord {
    pub date: Date,
    pub country: u16,
    /// Law in force before, and the law that replaced it (same group).
    pub from: u16,
    pub to: u16,
    pub by: Enactor,
}

impl HashState for LawChangeRecord {
    fn hash_state(&self, h: &mut StateHasher) {
        self.date.hash_state(h);
        h.write_u16(self.country);
        h.write_u16(self.from);
        h.write_u16(self.to);
        self.by.hash_state(h);
    }
}

/// The politics of every country, saved and hashed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoliticsState {
    /// Law in force by country and law group; `None` for a colonial group of a country without
    /// colonies.
    pub laws: Vec<Vec<Option<u16>>>,
    /// (law group, community ids) the targeted law of that group falls on, by country.
    pub targets: Vec<Vec<(u16, Vec<String>)>>,
    /// Share of the political weight of each interest group, per mille, by country.
    pub clout: Vec<Vec<u16>>,
    /// Every movement kind, by country.
    pub movements: Vec<Vec<MovementState>>,
    /// Consent to the laws in force, per mille, by country.
    pub legitimacy: Vec<u16>,
    /// Laws asked for by command and not yet enacted: (country, law), oldest first. Of two
    /// requests for the same law group of a country, the newer wins.
    pub pending: Vec<(u16, u16)>,
    /// Every law change since the start, oldest first.
    pub changes: Vec<LawChangeRecord>,
}

impl HashState for PoliticsState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.laws.len());
        for c in &self.laws {
            c.hash_state(h);
        }
        h.write_len(self.targets.len());
        for c in &self.targets {
            h.write_len(c.len());
            for (g, ids) in c {
                h.write_u16(*g);
                ids.hash_state(h);
            }
        }
        h.write_len(self.clout.len());
        for c in &self.clout {
            c.hash_state(h);
        }
        h.write_len(self.movements.len());
        for c in &self.movements {
            c.hash_state(h);
        }
        self.legitimacy.hash_state(h);
        self.pending.hash_state(h);
        self.changes.hash_state(h);
    }
}

impl PoliticsState {
    /// The state on the start date: the scenario's laws, nothing else moving yet.
    pub fn initial(p: &Politics, laws: &LawsInForce) -> Self {
        let n = laws.laws.len();
        Self {
            laws: laws.laws.clone(),
            targets: laws.targets.clone(),
            clout: vec![vec![0; p.interest_groups.len()]; n],
            movements: vec![vec![MovementState::default(); p.movements.len()]; n],
            legitimacy: vec![MILLE; n],
            pending: Vec::new(),
            changes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.laws.is_empty()
    }

    // ---- Read API for the interface -----------------------------------------------------

    /// Law in force of each law group (`None`: a colonial group the country does not hold).
    pub fn laws_in_force(&self, country: usize) -> &[Option<u16>] {
        self.laws.get(country).map_or(&[], Vec::as_slice)
    }

    /// Law in force of one group.
    pub fn law_in_force(&self, country: usize, group: usize) -> Option<u16> {
        self.laws_in_force(country).get(group).copied().flatten()
    }

    /// Share of the political weight of each interest group, per mille, in catalogue order.
    pub fn clout(&self, country: usize) -> &[u16] {
        self.clout.get(country).map_or(&[], Vec::as_slice)
    }

    /// Movements with any support: (movement kind, state), in catalogue order.
    pub fn active_movements(
        &self,
        country: usize,
    ) -> impl Iterator<Item = (u16, &MovementState)> + '_ {
        self.movements
            .get(country)
            .into_iter()
            .flatten()
            .enumerate()
            .filter(|(_, m)| m.support > 0)
            .map(|(k, m)| (k as u16, m))
    }

    /// Consent to the laws in force, per mille.
    pub fn legitimacy(&self, country: usize) -> u16 {
        self.legitimacy.get(country).copied().unwrap_or(MILLE)
    }

    /// Laws asked for and not yet enacted.
    pub fn pending_laws(&self, country: usize) -> impl Iterator<Item = u16> + '_ {
        self.pending
            .iter()
            .filter(move |(c, _)| usize::from(*c) == country)
            .map(|(_, l)| *l)
    }

    /// The law changes of one country, oldest first.
    pub fn law_changes(&self, country: usize) -> impl Iterator<Item = &LawChangeRecord> + '_ {
        self.changes
            .iter()
            .filter(move |r| usize::from(r.country) == country)
    }

    /// Asks for `law` in `country`; the politics system enacts it. Unknown countries and laws
    /// are dropped there, not here (the command has no catalogue).
    pub fn request(&mut self, country: u16, law: u16) {
        self.pending.retain(|&(c, l)| !(c == country && l == law));
        self.pending.push((country, law));
    }
}

/// The laws a country can choose from in `group`: every law of the group, or none for a
/// colonial group the country does not hold.
pub fn possible_laws<'a>(
    p: &'a Politics,
    state: &PoliticsState,
    country: usize,
    group: usize,
) -> &'a [u16] {
    match p.groups.get(group) {
        Some(g) if state.law_in_force(country, group).is_some() => &g.laws,
        _ => &[],
    }
}

/// Whose effects a law of `group` falls on, for the stance of the interest groups: the
/// colonies for the colonial groups, the targeted communities for a targeted law, the
/// metropole otherwise.
pub fn scope_of(p: &Politics, law: usize) -> Scope {
    let l = &p.laws[law];
    if p.groups[usize::from(l.group)].colonial {
        Scope::Colonies
    } else if l.targeted {
        Scope::Targeted
    } else {
        Scope::Metropole
    }
}

/// An interest group's stance on replacing the law in force with another, effect by effect.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stance {
    /// (effect index into [`EFFECTS`], weight × change of the combined effect), only the
    /// effects that move and that the group cares about. Positive: the group gains.
    pub terms: Vec<(u8, i64)>,
    /// Sum of the terms: above 0 the group backs the change, below 0 it opposes it.
    pub total: i64,
}

/// Stance of interest group `ig` on enacting `law` in `country`, against the laws in force.
/// The terms read the combined value of every effect under the whole set of laws, so a cap or
/// a multiplier from another law group counts as the people would feel it. Enacting the law
/// already in force gives an empty stance.
pub fn stance(
    p: &Politics,
    state: &PoliticsState,
    country: usize,
    ig: usize,
    law: usize,
) -> Stance {
    let Some(current) = state.laws.get(country) else {
        return Stance::default();
    };
    let Some(l) = p.laws.get(law) else {
        return Stance::default();
    };
    let g = usize::from(l.group);
    if current
        .get(g)
        .copied()
        .flatten()
        .is_none_or(|c| usize::from(c) == law)
    {
        return Stance::default();
    }
    let scope = scope_of(p, law);
    let before = p.effects(current, scope);
    let mut proposed = current.clone();
    proposed[g] = Some(law as u16);
    let after = p.effects(&proposed, scope);
    let terms: Vec<(u8, i64)> = p.interest_groups[ig]
        .interests
        .iter()
        .filter_map(|&(e, w)| {
            let d = after[usize::from(e)] - before[usize::from(e)];
            (d != 0).then_some((e, i64::from(w) * i64::from(d)))
        })
        .collect();
    let total = terms.iter().map(|(_, v)| v).sum();
    Stance { terms, total }
}

/// The value of every effect for the people of `scope` in `country`, indexed like [`EFFECTS`].
pub fn effects_in_force(
    p: &Politics,
    state: &PoliticsState,
    country: usize,
    scope: Scope,
) -> Vec<i32> {
    match state.laws.get(country) {
        Some(laws) => p.effects(laws, scope),
        None => EFFECTS.iter().map(|e| e.neutral()).collect(),
    }
}

// ---------------------------------------------------------------------------------------------
// System

/// Keeps the politics of every country: enacts the laws asked for and measures the clout of
/// the interest groups.
pub struct PoliticsSystem {
    pub politics: Arc<Politics>,
    /// Per mille of each profession in each interest group, resolved at load.
    shares: Vec<[u16; P]>,
}

impl PoliticsSystem {
    pub fn new(politics: Arc<Politics>) -> Self {
        let shares = politics
            .interest_groups
            .iter()
            .map(|g| {
                let mut row = [0u16; P];
                // Groups of one community recruit nobody until communities live in the pops.
                if g.community.is_none() {
                    for &(pi, s) in &g.professions {
                        row[usize::from(pi)] = s;
                    }
                }
                row
            })
            .collect();
        Self { politics, shares }
    }

    /// Clout of each interest group in `country`, per mille: half from its share of the
    /// metropole's people, half from its share of their income.
    fn clout(&self, world: &GameWorld, country: usize) -> Vec<u16> {
        let mut size = [0i128; P];
        let mut income = [0i128; P];
        for r in world
            .econ
            .regions
            .iter()
            .filter(|r| usize::from(r.market) == country && !r.colony)
        {
            for (p, pop) in r.pops.iter().enumerate().take(P) {
                size[p] += i128::from(pop.size.max(0));
                income[p] += i128::from(pop.income.raw().max(0));
            }
        }
        let (ts, ti) = (size.iter().sum::<i128>(), income.iter().sum::<i128>());
        self.shares
            .iter()
            .map(|row| {
                let part = |v: &[i128; P], total: i128| -> i128 {
                    if total <= 0 {
                        return 0;
                    }
                    row.iter()
                        .zip(v)
                        .map(|(&s, &x)| i128::from(s) * x)
                        .sum::<i128>()
                        / total
                };
                ((part(&size, ts) + part(&income, ti)) / 2).clamp(0, 1000) as u16
            })
            .collect()
    }
}

#[derive(Default)]
pub struct PoliticsChanges {
    /// (country, law) to enact, in request order.
    enact: Vec<(u16, u16)>,
    clout: Option<Vec<Vec<u16>>>,
}

impl System<GameWorld> for PoliticsSystem {
    type Changes = PoliticsChanges;

    fn name(&self) -> &'static str {
        "politics"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Daily
    }

    fn access(&self) -> Access {
        Access::new().read(ECON).read(POLITY).write(POLITY)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> PoliticsChanges {
        let s = &world.politics;
        let p = &*self.politics;
        let group = |l: u16| p.laws.get(usize::from(l)).map(|law| usize::from(law.group));
        let enact = s
            .pending
            .iter()
            .enumerate()
            .filter(|&(i, &(c, l))| {
                let Some(g) = group(l) else {
                    return false;
                };
                let superseded = s.pending[i + 1..]
                    .iter()
                    .any(|&(c2, l2)| c2 == c && group(l2) == Some(g));
                !superseded
                    && s.law_in_force(usize::from(c), g)
                        .is_some_and(|cur| cur != l)
            })
            .map(|(_, &x)| x)
            .collect();
        let clout = (ctx.date.day == 1 && !world.econ.regions.is_empty())
            .then(|| (0..s.laws.len()).map(|c| self.clout(world, c)).collect());
        PoliticsChanges { enact, clout }
    }

    fn apply(&self, ctx: &TickCtx, changes: PoliticsChanges, world: &mut GameWorld) {
        let s = &mut world.politics;
        let p = &*self.politics;
        for (c, l) in changes.enact {
            let g = usize::from(p.laws[usize::from(l)].group);
            let Some(slot) = s.laws.get_mut(usize::from(c)).and_then(|v| v.get_mut(g)) else {
                continue;
            };
            let Some(from) = slot.replace(l) else {
                continue;
            };
            if from != l {
                s.changes.push(LawChangeRecord {
                    date: ctx.date,
                    country: c,
                    from,
                    to: l,
                    by: Enactor::Command,
                });
            }
        }
        s.pending.clear();
        if let Some(clout) = changes.clout {
            s.clout = clout;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_newer_request_replaces_the_same_law_once() {
        let mut s = PoliticsState::default();
        s.request(0, 3);
        s.request(1, 3);
        s.request(0, 3);
        assert_eq!(s.pending, vec![(1, 3), (0, 3)]);
        assert_eq!(s.pending_laws(0).collect::<Vec<_>>(), vec![3]);
    }
}
