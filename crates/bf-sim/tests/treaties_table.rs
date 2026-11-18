//! The negotiation table (`treaty_table`, `docs/treaties.pt-BR.md` section 13): the AI answers by
//! the account, counters by touching as little as it can, a refused proposal waits six months, a
//! player's deadline lapses, and a denunciation ends an agreement by its notice or as a breach.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::GameWorld;
use bf_sim::treaty::{
    Agreement, AgreementStatus, BreachCause, Clause, ClauseCatalogue, ClauseKind, Proposal, Side,
};
use bf_sim::treaty_table::{Outcome, ProposalAnswer, week};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static D: OnceLock<GameData> = OnceLock::new();
    D.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn cat() -> &'static ClauseCatalogue {
    data().clauses.as_ref().unwrap()
}

fn pay(giver: Side, receiver: Side, amount: i64) -> Clause {
    Clause {
        kind: ClauseKind::Payment {
            amount: Fixed::from_int(amount),
        },
        giver,
        receiver,
        until: None,
        note: None,
    }
}

fn proposal(clauses: Vec<Clause>) -> Proposal {
    Proposal {
        side_a: vec!["GBR".into()],
        side_b: vec!["FRA".into()],
        clauses,
        term_years: None,
        notice_years: 0,
    }
}

fn treasury(w: &GameWorld, tag: &str) -> Fixed {
    let i = w.countries.iter().position(|(_, c)| c.tag == tag).unwrap();
    w.econ.markets[i].treasury
}

const D0: Date = Date::new(1925, 1, 5);

#[test]
fn the_ai_accepts_what_its_account_gains() {
    let mut w = data().new_world();
    let p = proposal(vec![pay(Side::Senior, Side::Junior, 1000)]);
    let id = w.treaties.table.propose("GBR", "FRA", p, D0).unwrap();
    let (g, f) = (treasury(&w, "GBR"), treasury(&w, "FRA"));
    week(&mut w, cat(), D0);
    let c = w.treaties.table.closed.iter().find(|c| c.id == id).unwrap();
    assert!(matches!(c.outcome, Outcome::Signed(_)), "{:?}", c.outcome);
    assert!(w.treaties.table.open.is_empty());
    assert_eq!(treasury(&w, "GBR"), g - Fixed::from_int(1000));
    assert_eq!(treasury(&w, "FRA"), f + Fixed::from_int(1000));
}

#[test]
fn the_ai_counters_by_lowering_its_worst_clause() {
    let mut w = data().new_world();
    // Britain gives 100 and asks 150: France counters, lowering what it gives.
    let p = proposal(vec![
        pay(Side::Senior, Side::Junior, 100),
        pay(Side::Junior, Side::Senior, 150),
    ]);
    let id = w
        .treaties
        .table
        .propose("GBR", "FRA", p.clone(), D0)
        .unwrap();
    week(&mut w, cat(), D0);
    let c = w.treaties.table.closed.iter().find(|c| c.id == id).unwrap();
    let Outcome::Countered(next) = c.outcome else {
        panic!("{:?}", c.outcome)
    };
    let q = w.treaties.table.open.iter().find(|q| q.id == next).unwrap();
    assert_eq!((q.from.as_str(), q.to.as_str(), q.round), ("FRA", "GBR", 2));
    assert_eq!(q.proposal.side_a, ["FRA"]);
    // Swapped sides: France (now senior) gives the lowered amount, below 150 and at most 100.
    let lowered = q
        .proposal
        .clauses
        .iter()
        .find_map(|c| match (&c.kind, &c.giver) {
            (ClauseKind::Payment { amount }, Side::Senior) => Some(*amount),
            _ => None,
        })
        .unwrap();
    assert!(
        lowered < Fixed::from_int(100) && lowered > Fixed::from_int(90),
        "{lowered:?}"
    );
    // The same world gives the same answer.
    let mut w2 = data().new_world();
    w2.treaties.table.propose("GBR", "FRA", p, D0).unwrap();
    week(&mut w2, cat(), D0);
    assert_eq!(w2.treaties.table.open, w.treaties.table.open);
}

