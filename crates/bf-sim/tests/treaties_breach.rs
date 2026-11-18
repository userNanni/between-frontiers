//! Agreements in force and the record of breaches (`docs/treaties.pt-BR.md`, section 9): the
//! treaties of 1925 as agreements behind their switch, the duties a war asks of third parties,
//! a breach and the cause it gives, the suspension in return, and the save.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::treaty::{
    Agreement, AgreementStatus, BreachCause, Clause, ClauseKind, DutyKind, Side, TreatyState,
};
use laplace_schelling::{Date, Fixed};

fn load(on: bool) -> GameData {
    let mut d = GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
    if on {
        d.switches.switch_on("treaties_in_force").unwrap();
    }
    d
}

fn mutual(kind: ClauseKind) -> Clause {
    Clause {
        kind,
        giver: Side::All,
        receiver: Side::All,
        until: None,
        note: None,
    }
}

fn pact(t: &mut TreatyState, key: &str, parties: &[&str], kept: Vec<Clause>) -> u32 {
    t.sign(Agreement {
        id: 0,
        key: key.into(),
        side_a: parties.iter().map(|p| p.to_string()).collect(),
        side_b: Vec::new(),
        signed: 1939,
        term_years: None,
        notice_years: 0,
        kept,
        status: AgreementStatus::InForce,
    })
}

#[test]
fn the_treaties_of_1925_are_agreements_only_with_their_switch() {
    let mut off = load(false);
    // Shipped on; switched off, only the agreements signed in play are kept.
    assert!(off.switches.treaties_in_force);
    off.switches.treaties_in_force = false;
    assert!(off.new_world().treaties.agreements.is_empty());
    let on = load(true);
    let w = on.new_world();
    let t = &w.treaties;
    assert!(!t.agreements.is_empty());
    let cat = on.clauses.as_ref().unwrap();
    for a in &t.agreements {
        assert!(!a.kept.is_empty(), "{}", a.key);
        for c in &a.kept {
            assert!(
                !cat.get(c.kind.key()).unwrap().once,
                "{}: {}",
                a.key,
                c.kind.key()
            );
        }
    }
    // In force on the start date: the alliance of 1921, not yet Locarno (in force in 1926).
    assert!(
        t.agreements
            .iter()
            .any(|a| a.key == "franco_polish_alliance")
    );
    assert!(
        !t.agreements
            .iter()
            .any(|a| a.key == "locarno_rhineland_pact")
    );
    // Germany attacks Poland: France is called to Poland's side.
    let duties = t.duties("GER", "POL");
    assert!(
        duties
            .iter()
            .any(|d| d.party == "FRA" && d.kind == DutyKind::JoinDefender),
        "{duties:?}"
    );
    // Saved and loaded, the same world.
    let bytes = bf_sim::save_format::encode_world(&w, None).unwrap();
    assert_eq!(bf_sim::save_format::decode_world(&bytes).unwrap(), w);
}

/// Italy in 1914: the Triple Alliance was defensive and Austria-Hungary declared first, so it
/// called nobody. The Pact of Steel of 1939 (art. III, offence and defence) calls in any war.
#[test]
fn a_defensive_pact_does_not_call_into_the_partners_own_war() {
    let mut t = TreatyState::default();
    pact(
        &mut t,
        "triple",
        &["AUH", "GER", "ITA"],
        vec![mutual(ClauseKind::Defence)],
    );
    let d = t.duties("AUH", "SER");
    assert!(d.is_empty(), "{d:?}");
    // Attacked, it calls the two others.
    let d = t.duties("FRA", "GER");
    let called: Vec<_> = d.iter().map(|d| (d.party.as_str(), d.kind)).collect();
    assert_eq!(
        called,
        [
            ("AUH", DutyKind::JoinDefender),
            ("ITA", DutyKind::JoinDefender)
        ]
    );
    let mut t = TreatyState::default();
    pact(
        &mut t,
        "steel",
        &["GER", "ITA"],
        vec![
            mutual(ClauseKind::Consultation),
            mutual(ClauseKind::Offence {
                against: Vec::new(),
            }),
            mutual(ClauseKind::Defence),
        ],
    );
    let d = t.duties("GER", "POL");
    assert_eq!(
        d.iter()
            .map(|d| (d.party.as_str(), d.kind))
            .collect::<Vec<_>>(),
        [("ITA", DutyKind::JoinAttacker)]
    );
    // Non-aggression is the attacker's own duty; neutrality keeps the third party out.
    let mut t = TreatyState::default();
    pact(
        &mut t,
        "na",
        &["GER", "SOV"],
        vec![mutual(ClauseKind::NonAggression)],
    );
    pact(
        &mut t,
        "neutral",
        &["SOV", "JAP"],
        vec![mutual(ClauseKind::Neutrality)],
    );
    let d = t.duties("GER", "SOV");
    assert_eq!(
        d.iter()
            .map(|d| (d.party.as_str(), d.kind))
            .collect::<Vec<_>>(),
        [("GER", DutyKind::NotAttack), ("JAP", DutyKind::StayOut)]
    );
}

