//! The balance of payments, measured: what crosses each border, by country, in the week, the
//! year so far and the last full year. A record only: no rule reads it and the world's hash
//! leaves it out (`GameWorld::payments`), so the simulation runs as without it.
//!
//! - **Goods**: exports and imports by pair at base prices (`week::Week::trade_by_pair`).
//! - **Services**: the services and finance a customs union trades (`week::union_services`).
//! - **Income**: interest on government bonds held abroad (`money::week`), the interest of the
//!   debts between governments (`treaty::Ledger::service_week`), and the profit foreign owners
//!   take home (`week::settle_region`, and the earnings kept for an expansion that did not
//!   come, released by `monthly::Investment`).
//! - **Transfers**: the money one treasury pays another by treaty or occupation. Trade's margins
//!   go back to each market's own pool (`week::margin_transfers`) and cross no border.
//! - **Financial account**: bonds sold and bought abroad, the principal repaid on them and on the
//!   debts between governments, and the works foreign funds pay for (`monthly::Investment`).
//!   Defaults, write-offs and the erosion of claims change what a claim is worth, not what
//!   moves: they are valuation and stay out.
//! - **Not settled**: `−(current account + financial account)`. Trade moves no money between
//!   countries (each market buys and sells at its own price), so
//!   the balance does not close with reserves; this line is what would have to move if trade
//!   settled (the owner's open question PO1).
//!
//! Money is in reference units, as the treasuries and pools hold it.
//!
//! **Settlement** (`game::Switches::payments_settle`, gold standard slice 2, [`settle`]): each
//! economy week the balance of the week before closes in reserves. Every flow but goods and
//! services already moved money from one country to the other when it happened (interest,
//! principal, bonds, remittances, treaties, works paid from abroad); the margins' transfers are
//! mostly each importer's own payment for its imports given back to its own pool
//! (`week::margin_transfers`), not money from abroad. So:
//! - the goods and services (`Lines::trade`) move between the pools: an exporter's pool
//!   receives what its sales abroad are worth at base prices, an importer's pays;
//! - the reserves change by the whole balance without the margins (`Lines::reserve_flow`): a
//!   surplus adds to them, a deficit takes from them, gold and foreign exchange in proportion.
//!   What a deficit cannot pay from its reserves it pays in its own currency: the countries in
//!   surplus hold it as foreign exchange (`Reserve::fx`), and it is what the country owes
//!   abroad (`ReservesState::owed`), for slice 3's rule. Reserves never go below zero;
//! - new gold: a share of each country's output of the good `gold` goes to its central bank
//!   (`SettleRules::monetary_share`), bought from the producers with money of the same pool.
//!
//! The world's money (pools and treasuries) is unchanged by the settlement to the last unit,
//! and the world's net reserves (reserves less what is owed abroad) change only by new gold.

use laplace_schelling::Fixed;
use laplace_schelling::hash::{HashState, StateHasher};
use serde::{Deserialize, Serialize};

