//! Who owns the plants that foreign capital built: the stakes, by cohort, and the memory of the
//! expropriations (`docs/foreign-capital.pt-BR.md`, sections 1 and 2).
//!
//! A plant stays one row per state and building kind (`econ::Plant`). A stake says how many of
//! its levels a foreign country's fund built, in which year and under which contract: two funds,
//! or the same fund in two years under two laws, are two stakes on the same plant. The stakes are
//! saved last in the world and hashed only when there is one, so a world where no foreign capital
//! ever moved keeps the hash it had before they existed.
//!
//! The money moves elsewhere: the monthly allocation in `econ::monthly` (a level built abroad
//! waits in [`ForeignState::building`] until the building trade finishes it) and the weekly
//! remittance in `econ::week` (the foreign share of a plant's profit goes to the owner's fund).
//! The contract (handover at term, expropriation) comes in a later step. This module holds the
//! state and the rules that keep it consistent with the plants.

use std::collections::BTreeMap;

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use laplace_schelling::schedule::ResourceId;

use crate::foreign_capital::{Handover, local_share};

/// The stakes and the works paid from abroad (`GameWorld::foreign`).
pub const FOREIGN: ResourceId = ResourceId(17);

/// Levels of one plant built by a foreign fund in one year under one contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stake {
    /// Index into `Economy::regions`: the state where the plant stands. The host country is that
    /// region's market at any date, so a border change carries the stake with the state.
    pub region: u32,
    /// Index into the catalogue's buildings.
    pub kind: u16,
    /// Market whose fund built the levels.
    pub owner: u16,
    /// Levels of the plant this stake holds.
    pub levels: i64,
    /// Year the levels were finished; the contract counts from it.
    pub built: i32,
    /// Share the local partner held when the contract began (zero: wholly foreign).
    pub partner: Fixed,
    /// Years of the contract; zero for none.
    pub term: u16,
    pub handover: Handover,
}

impl Stake {
    /// Share of the stake's levels the foreign owner holds in `year`, after what the contract
    /// has passed to the local partner.
    pub fn foreign_share(&self, year: i32) -> Fixed {
        Fixed::ONE
            - local_share(
                self.partner,
                i64::from(self.term),
                i64::from(year - self.built),
                self.handover,
            )
    }
}

/// Where a stake stands, who owns it and under which contract: the key of
/// [`ForeignState::stakes`], ordered by state, then kind, so the stakes of a plant (and of a
/// state) sit together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StakeKey {
    pub region: u32,
    pub kind: u16,
    pub owner: u16,
    pub built: i32,
    pub partner: Fixed,
    pub term: u16,
    pub handover: Handover,
}

impl StakeKey {
    /// The first and last keys of `region` (and of `kind` there, if given).
    fn bounds(region: u32, kind: Option<u16>) -> (Self, Self) {
        let lo = Self {
            region,
            kind: kind.unwrap_or(0),
            owner: 0,
            built: i32::MIN,
            partner: Fixed::from_raw(i64::MIN),
            term: 0,
            handover: Handover::Never,
        };
        let hi = Self {
            region,
            kind: kind.unwrap_or(u16::MAX),
            owner: u16::MAX,
            built: i32::MAX,
            partner: Fixed::from_raw(i64::MAX),
            term: u16::MAX,
            handover: Handover::Gradual,
        };
        (lo, hi)
    }
}

impl Stake {
    pub fn key(&self) -> StakeKey {
        StakeKey {
            region: self.region,
            kind: self.kind,
            owner: self.owner,
            built: self.built,
            partner: self.partner,
            term: self.term,
            handover: self.handover,
        }
    }

    pub fn from_key(k: &StakeKey, levels: i64) -> Self {
        Self {
            region: k.region,
            kind: k.kind,
            owner: k.owner,
            levels,
            built: k.built,
            partner: k.partner,
            term: k.term,
            handover: k.handover,
        }
    }
}

