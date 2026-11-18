//! What a government fixes over the budget's historical rule (`docs/state.pt-BR.md`, "Alavancas
//! do orçamento e da dívida"). Each lever is an `Option`: `None` is the rule, and with every lever
//! of every country at `None` the world keeps the main's hash. The values are saved in
//! `GameWorld::levers`; each market carries a copy (`Market::levers`, not saved) so that the
//! functions that read one market (`public::goods_budget`, `money::deficit_room`) see them.
//!
//! First slice: the payroll (D1), public pay (D2), the treasury's reserve and surplus (D4) and the
//! deficit cap (D5). Purchases and the military share (D3) and issuance (D9) have their fields
//! here, with no command yet.

use super::milli;
use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::state::Market;

/// What the treasury does with the cash beyond twice its reserve (D4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurplusUse {
    /// The rule: a tenth of it a week goes to purchases, and to paying back debt when the debt
    /// is above 60 % of GDP.
    #[default]
    Spend,
    /// A tenth of it a week pays back debt, whatever the debt.
    Repay,
    /// It stays in the treasury: a sovereign fund.
    Keep,
}

/// One country's levers. Values in thousandths, as the commands carry them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Levers {
    /// D1: servants and clergy per thousand people (`[servants, clergy]`).
    pub payroll: Option<[u16; 2]>,
    /// D2: public pay over the profession's private wage (the rule is 700).
    pub public_pay: Option<u16>,
    /// D3: the goods budget over the rule's (1000 is the rule), and the military share of it.
    pub purchases: Option<u16>,
    pub military: Option<u16>,
    /// D4: weeks of revenue kept in reserve (the rule is 13), and what the surplus is for.
    pub reserve_weeks: Option<u16>,
    pub surplus: Option<SurplusUse>,
    /// D5: the year's deficit for purchases, in thousandths of GDP.
    pub deficit_cap: Option<u16>,
    /// D9: shares of new issues at home, abroad in the own currency and abroad in dollars.
    pub issuance: Option<[u16; 3]>,
}

/// Share of the taxes a region's servants can collect (`Switches::fiscal_capacity`, owner's
/// decision O1 of 2026-11-17): below the servants the rule asks for (`monthly::STATE_SHARE`), the
/// state collects in proportion to what it has; above, no more. Linear, as the owner put it ("a
/// arrecadação cai na proporção que falta"); the idea is Besley and Persson's fiscal capacity
/// (2009, AER 99(4)). The rule's servants are a fixed share of the people, so a country on the
/// rule collects all but the lag of hiring behind growth.
pub fn collection(r: &super::state::Region) -> Fixed {
    let share = super::monthly::STATE_SHARE
        .iter()
        .find(|s| s.0 == super::prof::CIVIL_SERVANT)
        .map_or(0, |s| s.1);
    let needed = r.population() * share / 1000;
    if needed <= 0 {
        return Fixed::ONE;
    }
    let have = r.pops[super::prof::CIVIL_SERVANT].size;
    Fixed::from_ratio(have.min(needed), needed)
}

/// Floors and ceilings of what a command may fix. Design values for the first slice, each to be
/// replaced by a measured one with its source in the PR that switches the lever's screen on
/// (`docs/state.pt-BR.md`, the table of the commands).
pub const SERVANTS_PER_MILLE: (u16, u16) = (2, 80);
pub const CLERGY_PER_MILLE: (u16, u16) = (0, 15);
pub const PUBLIC_PAY_MILLE: (u16, u16) = (400, 1500);
pub const RESERVE_WEEKS: (u16, u16) = (4, 52);
pub const DEFICIT_CAP_MILLE: (u16, u16) = (0, 100);

/// Share of the excess of servants or clergy over a fixed payroll that leaves each month:
/// retirements, hiring freezes and lay-offs. Design value (a fortieth a month, about a quarter of
/// the excess in a year), to be calibrated on the cuts of Greece and the United Kingdom of
/// 2010–2015 in the PR that switches the payroll's screen on.
pub const PAYROLL_CUT_PER_MONTH: i64 = 40;

fn clamp((lo, hi): (u16, u16), v: u16) -> u16 {
    v.clamp(lo, hi)
}

