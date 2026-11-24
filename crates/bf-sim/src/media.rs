//! The reach of the mass media (`game::Switches::media`, slice 1 of the media, the owner's
//! decision MD1: reach measured, never a fixed historical series; docs/politics.pt-BR.md,
//! "Mídia"): each country's newspapers, radio sets, television sets and internet users per
//! thousand people, as stocks that move a little every year, and the share of the people they
//! reach (the indicator `media_reach`). Nothing else reads it yet: which movements and which
//! support it moves is slice 2 (MD2–MD6, for the owner).
//!
//! - **Newspapers** follow the literate: copies per literate person rise with the GDP per head
//!   toward a ceiling (`MediaRules::copies_per_literate`, `copies_half_income`), and the stock
//!   closes a share of the gap each year (`MediaRules::news_pace`).
//! - **Sets and users** are bought with the households' consumer electronics
//!   (`MediaRules::electronics`): each year the market's purchases of it are split between the
//!   media its technologies allow (radio, then television, then the web), divided by what a set
//!   costs, and added to the stock, which wears out over the set's life.
//! - **Reach**: a copy is read by some people, a set heard or watched by a household, a user is
//!   one person; the reach is the share of the people at least one medium reaches.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

/// The media in the order of `MediaState::markets`.
pub const MEDIA: [&str; 4] = ["newspapers", "radio", "television", "internet"];

/// What the rule reads (`data/politics/media.ron`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MediaRules {
    /// The good households buy their sets with, and the share of what they buy of it that is
    /// sets for the media (the rest is the other appliances).
    pub electronics: u16,
    pub sets_share: Fixed,
    /// For radio, television and internet: the technology that opens it (its industrial stage
    /// in the country), what one set or user costs in units of the good, and its life in years.
    pub sets: [(u16, Fixed, i64); 3],
    /// The share of the electronics bought that goes to each medium, by how many are open: with
    /// radio alone; with radio and television; with all three.
    pub split: [[Fixed; 3]; 3],
    /// Newspaper copies a literate person buys at most, and the GDP per head (1925 base prices,
    /// as `Economy::history`) at which he buys half of that; the share of the gap to that a year.
    pub copies_per_literate: Fixed,
    pub copies_half_income: Fixed,
    pub news_pace: Fixed,
    /// People one unit of each medium reaches.
    pub people_per_unit: [Fixed; 4],
}

/// `GameWorld::media`: by market, each medium per thousand people, and the consumer electronics
/// bought this year so far. Empty while `Switches::media` is off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaState {
    pub markets: Vec<[Fixed; 4]>,
    pub bought: Vec<Fixed>,
    /// The share of each market's people the media reached at the last year's end, per mille
    /// ([`MediaState::reach`]): what the indicator `media_reach` reads.
    pub reached: Vec<i32>,
}

impl MediaState {
    pub fn is_empty(&self) -> bool {
        self.markets.is_empty()
    }

    /// The share of market `mi`'s people the media reach, per mille.
    pub fn reach(&self, rules: &MediaRules, mi: usize) -> i32 {
        let Some(per) = self.markets.get(mi) else {
            return 0;
        };
        let mut missed = Fixed::ONE;
        for (x, people) in per.iter().zip(&rules.people_per_unit) {
            let covered = (*x * *people / 1000).clamp(Fixed::ZERO, Fixed::ONE);
            missed = missed * (Fixed::ONE - covered);
        }
        ((Fixed::ONE - missed) * 1000).to_int_round() as i32
    }

    /// Adds a week of each market's purchases of the electronics.
    pub fn add_week(&mut self, bought: &[Fixed]) {
        if self.bought.len() < bought.len() {
            self.bought.resize(bought.len(), Fixed::ZERO);
        }
        for (b, x) in self.bought.iter_mut().zip(bought) {
            *b += *x;
        }
    }
}

impl HashState for MediaState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.markets.len() as u64);
        for m in &self.markets {
            for x in m {
                x.hash_state(h);
            }
        }
        for b in &self.bought {
            b.hash_state(h);
        }
        for r in &self.reached {
            h.write_u32(*r as u32);
        }
    }
}

