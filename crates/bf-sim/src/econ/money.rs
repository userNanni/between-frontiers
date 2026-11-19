//! Money, central bank and public debt by country (phase 3, slice 3c).
//!
//! The ledgers of the simulation stay in real units: dollars at 1925 base prices, which also
//! serve as the world reference currency. Each country (market; colonies use the metropole's)
//! adds on top of them a price level, an inflation rate and an exchange rate against the
//! reference. Debt in the country's own currency loses real value with its inflation; debt in
//! foreign currency loses it with world inflation and grows when the currency falls in real
//! terms.
//!
//! Every week, after taxes and spending: inflation and the price level move, and the central
//! bank sets its rate (Taylor rule, or the player's rate) and the money it creates or
//! withdraws. The treasury sells bonds to bring its cash back to the reserve: to savers at home
//! first, then to a subordinate central bank within its rule (an autonomous one may not lend to
//! the treasury), then abroad (partly in foreign currency), and, with a subordinate central
//! bank, to it for the rest. Money the rule creates that the treasury does not take is lent to
//! banks. Interest is paid every week; a country whose foreign-currency service passes a limit,
//! or whose interest eats half its revenue, defaults.
//!
//! Phase 4 hook: the international regime (gold standard, Bretton Woods, floating, euro, the
//! reserve currency) replaces [`exchange_target`]. Until then every currency floats on
//! purchasing-power parity with inertia and a risk discount.
//!
//! See `docs/state.pt-BR.md` ("Moeda, banco central e dívida").

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

use super::catalogue::Catalogue;
use super::milli;
use super::state::{Economy, Market, Pop, Region};

/// Rates and shares in percent: `pct(25)` is 2.5 %.
pub const fn pct(tenths: i64) -> Fixed {
    Fixed::from_raw(tenths * 100)
}

/// `amount × rate_pct / 100 / 52`: a week of an annual rate given in percent.
pub fn weekly(amount: Fixed, rate_pct: Fixed) -> Fixed {
    amount.mul_ratio(rate_pct.raw(), 5_200_000)
}

/// `a / b` as a percentage; zero when `b` is not positive.
pub fn percent_of(a: Fixed, b: Fixed) -> Fixed {
    if b <= Fixed::ZERO {
        Fixed::ZERO
    } else {
        a.mul_div(Fixed::from_int(100), b)
    }
}

/// Price level and exchange rate in 1925.
pub const INDEX_BASE: Fixed = Fixed::from_int(100);
/// Inflation target of an autonomous central bank, and the neutral real rate of the world.
pub const TARGET: Fixed = pct(20);
pub const NEUTRAL_REAL: Fixed = pct(20);
/// Taylor rule: the rate moves this much per point of inflation above target and per point of
/// utilization above its trend.
const TAYLOR_INFLATION: Fixed = milli(500);
const TAYLOR_GAP: Fixed = milli(500);
/// Highest rate the rule or the player sets.
pub const MAX_RATE: Fixed = pct(1000);
/// Inflation: points per point of utilization above its trend and per point of the market
/// price index above its trend.
const PHILLIPS: Fixed = milli(300);
const HEAT: Fixed = milli(200);
/// Points of inflation per point of GDP a year of money created beyond the rule: what a
/// subordinate bank lends the treasury because nobody else would, or what a rate fixed below
/// the rule adds.
const MONEY_PASS: Fixed = milli(1500);
/// New money a growing economy holds, in points of GDP a year, created by the bank at a
/// neutral stance. There is no bank multiplier: this is broad money, the credit banks would
/// create folded into the central bank.
pub const NEUTRAL_MONEY: Fixed = pct(15);
/// The cycle terms read at most this many points of utilization and of the price index away
/// from their trends.
const MAX_GAP: Fixed = Fixed::from_int(5);
const MAX_HEAT: Fixed = Fixed::from_int(10);
/// Bounds on the inflation rate, in percent a year.
const INFLATION_FLOOR: Fixed = pct(-100);
const INFLATION_CEILING: Fixed = pct(20_000);
/// Inflation closes 1/13 of its gap to the target each week; expectations, 1/52; the trends of
/// utilization and prices, 1/104 (two years).
const INFLATION_WEEKS: i64 = 13;
const EXPECTATION_WEEKS: i64 = 52;
const TREND_WEEKS: i64 = 104;
/// Past this price level the currency is reformed: three zeros fall.
const REDENOMINATION: Fixed = Fixed::from_int(100_000_000);

/// The bank's stance is read at most this many points from the neutral rate.
const MAX_STANCE: Fixed = Fixed::from_int(10);
/// The real exchange rate stays within a quarter and four times parity.
const REAL_EXCHANGE_FLOOR: Fixed = Fixed::from_int(25);
const REAL_EXCHANGE_CEILING: Fixed = Fixed::from_int(400);
/// Highest risk premium, in points.
const MAX_PREMIUM: Fixed = pct(250);

/// Weeks of income a household keeps as cash; above it, savings are lent (bank deposits,
/// bonds) and are what the state borrows at home and abroad.
pub const BUFFER_WEEKS: i64 = 26;
/// Shares of the idle savings at home and of the investment pool that buy new bonds each week,
/// and of the idle savings of other countries that lend abroad.
const HOME_ABSORB: Fixed = milli(5);
const POOL_ABSORB: Fixed = milli(5);
const FOREIGN_ABSORB: Fixed = milli(5);
/// Share of the idle savings banks lend each week to the investment pool, after the state and
/// foreign borrowers took theirs: savings that sit idle are a leak of demand.
const BANK_LENDING: Fixed = milli(20);
/// Weeks for the cost of the debt stock to follow the market rate (five-year bonds).
const MATURITY_WEEKS: i64 = 260;
/// A treasury above twice its reserve pays back a tenth of the excess each week, while the
/// debt is above this share of GDP (below it, the excess is spent).
const REPAY_SHARE: Fixed = milli(100);
const REPAY_ABOVE: Fixed = milli(600);

/// Risk premium, in points: per unit of debt/GDP above 0.6 (weighted by the share held
/// abroad), per point of inflation above 3 %, per unit of foreign-currency debt/GDP, and per
/// unit of default history; savings at home take off a point per unit of GDP, up to three.
const DEBT_RISK: Fixed = pct(40);
const DEBT_SAFE: Fixed = milli(600);
const INFLATION_RISK: Fixed = milli(200);
const FX_RISK: Fixed = pct(60);
const DEFAULT_RISK: Fixed = pct(40);
const SAVINGS_DISCOUNT: Fixed = pct(10);
const SAVINGS_DISCOUNT_MAX: Fixed = pct(30);
/// Lowest rate on bonds.
const BOND_FLOOR: Fixed = pct(1);
/// Default: when the foreign-currency service (interest plus a tenth of the principal) passes
/// 30 % of revenue, or the treasury owes a quarter of a year's revenue it could not borrow
/// while it owes abroad, half of the foreign debt is written off and the country is shut out
/// of foreign lending for three years. Each default adds one to the history, which
/// loses a tenth a year.
const AMORTIZATION: Fixed = pct(100);
pub const DEFAULT_LIMIT: Fixed = pct(300);
const HAIRCUT: Fixed = milli(500);
/// When interest takes this share of revenue the state restructures all its debt outside the
/// central bank, writing off this share.
const INTEREST_LIMIT: Fixed = milli(500);
const RESTRUCTURING: Fixed = milli(400);
const EXCLUSION_YEARS: i32 = 3;
const HISTORY_DECAY: Fixed = milli(900);
/// The exchange rate closes 1/26 of its gap each week; a point of risk premium weakens the
/// currency by 3 % against parity.
const EXCHANGE_WEEKS: i64 = 26;
const EXCHANGE_RISK: Fixed = milli(30);
/// Under the historical rule central banks follow the treasury until 1990 (or until a year of
/// inflation above this), and are autonomous after.
const AUTONOMY_YEAR: i32 = 1990;
const STABILIZATION: Fixed = pct(2000);
/// Public works the state may borrow for in a year, as a share of GDP, while the debt is below
/// GDP and the rate below 15 %.
const PUBLIC_BORROWING: Fixed = milli(20);
const BORROWING_RATE_LIMIT: Fixed = pct(150);
/// Deficit the state runs on purchases, as a share of GDP, while debt is below 150 % of GDP and
/// the rate below 15 % (the room shrinks linearly to both limits).
pub const DEFICIT_TARGET: Fixed = milli(50);
const DEBT_LIMIT: Fixed = milli(1500);
/// Savers buy bonds at home at the square of the country's income against twice the world's
/// (capped at 1), and at least this: thin markets and few banks in poor countries.
const SHALLOW_MARKET: Fixed = milli(50);

