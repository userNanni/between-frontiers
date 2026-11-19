//! The occupation (owner's decision G3): what the occupier takes of the occupied land, the
//! garrison it has to keep there, the resistance, and the devastation the combat leaves behind.
//! Behind `Switches::war`; the numbers are in `data/military/war.ron` (`OccupationRules`).
//! Design in `docs/war.pt-BR.md`, "Ocupação".

use std::collections::BTreeMap;

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::fight::{Fighting, SegmentWeek};
use super::support::OccupationRules;
use crate::game::GameWorld;

/// A region under occupation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Occupied {
    pub region: u32,
    pub by: u16,
    pub weeks: u32,
    /// Per mille: how hostile the people are; the garrison they need grows with it.
    pub resistance: i32,
    /// Per mille of the garrison needed that the occupier kept there last week.
    pub garrison: i32,
    /// What the occupier took last week.
    pub taken: Fixed,
}

/// The devastation of a region: the share of its plant levels destroyed so far, the levels owed
/// (a fraction waiting to make a whole level) and the civilians killed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wreck {
    pub region: u32,
    pub share: i32,
    pub owed: Fixed,
    pub civilians: u64,
}

impl HashState for Occupied {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u32(self.region);
        h.write_u16(self.by);
        h.write_u32(self.weeks);
        h.write_i64(i64::from(self.resistance));
        h.write_i64(i64::from(self.garrison));
        self.taken.hash_state(h);
    }
}

impl HashState for Wreck {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u32(self.region);
        h.write_i64(i64::from(self.share));
        self.owed.hash_state(h);
        h.write_u64(self.civilians);
    }
}

/// Garrison men a region of `people` needs at `resistance` per mille (Quinlivan, 1995).
pub fn garrison_need(r: &OccupationRules, people: i64, resistance: i32) -> u64 {
    let per_mille = i64::from(r.garrison_calm)
        + i64::from(r.garrison_hostile - r.garrison_calm) * i64::from(resistance.clamp(0, 1000))
            / 1000;
    (people.max(0) * per_mille / 1000).max(0) as u64
}

/// Share of the GDP taken after `weeks` of occupation, per mille, before the garrison.
pub fn extraction(r: &OccupationRules, weeks: u32) -> i32 {
    let w = i64::from(weeks.min(r.extraction_weeks));
    let span = i64::from(r.extraction_weeks.max(1));
    (i64::from(r.extraction_start) + i64::from(r.extraction_cap - r.extraction_start) * w / span)
        as i32
}

/// (occupier, men needed, men held) for every occupier: the garrison it keeps, out of its land
/// men, at most `garrison_cap` of them. Read before the combat: the men held do not fight.
pub fn garrisons(
    r: &OccupationRules,
    world: &GameWorld,
    f: &Fighting,
    land_men: impl Fn(u16) -> u64,
) -> BTreeMap<u16, (u64, u64)> {
    let mut need: BTreeMap<u16, u64> = BTreeMap::new();
    for &(region, by) in &f.control {
        let people = world.econ.regions[region as usize].population();
        let resistance = f
            .occupied
            .binary_search_by_key(&region, |o| o.region)
            .map_or(r.resistance_start, |i| f.occupied[i].resistance);
        *need.entry(by).or_default() += garrison_need(r, people, resistance);
    }
    need.into_iter()
        .map(|(by, n)| {
            let cap = land_men(by) * r.garrison_cap.max(0) as u64 / 1000;
            (by, (n, n.min(cap)))
        })
        .collect()
}

/// What a week of occupation moves: (from the owner, to the occupier, money).
pub type Transfer = (u16, u16, Fixed);

/// A week of occupation: the occupied regions follow the control, the garrison's share sets
/// what is taken, and the resistance grows with what is taken and the garrison missing.
pub fn occupation_week(
    r: &OccupationRules,
    world: &GameWorld,
    f: &mut Fighting,
    garrisons: &BTreeMap<u16, (u64, u64)>,
) -> Vec<Transfer> {
    let regions = &world.econ.regions;
    let control = f.control.clone();
    f.occupied
        .retain(|o| control.iter().any(|&(k, by)| k == o.region && by == o.by));
    for &(region, by) in &control {
        if let Err(i) = f.occupied.binary_search_by_key(&region, |o| o.region) {
            f.occupied.insert(
                i,
                Occupied {
                    region,
                    by,
                    weeks: 0,
                    resistance: r.resistance_start,
                    garrison: 1000,
                    taken: Fixed::ZERO,
                },
            );
        }
    }
    // The income of every market and of every occupied region: the region's share of its
    // owner's GDP.
    let mut income: BTreeMap<u16, Fixed> = BTreeMap::new();
    for o in &f.occupied {
        let m = regions[o.region as usize].market;
        income.entry(m).or_insert_with(|| {
            regions
                .iter()
                .filter(|x| x.market == m)
                .flat_map(|x| x.pops.iter())
                .map(|p| p.income * Fixed::from_int(p.size))
                .sum()
        });
    }
    let mut out = Vec::new();
    for o in &mut f.occupied {
        o.weeks += 1;
        let (need, held) = garrisons.get(&o.by).copied().unwrap_or((0, 0));
        o.garrison = (held * 1000).checked_div(need).map_or(1000, |g| g as i32);
        let share = i64::from(extraction(r, o.weeks)) * i64::from(o.garrison) / 1000;
        let reg = &regions[o.region as usize];
        let m = reg.market;
        let mine: Fixed = reg
            .pops
            .iter()
            .map(|p| p.income * Fixed::from_int(p.size))
            .sum();
        let total = income.get(&m).copied().unwrap_or(Fixed::ZERO);
        let gdp = world
            .econ
            .markets
            .get(usize::from(m))
            .map_or(Fixed::ZERO, |k| k.gdp_last_year);
        o.taken = if total.is_positive() {
            gdp / Fixed::from_int(52) * (mine / total) * Fixed::from_ratio(share, 1000)
        } else {
            Fixed::ZERO
        };
        if o.taken.is_positive() {
            out.push((m, o.by, o.taken));
        }
        let shortfall = 1000 - i64::from(o.garrison.clamp(0, 1000));
        let grow = i64::from(r.resistance_per_extraction) * share / 1000
            + i64::from(r.resistance_per_shortfall) * shortfall / 1000
            - i64::from(r.resistance_calm) * i64::from(o.garrison.clamp(0, 1000)) / 1000;
        o.resistance = (i64::from(o.resistance) + grow).clamp(0, 1000) as i32;
    }
    out
}

