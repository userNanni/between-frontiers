//! The household fuel the countryside gathered itself (`game::Switches::household_biomass`;
//! docs/trade.pt-BR.md, "O combustível das famílias"): firewood, dung and straw that never went
//! through a market. Rural households (the pops that are not urban, `migration::is_urban`) buy
//! only the rest of their need `domestic_fuel`; townspeople buy all of it.
//!
//! The share is each country's, of 1925 (`data/economy/biomass.ron`), as a share of the rural
//! households' fuel. It holds while the country's GDP per head is under a threshold
//! (`BiomassRules::falls_above`) and falls in inverse proportion to it above (a design value:
//! rural India burned firewood, dung and straw for 90 % of its domestic energy around 1900 and
//! 93 % in 1970–71; the national share falls first with the towns, which buy their fuel).

use laplace_schelling::Fixed;

use super::Economy;

/// What the rule reads (`data/economy/biomass.ron`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BiomassRules {
    /// By market: the share of the rural households' fuel gathered in 1925, 0–1.
    pub rural_1925: Vec<Fixed>,
    /// GDP per head (1925 base prices, as `Economy::history`) above which the share falls in
    /// inverse proportion; a country richer than this in 1925 starts from its 1925 level.
    pub falls_above: Fixed,
}

/// By market, the share of the rural households' fuel gathered this year: the 1925 share times
/// the larger of the threshold and the 1925 GDP per head over last year's GDP per head, at most
/// the 1925 share. Before a full year has passed, the 1925 share.
pub fn shares(rules: &BiomassRules, econ: &Economy) -> Vec<Fixed> {
    let n = econ.markets.len();
    // GDP per head in 1925 and in the last year recorded, by market (`Economy::history`).
    let mut first = vec![Fixed::ZERO; n];
    let mut last = vec![Fixed::ZERO; n];
    for r in &econ.history {
        let mi = usize::from(r.market);
        if mi >= n || r.population <= 0 {
            continue;
        }
        let per_head = r.gdp / r.population;
        if r.year == 1925 {
            first[mi] = per_head;
        }
        last[mi] = per_head;
    }
    (0..n)
        .map(|mi| {
            let base = rules.rural_1925.get(mi).copied().unwrap_or(Fixed::ZERO);
            let from = first[mi].max(rules.falls_above);
            if first[mi] > Fixed::ZERO && last[mi] > from {
                base * (from / last[mi])
            } else {
                base
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::state::YearRecord;

    #[test]
    fn the_share_halves_when_the_country_is_twice_as_rich() {
        let mut econ = Economy::default();
        econ.markets = vec![crate::econ::Market::empty(1)];
        let rec = |year, gdp| YearRecord {
            year,
            market: 0,
            population: 100,
            gdp: Fixed::from_int(gdp),
            literacy: Fixed::ZERO,
            urban: Fixed::ZERO,
            basic_needs: Fixed::ZERO,
        };
        let mut rules = BiomassRules {
            rural_1925: vec![Fixed::from_ratio(9, 10)],
            falls_above: Fixed::ZERO,
        };
        assert_eq!(shares(&rules, &econ)[0], Fixed::from_ratio(9, 10));
        econ.history = vec![rec(1925, 1000), rec(1950, 2000)];
        assert_eq!(shares(&rules, &econ)[0], Fixed::from_ratio(45, 100));
        // Poorer than in 1925: the share stays.
        econ.history = vec![rec(1925, 1000), rec(1932, 800)];
        assert_eq!(shares(&rules, &econ)[0], Fixed::from_ratio(9, 10));
        // Under the threshold the share holds; above, it falls from the threshold.
        rules.falls_above = Fixed::from_int(15);
        econ.history = vec![rec(1925, 1000), rec(1950, 1400)];
        assert_eq!(shares(&rules, &econ)[0], Fixed::from_ratio(9, 10));
        econ.history = vec![rec(1925, 1000), rec(1980, 3000)];
        assert_eq!(shares(&rules, &econ)[0], Fixed::from_ratio(45, 100));
    }
}
