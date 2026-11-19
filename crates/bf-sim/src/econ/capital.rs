//! Capital that follows interest, and capital controls (`game::Switches::capital_follows_rate`,
//! which needs `payments_settle`; gold standard slice 4, owner's decision PO5;
//! docs/state.pt-BR.md, "Padrão-ouro, reservas e balanço de pagamentos").
//!
//! Besides the government bonds sold abroad (`money::week`), each country's savers hold private
//! claims in other countries: a long-term stock (bonds, deposits and loans of the private
//! sector) and a short-term one ("hot money"). Each week, by lender and borrower:
//!
//! - **Valuation.** A claim is in the borrower's currency: it is worth what the borrower's real
//!   exchange rate moved since last week (a devaluation costs the lenders; a country on gold
//!   whose prices rise faster than the world's is worth more in reference units).
//! - **Income.** The borrower's pool pays each claim the real yield of its bonds without the
//!   premium; it is paid under capital controls too.
//! - **Where the savings go.** The expected real return of a country ([`Outlook::ret`]) is
//!   its bond rate without the premium less its inflation and the change of its real exchange
//!   rate expected over a year (none at a parity, nor the gain a parity gives while prices rise
//!   faster than the world's: lenders do not expect that to last; the drift to the target when
//!   it floats), less the premium weighed by the lenders' aversion, a
//!   risk for a floating currency, and the devaluation a parity near its floor makes likely
//!   (`gold::at_risk`). A lender's wealth (its investment pool, what banks and capitalists
//!   hold, plus its claims; households' idle savings are a small part and stay home) is split
//!   between
//!   home and each borrower by a logit on the gap between the borrower's return and its own,
//!   weighed by the borrower's GDP: at equal returns `long_share` of the wealth sits abroad.
//!   Short-term capital follows the same returns with a larger elasticity and a faster pace.
//! - **Flows are gradual:** each week a claim closes `1/long_weeks` (`1/short_weeks`) of its
//!   gap to the target. New placements leave the lender's pool; withdrawals leave the
//!   borrower's pool and go back to the lender's. What moves is recorded in the financial
//!   account (`payments::Lines::capital_in` and the others), and settlement moves the reserves
//!   by it: an inflow raises the receiver's reserves, an outflow lowers the sender's.
//! - **Controls (PO5):** a country with capital controls (the player's choice, or by the rule
//!   when it holds its parity behind exchange control, `gold::Regime::Control`) neither lends
//!   abroad nor takes new capital from abroad, and its claims, both ways, are frozen; income
//!   is still paid, and its premium rises (`CapitalRules::control_premium`).
//!
//! The first week seeds the claims at their targets, without moving money: the positions of
//! 1925 are the rule's, not data. Every movement is a transfer between the pools and the
//! of two countries, so the world's money does not change.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::money::weekly;

/// The rule's numbers (`data/economy/capital.ron`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapitalRules {
    /// Share of a lender's wealth held abroad, long and short, when every return is equal.
    pub long_share: Fixed,
    pub short_share: Fixed,
    /// Semi-elasticity of the logit: a point of expected return multiplies a borrower's weight
    /// by about `e^beta`.
    pub long_beta: Fixed,
    pub short_beta: Fixed,
    /// Weeks a claim takes to close its gap to the target (it closes `1/weeks` each week).
    pub long_weeks: i64,
    pub short_weeks: i64,
    /// Points of return a point of premium takes off, beyond what the premium pays.
    pub risk_aversion: Fixed,
    /// Points of return a floating currency takes off.
    pub float_risk: Fixed,
    /// Points of expected depreciation a year of a parity at its floor; the risk starts at the
    /// floor times `1 + risk_band` and grows linearly to the floor.
    pub devaluation: Fixed,
    pub risk_band: Fixed,
    /// Points of premium capital controls add.
    pub control_premium: Fixed,
}

/// `GameWorld::capital`: the private claims across borders, by lender and borrower (state, in
/// the hash once the rule has run), and each country's capital controls. Empty while
/// `Switches::capital_follows_rate` is off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapitalState {
    /// Markets the claims are laid out for: `long[lender * n + borrower]`.
    pub n: u32,
    /// Long-term and short-term claims, reference units.
    pub long: Vec<Fixed>,
    pub short: Vec<Fixed>,
    /// Each borrower's real exchange rate when its claims were last valued.
    pub real_seen: Vec<Fixed>,
    /// Capital controls in force this week, by market.
    pub controls: Vec<bool>,
    /// The player's choice of controls (`GameCommand::SetCapitalControls`); `None` follows the
    /// rule (controls while holding a parity behind exchange control).
    pub chosen: Vec<Option<bool>>,
}

