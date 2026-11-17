//! Phase 3d: the politics of each country as the simulation runs it. The catalogue (laws,
//! interest groups, movements) is in [`crate::politics`]; this module holds what changes: the
//! laws in force, the clout of each interest group, the support of each movement and the record
//! of every law change, plus the system that keeps them and the read API of the interface.
//!
//! Everything is indexed by country in scenario order (the same index as `econ.markets`), by law
//! group, by interest group and by movement kind in catalogue order. Rules in
//! `docs/politics.pt-BR.md`.

use laplace_schelling::Date;
use laplace_schelling::hash::{HashState, StateHasher};
use laplace_schelling::schedule::ResourceId;
use serde::{Deserialize, Serialize};

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
    /// The character leading it (agitator, phase 7); none yet. A leader's push will add to the
    /// pressure of a cause that holds, never create one (`system::leader_push`).
    #[serde(default)]
    pub leader: Option<u32>,
}

impl HashState for MovementState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.support);
        h.write_u16(self.pressure);
        h.write_u8(self.weakest);
        if let Some(l) = self.leader {
            h.write_u32(l);
        }
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
    /// The character leading each interest group, by country (phase 7); empty until then.
    #[serde(default)]
    pub leaders: Vec<Vec<Option<u32>>>,
    /// Every movement kind, by country.
    pub movements: Vec<Vec<MovementState>>,
    /// Consent to the laws in force, per mille, by country.
    pub legitimacy: Vec<u16>,
    /// Laws asked for by command and not yet enacted: (country, law), oldest first. Of two
    /// requests for the same law group of a country, the newer wins.
    pub pending: Vec<(u16, u16)>,
    /// Every law change since the start, oldest first.
    pub changes: Vec<LawChangeRecord>,
    /// What a tax ceiling means in each country: the rate each class would pay without it and
    /// each class's share of income, measured monthly.
    #[serde(default)]
    pub taxes: Vec<TaxContext>,
}

/// Per mille, by tax class (lower, middle, upper): the income tax rate the budget rule (or the
/// player) asks for before any ceiling, and the class's share of the country's income.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxContext {
    pub wanted: [u16; 3],
    pub income: [u16; 3],
    /// The `fiscal_stress` indicator: in a fiscal crisis revenue weighs more.
    #[serde(default)]
    pub stress: u16,
}

impl HashState for TaxContext {
    fn hash_state(&self, h: &mut StateHasher) {
        self.wanted.hash_state(h);
        self.income.hash_state(h);
        h.write_u16(self.stress);
    }
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
        if !self.leaders.is_empty() {
            h.write_len(self.leaders.len());
            for c in &self.leaders {
                c.hash_state(h);
            }
        }
        h.write_len(self.movements.len());
        for c in &self.movements {
            c.hash_state(h);
        }
        self.legitimacy.hash_state(h);
        self.pending.hash_state(h);
        self.changes.hash_state(h);
        self.taxes.hash_state(h);
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
            leaders: Vec::new(),
            movements: vec![vec![MovementState::default(); p.movements.len()]; n],
            legitimacy: vec![MILLE; n],
            pending: Vec::new(),
            changes: Vec::new(),
            taxes: Vec::new(),
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
///
/// Tax ceilings count by the tax they let the state take, not by the ceiling: a class whose
/// rule rate is under both ceilings pays the same either way. What the ceilings add to the
/// revenue (each class's change of rate times its share of income) is a gain for the groups that
/// weigh `tax_collection`, and shows in that term.
pub fn stance(
    p: &Politics,
    state: &PoliticsState,
    country: usize,
    ig: usize,
    law: usize,
) -> Stance {
    match change(p, state, country, law) {
        Some(delta) => stance_of(p, ig, &delta),
        None => Stance::default(),
    }
}

/// [`stance`] of every interest group, in catalogue order.
pub fn stances(p: &Politics, state: &PoliticsState, country: usize, law: usize) -> Vec<Stance> {
    let delta = change(p, state, country, law);
    (0..p.interest_groups.len())
        .map(|ig| {
            delta
                .as_ref()
                .map_or_else(Stance::default, |d| stance_of(p, ig, d))
        })
        .collect()
}

fn stance_of(p: &Politics, ig: usize, delta: &[i64]) -> Stance {
    let terms: Vec<(u8, i64)> = p.interest_groups[ig]
        .interests
        .iter()
        .filter_map(|&(e, w)| {
            let d = delta[usize::from(e)];
            (d != 0).then_some((e, i64::from(w) * d))
        })
        .collect();
    let total = terms.iter().map(|(_, v)| v).sum();
    Stance { terms, total }
}

/// How every effect moves, as the people feel it, when `law` replaces the law in force of its
/// group; `None` when it is already in force or the country does not hold the group.
fn change(p: &Politics, state: &PoliticsState, country: usize, law: usize) -> Option<Vec<i64>> {
    let current = state.laws.get(country)?;
    let l = p.laws.get(law)?;
    let g = usize::from(l.group);
    if current
        .get(g)
        .copied()
        .flatten()
        .is_none_or(|c| usize::from(c) == law)
    {
        return None;
    }
    let scope = scope_of(p, law);
    let before = p.effects(current, scope);
    let mut proposed = current.clone();
    proposed[g] = Some(law as u16);
    let after = p.effects(&proposed, scope);
    let mut delta: Vec<i64> = after
        .iter()
        .zip(&before)
        .map(|(a, b)| i64::from(a - b))
        .collect();
    if let Some(t) = state.taxes.get(country) {
        // What the ceilings cut is charged on consumption (`econ::public::cap_taxes`): every
        // class pays the same indirect rate on top of its capped income tax.
        let ceiling = i64::from(
            (i128::from(crate::econ::public::INDIRECT_CEILING.raw()) * 1000
                / i128::from(laplace_schelling::Fixed::ONE.raw())) as i32,
        );
        let indirect = |v: &[i32]| -> i64 {
            let cut: i64 = TAX_CAPS
                .iter()
                .enumerate()
                .map(|(k, &e)| {
                    let wanted = i64::from(t.wanted[k]);
                    wanted - wanted.min(i64::from(v[e]))
                })
                .sum();
            (cut / 3).min(ceiling)
        };
        let (ind_before, ind_after) = (indirect(&before), indirect(&after));
        let mut revenue = 0i64;
        for (k, &e) in TAX_CAPS.iter().enumerate() {
            let wanted = i32::from(t.wanted[k]);
            let paid = |v: &[i32], ind: i64| i64::from(v[e].min(wanted)) + ind;
            delta[e] = paid(&after, ind_after) - paid(&before, ind_before);
            revenue += delta[e] * i64::from(t.income[k]) / 1000;
        }
        // A treasury in crisis makes every point of revenue count for more: up to three times.
        delta[TAX_COLLECTION] += revenue * (1000 + 2 * i64::from(t.stress)) / 1000;
    }
    Some(delta)
}

/// The ceilings of the lower, middle and upper class, and the collection rate, in [`EFFECTS`]
/// (checked by a test).
const TAX_CAPS: [usize; 3] = [2, 3, 4];
const TAX_COLLECTION: usize = 5;

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

pub(crate) mod measure;
mod system;

pub use system::{PoliticsSystem, ScoreRow, Scoreboard, score_against_history};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tax_effect_indices_match_the_catalogue() {
        use crate::politics::effect_index;
        assert_eq!(
            TAX_CAPS.map(Some),
            ["tax_cap_lower", "tax_cap_middle", "tax_cap_upper"].map(effect_index)
        );
        assert_eq!(Some(TAX_COLLECTION), effect_index("tax_collection"));
    }

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