macro_rules! lines {
    ($(#[$doc:meta] $field:ident),* $(,)?) => {
        /// One country's flows over a period, every line non-negative except `margins`
        /// (received when positive). Credits are what comes in, debits what goes out.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Lines {
            $(#[$doc] pub $field: Fixed,)*
        }

        impl std::ops::AddAssign for Lines {
            fn add_assign(&mut self, o: Self) {
                $(self.$field += o.$field;)*
            }
        }

        impl HashState for Lines {
            fn hash_state(&self, h: &mut StateHasher) {
                $(self.$field.hash_state(h);)*
            }
        }
    };
}

lines! {
    /// Goods sold abroad, base prices.
    exports,
    /// Goods bought abroad, base prices.
    imports,
    /// Services and finance sold across a customs union, base prices.
    services_exports,
    /// Services and finance bought across a customs union, base prices.
    services_imports,
    /// Interest on the bonds of other governments this country's savers hold.
    interest_received,
    /// Interest on this government's bonds held abroad.
    interest_paid,
    /// Interest another government paid this treasury on a debt between governments.
    ledger_interest_received,
    /// Interest this treasury paid another government.
    ledger_interest_paid,
    /// Profit of plants abroad sent home to their owners here.
    remittances_received,
    /// Profit of plants here sent to their owners abroad.
    remittances_paid,
    /// Trade's margins between countries: none, each market's own pool gets them (zero).
    margins,
    /// Money another treasury paid this one by treaty or occupation.
    transfers_received,
    /// Money this treasury paid another by treaty or occupation.
    transfers_paid,
    /// Government bonds sold abroad: what lenders abroad paid in.
    bonds_sold,
    /// Bonds of other governments this country's savers bought: what went out.
    bonds_bought,
    /// Principal of foreign bonds repaid to this country's lenders.
    principal_received,
    /// Principal of this government's bonds held abroad, repaid.
    principal_repaid,
    /// Principal another government repaid this treasury on a debt between governments.
    ledger_principal_received,
    /// Principal this treasury repaid another government.
    ledger_principal_repaid,
    /// Works here paid for by foreign funds.
    fdi_in,
    /// Works abroad this country's funds paid for.
    fdi_out,
    /// Long-term private capital in (`econ::capital`): placed here from abroad, or brought home.
    capital_in,
    /// Long-term private capital out: placed abroad from here, or taken home by lenders abroad.
    capital_out,
    /// Short-term capital ("hot money") that came in, as `capital_in`.
    hot_in,
    /// Short-term capital that went out, as `capital_out`.
    hot_out,
    /// Income on this country's private claims abroad, long and short.
    capital_income_received,
    /// Income on the private claims of savers abroad here.
    capital_income_paid,
}

impl Lines {
    pub fn goods(&self) -> Fixed {
        self.exports - self.imports
    }

    pub fn services(&self) -> Fixed {
        self.services_exports - self.services_imports
    }

    /// Interest on bonds, net.
    pub fn interest(&self) -> Fixed {
        self.interest_received - self.interest_paid
    }

    /// Interest of the debts between governments, net.
    pub fn ledger_interest(&self) -> Fixed {
        self.ledger_interest_received - self.ledger_interest_paid
    }

    pub fn remittances(&self) -> Fixed {
        self.remittances_received - self.remittances_paid
    }

    /// Income on private claims abroad (`econ::capital`), net.
    pub fn capital_income(&self) -> Fixed {
        self.capital_income_received - self.capital_income_paid
    }

    /// Primary income: interest of bonds and of the ledger, remittances, and the income of
    /// private claims.
    pub fn income(&self) -> Fixed {
        self.interest() + self.ledger_interest() + self.remittances() + self.capital_income()
    }

    /// Long-term private capital, net inflow (`econ::capital`).
    pub fn capital_long(&self) -> Fixed {
        self.capital_in - self.capital_out
    }

    /// Short-term private capital, net inflow.
    pub fn capital_short(&self) -> Fixed {
        self.hot_in - self.hot_out
    }

    /// Margins and transfers between treasuries.
    pub fn transfers(&self) -> Fixed {
        self.margins + self.transfers_received - self.transfers_paid
    }

    pub fn current_account(&self) -> Fixed {
        self.goods() + self.services() + self.income() + self.transfers()
    }

    /// Net inflow of the financial account: bonds, principal and direct investment.
    pub fn financial_account(&self) -> Fixed {
        self.bonds_sold - self.bonds_bought + self.principal_received - self.principal_repaid
            + self.ledger_principal_received
            - self.ledger_principal_repaid
            + self.fdi_in
            - self.fdi_out
            + self.capital_long()
            + self.capital_short()
    }

    /// What would have to move for the balance to close: `−(current + financial)`.
    pub fn not_settled(&self) -> Fixed {
        -(self.current_account() + self.financial_account())
    }

    /// Goods and services, net: the only flows that cross a border without money moving with
    /// them, and what settlement moves between the pools.
    pub fn trade(&self) -> Fixed {
        self.goods() + self.services()
    }

    /// The balance settled in reserves: the current and financial accounts without trade's
    /// margins, which are each market's own money given back to it, not money from abroad.
    pub fn reserve_flow(&self) -> Fixed {
        self.current_account() + self.financial_account() - self.margins
    }
}

/// One country's lines: the last economy week (and what other systems recorded since), the
/// calendar year so far, and the last full year.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Period {
    pub week: Lines,
    pub year: Lines,
    pub last_year: Lines,
}

/// `GameWorld::payments`: the balance of payments by market index. Outside the hash.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentsState {
    /// The calendar year `Period::year` holds; 0 before the first record.
    pub year: i32,
    pub markets: Vec<Period>,
    /// What debtors owed on bonds held abroad in a week when nobody held a claim abroad, as a
    /// lender would have recorded it (`interest_received`, `principal_received`): it stays with
    /// the debtors (`money::AbroadFlows::unreceived`) and is out of every other line. The
    /// lenders receive what the debtors pay to the last unit, so every line adds up to zero
    /// over the world without it.
    pub unreceived: Period,
}

