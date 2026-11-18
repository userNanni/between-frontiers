//! The war on land in the tick (phase 5, slice C): who controls each state, the segments of
//! every front, and a week of combat on each by `calc::land_step`. Behind `Switches::war`
//! (off). Design in `docs/war.pt-BR.md`, "A guerra no tick".
//!
//! A segment is a pair of neighbouring states held by the two sides of a war: the one held by
//! side A first. The fronts are the segments between each pair of enemy countries; they are
//! derived every week from the control and the map, never saved. What is saved is the control
//! of the states taken, the advance of each segment, the postures and the losses.

use std::collections::BTreeMap;
use std::sync::Arc;

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::calc::{LandField, LandSide, Posture, TerrainMix, apportion, land_step};
use super::catalogue::{Domain, Military, Mobility, Terrain};
use super::state::{CountryWar, men};
use crate::game::GameWorld;

/// Ratio of land fire at which the AI attacks a front (owner's decision G5: 1.5).
pub const AI_ATTACK_RATIO: (i64, i64) = (3, 2);

/// What the war step needs from the map, built at load.
#[derive(Clone, Debug)]
pub struct FightSetup {
    /// Land neighbours of each map state, by state id.
    pub neighbours: Arc<BTreeMap<u16, Vec<u16>>>,
    /// Depth of each state, km: half the side of a square of its area. The front takes the
    /// state when its advance passes it.
    pub depth_km: BTreeMap<u16, Fixed>,
}

/// The war on land, saved and hashed only when not empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fighting {
    /// (region, market of the occupier) for every region held by another than its owner, sorted
    /// by region. Occupation changes control, not the owner: the owner changes only by peace.
    pub control: Vec<(u32, u16)>,
    /// (from region, into region, km): how far the side holding `from` advanced into `into`.
    pub advance: Vec<(u32, u32, Fixed)>,
    /// (country, enemy, attack): the posture the player set on the front against `enemy`. The AI's
    /// posture is worked out each week (G5) and not saved.
    pub postures: Vec<(u16, u16, bool)>,
    /// (country, men): land casualties since its first war, killed, wounded and captured.
    pub losses: Vec<(u16, u64)>,
    /// The segments of the last week and what happened on each: what moved the front, for the
    /// war screen to read without working the week out again.
    pub last_week: Vec<SegmentWeek>,
}

impl Fighting {
    pub fn is_empty(&self) -> bool {
        self.control.is_empty()
            && self.advance.is_empty()
            && self.postures.is_empty()
            && self.losses.is_empty()
            && self.last_week.is_empty()
    }

    /// The market controlling region `r`, whose owner is `owner`.
    pub fn controller(&self, r: u32, owner: u16) -> u16 {
        self.control
            .binary_search_by_key(&r, |(k, _)| *k)
            .map_or(owner, |i| self.control[i].1)
    }

    fn set_control(&mut self, r: u32, by: u16, owner: u16) {
        match self.control.binary_search_by_key(&r, |(k, _)| *k) {
            Ok(i) if by == owner => {
                self.control.remove(i);
            }
            Ok(i) => self.control[i].1 = by,
            Err(_) if by == owner => {}
            Err(i) => self.control.insert(i, (r, by)),
        }
    }

    /// The posture `country` set against `enemy`, if any.
    pub fn posture(&self, country: u16, enemy: u16) -> Option<bool> {
        self.postures
            .iter()
            .find(|(c, e, _)| *c == country && *e == enemy)
            .map(|(_, _, a)| *a)
    }

    pub fn set_posture(&mut self, country: u16, enemy: u16, attack: bool) {
        match self
            .postures
            .iter_mut()
            .find(|(c, e, _)| *c == country && *e == enemy)
        {
            Some(p) => p.2 = attack,
            None => {
                self.postures.push((country, enemy, attack));
                self.postures.sort();
            }
        }
    }
}

/// One segment of a front this week, for the screen and the tests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentWeek {
    /// The region held by side A, and the one held by side B.
    pub from: u32,
    pub into: u32,
    /// The countries holding them.
    pub a: u16,
    pub b: u16,
    pub outcome: super::calc::LandOutcome,
}

/// What a week of fighting does, worked out in the forces system's compute.
#[derive(Clone, Debug, Default)]
pub struct FightWeek {
    pub fighting: Fighting,
    /// (country, share of its land elements lost this week).
    pub lost_share: Vec<(u16, Fixed)>,
    pub segments: Vec<SegmentWeek>,
}

/// The land strength of a country: men, the fire and protection of what it holds, and its
/// average training, over every land formation.
struct Strength {
    men: u64,
    fire: Fixed,
    protection: Fixed,
    training: Fixed,
}

