//! Invariants of the 1925 international data: `data/scenario/1925/diplomacy.ron` against the
//! `DiplomacyFile` contract, the countries of `countries.ron` and the Fluent names. Each rule of
//! `DiplomacyFile::validate` is also shown to refuse a broken copy.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::diplomacy::{DiplomacyFile, Regime, TreatyKind};
use bf_sim::scenario::CountriesFile;

const LOCALES: &[&str] = &["pt-BR", "en"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn load() -> (DiplomacyFile, CountriesFile) {
    (
        read("data/scenario/1925/diplomacy.ron"),
        read("data/scenario/1925/countries.ron"),
    )
}

/// Errors of `validate` on a copy of the data changed by `edit`.
fn errors_after(edit: impl FnOnce(&mut DiplomacyFile)) -> Vec<String> {
    let (mut file, countries) = load();
    edit(&mut file);
    file.validate(&countries).err().unwrap_or_default()
}

fn assert_refused(errors: &[String], needle: &str) {
    assert!(
        errors.iter().any(|e| e.contains(needle)),
        "expected an error containing {needle:?}, got {errors:?}"
    );
}

#[test]
fn the_data_is_valid() {
    let (file, countries) = load();
    if let Err(errors) = file.validate(&countries) {
        panic!("{} problems:\n{}", errors.len(), errors.join("\n"));
    }
}

#[test]
fn start_state_matches_1925() {
    let (file, _) = load();
    let start = file.start_date.as_str();
    // The League on 1925-01-01: 55 members, four permanent seats, no Germany yet.
    let league = file.members_on("league_of_nations", start);
    assert_eq!(league.len(), 55);
    let permanent: BTreeSet<&str> = league
        .iter()
        .filter(|m| m.permanent_seat)
        .map(|m| m.tag.as_str())
        .collect();
    assert_eq!(permanent, BTreeSet::from(["FRA", "GBR", "ITA", "JAP"]));
    assert!(league.iter().all(|m| m.tag != "GER" && m.tag != "USA"));
    // Locarno is signed in December 1925 and binds only from 1926.
    let locarno = |date: &str| {
        file.treaties_on(date)
            .any(|t| t.key == "locarno_rhineland_pact")
    };
    assert!(!locarno(start));
    assert!(!locarno("1925-12-31"));
    assert!(locarno("1926-09-14"));
    // Britain is off gold on the start date and back on 1925-04-28.
    assert_eq!(file.regime_on("GBR", start).unwrap().regime, Regime::Paper);
    assert!(matches!(
        file.regime_on("GBR", "1925-04-28").unwrap().regime,
        Regime::Gold { .. }
    ));
    assert!(matches!(
        file.regime_on("CHN", start).unwrap().regime,
        Regime::Silver { .. }
    ));
}

#[test]
fn summary_of_the_start_state() {
    let (file, _) = load();
    let start = file.start_date.as_str();
    let mut regimes: BTreeMap<&str, usize> = BTreeMap::new();
    for t in file.tariffs.iter().map(|t| t.tag.as_str()) {
        *regimes
            .entry(file.regime_on(t, start).unwrap().regime.key())
            .or_default() += 1;
    }
    let in_force = file.treaties_on(start).count();
    println!(
        "{} organizations, {} treaties ({in_force} in force on {start}), regimes {regimes:?}",
        file.organizations.len(),
        file.treaties.len()
    );
    // Most of the world was off gold in January 1925.
    assert!(regimes["paper"] > regimes["gold"]);
    let rates: Vec<f32> = file.tariffs.iter().map(|t| t.rate).collect();
    let mean = rates.iter().sum::<f32>() / rates.len() as f32;
    assert!((0.08..0.20).contains(&mean), "mean tariff {mean}");
}

#[test]
fn refuses_an_unknown_tag() {
    let errors = errors_after(|f| f.treaties[0].parties[0] = "XXX".into());
    assert_refused(&errors, "unknown tag XXX");
    let errors = errors_after(|f| f.organizations[0].members[0].tag = "ZZZ".into());
    assert_refused(&errors, "unknown tag ZZZ");
    let errors = errors_after(|f| {
        f.monetary[0].regime = Regime::Foreign {
            issuer: "QQQ".into(),
        }
    });
    assert_refused(&errors, "unknown tag QQQ");
}

#[test]
fn refuses_a_one_sided_treaty() {
    let errors = errors_after(|f| {
        let t = &mut f.treaties[0];
        t.parties.truncate(1);
    });
    assert_refused(&errors, "needs two parties");
    let errors = errors_after(|f| {
        let t = &mut f.treaties[0];
        let first = t.parties[0].clone();
        t.parties[1] = first;
    });
    assert_refused(&errors, "listed twice");
    // A one-sided kind needs a strong and a weak side.
    let errors = errors_after(|f| {
        let t = f
            .treaties
            .iter_mut()
            .find(|t| t.kind == TreatyKind::Occupation)
            .unwrap();
        t.senior = t.parties.clone();
    });
    assert_refused(&errors, "needs both sides");
}

#[test]
fn refuses_incoherent_dates() {
    let errors = errors_after(|f| {
        let t = &mut f.treaties[0];
        t.in_force = "1900-01-01".into();
    });
    assert_refused(&errors, "before it was signed");
    let errors = errors_after(|f| {
        let t = &mut f.treaties[0];
        t.until = Some(t.in_force.clone());
    });
    assert_refused(&errors, "before it begins");
    let errors = errors_after(|f| f.treaties[0].signed = "1925-02-30x".into());
    assert_refused(&errors, "bad date");
    let errors = errors_after(|f| f.organizations[0].members[0].joined = "1919-01-01".into());
    assert_refused(&errors, "joined before");
    // Two regimes at once.
    let errors = errors_after(|f| {
        let mut extra = f.monetary.iter().find(|p| p.tag == "USA").unwrap().clone();
        extra.from = "1920-01-01".into();
        extra.regime = Regime::Paper;
        f.monetary.push(extra);
    });
    assert_refused(&errors, "overlap");
}

#[test]
fn refuses_a_country_without_a_monetary_regime() {
    let errors = errors_after(|f| f.monetary.retain(|p| p.tag != "BRA"));
    assert_refused(&errors, "monetary BRA: no regime");
    let errors = errors_after(|f| {
        for p in f.monetary.iter_mut().filter(|p| p.tag == "GBR") {
            p.from = "1926-01-01".into();
            p.until = None;
        }
        f.monetary
            .retain(|p| p.tag != "GBR" || p.regime == Regime::Paper);
    });
    assert_refused(&errors, "monetary GBR: no regime");
}

#[test]
fn refuses_a_tariff_out_of_range() {
    let errors = errors_after(|f| f.tariffs[0].rate = 0.9);
    assert_refused(&errors, "outside 0..=");
    let errors = errors_after(|f| f.tariffs[0].rate = -0.01);
    assert_refused(&errors, "outside 0..=");
    let errors = errors_after(|f| f.tariffs.retain(|t| t.tag != "CHN"));
    assert_refused(&errors, "tariff CHN: missing");
    // Inside a customs union the junior applies the senior's tariff.
    let errors = errors_after(|f| {
        f.tariffs.iter_mut().find(|t| t.tag == "LUX").unwrap().rate = 0.2;
    });
    assert_refused(&errors, "own tariff inside a customs union");
}

#[test]
fn subjection_references_countries_ron() {
    // An instrument may name only a subject relation that countries.ron has.
    let errors = errors_after(|f| {
        let t = f
            .treaties
            .iter_mut()
            .find(|t| t.kind == TreatyKind::Subjection)
            .unwrap();
        t.parties[0] = "ARG".into();
    });
    assert_refused(&errors, "is not a subject of");
}

/// Message ids of a Fluent file, by simple line parsing.
fn fluent_ids(locale: &str, file: &str) -> BTreeSet<String> {
    let path = repo_root().join("assets/i18n").join(locale).join(file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with(' ') && l.contains('='))
        .map(|l| l.split_once('=').unwrap().0.trim().to_owned())
        .collect()
}

#[test]
fn fluent_names_in_every_locale() {
    let (file, _) = load();
    let mut wanted: BTreeSet<String> = BTreeSet::new();
    wanted.extend(file.organizations.iter().map(|o| format!("org-{}", o.key)));
    wanted.extend(file.treaties.iter().map(|t| format!("treaty-{}", t.key)));
    wanted.extend(
        TreatyKind::ALL
            .iter()
            .map(|k| format!("treaty-kind-{}", k.key())),
    );
    wanted.extend(Regime::KEYS.iter().map(|k| format!("regime-{k}")));
    let mut by_locale = Vec::new();
    for locale in LOCALES {
        let ids = fluent_ids(locale, "diplomacy.ftl");
        for id in &wanted {
            assert!(ids.contains(id), "{locale}: missing {id}");
        }
        by_locale.push(ids);
    }
    assert_eq!(by_locale[0], by_locale[1], "ids differ between locales");
}