/// Who the central bank answers to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CentralBank {
    /// Subordinate until 1990 (or a stabilization after hyperinflation), autonomous after.
    #[default]
    Historical,
    /// Follows its rule and never lends to the treasury: its money goes to banks.
    Autonomous,
    /// Buys whatever bonds nobody else does.
    Subordinate,
}

/// A country's currency, central bank and public debt. Amounts are real (reference units);
/// rates and inflation in percent a year.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Money {
    /// Consumer prices, 1925 = 100, and their level on the last January 1st.
    pub price_level: Fixed,
    pub price_level_jan: Fixed,
    /// Inflation now, over the last calendar year, and what people expect.
    pub inflation: Fixed,
    pub inflation_last_year: Fixed,
    pub expected: Fixed,
    /// Purchasing-power parity against the reference (local over world prices, 1925 = 100) and
    /// the exchange rate (local units per reference unit, 1925 = 100).
    pub parity: Fixed,
    pub exchange_rate: Fixed,
    /// Central bank: its rate, the player's rate when set, and who it answers to.
    pub policy_rate: Fixed,
    pub rate_command: Option<Fixed>,
    pub bank: CentralBank,
    /// Year the historical rule made the bank autonomous (0: not yet).
    pub autonomous_since: i32,
    /// Plants' utilization and the market price index (percent) and their two-year trends.
    pub utilization_trend: Fixed,
    pub index_trend: Fixed,
    /// Market rates on new bonds in local and in foreign currency, and the average cost of each
    /// stock.
    pub bond_rate: Fixed,
    pub fx_rate: Fixed,
    pub cost_local: Fixed,
    pub cost_fx: Fixed,
    /// Debt in the country's currency held at home (savers, banks), by the central bank and
    /// abroad; debt in foreign currency, in reference units.
    pub debt_home: Fixed,
    pub debt_bank: Fixed,
    pub debt_abroad: Fixed,
    pub debt_fx: Fixed,
    /// What the country's savers lent to other states (reference units).
    pub foreign_assets: Fixed,
    /// Defaults so far, the decaying history the premium reads, and the year foreign lenders
    /// come back.
    pub defaults: u16,
    pub default_history: Fixed,
    pub excluded_until: i32,
    /// Currency reforms (three zeros cut).
    pub redenominations: u16,
    /// Money created beyond the rule (forced lending to the treasury, a rate fixed below the
    /// rule), in points of GDP a year, smoothed over a year.
    pub monetized: Fixed,
    /// The central bank's loans to banks (the investment pool): money it created that the
    /// treasury did not need.
    pub credit: Fixed,
}

impl Default for Money {
    fn default() -> Self {
        Self {
            price_level: INDEX_BASE,
            price_level_jan: INDEX_BASE,
            inflation: Fixed::ZERO,
            inflation_last_year: Fixed::ZERO,
            expected: Fixed::ZERO,
            parity: INDEX_BASE,
            exchange_rate: INDEX_BASE,
            policy_rate: NEUTRAL_REAL,
            rate_command: None,
            bank: CentralBank::Historical,
            autonomous_since: 0,
            utilization_trend: Fixed::ZERO,
            index_trend: Fixed::ZERO,
            bond_rate: pct(45),
            fx_rate: pct(50),
            cost_local: pct(45),
            cost_fx: pct(50),
            debt_home: Fixed::ZERO,
            debt_bank: Fixed::ZERO,
            debt_abroad: Fixed::ZERO,
            debt_fx: Fixed::ZERO,
            foreign_assets: Fixed::ZERO,
            defaults: 0,
            default_history: Fixed::ZERO,
            excluded_until: 0,
            redenominations: 0,
            monetized: Fixed::ZERO,
            credit: Fixed::ZERO,
        }
    }
}

/// Debt the countries of 1925 start with, as shares of GDP: the war debts are not in the
/// scenario yet, so every country starts with the same, held at home and by its central bank.
const START_DEBT_HOME: Fixed = milli(250);
const START_DEBT_BANK: Fixed = milli(50);

impl Money {
    /// The money of a country whose GDP in 1925 is `gdp`.
    pub fn start(gdp: Fixed) -> Self {
        Self {
            debt_home: gdp * START_DEBT_HOME,
            debt_bank: gdp * START_DEBT_BANK,
            ..Self::default()
        }
    }

    /// Real exchange rate: above 100 the currency is cheaper than parity.
    pub fn real_exchange(&self) -> Fixed {
        if self.parity <= Fixed::ZERO {
            return INDEX_BASE;
        }
        self.exchange_rate
            .mul_div(INDEX_BASE, self.parity)
            .clamp(REAL_EXCHANGE_FLOOR, REAL_EXCHANGE_CEILING)
    }

    /// Foreign-currency debt in local real units.
    pub fn debt_fx_local(&self) -> Fixed {
        self.debt_fx.mul_div(self.real_exchange(), INDEX_BASE)
    }

    /// All public debt, in local real units.
    pub fn total_debt(&self) -> Fixed {
        self.debt_home + self.debt_bank + self.debt_abroad + self.debt_fx_local()
    }

    /// Whether the central bank lends the treasury whatever nobody else does, in `year`.
    pub fn subordinate(&self, year: i32) -> bool {
        match self.bank {
            CentralBank::Subordinate => true,
            CentralBank::Autonomous => false,
            CentralBank::Historical => self.autonomous_since == 0 && year < AUTONOMY_YEAR,
        }
    }
}

impl HashState for CentralBank {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u8(*self as u8);
    }
}

impl HashState for Money {
    fn hash_state(&self, h: &mut StateHasher) {
        for v in [
            self.price_level,
            self.price_level_jan,
            self.inflation,
            self.inflation_last_year,
            self.expected,
            self.parity,
            self.exchange_rate,
            self.policy_rate,
            self.utilization_trend,
            self.index_trend,
            self.bond_rate,
            self.fx_rate,
            self.cost_local,
            self.cost_fx,
            self.debt_home,
            self.debt_bank,
            self.debt_abroad,
            self.debt_fx,
            self.foreign_assets,
            self.default_history,
            self.monetized,
            self.credit,
        ] {
            v.hash_state(h);
        }
        self.rate_command.hash_state(h);
        self.bank.hash_state(h);
        h.write_i64(i64::from(self.autonomous_since));
        h.write_u16(self.defaults);
        h.write_i64(i64::from(self.excluded_until));
        h.write_u16(self.redenominations);
    }
}

/// How busy a market was this week: plants' utilization weighted by capacity at base prices,
/// and the price index of what traded (market over base prices), both in percent.
#[derive(Clone, Copy, Debug)]
pub struct Activity {
    pub utilization: Fixed,
    pub index: Fixed,
}

