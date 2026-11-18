//! Crisis and declaration (phase 5, slice B, draft) on the real scenario: a demand is answered
//! by the AI as W3 says, carried out as a treaty when yielded, lapses into a refusal, and a
//! refusal leads to war or to backing down.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d
    })
}

fn session() -> Session {
    let d = data();
    Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    )
}

fn index(s: &Session, tag: &str) -> usize {
    s.world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .unwrap()
}

fn demand(by: &str, target: &str, kind: ClauseKind) -> GameCommand {
    GameCommand::Demand {
        tag: by.into(),
        target: target.into(),
        demand: Proposal {
            side_a: vec![by.into()],
            side_b: vec![target.into()],
            clauses: vec![Clause {
                kind,
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

fn treasury(s: &Session, tag: &str) -> Fixed {
    s.world().econ.markets[index(s, tag)].treasury
}

/// A small payment demanded by a far larger army is yielded and paid, treasury to treasury.
#[test]
fn a_small_payment_demanded_by_a_far_larger_army_is_paid() {
    let mut s = session();
    s.sim.run_until(Date::new(1925, 3, 2));
    let fra = index(&s, "FRA");
    let bel = index(&s, "BEL");
    let mil = &data().war.as_ref().unwrap().military;
    let ratio = bf_sim::war::crisis::land_men(mil, &s.world().war, fra) * 1000
        / bf_sim::war::crisis::land_men(mil, &s.world().war, bel);
    assert!(ratio > 3000, "France's army is {ratio} ‰ of Belgium's");
    let amount = Fixed::from_int(10);
    let (fra_before, bel_before) = (treasury(&s, "FRA"), treasury(&s, "BEL"));
    s.submit(demand("FRA", "BEL", ClauseKind::Payment { amount }));
    s.sim.run_until(Date::new(1925, 3, 20));
    assert!(
        s.world().war.conflicts.crises.is_empty(),
        "the crisis closed"
    );
    assert!(s.world().war.conflicts.wars.is_empty());
    // The week's budget moves both treasuries too: the payment shows as the gap between them.
    let moved = (treasury(&s, "FRA") - fra_before) - (treasury(&s, "BEL") - bel_before);
    assert!(moved > amount, "{moved:?}");
}

/// Union is existential: refused whatever the odds, and the AI that demanded backs down.
#[test]
fn an_annexation_is_refused_and_the_ai_backs_down() {
    let mut s = session();
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(demand("FRA", "BEL", ClauseKind::Union));
    s.sim.run_until(Date::new(1925, 3, 20));
    let c = &s.world().war.conflicts;
    assert!(c.crises.is_empty() && c.wars.is_empty(), "{c:?}");
}

/// The player's demand refused: the player declares, and both sides call up their reserve.
#[test]
fn a_refusal_lets_the_player_declare_war() {
    let mut s = session();
    s.submit(GameCommand::ChoosePlayer { tag: "FRA".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(demand("FRA", "BEL", ClauseKind::Union));
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = {
        let c = &s.world().war.conflicts;
        assert_eq!(c.crises.len(), 1, "the refusal waits for the player");
        assert_eq!(c.crises[0].stage, bf_sim::war::Stage::Refused);
        c.crises[0].id
    };
    s.submit(GameCommand::Declare {
        tag: "FRA".into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    let (fra, bel) = (index(&s, "FRA"), index(&s, "BEL"));
    let w = s.world();
    assert_eq!(w.war.conflicts.wars.len(), 1);
    assert!(w.war.conflicts.at_war(fra as u16, bel as u16));
    assert!(w.war.countries[fra].mobilized && w.war.countries[bel].mobilized);
}

/// A demand on the player lapses into a refusal after the deadline.
#[test]
fn a_demand_left_unanswered_lapses_into_a_refusal() {
    let mut s = session();
    s.submit(GameCommand::ChoosePlayer { tag: "BEL".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(demand(
        "FRA",
        "BEL",
        ClauseKind::Payment {
            amount: Fixed::from_int(10),
        },
    ));
    s.sim.run_until(Date::new(1925, 3, 20));
    assert_eq!(s.world().war.conflicts.crises.len(), 1, "the deadline runs");
    s.sim.run_until(Date::new(1925, 4, 20));
    // Lapsed, refused, and France (the AI) backed down.
    assert!(s.world().war.conflicts.crises.is_empty());
    assert!(s.world().war.conflicts.wars.is_empty());
}
