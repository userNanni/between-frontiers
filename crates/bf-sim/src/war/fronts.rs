//! Fronts as objects of command (G2; the orchestrator's design of 2026-11-24, the owner's FR1
//! to FR6 pending, drafted with the recommended ones), behind the switch `fronts` (off).
//!
//! - FR1 a: a front is the line against one enemy, as the war window shows it.
//! - FR2 a: the player sets the share of the land army on a front; the fronts without an order
//!   share the rest by their segments, as before. Within a front the men go to its segments as
//!   before (evenly, or by the third generation's concentration).
//! - FR3 a: a front may have an enemy state as objective: its segments into that state weigh
//!   `objective_weight`, and the third generation's focus looks there first.
//! - FR4 a: a share kept in reserve stands off the line; each week `reserve_step` of it joins
//!   the segment where the country lost the most ground the week before.
//! - FR5 a: the men a front gains come from those in transit, `transfer_per_week` of the army
//!   a week; those a front gives up go into transit at once. In transit they do not fight.
//! - FR6 a: the AI keeps the rule of before (no orders); allocation by threat is phase 6.
//!
//! Every share is per mille of the country's land army in its main theatre. The shares on the
//! line, in reserve, with the fronts without an order, and in transit always sum to 1000: the
//! men are the same, only where they stand changes.

use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

/// `data/military/war.ron`, `fronts`; `None` with the switch off.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontRules {
    /// Weight of a segment into the front's objective, per mille of the others'. DESIGN VALUE.
    pub objective_weight: u32,
    /// Of the reserve, the part that joins the threatened segment each week, per mille. DESIGN
    /// VALUE.
    pub reserve_step: u32,
    /// Of the army, the most that reaches its new place each week, per mille. DESIGN VALUE.
    pub transfer_per_week: u16,
}

/// The player's order on the front against `enemy`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontOrder {
    pub enemy: u16,
    /// The share set, and the share on the line now, per mille.
    pub share: u16,
    pub now: u16,
    /// The enemy state the front aims at (`Region::state`).
    pub objective: Option<u16>,
}

/// A country's fronts under orders.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryFronts {
    pub country: u16,
    /// Sorted by enemy.
    pub fronts: Vec<FrontOrder>,
    /// The reserve set and the reserve now.
    pub reserve: u16,
    pub reserve_now: u16,
    /// The share now with the fronts without an order; their target is what the orders leave.
    pub rest_now: u16,
    /// The region the reserve reinforced last week, if any.
    pub reinforced: Option<u32>,
}

impl CountryFronts {
    /// What the fronts without an order are to have: what the orders and the reserve leave.
    pub fn rest(&self) -> u16 {
        1000u16.saturating_sub(self.reserve + self.fronts.iter().map(|f| f.share).sum::<u16>())
    }

    /// The share in transit: neither on a line nor in reserve.
    pub fn transit(&self) -> u16 {
        1000u16.saturating_sub(
            self.reserve_now + self.rest_now + self.fronts.iter().map(|f| f.now).sum::<u16>(),
        )
    }

    fn front_mut(&mut self, enemy: u16) -> &mut FrontOrder {
        let i = match self.fronts.binary_search_by_key(&enemy, |f| f.enemy) {
            Ok(i) => i,
            Err(i) => {
                self.fronts.insert(
                    i,
                    FrontOrder {
                        enemy,
                        share: 0,
                        now: 0,
                        objective: None,
                    },
                );
                i
            }
        };
        &mut self.fronts[i]
    }

    /// Puts the front against `enemy` under orders with its part of the rest now (`segments`:
    /// (enemy, segments of that front) for the fronts without an order), so that listing it
    /// moves no one.
    fn list(&mut self, enemy: u16, segments: &[(u16, u64)]) {
        if self.fronts.iter().any(|f| f.enemy == enemy) {
            return;
        }
        let unlisted: u64 = segments
            .iter()
            .filter(|(e, _)| !self.fronts.iter().any(|f| f.enemy == *e))
            .map(|(_, n)| *n)
            .sum();
        let mine = segments
            .iter()
            .find(|(e, _)| *e == enemy)
            .map_or(0, |(_, n)| *n);
        let part = (u64::from(self.rest_now) * mine)
            .checked_div(unlisted)
            .unwrap_or(0) as u16;
        self.rest_now -= part;
        let f = self.front_mut(enemy);
        f.share = part;
        f.now = part;
    }

    /// Sets the share of the front against `enemy`, within what the other orders and the
    /// reserve leave.
    pub fn set_share(&mut self, enemy: u16, share: u16, segments: &[(u16, u64)]) {
        self.list(enemy, segments);
        let others: u16 = self
            .fronts
            .iter()
            .filter(|f| f.enemy != enemy)
            .map(|f| f.share)
            .sum::<u16>()
            + self.reserve;
        self.front_mut(enemy).share = share.min(1000u16.saturating_sub(others));
    }

    pub fn set_objective(&mut self, enemy: u16, state: Option<u16>, segments: &[(u16, u64)]) {
        self.list(enemy, segments);
        self.front_mut(enemy).objective = state;
    }

    /// Sets the reserve, within what the fronts under orders leave.
    pub fn set_reserve(&mut self, share: u16) {
        let fronts: u16 = self.fronts.iter().map(|f| f.share).sum();
        self.reserve = share.min(1000u16.saturating_sub(fronts));
    }

