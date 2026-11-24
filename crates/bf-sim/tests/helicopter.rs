//! The technology `helicopter` (switch `helicopter`, off): with it the helicopter units need it
//! (1936/1942, ASME landmark 95, p. 2) and air assault needs it with `gas_turbine`; off, the
//! tree is main's and the helicopter units need `gas_turbine`. The ignored test prints who
//! reaches it, and when, against `gas_turbine`.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::Session;
use bf_sim::research::Stage;
use laplace_schelling::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_helicopter_units_need_what_the_switch_says() {
    for (rules, helicopter) in [("", false), ("helicopter", true)] {
        let d = GameData::load_with(&root(), rules).unwrap_or_else(|e| panic!("{e}"));
        let tree = d.tree.as_ref().unwrap();
        assert_eq!(tree.tech("helicopter").is_some(), helicopter);
        let mil = &d.war.as_ref().unwrap().military;
        let key = |i: u16| tree.techs[usize::from(i)].key.clone();
        let heli = mil
            .types
            .iter()
            .position(|t| t.key == "helicopter")
            .unwrap();
        let first: Vec<String> = mil.types[heli].generations[0]
            .requires
            .iter()
            .map(|&i| key(i))
            .collect();
        let airborne = mil.types.iter().position(|t| t.key == "airborne").unwrap();
        let assault: Vec<String> = mil.types[airborne].generations[1]
            .requires
            .iter()
            .map(|&i| key(i))
            .collect();
        println!("{rules:?}: helicopter needs {first:?}; air assault needs {assault:?}");
        if helicopter {
            assert!(first.contains(&"helicopter".to_owned()));
            assert!(assault.contains(&"helicopter".to_owned()));
            assert!(assault.contains(&"gas_turbine".to_owned()));
        } else {
            assert!(first.contains(&"gas_turbine".to_owned()));
            assert!(!assault.iter().any(|k| k == "helicopter"));
        }
    }
}

/// Who reaches the helicopter, and when, against the gas turbine that stood in for it.
#[test]
#[ignore = "45 years; run in the slot"]
fn who_flies_helicopters_and_when() {
    let d = GameData::load_with(&root(), "helicopter").unwrap_or_else(|e| panic!("{e}"));
    let tree = d.tree.clone().unwrap();
    let (heli, turbine) = (
        tree.tech("helicopter").unwrap(),
        tree.tech("gas_turbine").unwrap(),
    );
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    let n = s.world().countries.len();
    let mut seen: Vec<[Option<i32>; 2]> = vec![[None; 2]; n];
    for year in 1926..=1970 {
        s.sim.run_until(Date::new(year, 1, 1));
        let w = s.world();
        for c in 0..n {
            for (k, t) in [heli, turbine].into_iter().enumerate() {
                if seen[c][k].is_none() && w.research.stage[c][t] >= Stage::Industrial {
                    seen[c][k] = Some(year - 1);
                }
            }
        }
    }
    let w = s.world();
    println!(
        "first anywhere: helicopter {:?}, gas_turbine {:?} (stages: science, prototype, \
         industrial, diffused...)",
        w.research.firsts[heli], w.research.firsts[turbine]
    );
    let mut rows: Vec<(i32, String, Option<i32>)> = w
        .countries
        .iter()
        .enumerate()
        .filter_map(|(c, (_, x))| seen[c][0].map(|y| (y, x.tag.clone(), seen[c][1])))
        .collect();
    rows.sort();
    for (y, tag, turbine) in &rows {
        println!("{tag}: helicopter industrial {y}, gas turbine {turbine:?}");
    }
    println!(
        "{} countries industrial in the helicopter by 1970",
        rows.len()
    );
}
