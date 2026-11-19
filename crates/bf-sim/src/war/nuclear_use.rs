//! The use of the weapon (`docs/war.pt-BR.md`, "Arma nuclear na escada de crise", step 4; the
//! owner's N5, N6 and N7, decided 2026-11-19).
//!
//! - A strike launches warheads from their launchers on a country at war with the user:
//!   counterforce (its forces, few civilians) or countervalue (its cities). The target's missile
//!   defence stops its share; each warhead that arrives kills and wrecks.
//! - The dead weigh on the target's support for the war (the casualties term); on the user's own,
//!   nothing direct (N6). The user's security reputation takes a stain.
//! - N5, proportional retaliation: the AI answers the next week on the same rung, with as many
//!   warheads as it received; countervalue in full only when it has no land forces left to answer
//!   otherwise. An answer is not answered again.
//! - N7, the taboo as a stock: it grows each year without use while any warhead exists, and falls
//!   at every use, for all. The AI uses first only when occupied beyond a share that rises with
//!   the taboo.

use laplace_schelling::{Date, Fixed};
use serde::{Deserialize, Serialize};

use super::catalogue::{Domain, Military};
use super::state::CountryWar;
use crate::game::GameWorld;

/// What a strike aims at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Mode {
    /// The forces: the land formations of the target, with few civilians.
    Counterforce,
    /// The cities: the target's most populous home regions.
    Countervalue,
}

/// A use asked for (a command, or the AI): `warheads` from `by` on `target`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Use {
    pub by: u16,
    pub target: u16,
    pub mode: Mode,
    pub warheads: u32,
    /// An answer (N5): it is not answered again.
    pub answer: bool,
}

/// What a strike did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Strike {
    pub launched: u32,
    pub delivered: u32,
    pub killed: u64,
    /// Plant levels destroyed.
    pub wrecked: i64,
    /// Men of the target's land forces put out of the war.
    pub men: i64,
    /// The region each warhead that arrived fell on (N8: where the fallout starts).
    pub hit: Vec<u32>,
}

/// The warheads a country holds on launchers, ready to go.
pub fn deliverable(mil: &Military, c: &CountryWar) -> u64 {
    let (forces, _) = super::balance::forces_of(mil, c, 1000);
    forces
        .iter()
        .map(|f| f.launchers * u64::from(f.warheads_per_launcher))
        .sum()
}

/// N7: the taboo's threshold for a first use, per mille of the home people under the enemy.
pub fn first_use_threshold(first_use_occupied: u32, taboo: u32) -> u32 {
    let base = first_use_occupied.min(1000);
    base + (1000 - base) * taboo.min(1000) / 1000
}

/// The share of `country`'s home people under the enemy, per mille.
pub fn occupied(world: &GameWorld, country: u16) -> u32 {
    let (mut size, mut taken) = (0i64, 0i64);
    for (ri, r) in world.econ.regions.iter().enumerate() {
        if r.market != country || r.colony {
            continue;
        }
        let p = r.population();
        size += p;
        if world.war.fighting.controller(ri as u32, country) != country {
            taken += p;
        }
    }
    if size > 0 {
        (taken * 1000 / size) as u32
    } else {
        0
    }
}

/// Removes `n` warheads from `c`'s formations, from the last formation back.
/// The equivalent megatons per mille of each warhead spent (U3).
fn spend_warheads(mil: &Military, c: &mut CountryWar, mut n: u32) -> Vec<i64> {
    let Some(ty) = mil.unit("nuclear_warhead") else {
        return Vec::new();
    };
    let mut spent = Vec::new();
    for f in c.formations.iter_mut().rev().filter(|f| f.unit == ty) {
        if n == 0 {
            break;
        }
        let take = f.elements.to_int_floor().clamp(0, i64::from(n));
        f.elements -= Fixed::from_int(take);
        n -= take as u32;
        let emt = mil.spec(ty, usize::from(f.generation)).emt_mille();
        spent.extend(std::iter::repeat_n(emt, take as usize));
    }
    c.formations
        .retain(|f| f.unit != ty || f.elements > Fixed::ZERO);
    spent
}

/// The men of `c`'s land formations.
pub fn land_men(mil: &Military, c: &CountryWar) -> i64 {
    c.formations
        .iter()
        .filter(|f| mil.types[f.unit as usize].domain == Domain::Land)
        .map(|f| super::state::men(mil, f))
        .sum()
}

/// Puts `men` of `c`'s land formations out of the war, from every formation in proportion.
fn hit_land(mil: &Military, c: &mut CountryWar, men: i64) -> i64 {
    let all = land_men(mil, c);
    if all <= 0 || men <= 0 {
        return 0;
    }
    let share = Fixed::from_ratio(men.min(all), all);
    let mut out = 0;
    for f in c
        .formations
        .iter_mut()
        .filter(|f| mil.types[f.unit as usize].domain == Domain::Land)
    {
        let before = super::state::men(mil, f);
        f.elements -= f.elements * share;
        out += before - super::state::men(mil, f);
    }
    out
}

