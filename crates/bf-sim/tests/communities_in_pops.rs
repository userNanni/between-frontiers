//! Communities in the population: the rows of every profession add up to its pop after every
//! demography month, assimilation included (`docs/population.pt-BR.md`, "Comunidades na
//! simulação").

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::prof;
use bf_sim::game::Session;
use laplace_schelling::calendar::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn rows_add_up_to_the_pops_every_month() {
    let data = GameData::load(&root()).unwrap();
    assert!(data.community_table.is_some(), "communities data");
    let mut s = Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    let mut checked = 0;
    let mut last = 0;
    while s.clock().date < Date::new(1926, 2, 1) {
        s.sim.step();
        let w = s.sim.world();
        let month = s.clock().date.year * 12 + i32::from(s.clock().date.month);
        // Right after the demography of the 15th.
        if s.clock().date.day != 16 || month == last {
            continue;
        }
        last = month;
        let rows = &w.demography.communities;
        assert_eq!(rows.len(), w.econ.regions.len());
        let mut linked = 0;
        for (r, rr) in w.econ.regions.iter().zip(rows) {
            if rr.ids.is_empty() {
                continue;
            }
            linked += 1;
            for p in 0..prof::P {
                let sum: i64 = rr.people.iter().map(|row| row[p]).sum();
                assert_eq!(sum, r.pops[p].size, "state {} profession {p}", r.state);
            }
            if let Some(k) = rr.ids.iter().position(|&id| id == u16::MAX) {
                let others: i64 = rr.others.iter().map(|e| e.1).sum();
                assert_eq!(
                    others,
                    rr.people[k].iter().sum::<i64>(),
                    "state {}",
                    r.state
                );
            }
            assert!(rr.people.iter().flatten().all(|&v| v >= 0));
        }
        assert!(linked > 800, "{linked} states with communities");
        checked += 1;
    }
    assert!(checked >= 13, "{checked} months checked");
}
