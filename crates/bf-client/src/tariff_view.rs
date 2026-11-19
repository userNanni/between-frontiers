//! The player's tariffs, for the government's trade tab (`docs/trade.pt-BR.md`, section 13;
//! owner's decisions CT1 to CT3): the rate the historical rule gives (the trade law, moved by
//! the country's 1925 gap), the goods and partners the government took off it, what the country
//! imported of each good and from each partner, the agreements in force with each partner and
//! last week's customs revenue. Read in the simulation's thread when a week went by or the
//! table changed; the screen works out the rates from the table as the player sets it.

use bf_sim::econ::customs::{Duties, TariffTable};
use bf_sim::game::GameWorld;
use bf_sim::treaty::TariffTerm;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// One tradable good on the market this year.
#[derive(Clone, Debug, PartialEq)]
pub struct GoodLine {
    /// Index into the catalogue's goods.
    pub good: u16,
    /// What the country bought abroad last week, net of what it sold, at base prices (zero for
    /// a net exporter).
    pub imports: f64,
}

/// Goods of one category, in catalogue order.
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    /// The catalogue's key (`staple_food`).
    pub key: String,
    pub goods: Vec<GoodLine>,
}

/// Another country, as the player's tariff sees it.
#[derive(Clone, Debug, PartialEq)]
pub struct PartnerLine {
    /// Index into the countries (the map's and the simulation's order).
    pub country: usize,
    pub tag: String,
    /// What the player bought from it in the year of `TariffView::year`, at base prices.
    pub imports: f64,
    /// What the agreements in force do to the player's tariff on its goods.
    pub terms: Vec<TariffTerm>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TariffView {
    pub tag: String,
    /// Day the view was read on (`Date::to_days`).
    pub days: i64,
    /// The rate the historical rule gives every good, a fraction.
    pub rule: Fixed,
    /// The player's table as the simulation holds it.
    pub table: TariffTable,
    /// Key of the trade law in force (`protective_tariffs`), if any.
    pub law: Option<String>,
    pub categories: Vec<Category>,
    /// Every other country, the largest seller to the player first.
    pub partners: Vec<PartnerLine>,
    /// The year the partners' imports are of; `so_far`: the year running.
    pub year: i32,
    pub so_far: bool,
    /// Last week's customs revenue and last year's GDP.
    pub revenue: f64,
    pub gdp: f64,
    /// Whether the tariff that counts is on (`customs_duties`): off, the table weighs only on
    /// the choice between sellers and nothing goes to the treasury.
    pub on: bool,
}

/// Partners listed without a search: the largest by imports.
pub const TOP_PARTNERS: usize = 10;

/// What the view reads besides the world.
pub struct Sources<'a> {
    pub catalogue: &'a bf_sim::econ::Catalogue,
    pub politics: Option<&'a bf_sim::politics::Politics>,
    /// The 1925 gaps (`Rules::duties`); none: the trade law alone.
    pub duties: Option<&'a Duties>,
}

/// The player's rate by the rule this week.
fn rule_of(world: &GameWorld, ci: usize, duties: Option<&Duties>) -> Option<Fixed> {
    let m = world.econ.markets.get(ci)?;
    Some(match duties {
        Some(d) => d.rate(ci, m),
        None => Duties::default().rate(ci, m),
    })
}

/// Whether a view read before still holds: the same player, rule and table, within the week.
pub fn fresh(last: &TariffView, world: &GameWorld, date: Date, src: &Sources<'_>) -> bool {
    let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == last.tag) else {
        return false;
    };
    world.player_tag() == Some(last.tag.as_str())
        && date.to_days() < last.days + 7
        && rule_of(world, ci, src.duties) == Some(last.rule)
        && world.tariffs.countries.get(ci).cloned().unwrap_or_default() == last.table
}