/// [`Lines`] as worlds before version 41 wrote it, without the private capital
/// (`econ::capital`): frozen, it never changes again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinesBeforeCapital {
    pub exports: Fixed,
    pub imports: Fixed,
    pub services_exports: Fixed,
    pub services_imports: Fixed,
    pub interest_received: Fixed,
    pub interest_paid: Fixed,
    pub ledger_interest_received: Fixed,
    pub ledger_interest_paid: Fixed,
    pub remittances_received: Fixed,
    pub remittances_paid: Fixed,
    pub margins: Fixed,
    pub transfers_received: Fixed,
    pub transfers_paid: Fixed,
    pub bonds_sold: Fixed,
    pub bonds_bought: Fixed,
    pub principal_received: Fixed,
    pub principal_repaid: Fixed,
    pub ledger_principal_received: Fixed,
    pub ledger_principal_repaid: Fixed,
    pub fdi_in: Fixed,
    pub fdi_out: Fixed,
}

macro_rules! convert_lines {
    ($from:ty => $to:ident, $($field:ident),*) => {
        #[allow(clippy::needless_update)]
        impl From<$from> for $to {
            fn from(l: $from) -> Self {
                $to {
                    $($field: l.$field,)*
                    ..Default::default()
                }
            }
        }
    };
}

convert_lines!(LinesBeforeCapital => Lines, exports, imports, services_exports, services_imports,
    interest_received, interest_paid, ledger_interest_received, ledger_interest_paid,
    remittances_received, remittances_paid, margins, transfers_received, transfers_paid,
    bonds_sold, bonds_bought, principal_received, principal_repaid, ledger_principal_received,
    ledger_principal_repaid, fdi_in, fdi_out);
convert_lines!(Lines => LinesBeforeCapital, exports, imports, services_exports, services_imports,
    interest_received, interest_paid, ledger_interest_received, ledger_interest_paid,
    remittances_received, remittances_paid, margins, transfers_received, transfers_paid,
    bonds_sold, bonds_bought, principal_received, principal_repaid, ledger_principal_received,
    ledger_principal_repaid, fdi_in, fdi_out);

/// [`Period`] before version 41.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeriodBeforeCapital {
    pub week: LinesBeforeCapital,
    pub year: LinesBeforeCapital,
    pub last_year: LinesBeforeCapital,
}

/// [`PaymentsState`] as worlds 32 to 40 wrote it, before the private capital: frozen.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentsBeforeCapital {
    pub year: i32,
    pub markets: Vec<PeriodBeforeCapital>,
    pub unreceived: PeriodBeforeCapital,
}

impl From<PeriodBeforeCapital> for Period {
    fn from(p: PeriodBeforeCapital) -> Self {
        Self {
            week: p.week.into(),
            year: p.year.into(),
            last_year: p.last_year.into(),
        }
    }
}

impl From<&Period> for PeriodBeforeCapital {
    fn from(p: &Period) -> Self {
        Self {
            week: p.week.into(),
            year: p.year.into(),
            last_year: p.last_year.into(),
        }
    }
}

impl From<PaymentsBeforeCapital> for PaymentsState {
    fn from(p: PaymentsBeforeCapital) -> Self {
        Self {
            year: p.year,
            markets: p.markets.into_iter().map(Period::from).collect(),
            unreceived: p.unreceived.into(),
        }
    }
}

impl From<&PaymentsState> for PaymentsBeforeCapital {
    fn from(p: &PaymentsState) -> Self {
        Self {
            year: p.year,
            markets: p.markets.iter().map(PeriodBeforeCapital::from).collect(),
            unreceived: (&p.unreceived).into(),
        }
    }
}

impl PaymentsState {
    /// Grows the record to `n` markets and, in a new calendar year, closes the old one.
    pub fn fit(&mut self, n: usize, year: i32) {
        if self.markets.len() < n {
            self.markets.resize(n, Period::default());
        }
        if year > self.year {
            // A year without any record in between leaves nothing behind it.
            let gap = self.year != 0 && year > self.year + 1;
            for p in self.markets.iter_mut().chain([&mut self.unreceived]) {
                p.last_year = if gap {
                    Lines::default()
                } else {
                    std::mem::take(&mut p.year)
                };
                p.year = Lines::default();
            }
            self.year = year;
        }
    }

