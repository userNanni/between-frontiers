//! Fronts drawn by the player and divisions sent one by one (switch `fronts`; the owner's FR1
//! to FR11, 2026-11-28), on the real scenario: the Soviet Union, played, declares on Poland, and
//! Poland's allies join it. The overrun and collapse rules are off so the war goes on.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use bf_sim::war::fronts::{CountryFronts, Place, land_kinds};
use laplace_schelling::{Date, Fixed};

fn data(fronts: bool) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    d.switches.fronts = fronts;
    if let Some(c) = d.war.as_mut().and_then(|w| w.conduct.as_mut()) {
        c.support.overrun = 2000;
        c.support.capital_overrun = 2000;
        c.support.collapse = 0;
        c.support.peace = 0;
    }
    d
}

const ME: &str = "SOV";
const FOE: &str = "POL";

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

/// The Soviet Union, played, demands Poland whole, hears the refusal and declares.
fn at_war(d: &GameData) -> Session {
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: ME.into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: ME.into(),
        target: FOE.into(),
        demand: Proposal {
            side_a: vec![ME.into()],
            side_b: vec![FOE.into()],
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
        tag: ME.into(),
        crisis: id,
        war: true,
    });
    s.sim.run_until(Date::new(1925, 4, 1));
    s
}

fn fronts(w: &GameWorld) -> &CountryFronts {
    let me = index(w, ME);
    w.war.fronts.of(me).expect("the player's fronts")
}

/// At the start of the war the game lays down a front for each enemy in contact, covering the
/// whole line, and every division stands on one (FR8's default: no one starts with the line
/// empty). The divisions' elements are the formations', kind by kind.
#[test]
fn the_war_starts_with_a_front_for_each_enemy() {
    let d = data(true);
    let s = at_war(&d);
    let w = s.world();
    let cf = fronts(w);
    assert!(cf.fronts.len() >= 2, "{:?}", cf.fronts);
    assert!(cf.uncovered.is_empty(), "{:?}", cf.uncovered);
    // Every division of the first week on a front; those called up after it in the reserve.
    assert!(
        cf.divisions.iter().all(|d| d.place != Place::Idle),
        "no division left idle: {:?}",
        cf.divisions
            .iter()
            .filter(|d| d.place == Place::Idle)
            .collect::<Vec<_>>()
    );
    assert!(
        cf.divisions
            .iter()
            .filter(|d| !d.kind.called_up)
            .all(|d| matches!(d.place, Place::Front(_)))
    );
    let mil = &d.war.as_ref().unwrap().military;
    let kinds = land_kinds(mil, &w.war.countries[index(w, ME) as usize]);
    for (k, el) in kinds {
        let sum: Fixed = cf
            .divisions
            .iter()
            .filter(|x| x.kind == k)
            .map(|x| x.elements)
            .sum();
        // The fixed point's rounding of the week's losses, made good the next week.
        assert!(
            (sum - el).abs() <= Fixed::from_ratio(1, 10),
            "{k:?}: {sum:?} {el:?}"
        );
    }
}

/// A front deleted leaves its stretch empty (FR8): its divisions stand idle, the screen counts the
/// km left open, and the enemy walks in faster than against the front.
#[test]
fn a_stretch_left_open_is_empty_and_the_enemy_walks_in() {
    let d = data(true);
    let taken = |delete: bool| -> (usize, u32) {
        let mut s = at_war(&d);
        let (me, foe) = (index(s.world(), ME), index(s.world(), FOE));
        if delete {
            let id = fronts(s.world())
                .fronts
                .iter()
                .find(|f| f.enemy == foe)
                .map(|f| f.id)
                .unwrap();
            s.submit(GameCommand::DeleteFront {
                tag: ME.into(),
                front: id,
            });
        }
        s.sim.run_until(Date::new(1925, 4, 14));
        let w = s.world();
        let open_km: u32 = fronts(w).uncovered.iter().map(|(_, km)| *km).sum();
        let lost = w
            .war
            .fighting
            .control
            .iter()
            .filter(|(r, by)| w.econ.regions[*r as usize].market == me && *by == foe)
            .count();
        (lost, open_km)
    };
    let (held, _) = taken(false);
    let (open, km) = taken(true);
    assert!(km > 0, "the km left open are counted");
    assert!(
        open > held,
        "{open} states lost with the stretch open, {held} with the front"
    );
}