/// Activity of every market, from the plants and the week's prices.
pub fn activity(cat: &Catalogue, regions: &[Region], markets: &[Market]) -> Vec<Activity> {
    let nm = markets.len();
    let mut capacity = vec![Fixed::ZERO; nm];
    let mut used = vec![Fixed::ZERO; nm];
    for r in regions {
        let mi = r.market as usize;
        for p in &r.plants {
            let b = &cat.buildings[p.kind as usize];
            let v = b.methods[p.method as usize].output_value * r.tfp_for(b) * p.levels;
            capacity[mi] += v;
            used[mi] += v * p.utilization;
        }
    }
    markets
        .iter()
        .enumerate()
        .map(|(mi, m)| {
            let mut base = Fixed::ZERO;
            let mut now = Fixed::ZERO;
            for gi in 0..cat.goods.len() {
                let q = m.supply[gi].min(m.demand[gi]);
                base += q * cat.goods[gi].base_price;
                now += q * m.price[gi];
            }
            Activity {
                utilization: if capacity[mi] > Fixed::ZERO {
                    percent_of(used[mi], capacity[mi])
                } else {
                    Fixed::from_int(100)
                },
                index: if base > Fixed::ZERO {
                    percent_of(now, base)
                } else {
                    Fixed::from_int(100)
                },
            }
        })
        .collect()
}

/// Savings a pop holds above its cash buffer.
pub fn idle_savings(p: &Pop) -> Fixed {
    (p.savings - p.income.max(Fixed::ZERO) * BUFFER_WEEKS).max(Fixed::ZERO)
}

/// The world this week: GDP-weighted inflation (each country capped at 50 %) and the world
/// rate on safe debt in the reference currency.
#[derive(Clone, Copy, Debug)]
pub struct World {
    pub inflation: Fixed,
    pub rate: Fixed,
    /// GDP per person of the world.
    pub income: Fixed,
}

fn world(markets: &[Market], people: &[i64]) -> World {
    let gdp: Fixed = markets.iter().map(|m| m.gdp_last_year).sum();
    let inflation = if gdp > Fixed::ZERO {
        markets
            .iter()
            .map(|m| {
                m.money
                    .inflation
                    .min(pct(500))
                    .mul_div(m.gdp_last_year, gdp)
            })
            .sum()
    } else {
        Fixed::ZERO
    };
    let total_people: i64 = people.iter().sum();
    World {
        inflation,
        rate: (NEUTRAL_REAL + inflation).max(BOND_FLOOR),
        income: if total_people > 0 {
            gdp / total_people
        } else {
            Fixed::ZERO
        },
    }
}

/// Share of the foreign purchases of a country's bonds made in foreign currency: lenders want
/// dollars from poor countries, from those with high inflation and from those that defaulted
/// ("original sin").
pub fn fx_share(m: &Market, income: Fixed, world: &World) -> Fixed {
    let rich = if world.income > Fixed::ZERO {
        (income / (world.income * 2)).clamp(Fixed::ZERO, Fixed::ONE)
    } else {
        Fixed::ZERO
    };
    let inflation =
        (m.money.inflation_last_year.max(m.money.inflation) / 50).clamp(Fixed::ZERO, Fixed::ONE);
    (milli(800) - rich * milli(700) + inflation * milli(500) + m.money.default_history * milli(200))
        .clamp(Fixed::ZERO, milli(950))
}

/// Risk premium on the country's debt, in points.
/// `treaty_debt`: what the country owes other governments (`treaty::Ledger`), in local units; it
/// weighs as debt in foreign currency held abroad.
fn premium(m: &Market, gdp: Fixed, idle_home: Fixed, treaty_debt: Fixed) -> Fixed {
    let money = &m.money;
    let debt = money.total_debt() + treaty_debt;
    let ratio = if gdp > Fixed::ZERO {
        debt / gdp
    } else {
        Fixed::ZERO
    };
    let abroad = if debt > Fixed::ZERO {
        (money.debt_abroad + money.debt_fx_local() + treaty_debt) / debt
    } else {
        Fixed::ZERO
    };
    let fx_ratio = if gdp > Fixed::ZERO {
        (money.debt_fx_local() + treaty_debt) / gdp
    } else {
        Fixed::ZERO
    };
    let savings = if gdp > Fixed::ZERO {
        (SAVINGS_DISCOUNT * (idle_home / gdp)).min(SAVINGS_DISCOUNT_MAX)
    } else {
        Fixed::ZERO
    };
    let risk =
        DEBT_RISK * (ratio - DEBT_SAFE).max(Fixed::ZERO) * (milli(300) + abroad * milli(700))
            + INFLATION_RISK * (money.inflation - pct(30)).max(Fixed::ZERO)
            + FX_RISK * fx_ratio
            + DEFAULT_RISK * money.default_history;
    risk.min(MAX_PREMIUM) - savings
}

/// Exchange rate the currency drifts to: parity, weakened by the risk premium. Phase 4 puts
/// the international regime here (gold, Bretton Woods pegs, the euro).
pub fn exchange_target(money: &Money, premium: Fixed) -> Fixed {
    money.parity + money.parity * EXCHANGE_RISK * premium.max(Fixed::ZERO)
}

/// Public works the treasury may borrow for this month, beyond its cash.
pub fn borrowing_room(m: &Market) -> Fixed {
    let gdp = m.gdp_last_year;
    if gdp <= Fixed::ZERO || m.money.bond_rate >= BORROWING_RATE_LIMIT {
        return Fixed::ZERO;
    }
    let debt_room = (Fixed::ONE - m.money.total_debt() / gdp).max(Fixed::ZERO);
    let rate_room = Fixed::ONE - m.money.bond_rate / BORROWING_RATE_LIMIT;
    gdp.mul_div(PUBLIC_BORROWING, Fixed::from_int(12)) * debt_room * rate_room
}

/// Purchases the state may borrow for this week (the deficit rule).
pub fn deficit_room(m: &Market) -> Fixed {
    let gdp = m.gdp_last_year;
    if gdp <= Fixed::ZERO || m.money.bond_rate >= BORROWING_RATE_LIMIT {
        return Fixed::ZERO;
    }
    let debt_room = (Fixed::ONE - m.money.total_debt() / (gdp * DEBT_LIMIT)).max(Fixed::ZERO);
    let rate_room = Fixed::ONE - m.money.bond_rate / BORROWING_RATE_LIMIT;
    // The deficit target: the rule's, or the cap the government fixed (D5). The brakes of debt
    // and rates hold either way: the market lends no more because the government wants it.
    let target = m.levers.deficit_cap().unwrap_or(DEFICIT_TARGET);
    (gdp / 52) * target * debt_room * rate_room
}

/// Share of revenue the debt's interest takes (weekly interest over weekly revenue).
pub fn interest_share(m: &Market) -> Fixed {
    if m.budget.revenue_avg <= Fixed::ZERO {
        return Fixed::ZERO;
    }
    m.budget.interest / m.budget.revenue_avg
}

/// Real rate on the country's bonds, in percent.
pub fn real_rate(m: &Market) -> Fixed {
    m.money.bond_rate - m.money.expected.max(Fixed::ZERO)
}

/// What one market asks from foreign lenders this week.
/// One week's new bonds with the issue split fixed (D9), for the screen: sold at home (savers
/// and a subordinate bank), sold abroad in the country's own currency and in dollars, and left
/// unsold at home and abroad; and the share of local bonds lenders abroad would take.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IssueWeek {
    pub home: Fixed,
    pub abroad_local: Fixed,
    pub abroad_fx: Fixed,
    pub unsold_home: Fixed,
    pub unsold_abroad: Fixed,
    pub local_limit: Fixed,
}