    /// A new economy week starts: its lines start from zero.
    pub fn begin_week(&mut self, n: usize, year: i32) {
        self.fit(n, year);
        for p in self.markets.iter_mut().chain([&mut self.unreceived]) {
            p.week = Lines::default();
        }
    }

    /// Adds `lines` to market `mi`'s week and year.
    pub fn add(&mut self, mi: usize, year: i32, lines: Lines) {
        if self.markets.len() <= mi {
            self.fit(mi + 1, year);
        }
        self.fit(self.markets.len(), year);
        let p = &mut self.markets[mi];
        p.week += lines;
        p.year += lines;
    }

    /// Adds to the world's line of what no lender received.
    pub fn add_unreceived(&mut self, year: i32, lines: Lines) {
        self.fit(self.markets.len(), year);
        self.unreceived.week += lines;
        self.unreceived.year += lines;
    }

    /// One treasury pays another (a treaty's payment, occupation money).
    pub fn transfer(&mut self, from: usize, to: usize, year: i32, amount: Fixed) {
        if from == to || amount == Fixed::ZERO {
            return;
        }
        self.add(
            from,
            year,
            Lines {
                transfers_paid: amount,
                ..Lines::default()
            },
        );
        self.add(
            to,
            year,
            Lines {
                transfers_received: amount,
                ..Lines::default()
            },
        );
    }
}

impl PaymentsState {
    /// The week's trade by pair (`seller × n + buyer`, base prices), and the part of it that
    /// is services across a customs union.
    pub fn record_trade(
        &mut self,
        year: i32,
        n: usize,
        trade_value: &[Fixed],
        services: Option<&[Fixed]>,
    ) {
        self.fit(n, year);
        let mut lines = vec![Lines::default(); n];
        for (k, &v) in trade_value.iter().enumerate() {
            let (s, b) = (k / n, k % n);
            if s == b || v == Fixed::ZERO {
                continue;
            }
            let sv = services.map_or(Fixed::ZERO, |x| x[k]);
            lines[s].exports += v - sv;
            lines[b].imports += v - sv;
            lines[s].services_exports += sv;
            lines[b].services_imports += sv;
        }
        for (mi, l) in lines.into_iter().enumerate() {
            self.add(mi, year, l);
        }
    }

    /// Profit a plant in `host` sends to its owner in `owner`.
    pub fn record_remittance(&mut self, year: i32, owner: usize, host: usize, amount: Fixed) {
        if owner == host || amount == Fixed::ZERO {
            return;
        }
        let (paid, received) = (
            Lines {
                remittances_paid: amount,
                ..Lines::default()
            },
            Lines {
                remittances_received: amount,
                ..Lines::default()
            },
        );
        self.add(host, year, paid);
        self.add(owner, year, received);
    }

    /// A work in `host` paid for by a fund of `owner`.
    pub fn record_investment(&mut self, year: i32, owner: usize, host: usize, amount: Fixed) {
        if owner == host || amount == Fixed::ZERO {
            return;
        }
        let (out, into) = (
            Lines {
                fdi_out: amount,
                ..Lines::default()
            },
            Lines {
                fdi_in: amount,
                ..Lines::default()
            },
        );
        self.add(owner, year, out);
        self.add(host, year, into);
    }

    /// The week's bonds abroad and debts between governments (`money::week`).
    pub fn record_money(&mut self, year: i32, f: &super::money::AbroadFlows) {
        for mi in 0..f.sold.len() {
            self.add(
                mi,
                year,
                Lines {
                    bonds_sold: f.sold[mi],
                    bonds_bought: f.lent[mi],
                    interest_paid: f.interest_paid[mi],
                    principal_repaid: f.principal_repaid[mi],
                    interest_received: f.interest_received[mi],
                    principal_received: f.principal_received[mi],
                    ..Lines::default()
                },
            );
        }
        for (mi, c) in f.capital.iter().enumerate() {
            self.add(
                mi,
                year,
                Lines {
                    capital_in: c.capital_in,
                    capital_out: c.capital_out,
                    hot_in: c.hot_in,
                    hot_out: c.hot_out,
                    capital_income_received: c.income_received,
                    capital_income_paid: c.income_paid,
                    ..Lines::default()
                },
            );
        }
        self.add_unreceived(
            year,
            Lines {
                interest_received: f.unreceived.0,
                principal_received: f.unreceived.1,
                ..Lines::default()
            },
        );
        for &(debtor, creditor, interest, principal) in &f.ledger {
            let (debtor, creditor) = (usize::from(debtor), usize::from(creditor));
            self.add(
                debtor,
                year,
                Lines {
                    ledger_interest_paid: interest,
                    ledger_principal_repaid: principal,
                    ..Lines::default()
                },
            );
            self.add(
                creditor,
                year,
                Lines {
                    ledger_interest_received: interest,
                    ledger_principal_received: principal,
                    ..Lines::default()
                },
            );
        }
    }
}

