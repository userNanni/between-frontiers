//! Fronts as objects of command (`war::fronts`, switch `fronts`, off) on the real scenario:
//! Hungary, played, declares on Czechoslovakia, whose allies come in. Hungary fights on
//! several fronts; with orders it sets their shares, an objective and a reserve, and the men
//! move in weeks. The men are the same: those on the lines, in reserve and in transit add up to
//! the army.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use laplace_schelling::{Date, Fixed};

fn data(switch: bool) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    d.switches.fronts = switch;
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

fn at_war_pair(d: &GameData, a: &str, b: &str) -> Session {
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: a.into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: a.into(),
        target: b.into(),
        demand: Proposal {
            side_a: vec![a.into()],
            side_b: vec![b.into()],
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
        tag: a.into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    s
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

fn tag(w: &GameWorld, c: u16) -> String {
    w.countries
        .iter()
        .nth(usize::from(c))
        .unwrap()
        .1
        .tag
        .clone()
}

/// (enemy, segments, the men on them) of `c` last week.
fn lines(w: &GameWorld, c: u16) -> Vec<(String, u64, u64)> {
    let mut out: Vec<(String, u64, u64)> = Vec::new();
    for g in &w.war.fighting.last_week {
        // `number` is thousands of men.
        let k = |i: usize| (g.outcome.terms[i].number.to_f64_lossy() * 1000.0) as u64;
        let (e, men) = if g.a == c {
            (g.b, k(0))
        } else if g.b == c {
            (g.a, k(1))
        } else {
            continue;
        };
        let t = tag(w, e);
        match out.iter_mut().find(|x| x.0 == t) {
            Some(x) => {
                x.1 += 1;
                x.2 += men;
            }
            None => out.push((t, 1, men)),
        }
    }
    out
}

#[test]
fn hungary_orders_its_fronts() {
    let mut off = at_war(&data(false));
    let mut on = at_war(&data(true));
    for s in [&mut off, &mut on] {
        s.sim.run_until(Date::new(1925, 3, 30));
    }
    let w = on.world();
    let hun = index(w, "HUN");
    let before = lines(w, hun);
    println!("HUN's fronts, 30 March 1925: {before:?}");
    assert!(
        before.len() > 1,
        "Hungary fights on one front only: {before:?}"
    );
    // Most of the army against the first front, a tenth in reserve.
    let first = before[0].0.clone();
    on.submit(GameCommand::SetFrontShare {
        tag: "HUN".into(),
        enemy: first.clone(),
        share: 700,
    });
    on.submit(GameCommand::SetReserve {
        tag: "HUN".into(),
        share: 100,
    });
    let start = Date::new(1925, 3, 30);
    for week in 1..=8 {
        let until = Date::from_days(start.to_days() + 7 * week);
        off.sim.run_until(until);
        on.sim.run_until(until);
        let w = on.world();
        let f = w.war.fronts.of(hun).expect("Hungary has orders");
        let total: u16 =
            f.fronts.iter().map(|x| x.now).sum::<u16>() + f.reserve_now + f.rest_now + f.transit();
        assert_eq!(total, 1000, "week {week}: the men are the same");
        println!(
            "week {week}: {first} set {} now {}, reserve {}, rest {}, transit {}, reinforced \
             {:?}; lines on {:?}; off {:?}",
            f.fronts[0].share,
            f.fronts[0].now,
            f.reserve_now,
            f.rest_now,
            f.transit(),
            f.reinforced,
            lines(w, hun),
            lines(off.world(), hun)
        );
    }
    let f = on.world().war.fronts.of(hun).unwrap();
    assert_eq!(
        f.fronts[0].now, 700,
        "in eight weeks the front has its share"
    );
    assert!(
        off.world().war.fronts.is_empty(),
        "the switch off keeps no orders"
    );
}

/// The Soviet Union, played, declares on Poland: with an objective on the Polish front the
/// state aimed at falls in fewer weeks; the reserve joins where the enemy pushes.
#[test]
fn an_objective_draws_the_front() {
    let mut runs = Vec::new();
    for order in [false, true] {
        let mut s = at_war_pair(&data(true), "SOV", "POL");
        s.sim.run_until(Date::new(1925, 3, 30));
        let w = s.world();
        let (sov, pol) = (index(w, "SOV"), index(w, "POL"));
        // The Polish state behind the first segment.
        let target = w
            .war
            .fighting
            .last_week
            .iter()
            .find_map(|g| {
                if g.a == sov && g.b == pol {
                    Some(w.econ.regions[g.into as usize].state)
                } else if g.b == sov && g.a == pol {
                    Some(w.econ.regions[g.from as usize].state)
                } else {
                    None
                }
            })
            .expect("a Polish front");
        s.submit(GameCommand::SetPosture {
            tag: "SOV".into(),
            enemy: "POL".into(),
            attack: true,
        });
        if order {
            s.submit(GameCommand::SetFrontObjective {
                tag: "SOV".into(),
                enemy: "POL".into(),
                state: Some(target),
            });
            s.submit(GameCommand::SetReserve {
                tag: "SOV".into(),
                share: 100,
            });
        }
        let start = Date::new(1925, 3, 30);
        let mut taken = None;
        for week in 1..=26 {
            s.sim.run_until(Date::from_days(start.to_days() + 7 * week));
            let w = s.world();
            let held = w.econ.regions.iter().enumerate().any(|(r, x)| {
                x.state == target && w.war.fighting.controller(r as u32, x.market) == sov
            });
            if held && taken.is_none() {
                taken = Some(week);
            }
            if w.war.conflicts.wars.is_empty() {
                break;
            }
        }
        let w = s.world();
        println!(
            "SOV->POL, order {order}: target state {target} taken in week {taken:?}; lines {:?}; \
             fronts {:?}",
            lines(w, sov),
            w.war.fronts.of(sov)
        );
        runs.push(taken);
    }
    assert_eq!(
        runs[0], None,
        "without the order the state holds the half year"
    );
    assert!(runs[1].is_some(), "with the objective the state falls");
}