/// A country took foreign plants without paying for them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expropriation {
    /// Market that took them.
    pub host: u16,
    pub year: i32,
    /// Share of the foreign capital in the country that was taken, 0–1.
    pub share: Fixed,
}

/// Saved last in the world and hashed only when not empty. Kept in ordered maps: the rules
/// that read a plant's or a state's stakes (every week, every plant that shrinks) find them
/// by key, not by walking the world's.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignState {
    /// Levels of each stake.
    pub stakes: BTreeMap<StakeKey, i64>,
    pub expropriations: Vec<Expropriation>,
    /// Levels paid from abroad still in the host's queue: by (state, kind), the owners in the
    /// order they paid.
    pub building: BTreeMap<(u32, u16), Vec<u16>>,
    /// Earnings each owner reinvests in each host and has not yet built with, by (owner, host):
    /// the money sits in the host's fund, the claim is the owner's.
    pub retained: BTreeMap<(u16, u16), Fixed>,
}

impl ForeignState {
    pub fn is_empty(&self) -> bool {
        self.stakes.is_empty()
            && self.expropriations.is_empty()
            && self.building.is_empty()
            && self.retained.is_empty()
    }

    /// Every stake, in key order.
    pub fn stakes(&self) -> impl Iterator<Item = Stake> + '_ {
        self.stakes.iter().map(|(k, l)| Stake::from_key(k, *l))
    }

    /// Levels paid from abroad still building.
    pub fn building_count(&self) -> usize {
        self.building.values().map(Vec::len).sum()
    }

    /// Records a level paid by `owner`'s fund now in the queue of `region`.
    pub fn paid(&mut self, region: u32, kind: u16, owner: u16) {
        self.building.entry((region, kind)).or_default().push(owner);
    }

    /// What `owner` keeps in `host` to invest there.
    pub fn retained_in(&self, owner: u16, host: u16) -> Fixed {
        self.retained
            .get(&(owner, host))
            .copied()
            .unwrap_or(Fixed::ZERO)
    }

    /// Adds `amount` (negative to spend) to what `owner` keeps in `host`; an empty claim goes.
    pub fn retain(&mut self, owner: u16, host: u16, amount: Fixed) {
        let e = self.retained.entry((owner, host)).or_default();
        *e += amount;
        if *e <= Fixed::ZERO {
            self.retained.remove(&(owner, host));
        }
    }

    /// A level of `kind` in `region` was finished in `year`: if a foreign fund paid for one
    /// there, the first such becomes a stake of one level (added to the owner's stake of the
    /// same year, if any). The levels are alike, so which of the queued works of that kind
    /// finished first does not matter.
    pub fn finished(&mut self, region: u32, kind: u16, year: i32) {
        let Some(owners) = self.building.get_mut(&(region, kind)) else {
            return;
        };
        let owner = owners.remove(0);
        if owners.is_empty() {
            self.building.remove(&(region, kind));
        }
        self.add(Stake {
            region,
            kind,
            owner,
            levels: 1,
            built: year,
            partner: Fixed::ZERO,
            term: 0,
            handover: Handover::Never,
        });
    }

    /// Adds `stake`, merged with one of the same plant, owner, year and contract. A stake with
    /// no term has no use for its year (nothing passes at a date), so it merges with the owner's
    /// stake on the plant with no term and the same partner whatever its year, and keeps the
    /// year of the first: the stakes of a plant do not grow with the years of play.
    pub fn add(&mut self, stake: Stake) {
        if stake.levels <= 0 {
            return;
        }
        if stake.term == 0 {
            let (lo, hi) = StakeKey::bounds(stake.region, Some(stake.kind));
            let same = self.stakes.range_mut(lo..=hi).find(|(k, _)| {
                k.owner == stake.owner
                    && k.term == 0
                    && k.partner == stake.partner
                    && k.handover == stake.handover
            });
            if let Some((_, l)) = same {
                *l += stake.levels;
                return;
            }
        }
        *self.stakes.entry(stake.key()).or_default() += stake.levels;
    }

    /// Foreign owners of the plants of `region` in `year`, as (kind, owner, share of the plant's
    /// levels the owner holds after what the contracts passed to local partners), in key order.
    /// `levels_of` gives the plant's levels.
    pub fn owners_in(
        &self,
        region: u32,
        year: i32,
        levels_of: impl Fn(u16) -> i64,
    ) -> Vec<(u16, u16, Fixed)> {
        let (lo, hi) = StakeKey::bounds(region, None);
        let mut out: Vec<(u16, u16, Fixed)> = Vec::new();
        for (k, &l) in self.stakes.range(lo..=hi) {
            let levels = levels_of(k.kind);
            if levels <= 0 {
                continue;
            }
            let share = Stake::from_key(k, l)
                .foreign_share(year)
                .mul_ratio(l.min(levels), levels);
            // Keys run by kind, then owner: one owner's stakes on a plant sit together.
            match out.last_mut() {
                Some(e) if e.0 == k.kind && e.1 == k.owner => e.2 += share,
                _ => out.push((k.kind, k.owner, share)),
            }
        }
        out
    }

    /// The states that have stakes, in order.
    pub fn regions(&self) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for k in self.stakes.keys() {
            if out.last() != Some(&k.region) {
                out.push(k.region);
            }
        }
        out
    }

    /// Levels of the plant `kind` in `region` held by foreign stakes.
    pub fn foreign_levels(&self, region: u32, kind: u16) -> i64 {
        let (lo, hi) = StakeKey::bounds(region, Some(kind));
        self.stakes.range(lo..=hi).map(|(_, l)| *l).sum()
    }

    /// The plant `kind` in `region` went from `before` to `after` levels (depreciation, closing
    /// at a loss): each stake on it keeps the same share of the plant, rounded down, so the
    /// foreign levels never exceed the plant. Stakes left with no level go. A plant that grew
    /// (built at home) leaves the stakes as they are.
    pub fn plant_shrank(&mut self, region: u32, kind: u16, before: i64, after: i64) {
        if after >= before || before <= 0 {
            return;
        }
        let after = after.max(0);
        let (lo, hi) = StakeKey::bounds(region, Some(kind));
        let mut gone = Vec::new();
        for (k, l) in self.stakes.range_mut(lo..=hi) {
            *l = *l * after / before;
            if *l <= 0 {
                gone.push(*k);
            }
        }
        for k in gone {
            self.stakes.remove(&k);
        }
    }

    /// The region changed hands: stakes whose owner is the new host are no longer foreign and
    /// go (the levels stay in the plant, now wholly domestic). `market_of` gives a region's
    /// market after the change.
    pub fn border_changed(&mut self, market_of: impl Fn(u32) -> u16) {
        self.stakes.retain(|k, _| market_of(k.region) != k.owner);
        for ((region, _), owners) in self.building.iter_mut() {
            let host = market_of(*region);
            owners.retain(|o| *o != host);
        }
        self.building.retain(|_, owners| !owners.is_empty());
    }
}

