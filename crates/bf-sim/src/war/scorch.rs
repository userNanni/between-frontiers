//! Scorched earth (the war's missing functions, after the reserve by cohort; the orchestrator's
//! design of 2026-11-24, the owner's TE1 to TE5 pending, drafted with the recommended ones),
//! behind the switch `scorched_earth` (off). A country that ordered it against an enemy destroys,
//! the week a region it holds passes to that enemy's side, a share of the region's plant levels:
//! industry, mines and power first (TE2 a). The share grows with the weeks the region was fought
//! over before it fell, up to a ceiling: a region taken by surprise burns little (TE1 a). The
//! same order holds for land the country occupies (TE4 a). The order is the player's, given and
//! withdrawn in the Agendas (`GameCommand::SetScorchedEarth`); the AI does not give it (phase 6).
//! The levels come back through the works queue, as after any devastation; no political price in
//! this slice (TE5 a). Evacuating instead of destroying (TE3) is a later slice.

use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use crate::game::GameWorld;

/// `data/military/war.ron`, `occupation.scorched`; `None` with the switch off.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorchRules {
    /// Most of a region's plant levels destroyed, per mille. DESIGN VALUE.
    pub cap: i32,
    /// Weeks of fighting over the region for the whole `cap`; fewer give a part. DESIGN VALUE.
    pub full_weeks: u16,
    /// Building groups destroyed first, in this order; the rest after.
    pub first: Vec<String>,
    /// `first` resolved: (building kind, rank), rank the place of its group in `first`. Made at
    /// load.
    #[serde(skip)]
    pub ranks: Vec<(u16, u16)>,
}

impl ScorchRules {
    /// The share of a region destroyed after `weeks` of fighting over it, per mille.
    pub fn share(&self, weeks: u16) -> i32 {
        let full = self.full_weeks.max(1);
        (i64::from(self.cap) * i64::from(weeks.min(full)) / i64::from(full)) as i32
    }

    fn rank(&self, kind: u16) -> u16 {
        self.ranks
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or(u16::MAX, |(_, r)| *r)
    }
}

/// What a scorched region lost.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Burn {
    pub region: u32,
    /// The country that burned it, and the one it fell to.
    pub by: u16,
    pub to: u16,
    /// Plant levels destroyed, and that as a share of its levels, per mille.
    pub levels: i64,
    pub share: i32,
    /// `Date::to_days` of the week it fell.
    pub day: i64,
}

/// The orders and what they burned; empty with the switch off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorchState {
    /// (country, enemy): the country burns what falls to `enemy`'s side. Sorted.
    pub orders: Vec<(u16, u16)>,
    /// (region, weeks): how long each region has been fought over, while it is. Sorted.
    pub threatened: Vec<(u32, u16)>,
    /// What was burned, in order.
    pub burned: Vec<Burn>,
}

impl ScorchState {
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty() && self.threatened.is_empty() && self.burned.is_empty()
    }

    pub fn ordered(&self, country: u16, enemy: u16) -> bool {
        self.orders.binary_search(&(country, enemy)).is_ok()
    }

    /// Gives (`on`) or withdraws the order.
    pub fn set(&mut self, country: u16, enemy: u16, on: bool) {
        match (self.orders.binary_search(&(country, enemy)), on) {
            (Err(i), true) => self.orders.insert(i, (country, enemy)),
            (Ok(i), false) => {
                self.orders.remove(i);
            }
            _ => {}
        }
    }

    pub fn weeks(&self, region: u32) -> u16 {
        self.threatened
            .binary_search_by_key(&region, |(r, _)| *r)
            .map_or(0, |i| self.threatened[i].1)
    }
}

impl HashState for ScorchState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.orders.len());
        for (c, e) in &self.orders {
            h.write_u16(*c);
            h.write_u16(*e);
        }
        h.write_len(self.threatened.len());
        for (r, w) in &self.threatened {
            h.write_u32(*r);
            h.write_u16(*w);
        }
        h.write_len(self.burned.len());
        for b in &self.burned {
            h.write_u32(b.region);
            h.write_u16(b.by);
            h.write_u16(b.to);
            h.write_i64(b.levels);
            h.write_i64(i64::from(b.share));
            h.write_i64(b.day);
        }
    }
}

