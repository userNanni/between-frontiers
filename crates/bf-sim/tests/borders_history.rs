//! Invariants of the border data: the reference of sovereignty changes
//! (`data/history/borders.ron`) replayed on the 1925 map, the catalogue of countries that may be
//! born (`data/borders/candidates.ron`), their names in every locale and their flags. Each rule of
//! the validators is also shown to refuse a broken copy.

use std::collections::BTreeSet;
use std::path::PathBuf;

use bf_sim::borders::{BordersFile, CandidatesFile, ChangeKind};
use bf_sim::scenario::{CountriesFile, MapFile};

const LOCALES: &[&str] = &["pt-BR", "en"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

struct Data {
    borders: BordersFile,
    candidates: CandidatesFile,
    map: MapFile,
    countries: CountriesFile,
}

fn load() -> Data {
    Data {
        borders: read("data/history/borders.ron"),
        candidates: read("data/borders/candidates.ron"),
        map: read("assets/map/provinces.ron"),
        countries: read("data/scenario/1925/countries.ron"),
    }
}

fn border_errors(edit: impl FnOnce(&mut BordersFile)) -> Vec<String> {
    let mut d = load();
    edit(&mut d.borders);
    d.borders
        .validate(&d.map, &d.countries, &d.candidates)
        .err()
        .unwrap_or_default()
}

fn assert_refused(errors: &[String], needle: &str) {
    assert!(
        errors.iter().any(|e| e.contains(needle)),
        "expected an error containing {needle:?}, got {errors:?}"
    );
}

#[test]
fn the_reference_replays_on_the_1925_map() {
    let d = load();
    if let Err(e) = d.borders.validate(&d.map, &d.countries, &d.candidates) {
        panic!("{} errors:\n{}", e.len(), e.join("\n"));
    }
    if let Err(e) = d.candidates.validate(&d.map, &d.countries) {
        panic!("{} errors:\n{}", e.len(), e.join("\n"));
    }
}

#[test]
fn the_map_of_2025_is_the_modern_one_in_the_large() {
    let d = load();
    let owners = d.borders.owners_on(&d.map, "2025-12-31");
    let holds = |tag: &str| owners.values().filter(|o| o.as_str() == tag).count();
    // Federations of 1925 that dissolved hold nothing; their successors hold land.
    for gone in ["YUG", "CZE"] {
        assert_eq!(holds(gone), 0, "{gone}");
    }
    for born in [
        "PAK", "BGD", "NGA", "IDN", "KOR", "PRK", "UKR", "KAZ", "DZA", "COD",
    ] {
        assert!(holds(born) > 0, "{born} holds no state in 2025");
    }
    let colonial: usize = d
        .map
        .states
        .iter()
        .filter(|s| s.colony && owners[&s.key] == s.owner)
        .count();
    let total = d.map.states.iter().filter(|s| s.colony).count();
    assert!(
        colonial * 5 < total,
        "{colonial} of {total} colonial states still with the 1925 owner"
    );
}

#[test]
fn every_new_tag_has_a_catalogue_row_a_name_and_a_flag() {
    let d = load();
    let old: BTreeSet<&str> = d
        .countries
        .countries
        .iter()
        .map(|c| c.tag.as_str())
        .collect();
    for c in &d.borders.changes {
        for t in [&c.from, &c.to] {
            assert!(
                old.contains(t.as_str()) || d.candidates.get(t).is_some(),
                "{t} ({}) has no row in candidates.ron",
                c.date
            );
        }
    }
    let ftl: Vec<String> = LOCALES
        .iter()
        .map(|l| {
            std::fs::read_to_string(root().join(format!("assets/i18n/{l}/countries.ftl"))).unwrap()
        })
        .collect();
    for c in &d.candidates.countries {
        for (l, text) in LOCALES.iter().zip(&ftl) {
            for key in [
                format!("country-{} =", c.tag),
                format!("country-{}-capital =", c.tag),
            ] {
                assert!(text.contains(&key), "{l}: missing {key}");
            }
        }
        if let Some(f) = &c.flag {
            let png = root().join(format!("assets/flags/{f}.png"));
            assert!(png.is_file(), "missing {}", png.display());
        }
    }
}

#[test]
fn the_validators_refuse_broken_copies() {
    assert_refused(
        &border_errors(|b| b.changes[0].date = "1925-13-01".into()),
        "bad date",
    );
    assert_refused(&border_errors(|b| b.changes.swap(0, 5)), "out of order");
    assert_refused(
        &border_errors(|b| b.changes[0].to = "ZZZ".into()),
        "unknown tag",
    );
    assert_refused(
        &border_errors(|b| {
            let c = b
                .changes
                .iter_mut()
                .find(|c| c.kind == ChangeKind::Transfer)
                .unwrap();
            c.states.push("BRA-RJ".into());
        }),
        "is held by BRA",
    );
    assert_refused(
        &border_errors(|b| {
            let c = b
                .changes
                .iter_mut()
                .find(|c| c.kind == ChangeKind::Rename)
                .unwrap();
            c.states.push("BRA-RJ".into());
        }),
        "a rename moves nothing",
    );
    assert_refused(
        &border_errors(|b| b.changes[0].source.clear()),
        "needs a source",
    );
    let mut d = load();
    let first = d.candidates.countries[0].clone();
    d.candidates.countries.push(first);
    d.candidates.countries[1].tag = "BRA".into();
    let e = d.candidates.validate(&d.map, &d.countries).unwrap_err();
    assert_refused(&e, "listed twice");
    assert_refused(&e, "tag of a 1925 country");
}
