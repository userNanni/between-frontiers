//! Invariants of the 1925 country data: `data/scenario/1925/countries.ron` against the
//! `CountriesFile` contract, and the Fluent names in both locales.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::scenario::{CountriesFile, MapFile, Status, SubjectKind};

const TAGS: &[&str] = &[
    // Europe
    "GBR", "IRL", "FRA", "BEL", "NLD", "LUX", "CHE", "ESP", "POR", "ITA", "SMR", "GER", "SAA",
    "DNZ", "AUT", "HUN", "CZE", "POL", "LTU", "LAT", "EST", "FIN", "SWE", "NOR", "DEN", "ISL",
    "SOV", "ROM", "YUG", "BUL", "ALB", "GRE", "TUR", "LIE", "MCO", "AND", //
    // Americas
    "USA", "CAN", "NFL", "MEX", "GUA", "HON", "SAL", "NIC", "CRI", "PAN", "CUB", "HAI", "DOM",
    "COL", "VEN", "ECU", "PER", "BOL", "CHL", "ARG", "URY", "PRY", "BRA", //
    // Asia and Middle East
    "CHN", "JAP", "MGL", "TUV", "TIB", "SIA", "AFG", "PRS", "NEP", "BHU", "RAJ", "IRQ", "TRJ",
    "HEJ", "NEJ", "YEM", "OMA", "KUW", //
    // Africa and Oceania
    "EGY", "ETH", "LBR", "SAF", "AUS", "NZL",
];

const SUBJECTS: &[(&str, &str, SubjectKind)] = &[
    ("IRL", "GBR", SubjectKind::Dominion),
    ("CAN", "GBR", SubjectKind::Dominion),
    ("NFL", "GBR", SubjectKind::Dominion),
    ("AUS", "GBR", SubjectKind::Dominion),
    ("NZL", "GBR", SubjectKind::Dominion),
    ("SAF", "GBR", SubjectKind::Dominion),
    ("RAJ", "GBR", SubjectKind::CrownColony),
    ("IRQ", "GBR", SubjectKind::Mandate),
    ("TRJ", "GBR", SubjectKind::Mandate),
    ("BHU", "GBR", SubjectKind::Protectorate),
    ("OMA", "GBR", SubjectKind::Protectorate),
    ("KUW", "GBR", SubjectKind::Protectorate),
    ("MGL", "SOV", SubjectKind::Satellite),
    ("TUV", "SOV", SubjectKind::Satellite),
];

const GOVERNMENTS: &[&str] = &[
    "parliamentary_monarchy",
    "constitutional_monarchy",
    "absolute_monarchy",
    "presidential_republic",
    "parliamentary_republic",
    "one_party_state",
    "military_dictatorship",
    "theocracy",
    "colonial_administration",
    "free_city",
];

