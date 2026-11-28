//! The method the government fixes for a building kind (`econ::industry`,
//! `Switches::fixed_methods`, owner's decisions M1 and M2 of 2026-11-28) on the real scenario: a
//! country whose ownership law allows it fixes the oldest method of a kind its plants have left;
//! by the second January the plants of the kind run it, and new plants are built with it. A
//! laissez-faire country's command is refused. With the switch off the fixed method is stored
//! and does nothing.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use laplace_schelling::Date;

fn data(on: bool) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.switches.fixed_methods = on;
    d
}

fn session(d: &GameData) -> Session {
    Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    )
}

/// Levels of `kind` in market `mi`, and how many of them run method `method`.
fn levels(w: &GameWorld, mi: usize, kind: u16, method: u8) -> (i64, i64) {
    w.econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == mi)
        .flat_map(|r| r.plants.iter())
        .filter(|p| p.kind == kind && p.levels > 0)
        .fold((0, 0), |(all, on), p| {
            (
                all + p.levels,
                on + i64::from(p.method == method) * p.levels,
            )
        })
}

/// A country that may fix, and a kind most of whose levels there run a method other than the
/// first: (market, tag, kind).
fn case(d: &GameData, w: &GameWorld) -> (usize, String, u16) {
    for (mi, m) in w.econ.markets.iter().enumerate() {
        if !bf_sim::econ::industry::may_fix(&m.laws.effects[0]) {
            continue;
        }
        for (kind, b) in d.catalogue.buildings.iter().enumerate() {
            if b.public || b.informal || b.traditional || b.methods.len() < 2 {
                continue;
            }
            let (all, first) = levels(w, mi, kind as u16, 0);
            // The first method's inputs are within reach where some level already runs it.
            if all >= 20 && first * 4 < all && first > 0 {
                let tag = w.countries.iter().nth(mi).unwrap().1.tag.clone();
                return (mi, tag, kind as u16);
            }
        }
    }
    panic!("no country with state ownership has a kind that left its first method");
}

#[test]
fn the_plants_move_to_the_fixed_method_and_laissez_faire_may_not_fix() {
    let d = data(true);
    let mut s = session(&d);
    s.sim.step();
    let (mi, tag, kind) = case(&d, s.world());
    let before = levels(s.world(), mi, kind, 0);
    s.submit(GameCommand::SetMethod {
        tag: tag.clone(),
        kind,
        method: Some(0),
    });
    s.sim.step();
    assert_eq!(s.world().industry.fixed(mi, kind), Some(0));
    // A laissez-faire country may fix nothing.
    let lf = s
        .world()
        .econ
        .markets
        .iter()
        .position(|m| !bf_sim::econ::industry::may_fix(&m.laws.effects[0]))
        .expect("a laissez-faire country in 1925");
    let lf_tag = s.world().countries.iter().nth(lf).unwrap().1.tag.clone();
    s.submit(GameCommand::SetMethod {
        tag: lf_tag,
        kind,
        method: Some(0),
    });
    s.sim.step();
    assert_eq!(s.world().industry.fixed(lf, kind), None);
    s.sim.run_until(Date::new(1927, 1, 8));
    let after = levels(s.world(), mi, kind, 0);
    assert!(
        after.1 * 2 > after.0,
        "most levels of {kind} in {tag} should run the fixed method: {before:?} -> {after:?}"
    );
    // Handing back to the market leaves the world as if nothing had been fixed.
    s.submit(GameCommand::SetMethod {
        tag,
        kind,
        method: None,
    });
    s.sim.step();
    assert!(s.world().industry.is_empty());
}

#[test]
fn off_the_fixed_method_is_kept_and_does_nothing() {
    let d = data(false);
    let mut fixed = session(&d);
    let mut plain = session(&d);
    fixed.sim.step();
    plain.sim.step();
    let (mi, tag, kind) = case(&d, fixed.world());
    fixed.submit(GameCommand::SetMethod {
        tag,
        kind,
        method: Some(0),
    });
    fixed.sim.run_until(Date::new(1926, 1, 15));
    plain.sim.run_until(Date::new(1926, 1, 15));
    assert_eq!(fixed.world().industry.fixed(mi, kind), Some(0));
    assert_eq!(
        levels(fixed.world(), mi, kind, 0),
        levels(plain.world(), mi, kind, 0)
    );
    assert_eq!(
        fixed.world().econ.regions.len(),
        plain.world().econ.regions.len()
    );
}
