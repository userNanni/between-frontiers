//! Invariants of the 1925 deposits: `data/scenario/1925/deposits.ron` against the
//! `DepositsFile` contract and the states of `assets/map/provinces.ron`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::resources::{DepositsFile, RESOURCES};
use bf_sim::scenario::MapFile;

const START: i32 = 1925;

/// Resources that the economy needs from day one; each must have this many deposits.
const MIN_DEPOSITS: &[(&str, usize)] = &[
    ("copper", 10),
    ("bauxite", 10),
    ("oil", 10),
    ("coal", 10),
    ("iron_ore", 10),
    ("rare_earths", 3),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn deposits() -> DepositsFile {
    read("data/scenario/1925/deposits.ron")
}

#[test]
fn every_resource_has_one_unit() {
    let file = deposits();
    let mut units = BTreeMap::new();
    for (resource, unit) in &file.units {
        assert!(
            RESOURCES.contains(&resource.as_str()),
            "unit for unknown resource {resource}"
        );
        assert!(!unit.trim().is_empty(), "empty unit for {resource}");
        assert!(
            units.insert(resource.as_str(), unit).is_none(),
            "two units for {resource}"
        );
    }
    for resource in RESOURCES {
        assert!(units.contains_key(resource), "no unit for {resource}");
    }
}

#[test]
fn states_exist_on_the_map_once() {
    let map: MapFile = read("assets/map/provinces.ron");
    let keys: BTreeSet<&str> = map.states.iter().map(|s| s.key.as_str()).collect();
    let file = deposits();
    let mut seen = BTreeSet::new();
    for state in &file.states {
        assert!(
            keys.contains(state.state.as_str()),
            "unknown state {}",
            state.state
        );
        assert!(
            seen.insert(state.state.as_str()),
            "state {} listed twice",
            state.state
        );
        assert!(
            !state.deposits.is_empty(),
            "state {} without deposits",
            state.state
        );
    }
}

#[test]
fn deposits_are_consistent() {
    for state in deposits().states {
        for d in &state.deposits {
            let at = format!("{} {} ({})", state.state, d.resource, d.note);
            assert!(
                RESOURCES.contains(&d.resource.as_str()),
                "unknown resource: {at}"
            );
            assert!(
                (1..=5).contains(&d.size),
                "size {} outside 1..=5: {at}",
                d.size
            );
            assert!(
                (1..=2025).contains(&d.discovered),
                "discovered {}: {at}",
                d.discovered
            );
            assert!(
                d.output_1925.is_finite() && d.output_1925 >= 0.0,
                "output: {at}"
            );
            assert!(d.potential.is_finite(), "potential: {at}");
            assert!(
                d.potential >= d.output_1925,
                "potential below 1925 output: {at}"
            );
            if d.discovered > START {
                assert_eq!(d.output_1925, 0.0, "hidden deposit producing in 1925: {at}");
            }
            assert!(!d.note.trim().is_empty(), "no note: {at}");
        }
    }
}

#[test]
fn critical_resources_are_covered() {
    let file = deposits();
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for d in file.states.iter().flat_map(|s| &s.deposits) {
        *count.entry(d.resource.as_str()).or_default() += 1;
    }
    for (resource, min) in MIN_DEPOSITS {
        let n = count.get(resource).copied().unwrap_or(0);
        assert!(
            n >= *min,
            "{resource}: {n} deposits, expected at least {min}"
        );
    }
    for resource in RESOURCES {
        assert!(count.contains_key(resource), "no deposit of {resource}");
    }
}

#[test]
fn rare_earths_are_later_discoveries() {
    for state in deposits().states {
        for d in state
            .deposits
            .iter()
            .filter(|d| d.resource == "rare_earths")
        {
            assert!(
                d.discovered > START,
                "{}: rare earths known in {} ({})",
                state.state,
                d.discovered,
                d.note
            );
        }
    }
}

#[test]
fn world_output_in_1925_is_close_to_the_statistics() {
    // World production in 1925, in the units of the file; the data must land within ±15 %.
    const WORLD_1925: &[(&str, f32)] = &[
        ("coal", 1370.0),
        ("iron_ore", 165.0),
        ("copper", 1420.0),
        ("bauxite", 1380.0),
        ("oil", 146.0),
        ("gold", 590.0),
    ];
    let file = deposits();
    for (resource, world) in WORLD_1925 {
        let total: f32 = file
            .states
            .iter()
            .flat_map(|s| &s.deposits)
            .filter(|d| d.resource == *resource)
            .map(|d| d.output_1925)
            .sum();
        let ratio = total / world;
        assert!(
            (0.85..=1.15).contains(&ratio),
            "{resource}: {total} against {world} in 1925"
        );
    }
}
