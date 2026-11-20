//! Stocks by market (`game::Switches::market_stock`; owner's decision U4): a good marked
//! storable in the catalogue (`GoodDef::stock`) that finds no buyer is kept instead of lost.
//! The by-product of a mine run for another good (the uranium of the radium mines of 1925)
//! piles up, and a buyer that comes later (a nuclear programme) finds it.
//!
//! Each economy week, before trade:
//! - the stock joins the market's supply, so trade can carry it abroad and buyers at home take
//!   it with the new output;
//! - after the clearing, the stock sells at the share the whole supply sold
//!   (`Market::sell_ratio`), and what it fetched goes to the market's pool, as the goods that
//!   came in from abroad do (`week::margin_transfers`): the buyers paid for it and no plant sold
//!   it, so the money stays in the market;
//! - what is left unsold (stock and new output) is the new stock, up to `weeks` of the market's
//!   own output of the good; a stock already above that is not cut, only kept from growing, so
//!   a mine that closes leaves its pile behind;
//! - the good's price does not fall below `floor` times its base price.
//!
//! The stock is goods, not money: keeping it makes or destroys no money.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::Market;
use super::catalogue::Catalogue;

/// `GameWorld::stocks`: by market, the stock of each storable good (good index, quantity),
/// ascending by good. Empty while `Switches::market_stock` is off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockState {
    pub markets: Vec<Vec<(u16, Fixed)>>,
}

impl StockState {
    pub fn is_empty(&self) -> bool {
        self.markets.is_empty()
    }

    /// The stock of good `g` in market `mi`.
    pub fn of(&self, mi: usize, g: usize) -> Fixed {
        self.markets
            .get(mi)
            .and_then(|s| {
                s.binary_search_by_key(&(g as u16), |x| x.0)
                    .ok()
                    .map(|k| s[k].1)
            })
            .unwrap_or(Fixed::ZERO)
    }
}

impl HashState for StockState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.markets.len() as u64);
        for s in &self.markets {
            h.write_u64(s.len() as u64);
            for &(g, q) in s {
                h.write_u16(g);
                q.hash_state(h);
            }
        }
    }
}

/// Adds each market's stock to its supply (before trade).
pub fn offer(cat: &Catalogue, stocks: &StockState, supply: &mut [Vec<Fixed>]) {
    for (mi, s) in supply.iter_mut().enumerate() {
        for (g, good) in cat.goods.iter().enumerate() {
            if good.stock.is_some() {
                s[g] += stocks.of(mi, g);
            }
        }
    }
}

/// After the clearing of market `mi`: sells the stock at the market's share sold, pays what it
/// fetched into the pool, keeps what is left up to the cap and holds the price at the floor.
/// `made`: the market's own output of each good this week (before the stock and trade).
/// Returns the market's new stock.
pub fn settle(
    cat: &Catalogue,
    stocks: &StockState,
    mi: usize,
    made: &[Fixed],
    m: &mut Market,
) -> Vec<(u16, Fixed)> {
    let mut next = Vec::new();
    for (g, good) in cat.goods.iter().enumerate() {
        let Some((weeks, floor)) = good.stock else {
            continue;
        };
        let old = stocks.of(mi, g);
        let sold = old * m.sell_ratio[g];
        m.pool += sold * m.price[g];
        let unsold = (m.supply[g] - m.supply[g] * m.sell_ratio[g]).max(Fixed::ZERO);
        let cap = (made[g] * i64::from(weeks)).max(old);
        let kept = unsold.min(cap);
        if kept > Fixed::ZERO {
            next.push((g as u16, kept));
        }
        m.price[g] = m.price[g].max(good.base_price * floor);
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat() -> Catalogue {
        let good = |key: &str, stock| super::super::catalogue::Good {
            key: key.into(),
            category: "mineral".into(),
            base_price: Fixed::from_int(100),
            available_from: 1925,
            tradable: true,
            local: false,
            national_with: None,
            stock,
        };
        Catalogue {
            goods: vec![
                good("coal", None),
                good("uranium_ore", Some((10, Fixed::from_ratio(1, 10)))),
            ],
            ..Catalogue::default()
        }
    }

    #[test]
    fn the_stock_sells_with_the_supply_and_keeps_the_rest_up_to_the_cap() {
        let cat = cat();
        let stocks = StockState {
            markets: vec![vec![(1, Fixed::from_int(10))]],
        };
        let mut m = Market::empty(2);
        // 10 in stock and 20 made: 30 offered, half sold at 2 (under the floor of 10).
        m.supply = vec![Fixed::from_int(50), Fixed::from_int(30)];
        m.sell_ratio = vec![Fixed::ONE / 2, Fixed::ONE / 2];
        m.price = vec![Fixed::from_int(100), Fixed::from_int(2)];
        let made = [Fixed::from_int(50), Fixed::from_int(20)];
        let next = settle(&cat, &stocks, 0, &made, &mut m);
        // The stock's half sold went to the pool at the week's price.
        assert_eq!(m.pool, Fixed::from_int(10));
        // 15 unsold, under the cap of 10 weeks × 20.
        assert_eq!(next, [(1, Fixed::from_int(15))]);
        // The price is held at the floor; the good with no stock is left alone.
        assert_eq!(m.price[1], Fixed::from_int(10));
        assert_eq!(m.price[0], Fixed::from_int(100));
        // A mine that stops: the pile is not cut by the cap, only kept from growing.
        let big = StockState {
            markets: vec![vec![(1, Fixed::from_int(500))]],
        };
        let mut m = Market::empty(2);
        m.supply = vec![Fixed::ZERO, Fixed::from_int(500)];
        m.price = vec![Fixed::from_int(100), Fixed::from_int(10)];
        let next = settle(&cat, &big, 0, &[Fixed::ZERO, Fixed::ZERO], &mut m);
        assert_eq!(next, [(1, Fixed::from_int(500))]);
    }

    #[test]
    fn the_stock_is_found_by_good() {
        let s = StockState {
            markets: vec![vec![(2, Fixed::from_int(5)), (7, Fixed::ONE)]],
        };
        assert_eq!(s.of(0, 2), Fixed::from_int(5));
        assert_eq!(s.of(0, 7), Fixed::ONE);
        assert_eq!(s.of(0, 3), Fixed::ZERO);
        assert_eq!(s.of(1, 2), Fixed::ZERO);
    }
}