/// The player's tariffs; `None` without a player or an economy.
pub fn read(world: &GameWorld, date: Date, src: &Sources<'_>) -> Option<TariffView> {
    let tag = world.player_tag()?.to_owned();
    let ci = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let m = world.econ.markets.get(ci)?;
    let cat = src.catalogue;
    let rule = rule_of(world, ci, src.duties)?;
    let mut categories: Vec<Category> = Vec::new();
    for (g, good) in cat.goods.iter().enumerate() {
        if !good.tradable || good.available_from > date.year {
            continue;
        }
        let net = m.net_import.get(g).copied().unwrap_or(Fixed::ZERO);
        let line = GoodLine {
            good: g as u16,
            imports: (net.max(Fixed::ZERO) * good.base_price).to_f64_lossy(),
        };
        match categories.iter_mut().find(|c| c.key == good.category) {
            Some(c) => c.goods.push(line),
            None => categories.push(Category {
                key: good.category.clone(),
                goods: vec![line],
            }),
        }
    }
    // The trade law: the law in force whose effects set the tariff.
    let law = src.politics.and_then(|p| {
        let e = bf_sim::politics::effect_index("tariff")?;
        let laws = world.politics.laws_in_force(ci);
        laws.iter().flatten().find_map(|&l| {
            let law = p.laws.get(usize::from(l))?;
            law.effects
                .iter()
                .any(|(k, _)| usize::from(*k) == e)
                .then(|| law.key.clone())
        })
    });
    // Imports by partner: the last full year, or the year so far before the first one closed.
    let t = &world.econ.trade;
    let n = usize::from(t.n);
    let any = |v: &[Fixed]| v.iter().any(|x| x.is_positive());
    let (flows, year, so_far) = if any(&t.last_year) {
        (&t.last_year[..], date.year - 1, false)
    } else {
        (&t.year[..], date.year, true)
    };
    let tags: Vec<String> = world.countries.iter().map(|(_, c)| c.tag.clone()).collect();
    let day = format!("{:04}-{:02}-{:02}", date.year, date.month, date.day);
    let links = world.treaties.tariff_relations(&tags, &day);
    let mut partners: Vec<PartnerLine> = tags
        .iter()
        .enumerate()
        .filter(|(p, _)| *p != ci)
        .map(|(p, tag)| PartnerLine {
            country: p,
            tag: tag.clone(),
            imports: if p < n && ci < n {
                flows.get(p * n + ci).map_or(0.0, |x| x.to_f64_lossy())
            } else {
                0.0
            },
            terms: links
                .iter()
                .filter(|l| usize::from(l.giver) == ci && usize::from(l.receiver) == p)
                .map(|l| l.rule)
                .collect(),
        })
        .collect();
    partners.sort_by(|a, b| {
        b.imports
            .total_cmp(&a.imports)
            .then(a.country.cmp(&b.country))
    });
    Some(TariffView {
        tag,
        days: date.to_days(),
        rule,
        table: world.tariffs.countries.get(ci).cloned().unwrap_or_default(),
        law,
        categories,
        partners,
        year,
        so_far,
        revenue: world
            .tariffs
            .revenue
            .get(ci)
            .map_or(0.0, |r| r.to_f64_lossy()),
        gdp: m.gdp_last_year.to_f64_lossy(),
        on: src.duties.is_some(),
    })
}

/// The rate of `good` under `table`: fixed, or the rule's.
pub fn good_rate(table: &TariffTable, rule: Fixed, good: usize) -> Fixed {
    table.good(good).unwrap_or(rule)
}

impl TariffView {
    /// Every good shown, in category order.
    pub fn goods(&self) -> impl Iterator<Item = &GoodLine> {
        self.categories.iter().flat_map(|c| c.goods.iter())
    }

    /// The mean of the goods' rates under `table`, each good weighing the same.
    pub fn mean(&self, table: &TariffTable) -> Fixed {
        let (sum, k) = self
            .goods()
            .map(|g| good_rate(table, self.rule, usize::from(g.good)))
            .fold((Fixed::ZERO, 0i64), |(s, k), r| (s + r, k + 1));
        if k == 0 { self.rule } else { sum / k }
    }