/// The Pact of Steel in 1939: Germany broke the consultation first; Italy's defence and offence
/// are then suspended, and Italy holds a cause against Germany.
#[test]
fn a_breach_gives_a_cause_and_suspends_what_was_given_in_return() {
    let mut t = TreatyState::default();
    let id = pact(
        &mut t,
        "steel",
        &["GER", "ITA"],
        vec![
            mutual(ClauseKind::Consultation),
            mutual(ClauseKind::Offence {
                against: Vec::new(),
            }),
        ],
    );
    assert!(!t.suspended(id, 1, "ITA"));
    let date = Date::new(1939, 8, 22);
    let value = Fixed::from_int(1000);
    // Only a giver of the clause breaks it; a clause or an agreement that does not exist is
    // nothing.
    assert_eq!(
        t.record_breach(id, 0, "FRA", value, date, BreachCause::TestFailed),
        None
    );
    assert_eq!(
        t.record_breach(id, 9, "GER", value, date, BreachCause::TestFailed),
        None
    );
    assert_eq!(
        t.record_breach(id + 1, 0, "GER", value, date, BreachCause::TestFailed),
        None
    );
    assert!(t.breaches.is_empty());
    let b = t
        .record_breach(id, 0, "GER", value, date, BreachCause::TestFailed)
        .unwrap();
    assert_eq!(t.breaches[0].id, b);
    assert_eq!(t.breaches[0].harmed, ["ITA"]);
    assert_eq!(t.cause("ITA", "GER"), value);
    assert_eq!(t.cause("GER", "ITA"), Fixed::ZERO);
    // Italy's side is suspended; Germany's is not.
    assert!(t.suspended(id, 1, "ITA"));
    assert!(!t.suspended(id, 1, "GER"));
    // A call unanswered by Germany later is a breach of its own, against Italy.
    t.record_breach(
        id,
        1,
        "GER",
        value,
        date,
        BreachCause::CallUnanswered { war: 3 },
    )
    .unwrap();
    assert_eq!(t.cause("ITA", "GER"), value + value);
    // Three parties: the value is shared among those harmed.
    let tri = pact(
        &mut t,
        "tri",
        &["A", "B", "C"],
        vec![mutual(ClauseKind::Defence)],
    );
    t.record_breach(
        tri,
        0,
        "A",
        value,
        date,
        BreachCause::CallUnanswered { war: 4 },
    )
    .unwrap();
    assert_eq!(t.cause("B", "A"), Fixed::from_int(500));
    assert_eq!(t.cause("C", "A"), Fixed::from_int(500));
}

/// A world with a breach saves and loads with the same hash; the breach enters the hash.
#[test]
fn breaches_save_and_enter_the_hash() {
    let d = load(true);
    let mut w = d.new_world();
    let before = laplace_schelling::hash::hash_of(&w);
    let a = w
        .treaties
        .agreements
        .iter()
        .find(|a| a.key == "franco_polish_alliance")
        .unwrap()
        .id;
    w.treaties
        .record_breach(
            a,
            0,
            "FRA",
            Fixed::from_int(10),
            Date::new(1925, 3, 1),
            BreachCause::CallUnanswered { war: 0 },
        )
        .unwrap();
    assert_ne!(laplace_schelling::hash::hash_of(&w), before);
    let bytes = bf_sim::save_format::encode_world(&w, None).unwrap();
    let back = bf_sim::save_format::decode_world(&bytes).unwrap();
    assert_eq!(back, w);
    assert_eq!(
        laplace_schelling::hash::hash_of(&back),
        laplace_schelling::hash::hash_of(&w)
    );
}