/// One country's reserves, in reference units: monetary gold and foreign exchange, by the
/// market that issued the currency.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reserve {
    pub gold: Fixed,
    /// (issuer, amount), by issuer, none zero; never the country's own currency.
    pub fx: Vec<(u16, Fixed)>,
}

impl Reserve {
    pub fn total(&self) -> Fixed {
        self.gold + self.fx.iter().map(|x| x.1).sum::<Fixed>()
    }

    fn add_fx(&mut self, issuer: u16, amount: Fixed) {
        match self.fx.binary_search_by_key(&issuer, |x| x.0) {
            Ok(k) => self.fx[k].1 += amount,
            Err(k) => self.fx.insert(k, (issuer, amount)),
        }
    }
}

impl HashState for Reserve {
    fn hash_state(&self, h: &mut StateHasher) {
        self.gold.hash_state(h);
        h.write_u64(self.fx.len() as u64);
        for &(k, v) in &self.fx {
            h.write_u16(k);
            v.hash_state(h);
        }
    }
}

/// What settlement did to one country over a period.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settled {
    /// The balance settled in reserves (`Lines::reserve_flow`): positive in.
    pub balance: Fixed,
    /// Money paid into the pool (positive) or out of it: goods and services (`Lines::trade`).
    pub pool: Fixed,
    /// New monetary gold from the country's mines.
    pub gold: Fixed,
    /// The part of a deficit the reserves could not pay: paid in the country's own currency,
    /// now held abroad.
    pub unpaid: Fixed,
}

impl std::ops::AddAssign for Settled {
    fn add_assign(&mut self, o: Self) {
        self.balance += o.balance;
        self.pool += o.pool;
        self.gold += o.gold;
        self.unpaid += o.unpaid;
    }
}

/// One country's settlement: the last economy week, the calendar year so far, the last year.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettledPeriod {
    pub week: Settled,
    pub year: Settled,
    pub last_year: Settled,
}

/// `GameWorld::reserves`: each country's reserves (state, in the hash once settlement has
/// run) and a record of what settlement moved (out of the hash). Empty while
/// `game::Switches::payments_settle` is off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservesState {
    /// By market index.
    pub markets: Vec<Reserve>,
    /// The calendar year `SettledPeriod::year` holds.
    pub year: i32,
    /// By market index; a record.
    pub record: Vec<SettledPeriod>,
    /// The world's surpluses less its deficits in the last week, which settlement leaves
    /// unsettled (zero while every line of the balance adds up to zero over the world).
    pub mismatch: Fixed,
}

impl ReservesState {
    pub fn is_empty(&self) -> bool {
        self.markets.is_empty()
    }

    /// What market `mi` owes abroad: its currency in the other countries' reserves.
    pub fn owed(&self, mi: usize) -> Fixed {
        let Ok(k) = u16::try_from(mi) else {
            return Fixed::ZERO;
        };
        self.markets
            .iter()
            .filter_map(|r| {
                r.fx.binary_search_by_key(&k, |x| x.0)
                    .ok()
                    .map(|i| r.fx[i].1)
            })
            .sum()
    }

    /// Reserves less what is owed abroad, by market.
    pub fn net(&self) -> Vec<Fixed> {
        let mut net: Vec<Fixed> = self.markets.iter().map(Reserve::total).collect();
        for r in &self.markets {
            for &(k, v) in &r.fx {
                if let Some(n) = net.get_mut(usize::from(k)) {
                    *n -= v;
                }
            }
        }
        net
    }

    fn fit(&mut self, n: usize, year: i32) {
        if self.markets.len() < n {
            self.markets.resize(n, Reserve::default());
        }
        if self.record.len() < n {
            self.record.resize(n, SettledPeriod::default());
        }
        if year > self.year {
            let gap = self.year != 0 && year > self.year + 1;
            for p in &mut self.record {
                p.last_year = if gap {
                    Settled::default()
                } else {
                    std::mem::take(&mut p.year)
                };
                p.year = Settled::default();
            }
            self.year = year;
        }
    }
}