impl HashState for CapitalState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(u64::from(self.n));
        for v in self.long.iter().chain(&self.short).chain(&self.real_seen) {
            v.hash_state(h);
        }
        for (c, k) in self.controls.iter().zip(&self.chosen) {
            h.write_u8(u8::from(*c));
            h.write_u8(k.map_or(0, |x| u8::from(x) + 1));
        }
    }
}

impl CapitalState {
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Lays the claims out for `n` markets, keeping those already held.
    fn fit(&mut self, n: usize) {
        let old = self.n as usize;
        if n <= old {
            return;
        }
        let relay = |v: &[Fixed]| {
            let mut out = vec![Fixed::ZERO; n * n];
            for i in 0..old {
                out[i * n..i * n + old].copy_from_slice(&v[i * old..(i + 1) * old]);
            }
            out
        };
        self.long = relay(&self.long);
        self.short = relay(&self.short);
        self.real_seen.resize(n, Fixed::ZERO);
        self.controls.resize(n, false);
        self.chosen.resize(n, None);
        self.n = n as u32;
    }

    /// The week starts: the claims fit `n` markets and the controls are set, the player's
    /// choice or else the rule's (`control_regime`: holding a parity behind exchange control).
    /// Returns whether the claims were empty (the week that seeds them).
    pub fn begin(&mut self, n: usize, control_regime: &[bool]) -> bool {
        let seed = self.is_empty();
        self.fit(n);
        for (mi, c) in self.controls.iter_mut().enumerate() {
            *c =
                self.chosen[mi].unwrap_or_else(|| control_regime.get(mi).copied().unwrap_or(false));
        }
        seed
    }

    /// Market `mi`'s private claims abroad (long, short), and those of savers abroad on it.
    pub fn position(&self, mi: usize) -> ((Fixed, Fixed), (Fixed, Fixed)) {
        let n = self.n as usize;
        if mi >= n {
            return Default::default();
        }
        let row = |v: &[Fixed]| v[mi * n..(mi + 1) * n].iter().copied().sum::<Fixed>();
        let col = |v: &[Fixed]| (0..n).map(|i| v[i * n + mi]).sum::<Fixed>();
        (
            (row(&self.long), row(&self.short)),
            (col(&self.long), col(&self.short)),
        )
    }
}

/// What the week's money step knows of one market (`money::week`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outlook {
    /// Expected real return in reference units, points a year, before the rule's risks: the
    /// bond rate without the premium, less inflation and the real depreciation expected.
    pub ret: Fixed,
    pub premium: Fixed,
    pub floating: bool,
    /// Real yield paid on the claims held here, points a year.
    pub yield_pct: Fixed,
    pub gdp: Fixed,
    pub real_exchange: Fixed,
    /// The investment pool (banks and capitalists): what lenders place abroad from.
    pub pool: Fixed,
    /// Shut out of foreign lending (a default): takes no new capital.
    pub excluded: bool,
}

/// One market's lines of the week (`payments::Lines`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flows {
    pub capital_in: Fixed,
    pub capital_out: Fixed,
    pub hot_in: Fixed,
    pub hot_out: Fixed,
    pub income_received: Fixed,
    pub income_paid: Fixed,
}

/// What the week moved, by market.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Moved {
    /// Paid into the pool (negative: out of it): withdrawals and income.
    pub pool: Vec<Fixed>,
    pub lines: Vec<Flows>,
}

/// `e^x` in fixed point, within a few thousandths: `2^(x log2 e)`, the fraction by a cubic.
/// `x` is held in `[-15, 8]`.
fn soft_exp(x: Fixed) -> Fixed {
    let x = x.clamp(Fixed::from_int(-15), Fixed::from_int(8));
    let y = x.mul_ratio(1_442_695, 1_000_000);
    let n = y.to_int_floor();
    let f = y - Fixed::from_int(n);
    let frac = Fixed::ONE
        + f * (Fixed::from_ratio(693, 1000)
            + f * (Fixed::from_ratio(240, 1000) + f * Fixed::from_ratio(56, 1000)));
    if n >= 0 {
        frac * (1i64 << n)
    } else {
        frac.mul_ratio(1, 1i64 << (-n))
    }
}

