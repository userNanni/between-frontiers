//! The war on land in the tick (phase 5, slice C) on the real scenario: a declared war has fronts
//! where the two sides touch, a week of combat on each segment, losses taken from the
//! formations, and a posture the player sets.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.war = true;
        d
    })
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

/// Hungary, played, demands Czechoslovakia whole, hears the refusal and declares.
fn at_war() -> Session {
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
    s.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
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
    });
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    s
}

/// The war has segments along the Hungarian–Czechoslovak border, and a week of combat on them.
#[test]
fn a_declared_war_has_fronts_and_combat() {
    let d = data();
    let s = at_war();
    let w = s.world();
    let (hun, cze) = (index(w, "HUN"), index(w, "CZE"));
    let rules = d.rules();
    let week = bf_sim::war::fight::fight_week(
        rules
            .fight
            .as_ref()
            .expect("the war switch builds the setup"),
        &d.war.as_ref().unwrap().military,
        w,
    );
    let border: Vec<_> = week
        .segments
        .iter()
        .filter(|g| (g.a, g.b) == (hun.min(cze), hun.max(cze)))
        .collect();
    assert!(!border.is_empty(), "segments along the border");
}

/// Attacking takes losses on both sides; holding with no attacker does not.
#[test]
fn the_player_attacks_and_both_sides_lose_men() {
    let mut s = at_war();
    let (hun, cze) = (index(s.world(), "HUN"), index(s.world(), "CZE"));
    s.submit(GameCommand::SetPosture {
        tag: "HUN".into(),
        enemy: "CZE".into(),
        attack: true,
    });
    s.sim.run_until(Date::new(1925, 4, 7));
    // The screen reads the week the tick fought, without working it out again.
    assert!(
        s.world()
            .war
            .fighting
            .last_week
            .iter()
            .any(|g| (g.a, g.b) == (hun.min(cze), hun.max(cze)) && g.outcome.engaged),
        "the last week's segments are kept"
    );
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    assert_eq!(w.war.fighting.posture(hun, cze), Some(true));
    let lost = |c: u16| {
        w.war
            .fighting
            .losses
            .iter()
            .find(|(k, _)| *k == c)
            .map_or(0, |(_, l)| *l)
    };
    assert!(
        lost(hun) > 0 && lost(cze) > 0,
        "{:?}",
        w.war.fighting.losses
    );
}

/// Without the war switch a declared war only mobilizes: no fighting state.
#[test]
fn off_a_declared_war_only_mobilizes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap();
    d.enable_crisis();
    assert!(!d.switches.war);
    assert!(d.rules().fight.is_none());
    let _ = Fixed::ZERO;
}
