//! Goods made and used in the state: transport (railway, road, port) and, until the country
//! builds a national grid, electricity. Each state first uses what it makes; part of a surplus
//! reaches the neighbouring states of the same market over land; what is still left serves the
//! buyers that are not in any state (the state's own purchases, upkeep). A state short of
//! transport pays more for it, and its farms, mines and factories reach the market only in part.

use laplace_schelling::Fixed;

use super::catalogue::{Catalogue, GoodIx};
use super::state::{Economy, LocalGood, Market, Region};
use super::{PriceBand, fill, milli};
use crate::research::{ResearchState, Stage, TechTree};

/// Share of a state's surplus offered to its neighbours each week.
pub const SPILL: Fixed = milli(500);
/// A plant that ships goods loses at least this share of its output for the share of its
/// freight it could not get: without a railway, the mine's ore stays at the mine.
pub const ACCESS_WEIGHT: Fixed = milli(350);
/// Local prices swing less than the market's: 30%–300% of base.
const PRICES: PriceBand = PriceBand {
    floor: milli(300),
    ceiling: milli(3000),
};

/// For each market and each local good (`Catalogue::local` order): whether it already flows
/// across the whole country, like any other good.
pub fn national(
    cat: &Catalogue,
    tree: Option<&TechTree>,
    research: &ResearchState,
    markets: usize,
) -> Vec<Vec<bool>> {
    let techs: Vec<Option<usize>> = cat
        .local
        .iter()
        .map(|&g| {
            let key = cat.goods[g as usize].national_with.as_deref()?;
            tree?.tech(key)
        })
        .collect();
    (0..markets)
        .map(|mi| {
            techs
                .iter()
                .map(|t| {
                    t.is_some_and(|t| {
                        research
                            .stage
                            .get(mi)
                            .and_then(|s| s.get(t))
                            .is_some_and(|s| *s >= Stage::Industrial)
                    })
                })
                .collect()
        })
        .collect()
}

/// Prices, shares bought and shares sold as one state sees them: the market's, with the local
/// goods of the state in their place.
pub struct View {
    pub price: Vec<Fixed>,
    pub buy: Vec<Fixed>,
    pub sell: Vec<Fixed>,
}

impl View {
    /// The market's figures with `local` (in `Catalogue::local` order) in place of the local
    /// goods, except those `national` says flow across the whole country.
    pub fn new(cat: &Catalogue, m: &Market, local: &[LocalGood], national: &[bool]) -> Self {
        let mut v = Self {
            price: m.price.clone(),
            buy: m.buy_ratio.clone(),
            sell: m.sell_ratio.clone(),
        };
        for (li, (&g, l)) in cat.local.iter().zip(local).enumerate() {
            if national.get(li).copied().unwrap_or(false) {
                continue;
            }
            let g = g as usize;
            v.price[g] = l.price;
            v.buy[g] = l.buy;
            v.sell[g] = l.sell;
        }
        v
    }

    /// What the state saw last week: the prices people plan with.
    pub fn last_week(cat: &Catalogue, m: &Market, r: &Region, national: &[bool]) -> Self {
        Self::new(cat, m, &r.local, national)
    }
}

/// The outcome of one week, for every region (global order).
pub struct Cleared {
    /// New state of each local good in each region.
    pub regions: Vec<Vec<LocalGood>>,
    /// Share of each region's supply sold to its own people and its neighbours, before the
    /// country-wide buyers take part of what is left.
    pub sold_here: Vec<Vec<Fixed>>,
    /// What is left after the neighbours, offered to the country-wide buyers.
    pub leftover: Vec<Vec<Fixed>>,
}