impl Levers {
    /// Whether every lever is at its rule.
    pub fn is_rule(&self) -> bool {
        *self == Self::default()
    }

    /// The levers with every fixed value cut to its floor and ceiling.
    pub fn clamped(mut self) -> Self {
        self.payroll = self
            .payroll
            .map(|[s, c]| [clamp(SERVANTS_PER_MILLE, s), clamp(CLERGY_PER_MILLE, c)]);
        self.public_pay = self.public_pay.map(|v| clamp(PUBLIC_PAY_MILLE, v));
        self.reserve_weeks = self.reserve_weeks.map(|v| clamp(RESERVE_WEEKS, v));
        self.deficit_cap = self.deficit_cap.map(|v| clamp(DEFICIT_CAP_MILLE, v));
        self
    }

    /// Public pay over the private wage (D2).
    pub fn public_pay(&self) -> Fixed {
        self.public_pay
            .map_or(super::public::PUBLIC_PAY, |v| milli(i64::from(v)))
    }

    /// Weeks of revenue the treasury keeps (D4).
    pub fn reserve_weeks(&self) -> i64 {
        self.reserve_weeks
            .map_or(super::public::RESERVE_WEEKS, i64::from)
    }

    /// What the surplus is for (D4).
    pub fn surplus(&self) -> SurplusUse {
        self.surplus.unwrap_or_default()
    }

    /// The deficit target over GDP, when fixed (D5).
    pub fn deficit_cap(&self) -> Option<Fixed> {
        self.deficit_cap.map(|v| milli(i64::from(v)))
    }

    /// Servants and clergy per thousand people, when fixed (D1).
    pub fn payroll(&self) -> Option<[i64; 2]> {
        self.payroll.map(|p| p.map(i64::from))
    }
}

/// Every country's levers, by market index; empty while nobody has fixed anything.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeversState {
    pub countries: Vec<Levers>,
}

impl LeversState {
    /// No lever fixed anywhere: the state stays out of the hash.
    pub fn is_empty(&self) -> bool {
        self.countries.iter().all(Levers::is_rule)
    }

    /// The levers of country `ci`.
    pub fn of(&self, ci: usize) -> Levers {
        self.countries.get(ci).copied().unwrap_or_default()
    }

    /// Changes the levers of country `ci` and copies them to its market.
    pub fn set(&mut self, markets: &mut [Market], ci: usize, f: impl FnOnce(&mut Levers)) {
        if ci >= markets.len() {
            return;
        }
        if self.countries.len() <= ci {
            self.countries.resize(ci + 1, Levers::default());
        }
        let l = &mut self.countries[ci];
        f(l);
        *l = l.clamped();
        markets[ci].levers = *l;
    }

    /// Copies the saved levers to the markets (after loading a world).
    pub fn sync(&self, markets: &mut [Market]) {
        for (ci, m) in markets.iter_mut().enumerate() {
            m.levers = self.of(ci);
        }
    }
}

impl HashState for LeversState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.countries.len() as u64);
        for l in &self.countries {
            for v in [
                l.public_pay,
                l.purchases,
                l.military,
                l.reserve_weeks,
                l.deficit_cap,
            ] {
                h.write_i64(v.map_or(-1, i64::from));
            }
            for p in [
                l.payroll.map(|p| p.to_vec()),
                l.issuance.map(|p| p.to_vec()),
            ] {
                match p {
                    Some(v) => v.iter().for_each(|x| h.write_i64(i64::from(*x))),
                    None => h.write_i64(-1),
                }
            }
            h.write_i64(l.surplus.map_or(-1, |s| s as i64));
        }
    }
}

/// One lever as the screen shows it: what the rule gives now, what is fixed, and the limits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeverLine {
    pub rule: f64,
    pub fixed: Option<f64>,
    pub floor: f64,
    pub ceiling: f64,
    /// What the fixed value does to the treasury in a week against the rule, in real units
    /// (positive: more cash). `None` on the rule, or where a week's effect is not one number
    /// (the reserve).
    pub week_effect: Option<f64>,
}

