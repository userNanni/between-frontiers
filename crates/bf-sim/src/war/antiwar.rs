//! The movement against the war (the war's missing functions, after scorched earth; the
//! orchestrator's design of 2026-11-24, the owner's AW1 to AW5 pending, drafted with the
//! recommended ones), behind the switch `anti_war` (off).
//!
//! - AW1 a: the movement `anti_war` (`data/politics/movements.ron`) draws on workers, peasants
//!   and soldiers; its cause is the war weariness of those it recruits, 1000 less the war
//!   support (G4) of their interest groups, at least `movement.cause`'s bound, at war only.
//! - AW2 a: it moves every week of war, the yearly `growth` over 52, and fades by its
//!   `persistence` a week at a time in peace.
//! - AW3 a: the politics counts its mobilization in the legitimacy like any movement; the war
//!   industry loses the workers on strike (the staffing of `military_industry` plants, at the
//!   monthly labour), and the land army loses the fire of the soldiers in mutiny. The men on
//!   strike stay in the population and the consumption; the men in mutiny stay in their
//!   formations.
//! - AW4 a: it does not change a law or force the peace; with its mobilization at
//!   `collapse_mobilized` or more, the countdown to the fall of the government runs below the
//!   peace threshold, not only below the collapse.
//! - AW5 a: what is left of it when a war starts lowers the starting support, by `memory`.
//!
//! The support is kept in units of 1/100,000 so that a week's growth is not lost to rounding;
//! the politics shows it in per mille.

use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use crate::game::GameWorld;
use crate::politics::{Bound, Politics};

/// One per mille in the units of [`AntiWar::support`].
pub const UNIT: u32 = 100;

/// `data/military/war.ron`, `conduct.anti_war`; `None` with the switch off.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AntiWarRules {
    /// The movement's key in `movements.ron`.
    pub movement: String,
    /// Of the workers who back the movement, those on strike at a time, per mille. DESIGN VALUE.
    pub strike: u32,
    /// Of the soldiers who back the movement, those who refuse to fight, per mille. DESIGN
    /// VALUE.
    pub mutiny: u32,
    /// Mobilization (per mille of the people) from which the countdown to the fall runs below
    /// the peace threshold. DESIGN VALUE (the 4 % at which a movement moves a law).
    pub collapse_mobilized: u32,
    /// Per mille of the support left at the start of a war taken off the starting war support.
    /// DESIGN VALUE.
    pub memory: u32,
    /// Professions that strike (`economy::PROFESSIONS` keys) and that mutiny.
    pub strikers: Vec<String>,
    pub soldiers: Vec<String>,
    /// Resolved at load: the movement's index, and the profession indices.
    #[serde(skip)]
    pub kind: Option<usize>,
    #[serde(skip)]
    pub striker_ix: Vec<usize>,
    #[serde(skip)]
    pub soldier_ix: Vec<usize>,
}

/// A country's movement against the war.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AntiWar {
    pub country: u16,
    /// Of the people it recruits who back it, in 1/100,000.
    pub support: u32,
    /// How far its cause holds, per mille; the war weariness of its recruits.
    pub pressure: u16,
    pub weariness: u16,
    /// Mobilized, per mille of the people.
    pub mobilized: u16,
    /// Of the strikers' jobs in the war industry stopped, and of the land fire lost, per mille.
    pub strike: u16,
    pub mutiny: u16,
    /// The support it had (per mille) when the country's present war began; 0 in peace.
    pub memory: u16,
}

impl AntiWar {
    pub fn per_mille(&self) -> u16 {
        (self.support / UNIT).min(1000) as u16
    }
}

/// The movements, by country; empty with the switch off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AntiWarState {
    pub countries: Vec<AntiWar>,
}

impl AntiWarState {
    pub fn is_empty(&self) -> bool {
        self.countries.is_empty()
    }

    pub fn of(&self, country: u16) -> Option<&AntiWar> {
        self.countries
            .binary_search_by_key(&country, |a| a.country)
            .ok()
            .map(|i| &self.countries[i])
    }

    /// Per mille of the war industry's jobs on strike in `country`.
    pub fn strike(&self, country: u16) -> u16 {
        self.of(country).map_or(0, |a| a.strike)
    }

    /// Per mille of the land fire lost to mutiny in `country`.
    pub fn mutiny(&self, country: u16) -> u16 {
        self.of(country).map_or(0, |a| a.mutiny)
    }
}

impl HashState for AntiWarState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.countries.len());
        for a in &self.countries {
            h.write_u16(a.country);
            h.write_u32(a.support);
            h.write_u16(a.pressure);
            h.write_u16(a.weariness);
            h.write_u16(a.mobilized);
            h.write_u16(a.strike);
            h.write_u16(a.mutiny);
            h.write_u16(a.memory);
        }
    }
}