/// Clears the local goods state by state. `demand` and `supply` are each region's, from its
/// plants and people; `national` from [`national`].
pub fn clear(
    cat: &Catalogue,
    econ: &Economy,
    demand: &[&[Fixed]],
    supply: &[&[Fixed]],
    national: &[Vec<bool>],
) -> Cleared {
    let nr = econ.regions.len();
    let nl = cat.local.len();
    let mut regions = vec![vec![LocalGood::default(); nl]; nr];
    let mut sold_here = vec![vec![Fixed::ZERO; nl]; nr];
    let mut leftover = vec![vec![Fixed::ZERO; nl]; nr];
    for (li, &g) in cat.local.iter().enumerate() {
        let g = g as usize;
        let base = cat.goods[g].base_price;
        let mut own = vec![Fixed::ZERO; nr];
        let mut surplus = vec![Fixed::ZERO; nr];
        let mut deficit = vec![Fixed::ZERO; nr];
        for ri in 0..nr {
            let (d, s) = (demand[ri][g], supply[ri][g]);
            own[ri] = d.min(s);
            surplus[ri] = s - own[ri];
            deficit[ri] = d - own[ri];
        }
        // Neighbours: each surplus is offered to the neighbours short of the good, pro rata to
        // their shortage; a neighbour offered more than it lacks takes what it lacks, and every
        // offer to it shrinks alike. Nothing depends on the order of the regions.
        let mut offered_to = vec![Fixed::ZERO; nr];
        let shares = |ri: usize| -> Vec<(usize, Fixed)> {
            let r = &econ.regions[ri];
            let short: Vec<usize> = r
                .neighbours
                .iter()
                .map(|&n| n as usize)
                .filter(|&n| deficit[n] > Fixed::ZERO)
                .collect();
            let total: Fixed = short.iter().map(|&n| deficit[n]).sum();
            if total <= Fixed::ZERO || surplus[ri] <= Fixed::ZERO {
                return Vec::new();
            }
            let offer = surplus[ri] * SPILL;
            short
                .into_iter()
                .map(|n| (n, offer * deficit[n] / total))
                .collect()
        };
        let skip = |ri: usize| national[econ.regions[ri].market as usize][li];
        for ri in 0..nr {
            if skip(ri) {
                continue;
            }
            for (n, q) in shares(ri) {
                offered_to[n] += q;
            }
        }
        let taken: Vec<Fixed> = (0..nr).map(|n| fill(deficit[n], offered_to[n])).collect();
        let mut given = vec![Fixed::ZERO; nr];
        let mut received = vec![Fixed::ZERO; nr];
        for (ri, gave) in given.iter_mut().enumerate() {
            if skip(ri) {
                continue;
            }
            for (n, q) in shares(ri) {
                let q = q * taken[n];
                *gave += q;
                received[n] += q;
            }
        }
        for ri in 0..nr {
            if skip(ri) {
                continue;
            }
            let r = &econ.regions[ri];
            let (d, s) = (demand[ri][g], supply[ri][g]);
            let unmet = deficit[ri] - received[ri];
            let left = surplus[ri] - given[ri];
            leftover[ri][li] = left;
            sold_here[ri][li] = fill(own[ri] + given[ri], s);
            let old = r.local.get(li).map_or(base, |l| l.price);
            // What reaches the state: its own supply and its neighbours'.
            let at_hand = s + received[ri];
            regions[ri][li] = LocalGood {
                price: PRICES.next(old, base, at_hand, d, || {
                    (unmet - left) / d.min(at_hand).max(Fixed::EPSILON)
                }),
                buy: fill(own[ri] + received[ri], d),
                sell: sold_here[ri][li],
                demand: d,
            };
        }
    }
    Cleared {
        regions,
        sold_here,
        leftover,
    }
}

/// Folds the country-wide buyers into each region's sales and sets the market's figures of the
/// local goods: totals, and the demand-weighted average price.
pub fn finish(
    cat: &Catalogue,
    econ: &Economy,
    markets: &mut [Market],
    cleared: &mut Cleared,
    demand: &[&[Fixed]],
    supply: &[&[Fixed]],
    national: &[Vec<bool>],
) {
    let nm = markets.len();
    for (li, &g) in cat.local.iter().enumerate() {
        let g = g as usize;
        let mut d_total = vec![Fixed::ZERO; nm];
        let mut s_total = vec![Fixed::ZERO; nm];
        let mut weighted = vec![Fixed::ZERO; nm];
        for (ri, r) in econ.regions.iter().enumerate() {
            let mi = r.market as usize;
            let m = &markets[mi];
            if national[mi][li] {
                // The grid: the state sees the market.
                cleared.regions[ri][li] = LocalGood {
                    price: m.price[g],
                    buy: m.buy_ratio[g],
                    sell: m.sell_ratio[g],
                    demand: demand[ri][g],
                };
                continue;
            }
            let s = supply[ri][g];
            let l = &mut cleared.regions[ri][li];
            l.sell = fill(
                cleared.sold_here[ri][li] * s + cleared.leftover[ri][li] * m.sell_ratio[g],
                s,
            );
            d_total[mi] += demand[ri][g];
            s_total[mi] += s;
            weighted[mi] += demand[ri][g] * l.price;
        }
        for (mi, m) in markets.iter_mut().enumerate() {
            if national[mi][li] {
                continue;
            }
            // The market shows the whole country: its states' demand and the country-wide
            // buyers', all the supply.
            m.demand[g] += d_total[mi];
            m.supply[g] = s_total[mi];
            if d_total[mi] > Fixed::ZERO {
                m.price[g] = weighted[mi] / d_total[mi];
            }
        }
    }
}