/// One strike: `u.by` launches on `u.target`. `None` when they are not at war or nothing can be
/// launched.
pub fn strike(mil: &Military, world: &mut GameWorld, u: &Use, date: Date) -> Option<Strike> {
    let n = &mil.rules.nuclear;
    if u.by == u.target || !world.war.conflicts.at_war(u.by, u.target) {
        return None;
    }
    let by = usize::from(u.by);
    let launched =
        (u64::from(u.warheads)).min(deliverable(mil, world.war.countries.get(by)?)) as u32;
    if launched == 0 {
        return None;
    }
    let spent = spend_warheads(mil, &mut world.war.countries[by], launched);
    let stopped = world
        .war
        .countries
        .get(usize::from(u.target))
        .map_or(Fixed::ZERO, |t| {
            super::balance::defense_of(mil, t, u64::from(launched))
        });
    let delivered =
        launched - (Fixed::from_int(i64::from(launched)) * stopped).to_int_floor() as u32;
    let mut out = Strike {
        launched,
        delivered,
        ..Default::default()
    };
    // The target's home regions, the most populous first.
    let mut regions: Vec<(i64, u32)> = world
        .econ
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| r.market == u.target && !r.colony)
        .map(|(i, r)| (r.population(), i as u32))
        .filter(|(p, _)| *p > 0)
        .collect();
    regions.sort_unstable_by(|a, b| b.cmp(a));
    // U3: a warhead kills `killed_per_warhead` an equivalent megaton of its power.
    let per_emt = u64::from(n.killed_per_warhead);
    let per_emt = match u.mode {
        Mode::Countervalue => per_emt,
        Mode::Counterforce => per_emt * u64::from(n.counterforce_civilians) / 1000,
    };
    for k in 0..delivered as usize {
        let per = per_emt * spent.get(k).copied().unwrap_or(1000).max(0) as u64 / 1000;
        let Some(&(_, ri)) = regions.get(k % regions.len().max(1)) else {
            break;
        };
        let pop = world.econ.regions[ri as usize].population();
        if pop <= 0 {
            continue;
        }
        let dead = per.min(pop as u64);
        if u.mode == Mode::Countervalue {
            let levels: i64 = world.econ.regions[ri as usize]
                .plants
                .iter()
                .map(|p| p.levels)
                .sum();
            let wreck = (i128::from(levels) * i128::from(dead) * i128::from(n.wreck_factor)
                / (i128::from(pop) * 1000))
                .min(i128::from(levels)) as i64;
            super::occupation::wreck(world, ri, wreck);
            out.wrecked += wreck;
        }
        super::occupation::kill_civilians(world, ri, dead);
        out.killed += dead;
        out.hit.push(ri);
    }
    if u.mode == Mode::Counterforce
        && let Some(t) = world.war.countries.get_mut(usize::from(u.target))
    {
        out.men = hit_land(mil, t, i64::from(n.counterforce_men) * i64::from(delivered));
    }
    let nuc = &mut world.war.nuclear;
    nuc.add_dead(u.target, out.killed);
    // A use is a test: the arsenal is known to all.
    if let Err(i) = nuc.tested.binary_search(&u.by) {
        nuc.tested.insert(i, u.by);
    }
    nuc.taboo = nuc.taboo * (1000 - n.taboo_cut.min(1000)) / 1000;
    nuc.last_use = Some((date.year, date.month, date.day));
    // The stain on the user's security reputation: years of its forces' cost.
    if world.treaties.reputation_on
        && let Some(tag) = world.countries.iter().nth(by).map(|(_, c)| c.tag.clone())
    {
        let stain =
            super::third::forces_cost_year(world, u.by).mul_ratio(i64::from(n.use_stain), 1000);
        world.treaties.reputation.entry(tag).or_default().security += stain.max(Fixed::ZERO);
    }
    Some(out)
}

/// N5: the answer of the target of `u`, which received `s`: the same rung, as many warheads as
/// it received; countervalue in full when it has no land forces left. `None` for an answer, a
/// player's target, or a target with nothing to launch.
pub fn answer(
    mil: &Military,
    world: &GameWorld,
    u: &Use,
    s: &Strike,
    player: Option<u16>,
) -> Option<Use> {
    if u.answer || Some(u.target) == player {
        return None;
    }
    let t = world.war.countries.get(usize::from(u.target))?;
    let can = deliverable(mil, t);
    if can == 0 {
        return None;
    }
    let (mode, warheads) = if land_men(mil, t) <= 0 {
        (Mode::Countervalue, can as u32)
    } else {
        (u.mode, s.launched.min(can as u32))
    };
    Some(Use {
        by: u.target,
        target: u.by,
        mode,
        warheads,
        answer: true,
    })
}

