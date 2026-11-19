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

use laplace_schelling::Fixed;
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

    /// Primary income: interest of bonds and of the ledger, and remittances.
    pub fn income(&self) -> Fixed {
        self.interest() + self.ledger_interest() + self.remittances()
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
    }

    /// What would have to move for the balance to close: `−(current + financial)`.
    pub fn not_settled(&self) -> Fixed {
        -(self.current_account() + self.financial_account())
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
    /// What debtors paid on bonds held abroad that no lender received, as a lender would have
    /// (`interest_received`, `principal_received`); negative where lenders received more. The
    /// lenders' share is rounded to a thousandth (`money::week`), and a week with no claims
    /// abroad pays nobody: with it, every line adds up to zero over the world.
    pub unreceived: Period,
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