struct Plan {
    /// Still to be financed after home savers and the central bank's rule: asked abroad, then
    /// of a subordinate bank.
    need: Fixed,
    /// Bought at home this week: from idle household savings and from the pool.
    from_households: Fixed,
    /// Idle household savings here, and what foreign lenders may take from them.
    idle: Fixed,
    lendable: Fixed,
    fx_share: Fixed,
    /// Interest paid to foreign holders this week.
    abroad_interest: Fixed,
    /// With the issue split fixed (D9): the share of what is asked abroad meant for bonds in the
    /// country's own currency (the rest in dollars). `None` on the rule: the market splits.
    local_abroad: Option<Fixed>,
}

/// What crossed borders in a week's money step, by market (`econ::payments`): moved by
/// `week`, not decided by it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AbroadFlows {
    /// Bonds sold abroad, local and in dollars: what lenders abroad paid in.
    pub sold: Vec<Fixed>,
    /// What each market's savers lent abroad.
    pub lent: Vec<Fixed>,
    /// Interest paid on the bonds held abroad, and principal repaid on them.
    pub interest_paid: Vec<Fixed>,
    pub principal_repaid: Vec<Fixed>,
    /// What the lenders received, split into interest and principal in the payers' proportion.
    pub interest_received: Vec<Fixed>,
    pub principal_received: Vec<Fixed>,
    /// Owed abroad in a week when nobody holds a claim abroad (interest, principal): it stays
    /// with the debtors, back in their treasuries, and is out of the lines above. Otherwise
    /// zero: the lenders receive what the debtors pay to the last unit.
    pub unreceived: (Fixed, Fixed),
    /// The service of the debts between governments, by line: (debtor, creditor, interest,
    /// principal), from treasury to treasury.
    pub ledger: Vec<(u16, u16, Fixed, Fixed)>,
}

/// The weekly money step: prices, central banks, bonds, interest and defaults, for every
/// market. Runs after taxes and spending are in the treasuries. Returns what crossed borders.
pub fn week(econ: &mut Economy, activity: &[Activity], year: i32) -> AbroadFlows {
    let nm = econ.markets.len();
    let mut flows = AbroadFlows {
        sold: vec![Fixed::ZERO; nm],
        lent: vec![Fixed::ZERO; nm],
        interest_paid: vec![Fixed::ZERO; nm],
        principal_repaid: vec![Fixed::ZERO; nm],
        interest_received: vec![Fixed::ZERO; nm],
        principal_received: vec![Fixed::ZERO; nm],
        ..AbroadFlows::default()
    };
    let mut idle = vec![Fixed::ZERO; nm];
    let mut people = vec![0i64; nm];
    for r in &econ.regions {
        let mi = r.market as usize;
        for p in &r.pops {
            idle[mi] += idle_savings(p);
            people[mi] += p.size;
        }
    }
    let w = world(&econ.markets, &people);
    // Debts between governments (`treaty::Ledger`), in local units: they weigh on the premium
    // and on the default limit as foreign-currency debt does.
    let treaty_debt: Vec<(Fixed, Fixed)> = if econ.ledger.is_empty() {
        vec![(Fixed::ZERO, Fixed::ZERO); nm]
    } else {
        econ.markets
            .iter()
            .enumerate()
            .map(|(mi, m)| {
                let ex = m.money.real_exchange();
                let owed = econ.ledger.owed_by(mi as u16).mul_div(ex, INDEX_BASE);
                let service = econ
                    .ledger
                    .yearly_service(mi as u16, year)
                    .mul_div(ex, INDEX_BASE);
                (owed, service)
            })
            .collect()
    };
    let mut plans = Vec::with_capacity(nm);
    for (mi, m) in econ.markets.iter_mut().enumerate() {
        let income = if people[mi] > 0 {
            m.gdp_last_year / people[mi]
        } else {
            Fixed::ZERO
        };
        plans.push(home_step(
            m,
            activity[mi],
            idle[mi],
            income,
            &w,
            year,
            treaty_debt[mi].0,
        ));
    }
    // Foreign lenders: each market lends part of its idle savings; borrowers get their need
    // pro rata to what can be lent this week.
    let lendable: Fixed = plans.iter().map(|p| p.lendable).sum();
    let asked: Fixed = econ
        .markets
        .iter()
        .zip(&plans)
        .filter(|(m, _)| m.money.excluded_until <= year)
        .map(|(_, p)| p.need)
        .sum();
    let granted_share = if asked > Fixed::ZERO {
        crate::econ::fill(lendable, asked)
    } else {
        Fixed::ZERO
    };
    let mut lent = Fixed::ZERO;
    let mut paid_abroad = Fixed::ZERO;
    let mut written_off = Fixed::ZERO;
    let mut cuts = vec![Fixed::ZERO; nm];
    for (mi, (m, plan)) in econ.markets.iter_mut().zip(plans.iter_mut()).enumerate() {
        let abroad = if m.money.excluded_until <= year {
            plan.need * granted_share
        } else {
            Fixed::ZERO
        };
        // On the rule the lenders take the dollar share they demand (`fx_share`). With the split
        // fixed (D9), the government asks a local share; the lenders take local bonds only up to
        // `1 − fx_share`, and the local bonds they leave are not sold (no switch to dollars).
        let (fx, local) = match plan.local_abroad {
            None => {
                let fx = abroad * plan.fx_share;
                (fx, abroad - fx)
            }
            Some(d) => (
                abroad * (Fixed::ONE - d),
                abroad * d.min(Fixed::ONE - plan.fx_share),
            ),
        };
        let sold = fx + local;
        let real_fx = m.money.real_exchange();
        m.money.debt_fx += fx.mul_div(INDEX_BASE, real_fx);
        m.money.debt_abroad += local;
        m.treasury += sold;
        plan.need -= sold;
        lent += sold;
        if let Some(i) = &mut m.issue_last {
            i.abroad_local = local;
            i.abroad_fx = fx;
            i.unsold_abroad = plan.need;
            i.local_limit = Fixed::ONE - plan.fx_share;
        }
        let (interest, principal, lost, cut) = settle_week(m, plan, year, treaty_debt[mi]);
        cuts[mi] = cut;
        flows.sold[mi] = sold;
        flows.interest_paid[mi] = interest;
        flows.principal_repaid[mi] = principal;
        paid_abroad += interest + principal;
        written_off += lost;
    }
    // Lenders: their savings go out, interest and repayments come back to their pools, and a
    // default takes its loss from their claims, all pro rata. What the debtors paid reaches the
    // lenders to the last unit (largest remainder): interest and principal are split apart, so
    // each lender's two lines add up to what the payers paid on each.
    let claim: Vec<Fixed> = econ
        .markets
        .iter()
        .map(|m| m.money.foreign_assets)
        .collect();
    let claims: Fixed = claim.iter().copied().sum();
    let interest_abroad: Fixed = flows.interest_paid.iter().copied().sum();
    let principal_abroad = paid_abroad - interest_abroad;
    if claims > Fixed::ZERO {
        let mut rest = Vec::new();
        super::trade::split_exact_into(
            interest_abroad,
            &claim,
            &mut flows.interest_received,
            &mut rest,
        );
        super::trade::split_exact_into(
            principal_abroad,
            &claim,
            &mut flows.principal_received,
            &mut rest,
        );
    } else if paid_abroad > Fixed::ZERO {
        // Nobody holds a claim abroad: what the debtors set aside for it stays with them, back
        // in their treasuries, and nothing crosses a border (`AbroadFlows::unreceived`).
        flows.unreceived = (interest_abroad, principal_abroad);
        for (m, (i, p)) in econ.markets.iter_mut().zip(
            flows
                .interest_paid
                .iter_mut()
                .zip(flows.principal_repaid.iter_mut()),
        ) {
            m.treasury += *i + *p;
            *i = Fixed::ZERO;
            *p = Fixed::ZERO;
        }
    }
    let mut take = vec![Fixed::ZERO; nm];
    for (mi, (m, plan)) in econ.markets.iter_mut().zip(&plans).enumerate() {
        if claims > Fixed::ZERO {
            m.pool += flows.interest_received[mi] + flows.principal_received[mi];
            m.money.foreign_assets -= written_off * (claim[mi] / claims);
        }
        let out = if lendable > Fixed::ZERO {
            lent.mul_div(plan.lendable, lendable)
        } else {
            Fixed::ZERO
        };
        flows.lent[mi] = out;
        m.money.foreign_assets = (m.money.foreign_assets + out).max(Fixed::ZERO);
        // Banks lend what is left of this week's share to the investment pool.
        let to_pool = (plan.idle * BANK_LENDING - plan.from_households - out).max(Fixed::ZERO);
        m.pool += to_pool;
        // What leaves the households: bonds bought at home, loans abroad and to the pool.
        take[mi] = if plan.idle > Fixed::ZERO {
            crate::econ::fill(plan.from_households + out + to_pool, plan.idle)
        } else {
            Fixed::ZERO
        };
        // Lenders' claims lose value with world inflation.
        m.money.foreign_assets -= weekly(m.money.foreign_assets, w.inflation.max(Fixed::ZERO));
    }
    for r in &mut econ.regions {
        let mi = r.market as usize;
        if take[mi] <= Fixed::ZERO {
            continue;
        }
        for p in &mut r.pops {
            p.savings -= idle_savings(p) * take[mi];
        }
    }
    if !econ.ledger.is_empty() {
        flows.ledger = treaty_service(econ, &cuts, &w, year);
    }
    flows
}

