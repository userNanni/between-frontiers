//! Invariants of the 1925 land data: `data/scenario/1925/land.ron` against the `LandFile`
//! contract and the states of `assets/map/provinces.ron`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::resources::{CLIMATES, FARMS, LandFile};
use bf_sim::scenario::MapFile;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn land() -> LandFile {
    read("data/scenario/1925/land.ron")
}

fn map() -> MapFile {
    read("assets/map/provinces.ron")
}

#[test]
fn every_state_once() {
    let land = land();
    let map = map();
    let expected: BTreeSet<&str> = map.states.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(expected.len(), 870);
    let mut seen = BTreeSet::new();
    for s in &land.states {
        assert!(seen.insert(s.state.as_str()), "{}: listed twice", s.state);
        assert!(
            expected.contains(s.state.as_str()),
            "{}: not a state of the map",
            s.state
        );
    }
    let missing: Vec<_> = expected.difference(&seen).collect();
    assert!(missing.is_empty(), "states without land data: {missing:?}");
}

#[test]
fn climate_shares_are_valid_and_sum_to_one() {
    for s in &land().states {
        assert!(!s.climate.is_empty(), "{}: no climate", s.state);
        let mut keys = BTreeSet::new();
        for (k, v) in &s.climate {
            assert!(
                CLIMATES.contains(&k.as_str()),
                "{}: unknown climate {k}",
                s.state
            );
            assert!(keys.insert(k.as_str()), "{}: climate {k} twice", s.state);
            assert!(*v > 0.0 && *v <= 1.0, "{}: climate {k} = {v}", s.state);
        }
        let sum: f32 = s.climate.iter().map(|(_, v)| v).sum();
        assert!(
            (sum - 1.0).abs() <= 0.01,
            "{}: climate shares sum to {sum}",
            s.state
        );
    }
}

#[test]
fn farms_are_valid() {
    for s in &land().states {
        let mut keys = BTreeSet::new();
        for (k, v) in &s.farms {
            assert!(FARMS.contains(&k.as_str()), "{}: unknown farm {k}", s.state);
            assert!(keys.insert(k.as_str()), "{}: farm {k} twice", s.state);
            assert!((0.0..=1.0).contains(v), "{}: farm {k} = {v}", s.state);
        }
        if !s.coast {
            assert!(
                !keys.contains("fishery"),
                "{}: fishery without a coast",
                s.state
            );
        }
    }
}

#[test]
fn areas_fit_the_state() {
    let areas: BTreeMap<String, f32> = map()
        .states
        .into_iter()
        .map(|s| (s.key, s.area_km2))
        .collect();
    for s in &land().states {
        for (name, v) in [
            ("cropland", s.cropland_km2),
            ("pasture", s.pasture_km2),
            ("forest", s.forest_km2),
        ] {
            assert!(v.is_finite() && v >= 0.0, "{}: {name} = {v}", s.state);
        }
        let area = areas[&s.state];
        assert!(
            s.cropland_km2 + s.pasture_km2 <= area * 1.05,
            "{}: cropland {} + pasture {} > area {area}",
            s.state,
            s.cropland_km2,
            s.pasture_km2
        );
        assert!(
            s.forest_km2 <= area * 1.05,
            "{}: forest {} > area {area}",
            s.state,
            s.forest_km2
        );
    }
}

#[test]
fn world_totals_are_plausible_for_1925() {
    // HYDE 3.3 gives about 10.7 million km² of cropland and 19.3 of grazing land in 1925.
    let land = land();
    let crop: f64 = land.states.iter().map(|s| f64::from(s.cropland_km2)).sum();
    let pasture: f64 = land.states.iter().map(|s| f64::from(s.pasture_km2)).sum();
    let forest: f64 = land.states.iter().map(|s| f64::from(s.forest_km2)).sum();
    assert!((9.5e6..12.0e6).contains(&crop), "cropland {crop}");
    assert!((17.0e6..22.0e6).contains(&pasture), "pasture {pasture}");
    assert!((35.0e6..50.0e6).contains(&forest), "forest {forest}");
}

#[test]
fn known_states_look_right() {
    let land = land();
    let by_key: BTreeMap<&str, _> = land.states.iter().map(|s| (s.state.as_str(), s)).collect();
    let farm = |state: &str, kind: &str| -> f32 {
        by_key[state]
            .farms
            .iter()
            .find(|(k, _)| k == kind)
            .map_or(0.0, |(_, v)| *v)
    };
    let climate = |state: &str, kind: &str| -> f32 {
        by_key[state]
            .climate
            .iter()
            .find(|(k, _)| k == kind)
            .map_or(0.0, |(_, v)| *v)
    };
    assert!(climate("USA-IA", "humid_continental") > 0.9);
    assert!(climate("EGY-UPPER", "hot_desert") > 0.9);
    assert!(climate("MYS-FMS", "tropical_rainforest") > 0.9);
    assert!(climate("GBR-LONDON", "oceanic") > 0.9);
    assert!(by_key["PRT-LISBOA"].coast && !by_key["CHE-ZURICH"].coast && !by_key["USA-MI"].coast);
    assert!(by_key["RUS-AST"].coast, "the Caspian counts as a sea");
    assert!(farm("BRA-SP", "coffee_plantation") >= 0.9);
    assert!(farm("GHA-ASHANTI", "cocoa_plantation") >= 0.9);
    assert!(farm("EGY-WESTERN-DELTA", "cotton_plantation") >= 0.9);
    assert!(farm("IND-ASSAM", "tea_plantation") >= 0.9);
    assert!(farm("USA-IA", "maize_farm") >= 0.9);
    assert!(farm("NOR-VESTLANDET", "fishery") >= 0.9);
    assert!(farm("RUS-SA", "forestry") >= 0.9);
    assert_eq!(farm("SWE-NORRLAND", "coffee_plantation"), 0.0);
    assert_eq!(farm("CHE-ZURICH", "fishery"), 0.0);
}
