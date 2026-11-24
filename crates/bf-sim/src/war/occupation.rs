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

/// (occupier, men needed, men held) for every occupier: the garrison it keeps. Its garrison
/// formations stand first (item 16); what they miss comes out of its field land men, at most
/// `garrison_cap` of them. `land_men(c)` gives (garrison men, field men). Read before the
/// combat: the men held do not fight.
pub fn garrisons(
    r: &OccupationRules,
    world: &GameWorld,
    f: &Fighting,
    land_men: impl Fn(u16) -> (u64, u64),
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
            let (posts, field) = land_men(by);
            let cap = field * r.garrison_cap.max(0) as u64 / 1000;
            (by, (n, n.min(posts + cap)))
        })
        .collect()
}

/// The order of garrison formations of country `ci` (item 16): the men its garrison misses in
/// the regions it has held for `garrison_order_weeks` or more (by last week's garrison share),
/// one element of `garrison_unit` for each element's men missing; `None` without a shortfall,
/// without the unit, or with an order of it already open. The caller leaves the player out.
pub fn garrison_order(
    r: &OccupationRules,
    mil: &super::catalogue::Military,
    world: &GameWorld,
    ci: usize,
    c: &super::state::CountryWar,
) -> Option<super::state::ForceOrder> {
    let unit = r.garrison_unit.as_deref()?;
    let ty = mil.unit(unit)?;
    if c.queue
        .iter()
        .any(|o| o.unit == unit && o.ordered > o.delivered)
    {
        return None;
    }
    let short: u64 = world
        .war
        .fighting
        .occupied
        .iter()
        .filter(|o| usize::from(o.by) == ci && o.weeks >= r.garrison_order_weeks)
        .map(|o| {
            let people = world.econ.regions[o.region as usize].population();
            let need = garrison_need(r, people, o.resistance);
            need * (1000 - o.garrison.clamp(0, 1000)) as u64 / 1000
        })
        .sum();
    let per = u64::from(mil.spec(ty, 0).men.max(1));
    let n = short.div_ceil(per);
    (n > 0).then(|| super::state::ForceOrder {
        unit: unit.to_string(),
        generation: None,
        colonial: false,
        ordered: Fixed::from_int(n as i64),
        delivered: Fixed::ZERO,
        built: Vec::new(),
        buying: Vec::new(),
        stalled: 0,
    })
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
    fn garrison_formations_stand_before_the_field_army() {
        let r = rules();
        // 40 million calm people need 80,000 men; the field cap is half the field men.
        let need = garrison_need(&r, 40_000_000, 0);
        let held = |posts: u64, field: u64| {
            let cap = field * r.garrison_cap.max(0) as u64 / 1000;
            need.min(posts + cap)
        };
        assert_eq!(need, 80_000);
        // Without garrison formations, as before: half the field men at most.
        assert_eq!(held(0, 100_000), 50_000);
        // With 50,000 men in garrisons the field gives the other 30,000.
        assert_eq!(held(50_000, 100_000), 80_000);
        // Enough garrisons: the field gives nothing.
        assert_eq!(held(100_000, 100_000), 80_000);
    }

    #[test]
    fn the_garrison_grows_with_the_resistance() {
        let r = rules();
        assert_eq!(garrison_need(&r, 40_000_000, 0), 80_000);
        assert_eq!(garrison_need(&r, 40_000_000, 1000), 800_000);
    }
}