/// The week of the debts between governments: a default cuts the debtor's lines in the same
/// proportion as its market debt (each creditor loses its claim), then the service moves from
/// the debtors' treasuries to the creditors'. The service counts in the debtor's interest bill
/// and the stock in its debt. Returns what moved, by line: (debtor, creditor, interest,
/// principal).
fn treaty_service(
    econ: &mut Economy,
    cuts: &[Fixed],
    w: &World,
    year: i32,
) -> Vec<(u16, u16, Fixed, Fixed)> {
    for (mi, &cut) in cuts.iter().enumerate() {
        if cut > Fixed::ZERO {
            econ.ledger.haircut(mi as u16, cut);
        }
    }
    let n = econ.markets.len();
    let mut treasuries: Vec<Fixed> = econ.markets.iter().map(|m| m.treasury).collect();
    let revenue: Vec<Fixed> = econ.markets.iter().map(|m| m.budget.revenue_avg).collect();
    let exchange: Vec<Fixed> = econ
        .markets
        .iter()
        .map(|m| m.money.real_exchange().mul_div(Fixed::ONE, INDEX_BASE))
        .collect();
    let hundred = Fixed::from_int(100);
    let inflation: Vec<Fixed> = econ
        .markets
        .iter()
        .map(|m| m.money.inflation.mul_div(Fixed::ONE, hundred))
        .collect();
    let services = econ.ledger.service_week(
        &mut treasuries,
        &revenue,
        &exchange,
        &inflation,
        w.inflation.mul_div(Fixed::ONE, hundred),
        year,
    );
    let mut paid = vec![Fixed::ZERO; n];
    let mut moved = Vec::new();
    for (d, s) in econ.ledger.debts.iter().zip(&services) {
        paid[d.debtor as usize] += s.paid;
        if s.paid != Fixed::ZERO {
            moved.push((d.debtor, d.creditor, s.paid - s.principal, s.principal));
        }
    }
    econ.ledger.settle(year);
    for (mi, m) in econ.markets.iter_mut().enumerate() {
        m.treasury = treasuries[mi];
        m.budget.interest += paid[mi];
        let owed = econ
            .ledger
            .owed_by(mi as u16)
            .mul_div(m.money.real_exchange(), INDEX_BASE);
        m.budget.debt = m.money.total_debt() + owed;
    }
    moved
}

