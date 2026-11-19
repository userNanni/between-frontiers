//! Customs duties (`game::Switches::customs_duties`, docs/trade.pt-BR.md, section 13): the tariff
//! a country charges on what it imports is collected by its treasury and makes imports dearer, so
//! that a country buys less abroad the higher its tariff. Off, the tariff only weighs on the
//! choice between foreign sellers (`trade::equilibrium`), as it always did.
//!
//! The parameters are data (`PurchasesFile::trade.customs`), so that each of the owner's
//! decisions is a datum switched on or off:
//! - `revenue`: the duty goes to the treasury (CT3);
//! - `elasticity`: how much less a country imports for a higher tariff (CT4; zero, no effect);
//! - `rates_1925`: the 1925 rate of each country (`diplomacy.ron`, `tariffs`) instead of the
//!   step of its trade law, the law moving it from there (CT5).

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::catalogue::Catalogue;
use super::state::Market;

/// What the duties read besides the world: the gap between each country's 1925 rate and the
/// tariff of its trade law of 1925, by market (zero without `rates_1925`).
#[derive(Clone, Debug, Default)]
pub struct Duties {
    pub offset: Vec<Fixed>,
}

impl Duties {
    /// The gaps for the countries of the scenario, in market order: `rates[c]` the 1925 rate of
    /// `diplomacy.ron` (none: no gap) and `law[c]` the tariff of the trade law in force in 1925.
    pub fn new(rates: &[Option<Fixed>], law: &[Fixed]) -> Self {
        Self {
            offset: rates
                .iter()
                .zip(law)
                .map(|(r, l)| r.map_or(Fixed::ZERO, |r| r - *l))
                .collect(),
        }
    }

    /// The tariff market `mi` charges this week: the one of its trade law, moved by its 1925
    /// gap, never below zero.
    pub fn rate(&self, mi: usize, m: &Market) -> Fixed {
        self.rate_of(mi, m.laws.effects.first().map_or(Fixed::ZERO, |e| e.tariff))
    }

    /// The tariff of market `mi` whose trade law asks `law`.
    pub fn rate_of(&self, mi: usize, law: Fixed) -> Fixed {
        (law + self.offset.get(mi).copied().unwrap_or(Fixed::ZERO)).max(Fixed::ZERO)
    }
}

/// What a country seeks abroad at tariff `t`, as a share of what it would seek without one:
/// `1 / (1 + ε t)`, which falls with the elasticity `ε` at a zero tariff and stays positive.
pub fn import_factor(t: Fixed, elasticity: Fixed) -> Fixed {
    Fixed::ONE / (Fixed::ONE + elasticity * t)
}

/// The duty each market collects in a week: its tariff on the value of what it imported. With
/// trade by pair, `imports` holds each buyer's imports of the week from every partner at base
/// prices (`Week::trade_by_pair`, the world price at the border), gross trade's exchange of
/// varieties included; without it, the net imports at the price its buyers paid. It leaves the
/// importers' money (the market's pool, where the sale of the imports went) for the treasury:
/// the world's money is the same.
pub fn duties(
    cat: &Catalogue,
    markets: &[Market],
    rates: &[Fixed],
    imports: Option<&[Fixed]>,
) -> Vec<Fixed> {
    markets
        .iter()
        .zip(rates)
        .enumerate()
        .map(|(mi, (m, &t))| {
            if t <= Fixed::ZERO {
                return Fixed::ZERO;
            }
            let value: Fixed = match imports {
                Some(by_buyer) => by_buyer.get(mi).copied().unwrap_or(Fixed::ZERO),
                None => (0..cat.goods.len())
                    .filter(|&g| m.net_import[g] > Fixed::ZERO)
                    .map(|g| m.net_import[g] * m.sell_ratio[g] * m.price[g])
                    .sum(),
            };
            value * t
        })
        .collect()
}

/// Each buyer's imports from every other market, from the pair values `seller × n + buyer`.
pub fn imports_by_buyer(pairs: &[Fixed], n: usize) -> Vec<Fixed> {
    let mut out = vec![Fixed::ZERO; n];
    if n == 0 || pairs.len() != n * n {
        return out;
    }
    for (k, v) in pairs.iter().enumerate() {
        let (s, b) = (k / n, k % n);
        if s != b && *v > Fixed::ZERO {
            out[b] += *v;
        }
    }
    out
}