fn strength(mil: &Military, c: &CountryWar) -> Strength {
    let land: Vec<_> = c
        .formations
        .iter()
        .filter(|f| mil.types[f.unit as usize].domain == Domain::Land && f.elements.is_positive())
        .collect();
    let men_total: i64 = land.iter().map(|f| men(mil, f)).sum();
    let equipment: Vec<(u16, Fixed)> = land.iter().flat_map(|f| f.equipment.clone()).collect();
    let elements: Fixed = land.iter().map(|f| f.elements).sum();
    let training = if elements.is_positive() {
        land.iter().map(|f| f.training * f.elements).sum::<Fixed>() / elements
    } else {
        Fixed::ZERO
    };
    Strength {
        men: men_total.max(0) as u64,
        fire: mil.land_fire(&equipment),
        protection: mil.protection(&equipment),
        training,
    }
}

/// A week of war on land: segments, combat, advance and control. Pure: reads the world.
pub fn fight_week(setup: &FightSetup, mil: &Military, world: &GameWorld) -> FightWeek {
    let mut out = FightWeek {
        fighting: world.war.fighting.clone(),
        ..Default::default()
    };
    let wars = &world.war.conflicts.wars;
    if wars.is_empty() {
        return out;
    }
    let regions = &world.econ.regions;
    // Map state → region.
    let by_state: BTreeMap<u16, u32> = regions
        .iter()
        .enumerate()
        .map(|(i, r)| (r.state, i as u32))
        .collect();
    let player = world
        .countries
        .iter()
        .position(|(id, _)| Some(id) == world.player)
        .map(|i| i as u16);
    let controller = |r: u32| out.fighting.controller(r, regions[r as usize].market);
    let enemies = |a: u16, b: u16| world.war.conflicts.at_war(a, b);
    // The segments: (from, into) where `from`'s holder is at war with `into`'s, each pair once,
    // the lower-indexed holder's region first.
    let mut segments: Vec<(u32, u32, u16, u16)> = Vec::new();
    for (ri, r) in regions.iter().enumerate() {
        let a = controller(ri as u32);
        let Some(near) = setup.neighbours.get(&r.state) else {
            continue;
        };
        for s in near {
            let Some(&si) = by_state.get(s) else {
                continue;
            };
            let b = controller(si);
            if a < b && enemies(a, b) {
                segments.push((ri as u32, si, a, b));
            }
        }
    }
    segments.sort();
    segments.dedup();
    if segments.is_empty() {
        out.fighting.last_week.clear();
        return out;
    }
    // Each country's land strength, spread evenly over its segments.
    let mut count: BTreeMap<u16, u64> = BTreeMap::new();
    for &(_, _, a, b) in &segments {
        *count.entry(a).or_default() += 1;
        *count.entry(b).or_default() += 1;
    }
    let strengths: BTreeMap<u16, Strength> = count
        .keys()
        .filter_map(|&c| {
            world
                .war
                .countries
                .get(c as usize)
                .map(|cw| (c, strength(mil, cw)))
        })
        .collect();
    // Posture: the player's as set (defend by default); the AI attacks a front where its fire
    // is at least 1.5 times the enemy's (G5).
    let attacks = |c: u16, e: u16| -> bool {
        if Some(c) == player {
            return out.fighting.posture(c, e).unwrap_or(false);
        }
        let (Some(me), Some(them)) = (strengths.get(&c), strengths.get(&e)) else {
            return false;
        };
        me.fire * AI_ATTACK_RATIO.1 >= them.fire * AI_ATTACK_RATIO.0 && me.fire.is_positive()
    };
    let share_of = |c: u16, k: u64| -> Vec<u64> {
        let s = &strengths[&c];
        apportion(s.men, &vec![Fixed::ONE; count[&c] as usize])
            .into_iter()
            .take(k as usize)
            .collect()
    };
    let men_on: BTreeMap<u16, Vec<u64>> =
        count.iter().map(|(&c, &n)| (c, share_of(c, n))).collect();
    let mut next: BTreeMap<u16, usize> = BTreeMap::new();
    let mut lost: BTreeMap<u16, u64> = BTreeMap::new();
    let field = LandField {
        terrain: TerrainMix::only(Terrain::Open),
        winter: false,
        river: Fixed::ZERO,
    };
    let side = |c: u16, men_here: u64, attack: bool| -> LandSide {
        let s = &strengths[&c];
        let part = if s.men > 0 {
            Fixed::from_ratio(men_here as i64, s.men as i64)
        } else {
            Fixed::ZERO
        };
        LandSide {
            men: men_here,
            fire: s.fire * part,
            protection: s.protection,
            training: s.training,
            supply: Fixed::ONE,
            // Neutral until the air enters (slice C2).
            air: Fixed::from_ratio(1, 2),
            posture: if attack {
                Posture::Attack
            } else {
                Posture::Defend
            },
            mobility: Mobility::Foot,
            fortification: Fixed::ZERO,
            landing: false,
        }
    };
    for &(from, into, a, b) in &segments {
        let ia = *next.entry(a).or_default();
        let ib = *next.entry(b).or_default();
        *next.get_mut(&a).unwrap() += 1;
        *next.get_mut(&b).unwrap() += 1;
        let ma = men_on[&a].get(ia).copied().unwrap_or(0);
        let mb = men_on[&b].get(ib).copied().unwrap_or(0);
        let (sa, sb) = (side(a, ma, attacks(a, b)), side(b, mb, attacks(b, a)));
        let o = land_step(&mil.rules, &field, [&sa, &sb]);
        *lost.entry(a).or_default() += o.losses[0];
        *lost.entry(b).or_default() += o.losses[1];
        out.segments.push(SegmentWeek {
            from,
            into,
            a,
            b,
            outcome: o,
        });
    }
    // Advance and control.
    for seg in &out.segments {
        let km = seg.outcome.advance_km;
        if km == Fixed::ZERO {
            continue;
        }
        let (from, into, taker) = if km.is_positive() {
            (seg.from, seg.into, seg.a)
        } else {
            (seg.into, seg.from, seg.b)
        };
        let km = km.abs();
        let f = &mut out.fighting;
        let i = match f
            .advance
            .binary_search_by_key(&(from, into), |(x, y, _)| (*x, *y))
        {
            Ok(i) => i,
            Err(i) => {
                f.advance.insert(i, (from, into, Fixed::ZERO));
                i
            }
        };
        f.advance[i].2 += km;
        let depth = setup
            .depth_km
            .get(&regions[into as usize].state)
            .copied()
            .unwrap_or(Fixed::from_int(100));
        if f.advance[i].2 >= depth {
            // The state is taken: every advance into it or out of it starts again.
            f.set_control(into, taker, regions[into as usize].market);
            f.advance.retain(|(x, y, _)| *x != into && *y != into);
        }
    }
    out.fighting.last_week = out.segments.clone();
    // Losses: a share of each country's land elements.
    for (c, l) in lost {
        if l == 0 {
            continue;
        }
        let total = strengths[&c].men.max(1);
        out.lost_share
            .push((c, Fixed::from_ratio(l.min(total) as i64, total as i64)));
        match out.fighting.losses.binary_search_by_key(&c, |(k, _)| *k) {
            Ok(i) => out.fighting.losses[i].1 += l,
            Err(i) => out.fighting.losses.insert(i, (c, l)),
        }
    }
    out
}

