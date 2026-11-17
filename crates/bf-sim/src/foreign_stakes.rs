//! Who owns the plants that foreign capital built: the stakes, by cohort, and the memory of the
//! expropriations (`docs/foreign-capital.pt-BR.md`, sections 1 and 2).
//!
//! A plant stays one row per state and building kind (`econ::Plant`). A stake says how many of
//! its levels a foreign country's fund built, in which year and under which contract: two funds,
//! or the same fund in two years under two laws, are two stakes on the same plant. The stakes are
//! saved last in the world and hashed only when there is one, so a world where no foreign capital
//! ever moved keeps the hash it had before they existed.
//!
//! Nothing here moves money yet. The flow (monthly allocation, weekly remittance) and the
//! contract (handover at term, expropriation) come in the next steps of the design; this module
//! holds the state and the rules that keep it consistent with the plants.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use crate::foreign_capital::{Handover, local_share};

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

/// A country took foreign plants without paying for them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expropriation {
    /// Market that took them.
    pub host: u16,
    pub year: i32,
    /// Share of the foreign capital in the country that was taken, 0–1.
    pub share: Fixed,
}

/// Saved last in the world and hashed only when not empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignState {
    /// In order of creation; every rule walks them in that order.
    pub stakes: Vec<Stake>,
    pub expropriations: Vec<Expropriation>,
}

impl ForeignState {
    pub fn is_empty(&self) -> bool {
        self.stakes.is_empty() && self.expropriations.is_empty()
    }

    /// Levels of the plant `kind` in `region` held by foreign stakes.
    pub fn foreign_levels(&self, region: u32, kind: u16) -> i64 {
        self.stakes
            .iter()
            .filter(|s| s.region == region && s.kind == kind)
            .map(|s| s.levels)
            .sum()
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
        for s in &mut self.stakes {
            if s.region == region && s.kind == kind {
                s.levels = s.levels * after / before;
            }
        }
        self.stakes.retain(|s| s.levels > 0);
    }

    /// The region changed hands: stakes whose owner is the new host are no longer foreign and
    /// go (the levels stay in the plant, now wholly domestic). `market_of` gives a region's
    /// market after the change.
    pub fn border_changed(&mut self, market_of: impl Fn(u32) -> u16) {
        self.stakes.retain(|s| market_of(s.region) != s.owner);
    }
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

impl HashState for Stake {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u32(self.region);
        h.write_u16(self.kind);
        h.write_u16(self.owner);
        h.write_i64(self.levels);
        h.write_i64(i64::from(self.built));
        self.partner.hash_state(h);
        h.write_u16(self.term);
        self.handover.hash_state(h);
    }
}

impl HashState for Expropriation {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u16(self.host);
        h.write_i64(i64::from(self.year));
        self.share.hash_state(h);
    }
}

impl HashState for ForeignState {
    fn hash_state(&self, h: &mut StateHasher) {
        self.stakes.hash_state(h);
        self.expropriations.hash_state(h);
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

    #[test]
    fn a_plant_that_shrinks_takes_its_stakes_down_in_proportion() {
        let mut f = ForeignState {
            stakes: vec![
                stake(3, 7, 1, 6),
                stake(3, 7, 2, 3),
                stake(3, 8, 1, 4),
                stake(4, 7, 1, 2),
            ],
            ..Default::default()
        };
        // 12 levels, 9 foreign, down to 8: 6 → 4, 3 → 2.
        f.plant_shrank(3, 7, 12, 8);
        assert_eq!(f.foreign_levels(3, 7), 6);
        assert_eq!(f.stakes[0].levels, 4);
        assert_eq!(f.stakes[1].levels, 2);
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
        let mut f = ForeignState {
            stakes: vec![stake(3, 7, 1, 6), stake(4, 7, 1, 2), stake(3, 7, 2, 1)],
            ..Default::default()
        };
        // Region 3 passes to market 1: the stake of 1 there is domestic now; 2's stays.
        f.border_changed(|r| if r == 3 { 1 } else { 5 });
        assert_eq!(f.stakes, vec![stake(4, 7, 1, 2), stake(3, 7, 2, 1)]);
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