impl HashState for ReservesState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_u64(self.markets.len() as u64);
        for r in &self.markets {
            r.hash_state(h);
        }
    }
}

/// What settlement reads besides the world (`data/economy/reserves.ron`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettleRules {
    /// The good whose output feeds the reserves, and its base price.
    pub gold: crate::econ::catalogue::GoodIx,
    pub gold_price: Fixed,
    /// Share of the mines' output that goes to the central bank.
    pub monetary_share: Fixed,
    /// Each market's monetary gold at the end of 1925, reference units: the reserves when
    /// settlement first runs.
    pub start: Vec<Fixed>,
}

/// The week's settlement (module doc): runs at the start of the economy week, before
/// `PaymentsState::begin_week`, over the lines of the week before and what other systems
/// recorded since, and the markets' gold output of that week (`markets` as that week left
/// them). Moves the reserves and returns what each pool receives (negative: pays).
pub fn settle(
    rules: &SettleRules,
    markets: &[super::Market],
    payments: &PaymentsState,
    reserves: &mut ReservesState,
    year: i32,
) -> Vec<Fixed> {
    let n = markets.len();
    if reserves.is_empty() {
        reserves.markets = rules
            .start
            .iter()
            .map(|&gold| Reserve {
                gold,
                fx: Vec::new(),
            })
            .collect();
    }
    reserves.fit(n, year);
    let mut week = vec![Settled::default(); n];
    // New gold: bought from the mines by the central bank, paid out of the pool the mines'
    // money goes to, so the pool is unchanged.
    let g = usize::from(rules.gold);
    for (mi, m) in markets.iter().enumerate() {
        let made = (m.supply.get(g).copied().unwrap_or(Fixed::ZERO)
            - m.net_import
                .get(g)
                .copied()
                .unwrap_or(Fixed::ZERO)
                .max(Fixed::ZERO))
        .max(Fixed::ZERO);
        let gold = made * rules.monetary_share * rules.gold_price;
        reserves.markets[mi].gold += gold;
        week[mi].gold = gold;
    }
    // Goods and services move between the pools.
    let mut pools = vec![Fixed::ZERO; n];
    for (mi, w) in week.iter_mut().enumerate() {
        let Some(l) = payments.markets.get(mi).map(|p| p.week) else {
            continue;
        };
        pools[mi] = l.trade();
        w.pool = l.trade();
        w.balance = l.reserve_flow();
    }
    // The balance in reserves: the deficits pay into one bundle (gold and currencies), which
    // the surpluses share.
    let mut deficit: Vec<Fixed> = week.iter().map(|w| (-w.balance).max(Fixed::ZERO)).collect();
    let mut surplus: Vec<Fixed> = week.iter().map(|w| w.balance.max(Fixed::ZERO)).collect();
    let (d, s) = (
        deficit.iter().copied().sum::<Fixed>(),
        surplus.iter().copied().sum::<Fixed>(),
    );
    reserves.mismatch = s - d;
    let settled = d.min(s);
    let mut rest = Vec::new();
    for (side, total) in [(&mut deficit, d), (&mut surplus, s)] {
        if total > settled {
            let weights = side.clone();
            super::trade::split_exact_into(settled, &weights, side, &mut rest);
        }
    }
    // The bundle: gold (`None`) and each currency (`Some(issuer)`).
    let mut bundle: std::collections::BTreeMap<Option<u16>, Fixed> = Default::default();
    let mut weights = Vec::new();
    let mut parts = Vec::new();
    for (mi, &owe) in deficit.iter().enumerate() {
        if owe <= Fixed::ZERO {
            continue;
        }
        let r = &mut reserves.markets[mi];
        let pay = owe.min(r.total().max(Fixed::ZERO));
        if pay > Fixed::ZERO {
            weights.clear();
            weights.push(r.gold);
            weights.extend(r.fx.iter().map(|x| x.1));
            parts.clear();
            parts.resize(weights.len(), Fixed::ZERO);
            super::trade::split_exact_into(pay, &weights, &mut parts, &mut rest);
            r.gold -= parts[0];
            *bundle.entry(None).or_default() += parts[0];
            for (x, &p) in r.fx.iter_mut().zip(&parts[1..]) {
                x.1 -= p;
                *bundle.entry(Some(x.0)).or_default() += p;
            }
            r.fx.retain(|x| x.1 != Fixed::ZERO);
        }
        let unpaid = owe - pay;
        if unpaid > Fixed::ZERO {
            *bundle.entry(Some(mi as u16)).or_default() += unpaid;
            week[mi].unpaid = unpaid;
        }
    }
    // Each component goes to the surpluses in proportion to what each still has to receive;
    // the last one fills them exactly, so each receives its surplus to the last unit.
    let last = bundle.len().saturating_sub(1);
    for (i, (&c, &amount)) in bundle.iter().enumerate() {
        if i == last {
            parts.clone_from(&surplus);
        } else {
            parts.clear();
            parts.resize(n, Fixed::ZERO);
            super::trade::split_exact_into(amount, &surplus, &mut parts, &mut rest);
        }
        for (j, &p) in parts.iter().enumerate() {
            if p <= Fixed::ZERO {
                continue;
            }
            surplus[j] -= p;
            match c {
                None => reserves.markets[j].gold += p,
                // A country's own currency back home is no reserve: what it owed is paid.
                Some(k) if usize::from(k) == j => {}
                Some(k) => reserves.markets[j].add_fx(k, p),
            }
        }
    }
    debug_assert!(surplus.iter().all(|&x| x == Fixed::ZERO) || bundle.is_empty());
    for (p, w) in reserves.record.iter_mut().zip(week) {
        p.week = w;
        p.year += w;
    }
    pools
}