/// Whether `country`'s order holds against `taker`: an order against `taker` itself, or against
/// a country on `taker`'s side of a war `country` fights.
pub fn holds(world: &GameWorld, s: &ScorchState, country: u16, taker: u16) -> bool {
    if s.ordered(country, taker) {
        return true;
    }
    world.war.conflicts.wars.iter().any(|w| {
        let theirs = if w.side_a.contains(&country) {
            &w.side_b
        } else if w.side_b.contains(&country) {
            &w.side_a
        } else {
            return false;
        };
        theirs.contains(&taker) && theirs.iter().any(|e| s.ordered(country, *e))
    })
}

/// A week: the regions being fought over count one more week (those no longer are forgotten),
/// and each region that changed hands this week burns if the side that lost it ordered so.
/// `before` and `after` give the controller of a region before and after the week's combat;
/// `fought` the regions an advance went into this week. Returns the new state and (region,
/// levels) to destroy.
pub fn week(
    rules: &ScorchRules,
    world: &GameWorld,
    fought: &[u32],
    taken: &[(u32, u16, u16)],
    day: i64,
) -> (ScorchState, Vec<(u32, i64)>) {
    let old = &world.war.scorch;
    let mut s = ScorchState {
        orders: old.orders.clone(),
        threatened: Vec::new(),
        burned: old.burned.clone(),
    };
    let mut fought = fought.to_vec();
    fought.sort_unstable();
    fought.dedup();
    let mut out = Vec::new();
    for &(region, lost_by, taker) in taken {
        if !holds(world, old, lost_by, taker) {
            continue;
        }
        let weeks = old.weeks(region).saturating_add(1);
        let share = rules.share(weeks);
        let levels: i64 = world.econ.regions[region as usize]
            .plants
            .iter()
            .map(|p| p.levels)
            .sum();
        let n = levels * i64::from(share) / 1000;
        if n > 0 {
            out.push((region, n));
        }
        s.burned.push(Burn {
            region,
            by: lost_by,
            to: taker,
            levels: n,
            share,
            day,
        });
    }
    for r in fought {
        if taken.iter().any(|(t, _, _)| *t == r) {
            continue;
        }
        s.threatened.push((r, old.weeks(r).saturating_add(1)));
    }
    (s, out)
}

/// Destroys `levels` plant levels in `region`: the groups of `first` in their order, the largest
/// plant first within a rank, then the rest.
pub fn burn(rules: &ScorchRules, world: &mut GameWorld, region: u32, mut levels: i64) {
    let plants = &mut world.econ.regions[region as usize].plants;
    while levels > 0 {
        let Some(p) = plants
            .iter_mut()
            .filter(|p| p.levels > 0)
            .min_by_key(|p| (rules.rank(p.kind), std::cmp::Reverse(p.levels), p.kind))
        else {
            break;
        };
        p.levels -= 1;
        levels -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> ScorchRules {
        ScorchRules {
            cap: 500,
            full_weeks: 4,
            first: vec![],
            ranks: vec![],
        }
    }

    #[test]
    fn the_share_grows_with_the_weeks_of_fighting() {
        let r = rules();
        assert_eq!(r.share(0), 0);
        assert_eq!(r.share(1), 125);
        assert_eq!(r.share(4), 500);
        assert_eq!(r.share(40), 500);
    }

    #[test]
    fn orders_are_given_and_withdrawn() {
        let mut s = ScorchState::default();
        s.set(3, 1, true);
        s.set(2, 1, true);
        s.set(3, 1, true);
        assert_eq!(s.orders, vec![(2, 1), (3, 1)]);
        s.set(3, 1, false);
        assert!(!s.ordered(3, 1) && s.ordered(2, 1));
    }
}
