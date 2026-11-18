//! The monthly step of the agreements (`treaty_system`): a fixed term runs out in January,
//! tribute is paid from treasury to treasury and a treasury that cannot pay is a breach, once a
//! year. Behind `Switches::clause_tests`.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameWorld, Session};
use bf_sim::treaty::{Agreement, AgreementStatus, BreachCause, Clause, ClauseKind, Side};
use bf_sim::treaty_system::{TreatySetup, apply_month, month};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static D: OnceLock<GameData> = OnceLock::new();
    D.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn setup() -> TreatySetup {
    TreatySetup {
        decay: Fixed::from_ratio(1, 10),
        government: Vec::new(),
        clauses: None,
    }
}

fn ix(w: &GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
}

fn tribute(w: &mut GameWorld, yearly: i64, term: Option<u16>) -> u32 {
    w.treaties.sign(Agreement {
        id: 0,
        key: "tribute".into(),
        side_a: vec!["GBR".into()],
        side_b: vec!["EGY".into()],
        signed: 1925,
        term_years: term,
        notice_years: 0,
        kept: vec![Clause {
            kind: ClauseKind::Tribute {
                yearly: Fixed::from_int(yearly),
            },
            giver: Side::Junior,
            receiver: Side::Senior,
            until: None,
            note: None,
        }],
        status: AgreementStatus::InForce,
    })
}

fn money(w: &GameWorld) -> Fixed {
    w.econ.markets.iter().map(|m| m.treasury).sum()
}

#[test]
fn tribute_moves_a_twelfth_from_treasury_to_treasury() {
    let mut w = data().new_world();
    tribute(&mut w, 1200, None);
    let (egy, gbr) = (ix(&w, "EGY"), ix(&w, "GBR"));
    w.econ.markets[egy].treasury = Fixed::from_int(10_000);
    let before = money(&w);
    let (e0, g0) = (w.econ.markets[egy].treasury, w.econ.markets[gbr].treasury);
    let m = month(&w, &setup(), true, Date::new(1925, 3, 2));
    assert_eq!(m.payments, [(egy as u16, gbr as u16, Fixed::from_int(100))]);
    assert!(m.unpaid.is_empty() && m.expire.is_none());
    apply_month(&mut w, m, Date::new(1925, 3, 2));
    assert_eq!(w.econ.markets[egy].treasury, e0 - Fixed::from_int(100));
    assert_eq!(w.econ.markets[gbr].treasury, g0 + Fixed::from_int(100));
    assert_eq!(money(&w), before);
    assert!(w.treaties.breaches.is_empty());
}

#[test]
fn a_tribute_not_paid_is_a_breach_once_a_year() {
    let mut w = data().new_world();
    let a = tribute(&mut w, 1200, None);
    let egy = ix(&w, "EGY");
    w.econ.markets[egy].treasury = Fixed::from_int(50);
    let before = money(&w);
    for m in 1..=3u8 {
        let mo = month(&w, &setup(), true, Date::new(1930, m, 2));
        assert!(mo.payments.is_empty());
        apply_month(&mut w, mo, Date::new(1930, m, 2));
    }
    assert_eq!(money(&w), before);
    assert_eq!(w.treaties.breaches.len(), 1);
    let b = &w.treaties.breaches[0];
    assert_eq!(
        (b.agreement, b.by.as_str(), b.cause),
        (a, "EGY", BreachCause::ServiceUnpaid)
    );
    assert_eq!(w.treaties.cause("GBR", "EGY"), Fixed::from_int(100));
    // A new year, a new breach.
    let mo = month(&w, &setup(), true, Date::new(1931, 1, 2));
    apply_month(&mut w, mo, Date::new(1931, 1, 2));
    assert_eq!(w.treaties.breaches.len(), 2);
}

#[test]
fn a_fixed_term_runs_out_in_january() {
    let mut w = data().new_world();
    tribute(&mut w, 0, Some(10));
    tribute(&mut w, 0, None);
    let mo = month(&w, &setup(), true, Date::new(1934, 12, 2));
    assert_eq!(mo.expire, None);
    apply_month(&mut w, mo, Date::new(1934, 12, 2));
    assert_eq!(w.treaties.agreements.len(), 2);
    let mo = month(&w, &setup(), true, Date::new(1935, 1, 2));
    assert_eq!(mo.expire, Some(1935));
    apply_month(&mut w, mo, Date::new(1935, 1, 2));
    assert_eq!(w.treaties.agreements.len(), 1);
    assert_eq!(
        w.treaties.ended[0].status,
        AgreementStatus::Ended { year: 1935 }
    );
}

/// Off, the system is not in the schedule: a world with a tribute pays nothing.
#[test]
fn nothing_runs_without_the_switch() {
    let d = data();
    assert!(!d.switches.clause_tests);
    let mut w = d.new_world();
    tribute(&mut w, 1200, None);
    let egy = ix(&w, "EGY");
    let run = |w: GameWorld, rules| {
        let mut s = Session::new(w, 1925, Date::new(1925, 1, 1), 2, Some(rules));
        s.sim.run_until(Date::new(1925, 2, 10));
        s.world().clone()
    };
    let off = run(w.clone(), d.rules());
    let mut on = d.switches;
    on.switch_on("clause_tests").unwrap();
    let on = run(w, d.rules_with(on));
    // Two months: the 2nd of January and of February paid a hundred each. The money step keeps
    // the treasury at its reserve, so what Egypt paid shows in its debt, not in its treasury.
    assert_ne!(
        laplace_schelling::hash::hash_of(&off),
        laplace_schelling::hash::hash_of(&on)
    );
    let debt = |x: &GameWorld| x.econ.markets[egy].money.total_debt();
    assert!(debt(&on) > debt(&off), "{:?} {:?}", debt(&on), debt(&off));
    assert!(on.treaties.breaches.is_empty());
}