/// What the screen shows of a country's balance of payments: the last full year's lines
/// (Government → Money and debt).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaymentsView {
    pub goods: Fixed,
    pub services: Fixed,
    pub interest: Fixed,
    pub ledger_interest: Fixed,
    pub remittances: Fixed,
    pub transfers: Fixed,
    pub current_account: Fixed,
    pub bonds: Fixed,
    pub principal: Fixed,
    pub ledger_principal: Fixed,
    pub direct_investment: Fixed,
    /// Income of private claims, net, and private capital, long and short, net inflow
    /// (`econ::capital`).
    pub capital_income: Fixed,
    pub capital_long: Fixed,
    pub capital_short: Fixed,
    pub financial_account: Fixed,
    pub not_settled: Fixed,
    /// The year these lines are of; 0 before a full year has passed.
    pub year: i32,
    /// The country's GDP of that year, to read each line as a share of it.
    pub gdp: Fixed,
}

/// The last full year's balance of `market`.
pub fn view(world: &crate::game::GameWorld, market: usize) -> PaymentsView {
    let p = &world.payments;
    let Some(l) = p.markets.get(market).map(|x| x.last_year) else {
        return PaymentsView::default();
    };
    PaymentsView {
        goods: l.goods(),
        services: l.services(),
        interest: l.interest(),
        ledger_interest: l.ledger_interest(),
        remittances: l.remittances(),
        transfers: l.transfers(),
        current_account: l.current_account(),
        bonds: l.bonds_sold - l.bonds_bought,
        principal: l.principal_received - l.principal_repaid,
        ledger_principal: l.ledger_principal_received - l.ledger_principal_repaid,
        direct_investment: l.fdi_in - l.fdi_out,
        capital_income: l.capital_income(),
        capital_long: l.capital_long(),
        capital_short: l.capital_short(),
        financial_account: l.financial_account(),
        not_settled: l.not_settled(),
        year: if p.year > 0 { p.year - 1 } else { 0 },
        gdp: world
            .econ
            .markets
            .get(market)
            .map_or(Fixed::ZERO, |m| m.gdp_last_year),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exports(v: i64) -> Lines {
        Lines {
            exports: Fixed::from_int(v),
            ..Lines::default()
        }
    }

    #[test]
    fn the_year_rolls_in_the_first_week_of_january() {
        let mut p = PaymentsState::default();
        p.begin_week(2, 1925);
        p.add(0, 1925, exports(3));
        p.begin_week(2, 1925);
        p.add(0, 1925, exports(4));
        assert_eq!(p.markets[0].week, exports(4));
        assert_eq!(p.markets[0].year, exports(7));
        assert_eq!(p.markets[0].last_year, Lines::default());
        // A treaty's payment on the first of January, before the week: the new year already.
        p.transfer(1, 0, 1926, Fixed::from_int(2));
        assert_eq!(p.markets[0].last_year, exports(7));
        assert_eq!(p.markets[0].year.transfers_received, Fixed::from_int(2));
        p.begin_week(2, 1926);
        p.add(0, 1926, exports(5));
        assert_eq!(p.markets[0].year.exports, Fixed::from_int(5));
        assert_eq!(p.markets[0].last_year, exports(7));
        assert_eq!(p.markets[1].last_year, Lines::default());
        assert_eq!(p.markets[1].year.transfers_paid, Fixed::from_int(2));
    }

    #[test]
    fn a_year_without_records_leaves_an_empty_last_year() {
        let mut p = PaymentsState::default();
        p.add(0, 1925, exports(3));
        p.add(0, 1927, exports(1));
        assert_eq!(p.markets[0].last_year, Lines::default());
        assert_eq!(p.markets[0].year, exports(1));
    }

    #[test]
    fn the_market_grows_with_a_record() {
        let mut p = PaymentsState::default();
        p.add(3, 1925, exports(1));
        assert_eq!(p.markets.len(), 4);
        assert_eq!(p.markets[3].year, exports(1));
    }

    fn rules(start: &[i64]) -> SettleRules {
        SettleRules {
            gold: 0,
            gold_price: Fixed::from_int(10),
            monetary_share: Fixed::from_int(1) / 2,
            start: start.iter().map(|&x| Fixed::from_int(x)).collect(),
        }
    }

    /// Market `s` sold `v` of goods to market `b` last week.
    fn sale(n: usize, s: usize, b: usize, v: i64) -> PaymentsState {
        let mut p = PaymentsState::default();
        p.begin_week(n, 1926);
        let mut value = vec![Fixed::ZERO; n * n];
        value[s * n + b] = Fixed::from_int(v);
        p.record_trade(1926, n, &value, None);
        p
    }

    #[test]
    fn a_deficit_pays_from_its_reserves_then_in_its_own_currency() {
        let markets = vec![super::super::Market::empty(1); 3];
        let r = rules(&[100, 30, 0]);
        let mut res = ReservesState::default();
        // Market 1 buys 50 from market 0: it has 30 of gold, the rest it owes.
        let pools = settle(&r, &markets, &sale(3, 0, 1, 50), &mut res, 1926);
        assert_eq!(pools, [50, -50, 0].map(Fixed::from_int));
        assert_eq!(res.markets[1].total(), Fixed::ZERO);
        assert_eq!(res.markets[0].gold, Fixed::from_int(130));
        assert_eq!(res.markets[0].fx, [(1, Fixed::from_int(20))]);
        assert_eq!(res.owed(1), Fixed::from_int(20));
        assert_eq!(res.record[1].week.unpaid, Fixed::from_int(20));
        assert_eq!(res.net(), [150, -20, 0].map(Fixed::from_int));
        // Market 1 sells 25 back: market 0 pays from its reserves pro rata, in gold and in
        // market 1's own currency, which comes home and is owed no more.
        let pools = settle(&r, &markets, &sale(3, 1, 0, 25), &mut res, 1926);
        assert_eq!(pools, [-25, 25, 0].map(Fixed::from_int));
        assert_eq!(res.net(), [125, 5, 0].map(Fixed::from_int));
        assert!(res.markets.iter().all(|m| m.gold >= Fixed::ZERO));
        assert_eq!(
            res.net().iter().copied().sum::<Fixed>(),
            Fixed::from_int(130)
        );
    }

    #[test]
    fn the_mines_add_gold_without_money() {
        let mut markets = vec![super::super::Market::empty(1); 2];
        markets[1].supply[0] = Fixed::from_int(4);
        let mut res = ReservesState::default();
        let pools = settle(
            &rules(&[0, 0]),
            &markets,
            &PaymentsState::default(),
            &mut res,
            1926,
        );
        assert_eq!(pools, [Fixed::ZERO; 2]);
        // Half of 4 kg at 10.
        assert_eq!(res.markets[1].gold, Fixed::from_int(20));
        assert_eq!(res.record[1].week.gold, Fixed::from_int(20));
    }

    #[test]
    fn not_settled_closes_the_balance() {
        let l = Lines {
            exports: Fixed::from_int(10),
            imports: Fixed::from_int(4),
            interest_paid: Fixed::from_int(1),
            bonds_sold: Fixed::from_int(2),
            ..Lines::default()
        };
        assert_eq!(l.current_account(), Fixed::from_int(5));
        assert_eq!(l.financial_account(), Fixed::from_int(2));
        assert_eq!(l.not_settled(), Fixed::from_int(-7));
    }
}
