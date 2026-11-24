//! Scorched earth (`war::scorch`, switch `scorched_earth`, off) on the real scenario: Hungary,
//! played, declares on Czechoslovakia, which takes Hungarian regions. With Hungary's order
//! against Czechoslovakia, each region that falls loses a share of its plant levels, industry
//! first; the occupier then takes less from it. Without the order, or with the switch off,
//! nothing burns.

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
    d.switches.scorched_earth = switch;
    // The overrun rule off: the war goes on however much the front takes.
    let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
    c.support.overrun = 2000;
    c.support.capital_overrun = 2000;
    d
}

/// Hungary, played, demands a payment of Czechoslovakia, hears the refusal and declares;
/// `order` gives Hungary's scorched earth against Czechoslovakia the week after.
fn at_war(d: &GameData, order: bool) -> Session {
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
    if order {
        s.submit(GameCommand::SetScorchedEarth {
            tag: "HUN".into(),
            enemy: "CZE".into(),
            on: true,
        });
    }
    s.sim.step();
    s
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

/// Plant levels of the Hungarian regions the enemy holds, and what it took from them
/// last week.
fn held(w: &GameWorld) -> (Vec<u32>, i64, Fixed) {
    let hun = index(w, "HUN");
    let regions: Vec<u32> = w
        .war
        .fighting
        .control
        .iter()
        .filter(|(r, by)| *by != hun && w.econ.regions[*r as usize].market == hun)
        .map(|(r, _)| *r)
        .collect();
    let levels = regions
        .iter()
        .map(|r| {
            w.econ.regions[*r as usize]
                .plants
                .iter()
                .map(|p| p.levels)
                .sum::<i64>()
        })
        .sum();
    let taken = w
        .war
        .fighting
        .occupied
        .iter()
        .filter(|o| regions.contains(&o.region))
        .map(|o| o.taken)
        .sum();
    (regions, levels, taken)
}

#[test]
fn what_falls_burns_with_the_order() {
    let on = data(true);
    let mut with = at_war(&on, true);
    let mut without = at_war(&on, false);
    let mut off = at_war(&data(false), true);
    for s in [&mut with, &mut without, &mut off] {
        s.sim.run_until(Date::new(1925, 6, 1));
    }
    let w = with.world();
    let (hun, cze) = (index(w, "HUN"), index(w, "CZE"));
    assert!(w.war.scorch.ordered(hun, cze));
    let burned = &w.war.scorch.burned;
    assert!(
        !burned.is_empty(),
        "Czechoslovakia took Hungarian land and nothing burned"
    );
    // Czechoslovakia's allies take Hungarian land too: the order holds against its whole side.
    let side: Vec<u16> = w
        .war
        .conflicts
        .wars
        .iter()
        .flat_map(|x| x.side_b.iter().copied())
        .collect();
    assert!(side.contains(&cze));
    assert!(
        burned.iter().all(|b| b.by == hun && side.contains(&b.to)),
        "{burned:?}"
    );
    assert!(burned.iter().all(|b| b.share > 0 && b.share <= 500));
    assert!(without.world().war.scorch.burned.is_empty());
    assert!(
        off.world().war.scorch.burned.is_empty(),
        "the switch off burns nothing"
    );
    // The same regions fell in the three runs (the order changes nothing before the fall).
    let (rw, lw, tw) = held(w);
    let (ro, lo, to) = held(without.world());
    let (rf, lf, _) = held(off.world());
    assert_eq!(rw, ro);
    assert_eq!(ro, rf);
    println!("Hungarian regions taken by June 1925: {}", rw.len());
    for b in burned {
        println!(
            "  region {}: {} levels burned ({} per mille)",
            b.region, b.levels, b.share
        );
    }
    println!(
        "plant levels in them: with the order {lw}, without {lo} (switch off {lf}); \
         taken last week: with {:.3}, without {:.3}",
        tw.to_f64_lossy(),
        to.to_f64_lossy()
    );
    assert!(lw < lo, "{lw} against {lo}");
    assert_eq!(lo, lf);
    // The extraction is a share of last year's GDP: it falls only the year after.
    let _ = (tw, to);
    for s in [&mut with, &mut without] {
        s.sim.run_until(Date::new(1926, 3, 1));
    }
    let (rw2, lw2, tw2) = held(with.world());
    let (ro2, lo2, to2) = held(without.world());
    println!(
        "March 1926: regions {} and {}; levels with {lw2}, without {lo2}; taken last week \
         with {:.3}, without {:.3}",
        rw2.len(),
        ro2.len(),
        tw2.to_f64_lossy(),
        to2.to_f64_lossy()
    );
    if rw2 == ro2 && !rw2.is_empty() {
        assert!(
            tw2 < to2,
            "the occupier takes less from burned land the year after"
        );
    }
}

/// The order is withdrawn by the same command.
#[test]
fn the_order_is_withdrawn() {
    let d = data(true);
    let mut s = at_war(&d, true);
    s.submit(GameCommand::SetScorchedEarth {
        tag: "HUN".into(),
        enemy: "CZE".into(),
        on: false,
    });
    s.sim.step();
    assert!(s.world().war.scorch.orders.is_empty());
}