/// N7: the AI's first uses this week. A country attacked (the defending side of a war) that is
/// not played, holds warheads on launchers and has its home occupied beyond the taboo's
/// threshold strikes its attacker's forces with all it can launch.
pub fn first_uses(mil: &Military, world: &GameWorld, player: Option<u16>) -> Vec<Use> {
    let n = &mil.rules.nuclear;
    let line = first_use_threshold(n.first_use_occupied, world.war.nuclear.taboo);
    let mut out = Vec::new();
    for w in &world.war.conflicts.wars {
        let Some(&attacker) = w.side_a.first() else {
            continue;
        };
        for &m in &w.side_b {
            if Some(m) == player || out.iter().any(|u: &Use| u.by == m) {
                continue;
            }
            let Some(c) = world.war.countries.get(usize::from(m)) else {
                continue;
            };
            let can = deliverable(mil, c);
            if can == 0 || occupied(world, m) < line {
                continue;
            }
            out.push(Use {
                by: m,
                target: attacker,
                mode: Mode::Counterforce,
                warheads: can as u32,
                answer: false,
            });
        }
    }
    out
}

/// N7: a year passes. With any warhead in the world and no use in the year, the taboo grows by
/// a `years`-th.
pub fn taboo_year(mil: &Military, world: &mut GameWorld, year: i32) {
    let n = &mil.rules.nuclear;
    let Some(ty) = mil.unit("nuclear_warhead") else {
        return;
    };
    let any = world.war.countries.iter().any(|c| {
        c.formations
            .iter()
            .any(|f| f.unit == ty && f.elements >= Fixed::ONE)
    });
    let nuc = &mut world.war.nuclear;
    let used = nuc.last_use.is_some_and(|(y, _, _)| y >= year - 1);
    if any && !used {
        nuc.taboo = (nuc.taboo + 1000 / n.taboo_years.max(1)).min(1000);
    }
}

/// N8: the fallout of a strike. Each warhead that arrived takes `fallout_loss` of the
/// productivity of the region it fell on, and `fallout_downwind` of that of the regions of the
/// neighbouring states (no winds: all neighbours, as a stand-in for the side the wind blows to).
/// A loss compounds on what is left; the stock is kept by region.
pub fn fallout(
    mil: &Military,
    world: &mut GameWorld,
    neighbours: &std::collections::BTreeMap<u16, Vec<u16>>,
    hit: &[u32],
) {
    let n = &mil.rules.nuclear;
    let mut adds: Vec<(u32, u32)> = Vec::new();
    for &ri in hit {
        adds.push((ri, n.fallout_loss));
        let Some(r) = world.econ.regions.get(ri as usize) else {
            continue;
        };
        let near = neighbours.get(&r.state).cloned().unwrap_or_default();
        for (rj, other) in world.econ.regions.iter().enumerate() {
            if near.contains(&other.state) {
                adds.push((rj as u32, n.fallout_loss * n.fallout_downwind / 1000));
            }
        }
    }
    for (ri, add) in adds {
        let old = world.war.nuclear.fallout_of(ri);
        let new = (old + add * (1000 - old) / 1000).min(n.fallout_cap);
        set_fallout(world, ri, old, new);
    }
}

/// Moves `region`'s fallout from `old` to `new` (per mille) and its productivity with it: the
/// productivity left is `1 - fallout` of what it would be without it.
fn set_fallout(world: &mut GameWorld, region: u32, old: u32, new: u32) {
    if old == new {
        return;
    }
    if let Some(r) = world.econ.regions.get_mut(region as usize) {
        r.tfp = r
            .tfp
            .mul_ratio(i64::from(1000 - new), i64::from((1000 - old).max(1)));
    }
    world.war.nuclear.set_fallout(region, new);
}

/// N8: a year passes. The fallout of every region decays by `fallout_keep` (the half-life of
/// caesium-137), and the productivity comes back with it; under 1 per mille it is gone.
pub fn fallout_year(mil: &Military, world: &mut GameWorld) {
    let keep = mil.rules.nuclear.fallout_keep;
    let list = world.war.nuclear.fallout.clone();
    for (ri, old) in list {
        let new = old * keep / 1000;
        set_fallout(world, ri, old, if new < 1 { 0 } else { new });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_threshold_rises_with_the_taboo() {
        assert_eq!(first_use_threshold(300, 0), 300);
        assert_eq!(first_use_threshold(300, 500), 650);
        assert_eq!(first_use_threshold(300, 1000), 1000);
    }
}