/// The return lenders expect of each market (module doc), points a year.
pub fn returns(rules: &CapitalRules, outlook: &[Outlook], risk: &[Fixed]) -> Vec<Fixed> {
    outlook
        .iter()
        .enumerate()
        .map(|(k, o)| {
            o.ret
                - rules.risk_aversion * o.premium.max(Fixed::ZERO)
                - if o.floating {
                    rules.float_risk
                } else {
                    Fixed::ZERO
                }
                - risk.get(k).copied().unwrap_or(Fixed::ZERO) * rules.devaluation
        })
        .collect()
}

/// The week's private capital (module doc). `risk`, by market, is how likely a devaluation is
/// (0 to 1, `gold::at_risk`); `seed` places the claims at their targets without moving money.
pub fn step(
    rules: &CapitalRules,
    st: &mut CapitalState,
    outlook: &[Outlook],
    risk: &[Fixed],
    seed: bool,
) -> Moved {
    let n = outlook.len();
    st.fit(n);
    let stride = st.n as usize;
    let at = |i: usize, j: usize| i * stride + j;
    let mut moved = Moved {
        pool: vec![Fixed::ZERO; n],
        lines: vec![Flows::default(); n],
    };
    // Valuation: claims are in the borrower's currency.
    for (j, o) in outlook.iter().enumerate() {
        let (seen, now) = (st.real_seen[j], o.real_exchange);
        if seen > Fixed::ZERO && now > Fixed::ZERO && now != seen {
            for i in 0..n {
                for v in [&mut st.long[at(i, j)], &mut st.short[at(i, j)]] {
                    *v = v.mul_div(seen, now);
                }
            }
        }
        st.real_seen[j] = now;
    }
    // Income, paid under controls too.
    for i in 0..n {
        for (j, o) in outlook.iter().enumerate() {
            if i == j {
                continue;
            }
            let paid = weekly(
                st.long[at(i, j)] + st.short[at(i, j)],
                o.yield_pct.max(Fixed::ZERO),
            );
            if paid > Fixed::ZERO {
                moved.lines[j].income_paid += paid;
                moved.lines[i].income_received += paid;
                moved.pool[j] -= paid;
                moved.pool[i] += paid;
            }
        }
    }
    let ret = returns(rules, outlook, risk);
    let world: Fixed = outlook.iter().map(|o| o.gdp.max(Fixed::ZERO)).sum();
    let wealth: Vec<Fixed> = (0..n)
        .map(|i| {
            outlook[i].pool.max(Fixed::ZERO)
                + st.long[at(i, 0)..at(i, 0) + n]
                    .iter()
                    .copied()
                    .sum::<Fixed>()
                + st.short[at(i, 0)..at(i, 0) + n]
                    .iter()
                    .copied()
                    .sum::<Fixed>()
        })
        .collect();
    // The flows each claim wants this week (positive: the lender places more).
    let mut flow = [vec![Fixed::ZERO; n * n], vec![Fixed::ZERO; n * n]];
    let mut weights = vec![Fixed::ZERO; n];
    for (kind, (share, beta, weeks)) in [
        (rules.long_share, rules.long_beta, rules.long_weeks),
        (rules.short_share, rules.short_beta, rules.short_weeks),
    ]
    .into_iter()
    .enumerate()
    {
        if share <= Fixed::ZERO || world <= Fixed::ZERO {
            continue;
        }
        // GDP as parts per million of the world's, so that small markets keep a weight.
        let ppm = Fixed::from_int(1_000_000);
        let home = ppm.mul_div(Fixed::ONE - share, share);
        for i in 0..n {
            let mut abroad = Fixed::ZERO;
            for (j, w) in weights.iter_mut().enumerate() {
                *w = if i == j {
                    Fixed::ZERO
                } else {
                    outlook[j]
                        .gdp
                        .max(Fixed::ZERO)
                        .mul_div(ppm, world)
                        .max(Fixed::EPSILON)
                        * soft_exp(beta * (ret[j] - ret[i]))
                };
                abroad += *w;
            }
            for (j, &w) in weights.iter().enumerate() {
                if i == j {
                    continue;
                }
                let held = if kind == 0 {
                    &mut st.long[at(i, j)]
                } else {
                    &mut st.short[at(i, j)]
                };
                let mut target = wealth[i].mul_div(w, home + abroad);
                if seed {
                    *held = target;
                    continue;
                }
                if st.controls[i] || st.controls[j] {
                    target = *held;
                }
                if outlook[j].excluded {
                    target = target.min(*held);
                }
                flow[kind][i * n + j] = (target - *held) / weeks.max(1);
            }
        }
    }
    if seed {
        // The targets were taken on the pool alone; the wealth is the pool plus the claims, so
        // each lender's claims scale by `1 / (1 - share abroad)`: the seed is already at target.
        for (i, o) in outlook.iter().enumerate() {
            let row = at(i, 0)..at(i, 0) + n;
            let held: Fixed = st.long[row.clone()].iter().copied().sum::<Fixed>()
                + st.short[row.clone()].iter().copied().sum::<Fixed>();
            let pool = o.pool.max(Fixed::ZERO);
            if held <= Fixed::ZERO || held >= pool {
                continue;
            }
            for v in st.long[row.clone()]
                .iter_mut()
                .chain(st.short[row].iter_mut())
            {
                *v = v.mul_div(pool, pool - held);
            }
        }
        return moved;
    }
    // New placements are capped by the lender's pool, withdrawals by the borrower's
    // pool: each scaled down together.
    let mut placed = vec![Fixed::ZERO; n];
    let mut withdrawn = vec![Fixed::ZERO; n];
    for f in &flow {
        for i in 0..n {
            for j in 0..n {
                let v = f[i * n + j];
                if v > Fixed::ZERO {
                    placed[i] += v;
                } else {
                    withdrawn[j] -= v;
                }
            }
        }
    }
    let place_scale: Vec<Fixed> = (0..n)
        .map(|i| super::fill(outlook[i].pool.max(Fixed::ZERO), placed[i]))
        .collect();
    let withdraw_scale: Vec<Fixed> = (0..n)
        .map(|j| super::fill(outlook[j].pool.max(Fixed::ZERO), withdrawn[j]))
        .collect();
    for (kind, f) in flow.iter().enumerate() {
        for i in 0..n {
            for j in 0..n {
                let v = f[i * n + j];
                if v == Fixed::ZERO {
                    continue;
                }
                let held = if kind == 0 {
                    &mut st.long[at(i, j)]
                } else {
                    &mut st.short[at(i, j)]
                };
                let (from, to, amount) = if v > Fixed::ZERO {
                    let a = v * place_scale[i];
                    moved.pool[i] -= a;
                    moved.pool[j] += a;
                    *held += a;
                    (i, j, a)
                } else {
                    let a = (-v * withdraw_scale[j]).min(*held);
                    moved.pool[j] -= a;
                    moved.pool[i] += a;
                    *held -= a;
                    (j, i, a)
                };
                if amount == Fixed::ZERO {
                    continue;
                }
                if kind == 0 {
                    moved.lines[from].capital_out += amount;
                    moved.lines[to].capital_in += amount;
                } else {
                    moved.lines[from].hot_out += amount;
                    moved.lines[to].hot_in += amount;
                }
            }
        }
    }
    moved
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn rules() -> CapitalRules {
        CapitalRules {
            long_share: Fixed::from_ratio(1, 10),
            short_share: Fixed::from_ratio(2, 100),
            long_beta: Fixed::from_ratio(3, 10),
            short_beta: Fixed::ONE,
            long_weeks: 104,
            short_weeks: 8,
            risk_aversion: Fixed::from_ratio(1, 2),
            float_risk: Fixed::ONE,
            devaluation: Fixed::from_int(10),
            risk_band: Fixed::from_ratio(1, 4),
            control_premium: Fixed::ONE,
        }
    }

    /// Three alike markets on gold, each with a return of 3 points.
    fn alike() -> Vec<Outlook> {
        vec![
            Outlook {
                ret: Fixed::from_int(3),
                yield_pct: Fixed::from_int(3),
                gdp: Fixed::from_int(100_000),
                real_exchange: Fixed::from_int(100),
                pool: Fixed::from_int(50_000),
                ..Outlook::default()
            };
            3
        ]
    }

    fn seeded(o: &[Outlook]) -> CapitalState {
        let mut st = CapitalState::default();
        assert!(st.begin(o.len(), &[]));
        let m = step(&rules(), &mut st, o, &[], true);
        assert!(
            m.lines.iter().all(|l| *l == Flows::default()),
            "seeding moves nothing"
        );
        st
    }

    /// Every line adds up to zero over the world, and the pools of the world keep their money.
    fn conserved(m: &Moved) {
        let sum = |f: &dyn Fn(&Flows) -> Fixed| m.lines.iter().map(f).sum::<Fixed>();
        assert_eq!(sum(&|l| l.capital_in), sum(&|l| l.capital_out));
        assert_eq!(sum(&|l| l.hot_in), sum(&|l| l.hot_out));
        assert_eq!(sum(&|l| l.income_received), sum(&|l| l.income_paid));
        assert_eq!(m.pool.iter().copied().sum::<Fixed>(), Fixed::ZERO);
    }

    fn net(l: &Flows) -> Fixed {
        l.capital_in - l.capital_out + l.hot_in - l.hot_out
    }

    #[test]
    fn soft_exp_is_close_to_exp() {
        for (x, e) in [
            (0, 1.0),
            (1, std::f64::consts::E),
            (-1, 0.368),
            (3, 20.09),
            (-5, 0.0067),
        ] {
            let v = soft_exp(Fixed::from_int(x)).to_f64_lossy();
            assert!((v - e).abs() <= e * 0.01 + 0.002, "e^{x}: {v} against {e}");
        }
        assert!(soft_exp(Fixed::from_int(2)) > soft_exp(Fixed::ONE));
    }

    #[test]
    fn at_equal_returns_nothing_moves_but_income() {
        let o = alike();
        let mut st = seeded(&o);
        let m = step(&rules(), &mut st, &o, &[], false);
        conserved(&m);
        for l in &m.lines {
            assert_eq!(net(l), Fixed::ZERO);
            assert!(l.income_received > Fixed::ZERO);
            assert_eq!(l.income_received, l.income_paid);
        }
    }

    #[test]
    fn capital_goes_where_the_return_is_higher_and_gradually() {
        let mut o = alike();
        let mut st = seeded(&o);
        let (_, (held_long, held_short)) = st.position(0);
        // Market 0 raises its return by two points.
        o[0].ret += Fixed::from_int(2);
        let m = step(&rules(), &mut st, &o, &[], false);
        conserved(&m);
        assert!(net(&m.lines[0]) > Fixed::ZERO, "{:?}", m.lines[0]);
        assert!(net(&m.lines[1]) < Fixed::ZERO && net(&m.lines[2]) < Fixed::ZERO);
        // Its own lenders bring part of theirs home, and send none out.
        assert_eq!(m.lines[0].capital_out, Fixed::ZERO);
        // Gradual: a week moves a share of the gap, not the gap.
        let (_, (long, short)) = st.position(0);
        assert!(long - held_long < held_long / 10, "{held_long} -> {long}");
        assert!(short - held_short < held_short, "{held_short} -> {short}");
        // Lowering it below the others' sends it out.
        o[0].ret -= Fixed::from_int(4);
        let m = step(&rules(), &mut st, &o, &[], false);
        conserved(&m);
        assert!(net(&m.lines[0]) < Fixed::ZERO);
    }

    #[test]
    fn hot_money_leaves_a_parity_at_risk_first() {
        let o = alike();
        let mut st = seeded(&o);
        let (_, (long, short)) = st.position(0);
        let risk = [Fixed::from_ratio(1, 2), Fixed::ZERO, Fixed::ZERO];
        let m = step(&rules(), &mut st, &o, &risk, false);
        conserved(&m);
        let l = m.lines[0];
        assert!(l.hot_out > l.hot_in && l.capital_out > l.capital_in);
        let hot = (l.hot_out - l.hot_in) / short;
        let slow = (l.capital_out - l.capital_in) / long;
        assert!(hot > slow * 5, "hot {hot}, long {slow}");
    }

    #[test]
    fn controls_close_the_account_both_ways_and_income_is_still_paid() {
        let mut o = alike();
        let mut st = seeded(&o);
        let before = st.position(0);
        st.chosen[0] = Some(true);
        assert!(!st.begin(3, &[]));
        assert!(st.controls[0]);
        o[0].ret += Fixed::from_int(3);
        let m = step(&rules(), &mut st, &o, &[], false);
        conserved(&m);
        let l = m.lines[0];
        assert_eq!(
            (l.capital_in, l.capital_out, l.hot_in, l.hot_out),
            (Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO)
        );
        assert!(l.income_paid > Fixed::ZERO && l.income_received > Fixed::ZERO);
        assert_eq!(st.position(0), before);
        // The rule's controls: a parity behind exchange control, unless the player chose.
        st.chosen[0] = None;
        st.begin(3, &[false, true, false]);
        assert_eq!(st.controls, [false, true, false]);
    }

    #[test]
    fn a_devaluation_costs_the_lenders() {
        let mut o = alike();
        let mut st = seeded(&o);
        let (_, (long, _)) = st.position(0);
        o[0].real_exchange = Fixed::from_int(125);
        step(&rules(), &mut st, &o, &[], false);
        let (_, (after, _)) = st.position(0);
        assert!(after < long.mul_ratio(81, 100), "{long} -> {after}");
    }
}
