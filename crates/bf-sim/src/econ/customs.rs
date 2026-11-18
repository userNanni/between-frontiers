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

/// The duty each market collects in a week: its tariff on the value of what it imported, at
/// the price its buyers paid. It leaves the importers' money (the market's pool, where the sale
/// of the imports went) for the treasury: the world's money is the same.
pub fn duties(cat: &Catalogue, markets: &[Market], rates: &[Fixed]) -> Vec<Fixed> {
    markets
        .iter()
        .zip(rates)
        .map(|(m, &t)| {
            if t <= Fixed::ZERO {
                return Fixed::ZERO;
            }
            let value: Fixed = (0..cat.goods.len())
                .filter(|&g| m.net_import[g] > Fixed::ZERO)
                .map(|g| m.net_import[g] * m.sell_ratio[g] * m.price[g])
                .sum();
            value * t
        })
        .collect()
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
