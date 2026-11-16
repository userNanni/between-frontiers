//! The communities of 1925: `data/culture/*.ron` and `data/scenario/1925/communities.ron` against
//! the `bf_sim::communities` contract and the map, and the rows the ABC rule makes of them,
//! measured against the population budget of `docs/architecture.pt-BR.md` ("Tamanho").

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::communities::{
    Basis, CULTURE_FAMILIES, Communities, CommunitiesFile, CommunityError, CultureDef,
    CulturesFile, RELIGION_FAMILIES, ReligionDef, ReligionsFile, RowRule, StateComposition,
};
use bf_sim::economy::PROFESSIONS;
use bf_sim::resources::LandFile;
use bf_sim::scenario::{CountriesFile, MapFile, MapMeta, StateDef};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

struct Data {
    cultures: CulturesFile,
    religions: ReligionsFile,
    file: CommunitiesFile,
    map: MapFile,
}

fn data() -> Data {
    Data {
        cultures: read("data/culture/cultures.ron"),
        religions: read("data/culture/religions.ron"),
        file: read("data/scenario/1925/communities.ron"),
        map: read("assets/map/provinces.ron"),
    }
}

fn built(d: &Data) -> Communities {
    Communities::build(&d.cultures, &d.religions, &d.file, &d.map).unwrap_or_else(|e| panic!("{e}"))
}

/// State population as the economy approximates it while `population.ron` does not exist
/// (`econ::setup`: the country's population split by cropland plus 2 % of the area).
fn populations(map: &MapFile) -> BTreeMap<String, f64> {
    let countries: CountriesFile = read("data/scenario/1925/countries.ron");
    let land: LandFile = read("data/scenario/1925/land.ron");
    let crop: BTreeMap<&str, f64> = land
        .states
        .iter()
        .map(|s| (s.state.as_str(), f64::from(s.cropland_km2)))
        .collect();
    let weight = |s: &StateDef| {
        crop.get(s.key.as_str()).copied().unwrap_or(0.0) + f64::from(s.area_km2) * 0.02 + 1.0
    };
    let mut sums: BTreeMap<(&str, bool), f64> = BTreeMap::new();
    for s in &map.states {
        *sums.entry((s.owner.as_str(), s.colony)).or_default() += weight(s);
    }
    map.states
        .iter()
        .map(|s| {
            let c = countries.countries.iter().find(|c| c.tag == s.owner);
            let total = c.map_or(0, |c| {
                if s.colony {
                    c.colonies.as_ref().map_or(0, |x| x.population)
                } else {
                    c.population
                }
            }) as f64;
            let share = weight(s) / sums[&(s.owner.as_str(), s.colony)];
            (s.key.clone(), total * share)
        })
        .collect()
}

#[test]
fn the_1925_data_holds_together() {
    let d = data();
    let c = built(&d);
    assert_eq!(c.states.len(), 870);
    for s in &c.states {
        assert!(
            s.shares.len() <= 10,
            "{}: {} communities",
            s.state,
            s.shares.len()
        );
    }
    // Every culture and religion in the lists is used, so the lists do not rot.
    let cultures: BTreeSet<u16> = c.communities.iter().map(|&(c, _)| c).collect();
    let religions: BTreeSet<u16> = c.communities.iter().map(|&(_, r)| r).collect();
    for (i, k) in d.cultures.cultures.iter().enumerate() {
        assert!(
            cultures.contains(&(i as u16)),
            "culture {} lives nowhere",
            k.key
        );
    }
    for (i, k) in d.religions.religions.iter().enumerate() {
        assert!(
            religions.contains(&(i as u16)),
            "religion {} lives nowhere",
            k.key
        );
    }
    // The game loads the same files.
    let game = bf_sim::data::GameData::load(&repo_root()).unwrap();
    assert_eq!(game.communities.map(|c| c.states.len()), Some(870));
}