/// Prices, the central bank and the bonds sold at home for one market; returns what is left
/// for foreign lenders.
fn home_step(
    m: &mut Market,
    act: Activity,
    idle: Fixed,
    income: Fixed,
    w: &World,
    year: i32,
    treaty_debt: Fixed,
) -> Plan {
    let gdp = m.gdp_last_year;
    let gdp_week = gdp / 52;
    let premium = premium(m, gdp, idle, treaty_debt);
    let subordinate = m.money.subordinate(year);
    let money = &mut m.money;
    // Trends start at the first week's values.
    if money.utilization_trend <= Fixed::ZERO {
        money.utilization_trend = act.utilization;
        money.index_trend = act.index;
    }
    let gap = (act.utilization - money.utilization_trend).clamp(-MAX_GAP, MAX_GAP);
    let heat = (act.index - money.index_trend).clamp(-MAX_HEAT, MAX_HEAT);
    money.utilization_trend += (act.utilization - money.utilization_trend) / TREND_WEEKS;
    money.index_trend += (act.index - money.index_trend) / TREND_WEEKS;
    // Inflation: expectations (anchored on the target by an autonomous bank, on the past and
    // the world otherwise), the business cycle, and money lent to the treasury.
    let anchor = if subordinate {
        money.expected * milli(600) + w.inflation * milli(200) + TARGET * milli(200)
    } else {
        TARGET * milli(700) + money.expected * milli(300)
    };
    let target = anchor + PHILLIPS * gap + HEAT * heat + MONEY_PASS * money.monetized;
    money.inflation = (money.inflation + (target - money.inflation) / INFLATION_WEEKS)
        .clamp(INFLATION_FLOOR, INFLATION_CEILING);
    money.expected += (money.inflation - money.expected) / EXPECTATION_WEEKS;
    money.price_level += weekly(money.price_level, money.inflation);
    money.parity += weekly(money.parity, money.inflation - w.inflation);
    if money.price_level > REDENOMINATION {
        money.price_level = money.price_level.mul_ratio(1, 1000);
        money.price_level_jan = money.price_level_jan.mul_ratio(1, 1000);
        money.parity = money.parity.mul_ratio(1, 1000);
        money.exchange_rate = money.exchange_rate.mul_ratio(1, 1000);
        money.redenominations += 1;
    }
    let target_rate = exchange_target(money, premium);
    money.exchange_rate += (target_rate - money.exchange_rate) / EXCHANGE_WEEKS;
    // The central bank: Taylor rule, or the player's rate; floored at zero.
    let neutral = NEUTRAL_REAL + money.inflation;
    let taylor = neutral + TAYLOR_INFLATION * (money.inflation - TARGET) + TAYLOR_GAP * gap;
    let wanted = money.rate_command.unwrap_or(taylor);
    money.policy_rate = wanted.clamp(Fixed::ZERO, MAX_RATE);
    // Money the bank creates this week (negative: withdraws), in points of weekly GDP: what a
    // growing economy holds plus the stance against the neutral rate (below the zero floor the
    // rule keeps easing). A subordinate bank never withdraws.
    let stance = (neutral - wanted).clamp(-MAX_STANCE, MAX_STANCE);
    let rule_stance = (neutral - taylor).clamp(-MAX_STANCE, MAX_STANCE);
    let mut purchase = gdp_week.mul_div(NEUTRAL_MONEY + stance, Fixed::from_int(100));
    let beyond_rule = gdp_week.mul_div(
        (stance - rule_stance).max(Fixed::ZERO),
        Fixed::from_int(100),
    );
    if subordinate {
        purchase = purchase.max(Fixed::ZERO);
    }
    // Market rates: the world rate with this country's inflation, half way to the bank's rate,
    // plus the premium; foreign-currency bonds pay the world rate plus the premium.
    let local = (money.policy_rate * milli(500)
        + (w.rate + money.inflation - w.inflation) * milli(500)
        + premium)
        .max(BOND_FLOOR);
    let fx = (w.rate + premium.max(Fixed::ZERO)).max(BOND_FLOOR);
    money.bond_rate = local;
    money.fx_rate = fx;
    money.cost_local += (local - money.cost_local) / MATURITY_WEEKS;
    money.cost_fx += (fx - money.cost_fx) / MATURITY_WEEKS;
    // Interest of the week. The central bank hands its interest back to the treasury.
    let home_interest = weekly(money.debt_home, money.cost_local);
    let abroad_interest =
        weekly(money.debt_abroad, money.cost_local) + weekly(money.debt_fx_local(), money.cost_fx);
    m.treasury -= home_interest + abroad_interest;
    m.pool += home_interest;
    m.budget.interest = home_interest + abroad_interest;
    let money = &mut m.money;
    // Inflation and the real exchange rate change the real value of the stock.
    let erosion = money.inflation.max(INFLATION_FLOOR);
    money.debt_home -= weekly(money.debt_home, erosion);
    money.debt_bank -= weekly(money.debt_bank, erosion);
    money.debt_abroad -= weekly(money.debt_abroad, erosion);
    money.debt_fx -= weekly(money.debt_fx, w.inflation.max(Fixed::ZERO));
    money.credit -= weekly(money.credit, erosion);
    // The treasury tops its cash up to the reserve (the rule's weeks, or the fixed ones, D4).
    let reserve = m.budget.revenue_avg * m.levers.reserve_weeks();
    let mut need = (reserve - m.treasury).max(Fixed::ZERO);
    // Where the treasury issues (D9): fixed, home savers and the central bank see only the home
    // share, and what is meant for abroad goes abroad; what a place does not buy is not sold
    // elsewhere. On the rule, home savers first, then the bank, then abroad.
    let split = m.levers.issuance_shares();
    let mut abroad_asked = Fixed::ZERO;
    if let Some([home_s, local_s, fx_s]) = split {
        abroad_asked = need * (local_s + fx_s);
        need *= home_s;
    }
    // 1. Savers at home: idle household savings, then banks and capitalists (the pool).
    let depth = if w.income > Fixed::ZERO {
        let rich = (income / (w.income * 2)).min(Fixed::ONE);
        (rich * rich).max(SHALLOW_MARKET)
    } else {
        Fixed::ONE
    };
    let cap_households = idle * HOME_ABSORB * depth;
    let cap_pool = m.pool.max(Fixed::ZERO) * POOL_ABSORB * depth;
    let home = need.min(cap_households + cap_pool);
    let from_households = if cap_households + cap_pool > Fixed::ZERO {
        home.mul_div(cap_households, cap_households + cap_pool)
    } else {
        Fixed::ZERO
    };
    m.pool -= home - from_households;
    m.money.debt_home += home;
    m.treasury += home;
    need -= home;
    // 2. The central bank: a subordinate bank buys the new bonds the treasury still needs
    // first (an autonomous one may not lend to the treasury); the rest is lent to banks. When
    // it withdraws, it calls back loans first, then sells bonds to savers at home.
    let mut lent_to_treasury = Fixed::ZERO;
    let mut from_households = from_households;
    if purchase > Fixed::ZERO {
        let primary = if subordinate {
            purchase.min(need)
        } else {
            Fixed::ZERO
        };
        lent_to_treasury += primary;
        need -= primary;
        let credit = purchase - primary;
        m.money.credit += credit;
        m.pool += credit;
    } else if purchase < Fixed::ZERO {
        let called = (-purchase).min(m.money.credit).min(m.pool.max(Fixed::ZERO));
        m.money.credit -= called;
        m.pool -= called;
        purchase += called;
        let room = (cap_households + cap_pool - home).max(Fixed::ZERO);
        let sold = (-purchase).min(m.money.debt_bank).min(room);
        m.money.debt_bank -= sold;
        m.money.debt_home += sold;
        let by_households = if room > Fixed::ZERO {
            sold.mul_div((cap_households - from_households).max(Fixed::ZERO), room)
        } else {
            Fixed::ZERO
        };
        from_households += by_households;
        m.pool -= sold - by_households;
    }
    m.money.debt_bank += lent_to_treasury;
    m.treasury += lent_to_treasury;
    m.budget.issued = lent_to_treasury;
    m.budget.beyond_rule = beyond_rule;
    let fx_share = fx_share(m, income, w);
    let local_abroad = split.map(|[_, local_s, fx_s]| {
        let abroad = local_s + fx_s;
        if abroad > Fixed::ZERO {
            local_s / abroad
        } else {
            Fixed::ZERO
        }
    });
    m.issue_last = split.map(|_| IssueWeek {
        home: home + lent_to_treasury,
        unsold_home: need,
        ..Default::default()
    });
    Plan {
        need: if split.is_some() { abroad_asked } else { need },
        from_households,
        idle,
        lendable: (idle - from_households).max(Fixed::ZERO) * FOREIGN_ABSORB,
        fx_share,
        abroad_interest,
        local_abroad,
    }
}

