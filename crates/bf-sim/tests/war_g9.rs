//! G9 (the owner, 2026-11-19: (b)): the people under the enemy count in proportion to the
//! enemy's advance over each state's depth, before the state changes hands; the threshold of
//! 500 per mille with the capital taken stays. Against the rule before (all at once when the
//! state changes hands): the week the government falls, the people under the enemy week by
//! week, and how long the war lasts, for a small country with a shallow capital and a large one.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use laplace_schelling::{Date, Fixed};

fn load(by_advance: bool, treaties: bool) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    d.switches.treaties_in_force = treaties;
    let c = d.war.as_mut().unwrap().conduct.as_mut().unwrap();
    c.support.occupation_by_advance = by_advance;
    d
}

/// The people of `m`'s home regions under the enemy, per mille, as the support reads them.
fn under(d: &GameData, w: &GameWorld, m: u16, by_advance: bool) -> i64 {
    let rules = d.rules();
    let depth = &rules.fight.as_ref().unwrap().depth_km;
    let f = &w.war.fighting;
    let (mut size, mut taken) = (0i64, 0i64);
    for (ri, r) in w.econ.regions.iter().enumerate() {
        if r.market != m || r.colony {
            continue;
        }
        let p = r.population();
        size += p;
        if f.controller(ri as u32, m) != m {
            taken += p;
        } else if by_advance {
            let km = f
                .advance
                .iter()
                .filter(|(_, into, _)| *into == ri as u32)
                .map(|(_, _, km)| *km)
                .max()
                .unwrap_or(Fixed::ZERO);
            let dd = depth.get(&r.state).copied().unwrap_or(Fixed::from_int(100));
            if km.is_positive() {
                taken += (Fixed::from_int(p) * (km / dd).min(Fixed::ONE)).to_int_floor();
            }
        }
    }
    if size > 0 { taken * 1000 / size } else { 0 }
}

/// (treaties, by advance) → (the week the war ended, the declarer's people under the enemy each
/// week, per mille).
type Runs = Vec<((bool, bool), Option<u32>, Vec<i64>)>;

fn conference(by: &str, target: &str, weeks: u32) -> Runs {
    let mut runs = Vec::new();
    for (treaties, by_advance) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut ended = None;
        let mut series = Vec::new();
        let d = load(by_advance, treaties);
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        s.submit(GameCommand::ChoosePlayer { tag: by.into() });
        s.sim.run_until(Date::new(1925, 3, 2));
        s.submit(GameCommand::Demand {
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
        });
        s.sim.run_until(Date::new(1925, 3, 20));
        let id = s.world().war.conflicts.crises[0].id;
        s.submit(GameCommand::Declare {
            tag: by.into(),
            crisis: id,
            war: true,
        });
        let idx = |w: &GameWorld, t: &str| {
            w.countries.iter().position(|(_, c)| c.tag == t).unwrap() as u16
        };
        let (a, b) = (idx(s.world(), by), idx(s.world(), target));
        eprintln!("G9 ==== {by} x {target}, treaties {treaties}, by advance {by_advance}");
        eprintln!(
            "G9 week | under the enemy {by} {target} (per mille) | support {by} {target} | fallen"
        );
        let start = Date::new(1925, 3, 22).to_days();
        for n in 1..=weeks {
            s.sim.run_until(Date::from_days(start + 7 * i64::from(n)));
            let w = s.world();
            let sup = |m: u16| {
                w.war
                    .fighting
                    .support_of(m)
                    .map(|x| (x.value, x.fallen.is_some()))
            };
            eprintln!(
                "G9 {n} | {} {} | {:?} {:?} | wars {}",
                under(&d, w, a, by_advance),
                under(&d, w, b, by_advance),
                sup(a),
                sup(b),
                w.war.conflicts.wars.len()
            );
            let fell = |m: u16| {
                w.war
                    .fighting
                    .support_of(m)
                    .is_some_and(|x| x.fallen.is_some())
            };
            if fell(a) || fell(b) {
                eprintln!(
                    "G9 fallen in week {n}: {by} {} {target} {}",
                    fell(a),
                    fell(b)
                );
            }
            if w.war.conflicts.wars.is_empty() {
                eprintln!("G9 ended in week {n}");
                ended = Some(n);
                break;
            }
            series.push(under(&d, w, a, by_advance));
        }
        runs.push(((treaties, by_advance), ended, series));
    }
    runs
}

fn run(runs: &Runs, treaties: bool, by_advance: bool) -> (Option<u32>, &[i64]) {
    let r = runs
        .iter()
        .find(|(k, _, _)| *k == (treaties, by_advance))
        .unwrap();
    (r.1, &r.2)
}

/// Hungary against Czechoslovakia: the shallow capital falls in the first weeks either way; with
/// the advance counted, the government falls no later.
#[test]
fn g9_hungary_czechoslovakia() {
    let runs = conference("HUN", "CZE", 26);
    let (cohort, _) = run(&runs, false, false);
    let (gradual, _) = run(&runs, false, true);
    assert!(gradual.unwrap() <= cohort.unwrap());
    assert!(gradual.unwrap() <= 2, "{gradual:?}");
}

/// Germany against Poland, alone: the people under the enemy show from the first week, before
/// any state changes hands, and grow; the government falls by the threshold earlier than when a
/// state counted only once taken (week 9 against 12, measured 2026-11-19).
#[test]
fn g9_germany_poland() {
    let runs = conference("GER", "POL", 52);
    let (cohort, before) = run(&runs, false, false);
    let (gradual, series) = run(&runs, false, true);
    assert_eq!(before[0], 0, "all at once: nothing before a state falls");
    assert!(series[0] > 0, "the advance counts from the first week");
    assert!(series.windows(2).all(|p| p[1] >= p[0]), "{series:?}");
    assert!(gradual.unwrap() < cohort.unwrap(), "{gradual:?} {cohort:?}");
}