const LOCALES: &[&str] = &["pt-BR", "en"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load() -> CountriesFile {
    let path = repo_root().join("data/scenario/1925/countries.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Message id -> attribute names, from simple line parsing of a Fluent file.
fn fluent_messages(locale: &str, file: &str) -> BTreeMap<String, BTreeSet<String>> {
    let path = repo_root().join("assets/i18n").join(locale).join(file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut messages = BTreeMap::new();
    let mut current: Option<String> = None;
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(attr) = line
            .trim_start()
            .strip_prefix('.')
            .filter(|_| line.starts_with(' '))
        {
            let (name, value) = attr
                .split_once('=')
                .unwrap_or_else(|| panic!("{file}:{}: bad attribute", n + 1));
            assert!(
                !value.trim().is_empty(),
                "{locale}/{file}:{}: empty attribute value",
                n + 1
            );
            let id = current
                .as_ref()
                .unwrap_or_else(|| panic!("{file}:{}: attribute outside a message", n + 1));
            let attrs: &mut BTreeSet<String> =
                messages.get_mut(id).expect("current message exists");
            attrs.insert(name.trim().to_owned());
            continue;
        }
        let (id, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("{locale}/{file}:{}: expected `id = value`", n + 1));
        let id = id.trim().to_owned();
        assert!(
            id.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "{locale}/{file}:{}: bad message id {id:?}",
            n + 1
        );
        assert!(
            !value.trim().is_empty(),
            "{locale}/{file}:{}: empty value for {id}",
            n + 1
        );
        assert!(
            messages.insert(id.clone(), BTreeSet::new()).is_none(),
            "{locale}/{file}: duplicate {id}"
        );
        current = Some(id);
    }
    messages
}

#[test]
fn start_date_and_tag_list() {
    let file = load();
    assert_eq!(file.start_date, "1925-01-01");
    let tags: Vec<&str> = file.countries.iter().map(|c| c.tag.as_str()).collect();
    let unique: BTreeSet<&str> = tags.iter().copied().collect();
    assert_eq!(unique.len(), tags.len(), "duplicate tags");
    let expected: BTreeSet<&str> = TAGS.iter().copied().collect();
    assert_eq!(expected.len(), TAGS.len());
    assert_eq!(unique, expected);
    for tag in &tags {
        assert!(
            tag.len() == 3 && tag.chars().all(|c| c.is_ascii_uppercase()),
            "bad tag {tag}"
        );
    }
}

#[test]
fn subjects_and_overlords() {
    let file = load();
    let by_tag: BTreeMap<&str, _> = file.countries.iter().map(|c| (c.tag.as_str(), c)).collect();
    for country in &file.countries {
        let expected = SUBJECTS.iter().find(|(t, _, _)| *t == country.tag);
        match (&country.status, expected) {
            (Status::Sovereign, None) => {}
            (Status::Subject { overlord, kind }, Some((_, want_overlord, want_kind))) => {
                assert_eq!(overlord, want_overlord, "{}", country.tag);
                assert_eq!(kind, want_kind, "{}", country.tag);
                let lord = by_tag
                    .get(overlord.as_str())
                    .unwrap_or_else(|| panic!("{}: unknown overlord", country.tag));
                assert_eq!(
                    lord.status,
                    Status::Sovereign,
                    "{}: overlord {overlord} is not sovereign",
                    country.tag
                );
            }
            (status, _) => panic!("{}: unexpected status {status:?}", country.tag),
        }
    }
}

#[test]
fn values_in_range() {
    let file = load();
    let mut colors = BTreeSet::new();
    for c in &file.countries {
        assert!(
            GOVERNMENTS.contains(&c.government.as_str()),
            "{}: government {}",
            c.tag,
            c.government
        );
        assert!(c.population > 0, "{}: population", c.tag);
        assert!(
            (100..100_000).contains(&c.gdp_per_capita),
            "{}: gdp_per_capita {}",
            c.tag,
            c.gdp_per_capita
        );
        let (lon, lat) = c.capital_lon_lat;
        assert!(
            (-180.0..=180.0).contains(&lon) && (-90.0..=90.0).contains(&lat),
            "{}: capital",
            c.tag
        );
        assert!(!c.source.trim().is_empty(), "{}: source", c.tag);
        assert!(
            colors.insert(c.color),
            "{}: colour {:?} already used",
            c.tag,
            c.color
        );
    }
    let world: u64 = file
        .countries
        .iter()
        .map(|c| c.population + c.colonies.as_ref().map_or(0, |col| col.population))
        .sum();
    assert!(
        (1_850_000_000..2_050_000_000).contains(&world),
        "world population {world}"
    );
}

/// Tags allowed to leave a social indicator as `None` (none so far).
const SOCIAL_NONE: &[&str] = &[];

#[test]
fn social_indicators() {
    let file = load();
    let (mut people, mut literate, mut years, mut urban) = (0.0, 0.0, 0.0, 0.0);
    for c in &file.countries {
        let allowed = SOCIAL_NONE.contains(&c.tag.as_str());
        let (Some(lit), Some(life), Some(urb)) = (c.literacy, c.life_expectancy, c.urbanization)
        else {
            assert!(allowed, "{}: missing a social indicator", c.tag);
            continue;
        };
        assert!((0.0..=1.0).contains(&lit), "{}: literacy {lit}", c.tag);
        assert!(
            (15.0..=75.0).contains(&life),
            "{}: life_expectancy {life}",
            c.tag
        );
        assert!((0.0..=1.0).contains(&urb), "{}: urbanization {urb}", c.tag);
        assert!(
            !c.social_source.trim().is_empty(),
            "{}: social_source",
            c.tag
        );
        let mut parts = vec![(c.population, lit, life, urb)];
        if let Some(col) = &c.colonies {
            let (Some(lit), Some(life), Some(urb)) =
                (col.literacy, col.life_expectancy, col.urbanization)
            else {
                panic!("{}: colonies miss a social indicator", c.tag);
            };
            assert!(
                (0.0..=1.0).contains(&lit),
                "{}: colonial literacy {lit}",
                c.tag
            );
            assert!(
                (15.0..=75.0).contains(&life),
                "{}: colonial life_expectancy {life}",
                c.tag
            );
            assert!(
                (0.0..=1.0).contains(&urb),
                "{}: colonial urbanization {urb}",
                c.tag
            );
            parts.push((col.population, lit, life, urb));
        }
        for (pop, lit, life, urb) in parts {
            let p = pop as f64;
            people += p;
            literate += p * f64::from(lit);
            years += p * f64::from(life);
            urban += p * f64::from(urb);
        }
    }
    // World means weighted by population, against the 1925 orders of magnitude.
    let (lit, life, urb) = (literate / people, years / people, urban / people);
    assert!((0.30..=0.45).contains(&lit), "world literacy {lit}");
    assert!(
        (30.0..=40.0).contains(&life),
        "world life expectancy {life}"
    );
    assert!((0.18..=0.25).contains(&urb), "world urbanization {urb}");
}

#[test]
fn fluent_names_in_every_locale() {
    let file = load();
    let mut ids_by_locale = Vec::new();
    for locale in LOCALES {
        let countries = fluent_messages(locale, "countries.ftl");
        let governments = fluent_messages(locale, "governments.ftl");
        for c in &file.countries {
            let name = format!("country-{}", c.tag);
            let attrs = countries
                .get(&name)
                .unwrap_or_else(|| panic!("{locale}: missing {name}"));
            assert!(
                attrs.contains("article"),
                "{locale}: {name} has no .article"
            );
            let capital = format!("country-{}-capital", c.tag);
            assert!(
                countries.contains_key(&capital),
                "{locale}: missing {capital}"
            );
        }
        for key in GOVERNMENTS {
            let id = format!("government-{key}");
            assert!(governments.contains_key(&id), "{locale}: missing {id}");
        }
        let ids: BTreeSet<String> = countries
            .into_keys()
            .chain(governments.into_keys())
            .collect();
        ids_by_locale.push((locale, ids));
    }
    let (first, reference) = &ids_by_locale[0];
    for (locale, ids) in &ids_by_locale[1..] {
        assert_eq!(
            ids, reference,
            "message ids differ between {first} and {locale}"
        );
    }
}

/// Empire totals (metropole plus colonies) before the figures were split, for every owner with
/// colonies: tag, population, GDP per capita.
const EMPIRES: &[(&str, u64, u32)] = &[
    ("GBR", 117_808_000, 3806),
    ("FRA", 106_431_000, 3264),
    ("BEL", 19_273_000, 3467),
    ("NLD", 65_319_000, 2227),
    ("ESP", 23_104_000, 3624),
    ("POR", 15_042_000, 1459),
    ("ITA", 42_288_000, 4357),
    ("USA", 128_600_000, 10217),
    ("JAP", 83_515_000, 2878),
    ("DEN", 3_443_000, 6931),
    ("SAF", 7_888_000, 2133),
    ("AUS", 6_438_000, 8155),
    ("NZL", 1_426_000, 8114),
];

#[test]
fn metropole_and_colonies_add_up_to_the_empire() {
    let file = load();
    let with_colonies: BTreeSet<&str> = file
        .countries
        .iter()
        .filter(|c| c.colonies.is_some())
        .map(|c| c.tag.as_str())
        .collect();
    let expected: BTreeSet<&str> = EMPIRES.iter().map(|(t, _, _)| *t).collect();
    assert_eq!(with_colonies, expected);
    for &(tag, pop, gdp) in EMPIRES {
        let c = file.countries.iter().find(|c| c.tag == tag).unwrap();
        let col = c.colonies.as_ref().unwrap();
        assert!(col.population > 0, "{tag}: colonial population");
        assert!(
            (100..100_000).contains(&col.gdp_per_capita),
            "{tag}: colonial gdp_per_capita {}",
            col.gdp_per_capita
        );
        let total = c.population + col.population;
        // Populations are rounded to the thousand.
        assert!(total.abs_diff(pop) <= 1_000, "{tag}: {total} against {pop}");
        let weighted = (c.population as f64 * f64::from(c.gdp_per_capita)
            + col.population as f64 * f64::from(col.gdp_per_capita))
            / total as f64;
        assert!(
            (weighted - f64::from(gdp)).abs() <= 1.0,
            "{tag}: GDP per capita {weighted:.1} against {gdp}"
        );
    }
}

#[test]
fn colonies_match_the_map() {
    let file = load();
    let path = repo_root().join("assets/map/provinces.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let map: MapFile = ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut metropole = BTreeSet::new();
    let mut colonial = BTreeSet::new();
    for p in &map.provinces {
        if p.colony {
            colonial.insert(p.owner.as_str());
        } else {
            metropole.insert(p.owner.as_str());
        }
    }
    for c in &file.countries {
        assert_eq!(
            c.colonies.is_some(),
            colonial.contains(c.tag.as_str()),
            "{}: colonies in countries.ron and colony provinces in provinces.ron disagree",
            c.tag
        );
        assert!(
            metropole.contains(c.tag.as_str()),
            "{}: no metropolitan province",
            c.tag
        );
    }
    let tags: BTreeSet<&str> = file.countries.iter().map(|c| c.tag.as_str()).collect();
    for owner in colonial {
        assert!(
            tags.contains(owner),
            "colonies owned by unknown tag {owner}"
        );
    }
}