/// After foreign lending: the subordinate bank covers the rest, a surplus repays debt, and a
/// foreign-currency service past the limit defaults.
/// `treaty`: what the country owes other governments and its yearly service, local units. Returns
/// what was paid to foreign holders (interest, principal), what they lost, and the share of the
/// treaty debts a default writes off (zero without a default).
fn settle_week(
    m: &mut Market,
    plan: &Plan,
    year: i32,
    treaty: (Fixed, Fixed),
) -> (Fixed, Fixed, Fixed, Fixed) {
    let (treaty_owed, treaty_service) = treaty;
    let gdp_week = m.gdp_last_year / 52;
    let mut forced = Fixed::ZERO;
    // With the split fixed (D9), the bank does not cover what was meant for abroad.
    if plan.need > Fixed::ZERO && m.money.subordinate(year) && plan.local_abroad.is_none() {
        m.money.debt_bank += plan.need;
        m.treasury += plan.need;
        m.budget.issued += plan.need;
        forced = plan.need;
    }
    let mut repaid = Fixed::ZERO;
    // A surplus pays back foreign-currency debt first, then debt held abroad, then at home.
    let reserve = m.budget.revenue_avg * m.levers.reserve_weeks();
    let excess = m.treasury - reserve * 2;
    // The rule repays above 60 % of GDP; a surplus fixed for repaying repays whatever the debt,
    // and one kept repays nothing (D4).
    let repays = match m.levers.surplus() {
        super::levers::SurplusUse::Spend => m.money.total_debt() > m.gdp_last_year * REPAY_ABOVE,
        super::levers::SurplusUse::Repay => m.money.total_debt() > Fixed::ZERO,
        super::levers::SurplusUse::Keep => false,
    };
    if excess > Fixed::ZERO && repays {
        let mut repay = excess * REPAY_SHARE;
        let fx = repay.min(m.money.debt_fx_local());
        if fx > Fixed::ZERO {
            m.money.debt_fx -= fx.mul_div(INDEX_BASE, m.money.real_exchange());
            m.money.debt_fx = m.money.debt_fx.max(Fixed::ZERO);
        }
        let abroad = (repay - fx).min(m.money.debt_abroad);
        m.money.debt_abroad -= abroad;
        repaid += fx + abroad;
        repay -= fx + abroad;
        let home = repay.min(m.money.debt_home);
        m.money.debt_home -= home;
        m.pool += home;
        m.treasury -= fx + abroad + home;
    }
    // Money created beyond the rule, as points of GDP a year, smoothed over a year.
    let monetized = percent_of(forced + m.budget.beyond_rule, gdp_week);
    m.money.monetized += (monetized - m.money.monetized) / 52;
    // Default: on a foreign-currency service past the limit, or on arrears of a quarter of a
    // year's revenue owing abroad; at most once a year.
    let mut lost = Fixed::ZERO;
    let yearly_revenue = m.budget.revenue_avg * 52;
    let service = m
        .money
        .debt_fx_local()
        .mul_div(m.money.cost_fx + AMORTIZATION, Fixed::from_int(100))
        + treaty_service;
    let unpayable =
        yearly_revenue > Fixed::ZERO && percent_of(service, yearly_revenue) > DEFAULT_LIMIT;
    let arrears = m.treasury < -yearly_revenue.mul_ratio(1, 4)
        && (m.money.debt_fx > Fixed::ZERO
            || m.money.debt_abroad > Fixed::ZERO
            || treaty_owed > Fixed::ZERO);
    let crushing = m.budget.interest + treaty_service / 52 > m.budget.revenue_avg * INTEREST_LIMIT;
    let mut cut = Fixed::ZERO;
    if crushing && m.money.excluded_until - EXCLUSION_YEARS < year {
        // Restructuring of the whole debt held outside the central bank.
        let home = m.money.debt_home * RESTRUCTURING;
        let abroad = m.money.debt_abroad * RESTRUCTURING;
        let fx = m.money.debt_fx * RESTRUCTURING;
        m.money.debt_home -= home;
        m.money.debt_abroad -= abroad;
        m.money.debt_fx -= fx;
        lost = abroad + fx;
        cut = RESTRUCTURING;
        m.money.defaults += 1;
        m.money.default_history += Fixed::ONE;
        m.money.excluded_until = year + EXCLUSION_YEARS;
    } else if (unpayable || arrears) && m.money.excluded_until - EXCLUSION_YEARS < year {
        lost = m.money.debt_fx * HAIRCUT;
        m.money.debt_fx -= lost;
        if arrears {
            let abroad = m.money.debt_abroad * HAIRCUT;
            m.money.debt_abroad -= abroad;
            lost += abroad;
        }
        cut = HAIRCUT;
        m.money.defaults += 1;
        m.money.default_history += Fixed::ONE;
        m.money.excluded_until = year + EXCLUSION_YEARS;
    }
    m.budget.debt = m.money.total_debt();
    (plan.abroad_interest, repaid, lost, cut)
}

