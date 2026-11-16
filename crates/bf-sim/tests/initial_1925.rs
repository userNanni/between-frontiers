//! Invariants of the 1925 initial conditions: `data/scenario/1925/population.ron` and
//! `industry.ron` against the `PopulationFile` and `IndustryFile` contracts, the states of
//! `assets/map/provinces.ron`, the countries of `countries.ron` and the goods catalogue.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::economy::GoodsFile;
use bf_sim::initial::{IndustryFile, PopulationFile};
use bf_sim::scenario::{CountriesFile, MapFile};

/// Output keys that are not goods: infrastructure stocks, and pig iron (an intermediate that the
/// economy turns into steel, kept to seed blast furnaces apart from steelworks).
const NON_GOOD_KEYS: &[&str] = &[
    "railways_km",
    "merchant_fleet_grt",
    "telephones",
    "pig_iron",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn population() -> PopulationFile {
    read("data/scenario/1925/population.ron")
}

fn industry() -> IndustryFile {
    read("data/scenario/1925/industry.ron")
}

fn countries() -> CountriesFile {
    read("data/scenario/1925/countries.ron")
}

fn map() -> MapFile {
    read("assets/map/provinces.ron")
}

#[test]
fn every_state_once() {
    let pop = population();
    let map = map();
    let expected: BTreeSet<&str> = map.states.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(expected.len(), 870);
    let mut seen = BTreeSet::new();
    for s in &pop.states {
        assert!(seen.insert(s.state.as_str()), "{}: listed twice", s.state);
        assert!(
            expected.contains(s.state.as_str()),
            "{}: not a state of the map",
            s.state
        );
    }
    let missing: Vec<_> = expected.difference(&seen).collect();
    assert!(missing.is_empty(), "states without population: {missing:?}");
}

#[test]
fn state_populations_sum_to_countries() {
    let pop = population();
    let map = map();
    let by_key: BTreeMap<&str, u64> = pop
        .states
        .iter()
        .map(|s| (s.state.as_str(), s.population))
        .collect();
    // (owner, colony) -> people
    let mut sums: BTreeMap<(&str, bool), u64> = BTreeMap::new();
    for s in &map.states {
        *sums.entry((s.owner.as_str(), s.colony)).or_default() += by_key[s.key.as_str()];
    }
    let countries = countries();
    for c in &countries.countries {
        let metro = sums.remove(&(c.tag.as_str(), false)).unwrap_or(0);
        assert_eq!(metro, c.population, "{}: metropole states", c.tag);
        let colonial = sums.remove(&(c.tag.as_str(), true));
        match &c.colonies {
            Some(col) => assert_eq!(colonial, Some(col.population), "{}: colonial states", c.tag),
            None => assert_eq!(
                colonial, None,
                "{}: colonial states without colonies",
                c.tag
            ),
        }
    }
    assert!(sums.is_empty(), "states of unknown owners: {sums:?}");
}

#[test]
fn urban_shares_are_valid_and_match_countries() {
    let pop = population();
    let map = map();
    let by_key: BTreeMap<&str, (u64, f32)> = pop
        .states
        .iter()
        .map(|s| (s.state.as_str(), (s.population, s.urban)))
        .collect();
    for s in &pop.states {
        assert!(
            (0.0..=1.0).contains(&s.urban),
            "{}: urban {}",
            s.state,
            s.urban
        );
    }
    let mut blocks: BTreeMap<(&str, bool), (f64, f64)> = BTreeMap::new();
    for s in &map.states {
        let (p, u) = by_key[s.key.as_str()];
        let e = blocks.entry((s.owner.as_str(), s.colony)).or_default();
        e.0 += p as f64;
        e.1 += p as f64 * u as f64;
    }
    for c in &countries().countries {
        let targets = [
            (false, c.urbanization),
            (true, c.colonies.as_ref().and_then(|col| col.urbanization)),
        ];
        for (colony, target) in targets {
            let (Some(target), Some(&(p, pu))) = (target, blocks.get(&(c.tag.as_str(), colony)))
            else {
                continue;
            };
            let mean = pu / p;
            assert!(
                (mean - target as f64).abs() < 0.005,
                "{} (colonies: {colony}): urban mean {mean:.4} against {target}",
                c.tag
            );
        }
    }
}

#[test]
fn workforce_is_complete_and_valid() {
    let pop = population();
    let countries = countries();
    let mut seen = BTreeSet::new();
    for w in &pop.workforce {
        assert!(
            seen.insert((w.country.as_str(), w.colonies)),
            "{} (colonies: {}): listed twice",
            w.country,
            w.colonies
        );
        let sum = w.agriculture + w.industry + w.services;
        assert!(
            (sum - 1.0).abs() <= 0.01,
            "{}: sectors sum to {sum}",
            w.country
        );
        for (name, v) in [
            ("agriculture", w.agriculture),
            ("industry", w.industry),
            ("services", w.services),
        ] {
            assert!((0.0..=1.0).contains(&v), "{}: {name} {v}", w.country);
        }
        assert!(
            (0.2..=0.7).contains(&w.participation),
            "{}: participation {}",
            w.country,
            w.participation
        );
    }
    for c in &countries.countries {
        assert!(
            seen.contains(&(c.tag.as_str(), false)),
            "{}: no metropole workforce",
            c.tag
        );
        assert_eq!(
            seen.contains(&(c.tag.as_str(), true)),
            c.colonies.is_some(),
            "{}: colonial workforce must exist exactly when the country has colonies",
            c.tag
        );
    }
}

#[test]
fn industry_keys_units_and_countries_are_valid() {
    let ind = industry();
    let goods: GoodsFile = read("data/economy/goods.ron");
    let good_keys: BTreeSet<&str> = goods.goods.iter().map(|g| g.key.as_str()).collect();
    let mut units = BTreeMap::new();
    for (k, u) in &ind.units {
        assert!(
            good_keys.contains(k.as_str()) || NON_GOOD_KEYS.contains(&k.as_str()),
            "unit for unknown key {k}"
        );
        assert!(!u.trim().is_empty(), "{k}: empty unit");
        assert!(units.insert(k.as_str(), u).is_none(), "{k}: unit twice");
    }
    let tags: BTreeSet<String> = countries().countries.into_iter().map(|c| c.tag).collect();
    let mut seen = BTreeSet::new();
    for c in &ind.countries {
        assert!(tags.contains(&c.country), "{}: unknown country", c.country);
        assert!(
            seen.insert(c.country.as_str()),
            "{}: listed twice",
            c.country
        );
        assert!(!c.source.trim().is_empty(), "{}: no source", c.country);
        let mut keys = BTreeSet::new();
        for (k, v) in &c.output {
            assert!(
                units.contains_key(k.as_str()),
                "{}: {k} has no unit",
                c.country
            );
            assert!(keys.insert(k.as_str()), "{}: {k} twice", c.country);
            assert!(v.is_finite() && *v >= 0.0, "{}: {k} = {v}", c.country);
        }
    }
    let missing: Vec<_> = tags.iter().filter(|t| !seen.contains(t.as_str())).collect();
    assert!(
        missing.is_empty(),
        "countries without industry: {missing:?}"
    );
}

#[test]
fn industry_world_totals_are_plausible() {
    let ind = industry();
    let mut total: BTreeMap<&str, f64> = BTreeMap::new();
    for c in &ind.countries {
        for (k, v) in &c.output {
            *total.entry(k.as_str()).or_default() += *v as f64;
        }
    }
    // Reference world figures for 1925 (INITIAL.md), with a wide band.
    for (key, lo, hi) in [
        ("steel", 80_000.0, 100_000.0),
        ("cement", 55_000.0, 75_000.0),
        ("electricity", 170_000.0, 210_000.0),
        ("aluminium", 160.0, 210.0),
        ("automobiles", 4_400.0, 5_200.0),
    ] {
        let t = total[key];
        assert!((lo..=hi).contains(&t), "world {key} = {t}");
    }
}