/// The first slice's levers of one country, for the budget screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeversView {
    /// Servants and clergy per thousand people.
    pub servants: LeverLine,
    pub clergy: LeverLine,
    /// Public pay over the private wage.
    pub public_pay: LeverLine,
    pub reserve_weeks: LeverLine,
    pub surplus: SurplusUse,
    pub surplus_fixed: bool,
    /// The year's deficit for purchases over GDP.
    pub deficit_cap: LeverLine,
    /// With `Switches::fiscal_capacity` on, the share of last week's taxes not collected for
    /// lack of servants (0–1); `None` with the switch off.
    pub uncollected: Option<f64>,
}

/// What the budget screen reads for country `ci`.
pub fn view(m: &Market) -> LeversView {
    let l = &m.levers;
    let f = |v: Fixed| v.raw() as f64 / 1000.0;
    let line = |rule: f64, fixed: Option<u16>, (lo, hi): (u16, u16)| LeverLine {
        rule,
        fixed: fixed.map(f64::from),
        floor: f64::from(lo),
        ceiling: f64::from(hi),
        week_effect: None,
    };
    // A week of each lever against the rule, by the tick's own sums.
    let b = &m.budget;
    let rule_pay = f(super::public::PUBLIC_PAY);
    // Public pay: last week's payroll was paid at the fixed share; at the rule's it would be
    // payroll × rule / fixed. Cash rises by what the fixed pay saves.
    let pay_effect = l.public_pay.map(|v| {
        let fixed = f64::from(v) / 1000.0;
        let payroll = f(b.payroll);
        payroll * rule_pay / fixed - payroll
    });
    // Servants: those missing against the rule cost no pay but, with the capacity to collect
    // on, the taxes they would have collected. Last week's sums give the second.
    let servants_effect = l.payroll.and(m.uncollected).map(|u| -f(u));
    // Deficit cap: the purchases the cap allows against the rule's, a week.
    let deficit_effect = l.deficit_cap.map(|_| {
        let mut rule = m.clone();
        rule.levers.deficit_cap = None;
        f(super::money::deficit_room(&rule)) - f(super::money::deficit_room(m))
    });
    let share = |p: usize| {
        super::monthly::STATE_SHARE
            .iter()
            .find(|s| s.0 == p)
            .map_or(0.0, |s| s.1 as f64)
    };
    let with = |mut line: LeverLine, e: Option<f64>| {
        line.week_effect = e;
        line
    };
    let uncollected = m.uncollected.map(|u| {
        let due = f(b.revenue) + f(u);
        if due > 0.0 { f(u) / due } else { 0.0 }
    });
    LeversView {
        uncollected,
        servants: with(
            line(
                share(super::prof::CIVIL_SERVANT),
                l.payroll.map(|p| p[0]),
                SERVANTS_PER_MILLE,
            ),
            servants_effect,
        ),
        clergy: line(
            share(super::prof::CLERGY),
            l.payroll.map(|p| p[1]),
            CLERGY_PER_MILLE,
        ),
        public_pay: with(
            line(
                f(super::public::PUBLIC_PAY) * 1000.0,
                l.public_pay,
                PUBLIC_PAY_MILLE,
            ),
            pay_effect,
        ),
        reserve_weeks: line(
            super::public::RESERVE_WEEKS as f64,
            l.reserve_weeks,
            RESERVE_WEEKS,
        ),
        surplus: l.surplus(),
        surplus_fixed: l.surplus.is_some(),
        deficit_cap: with(
            line(
                f(super::money::DEFICIT_TARGET) * 1000.0,
                l.deficit_cap,
                DEFICIT_CAP_MILLE,
            ),
            deficit_effect,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_is_empty_and_limits_cut() {
        let mut s = LeversState::default();
        assert!(s.is_empty());
        let mut markets = vec![Market::empty(0), Market::empty(0)];
        s.set(&mut markets, 1, |l| {
            l.public_pay = Some(5000);
            l.reserve_weeks = Some(1);
        });
        assert!(!s.is_empty());
        assert_eq!(markets[1].levers.public_pay, Some(1500));
        assert_eq!(markets[1].levers.reserve_weeks(), 4);
        assert_eq!(markets[0].levers, Levers::default());
        s.set(&mut markets, 1, |l| *l = Levers::default());
        assert!(s.is_empty(), "back to the rule");
    }
}