#[test]
fn rows_fit_the_population_budget() {
    let d = data();
    let c = built(&d);
    let rule = RowRule::PROPOSED;
    let rows = c.rows(&rule, &populations(&d.map));
    let professions = PROFESSIONS.len();
    let lines: Vec<usize> = rows.iter().map(|r| r.lines(professions)).collect();
    let total: usize = lines.iter().sum();
    let max = lines.iter().copied().max().unwrap();
    let with_rows: Vec<usize> = rows.iter().map(|r| r.rows.len()).collect();
    let others = rows.iter().filter(|r| !r.others.is_empty()).count();
    eprintln!(
        "rows per state: mean {:.2}, max {} (+ other minorities in {others} states); \
         lines (state × community × profession, {professions} professions): total {total}, \
         mean {:.1}, max {max}",
        with_rows.iter().sum::<usize>() as f64 / rows.len() as f64,
        with_rows.iter().max().unwrap(),
        total as f64 / rows.len() as f64,
    );
    for r in &rows {
        assert!(!r.rows.is_empty(), "{}: no row", r.state);
        assert!(
            r.rows.len() <= rule.max_rows,
            "{}: {} rows",
            r.state,
            r.rows.len()
        );
    }
    // Budget of "Tamanho": about 900 states × 6 communities × 12 professions ≈ 65 thousand
    // lines, under the 140–160 thousand pops at which Victoria 3 slows down. The start date
    // has to sit inside it with room for the century to grow.
    assert!(total <= 65_000, "{total} lines in 1925, budget 65 000");
    assert!(max <= (rule.max_rows + 1) * professions);
}

#[test]
fn small_groups_with_power_get_a_row() {
    let d = data();
    let c = built(&d);
    let rows = c.rows(&RowRule::PROPOSED, &populations(&d.map));
    let has_row = |state: &str, culture: &str, religion: &str| {
        let id = c
            .find(culture, religion)
            .unwrap_or_else(|| panic!("{culture}/{religion} is not a community"));
        rows.iter()
            .find(|r| r.state == state)
            .unwrap_or_else(|| panic!("{state} is not a state"))
            .rows
            .contains(&id)
    };
    // Size alone: the two communities of Northern Ireland; Tutsi in Ruanda-Urundi.
    assert!(has_row("GBR-NORTHERN-IRELAND", "irish", "catholic"));
    assert!(
        has_row("BEL-RUANDA-URUNDI", "tutsi", "folk")
            || has_row("BEL-RUANDA-URUNDI", "tutsi", "catholic")
    );
    // Relevance: the whites of South Africa, the Jews of Palestine.
    assert!(has_row("ZAF-TRANSVAAL", "afrikaner", "protestant"));
    assert!(has_row("ZAF-TRANSVAAL", "english", "protestant"));
    assert!(has_row("GBR-PALESTINE", "ashkenazi", "jewish"));
}

// ---------------------------------------------------------------- the checks refuse bad data

fn tiny() -> Data {
    let state = |id: u16, key: &str| StateDef {
        id,
        key: key.to_string(),
        name: key.to_string(),
        owner: "AAA".to_string(),
        colony: false,
        center: (0.0, 0.0),
        area_km2: 1.0,
        provinces: vec![id],
        neighbours: Vec::new(),
    };
    let comp = |state: &str, shares: &[(&str, &str, f32)]| StateComposition {
        state: state.to_string(),
        shares: shares
            .iter()
            .map(|&(c, r, v)| (c.to_string(), r.to_string(), v))
            .collect(),
        basis: Basis::Census,
        source: "test".to_string(),
    };
    Data {
        cultures: CulturesFile {
            cultures: vec![
                CultureDef {
                    key: "aa".into(),
                    family: CULTURE_FAMILIES[0].into(),
                },
                CultureDef {
                    key: "bb".into(),
                    family: CULTURE_FAMILIES[0].into(),
                },
            ],
        },
        religions: ReligionsFile {
            religions: vec![ReligionDef {
                key: "rr".into(),
                family: RELIGION_FAMILIES[0].into(),
            }],
        },
        file: CommunitiesFile {
            communities: vec![("aa".into(), "rr".into()), ("bb".into(), "rr".into())],
            relevance: Vec::new(),
            states: vec![
                comp("S1", &[("aa", "rr", 0.7), ("bb", "rr", 0.3)]),
                comp("S2", &[("aa", "rr", 1.0)]),
            ],
        },
        map: MapFile {
            meta: MapMeta {
                width: 1,
                height: 1,
                projection: "miller".into(),
                lat_top: 84.0,
                lat_bottom: -58.0,
                sources: Vec::new(),
            },
            provinces: Vec::new(),
            states: vec![state(1, "S1"), state(2, "S2")],
        },
    }
}