/// Takes `share` of every land formation of `c`, elements and equipment alike.
pub fn take_losses(mil: &Military, c: &mut CountryWar, share: Fixed) {
    let keep = (Fixed::ONE - share).max(Fixed::ZERO);
    for f in c
        .formations
        .iter_mut()
        .filter(|f| mil.types[f.unit as usize].domain == Domain::Land)
    {
        f.elements *= keep;
        for e in &mut f.equipment {
            e.1 *= keep;
        }
    }
    c.formations.retain(|f| f.elements.is_positive());
}

impl HashState for Fighting {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.control.len());
        for (r, c) in &self.control {
            h.write_u32(*r);
            h.write_u16(*c);
        }
        h.write_len(self.advance.len());
        for (a, b, km) in &self.advance {
            h.write_u32(*a);
            h.write_u32(*b);
            km.hash_state(h);
        }
        h.write_len(self.postures.len());
        for (c, e, a) in &self.postures {
            h.write_u16(*c);
            h.write_u16(*e);
            a.hash_state(h);
        }
        h.write_len(self.losses.len());
        for (c, l) in &self.losses {
            h.write_u16(*c);
            h.write_u64(*l);
        }
        h.write_len(self.last_week.len());
        for g in &self.last_week {
            h.write_u32(g.from);
            h.write_u32(g.into);
            h.write_u16(g.a);
            h.write_u16(g.b);
            let o = &g.outcome;
            o.engaged.hash_state(h);
            for p in o.power {
                p.hash_state(h);
            }
            for l in o.losses {
                h.write_u64(l);
            }
            o.advance_km.hash_state(h);
        }
    }
}
