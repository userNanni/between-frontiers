//! The density of the target (`war::target`), against history, measured and not tuned:
//! - one fission warhead of 12.5 kt and one of 22 kt on the game's Japan of 1945, against the
//!   68,000 and 38,000 killed of Hiroshima and Nagasaki (Glasstone and Dolan, *The Effects of
//!   Nuclear Weapons*, 1977, p. 544, table 12.09). The table also gives the core density of the
//!   rule, so this is a reading, not a check;
//! - 100 and 200 equivalent megatons on the game's Soviet Union of 1964, against about 15 % and
//!   21 % of its people (Enthoven and Smith, 1971, *How Much Is Enough?*, p. 207, as the FRUS
//!   1964-68, vol. X, doc. 102, quotes it): the independent check.
//!
//! Both worlds come from one run from 1925 with the switches of `data/rules.ron` (war off), the
//! world of the main series.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameWorld, Session};
use bf_sim::war::target;
use laplace_schelling::Date;

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

fn people(w: &GameWorld, m: u16) -> i64 {
    w.econ
        .regions
        .iter()
        .filter(|r| r.market == m && !r.colony)
        .map(|r| r.population())
        .sum()
}

/// `emt` in equivalent megatons per mille.
fn emt_of_kt(kt: f64) -> u64 {
    ((kt / 1000.0).powf(2.0 / 3.0) * 1000.0).round() as u64
}

#[test]
#[ignore = "long: 1925 to 1964; run in the slot"]
fn the_density_of_the_target_against_history() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    let mut forces = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    forces.enable_forces();
    let n = &forces.war.as_ref().unwrap().military.rules.nuclear;
    let names: BTreeMap<u16, String> = d
        .border_rules
        .state_ids
        .iter()
        .map(|(k, v)| (*v, k.clone()))
        .collect();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    eprintln!(
        "lethal area {} km² an equivalent megaton; core density {} a km²",
        n.lethal_km2_per_emt, n.core_density
    );

    // Hiroshima and Nagasaki.
    s.sim.run_until(Date::new(1945, 8, 1));
    let w = s.world();
    let jpn = index(w, "JAP");
    let cities = target::cities(w, jpn);
    let (pop, urban) = (people(w, jpn), cities.iter().map(|c| c.1).sum::<i64>());
    eprintln!(
        "Japan 1945: {:.1} M people, {:.1} M in cities ({:.0} %), {} cities; largest: {}",
        pop as f64 / 1e6,
        urban as f64 / 1e6,
        urban as f64 * 100.0 / pop as f64,
        cities.len(),
        cities
            .iter()
            .take(3)
            .map(|c| format!(
                "{} {:.2} M",
                names
                    .get(&w.econ.regions[c.0 as usize].state)
                    .cloned()
                    .unwrap_or_default(),
                c.1 as f64 / 1e6
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    for (name, kt, real) in [("Hiroshima", 12.5, 68_000), ("Nagasaki", 22.0, 38_000)] {
        let e = emt_of_kt(kt);
        let area = target::area_of(n.lethal_km2_per_emt, e);
        let p = target::plan(&cities, n.core_density, &[area]);
        let (ci, dead) = p[0];
        eprintln!(
            "{name}: {kt} kt = {e} per mille EMT, {:.2} km²; dead in the game {dead} on a city of {:.2} M ({}), real {real} killed of 256,300 and 173,800 at risk",
            area as f64 / 1000.0,
            cities[ci].1 as f64 / 1e6,
            names
                .get(&w.econ.regions[cities[ci].0 as usize].state)
                .cloned()
                .unwrap_or_default()
        );
        assert!(dead > 0 && dead <= cities[ci].1);
    }

    // The Soviet Union of 1964.
    s.sim.run_until(Date::new(1964, 7, 1));
    let w = s.world();
    let sov = index(w, "SOV");
    let cities = target::cities(w, sov);
    let (pop, urban) = (people(w, sov), cities.iter().map(|c| c.1).sum::<i64>());
    eprintln!(
        "Soviet Union 1964: {:.1} M people, {:.1} M in cities ({:.0} %), {} cities; the rural {:.0} % are outside the target by construction",
        pop as f64 / 1e6,
        urban as f64 / 1e6,
        urban as f64 * 100.0 / pop as f64,
        cities.len(),
        100.0 - urban as f64 * 100.0 / pop as f64
    );
    eprintln!("EMT on the Soviet Union | warheads | dead | share of the people | real (FRUS)");
    for (total, size, real) in [
        (100u64, 1000u64, "15 %"),
        (200, 1000, "21 %"),
        (100, 76, "—"),
        (400, 1000, "—"),
    ] {
        let k = (total * 1000 / size) as usize;
        let area = target::area_of(n.lethal_km2_per_emt, size);
        let dead: i64 = target::plan(&cities, n.core_density, &vec![area; k])
            .iter()
            .map(|x| x.1)
            .sum();
        assert!(dead <= urban);
        eprintln!(
            "{total} (of {:.3} each) | {k} | {:.1} M | {:.1} % | {real}",
            size as f64 / 1000.0,
            dead as f64 / 1e6,
            dead as f64 * 100.0 / pop as f64
        );
    }
    // The unacceptable damage of the balance: a quarter of the people.
    let quarter = pop * i64::from(n.unacceptable_share) / 1000;
    for size in [76u64, 1000, 1251] {
        let area = target::area_of(n.lethal_km2_per_emt, size);
        let k = target::warheads_to_kill(&cities, n.core_density, area, quarter, 100_000);
        eprintln!(
            "warheads of {:.3} EMT to kill a quarter of the Soviet people: {k:?}",
            size as f64 / 1000.0
        );
    }
}