/// What a market is at the end of a year, for [`year`].
pub struct MarketYear {
    pub population: i64,
    pub literacy: Fixed,
    pub gdp_per_head: Fixed,
    /// Radio, television, internet open (their technology industrial in the country).
    pub open: [bool; 3],
}

/// The year closes: each market's stocks move, and this year's purchases start again.
pub fn year(rules: &MediaRules, state: &mut MediaState, markets: &[MarketYear]) {
    let n = markets.len();
    let first = state.markets.is_empty();
    if state.markets.len() < n {
        state.markets.resize(n, [Fixed::ZERO; 4]);
        state.bought.resize(n, Fixed::ZERO);
    }
    for (mi, m) in markets.iter().enumerate() {
        let per = &mut state.markets[mi];
        if m.population <= 0 {
            continue;
        }
        // Newspapers: toward the literate's copies; the first year starts at them (the 1925
        // press was there before the rule).
        let y = m.gdp_per_head.max(Fixed::ZERO);
        let copies = rules.copies_per_literate * y / (y + rules.copies_half_income).max(Fixed::ONE);
        let target = m.literacy * copies * 1000;
        if first {
            per[0] = target;
        } else {
            per[0] += (target - per[0]) * rules.news_pace;
        }
        // Sets: the year's electronics split among the open media, worn by their lives.
        let open = m.open.iter().filter(|o| **o).count();
        let per_thousand = state.bought[mi] * rules.sets_share * 1000 / m.population;
        for k in 0..3 {
            let (_, price, life) = rules.sets[k];
            let share = if open == 0 || !m.open[k] {
                Fixed::ZERO
            } else {
                rules.split[open - 1][k]
            };
            let new = if price > Fixed::ZERO {
                per_thousand * share / price
            } else {
                Fixed::ZERO
            };
            let worn = per[k + 1] / life.max(1);
            per[k + 1] = (per[k + 1] + new - worn).max(Fixed::ZERO);
        }
        state.bought[mi] = Fixed::ZERO;
    }
    state.reached = (0..n).map(|mi| state.reach(rules, mi)).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> MediaRules {
        MediaRules {
            electronics: 0,
            sets_share: Fixed::ONE,
            sets: [
                (0, Fixed::ONE, 10),
                (1, Fixed::from_int(2), 10),
                (2, Fixed::ONE, 5),
            ],
            split: [
                [Fixed::ONE, Fixed::ZERO, Fixed::ZERO],
                [
                    Fixed::from_ratio(3, 10),
                    Fixed::from_ratio(7, 10),
                    Fixed::ZERO,
                ],
                [
                    Fixed::from_ratio(1, 10),
                    Fixed::from_ratio(4, 10),
                    Fixed::from_ratio(5, 10),
                ],
            ],
            copies_per_literate: Fixed::from_ratio(1, 2),
            copies_half_income: Fixed::from_int(1000),
            news_pace: Fixed::from_ratio(1, 10),
            people_per_unit: [
                Fixed::from_ratio(5, 2),
                Fixed::from_int(4),
                Fixed::from_int(4),
                Fixed::ONE,
            ],
        }
    }

    #[test]
    fn stocks_grow_with_literacy_and_purchases_and_wear_out() {
        let r = rules();
        let mut st = MediaState::default();
        st.add_week(&[Fixed::from_int(100)]);
        let m = MarketYear {
            population: 1000,
            literacy: Fixed::from_ratio(1, 2),
            gdp_per_head: Fixed::from_int(1000),
            open: [true, false, false],
        };
        year(&r, &mut st, std::slice::from_ref(&m));
        // Newspapers: the first year at 0.5 literate × 0.25 copies × 1000 = 125.
        assert_eq!(st.markets[0][0], Fixed::from_int(125));
        // 100 units bought by 1,000 people: 100 radios per thousand, nothing else.
        assert_eq!(st.markets[0][1], Fixed::from_int(100));
        assert_eq!(st.markets[0][2], Fixed::ZERO);
        assert_eq!(st.bought[0], Fixed::ZERO);
        // A year with nothing bought: a tenth of the radios wear out.
        year(&r, &mut st, std::slice::from_ref(&m));
        assert_eq!(st.markets[0][1], Fixed::from_int(90));
        // 90 radios × 4 people and 125 copies × 2.5 reach most of the 1,000.
        assert!(st.reach(&r, 0) > 380);
    }
}