/// What the combat destroyed this week: (region, plant levels) and (region, civilians killed).
#[derive(Clone, Debug, Default)]
pub struct Devastation {
    pub levels: Vec<(u32, i64)>,
    pub civilians: Vec<(u32, u64)>,
}

/// A week of devastation: each engaged segment destroys a share of the plant levels of its two
/// states, by the weight of its casualties, up to `devastation_cap`, and kills civilians.
pub fn devastation_week(
    r: &OccupationRules,
    world: &GameWorld,
    f: &mut Fighting,
    segments: &[SegmentWeek],
) -> Devastation {
    let mut out = Devastation::default();
    let mut hit: BTreeMap<u32, i64> = BTreeMap::new();
    for g in segments.iter().filter(|g| g.outcome.engaged) {
        let losses: u64 = g.outcome.losses.iter().sum();
        let intensity = (losses * 1000 / r.devastation_full_losses.max(1)).min(1000) as i64;
        for region in [g.from, g.into] {
            let e = hit.entry(region).or_default();
            *e = (*e).max(intensity);
        }
    }
    for (region, intensity) in hit {
        let reg = &world.econ.regions[region as usize];
        let i = match f.wrecks.binary_search_by_key(&region, |w| w.region) {
            Ok(i) => i,
            Err(i) => {
                f.wrecks.insert(
                    i,
                    Wreck {
                        region,
                        share: 0,
                        owed: Fixed::ZERO,
                        civilians: 0,
                    },
                );
                i
            }
        };
        let w = &mut f.wrecks[i];
        let add = (i64::from(r.devastation_per_week) * intensity / 1000)
            .min(i64::from(r.devastation_cap - w.share).max(0));
        if add > 0 {
            let levels: i64 = reg.plants.iter().map(|p| p.levels).sum();
            w.share += add as i32;
            w.owed += Fixed::from_int(levels) * Fixed::from_ratio(add, 1000);
            let whole = w.owed.to_int_trunc();
            if whole > 0 {
                w.owed -= Fixed::from_int(whole);
                out.levels.push((region, whole));
            }
        }
        let killed = (i128::from(reg.population().max(0))
            * i128::from(r.civilians_per_million)
            * i128::from(intensity)
            / 1_000_000_000) as u64;
        if killed > 0 {
            w.civilians += killed;
            out.civilians.push((region, killed));
        }
    }
    out
}

/// Destroys `levels` plant levels in `region`, from the plant with the most levels down.
pub fn wreck(world: &mut GameWorld, region: u32, mut levels: i64) {
    let plants = &mut world.econ.regions[region as usize].plants;
    while levels > 0 {
        let Some(p) = plants
            .iter_mut()
            .filter(|p| p.levels > 0)
            .max_by_key(|p| (p.levels, std::cmp::Reverse(p.kind)))
        else {
            break;
        };
        p.levels -= 1;
        levels -= 1;
    }
}

/// Kills `n` civilians in `region`: from every pop and every age group, in proportion.
pub fn kill_civilians(world: &mut GameWorld, region: u32, n: u64) {
    let ri = region as usize;
    let reg = &mut world.econ.regions[ri];
    let total = reg.population();
    if total <= 0 {
        return;
    }
    let n = (n as i64).min(total);
    let sizes: Vec<Fixed> = reg.pops.iter().map(|p| Fixed::from_int(p.size)).collect();
    for (p, d) in reg
        .pops
        .iter_mut()
        .zip(super::calc::apportion(n as u64, &sizes))
    {
        p.size = (p.size - d as i64).max(0);
    }
    if let Some(ages) = world.demography.ages.get_mut(ri) {
        let all: i64 = ages.iter().sum();
        if all > 0 {
            let weights: Vec<Fixed> = ages.iter().map(|&a| Fixed::from_int(a)).collect();
            for (a, d) in ages
                .iter_mut()
                .zip(super::calc::apportion(n.min(all) as u64, &weights))
            {
                *a = (*a - d as i64).max(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> OccupationRules {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/military/war.ron"
        ))
        .unwrap();
        ron::from_str::<super::super::support::ConductFile>(&text)
            .unwrap()
            .occupation
    }

    /// Occupied France: a fifth in the first year rising to about a third (1943).
    #[test]
    fn the_share_taken_rises_to_a_third() {
        let r = rules();
        assert_eq!(extraction(&r, 0), 200);
        assert!(extraction(&r, 52) > 200 && extraction(&r, 52) < 330);
        assert_eq!(extraction(&r, 156), 330);
        assert_eq!(extraction(&r, 1000), 330);
    }

    /// Quinlivan: some 2 per thousand for a calm people, 20 for a hostile one.
    #[test]
    fn the_garrison_grows_with_the_resistance() {
        let r = rules();
        assert_eq!(garrison_need(&r, 40_000_000, 0), 80_000);
        assert_eq!(garrison_need(&r, 40_000_000, 1000), 800_000);
    }
}