    /// A week of moving: what stands above its share goes into transit at once; what stands
    /// below draws from transit up to `rate` of the army, the fronts first, then the reserve,
    /// then the fronts without an order.
    pub fn step(&mut self, rate: u16) {
        let rest = self.rest();
        for f in &mut self.fronts {
            f.now = f.now.min(f.share);
        }
        self.reserve_now = self.reserve_now.min(self.reserve);
        self.rest_now = self.rest_now.min(rest);
        let mut moving = self.transit().min(rate);
        let mut take = |now: &mut u16, target: u16| {
            let add = target.saturating_sub(*now).min(moving);
            *now += add;
            moving -= add;
        };
        for f in &mut self.fronts {
            take(&mut f.now, f.share);
        }
        take(&mut self.reserve_now, self.reserve);
        take(&mut self.rest_now, rest);
    }

    /// Per mille of the army on each front's line (`segments`: every front of the main theatre
    /// with its segments); the fronts without an order share `rest_now` by their segments.
    pub fn on_line(&self, segments: &[(u16, u64)]) -> Vec<(u16, u16)> {
        let unlisted: u64 = segments
            .iter()
            .filter(|(e, _)| !self.fronts.iter().any(|f| f.enemy == *e))
            .map(|(_, n)| *n)
            .sum();
        segments
            .iter()
            .map(|&(e, n)| match self.fronts.iter().find(|f| f.enemy == e) {
                Some(f) => (e, f.now),
                None if unlisted > 0 => (e, (u64::from(self.rest_now) * n / unlisted) as u16),
                None => (e, 0),
            })
            .collect()
    }
}

/// The orders of every country, sorted by country; empty with the switch off or no order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontsState {
    pub countries: Vec<CountryFronts>,
}

impl FrontsState {
    pub fn is_empty(&self) -> bool {
        self.countries.is_empty()
    }

    pub fn of(&self, country: u16) -> Option<&CountryFronts> {
        self.countries
            .binary_search_by_key(&country, |c| c.country)
            .ok()
            .map(|i| &self.countries[i])
    }

    /// The country's entry, made with the whole army with the fronts without an order.
    pub fn entry(&mut self, country: u16) -> &mut CountryFronts {
        let i = match self.countries.binary_search_by_key(&country, |c| c.country) {
            Ok(i) => i,
            Err(i) => {
                self.countries.insert(
                    i,
                    CountryFronts {
                        country,
                        rest_now: 1000,
                        ..Default::default()
                    },
                );
                i
            }
        };
        &mut self.countries[i]
    }
}

impl HashState for FrontsState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.countries.len());
        for c in &self.countries {
            h.write_u16(c.country);
            h.write_len(c.fronts.len());
            for f in &c.fronts {
                h.write_u16(f.enemy);
                h.write_u16(f.share);
                h.write_u16(f.now);
                h.write_u16(f.objective.unwrap_or(u16::MAX));
            }
            h.write_u16(c.reserve);
            h.write_u16(c.reserve_now);
            h.write_u16(c.rest_now);
            h.write_u32(c.reinforced.unwrap_or(u32::MAX));
        }
    }
}

/// The segments of each front of `country` last week: (enemy, segments).
pub fn segments_of(f: &super::fight::Fighting, country: u16) -> Vec<(u16, u64)> {
    let mut out: Vec<(u16, u64)> = Vec::new();
    for g in &f.last_week {
        let enemy = if g.a == country {
            g.b
        } else if g.b == country {
            g.a
        } else {
            continue;
        };
        match out.iter_mut().find(|(e, _)| *e == enemy) {
            Some(x) => x.1 += 1,
            None => out.push((enemy, 1)),
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sums(c: &CountryFronts) -> u16 {
        c.fronts.iter().map(|f| f.now).sum::<u16>() + c.reserve_now + c.rest_now + c.transit()
    }

    /// The men are the same: on the lines, in reserve, with the fronts without an order and in
    /// transit, always the whole army, week after week, whatever the orders.
    #[test]
    fn the_shares_always_sum_to_the_army() {
        let segs = [(3u16, 9u64), (5, 3)];
        let mut s = FrontsState::default();
        let c = s.entry(1);
        assert_eq!(c.on_line(&segs), vec![(3, 750), (5, 250)]);
        c.set_share(3, 600, &segs);
        // Listing the front moved no one: it starts with its part of the rest.
        assert_eq!((c.fronts[0].now, c.rest_now), (750, 250));
        c.set_reserve(150);
        assert_eq!(sums(c), 1000);
        for _ in 0..12 {
            c.step(100);
            assert_eq!(sums(c), 1000);
            assert!(c.fronts.iter().all(|f| f.now <= f.share));
        }
        assert_eq!(c.fronts[0].now, 600);
        assert_eq!(c.reserve_now, 150);
        assert_eq!(c.rest_now, 250);
        assert_eq!(c.transit(), 0);
        // Moving 400 more to the front against 3 takes weeks.
        c.set_reserve(0);
        c.set_share(3, 750, &segs);
        c.step(100);
        assert_eq!((c.fronts[0].now, c.transit()), (700, 50));
        assert_eq!(sums(c), 1000);
    }

    #[test]
    fn an_order_cannot_pass_the_army() {
        let segs = [(3u16, 1u64), (5, 1)];
        let mut s = FrontsState::default();
        let c = s.entry(0);
        c.set_share(3, 700, &segs);
        c.set_reserve(500);
        assert_eq!(c.reserve, 300);
        c.set_share(5, 900, &segs);
        assert_eq!(c.fronts[1].share, 0);
        assert_eq!(c.rest(), 0);
    }
}