/// Market `m` pays `duty`: from its pool, where the importers' money went, to its treasury,
/// counted in the week's revenue. Nothing is made or lost.
pub fn pay(m: &mut Market, duty: Fixed) {
    m.pool -= duty;
    m.treasury += duty;
    m.budget.revenue += duty;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::milli;

    #[test]
    fn a_table_fixes_goods_and_partners_and_hands_them_back() {
        let mut t = TariffTable::default();
        assert!(t.is_rule());
        t.set_good(7, Some(400));
        t.set_good(3, Some(0));
        t.set_partner(5, Some(-50));
        assert_eq!(t.goods, vec![(3, 0), (7, 400)]);
        assert_eq!(t.good(7), Some(milli(400)));
        assert_eq!(t.good(3), Some(Fixed::ZERO));
        assert_eq!(t.good(4), None);
        assert_eq!(t.partner(5), -milli(50));
        assert_eq!(t.partner(6), Fixed::ZERO);
        t.set_good(7, None);
        t.set_good(3, None);
        t.set_partner(5, Some(0));
        assert!(t.is_rule());
        let mut s = TariffState::default();
        s.table_mut(2).set_good(1, Some(100));
        assert!(!s.is_empty());
        assert!(s.table(0).is_none());
        assert_eq!(s.table(2).and_then(|t| t.good(1)), Some(milli(100)));
    }

    #[test]
    fn a_charge_reads_the_rule_the_table_and_the_partner() {
        let mut tables = TariffState::default();
        tables.table_mut(1).set_good(0, Some(400));
        tables.table_mut(1).set_partner(2, Some(100));
        tables.table_mut(1).set_partner(0, Some(-500));
        let c = Charge {
            rule: vec![milli(100), milli(250), milli(50)],
            tables: &tables,
            pairs: crate::econ::trade::PairRules::new(None, "1925-01-01", None, 3),
        };
        // Market 0 on the rule; market 1 fixed good 0 at 40 %, +10 points on 2, -50 on 0.
        assert_eq!(c.rate(1, 0, 0), milli(100));
        assert_eq!(c.rate(2, 1, 0), milli(500));
        assert_eq!(c.rate(2, 1, 3), milli(350));
        // A preference larger than the rate stops at zero.
        assert_eq!(c.rate(0, 1, 3), Fixed::ZERO);
    }

    #[test]
    fn a_higher_tariff_imports_less_and_none_imports_all() {
        let e = Fixed::ONE;
        assert_eq!(import_factor(Fixed::ZERO, e), Fixed::ONE);
        assert!(import_factor(milli(250), e) < import_factor(milli(100), e));
        assert_eq!(import_factor(milli(250), Fixed::ZERO), Fixed::ONE);
        assert!(import_factor(milli(600), e) > Fixed::ZERO);
    }

    #[test]
    fn the_1925_gap_moves_the_law_and_never_below_zero() {
        let d = Duties::new(&[Some(milli(90)), None], &[milli(20), milli(100)]);
        assert_eq!(d.offset, vec![milli(70), Fixed::ZERO]);
        assert_eq!(d.rate_of(0, milli(20)), milli(90));
        assert_eq!(d.rate_of(1, milli(20)), milli(20));
        // The law moves the rate from its 1925 level: protection adds its step.
        assert_eq!(d.rate_of(0, milli(250)), milli(320));
        let low = Duties::new(&[Some(Fixed::ZERO)], &[milli(250)]);
        assert_eq!(low.rate_of(0, milli(20)), Fixed::ZERO);
    }
}

/// What a government fixed of its tariff (owner's decision CT1 and CT2, docs/trade.pt-BR.md,
/// section 13): the goods it took off the historical rule, each with its rate, and the points it
/// adds to (surcharge) or takes off (preference) the tariff of every good for a partner. Empty:
/// every good on the rule, no partner apart.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TariffTable {
    /// `(good, rate in thousandths)`, ordered by good, one entry per good fixed.
    pub goods: Vec<(u16, u16)>,
    /// `(partner market, points in thousandths)`, ordered by partner: positive a surcharge,
    /// negative a preference.
    pub partners: Vec<(u16, i16)>,
}

