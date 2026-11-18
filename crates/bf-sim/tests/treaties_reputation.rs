//! The reputation of breaches (`docs/treaties.pt-BR.md` 9.4, owner's decisions P1–P3 of
//! 2026-11-18): two kinds, a tenth forgotten a year, a change of regime halving the security
//! one. Behind `Switches::reputation`.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::polity::{Enactor, LawChangeRecord};
use bf_sim::treaty::{
    Agreement, AgreementStatus, BreachCause, Clause, ClauseKind, Side, TreatyState,
};
use bf_sim::treaty_system::{TreatySetup, apply_month, month};
use laplace_schelling::{Date, Fixed};

fn mutual(kind: ClauseKind) -> Clause {
    Clause {
        kind,
        giver: Side::All,
        receiver: Side::All,
        until: None,
        note: None,
    }
}

fn pact(t: &mut TreatyState) -> u32 {
    t.sign(Agreement {
        id: 0,
        key: "pact".into(),
        side_a: vec!["FRA".into(), "POL".into()],
        side_b: Vec::new(),
        signed: 1925,
        term_years: None,
        notice_years: 0,
        kept: vec![
            mutual(ClauseKind::Defence),
            mutual(ClauseKind::Tribute {
                yearly: Fixed::from_int(12),
            }),
        ],
        status: AgreementStatus::InForce,
    })
}

#[test]
fn a_breach_weighs_on_the_reputation_of_its_kind_only_when_kept() {
    let date = Date::new(1938, 9, 30);
    let v = Fixed::from_int(1000);
    let mut off = TreatyState::default();
    let a = pact(&mut off);
    off.record_breach(a, 0, "FRA", v, date, BreachCause::CallUnanswered { war: 1 })
        .unwrap();
    assert!(
        off.reputation.is_empty(),
        "off: breaches give causes, nothing more"
    );
    let mut t = TreatyState {
        reputation_on: true,
        ..Default::default()
    };
    let a = pact(&mut t);
    // France does not honour Czechoslovakia's call (1938): a security breach.
    t.record_breach(a, 0, "FRA", v, date, BreachCause::CallUnanswered { war: 1 })
        .unwrap();
    // An unpaid tribute: a general one.
    t.record_breach(
        a,
        1,
        "FRA",
        Fixed::from_int(10),
        date,
        BreachCause::ServiceUnpaid,
    )
    .unwrap();
    let r = t.reputation["FRA"];
    assert_eq!((r.security, r.general), (v, Fixed::from_int(10)));
    // A tenth a year is forgotten; a change of regime halves the security reputation only.
    t.decay_reputation(Fixed::from_ratio(1, 10));
    assert_eq!(t.reputation["FRA"].security, Fixed::from_int(900));
    t.new_regime("FRA");
    assert_eq!(t.reputation["FRA"].security, Fixed::from_int(450));
    assert_eq!(t.reputation["FRA"].general, Fixed::from_int(9));
}

/// The monthly step finds the change of a form of government since its last run and forgets a
/// tenth in January; without the reputation kept it does neither.
#[test]
fn the_month_finds_a_new_regime_and_forgets_in_january() {
    let d = GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
    let p = d.politics.as_ref().unwrap();
    let g = p.group("government").unwrap();
    let laws = p.groups[g].laws.clone();
    let setup = TreatySetup {
        decay: Fixed::from_ratio(1, 10),
        government: laws.clone(),
    };
    let mut w = d.new_world();
    let ita = w
        .countries
        .iter()
        .position(|(_, c)| c.tag == "ITA")
        .unwrap() as u16;
    w.politics.changes.push(LawChangeRecord {
        date: Date::new(1926, 3, 20),
        country: ita,
        from: laws[0],
        to: laws[1],
        by: Enactor::Groups,
    });
    // Not kept: nothing.
    let m = month(&w, &setup, false, Date::new(1926, 4, 2));
    assert!(m.new_regimes.is_empty() && m.decay.is_none());
    w.treaties.reputation_on = true;
    w.treaties.reputation.insert(
        "ITA".into(),
        bf_sim::treaty::Reputation {
            general: Fixed::ZERO,
            security: Fixed::from_int(100),
        },
    );
    let m = month(&w, &setup, false, Date::new(1926, 4, 2));
    assert_eq!(m.new_regimes, ["ITA"]);
    assert!(
        m.expire.is_none() && m.payments.is_empty(),
        "tests off: no tribute"
    );
    apply_month(&mut w, m, Date::new(1926, 4, 2));
    assert_eq!(w.treaties.reputation["ITA"].security, Fixed::from_int(50));
    // The next month the same change is not counted again.
    assert!(
        month(&w, &setup, false, Date::new(1926, 5, 2))
            .new_regimes
            .is_empty()
    );
    let m = month(&w, &setup, false, Date::new(1927, 1, 2));
    assert_eq!(m.decay, Some(Fixed::from_ratio(1, 10)));
}
