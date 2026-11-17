//! What the interstate debts tab reads: the player's lines of the ledger of debts between
//! governments (`treaty::Ledger`), owed and owed to, copied out of the simulation once a month.

use bf_sim::game::GameWorld;
use bf_sim::treaty::{Currency, InterstateDebt, Obligation, Pledge};
use laplace_schelling::calendar::Date;

/// Where a line stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Paid as due.
    Current,
    /// Service due and not paid.
    Arrears,
    /// Suspended by a moratorium.
    Moratorium,
    /// Repudiated by the debtor: written off at once, kept for the record.
    Broken,
}

impl Standing {
    pub fn key(self) -> &'static str {
        match self {
            Standing::Current => "ledger-standing-current",
            Standing::Arrears => "ledger-standing-arrears",
            Standing::Moratorium => "ledger-standing-moratorium",
            Standing::Broken => "ledger-standing-broken",
        }
    }
}

/// One debt, from the player's side.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub id: u32,
    /// Tag of the other government.
    pub other: String,
    /// The player owes it (else the other owes the player).
    pub owed_by_us: bool,
    /// Key of the treaty that made or last revised it.
    pub treaty: String,
    pub currency: Currency,
    /// Outstanding principal, reference units.
    pub principal: f64,
    pub rate: f64,
    pub annuity: f64,
    pub years_left: u16,
    pub deferrable: f64,
    pub transfer_cap: Option<f64>,
    pub pledge: Option<Pledge>,
    pub contingent: f64,
    pub arrears: f64,
    pub suspended_until: i32,
    pub standing: Standing,
    /// Service paid since it was signed (since the start date for the lines of 1925),
    /// reference units; `None` where the ledger does not keep it.
    pub paid: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LedgerView {
    pub month: (i32, u8),
    pub tag: String,
    /// Owed by the player, then owed to it; largest principal first.
    pub lines: Vec<Line>,
}

impl LedgerView {
    pub fn owed(&self, by_us: bool) -> impl Iterator<Item = &Line> {
        self.lines.iter().filter(move |l| l.owed_by_us == by_us)
    }
}

fn standing(d: &InterstateDebt, year: i32) -> Standing {
    if matches!(d.status, Obligation::Repudiated { .. }) {
        Standing::Broken
    } else if d.suspended_until >= year {
        Standing::Moratorium
    } else if d.arrears > laplace_schelling::Fixed::ZERO {
        Standing::Arrears
    } else {
        Standing::Current
    }
}

/// Reads the lines of country `tag`; `None` without a player or an economy.
pub fn read(world: &GameWorld, tag: Option<&str>, date: Date) -> Option<LedgerView> {
    let tag = tag?;
    let c = world.countries.iter().position(|(_, x)| x.tag == tag)?;
    if world.econ.markets.is_empty() {
        return None;
    }
    let tag_of = |m: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(m))
            .map_or_else(|| "?".into(), |(_, x)| x.tag.clone())
    };
    let f = |x: laplace_schelling::Fixed| x.to_f64_lossy();
    let mut lines: Vec<Line> = world
        .econ
        .ledger
        .debts
        .iter()
        // A line paid off leaves the tab; a repudiated one stays, as broken.
        .chain(
            world
                .econ
                .ledger
                .closed
                .iter()
                .filter(|d| matches!(d.status, Obligation::Repudiated { .. })),
        )
        .filter(|d| usize::from(d.debtor) == c || usize::from(d.creditor) == c)
        .map(|d| {
            let owed_by_us = usize::from(d.debtor) == c;
            Line {
                id: d.id,
                other: tag_of(if owed_by_us { d.creditor } else { d.debtor }),
                owed_by_us,
                treaty: d.treaty.clone(),
                currency: d.currency,
                principal: f(d.principal),
                rate: f(d.rate),
                annuity: f(d.annuity),
                years_left: d.years_left,
                deferrable: f(d.deferrable),
                transfer_cap: d.transfer_cap.map(f),
                pledge: d.pledge,
                contingent: f(d.contingent),
                arrears: f(d.arrears),
                suspended_until: d.suspended_until,
                standing: standing(d, date.year),
                paid: world.econ.ledger.record.then(|| f(d.paid)),
            }
        })
        .collect();
    lines.sort_by(|a, b| {
        b.owed_by_us
            .cmp(&a.owed_by_us)
            .then(b.principal.total_cmp(&a.principal))
            .then(a.id.cmp(&b.id))
    });
    Some(LedgerView {
        month: (date.year, date.month),
        tag: tag.to_owned(),
        lines,
    })
}

/// The principal each 1925 line was signed with, by (treaty, debtor, creditor), reference
/// units: the ledger keeps only what is left, so the amortized part is this minus the balance.
/// Lines made later in the game are not here.
pub type Signed = std::collections::HashMap<(String, String, String), f64>;

pub fn signed_1925(data: &bf_sim::data::GameData) -> Signed {
    let (Some(cat), Some(debts)) = (&data.clauses, &data.war_debts) else {
        return Signed::new();
    };
    let ledger = bf_sim::treaty::ledger_1925(debts, &cat.policy, &data.countries);
    let tag = |i: u16| {
        data.countries
            .countries
            .get(usize::from(i))
            .map_or_else(String::new, |c| c.tag.clone())
    };
    ledger
        .debts
        .iter()
        .map(|d| {
            (
                (d.treaty.clone(), tag(d.debtor), tag(d.creditor)),
                d.principal.to_f64_lossy(),
            )
        })
        .collect()
}
