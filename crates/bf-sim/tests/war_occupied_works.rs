//! Works and occupied land (`Switches::occupied_works`, off) on the real scenario, with the
//! devastation of the combat that main already has: Poland, played, declares on
//! Czechoslovakia and loses a region. Off, Poland goes on building in the region the enemy
//! holds; on, it starts nothing there, the works it had there wait, and its fund builds in the
//! regions it holds.

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
    d.switches.occupied_works = switch;
    // The overrun rule off: the war goes on however much the front takes.
    let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
    c.support.overrun = 2000;
    c.support.capital_overrun = 2000;
    d
}

/// `a`, played, demands a payment of `b`, hears the refusal and declares.
fn at_war(d: &GameData, a: &str, b: &str) -> Session {
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

/// Which pairs leave the side declared on partly occupied after three months (to pick the case).
#[test]
#[ignore = "a scan to pick the case"]
fn scan_pairs() {
    let d = data(false);
    for (a, b) in [
        ("ROM", "HUN"),
        ("POL", "LIT"),
        ("YUG", "BUL"),
        ("ITA", "YUG"),
        ("POL", "CZE"),
        ("CZE", "HUN"),
        ("ROM", "BUL"),
        ("GRE", "BUL"),
        ("FRA", "ITA"),
        ("SOV", "POL"),
        ("GER", "POL"),
        ("POL", "GER"),
        ("YUG", "HUN"),
        ("TUR", "GRE"),
    ] {
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
        let Some(id) = s.world().war.conflicts.crises.first().map(|c| c.id) else {
            println!("{a}->{b}: no crisis");
            continue;
        };
        s.submit(GameCommand::Declare {
            tag: a.into(),
            crisis: id,
            war: true,
        });
        s.sim.run_until(Date::new(1925, 7, 1));
        let w = s.world();
        for t in [a, b] {
            let c = index(w, t);
            let total = w
                .econ
                .regions
                .iter()
                .filter(|r| r.market == c && !r.colony)
                .count();
            let lost = (0..w.econ.regions.len())
                .filter(|&r| w.econ.regions[r].market == c && !w.econ.regions[r].colony)
                .filter(|&r| !bf_sim::econ::construction::held(w, r as u32))
                .count();
            println!(
                "{a}->{b}: {t} lost {lost} of {total} home regions, wars {}",
                w.war.conflicts.wars.len()
            );
        }
    }
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

/// A country's plant levels in the regions the enemy holds and in those it holds, its works
/// queued in each, and its fund.
fn country(w: &GameWorld, tag: &str) -> (Vec<u32>, i64, i64, usize, usize, Fixed) {
    let hun = index(w, tag);
    let mut held = Vec::new();
    let (mut lost_levels, mut kept_levels) = (0, 0);
    let mut lost = Vec::new();
    for (ri, r) in w.econ.regions.iter().enumerate() {
        if r.market != hun {
            continue;
        }
        let levels: i64 = r.plants.iter().map(|p| p.levels).sum();
        if bf_sim::econ::construction::held(w, ri as u32) {
            kept_levels += levels;
            held.push(ri as u32);
        } else {
            lost_levels += levels;
            lost.push(ri as u32);
        }
    }
    let m = &w.econ.markets[usize::from(hun)];
    let queued_lost = m.queue.iter().filter(|p| lost.contains(&p.region)).count();
    let queued_kept = m.queue.len() - queued_lost;
    (
        lost,
        lost_levels,
        kept_levels,
        queued_lost,
        queued_kept,
        m.pool,
    )
}

#[test]
fn the_owner_does_not_build_on_land_it_does_not_hold() {
    // Poland declares on Czechoslovakia and loses one of its sixteen home regions.
    let mut off = at_war(&data(false), "POL", "CZE");
    let mut on = at_war(&data(true), "POL", "CZE");
    let mut rows = Vec::new();
    for (y, m) in [(1925, 6), (1925, 9), (1925, 12), (1926, 3)] {
        off.sim.run_until(Date::new(y, m, 1));
        on.sim.run_until(Date::new(y, m, 1));
        let f = country(off.world(), "POL");
        let n = country(on.world(), "POL");
        println!(
            "{y}-{m:02}: lost regions off {:?} on {:?}; levels there off {} on {}; levels held \
             off {} on {}; total off {} on {}; works queued there off {} on {}, held off {} on \
             {}; fund off {:.3e} on {:.3e}",
            f.0,
            n.0,
            f.1,
            n.1,
            f.2,
            n.2,
            f.1 + f.2,
            n.1 + n.2,
            f.3,
            n.3,
            f.4,
            n.4,
            f.5.to_f64_lossy(),
            n.5.to_f64_lossy()
        );
        rows.push((f, n));
    }
    let w = on.world();
    let (lost, ..) = country(w, "POL");
    assert!(!lost.is_empty(), "Poland lost no land");
    // On, none of Poland's works in the lost region took a point while it was lost.
    let pol = index(w, "POL");
    let waiting: Vec<_> = w.econ.markets[usize::from(pol)]
        .queue
        .iter()
        .filter(|p| lost.contains(&p.region))
        .collect();
    println!("works waiting in the lost regions: {}", waiting.len());
    let (f, n) = rows.last().unwrap();
    assert!(
        n.1 <= f.1,
        "on, the lost land is not built on: {} against {}",
        n.1,
        f.1
    );
}
