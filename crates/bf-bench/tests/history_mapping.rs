//! `tools/history/mapping.csv` covers both sides of the historical comparison: every 1925 tag
//! (and every colonial empire) receives at least one modern country, and every modern country
//! of the reference series in `data/history/` goes to some 1925 tag.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::scenario::CountriesFile;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(root().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Rows of a CSV file without its `#` comment lines: header and records, split on commas
/// (the files read here never quote a comma).
fn csv(path: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let text = read(path);
    let mut lines = text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    let header = lines
        .next()
        .expect("header")
        .split(',')
        .map(str::to_owned)
        .collect();
    let rows = lines
        .map(|l| l.split(',').map(str::to_owned).collect())
        .collect();
    (header, rows)
}

struct Row {
    iso3: String,
    tag: String,
    part: String,
    carved: Option<f64>,
}

fn mapping() -> Vec<Row> {
    let (header, rows) = csv("tools/history/mapping.csv");
    assert_eq!(
        header,
        [
            "iso3",
            "country",
            "tag",
            "part",
            "pop1925_k",
            "region",
            "note"
        ]
    );
    rows.into_iter()
        .map(|r| {
            assert_eq!(r.len(), 7, "{r:?}");
            Row {
                iso3: r[0].clone(),
                tag: r[2].clone(),
                part: r[3].clone(),
                carved: (!r[4].is_empty())
                    .then(|| r[4].parse().unwrap_or_else(|_| panic!("pop1925_k {r:?}"))),
            }
        })
        .collect()
}

#[test]
fn every_tag_and_every_empire_receives_a_modern_country() {
    let file: CountriesFile = ron::from_str(&read("data/scenario/1925/countries.ron")).unwrap();
    let rows = mapping();
    let have: BTreeSet<(String, String)> = rows
        .iter()
        .map(|r| (r.tag.clone(), r.part.clone()))
        .collect();
    let mut want = BTreeSet::new();
    for c in &file.countries {
        want.insert((c.tag.clone(), "metropole".to_owned()));
        if c.colonies.is_some() {
            want.insert((c.tag.clone(), "colonies".to_owned()));
        }
    }
    let missing: Vec<_> = want.difference(&have).collect();
    assert!(
        missing.is_empty(),
        "units without a modern country: {missing:?}"
    );
    let unknown: Vec<_> = have.difference(&want).collect();
    assert!(
        unknown.is_empty(),
        "mapping names units that do not exist: {unknown:?}"
    );
}

#[test]
fn every_modern_country_goes_to_exactly_one_main_unit() {
    let rows = mapping();
    let mut main: BTreeMap<&str, usize> = BTreeMap::new();
    for r in &rows {
        assert!(
            r.part == "metropole" || r.part == "colonies",
            "{}: part {}",
            r.iso3,
            r.part
        );
        match r.carved {
            None => *main.entry(&r.iso3).or_default() += 1,
            Some(k) => assert!(k > 0.0, "{}: empty carve", r.iso3),
        }
    }
    for r in &rows {
        assert_eq!(
            main.get(r.iso3.as_str()),
            Some(&1),
            "{} needs exactly one row without pop1925_k (the rest of the country)",
            r.iso3
        );
    }
    // Every country of the reference series is mapped.
    let mapped: BTreeSet<&str> = main.keys().copied().collect();
    for (file, col) in [
        ("data/history/population.csv", 0),
        ("data/history/urban.csv", 0),
        ("data/history/literacy.csv", 0),
        ("data/history/production.csv", 0),
        ("data/history/wdi.csv", 0),
    ] {
        let (_, data) = csv(file);
        let codes: BTreeSet<&str> = data.iter().map(|r| r[col].as_str()).collect();
        let missing: Vec<_> = codes.difference(&mapped).collect();
        // Small territories with no population series of their own may appear in other series;
        // they must be mapped too.
        assert!(
            missing.is_empty(),
            "{file}: countries not in mapping.csv: {missing:?}"
        );
    }
    // Maddison: modern codes are mapped; former states are split through their members.
    let (_, data) = csv("data/history/maddison.csv");
    let former = ["SUN", "CSK", "YUG", "SDN_FORMER"];
    let codes: BTreeSet<&str> = data.iter().map(|r| r[0].as_str()).collect();
    let missing: Vec<_> = codes
        .iter()
        .filter(|c| !mapped.contains(*c) && !former.contains(c))
        .collect();
    assert!(
        missing.is_empty(),
        "maddison.csv: countries not in mapping.csv: {missing:?}"
    );
}
