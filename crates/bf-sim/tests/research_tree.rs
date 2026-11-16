//! The technology tree loads against the economy catalogue, covers every method and good that
//! did not exist in 1925, has names in every language, keeps its dates inside its eras and adds
//! up to a century of plausible productivity growth.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::economy::BUILDING_GROUPS;
use bf_sim::research::{EFFECTS, ERAS, FIELDS, Stage, TechTree, TechnologiesFile};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load() -> (GameData, TechnologiesFile) {
    let data = GameData::load(&root()).unwrap();
    let text = std::fs::read_to_string(root().join("data/research/technologies.ron")).unwrap();
    let file: TechnologiesFile = ron::from_str(&text).unwrap();
    (data, file)
}

fn tree(data: &GameData) -> &TechTree {
    data.tree.as_ref().expect("data/research/technologies.ron")
}

/// First year after the era, the end of the game for the last one.
fn era_end(era: u8) -> i32 {
    ERAS.get(usize::from(era) + 1).copied().unwrap_or(2025)
}

#[test]
fn tree_builds_and_seeds_1925() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    assert!(!tree.techs.is_empty());
    let world = data.new_world();
    let r = &world.research;
    assert_eq!(r.stage.len(), data.countries.countries.len());
    // Whatever is industrial somewhere in 1925 is recorded as such.
    for (t, tech) in tree.techs.iter().enumerate() {
        let anywhere = r.stage.iter().any(|c| c[t] >= Stage::Industrial);
        assert_eq!(
            anywhere,
            r.firsts[t][Stage::Industrial as usize] == 1925,
            "{}",
            tech.key
        );
    }
}

#[test]
fn every_method_newer_than_1925_has_a_technology() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    let mut missing = Vec::new();
    for (b, building) in data.catalogue.buildings.iter().enumerate() {
        for (m, method) in building.methods.iter().enumerate() {
            if method.available_from > 1925 && tree.method_tech[b][m].is_none() {
                missing.push(format!("{}.{}", building.key, method.key));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "methods without a technology: {missing:?}"
    );
}

#[test]
fn the_first_method_of_a_1925_building_stays_free() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    for (b, building) in data.catalogue.buildings.iter().enumerate() {
        let first = &building.methods[0];
        if first.available_from <= 1925 {
            assert!(
                tree.method_tech[b][0].is_none(),
                "{}.{} is how the building works in 1925; no technology may lock it",
                building.key,
                first.key
            );
        }
    }
}

#[test]
fn every_good_newer_than_1925_has_a_technology() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    let missing: Vec<_> = data
        .catalogue
        .goods
        .iter()
        .enumerate()
        .filter(|(g, good)| good.available_from > 1925 && tree.good_techs[*g].is_empty())
        .map(|(_, good)| good.key.as_str())
        .collect();
    assert!(
        missing.is_empty(),
        "goods without a technology: {missing:?}"
    );
}

#[test]
fn every_field_and_era_has_technologies() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    for (f, field) in FIELDS.iter().enumerate() {
        assert!(
            tree.techs.iter().any(|t| usize::from(t.field) == f),
            "no technology in {field}"
        );
    }
    for era in 0..ERAS.len() {
        let n = tree
            .techs
            .iter()
            .filter(|t| usize::from(t.era) == era)
            .count();
        assert!(n >= 10, "era {era} has only {n} technologies");
    }
    for (e, effect) in EFFECTS.iter().enumerate() {
        assert!(
            tree.techs
                .iter()
                .any(|t| t.effects.iter().any(|(x, _)| usize::from(*x) == e)),
            "no technology changes {effect}"
        );
    }
}

#[test]
fn every_technology_has_a_name_and_description_in_every_language() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    for lang in ["pt-BR", "en"] {
        let text = std::fs::read_to_string(root().join(format!("assets/i18n/{lang}/research.ftl")))
            .unwrap();
        let messages: BTreeMap<&str, &str> = text
            .lines()
            .filter_map(|l| l.split_once(" = "))
            .map(|(id, value)| (id.trim(), value.trim()))
            .collect();
        for t in &tree.techs {
            for id in [format!("tech-{}", t.key), format!("tech-{}-desc", t.key)] {
                assert!(
                    messages.get(id.as_str()).is_some_and(|v| !v.is_empty()),
                    "{lang} lacks {id}"
                );
            }
        }
        // No name left behind by a technology that was removed or renamed.
        for id in messages.keys() {
            let Some(rest) = id.strip_prefix("tech-") else {
                continue;
            };
            if ["field-", "era-", "stage-"]
                .iter()
                .any(|p| rest.starts_with(p))
            {
                continue;
            }
            let key = rest.strip_suffix("-desc").unwrap_or(rest);
            assert!(tree.tech(key).is_some(), "{lang}: {id} names no technology");
        }
    }
}