/// Foreign capital by pair, for the influence's investment channel (`influence::dependence`):
/// the value of the plants of host B owned by A over B's GDP of last year, and the sum over
/// all owners of B. Plants still in the queue do not count; a stake counts the share of its
/// levels the contract has not passed to the local partner. Values at what a level of that
/// kind costs in that state now (`econ::monthly::level_capital`). Read-only: the stakes move on
/// day 8 (works abroad, plants closed), on Mondays (works finished) and on January 1st
/// (depreciation), so a reader on any day sees the state as of the last of those.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnedShares {
    /// (owner, host) → value over the host's GDP, only pairs with a stake.
    pub pair: std::collections::BTreeMap<(u16, u16), Fixed>,
    /// By host: the sum over its foreign owners.
    pub total: Vec<Fixed>,
}

impl OwnedShares {
    /// `owned_share(A, B)`.
    pub fn of(&self, owner: u16, host: u16) -> Fixed {
        self.pair
            .get(&(owner, host))
            .copied()
            .unwrap_or(Fixed::ZERO)
    }

    /// `foreign_share(B)`: every foreign owner together.
    pub fn foreign(&self, host: u16) -> Fixed {
        self.total
            .get(usize::from(host))
            .copied()
            .unwrap_or(Fixed::ZERO)
    }
}