/// A division sent to another front spends weeks in transit (FR10) and does not fight on the way.
#[test]
fn a_division_sent_spends_weeks_in_transit() {
    let d = data(true);
    let mut s = at_war(&d);
    let cf = fronts(s.world()).clone();
    let (a, b) = (cf.fronts[0].id, cf.fronts[cf.fronts.len() - 1].id);
    let div = cf
        .divisions
        .iter()
        .find(|x| x.place == Place::Front(a))
        .unwrap()
        .id;
    s.submit(GameCommand::SendDivisions {
        tag: ME.into(),
        divisions: vec![div],
        to: Place::Front(b),
    });
    // The next week's step works out the weeks in transit; the division is on the way.
    s.sim
        .run_until(Date::from_days(s.clock().date.to_days() + 7));
    let x = fronts(s.world())
        .divisions
        .iter()
        .find(|x| x.id == div)
        .unwrap()
        .clone();
    assert_eq!(x.place, Place::Front(b));
    assert!(x.transit >= 1 && x.transit < u16::MAX, "{x:?}");
}

/// The men an order raises during the war are new divisions in the reserve (FR11), and from there
/// go to the fronts; none is left idle.
#[test]
fn new_divisions_go_to_the_reserve() {
    let d = data(true);
    let mut s = at_war(&d);
    // A formation raised in the war (as an order delivers it): 30 elements of infantry.
    {
        let w = s.sim.world_mut();
        let me = index(w, ME) as usize;
        let cw = &mut w.war.countries[me];
        let mut f = cw.formations[0].clone();
        f.raised = true;
        f.elements = Fixed::from_int(30);
        cw.formations.push(f);
    }
    s.sim
        .run_until(Date::from_days(s.clock().date.to_days() + 7));
    let cf = fronts(s.world());
    let raised: Vec<_> = cf.divisions.iter().filter(|x| x.kind.raised).collect();
    assert!(!raised.is_empty(), "the men raised are divisions");
    assert!(raised.iter().all(|x| x.place != Place::Idle), "{raised:?}");
}

/// The AI keeps the rule of before (FR6): with no player, the switch on changes nothing.
#[test]
fn without_a_player_the_switch_changes_nothing() {
    let run = |fronts: bool| {
        let d = data(fronts);
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        s.sim.run_until(Date::new(1925, 9, 1));
        s.state_hash()
    };
    assert_eq!(run(false), run(true));
}

/// The cost of the fronts with some 600 divisions (the orchestrator's measure): the Soviet Union's
/// land army of 1925 cut into divisions of one element, half a year of war against Poland and
/// its allies, the forces system by the clock in each system, with the switch on and off. Run
/// alone, in the bench slot:
/// `bf-bench-slot bf-cargo test --release -p bf-sim --test war_fronts -- --ignored cost --nocapture`.
#[test]
#[ignore]
fn the_cost_of_six_hundred_divisions() {
    let run = |on: bool| -> (f64, f64, usize) {
        let mut d = data(on);
        if let Some(f) = d
            .war
            .as_mut()
            .and_then(|w| w.conduct.as_mut())
            .and_then(|c| c.fronts.as_mut())
        {
            f.division_elements = 1;
        }
        let mut s = at_war(&d);
        let before = s.sim.schedule().stats();
        s.sim.run_until(Date::new(1925, 10, 1));
        let after = s.sim.schedule().stats();
        let spent = |name: &str| -> f64 {
            let t = |v: &[laplace_schelling::schedule::SystemStats]| {
                v.iter()
                    .find(|x| x.name == name)
                    .map_or(0.0, |x| (x.compute + x.apply).as_secs_f64())
            };
            t(&after) - t(&before)
        };
        let total: f64 = after.iter().map(|x| spent(x.name)).sum();
        let n = s
            .world()
            .war
            .fronts
            .countries
            .first()
            .map_or(0, |c| c.divisions.len());
        (spent("forces") * 1000.0, total * 1000.0, n)
    };
    let (off, off_total, _) = run(false);
    let (on, on_total, n) = run(true);
    println!(
        "half a year at war: forces {off:.1} ms off ({:.2} %), {on:.1} ms on with {n} divisions ({:.2} %); per week {:.3} ms more",
        off / off_total * 100.0,
        on / on_total * 100.0,
        (on - off) / 26.0
    );
    assert!(n > 400, "{n} divisions");
    assert!(on / on_total < 0.15);
}
