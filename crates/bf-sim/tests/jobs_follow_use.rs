//! Jobs follow use behind its switch (`game::Switches::jobs_follow_use`): off by default; on, a
//! short run lays off part of the crews of the plants that sell less than they could make, so
//! the path switched on keeps compiling and running in the main.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{Session, Switches};
use laplace_schelling::Date;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn employed(switches: Switches) -> (i64, laplace_schelling::StateHash) {
    let d = data();
    let mut rules = d.rules();
    rules.switches = switches;
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(1925, 4, 2));
    let jobs = s
        .world()
        .econ
        .regions
        .iter()
        .flat_map(|r| r.pops.iter())
        .map(|p| p.employed)
        .sum();
    (jobs, s.state_hash())
}

#[test]
fn off_by_default_and_on_it_lays_off_the_idle_crews() {
    assert!(!data().rules().switches.jobs_follow_use, "ships off");
    let (off, off_hash) = employed(Switches::default());
    let (on, on_hash) = employed(Switches {
        jobs_follow_use: true,
        ..Switches::default()
    });
    assert_ne!(off_hash, on_hash);
    assert!(on < off, "on {on}, off {off}");
}

#[test]
fn switches_parse_by_name() {
    assert_eq!(Switches::parse("").unwrap(), Switches::default());
    assert!(
        Switches::parse(" jobs_follow_use ")
            .unwrap()
            .jobs_follow_use
    );
    assert!(Switches::parse("no_such_rule").is_err());
}