    /// What a partner's goods pay on average under `table`: each good's rate plus the
    /// partner's points, never below zero, under the agreements in force, each good weighing
    /// the same (as `customs::Charge::rate`).
    pub fn effective(&self, table: &TariffTable, p: &PartnerLine) -> Fixed {
        let points = table.partner(p.country);
        let (sum, k) = self
            .goods()
            .map(|g| {
                let own =
                    (good_rate(table, self.rule, usize::from(g.good)) + points).max(Fixed::ZERO);
                p.terms.iter().fold(own, |t, term| term.apply(t))
            })
            .fold((Fixed::ZERO, 0i64), |(s, k), r| (s + r, k + 1));
        if k == 0 { Fixed::ZERO } else { sum / k }
    }

    /// The customs revenue the table would collect in the year of `year` (or so far), from every
    /// partner's imports at its effective tariff: what the screen shows while the tariff that
    /// counts is off.
    pub fn would_collect(&self, table: &TariffTable) -> f64 {
        self.partners
            .iter()
            .map(|p| p.imports * self.effective(table, p).to_f64_lossy())
            .sum()
    }

    /// What the agreements with `p` hold the adjustment to, when they cut what it asks: the
    /// cap (`Some(cap)`) or a customs union (`Some(0)`). `None` when the adjustment passes.
    pub fn limit(&self, table: &TariffTable, p: &PartnerLine) -> Option<Fixed> {
        let points = table.partner(p.country);
        if points <= Fixed::ZERO {
            return None;
        }
        if p.terms.contains(&TariffTerm::Union) {
            return Some(Fixed::ZERO);
        }
        let cap = p
            .terms
            .iter()
            .filter_map(|t| match t {
                TariffTerm::Cap(c) => Some(*c),
                _ => None,
            })
            .min()?;
        // Some good goes past the cap once the points are added.
        self.goods()
            .any(|g| good_rate(table, self.rule, usize::from(g.good)) + points > cap)
            .then_some(cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn milli(v: i64) -> Fixed {
        Fixed::from_ratio(v, 1000)
    }

    fn view(terms: Vec<TariffTerm>) -> TariffView {
        TariffView {
            tag: "BRA".into(),
            days: 0,
            rule: milli(260),
            table: TariffTable::default(),
            law: None,
            categories: vec![Category {
                key: "staple_food".into(),
                goods: (0..2)
                    .map(|g| GoodLine {
                        good: g,
                        imports: 0.0,
                    })
                    .collect(),
            }],
            partners: vec![PartnerLine {
                country: 1,
                tag: "ARG".into(),
                imports: 1.0,
                terms,
            }],
            year: 1925,
            so_far: true,
            revenue: 0.0,
            gdp: 1.0,
        }
    }

    #[test]
    fn a_fixed_good_moves_the_mean_and_the_partner_s_rate() {
        let v = view(Vec::new());
        let mut t = TariffTable::default();
        assert_eq!(v.mean(&t), milli(260));
        t.set_good(1, Some(400));
        assert_eq!(v.mean(&t), milli(330));
        t.set_partner(1, Some(-50));
        assert_eq!(v.effective(&t, &v.partners[0]), milli(280));
        assert_eq!(v.limit(&t, &v.partners[0]), None);
    }

    #[test]
    fn the_agreements_hold_the_surcharge_and_say_so() {
        let v = view(vec![TariffTerm::Cap(milli(150))]);
        let mut t = TariffTable::default();
        assert_eq!(v.effective(&t, &v.partners[0]), milli(150));
        t.set_partner(1, Some(100));
        assert_eq!(v.effective(&t, &v.partners[0]), milli(150));
        assert_eq!(v.limit(&t, &v.partners[0]), Some(milli(150)));
        let u = view(vec![TariffTerm::Union]);
        assert_eq!(u.effective(&t, &u.partners[0]), Fixed::ZERO);
        assert_eq!(u.limit(&t, &u.partners[0]), Some(Fixed::ZERO));
        let m = view(vec![TariffTerm::Margin(milli(50))]);
        t.set_partner(1, Some(-300));
        assert_eq!(m.effective(&t, &m.partners[0]), Fixed::ZERO);
    }
}