impl TariffTable {
    pub fn is_rule(&self) -> bool {
        self.goods.is_empty() && self.partners.is_empty()
    }

    /// The rate fixed for `good`, if any.
    pub fn good(&self, good: usize) -> Option<Fixed> {
        let g = u16::try_from(good).ok()?;
        self.goods
            .binary_search_by_key(&g, |&(k, _)| k)
            .ok()
            .map(|i| Fixed::from_ratio(i64::from(self.goods[i].1), 1000))
    }

    /// The points added for `partner` (zero when none).
    pub fn partner(&self, partner: usize) -> Fixed {
        u16::try_from(partner)
            .ok()
            .and_then(|p| self.partners.binary_search_by_key(&p, |&(k, _)| k).ok())
            .map_or(Fixed::ZERO, |i| {
                Fixed::from_ratio(i64::from(self.partners[i].1), 1000)
            })
    }

    /// Fixes `good` at `per_mille`, or with `None` hands it back to the rule.
    pub fn set_good(&mut self, good: u16, per_mille: Option<u16>) {
        set_sorted(&mut self.goods, good, per_mille);
    }

    /// Sets the points for `partner`, or with `None` (or zero) drops them.
    pub fn set_partner(&mut self, partner: u16, per_mille: Option<i16>) {
        set_sorted(&mut self.partners, partner, per_mille.filter(|&v| v != 0));
    }
}

fn set_sorted<V: Copy>(list: &mut Vec<(u16, V)>, key: u16, value: Option<V>) {
    match (list.binary_search_by_key(&key, |&(k, _)| k), value) {
        (Ok(i), Some(v)) => list[i].1 = v,
        (Ok(i), None) => {
            list.remove(i);
        }
        (Err(i), Some(v)) => list.insert(i, (key, v)),
        (Err(_), None) => {}
    }
}

/// The governments' tariff tables, by market, and the customs revenue each collected last week
/// (the budget's line, owner's decision CT3). `GameWorld::tariffs`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TariffState {
    pub countries: Vec<TariffTable>,
    /// Last week's duties, by market; a record for the budget, read by nothing in the tick.
    pub revenue: Vec<Fixed>,
}

impl TariffState {
    /// Whether every government is on the rule: such a world hashes as before the tables.
    pub fn is_empty(&self) -> bool {
        self.countries.iter().all(TariffTable::is_rule)
    }

    pub fn table(&self, mi: usize) -> Option<&TariffTable> {
        self.countries.get(mi).filter(|t| !t.is_rule())
    }

    /// Market `mi`'s table, grown to hold it.
    pub fn table_mut(&mut self, mi: usize) -> &mut TariffTable {
        if self.countries.len() <= mi {
            self.countries.resize(mi + 1, TariffTable::default());
        }
        &mut self.countries[mi]
    }
}

/// The tables only: last week's revenue is a record that no rule reads.
impl HashState for TariffState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.countries.len() as u64);
        for t in &self.countries {
            h.write_u64(t.goods.len() as u64);
            for &(g, r) in &t.goods {
                h.write_u64(u64::from(g));
                h.write_u64(u64::from(r));
            }
            h.write_u64(t.partners.len() as u64);
            for &(p, v) in &t.partners {
                h.write_u64(u64::from(p));
                h.write_i64(i64::from(v));
            }
        }
    }
}

/// What a week's duties read: each market's rate by the rule, the governments' tables and the
/// relations in force by pair. The duty on a flow of good `g` from `seller` to `buyer` is its
/// value times the buyer's rate for `g` (fixed, or the rule's), plus the seller's points, under
/// the agreements in force (a customs union pays none), never below zero.
pub struct Charge<'a> {
    pub rule: Vec<Fixed>,
    pub tables: &'a TariffState,
    pub pairs: super::trade::PairRules,
}

impl Charge<'_> {
    pub fn rate(&self, seller: usize, buyer: usize, good: usize) -> Fixed {
        let rule = self.rule.get(buyer).copied().unwrap_or(Fixed::ZERO);
        let own = match self.tables.table(buyer) {
            Some(t) => (t.good(good).unwrap_or(rule) + t.partner(seller)).max(Fixed::ZERO),
            None => rule,
        };
        self.pairs.apply(seller, buyer, own)
    }
}
