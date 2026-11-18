//! Third parties in a war (phase 5, slice B, W4) on the treaties of 1925 in force: who the
//! agreements call in when a war is declared, by which clause, and what not answering costs.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, DutyKind, Proposal, Side};
use bf_sim::war::third::{CallSide, who_would_join};
use laplace_schelling::Date;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.treaties_in_force = true;
        d
    })
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

fn union(by: &str, target: &str) -> GameCommand {
    GameCommand::Demand {
        tag: by.into(),
        target: target.into(),
        demand: Proposal {
            side_a: vec![by.into()],
            side_b: vec![target.into()],
            clauses: vec![Clause {
                kind: ClauseKind::Union,
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
        weeks: None,
    }
}

/// Before the declaration, the screen reads who would join: Czechoslovakia's allies of the
/// Little Entente and France, all beside Czechoslovakia.
#[test]
fn the_little_entente_and_france_would_defend_czechoslovakia() {
    let w = data().new_world();
    assert!(
        !w.treaties.agreements.is_empty(),
        "the 1925 treaties are in force"
    );
    let entries = who_would_join(&w, "HUN", "CZE");
    let defenders: Vec<&str> = entries
        .iter()
        .filter(|e| e.side == Some(CallSide::Defender) && !e.barred && !e.suspended)
        .map(|e| e.party.as_str())
        .collect();
    for ally in ["YUG", "ROM", "FRA"] {
        assert!(defenders.contains(&ally), "{ally}: {entries:?}");
    }
    assert!(entries.iter().all(|e| e.kind != DutyKind::JoinAttacker));
}

/// Hungary demands Czechoslovakia whole, hears the refusal and declares: the AI allies enter
/// beside Czechoslovakia at once and call up their reserve.
#[test]
fn declared_war_brings_in_the_allies() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "HUN".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(union("HUN", "CZE"));
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    let w = s.world();
    let war = &w.war.conflicts.wars[0];
    assert_eq!(war.side_a, vec![index(w, "HUN")]);
    for t in ["CZE", "YUG", "ROM", "FRA"] {
        let m = index(w, t);
        assert!(war.side_b.contains(&m), "{t} {:?}", war.side_b);
        assert!(w.war.countries[m as usize].mobilized, "{t} called up");
    }
    assert!(w.war.conflicts.calls.is_empty(), "no call for the AI");
}

/// The player called by an agreement does not answer: a breach of the clause, and a cause for
/// the party left alone.
#[test]
fn a_call_not_answered_is_a_breach() {
    let d = data();
    // A few weeks in, so the forces have priced their orders: the value of the breach.
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1925, 3, 2));
    let mut w = s.world().clone();
    let fra = index(&w, "FRA");
    w.player = w.countries.iter().nth(fra as usize).map(|(id, _)| id);
    let (hun, cze) = (index(&w, "HUN"), index(&w, "CZE"));
    w.war.conflicts.wars.push(bf_sim::war::WarOn {
        id: 0,
        side_a: vec![hun],
        side_b: vec![cze],
        crisis: 0,
        demand: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
            clauses: Vec::new(),
            term_years: None,
            notice_years: 0,
        },
        since: (1925, 3, 2),
    });
    w.war.conflicts.next_id = 1;
    let date = Date::new(1925, 3, 2);
    bf_sim::war::third::on_declaration(&mut w, 0, 2, date);
    let call = w
        .war
        .conflicts
        .calls
        .iter()
        .find(|c| c.party == fra)
        .cloned()
        .expect("France is called");
    assert_eq!(call.side, CallSide::Defender);
    assert!(!w.war.conflicts.wars[0].side_b.contains(&fra), "not yet in");
    assert!(w.treaties.breaches.is_empty());
    bf_sim::war::third::answer(&mut w, 0, fra, false, date);
    assert!(w.war.conflicts.calls.is_empty());
    assert!(!w.war.conflicts.wars[0].side_b.contains(&fra));
    assert_eq!(w.treaties.breaches.len(), 1);
    assert_eq!(w.treaties.breaches[0].by, "FRA");
    assert!(w.treaties.cause("CZE", "FRA").is_positive());
}

/// A call left to lapse is honoured by the rules (P2, recommended).
#[test]
fn a_lapsed_call_is_honoured() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "FRA".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    // Hungary is the AI and would back down at the refusal: the war is set up by hand.
    let mut w = s.world().clone();
    let (hun, cze, fra) = (index(&w, "HUN"), index(&w, "CZE"), index(&w, "FRA"));
    w.war.conflicts.wars.push(bf_sim::war::WarOn {
        id: 0,
        side_a: vec![hun],
        side_b: vec![cze],
        crisis: 0,
        demand: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
            clauses: Vec::new(),
            term_years: None,
            notice_years: 0,
        },
        since: (1925, 3, 2),
    });
    w.war.conflicts.next_id = 1;
    bf_sim::war::third::on_declaration(
        &mut w,
        0,
        bf_sim::war::crisis::RULES_DEADLINE,
        Date::new(1925, 3, 2),
    );
    assert!(w.war.conflicts.calls.iter().any(|c| c.party == fra));
    let mut s = Session::new(w, 1925, Date::new(1925, 3, 2), 2, Some(d.rules()));
    s.sim.run_until(Date::new(1925, 4, 20));
    let w = s.world();
    assert!(w.war.conflicts.calls.is_empty(), "the call lapsed");
    assert!(
        w.war.conflicts.wars[0].side_b.contains(&fra),
        "and was honoured"
    );
    assert!(w.treaties.breaches.is_empty());
}
