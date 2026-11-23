//! The player's market good by good, for the economy window's goods tab: the price against the
//! base and the world's, what the country makes and uses, what crosses its border, and the
//! market's stock of a storable good. The market keeps last week's figures; they are shown a
//! year at that pace (week × 52), in dollars at base prices so the goods add up. Read once a
//! month.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// One good of the market.
#[derive(Clone, Debug, PartialEq)]
pub struct GoodRow {
    /// Index into the catalogue's goods.
    pub good: u16,
    pub key: String,
    pub category: String,
    /// The market's price over the base price, and the world's.
    pub price: f64,
    pub world: f64,
    /// A year at last week's pace, dollars at base prices.
    pub made: f64,
    pub used: f64,
    pub imports: f64,
    pub exports: f64,
    /// The market's stock of a storable good (`econ::stock`), at base prices; `None` for a
    /// good that does not keep.
    pub stock: Option<f64>,
    /// The price over the base, year by year, in the years the good had a market here
    /// (`econ::series`, from save 45).
    pub price_years: Vec<(i32, f64)>,
    /// Buildings whose methods make it (catalogue index and key), and the keys of those that
    /// take it.
    pub makers: Vec<(u16, String)>,
    pub takers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GoodsView {
    pub month: (i32, u8),
    pub goods: Vec<GoodRow>,
}

/// The player's market, good by good; `None` without a player or an economy. Goods not yet
/// available, and those with nothing made, used or traded, are left out.
pub fn read(world: &GameWorld, tag: &str, cat: &Catalogue, date: Date) -> Option<GoodsView> {
    let ci = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let m = world.econ.markets.get(ci)?;
    let year = |x: Fixed| f(x) * 52.0;
    let mut goods = Vec::new();
    for (g, d) in cat.goods.iter().enumerate() {
        if d.available_from > date.year {
            continue;
        }
        let base = f(d.base_price);
        let at = |q: f64| q * base;
        let net = m.net_import.get(g).copied().unwrap_or(Fixed::ZERO);
        let imports = year(net.max(Fixed::ZERO));
        let exports = year((-net).max(Fixed::ZERO));
        let supply = m.supply.get(g).map_or(0.0, |x| year(*x));
        let demand = m.demand.get(g).map_or(0.0, |x| year(*x));
        let met = m.buy_ratio.get(g).map_or(0.0, |x| f(*x));
        let made = (supply - imports).max(0.0);
        let used = ((demand - exports).max(0.0)) * met;
        let stock = d.stock.map(|_| at(f(world.stocks.of(ci, g))));
        if made <= 0.0
            && used <= 0.0
            && imports <= 0.0
            && exports <= 0.0
            && stock.is_none_or(|s| s <= 0.0)
        {
            continue;
        }
        let gi = g as u16;
        let makers = (0..cat.buildings.len())
            .filter(|&b| {
                cat.buildings[b]
                    .methods
                    .iter()
                    .any(|me| me.outputs.iter().any(|(o, _)| *o == gi))
            })
            .map(|b| (b as u16, cat.buildings[b].key.clone()))
            .collect();
        let takers = (0..cat.buildings.len())
            .filter(|&b| {
                cat.buildings[b]
                    .methods
                    .iter()
                    .any(|me| me.inputs.iter().any(|(i, _)| *i == gi))
            })
            .map(|b| cat.buildings[b].key.clone())
            .collect();
        let price = |p: Option<&Fixed>| {
            if base > 0.0 {
                p.map_or(1.0, |x| f(*x) / base)
            } else {
                1.0
            }
        };
        let price_years = world
            .series
            .prices
            .iter()
            .filter(|y| usize::from(y.market) == ci)
            .filter_map(|y| {
                let pm = *y.per_mille.get(g)?;
                (pm > 0).then(|| (y.year, f64::from(pm) / 1000.0))
            })
            .collect();
        goods.push(GoodRow {
            price_years,
            good: gi,
            key: d.key.clone(),
            category: d.category.clone(),
            price: price(m.price.get(g)),
            world: price(world.econ.world_price.get(g)),
            made: at(made),
            used: at(used),
            imports: at(imports),
            exports: at(exports),
            stock,
            makers,
            takers,
        });
    }
    Some(GoodsView {
        month: (date.year, date.month),
        goods,
    })
}