/// [`OwnedShares`] of the world in `year`, one pass over the stakes.
pub fn owned_shares(
    world: &crate::game::GameWorld,
    cat: &crate::econ::catalogue::Catalogue,
    year: i32,
) -> OwnedShares {
    let econ = &world.econ;
    let mut value: std::collections::BTreeMap<(u16, u16), Fixed> = Default::default();
    for s in world.foreign.stakes() {
        let r = &econ.regions[s.region as usize];
        let m = &econ.markets[r.market as usize];
        let b = &cat.buildings[s.kind as usize];
        let level = crate::econ::monthly::level_capital(
            cat,
            usize::from(s.kind),
            0,
            crate::econ::plant_tfp(r, m, b),
        );
        *value.entry((s.owner, r.market)).or_default() += level * s.levels * s.foreign_share(year);
    }
    let mut out = OwnedShares {
        total: vec![Fixed::ZERO; econ.markets.len()],
        ..Default::default()
    };
    for ((owner, host), v) in value {
        let gdp = econ.markets[usize::from(host)].gdp_last_year;
        if gdp <= Fixed::ZERO {
            continue;
        }
        let share = v / gdp;
        out.pair.insert((owner, host), share);
        out.total[usize::from(host)] += share;
    }
    out
}

impl HashState for Handover {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u8(match self {
            Handover::Never => 0,
            Handover::AtTerm => 1,
            Handover::Gradual => 2,
        });
    }
}

impl HashState for ForeignState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_len(self.stakes.len());
        for (k, l) in &self.stakes {
            h.write_u32(k.region);
            h.write_u16(k.kind);
            h.write_u16(k.owner);
            h.write_i64(i64::from(k.built));
            k.partner.hash_state(h);
            h.write_u16(k.term);
            k.handover.hash_state(h);
            h.write_i64(*l);
        }
        self.expropriations.hash_state(h);
        h.write_len(self.building.len());
        for ((region, kind), owners) in &self.building {
            h.write_u32(*region);
            h.write_u16(*kind);
            h.write_len(owners.len());
            for o in owners {
                h.write_u16(*o);
            }
        }
        h.write_len(self.retained.len());
        for ((owner, host), amount) in &self.retained {
            h.write_u16(*owner);
            h.write_u16(*host);
            amount.hash_state(h);
        }
    }
}