fn error(d: &Data) -> CommunityError {
    Communities::build(&d.cultures, &d.religions, &d.file, &d.map).unwrap_err()
}

#[test]
fn the_fixture_is_valid() {
    let d = tiny();
    let c = built(&d);
    let rows = c.rows(&RowRule::PROPOSED, &BTreeMap::new());
    assert_eq!(rows[0].rows.len(), 2);
    assert_eq!(rows[1].rows.len(), 1);
    assert!(rows[1].others.is_empty());
}

#[test]
fn refuses_a_broken_reference() {
    let mut d = tiny();
    d.file.communities.push(("zz".into(), "rr".into()));
    assert_eq!(
        error(&d),
        CommunityError::BrokenCommunity {
            culture: "zz".into(),
            religion: "rr".into()
        }
    );
    let mut d = tiny();
    d.cultures.cultures[1].family = "elvish".into();
    assert!(matches!(
        error(&d),
        CommunityError::UnknownCultureFamily { .. }
    ));
}

#[test]
fn refuses_a_state_without_composition() {
    let mut d = tiny();
    d.file.states.pop();
    assert_eq!(error(&d), CommunityError::MissingState("S2".into()));
    let mut d = tiny();
    d.file.states[1].shares.clear();
    assert_eq!(error(&d), CommunityError::EmptyState("S2".into()));
    let mut d = tiny();
    d.file.states[1].state = "S9".into();
    assert_eq!(error(&d), CommunityError::UnknownState("S9".into()));
}

#[test]
fn refuses_shares_that_do_not_sum_to_one() {
    let mut d = tiny();
    d.file.states[0].shares[1].2 = 0.2;
    assert!(matches!(
        error(&d),
        CommunityError::SharesDoNotSumToOne { ref state, .. } if state == "S1"
    ));
    let mut d = tiny();
    d.file.states[0].shares[1].2 = -0.3;
    assert!(matches!(error(&d), CommunityError::ShareOutOfRange { .. }));
}

#[test]
fn refuses_a_community_outside_the_list() {
    let mut d = tiny();
    d.file.states[1].shares = vec![("bb".into(), "xx".into(), 1.0)];
    assert_eq!(
        error(&d),
        CommunityError::UnknownCommunity {
            state: "S2".into(),
            culture: "bb".into(),
            religion: "xx".into(),
        }
    );
    // And a listed community that lives nowhere.
    let mut d = tiny();
    d.file.states[0].shares = vec![("aa".into(), "rr".into(), 1.0)];
    assert_eq!(
        error(&d),
        CommunityError::UnusedCommunity {
            culture: "bb".into(),
            religion: "rr".into()
        }
    );
}

// ---------------------------------------------------------------- names

#[test]
fn every_culture_and_religion_has_a_name_in_every_language() {
    let d = data();
    for lang in ["pt-BR", "en"] {
        let text = std::fs::read_to_string(
            repo_root().join(format!("assets/i18n/{lang}/communities.ftl")),
        )
        .unwrap();
        let messages: BTreeMap<&str, &str> = text
            .lines()
            .filter_map(|l| l.split_once(" = "))
            .map(|(id, value)| (id.trim(), value.trim()))
            .collect();
        let mut expected: BTreeSet<String> = BTreeSet::new();
        expected.extend(
            d.cultures
                .cultures
                .iter()
                .map(|c| format!("culture-{}", c.key)),
        );
        expected.extend(
            d.religions
                .religions
                .iter()
                .map(|r| format!("religion-{}", r.key)),
        );
        expected.extend(
            CULTURE_FAMILIES
                .iter()
                .map(|f| format!("culture-family-{f}")),
        );
        expected.extend(
            RELIGION_FAMILIES
                .iter()
                .map(|f| format!("religion-family-{f}")),
        );
        for id in &expected {
            assert!(
                messages.get(id.as_str()).is_some_and(|v| !v.is_empty()),
                "{lang} lacks {id}"
            );
        }
        for id in messages.keys() {
            if id.starts_with("culture-") || id.starts_with("religion-") {
                assert!(expected.contains(*id), "{lang}: {id} names nothing");
            }
        }
    }
}
