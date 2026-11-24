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
    let setup = data().rules().crisis.clone().unwrap();
    let ratio = bf_sim::war::crisis::force_ratio(&setup, mil, s.world(), fra as u16, bel as u16);
    assert!(ratio > 3000, "France's force is {ratio} ‰ of Belgium's");
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

/// W3 weighs combat power, not men: the cases before (men on land) and after (combat power, and
/// across the sea the share of the two navies), for a peripheral demand of a tenth.
#[test]
fn the_target_weighs_combat_power_not_men() {
    let mut s = session();
    s.sim.run_until(Date::new(1925, 3, 2));
    let mil = &data().war.as_ref().unwrap().military;
    let setup = data().rules().crisis.clone().unwrap();
    let w = s.world();
    let mut lines = Vec::new();
    for (a, b) in [
        ("FRA", "BEL"),
        ("CHN", "JAP"),
        ("ITA", "ETH"),
        ("GER", "POL"),
        ("USA", "MEX"),
    ] {
        let (x, y) = (index(&s, a) as u16, index(&s, b) as u16);
        let men = bf_sim::war::crisis::land_men(mil, &w.war, usize::from(x)) * 1000
            / bf_sim::war::crisis::land_men(mil, &w.war, usize::from(y)).max(1);
        let force = bf_sim::war::crisis::force_ratio(&setup, mil, w, x, y);
        let yields = |r: i64| bf_sim::war::weigh(&setup.rules, r, 100, false).yields;
        println!(
            "{a} -> {b}: men {men} ‰ (yields {}), combat power {force} ‰ (yields {}), land border {}",
            yields(men),
            yields(force),
            bf_sim::war::crisis::land_border(&setup, w, x, y)
        );
        lines.push((a, b, yields(force)));
    }
    assert!(lines.iter().any(|l| l.0 == "FRA" && l.2));
    // The same men with half their equipment and half their training weigh a quarter: China
    // with the arms and the drill of a warlord army no longer makes Japan yield.
    let chn = index(&s, "CHN");
    let jap = index(&s, "JAP") as u16;
    let mut weak = w.clone();
    for f in &mut weak.war.countries[chn].formations {
        for (_, q) in &mut f.equipment {
            *q /= Fixed::from_int(2);
        }
        f.training /= Fixed::from_int(2);
    }
    let before = bf_sim::war::crisis::force_ratio(&setup, mil, w, chn as u16, jap);
    let after = bf_sim::war::crisis::force_ratio(&setup, mil, &weak, chn as u16, jap);
    println!(
        "CHN -> JAP with half the equipment and half the training: {after} ‰ (was {before} ‰)"
    );
    assert!(after * 3 < before, "{after} against {before}");
    assert!(!bf_sim::war::weigh(&setup.rules, after, 100, false).yields);
}