/// The local good a building makes, if it makes one that clears state by state.
pub fn local_output(cat: &Catalogue, kind: usize) -> Option<usize> {
    let g = cat.buildings[kind].methods[0].outputs.first()?.0;
    cat.local.iter().position(|&l| l == g)
}

/// Index of `g` among the local goods.
pub fn local_index(cat: &Catalogue, g: GoodIx) -> Option<usize> {
    cat.local.iter().position(|&l| l == g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::catalogue::Good;
    use crate::econ::state::empty_pops;

    fn cat() -> Catalogue {
        Catalogue {
            goods: vec![Good {
                key: "transportation".into(),
                category: "service".into(),
                base_price: Fixed::from_int(1000),
                available_from: 1925,
                tradable: false,
                local: true,
                national_with: None,
            }],
            local: vec![0],
            freight: Some(0),
            ..Catalogue::default()
        }
    }

    fn region(neighbours: Vec<u32>) -> Region {
        Region {
            market: 0,
            colony: false,
            pops: empty_pops(),
            plants: Vec::new(),
            literacy: Fixed::ZERO,
            deposits: Vec::new(),
            land: Vec::new(),
            frontier: Vec::new(),
            wild_km2: [0; 2],
            cleared_km2: [0; 2],
            subsistence: Fixed::ZERO,
            tfp: Fixed::ONE,
            tfp_start: Fixed::ONE,
            state: 0,
            neighbours,
            local: Vec::new(),
        }
    }

    fn run(neighbours: [Vec<u32>; 3], d: [i64; 3], s: [i64; 3]) -> Cleared {
        let econ = Economy {
            regions: neighbours.into_iter().map(region).collect(),
            markets: vec![Market::empty(1)],
            ..Economy::default()
        };
        let d: Vec<Vec<Fixed>> = d.iter().map(|v| vec![Fixed::from_int(*v)]).collect();
        let s: Vec<Vec<Fixed>> = s.iter().map(|v| vec![Fixed::from_int(*v)]).collect();
        let dr: Vec<&[Fixed]> = d.iter().map(|v| &v[..]).collect();
        let sr: Vec<&[Fixed]> = s.iter().map(|v| &v[..]).collect();
        clear(&cat(), &econ, &dr, &sr, &[vec![false]])
    }

    #[test]
    fn a_surplus_reaches_the_neighbours_short_of_it() {
        // 0 has 100 spare; 1 lacks 40, 2 lacks 10; half of the surplus is offered.
        let c = run([vec![1, 2], vec![0], vec![0]], [0, 40, 10], [100, 0, 0]);
        assert_eq!(c.regions[1][0].buy, Fixed::ONE);
        assert_eq!(c.regions[2][0].buy, Fixed::ONE);
        assert_eq!(c.leftover[0][0], Fixed::from_int(50));
    }

    #[test]
    fn neighbours_share_a_small_surplus_by_their_shortage() {
        let c = run([vec![1, 2], vec![0], vec![0]], [0, 60, 20], [40, 0, 0]);
        // 20 offered: 15 to the one lacking 60, 5 to the one lacking 20.
        assert_eq!(c.regions[1][0].buy, Fixed::from_ratio(15, 60));
        assert_eq!(c.regions[2][0].buy, Fixed::from_ratio(5, 20));
    }

    #[test]
    fn without_a_border_nothing_arrives_and_the_price_climbs() {
        let c = run([vec![], vec![], vec![]], [0, 60, 0], [100, 0, 0]);
        assert_eq!(c.regions[1][0].buy, Fixed::ZERO);
        assert!(c.regions[1][0].price > Fixed::from_int(1000));
        assert!(c.regions[0][0].price < Fixed::from_int(1000));
    }

    #[test]
    fn the_order_of_the_regions_does_not_matter() {
        let a = run([vec![1], vec![0, 2], vec![1]], [10, 0, 30], [0, 50, 0]);
        let b = run([vec![1], vec![0, 2], vec![1]], [30, 0, 10], [0, 50, 0]);
        assert_eq!(a.regions[0][0].buy, b.regions[2][0].buy);
        assert_eq!(a.regions[2][0].buy, b.regions[0][0].buy);
    }
}
