//! Trade by partner (`docs/trade.pt-BR.md`), for the country panel: what each country sold to
//! and bought from each other in the last full year (the year so far before the first one
//! closes), at base prices, its own tariff, and the tariff relations of the agreements in force
//! between each pair (customs union, preference, cap). Read once a month in the simulation's
//! thread, for every country; `None` with trade by pair off.

use bf_sim::game::GameWorld;
use bf_sim::treaty::TariffTerm;
use laplace_schelling::calendar::Date;

/// Partners shown for each country, the largest first.
pub const PARTNERS: usize = 8;

/// One partner of a country.
#[derive(Clone, Debug, PartialEq)]
pub struct Partner {
    /// Index into the countries (the map's and the simulation's order).
    pub country: usize,
    /// What the country sold to the partner, and bought from it.
    pub exports: f64,
    pub imports: f64,
    /// What the agreements in force do to the country's tariff on the partner's goods.
    pub charges: Vec<TariffTerm>,
    /// What they do to the partner's tariff on the country's goods.
    pub charged: Vec<TariffTerm>,
}

impl Partner {
    /// Inside one customs territory: no tariff either way.
    pub fn union(&self) -> bool {
        self.charges.contains(&TariffTerm::Union)
    }
}

/// One country's trade.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CountryTrade {
    pub exports: f64,
    pub imports: f64,
    /// Its own tariff, a fraction, before the agreements.
    pub tariff: f64,
    pub partners: Vec<Partner>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TradeView {
    pub month: (i32, u8),
    /// The year the figures are of; `so_far`: the year running, before the first one closed.
    pub year: i32,
    pub so_far: bool,
    pub countries: Vec<CountryTrade>,
}

/// Every country's trade; `None` before trade by pair has anything to show.
pub fn read(world: &GameWorld, date: Date) -> Option<TradeView> {
    let t = &world.econ.trade;
    let n = usize::from(t.n);
    if n == 0 {
        return None;
    }
    // The last full year, or the year so far before the first one closed.
    let any = |v: &[laplace_schelling::Fixed]| v.iter().any(|x| x.is_positive());
    let (flows, year, so_far) = if any(&t.last_year) {
        (&t.last_year, date.year - 1, false)
    } else if any(&t.year) {
        (&t.year, date.year, true)
    } else {
        return None;
    };
    let at = |s: usize, b: usize| flows.get(s * n + b).map_or(0.0, |x| x.to_f64_lossy());
    let tags: Vec<String> = world.countries.iter().map(|(_, c)| c.tag.clone()).collect();
    let day = format!("{:04}-{:02}-{:02}", date.year, date.month, date.day);
    let links = world.treaties.tariff_relations(&tags, &day);
    let terms = |giver: usize, receiver: usize| -> Vec<TariffTerm> {
        links
            .iter()
            .filter(|l| usize::from(l.giver) == giver && usize::from(l.receiver) == receiver)
            .map(|l| l.rule)
            .collect()
    };
    let countries = (0..n.min(tags.len()))
        .map(|c| {
            let mut partners: Vec<Partner> = (0..n)
                .filter(|p| *p != c)
                .map(|p| Partner {
                    country: p,
                    exports: at(c, p),
                    imports: at(p, c),
                    charges: Vec::new(),
                    charged: Vec::new(),
                })
                .filter(|p| p.exports > 0.0 || p.imports > 0.0)
                .collect();
            partners.sort_by(|a, b| {
                (b.exports + b.imports)
                    .total_cmp(&(a.exports + a.imports))
                    .then(a.country.cmp(&b.country))
            });
            partners.truncate(PARTNERS);
            for p in &mut partners {
                p.charges = terms(c, p.country);
                p.charged = terms(p.country, c);
            }
            CountryTrade {
                exports: (0..n).map(|b| at(c, b)).sum(),
                imports: (0..n).map(|s| at(s, c)).sum(),
                tariff: world
                    .econ
                    .markets
                    .get(c)
                    .and_then(|m| m.laws.effects.first())
                    .map_or(0.0, |e| e.tariff.to_f64_lossy()),
                partners,
            }
        })
        .collect();
    Some(TradeView {
        month: (date.year, date.month),
        year,
        so_far,
        countries,
    })
}