/// Per mille of the professions' people in each interest group's support: the support of
/// profession `p` is the mean of its groups' support, weighted by the share of `p` each holds.
fn by_profession(p: &Politics, groups: &[i32]) -> Vec<Option<i32>> {
    let n = crate::econ::prof::P;
    let mut num = vec![0i64; n];
    let mut den = vec![0i64; n];
    for (g, ig) in p.interest_groups.iter().enumerate() {
        if ig.community.is_some() {
            continue;
        }
        let Some(&s) = groups.get(g) else { continue };
        for &(q, share) in &ig.professions {
            num[usize::from(q)] += i64::from(s) * i64::from(share);
            den[usize::from(q)] += i64::from(share);
        }
    }
    (0..n)
        .map(|q| (den[q] > 0).then(|| (num[q] / den[q]) as i32))
        .collect()
}

/// The week: each country's movement grows while its war wears its recruits past the bound,
/// fades out of war, and sets what strikes and mutinies. Reads the war support of last week.
pub fn week(rules: &AntiWarRules, p: &Politics, world: &GameWorld) -> AntiWarState {
    let Some(k) = rules.kind else {
        return AntiWarState::default();
    };
    let kind = &p.movements[k];
    let bound = kind
        .cause
        .iter()
        .find_map(|(_, b)| match b {
            Bound::AtLeast(t) => Some(*t),
            Bound::AtMost(_) => None,
        })
        .unwrap_or(600);
    let n = crate::econ::prof::P;
    let mut row = vec![0i64; n];
    for &(q, share) in &kind.professions {
        row[usize::from(q)] = i64::from(share);
    }
    if kind.professions.is_empty() {
        row.fill(1000);
    }
    let countries = world.countries.len();
    // People by country and profession, the metropole's.
    let mut size = vec![vec![0i64; n]; countries];
    for reg in world.econ.regions.iter().filter(|r| !r.colony) {
        let m = usize::from(reg.market);
        if m >= countries {
            continue;
        }
        for (q, pop) in reg.pops.iter().enumerate().take(n) {
            size[m][q] += pop.size.max(0);
        }
    }
    let at_war = |c: u16| {
        world
            .war
            .conflicts
            .wars
            .iter()
            .any(|w| w.side_a.contains(&c) || w.side_b.contains(&c))
    };
    let mut out = Vec::new();
    for (c, size) in size.iter().enumerate() {
        let m = c as u16;
        let old = world.war.antiwar.of(m).cloned().unwrap_or(AntiWar {
            country: m,
            ..Default::default()
        });
        let war = at_war(m);
        let support = world.war.fighting.support.iter().find(|s| s.country == m);
        // The weariness of the recruits: 1000 less their groups' war support.
        let weariness = support.map_or(0, |s| {
            let prof = by_profession(p, &s.groups);
            let (mut num, mut den) = (0i64, 0i64);
            for q in 0..n {
                let w = row[q] * size[q];
                if w > 0 {
                    num += w * i64::from(1000 - prof[q].unwrap_or(s.value).clamp(0, 1000));
                    den += w;
                }
            }
            if den > 0 { (num / den) as i32 } else { 0 }
        });
        let pressure = if war {
            crate::politics::margin(Bound::AtLeast(bound), weariness, (0, 1000))
        } else {
            0
        };
        let mut a = AntiWar {
            country: m,
            support: old.support,
            pressure: pressure as u16,
            weariness: weariness.clamp(0, 1000) as u16,
            memory: if !war {
                0
            } else if support.is_none() {
                // The first week of the war: what is left of the movement is remembered.
                old.per_mille()
            } else {
                old.memory
            },
            ..Default::default()
        };
        if pressure > 0 {
            // `growth` per mille a year at full pressure, a 52nd each week.
            let add = u64::from(kind.growth) * u64::from(UNIT) * pressure as u64 / 1000 / 52;
            a.support = (u64::from(a.support) + add.max(1)).min(1000 * u64::from(UNIT)) as u32;
        } else if a.support > 0 && !war {
            // Halves every `persistence` years, a week at a time (ln 2 ≈ 0.693).
            let cut = u64::from(a.support) * 693 / (1000 * 52 * u64::from(kind.persistence.max(1)));
            a.support = a.support.saturating_sub(cut.max(1) as u32);
        }
        if a.support == 0 && !war {
            continue;
        }
        let people: i64 = size.iter().sum();
        let recruits: i64 = (0..n).map(|q| row[q] * size[q] / 1000).sum();
        let backing = i64::from(a.per_mille());
        if war && people > 0 {
            a.mobilized = (backing * recruits / people).clamp(0, 1000) as u16;
            let share = |ix: &[usize]| -> i64 {
                // Of those professions, the part the movement recruits and that backs it.
                let (mut back, mut all) = (0i64, 0i64);
                for &q in ix {
                    back += row[q] * size[q] / 1000 * backing / 1000;
                    all += size[q];
                }
                if all > 0 { back * 1000 / all } else { 0 }
            };
            a.strike =
                (share(&rules.striker_ix) * i64::from(rules.strike) / 1000).clamp(0, 1000) as u16;
            a.mutiny =
                (share(&rules.soldier_ix) * i64::from(rules.mutiny) / 1000).clamp(0, 1000) as u16;
        }
        out.push(a);
    }
    AntiWarState { countries: out }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_mille_reads_the_fine_units() {
        let a = AntiWar {
            support: 12_345,
            ..Default::default()
        };
        assert_eq!(a.per_mille(), 123);
    }
}
