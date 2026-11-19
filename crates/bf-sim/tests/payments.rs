//! The balance of payments (`econ::payments`) is a record of what crossed borders: every week,
//! each line adds up to zero over the world (what one country exports another imports, what one
//! pays another receives), and the world's hash leaves the record out.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::payments::Lines;
use bf_sim::game::{Session, Switches};
use laplace_schelling::hash::hash_of;
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

/// The world's sum of each pair of lines that must cancel, as (name, credits − debits).
fn world_balance(markets: &[Lines], unreceived: &Lines) -> Vec<(&'static str, Fixed)> {
    let sum = |f: &dyn Fn(&Lines) -> Fixed| markets.iter().map(f).sum::<Fixed>();
    vec![
        ("goods", sum(&|l| l.exports - l.imports)),
        (
            "services",
            sum(&|l| l.services_exports - l.services_imports),
        ),
        (
            "interest",
            sum(&|l| l.interest_received - l.interest_paid) + unreceived.interest_received,
        ),
        (
            "ledger interest",
            sum(&|l| l.ledger_interest_received - l.ledger_interest_paid),
        ),
        (
            "remittances",
            sum(&|l| l.remittances_received - l.remittances_paid),
        ),
        ("margins", sum(&|l| l.margins)),
        (
            "transfers",
            sum(&|l| l.transfers_received - l.transfers_paid),
        ),
        ("bonds", sum(&|l| l.bonds_sold - l.bonds_bought)),
        (
            "principal",
            sum(&|l| l.principal_received - l.principal_repaid) + unreceived.principal_received,
        ),
        (
            "ledger principal",
            sum(&|l| l.ledger_principal_received - l.ledger_principal_repaid),
        ),
        ("direct investment", sum(&|l| l.fdi_in - l.fdi_out)),
    ]
}

fn weeks_of_payments(switches: Switches, weeks: usize) {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(switches)),
    );
    let mut date = Date::new(1925, 1, 5);
    let mut moved = Lines::default();
    let mut unreceived = Fixed::ZERO;
    for _ in 0..weeks {
        date = Date::from_days(date.to_days() + 7);
        s.sim.run_until(date);
        let p = &s.world().payments;
        let week: Vec<Lines> = p.markets.iter().map(|m| m.week).collect();
        // One raw unit per market: the lenders' shares of what was lent are rounded once each.
        let tolerance = Fixed::from_raw(week.len() as i64);
        for (line, total) in world_balance(&week, &p.unreceived.week) {
            assert!(
                total.abs() <= tolerance,
                "week of {date}: {line} adds up to {total} over the world"
            );
        }
        for l in &week {
            moved += *l;
        }
        unreceived +=
            p.unreceived.week.interest_received.abs() + p.unreceived.week.principal_received.abs();
        // The year so far adds up too.
        let year: Vec<Lines> = p.markets.iter().map(|m| m.year).collect();
        let tolerance = Fixed::from_raw(year.len() as i64 * 53);
        for (line, total) in world_balance(&year, &p.unreceived.year) {
            assert!(
                total.abs() <= tolerance,
                "year to {date}: {line} adds up to {total} over the world"
            );
        }
    }
    assert!(moved.exports > Fixed::ZERO, "goods crossed borders");
    assert!(moved.interest_paid > Fixed::ZERO, "interest went abroad");
    assert!(moved.bonds_sold > Fixed::ZERO, "bonds were sold abroad");
    eprintln!(
        "{weeks} weeks: exports {}, interest paid abroad {}, received by no lender {}",
        moved.exports, moved.interest_paid, unreceived
    );
    // The record is out of the hash: the same world without it hashes the same.
    let mut bare = s.world().clone();
    bare.payments = Default::default();
    assert_eq!(hash_of(&bare), hash_of(s.world()));
    assert_eq!(hash_of(s.world()), s.state_hash());
}

#[test]
fn the_shipped_rules_balance_every_week() {
    weeks_of_payments(Switches::shipped(), 16);
}

#[test]
fn services_remittances_and_the_ledger_balance_every_week() {
    let mut switches = Switches::shipped();
    for name in ["union_services", "foreign_flow", "ledger_record"] {
        switches.switch_on(name).unwrap();
    }
    weeks_of_payments(switches, 16);
}
