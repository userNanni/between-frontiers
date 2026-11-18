//! The thread count never changes the world: two years with every rule off and one year with
//! every switch on give the same state hash on 1 and on 12 threads. Parallel work collects in
//! index order and sums in id order; this test is what catches a sum that does not.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{Session, Switches};
use laplace_schelling::Date;
use laplace_schelling::hash::StateHash;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(data: &GameData, threads: usize, switches: Switches, until: Date) -> StateHash {
    let mut rules = data.rules();
    rules.switches = switches;
    let mut s = Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        threads,
        Some(rules),
    );
    s.sim.run_until(until);
    s.state_hash()
}

#[test]
fn one_and_twelve_threads_give_the_same_world() {
    let data = GameData::load(&root()).unwrap();
    let off = Switches::default();
    let end = Date::new(1927, 1, 1);
    assert_eq!(
        run(&data, 1, off, end),
        run(&data, 12, off, end),
        "rules off"
    );
    let on = Switches::parse(&Switches::NAMES.join(",")).unwrap();
    let end = Date::new(1926, 1, 1);
    assert_eq!(
        run(&data, 1, on, end),
        run(&data, 12, on, end),
        "every switch on"
    );
}
