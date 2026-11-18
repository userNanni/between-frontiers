//! The bridge from clauses to effects (`treaty_exec`): a peace with cession and reparation, an
//! independence born owing, a release as a subject, a union, a payment and a revision. Every
//! settlement conserves what the border operations conserve (`borders::Totals`), the ledger's
//! claims and debts included.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::borders::Totals;
use bf_sim::borders::op;
use bf_sim::data::GameData;
use bf_sim::game::{GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, DebtTerms, Proposal, Side};
use bf_sim::treaty_exec::{self, Settlement};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

/// The world a month in, so that the books are not zero.
fn world() -> GameWorld {
    static WORLD: OnceLock<GameWorld> = OnceLock::new();
    WORLD
        .get_or_init(|| {
            let d = data();
            let mut s = Session::new(
                d.new_world(),
                1925,
                Date::new(1925, 1, 1),
                2,
                Some(d.rules()),
            );
            s.sim.run_until(Date::new(1925, 2, 1));
            s.world().clone()
        })
        .clone()
}

fn ix(w: &GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
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

fn parties(t: &[&str]) -> Side {
    Side::Parties(t.iter().map(|x| x.to_string()).collect())
}

fn debt(principal: i64) -> DebtTerms {
    DebtTerms {
        principal: Fixed::from_int(principal),
        rate: Fixed::from_ratio(5, 100),
        annuity: Fixed::from_int(principal / 20),
        years: 30,
        currency: Default::default(),
        deferrable: Fixed::ZERO,
        transfer_cap: None,
        pledge: None,
        contingent: Fixed::ZERO,
        grace_years: 0,
    }
}

/// Settles and runs a treaty on the world, checking that every total holds.
fn sign(w: &mut GameWorld, p: &Proposal, key: &str) -> Settlement {
    let cat = data().clauses.as_ref().unwrap();
    let s = treaty_exec::settle(cat, p, key).unwrap();
    let before = Totals::of(w);
    let money: Fixed = w.econ.markets.iter().map(|m| m.treasury).sum();
    let signed = w.treaties.agreements.len();
    let id = treaty_exec::queue(w, &s, 1925).unwrap();
    // The agreement is recorded with its parties and what holds while in force.
    assert_eq!(w.treaties.agreements.len(), signed + 1);
    let a = w.treaties.agreements.last().unwrap();
    assert_eq!((a.id, a.key.as_str(), a.signed), (id, key, 1925));
    assert_eq!((&a.side_a, &a.side_b), (&p.side_a, &p.side_b));
    assert_eq!(a.kept, s.kept);
    op::run_pending(w, &data().border_rules, Date::new(1925, 2, 1));
    assert!(w.borders.refused.is_empty(), "{:?}", w.borders.refused);
    let mut after = Totals::of(w);
    for c in &w.borders.changes {
        after.diaspora += c.repatriated.0;
        after.arrived += c.repatriated.1;
    }
    assert_eq!(after.people, before.people);
    assert_eq!(after.ledger_net, Fixed::ZERO);
    assert_eq!(after.debt, before.debt);
    assert_eq!(
        w.econ.markets.iter().map(|m| m.treasury).sum::<Fixed>(),
        money
    );
    assert!(op::tables_agree(w));
    s
}

#[test]
fn a_peace_with_cession_and_reparation() {
    let mut w = world();
    let (ger, pol, fra) = (ix(&w, "GER"), ix(&w, "POL"), ix(&w, "FRA"));
    let owed = w.econ.ledger.owed_by(ger as u16);
    let p = Proposal {
        side_a: vec!["FRA".into(), "POL".into()],
        side_b: vec!["GER".into()],
        clauses: vec![
            clause(
                ClauseKind::CedeStates {
                    states: vec!["GER-OBERSCHLESIEN".into()],
                },
                parties(&["GER"]),
                parties(&["POL"]),
            ),
            clause(
                ClauseKind::Debt {
                    terms: Some(debt(1_000_000_000)),
                },
                parties(&["GER"]),
                parties(&["FRA"]),
            ),
            clause(
                ClauseKind::Demilitarized {
                    states: vec!["DEU-RP".into()],
                },
                parties(&["GER"]),
                Side::Senior,
            ),
        ],
        term_years: None,
        notice_years: 0,
    };
    let s = sign(&mut w, &p, "peace-test");
    let state = data()
        .map
        .states
        .iter()
        .position(|st| st.key == "GER-OBERSCHLESIEN")
        .unwrap();
    assert_eq!(usize::from(w.econ.regions[state].market), pol);
    assert_eq!(
        w.econ.ledger.owed_by(ger as u16),
        owed + Fixed::from_int(1_000_000_000)
    );
    assert!(
        w.econ
            .ledger
            .debts
            .iter()
            .any(|d| d.treaty == "peace-test" && usize::from(d.creditor) == fra)
    );
    // The demilitarised zone holds while the agreement is in force.
    assert_eq!(s.kept.len(), 1);
}

#[test]
fn an_independence_born_owing_and_a_release_as_subject() {
    let mut w = world();
    let gbr = ix(&w, "GBR");
    let n = w.countries.len();
    // The new country is a side of the treaty that creates it (as the independence by
    // movement writes it); it is not a signatory of its own creation.
    let p = Proposal {
        side_a: vec!["GBR".into()],
        side_b: vec!["NGA".into()],
        clauses: vec![
            clause(
                ClauseKind::ReleaseCountry {
                    states: [
                        "NGA-BENUE-NIGER",
                        "NGA-BORNU-ADAMAWA",
                        "NGA-EASTERN",
                        "NGA-KANO-SOKOTO",
                        "NGA-WESTERN",
                    ]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
                    tag: "NGA".into(),
                    bundle: Some("protectorate".into()),
                },
                parties(&["GBR"]),
                parties(&["GBR"]),
            ),
            clause(
                ClauseKind::Debt {
                    terms: Some(debt(500_000_000)),
                },
                parties(&["NGA"]),
                parties(&["GBR"]),
            ),
        ],
        term_years: None,
        notice_years: 0,
    };
    sign(&mut w, &p, "independence-test");
    let nga = ix(&w, "NGA");
    assert_eq!(nga, n);
    assert_eq!(
        w.econ.ledger.owed_by(nga as u16),
        Fixed::from_int(500_000_000)
    );
    assert_eq!(
        w.borders.subjects[nga]
            .as_ref()
            .map(|s| usize::from(s.overlord)),
        Some(gbr)
    );
}

#[test]
fn a_union_payment_and_revision() {
    let mut w = world();
    let (aut, ger, fra) = (ix(&w, "AUT"), ix(&w, "GER"), ix(&w, "FRA"));
    let fra_cash = w.econ.markets[fra].treasury;
    let mut forgiven = debt(1);
    forgiven.principal = Fixed::ZERO;
    forgiven.annuity = Fixed::ZERO;
    forgiven.grace_years = 3;
    let p = Proposal {
        side_a: vec!["GER".into()],
        side_b: vec!["AUT".into(), "FRA".into()],
        clauses: vec![
            clause(ClauseKind::Union, parties(&["AUT"]), parties(&["GER"])),
            clause(
                ClauseKind::Payment {
                    amount: Fixed::from_int(2_000_000),
                },
                parties(&["GER"]),
                parties(&["FRA"]),
            ),
            clause(
                ClauseKind::ReviseDebt {
                    treaty: "dawes_plan".into(),
                    terms: forgiven,
                },
                parties(&["FRA"]),
                parties(&["GER"]),
            ),
        ],
        term_years: None,
        notice_years: 0,
    };
    sign(&mut w, &p, "union-test");
    assert_eq!(
        w.econ
            .regions
            .iter()
            .filter(|r| usize::from(r.market) == aut)
            .count(),
        0
    );
    assert_eq!(
        w.econ.markets[fra].treasury,
        fra_cash + Fixed::from_int(2_000_000)
    );
    let line = w
        .econ
        .ledger
        .debts
        .iter()
        .find(|d| usize::from(d.creditor) == fra && usize::from(d.debtor) == ger)
        .unwrap();
    assert_eq!(line.principal, Fixed::ZERO, "France forgave its share");
    assert_eq!(line.suspended_until, 1927);
}

#[test]
fn what_cannot_be_carried_out_is_refused() {
    let cat = data().clauses.as_ref().unwrap();
    let p = Proposal {
        side_a: vec!["FRA".into(), "GBR".into()],
        side_b: vec!["GER".into()],
        clauses: vec![clause(
            ClauseKind::CedeStates {
                states: vec!["DEU-RP".into()],
            },
            Side::Junior,
            Side::Senior,
        )],
        term_years: None,
        notice_years: 0,
    };
    assert!(treaty_exec::settle(cat, &p, "x").is_err(), "two receivers");
}

/// The record of each line follows the rule switch (`data/rules.ron`, off): off, the 1925 lines
/// carry no record and hash as before; on, they record what is paid from the first week.
#[test]
fn the_record_of_the_ledger_follows_its_switch() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut on = GameData::load(&root).unwrap();
    assert!(!on.switches.ledger_record);
    assert!(!data().new_world().econ.ledger.record);
    on.switches.switch_on("ledger_record").unwrap();
    let w = on.new_world();
    assert!(w.econ.ledger.record);
    let mut s = Session::new(w, 1925, Date::new(1925, 1, 1), 2, Some(on.rules()));
    s.sim.run_until(Date::new(1925, 2, 1));
    let ledger = &s.world().econ.ledger;
    assert!(ledger.debts.iter().any(|d| d.paid > Fixed::ZERO));
    // Off, nothing is recorded.
    assert!(
        world()
            .econ
            .ledger
            .debts
            .iter()
            .all(|d| d.paid == Fixed::ZERO)
    );
}
