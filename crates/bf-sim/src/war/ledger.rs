//! The war ledger (decision 7 of `docs/war.pt-BR.md`): what the war costs a country, week by
//! week and in total, always on screen. People (dead and invalid by age), goods and equipment,
//! money and debt, output that never existed, and prices. A long war costs the winner too: the
//! ledger feeds the spending and price terms of [`crate::war::exhaustion`], which grow with the
//! weeks even when nothing is lost on the field.

use laplace_schelling::Fixed;

use super::calc::AgeBand;

/// One week of war for one country. Money in the country's currency at the week's prices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WarWeek {
    /// Soldiers killed, by age band.
    pub killed: Vec<AgeBand>,
    /// Soldiers who will not work again.
    pub invalid: u64,
    /// Civilians killed: bombing, siege, hunger under blockade, occupation.
    pub civilians_killed: u64,
    /// Value of the goods the forces consumed (food, fuel, ammunition, maintenance).
    pub goods: Fixed,
    /// Value of the equipment lost.
    pub equipment_lost: Fixed,
    /// War spending of the state (pay, purchases, transport).
    pub spending: Fixed,
    /// New debt the state took to pay it.
    pub borrowed: Fixed,
    /// Output lost: work the mobilized did not do, plants destroyed or occupied, trade cut.
    pub output_lost: Fixed,
    /// Change of the price index this week, as a share (0.01 is 1 %).
    pub inflation: Fixed,
}

/// The sums since the war began, and the last week.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WarLedger {
    pub weeks: u32,
    /// Soldiers killed, by age band, merged by band.
    pub killed: Vec<AgeBand>,
    pub invalid: u64,
    pub civilians_killed: u64,
    pub goods: Fixed,
    pub equipment_lost: Fixed,
    pub spending: Fixed,
    pub borrowed: Fixed,
    pub output_lost: Fixed,
    /// Price index since the war began (1 at the start).
    pub price_index: Fixed,
    pub last: WarWeek,
}

impl WarLedger {
    pub fn new() -> Self {
        Self {
            price_index: Fixed::ONE,
            ..Self::default()
        }
    }

    /// Adds one week.
    pub fn record(&mut self, week: &WarWeek) {
        self.weeks += 1;
        for band in &week.killed {
            match self
                .killed
                .iter_mut()
                .find(|b| b.from == band.from && b.to == band.to)
            {
                Some(b) => b.men += band.men,
                None => self.killed.push(*band),
            }
        }
        self.killed.sort_by_key(|b| (b.from, b.to));
        self.invalid += week.invalid;
        self.civilians_killed += week.civilians_killed;
        self.goods += week.goods;
        self.equipment_lost += week.equipment_lost;
        self.spending += week.spending;
        self.borrowed += week.borrowed;
        self.output_lost += week.output_lost;
        self.price_index = self.price_index * (Fixed::ONE + week.inflation);
        self.last = week.clone();
    }

    pub fn soldiers_killed(&self) -> u64 {
        self.killed.iter().map(|b| b.men).sum()
    }

    /// Spending since the war began over a year of output, for the exhaustion.
    pub fn spending_share(&self, yearly_output: Fixed) -> Fixed {
        if yearly_output.is_positive() {
            self.spending / yearly_output
        } else {
            Fixed::ZERO
        }
    }

    /// Price rise since the war began: 0 with no rise, 1 at a doubling.
    pub fn price_rise(&self) -> Fixed {
        (self.price_index - Fixed::ONE).max(Fixed::ZERO)
    }

    /// Everything the war took, in money: goods, equipment and output lost (spending is how
    /// it was paid for, not an extra cost).
    pub fn total_cost(&self) -> Fixed {
        self.goods + self.equipment_lost + self.output_lost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::war::calc::tests::rules;
    use crate::war::calc::{ExhaustionInputs, exhaustion};

    fn pm(n: i64) -> Fixed {
        Fixed::from_ratio(n, 1000)
    }

    #[test]
    fn the_ledger_adds_weeks_and_merges_ages() {
        let mut l = WarLedger::new();
        let week = WarWeek {
            killed: vec![
                AgeBand {
                    from: 20,
                    to: 25,
                    men: 100,
                },
                AgeBand {
                    from: 18,
                    to: 20,
                    men: 10,
                },
            ],
            invalid: 20,
            goods: Fixed::from_int(50),
            spending: Fixed::from_int(80),
            output_lost: Fixed::from_int(30),
            inflation: pm(10),
            ..WarWeek::default()
        };
        l.record(&week);
        l.record(&week);
        assert_eq!(l.weeks, 2);
        assert_eq!(
            l.killed,
            vec![
                AgeBand {
                    from: 18,
                    to: 20,
                    men: 20
                },
                AgeBand {
                    from: 20,
                    to: 25,
                    men: 200
                }
            ]
        );
        assert_eq!(l.soldiers_killed(), 220);
        assert_eq!(l.total_cost(), Fixed::from_int(160));
        assert_eq!(l.price_index, pm(1020));
        assert_eq!(l.spending_share(Fixed::from_int(1600)), pm(100));
    }

    /// A war won without losses still exhausts the winner: it costs money every week, prices
    /// rise and the weeks add up (Sun Tzu, ch. II).
    #[test]
    fn a_long_war_exhausts_even_without_defeat() {
        let r = rules();
        let mut l = WarLedger::new();
        let quiet = WarWeek {
            spending: Fixed::from_int(10),
            inflation: pm(3),
            ..WarWeek::default()
        };
        let mut last = 0;
        for year in 1..=6 {
            for _ in 0..52 {
                l.record(&quiet);
            }
            let e = exhaustion(
                &r,
                &ExhaustionInputs {
                    casualties: 0,
                    under_arms: 100_000,
                    men_of_age: 1_000_000,
                    shortage: Fixed::ZERO,
                    occupied: Fixed::ZERO,
                    weeks: l.weeks,
                    industry_lost: Fixed::ZERO,
                    spending_share: l.spending_share(Fixed::from_int(2000)),
                    price_rise: l.price_rise(),
                },
            );
            assert_eq!(e.casualties, 0);
            assert!(e.total > last, "year {year}: {} after {last}", e.total);
            last = e.total;
        }
    }
}
