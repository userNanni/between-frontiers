//! Treaties in play: a proposal by command, answered on the day by the same account for every
//! party, the recognition gate and the ratification by the interest groups; the one-off clauses
//! act (payment, debt, recognition), the others stay as an agreement in force.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use bf_sim::treaty::{Clause, ClauseKind, DebtTerms, Outcome, Proposal, Side};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).expect("game data")
    })
}

fn session() -> Session {
    let d = data();
    Session::new(d.new_world(), 7, Date::new(1925, 1, 1), 1, Some(d.rules()))
}

fn clause(kind: ClauseKind, giver: Side, receiver: Side) -> Clause {
    Clause {
        kind,
        giver,
        receiver,
        until: None,
        note: None,
    }
}

fn proposal(a: &str, b: &str, clauses: Vec<Clause>) -> Proposal {
    Proposal {
        side_a: vec![a.into()],
        side_b: vec![b.into()],
        clauses,
        term_years: None,
        notice_years: 0,
    }
}

/// Proposes and runs one day; returns the answer.
fn propose(s: &mut Session, p: Proposal, ratify_anyway: bool) -> Outcome {
    let before = s.world().treaties.answers.len();
    s.submit(GameCommand::ProposeTreaty {
        proposal: p,
        ratify_anyway,
    });
    s.sim.step();
    let answers = &s.world().treaties.answers;
    assert_eq!(answers.len(), before + 1, "one answer a proposal");
    answers.last().unwrap().outcome.clone()
}

fn index(s: &Session, tag: &str) -> usize {
    s.world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .unwrap()
}

#[test]
fn recognition_first_then_alliance() {
    let mut s = session();
    let defence = || {
        proposal(
            "USA",
            "SOV",
            vec![clause(ClauseKind::Defence, Side::All, Side::All)],
        )
    };
    // The United States did not recognise the Soviet Union in 1925: no alliance.
    assert!(matches!(
        propose(&mut s, defence(), false),
        Outcome::NotRecognised { .. }
    ));
    // Recognition, both ways, costs neither side anything and passes.
    let rec = proposal(
        "USA",
        "SOV",
        vec![clause(
            ClauseKind::Recognition { border: false },
            Side::All,
            Side::All,
        )],
    );
    assert_eq!(
        propose(&mut s, rec, false),
        Outcome::Signed { legitimacy_paid: 0 }
    );
    let (usa, sov) = (index(&s, "USA") as u16, index(&s, "SOV") as u16);
    assert!(s.world().treaties.recognition.mutual(usa, sov));
    assert_eq!(s.world().treaties.agreements.len(), 1);
    // Now the alliance passes the gate (it is valued, and dormant until war exists).
    assert!(!matches!(
        propose(&mut s, defence(), false),
        Outcome::NotRecognised { .. }
    ));
}

#[test]
fn a_payment_moves_money_and_nothing_else() {
    let mut s = session();
    let (ger, fra) = (index(&s, "GER"), index(&s, "FRA"));
    let cash =
        |s: &Session| -> Vec<Fixed> { s.world().econ.markets.iter().map(|m| m.treasury).collect() };
    let before = cash(&s);
    let amount = Fixed::from_int(1_000_000_000);
    let p = proposal(
        "GER",
        "FRA",
        vec![clause(
            ClauseKind::Payment { amount },
            Side::Senior,
            Side::Junior,
        )],
    );
    let outcome = propose(&mut s, p, true);
    assert!(matches!(outcome, Outcome::Signed { .. }), "{outcome:?}");
    let after = cash(&s);
    // 1925-01-01 is a Thursday: no weekly step ran; only the payment moved treasuries.
    assert_eq!(after[ger], before[ger] - amount);
    assert_eq!(after[fra], before[fra] + amount);
    assert_eq!(
        after.iter().copied().sum::<Fixed>(),
        before.iter().copied().sum::<Fixed>()
    );
    // A payment leaves nothing in force.
    assert!(s.world().treaties.agreements.is_empty());
}

#[test]
fn the_answering_side_refuses_a_loss() {
    let mut s = session();
    // France asks Germany for money and gives nothing: Germany's account is negative.
    let p = proposal(
        "FRA",
        "GER",
        vec![clause(
            ClauseKind::Payment {
                amount: Fixed::from_int(1_000_000_000),
            },
            Side::Junior,
            Side::Senior,
        )],
    );
    match propose(&mut s, p, false) {
        Outcome::Refused { party, total } => {
            assert_eq!(party, "GER");
            assert!(total < Fixed::ZERO);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn ratification_below_the_bar_is_refused_or_paid_for() {
    // Britain yields its foreign policy to France: every British group loses sovereignty.
    let yield_policy = || {
        proposal(
            "GBR",
            "FRA",
            vec![clause(
                ClauseKind::ForeignPolicy,
                Side::Senior,
                Side::Junior,
            )],
        )
    };
    let mut s = session();
    match propose(&mut s, yield_policy(), false) {
        Outcome::NotRatified {
            party,
            support,
            required,
        } => {
            assert_eq!(party, "GBR");
            assert!(support < required, "{support} < {required}");
        }
        other => panic!("{other:?}"),
    }
    // The proposer may ratify anyway, paying legitimacy.
    let gbr = index(&s, "GBR");
    let before = s.world().politics.legitimacy[gbr];
    match propose(&mut s, yield_policy(), true) {
        Outcome::Signed { legitimacy_paid } => {
            assert!(legitimacy_paid > 0);
            assert!(s.world().politics.legitimacy[gbr] < before);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(s.world().treaties.agreements.len(), 1);
}

#[test]
fn a_debt_clause_writes_the_ledger() {
    let mut s = session();
    let (gbr, fra) = (index(&s, "GBR") as u16, index(&s, "FRA") as u16);
    let owed = s.world().econ.ledger.owed_by(gbr);
    let terms = DebtTerms {
        principal: Fixed::from_int(10_000_000_000),
        rate: Fixed::from_ratio(3, 100),
        annuity: Fixed::from_int(600_000_000),
        years: 30,
        currency: Default::default(),
        deferrable: Fixed::ZERO,
        transfer_cap: None,
        pledge: None,
        contingent: Fixed::ZERO,
        grace_years: 0,
    };
    let p = proposal(
        "GBR",
        "FRA",
        vec![clause(
            ClauseKind::Debt { terms: Some(terms) },
            Side::Senior,
            Side::Junior,
        )],
    );
    let outcome = propose(&mut s, p, true);
    assert!(matches!(outcome, Outcome::Signed { .. }), "{outcome:?}");
    let l = &s.world().econ.ledger;
    assert_eq!(l.owed_by(gbr), owed + Fixed::from_int(10_000_000_000));
    assert!(l.claims_of(fra) >= Fixed::from_int(10_000_000_000));
    let n = s.world().countries.len() as u16;
    let net: Fixed = (0..n).map(|c| l.claims_of(c) - l.owed_by(c)).sum();
    assert_eq!(net, Fixed::ZERO);
}

#[test]
fn a_save_in_the_middle_keeps_the_answers() {
    let mut s = session();
    let p = proposal(
        "USA",
        "SOV",
        vec![clause(
            ClauseKind::Recognition { border: false },
            Side::All,
            Side::All,
        )],
    );
    propose(&mut s, p, false);
    let bytes = s.save("test", "mid").unwrap();
    let loaded = Session::load(&bytes, 1, Some(data().rules())).unwrap();
    assert_eq!(loaded.world().treaties, s.world().treaties);
    assert_eq!(loaded.state_hash(), s.state_hash());
}