#[test]
fn dates_fall_inside_the_era() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    // A breakthrough may be shown a few years before its era or reach industry a few years after.
    const SLACK: i32 = 5;
    for t in &tree.techs {
        assert!(
            (1850..=2025).contains(&t.demonstrated) && t.industrialized <= 2025,
            "{}: dates out of the game",
            t.key
        );
        if t.era == 0 {
            assert!(
                t.demonstrated < 1925,
                "{}: era 0 but shown in {}",
                t.key,
                t.demonstrated
            );
        } else {
            let start = ERAS[usize::from(t.era)];
            let end = era_end(t.era);
            assert!(
                t.demonstrated < end + SLACK && t.industrialized >= start - SLACK,
                "{}: {}–{} misses era {} ({start}–{end})",
                t.key,
                t.demonstrated,
                t.industrialized,
                t.era
            );
        }
        assert!(
            t.industrialized - t.demonstrated <= 30,
            "{}: thirty years from prototype to industry",
            t.key
        );
    }
}

#[test]
fn costs_grow_with_the_era() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    let ranges = [
        (20, 60),
        (60, 150),
        (150, 400),
        (400, 1000),
        (1000, 2500),
        (2500, 6000),
    ];
    for t in &tree.techs {
        let (lo, hi) = ranges[usize::from(t.era)];
        assert!(
            (lo..=hi).contains(&t.cost),
            "{}: cost {} outside era {} ({lo}–{hi})",
            t.key,
            t.cost,
            t.era
        );
    }
}

#[test]
fn requirements_come_from_the_same_era_or_before() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    for t in &tree.techs {
        for &r in &t.requires {
            let req = &tree.techs[usize::from(r)];
            assert!(req.era <= t.era, "{} requires the later {}", t.key, req.key);
        }
    }
}

#[test]
fn the_1925_start_is_consistent() {
    let (data, file) = load();
    let tags: BTreeSet<&str> = data
        .countries
        .countries
        .iter()
        .map(|c| c.tag.as_str())
        .collect();
    for t in &file.technologies {
        match &t.start {
            None => assert!(t.era > 0, "{}: era 0 needs a 1925 stage", t.key),
            Some(start) => {
                if t.era > 0 {
                    // Nothing after 1925 was more than an idea when the game starts.
                    assert_eq!(start.default, Stage::Unknown, "{}", t.key);
                    assert!(
                        start.countries.iter().all(|(_, s)| *s == Stage::Science),
                        "{}: only the science stage before its era",
                        t.key
                    );
                }
                let mut seen = BTreeSet::new();
                for (tag, _) in &start.countries {
                    assert!(
                        tags.contains(tag.as_str()),
                        "{}: unknown country {tag}",
                        t.key
                    );
                    assert!(seen.insert(tag), "{}: {tag} twice", t.key);
                }
            }
        }
    }
}

/// Over 1925–2025 the frontier gains about 1.5 % a year: sum ln(1 + g) of the eras 1–5
/// technologies of a group lands near 1.55 (x4.7). Services gain much less (Baumol).
#[test]
fn productivity_adds_up_to_a_century_of_growth() {
    let data = GameData::load(&root()).unwrap();
    let tree = tree(&data);
    let mut sums: BTreeMap<&str, f64> = BUILDING_GROUPS.iter().map(|g| (*g, 0.0)).collect();
    for t in tree.techs.iter().filter(|t| t.era > 0) {
        for (group, per_mille) in &t.productivity {
            *sums.get_mut(group.as_str()).unwrap() += (f64::from(*per_mille) / 1000.0).ln_1p();
        }
    }
    for (group, sum) in sums {
        let range = if group == "services" {
            0.4..=0.6
        } else {
            1.4..=1.7
        };
        assert!(
            range.contains(&sum),
            "{group}: sum ln(1+g) = {sum:.3} (x{:.2}) outside {range:?}",
            sum.exp()
        );
    }
}
