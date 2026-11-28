//! The movement against the war (`war::antiwar`, switch `anti_war`, off) on the real scenario:
//! Hungary, played, declares on Czechoslovakia without a cause and loses land. Its workers,
//! peasants and soldiers tire of the war; the movement grows every week, strikes in the war
//! industry and refuses to fight. With the switch off nothing of it exists.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use laplace_schelling::{Date, Fixed};

fn data(switch: bool) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // The switch is read at load: off, the movement leaves the catalogue.
    let rules = if switch { "anti_war" } else { "" };
    let mut d = GameData::load_with(&root, rules).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    d.switches.anti_war = switch;
    // The overrun rule off: the war goes on however much the front takes.
    let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
    c.support.overrun = 2000;
    c.support.capital_overrun = 2000;
    d
}

/// Hungary, played, demands a payment of Czechoslovakia, hears the refusal and declares.
fn at_war(d: &GameData) -> Session {
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
                kind: ClauseKind::Payment {
                    amount: Fixed::from_int(100),
                },
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

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

#[test]
fn hungary_tires_of_its_war() {
    let on = data(true);
    let kind = on
        .politics
        .as_ref()
        .unwrap()
        .movements
        .iter()
        .position(|m| m.key == "anti_war")
        .expect("the movement is in the catalogue with the switch on");
    let mut s = at_war(&on);
    let hun = index(s.world(), "HUN");
    let mut peak = 0u16;
    let mut seen = Vec::new();
    for (y, m) in [
        (1925, 5),
        (1925, 7),
        (1925, 9),
        (1925, 11),
        (1926, 1),
        (1926, 3),
    ] {
        s.sim.run_until(Date::new(y, m, 1));
        let w = s.world();
        let at_war = !w.war.conflicts.wars.is_empty();
        let a = w.war.antiwar.of(hun).cloned().unwrap_or_default();
        let support = w
            .war
            .fighting
            .support
            .iter()
            .find(|x| x.country == hun)
            .map_or(-1, |x| x.value);
        let mirrored = w.politics.movements[usize::from(hun)][kind].support;
        println!(
            "{y}-{m:02}: at war {at_war}, war support {support}, weariness {}, movement {} per \
             mille (politics {mirrored}), mobilized {}, strike {}, mutiny {}, legitimacy {}",
            a.weariness,
            a.per_mille(),
            a.mobilized,
            a.strike,
            a.mutiny,
            w.politics.legitimacy(usize::from(hun))
        );
        if let Some(x) = w.war.fighting.support.iter().find(|x| x.country == hun) {
            println!("  countdown {:?}, fallen {:?}", x.countdown, x.fallen);
        }
        peak = peak.max(a.per_mille());
        seen.push(a);
    }
    assert!(peak > 0, "the movement never grew");
    assert!(seen.iter().any(|a| a.strike > 0 && a.mutiny > 0));
}

#[test]
fn off_the_movement_does_not_exist() {
    let off = data(false);
    assert!(
        off.politics
            .as_ref()
            .unwrap()
            .movements
            .iter()
            .all(|m| m.key != "anti_war")
    );
    let mut s = at_war(&off);
    let hun = index(s.world(), "HUN");
    for (y, m) in [(1925, 7), (1925, 11), (1926, 1), (1926, 3), (1926, 9)] {
        s.sim.run_until(Date::new(y, m, 1));
        let w = s.world();
        assert!(w.war.antiwar.is_empty());
        let x = w.war.fighting.support.iter().find(|x| x.country == hun);
        println!(
            "off {y}-{m:02}: at war {}, war support {:?}, countdown {:?}",
            !w.war.conflicts.wars.is_empty(),
            x.map(|x| x.value),
            x.and_then(|x| x.countdown)
        );
    }
}
