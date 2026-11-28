//! What a government fixes over the market in industry (`docs/wiki/o-que-o-jogador-decide.md`,
//! owner's decisions M1, M2 and E0 to E4 of 2026-11-28), behind `Switches::fixed_methods`.
//!
//! - **The method of a building kind (M1, M2):** the government may fix the method every plant
//!   of a kind in the country runs, private or the state's. At the yearly switch the plants
//!   move to it, paid by the investment pool as any switch, ahead of the switches the market
//!   wants; new plants of the kind are built with it. The owners lose the profit of the
//!   difference and nothing more (M2). Under the `laissez_faire` ownership law
//!   (`state_ownership` zero) the government may fix nothing, and what was fixed rests until a
//!   law with state ownership is back.
//! - **The state's levels (E0):** of a plant's levels, those the state owns (`state_levels`);
//!   kept here, for the slice that builds them.
//!
//! With nothing fixed anywhere the state stays out of the hash and the save holds two empty
//! lists.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::state::LawEffects;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndustryState {
    /// M1: by market, the method each building kind must run (kind, method), sorted by kind.
    pub methods: Vec<Vec<(u16, u8)>>,
    /// E0: by region, the levels of each plant the state owns (kind, levels), sorted by kind.
    pub state_levels: Vec<Vec<(u16, i64)>>,
    /// The productive works the government ordered and that are not done yet (region, kind,
    /// works), sorted: a public work of the kind done in the region is the state's while one
    /// is open. The rule's own public works (railways, roads, housing) stay out.
    pub orders: Vec<(u32, u16, i64)>,
}

/// Whether the ownership law lets the government fix methods: every law but `laissez_faire`,
/// the only one with no state ownership (`data/politics/laws.ron`).
pub fn may_fix(laws: &LawEffects) -> bool {
    laws.state_ownership > Fixed::ZERO
}

impl IndustryState {
    /// Nothing fixed and nothing owned anywhere: out of the hash.
    pub fn is_empty(&self) -> bool {
        self.methods.iter().all(Vec::is_empty)
            && self.state_levels.iter().all(Vec::is_empty)
            && self.orders.is_empty()
    }

    /// The method market `mi` fixed for building `kind`, if any.
    pub fn fixed(&self, mi: usize, kind: u16) -> Option<u8> {
        let list = self.methods.get(mi)?;
        list.binary_search_by_key(&kind, |e| e.0)
            .ok()
            .map(|i| list[i].1)
    }

    /// Fixes (or, with `None`, hands back to the market) the method of `kind` in market `mi`.
    pub fn set_method(&mut self, mi: usize, kind: u16, method: Option<u8>) {
        if self.methods.len() <= mi {
            if method.is_none() {
                return;
            }
            self.methods.resize(mi + 1, Vec::new());
        }
        let list = &mut self.methods[mi];
        match (list.binary_search_by_key(&kind, |e| e.0), method) {
            (Ok(i), Some(m)) => list[i].1 = m,
            (Ok(i), None) => {
                list.remove(i);
            }
            (Err(i), Some(m)) => list.insert(i, (kind, m)),
            (Err(_), None) => {}
        }
        // An empty tail leaves the world as if nothing had been fixed.
        while self.methods.last().is_some_and(Vec::is_empty) {
            self.methods.pop();
        }
    }
}

impl HashState for IndustryState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.methods.len() as u64);
        for list in &self.methods {
            h.write_u64(list.len() as u64);
            for &(k, m) in list {
                h.write_u16(k);
                h.write_u8(m);
            }
        }
        h.write_u64(self.orders.len() as u64);
        for &(r, k, n) in &self.orders {
            h.write_u32(r);
            h.write_u16(k);
            h.write_i64(n);
        }
        h.write_u64(self.state_levels.len() as u64);
        for list in &self.state_levels {
            h.write_u64(list.len() as u64);
            for &(k, l) in list {
                h.write_u16(k);
                h.write_i64(l);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixing_and_handing_back_leaves_it_empty() {
        let mut s = IndustryState::default();
        s.set_method(2, 7, Some(1));
        s.set_method(2, 3, Some(0));
        assert_eq!(s.fixed(2, 7), Some(1));
        assert_eq!(s.fixed(2, 3), Some(0));
        assert_eq!(s.fixed(1, 7), None);
        assert!(!s.is_empty());
        s.set_method(2, 7, None);
        s.set_method(2, 3, None);
        assert!(s.is_empty());
        assert!(s.methods.is_empty());
        s.set_method(9, 1, None);
        assert!(s.methods.is_empty());
    }
}