impl HashState for Expropriation {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.host);
        h.write_i64(i64::from(self.year));
        self.share.hash_state(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::milli;

    fn stake(region: u32, kind: u16, owner: u16, levels: i64) -> Stake {
        Stake {
            region,
            kind,
            owner,
            levels,
            built: 1980,
            partner: Fixed::ZERO,
            term: 0,
            handover: Handover::Never,
        }
    }

    fn with(stakes: &[Stake]) -> ForeignState {
        let mut f = ForeignState::default();
        for s in stakes {
            f.add(s.clone());
        }
        f
    }

    #[test]
    fn a_plant_that_shrinks_takes_its_stakes_down_in_proportion() {
        let mut f = with(&[
            stake(3, 7, 1, 6),
            stake(3, 7, 2, 3),
            stake(3, 8, 1, 4),
            stake(4, 7, 1, 2),
        ]);
        // 12 levels, 9 foreign, down to 8: 6 → 4, 3 → 2.
        f.plant_shrank(3, 7, 12, 8);
        assert_eq!(f.foreign_levels(3, 7), 6);
        let levels: Vec<i64> = f.stakes().map(|s| s.levels).collect();
        assert_eq!(levels, vec![4, 2, 4, 2]);
        // Other plants untouched.
        assert_eq!(f.foreign_levels(3, 8), 4);
        assert_eq!(f.foreign_levels(4, 7), 2);
        // Down to 1 level: the stakes round to zero and go, never above the plant.
        f.plant_shrank(3, 7, 8, 1);
        assert_eq!(f.foreign_levels(3, 7), 0);
        assert_eq!(f.stakes.len(), 2);
        // Growth at home leaves them.
        f.plant_shrank(3, 8, 4, 9);
        assert_eq!(f.foreign_levels(3, 8), 4);
    }

    #[test]
    fn a_state_that_returns_to_the_owner_ends_its_stakes() {
        let mut f = with(&[stake(3, 7, 1, 6), stake(4, 7, 1, 2), stake(3, 7, 2, 1)]);
        f.paid(3, 9, 1);
        f.paid(3, 9, 2);
        // Region 3 passes to market 1: the stake and the work of 1 there are domestic now.
        f.border_changed(|r| if r == 3 { 1 } else { 5 });
        let left: Vec<Stake> = f.stakes().collect();
        assert_eq!(left, vec![stake(3, 7, 2, 1), stake(4, 7, 1, 2)]);
        assert_eq!(f.building.get(&(3, 9)), Some(&vec![2]));
    }

    #[test]
    fn a_finished_level_paid_from_abroad_becomes_a_stake() {
        let mut f = ForeignState::default();
        f.paid(3, 7, 1);
        f.paid(3, 7, 2);
        assert_eq!(f.building_count(), 2);
        // A level of another kind finishes: nothing.
        f.finished(3, 8, 1980);
        assert!(f.stakes.is_empty());
        f.finished(3, 7, 1980);
        f.finished(3, 7, 1980);
        f.finished(3, 7, 1980); // a home level: no foreign work left
        let made: Vec<Stake> = f.stakes().collect();
        assert_eq!(made, vec![stake(3, 7, 1, 1), stake(3, 7, 2, 1)]);
        assert!(f.building.is_empty());
        // The same owner, plant and year adds to its stake.
        f.paid(3, 7, 1);
        f.finished(3, 7, 1980);
        assert_eq!(f.foreign_levels(3, 7), 3);
        // Owners of the region's plants: 2 and 1 of 10 levels.
        let owners = f.owners_in(3, 1980, |_| 10);
        assert_eq!(owners, vec![(7, 1, milli(200)), (7, 2, milli(100))]);
        assert_eq!(f.regions(), vec![3]);
    }

    #[test]
    fn reinvested_earnings_add_up_and_a_spent_claim_goes() {
        let mut f = ForeignState::default();
        f.retain(1, 4, milli(300));
        f.retain(2, 4, milli(100));
        f.retain(1, 4, milli(200));
        assert_eq!(f.retained_in(1, 4), milli(500));
        assert_eq!(f.retained_in(2, 4), milli(100));
        f.retain(1, 4, -milli(500));
        assert_eq!(f.retained_in(1, 4), Fixed::ZERO);
        assert_eq!(f.retained.len(), 1);
        assert!(!f.is_empty());
    }

    #[test]
    fn the_contract_passes_the_plant_to_the_partner() {
        let mut s = stake(3, 7, 1, 10);
        s.partner = milli(400);
        s.term = 10;
        s.handover = Handover::AtTerm;
        assert_eq!(s.foreign_share(1985), milli(600));
        assert_eq!(s.foreign_share(1990), Fixed::ZERO);
        s.handover = Handover::Gradual;
        assert_eq!(s.foreign_share(1985), milli(300));
    }
}