#[test]
fn a_refused_proposal_waits_six_months() {
    let mut w = data().new_world();
    // A demand with nothing in return: France refuses (nothing to lower that keeps a clause).
    let p = proposal(vec![pay(Side::Junior, Side::Senior, 500)]);
    let id = w
        .treaties
        .table
        .propose("GBR", "FRA", p.clone(), D0)
        .unwrap();
    week(&mut w, cat(), D0);
    let c = w.treaties.table.closed.iter().find(|c| c.id == id).unwrap();
    assert!(
        matches!(c.outcome, Outcome::Refused(Some(_))),
        "{:?}",
        c.outcome
    );
    assert_eq!(w.treaties.table.propose("GBR", "FRA", p.clone(), D0), None);
    // Six months later it may come back.
    let later = Date::new(1925, 8, 4);
    week(&mut w, cat(), later);
    assert!(w.treaties.table.propose("GBR", "FRA", p, later).is_some());
}

#[test]
fn a_players_deadline_lapses_and_a_player_answers() {
    let mut w = data().new_world();
    w.player = w
        .countries
        .iter()
        .find(|(_, c)| c.tag == "FRA")
        .map(|(id, _)| id);
    let p = proposal(vec![pay(Side::Senior, Side::Junior, 10)]);
    let a = w
        .treaties
        .table
        .propose("GBR", "FRA", p.clone(), D0)
        .unwrap();
    week(&mut w, cat(), D0);
    assert_eq!(w.treaties.table.open.len(), 1, "the player has four weeks");
    week(&mut w, cat(), Date::new(1925, 2, 2));
    assert!(w.treaties.table.open.is_empty());
    let c = w.treaties.table.closed.iter().find(|c| c.id == a).unwrap();
    assert_eq!(c.outcome, Outcome::Lapsed);
    // Answered in time, it is signed.
    let mut p2 = p;
    p2.clauses[0] = pay(Side::Senior, Side::Junior, 11);
    let b = w.treaties.table.propose("GBR", "FRA", p2, D0).unwrap();
    w.treaties
        .table
        .answers
        .push((b, "FRA".into(), ProposalAnswer::Accept));
    week(&mut w, cat(), Date::new(1925, 1, 12));
    let c = w.treaties.table.closed.iter().find(|c| c.id == b).unwrap();
    assert!(matches!(c.outcome, Outcome::Signed(_)));
}

fn pact(w: &mut GameWorld, notice: u16) -> u32 {
    w.treaties.sign(Agreement {
        id: 0,
        key: "pact".into(),
        side_a: vec!["GBR".into(), "FRA".into()],
        side_b: Vec::new(),
        signed: 1925,
        term_years: None,
        notice_years: notice,
        kept: vec![Clause {
            kind: ClauseKind::Defence,
            giver: Side::All,
            receiver: Side::All,
            until: None,
            note: None,
        }],
        status: AgreementStatus::InForce,
    })
}

#[test]
fn a_denunciation_ends_by_its_notice_or_as_a_breach() {
    let mut w = data().new_world();
    let a = pact(&mut w, 2);
    w.treaties.table.denounce.push((a, "GBR".into()));
    week(&mut w, cat(), Date::new(1930, 3, 3));
    let ag = w.treaties.in_force(a).unwrap();
    assert_eq!(ag.term_years, Some(7), "ends in 1932, two years of notice");
    assert!(w.treaties.breaches.is_empty());
    let b = pact(&mut w, 0);
    w.treaties.table.denounce.push((b, "GBR".into()));
    week(&mut w, cat(), Date::new(1930, 3, 10));
    assert!(w.treaties.in_force(b).is_none());
    assert_eq!(w.treaties.breaches.len(), 1);
    assert_eq!(w.treaties.breaches[0].cause, BreachCause::Denounced);
    // Only a party denounces.
    w.treaties.table.denounce.push((a, "USA".into()));
    week(&mut w, cat(), Date::new(1930, 3, 17));
    assert!(w.treaties.in_force(a).is_some());
}