/// On January 1st: last year's inflation, the decay of the default history, and the historical
/// rule's switch to an autonomous bank.
pub fn yearly(m: &mut Market, year: i32) {
    let money = &mut m.money;
    if money.price_level_jan > Fixed::ZERO {
        money.inflation_last_year =
            percent_of(money.price_level, money.price_level_jan) - Fixed::from_int(100);
    }
    money.price_level_jan = money.price_level;
    money.default_history *= HISTORY_DECAY;
    if money.bank == CentralBank::Historical
        && money.autonomous_since == 0
        && (year >= AUTONOMY_YEAR || money.inflation_last_year > STABILIZATION)
    {
        money.autonomous_since = year;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn market(gdp: i64) -> Market {
        let mut m = Market::empty(0);
        m.gdp_last_year = Fixed::from_int(gdp);
        m.budget.revenue_avg = Fixed::from_int(gdp / 52 / 10);
        m.money = Money::start(m.gdp_last_year);
        m
    }

    fn calm() -> Activity {
        Activity {
            utilization: Fixed::from_int(90),
            index: Fixed::from_int(100),
        }
    }

    fn reparation(debtor: u16, creditor: u16, principal: i64) -> crate::treaty::InterstateDebt {
        crate::treaty::InterstateDebt {
            id: 0,
            debtor,
            creditor,
            treaty: "t".into(),
            currency: crate::treaty::Currency::Reference,
            principal: Fixed::from_int(principal),
            rate: Fixed::from_ratio(5, 100),
            annuity: Fixed::from_int(principal / 20),
            years_left: 30,
            deferrable: Fixed::ZERO,
            transfer_cap: None,
            pledge: None,
            contingent: Fixed::ZERO,
            arrears: Fixed::ZERO,
            suspended_until: 0,
            paid: Fixed::ZERO,
            written_off: Fixed::ZERO,
            status: Default::default(),
        }
    }

    #[test]
    fn treaty_debt_moves_money_between_treasuries_and_is_cut_in_a_default() {
        let mut econ = Economy {
            markets: vec![market(1_000_000), market(2_000_000)],
            ..Economy::default()
        };
        econ.ledger.create(reparation(0, 1, 500_000));
        let w = World {
            inflation: Fixed::from_int(2),
            rate: Fixed::from_int(4),
            income: Fixed::ONE,
        };
        let cash = |e: &Economy| e.markets.iter().map(|m| m.treasury).sum::<Fixed>();
        let before = cash(&econ);
        for week in 0..52 {
            treaty_service(&mut econ, &[Fixed::ZERO; 2], &w, 1926 + week / 52);
        }
        // The service left the debtor and reached the creditor; nothing was created.
        assert_eq!(cash(&econ), before);
        assert!(econ.markets[0].treasury < econ.markets[1].treasury);
        assert!(econ.markets[0].budget.interest > Fixed::ZERO);
        // The debt shows in the debtor's stock, the claim is the same number.
        assert!(econ.markets[0].budget.debt > econ.markets[0].money.total_debt());
        assert_eq!(econ.ledger.owed_by(0), econ.ledger.claims_of(1));
        // A default writes off the same share of the treaty debt.
        let owed = econ.ledger.owed_by(0);
        treaty_service(&mut econ, &[HAIRCUT, Fixed::ZERO], &w, 1927);
        assert!(econ.ledger.owed_by(0) < owed * (Fixed::ONE - HAIRCUT) + Fixed::ONE);
        assert_eq!(cash(&econ), before);
    }

    #[test]
    fn the_treaty_service_counts_in_the_default_limit() {
        // The same market, with and without a heavy treaty service: only the second defaults.
        let plan = |m: &mut Market| {
            home_step(
                m,
                calm(),
                Fixed::ZERO,
                Fixed::ONE,
                &World {
                    inflation: Fixed::from_int(2),
                    rate: Fixed::from_int(4),
                    income: Fixed::ONE,
                },
                1950,
                Fixed::ZERO,
            )
        };
        let mut free = market(1_000_000);
        let p = plan(&mut free);
        let (_, _, _, cut) = settle_week(&mut free, &p, 1950, (Fixed::ZERO, Fixed::ZERO));
        assert_eq!(cut, Fixed::ZERO);
        let mut bound = market(1_000_000);
        let p = plan(&mut bound);
        let yearly_revenue = bound.budget.revenue_avg * 52;
        let (_, _, _, cut) = settle_week(&mut bound, &p, 1950, (yearly_revenue, yearly_revenue));
        assert!(cut > Fixed::ZERO);
        assert_eq!(bound.money.defaults, 1);
    }

    #[test]
    fn a_weekly_rate_in_percent() {
        assert_eq!(weekly(Fixed::from_int(5200), pct(100)), Fixed::from_int(10));
        assert_eq!(
            percent_of(Fixed::from_int(1), Fixed::from_int(4)),
            Fixed::from_int(25)
        );
    }

    #[test]
    fn a_deficit_becomes_debt_at_home_first() {
        let mut m = market(52_000);
        m.treasury = Fixed::from_int(-1000);
        let w = World {
            inflation: TARGET,
            rate: NEUTRAL_REAL + TARGET,
            income: Fixed::ONE,
        };
        let debt = m.money.debt_home;
        // A country twice as rich as the world: a deep market at home.
        let plan = home_step(
            &mut m,
            calm(),
            Fixed::from_int(1_000_000),
            Fixed::from_int(2),
            &w,
            1950,
            Fixed::ZERO,
        );
        assert!(m.money.debt_home > debt, "home savers bought bonds");
        assert!(plan.from_households > Fixed::ZERO);
        assert_eq!(plan.need, Fixed::ZERO);
        assert!(m.treasury >= Fixed::ZERO);
    }

    #[test]
    fn a_subordinate_bank_prints_what_nobody_buys() {
        let mut m = market(52_000);
        m.money.bank = CentralBank::Subordinate;
        m.money.excluded_until = 3000;
        m.treasury = Fixed::from_int(-5000);
        let w = World {
            inflation: TARGET,
            rate: NEUTRAL_REAL + TARGET,
            income: Fixed::ONE,
        };
        let bank = m.money.debt_bank;
        let plan = home_step(
            &mut m,
            calm(),
            Fixed::ZERO,
            Fixed::ONE,
            &w,
            1950,
            Fixed::ZERO,
        );
        settle_week(&mut m, &plan, 1950, (Fixed::ZERO, Fixed::ZERO));
        assert!(m.money.debt_bank > bank + Fixed::from_int(4000));
        assert!(m.money.monetized > Fixed::ZERO);
    }

    #[test]
    fn inflation_erodes_debt_in_own_currency_only() {
        let mut m = market(52_000);
        m.money.inflation = pct(500);
        m.money.expected = pct(500);
        m.money.debt_fx = Fixed::from_int(10_000);
        m.treasury = Fixed::from_int(1300);
        let home = m.money.debt_home;
        let w = World {
            inflation: Fixed::ZERO,
            rate: NEUTRAL_REAL,
            income: Fixed::ONE,
        };
        home_step(
            &mut m,
            calm(),
            Fixed::from_int(1_000_000),
            Fixed::ONE,
            &w,
            1950,
            Fixed::ZERO,
        );
        assert!(m.money.debt_home < home);
        assert_eq!(m.money.debt_fx, Fixed::from_int(10_000));
    }

    #[test]
    fn heavy_foreign_currency_service_defaults() {
        let mut m = market(52_000);
        m.money.debt_fx = Fixed::from_int(40_000);
        let plan = Plan {
            need: Fixed::ZERO,
            from_households: Fixed::ZERO,
            idle: Fixed::ZERO,
            lendable: Fixed::ZERO,
            fx_share: Fixed::ZERO,
            abroad_interest: Fixed::ZERO,
            local_abroad: None,
        };
        let (_, _, lost, _) = settle_week(&mut m, &plan, 1950, (Fixed::ZERO, Fixed::ZERO));
        assert_eq!(lost, Fixed::from_int(20_000));
        assert_eq!(m.money.defaults, 1);
        assert_eq!(m.money.excluded_until, 1953);
        // The same debt in the country's own currency, held at home, does not default.
        let mut japan = market(52_000);
        japan.money.debt_home = Fixed::from_int(100_000);
        let (_, _, lost, _) = settle_week(&mut japan, &plan, 1950, (Fixed::ZERO, Fixed::ZERO));
        assert_eq!(lost, Fixed::ZERO);
        assert_eq!(japan.money.defaults, 0);
    }

    #[test]
    fn the_rule_tightens_when_inflation_runs_above_target() {
        let mut m = market(52_000);
        m.money.inflation = pct(100);
        m.money.expected = pct(100);
        let w = World {
            inflation: TARGET,
            rate: NEUTRAL_REAL + TARGET,
            income: Fixed::ONE,
        };
        let hot = Activity {
            utilization: Fixed::from_int(90),
            index: Fixed::from_int(100),
        };
        home_step(&mut m, hot, Fixed::ZERO, Fixed::ONE, &w, 1995, Fixed::ZERO);
        assert!(m.money.policy_rate > NEUTRAL_REAL + m.money.inflation);
        // The player's rate wins, and money beyond the rule shows.
        m.money.rate_command = Some(Fixed::ZERO);
        home_step(&mut m, hot, Fixed::ZERO, Fixed::ONE, &w, 1995, Fixed::ZERO);
        assert!(m.budget.beyond_rule > Fixed::ZERO);
        assert_eq!(m.money.policy_rate, Fixed::ZERO);
    }

    #[test]
    fn the_japanese_case_and_the_dollar_debt() {
        // Twice GDP in its own currency, held at home, with savings to spare: a negative
        // premium, and no default.
        let mut japan = market(52_000);
        japan.money.debt_home = Fixed::from_int(104_000);
        let jp = premium(
            &japan,
            japan.gdp_last_year,
            Fixed::from_int(156_000),
            Fixed::ZERO,
        );
        assert!(jp < Fixed::ZERO, "{jp}");
        // Two fifths of GDP in dollars, few savings: dearer, and past the default limit.
        let mut argentina = market(52_000);
        argentina.money.debt_fx = Fixed::from_int(20_800);
        let ar = premium(
            &argentina,
            argentina.gdp_last_year,
            Fixed::ZERO,
            Fixed::ZERO,
        );
        assert!(ar > jp + pct(20), "{ar} against {jp}");
        let plan = Plan {
            need: Fixed::ZERO,
            from_households: Fixed::ZERO,
            idle: Fixed::ZERO,
            lendable: Fixed::ZERO,
            fx_share: Fixed::ZERO,
            abroad_interest: Fixed::ZERO,
            local_abroad: None,
        };
        settle_week(&mut japan, &plan, 1990, (Fixed::ZERO, Fixed::ZERO));
        settle_week(&mut argentina, &plan, 1990, (Fixed::ZERO, Fixed::ZERO));
        assert_eq!(japan.money.defaults, 0);
        assert_eq!(argentina.money.defaults, 1);
        // The default raises the premium for years.
        let after = premium(
            &argentina,
            argentina.gdp_last_year,
            Fixed::ZERO,
            Fixed::ZERO,
        );
        assert!(after > DEFAULT_RISK / 2, "{after}");
    }

    #[test]
    fn an_autonomous_bank_does_not_lend_to_the_treasury() {
        let mut m = market(52_000);
        m.money.bank = CentralBank::Autonomous;
        m.treasury = Fixed::from_int(-5000);
        let w = World {
            inflation: TARGET,
            rate: NEUTRAL_REAL + TARGET,
            income: Fixed::ONE,
        };
        let plan = home_step(
            &mut m,
            calm(),
            Fixed::ZERO,
            Fixed::ONE,
            &w,
            1950,
            Fixed::ZERO,
        );
        assert_eq!(m.budget.issued, Fixed::ZERO);
        assert!(m.money.credit > Fixed::ZERO, "its money went to banks");
        assert!(plan.need > Fixed::from_int(5000), "the gap is asked abroad");
    }

    #[test]
    fn the_historical_bank_becomes_autonomous() {
        let mut m = market(52_000);
        assert!(m.money.subordinate(1950));
        yearly(&mut m, 1950);
        assert!(m.money.subordinate(1951));
        yearly(&mut m, 1990);
        assert!(!m.money.subordinate(1990));
        let mut hyper = market(52_000);
        hyper.money.price_level = Fixed::from_int(10_000);
        yearly(&mut hyper, 1960);
        assert_eq!(hyper.money.autonomous_since, 1960);
    }
}
