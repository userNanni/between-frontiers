//! The economy catalogue is consistent: every reference resolves, every land use and deposit
//! has its building, every method only uses goods that exist by its year, and every good is
//! made by something and used by something.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bf_sim::economy::{
    BUILDING_GROUPS, BuildingsFile, GOOD_CATEGORIES, GoodsFile, NeedsFile, PROFESSIONS,
};
use bf_sim::resources::{FARMS, RESOURCES};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load<T: serde::de::DeserializeOwned>(path: &str) -> T {
    let text = std::fs::read_to_string(root().join(path)).unwrap();
    ron::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"))
}

struct Catalogue {
    goods: GoodsFile,
    buildings: BuildingsFile,
    needs: NeedsFile,
}

fn catalogue() -> Catalogue {
    Catalogue {
        goods: load("data/economy/goods.ron"),
        buildings: load("data/economy/buildings.ron"),
        needs: load("data/economy/needs.ron"),
    }
}

#[test]
fn references_resolve() {
    let c = catalogue();
    let mut since = BTreeMap::new();
    for g in &c.goods.goods {
        assert!(
            GOOD_CATEGORIES.contains(&g.category.as_str()),
            "{}: category {}",
            g.key,
            g.category
        );
        assert!(
            since.insert(g.key.as_str(), g.available_from).is_none(),
            "duplicate good {}",
            g.key
        );
        assert!(g.base_price > 0, "{}", g.key);
    }
    let mut keys = BTreeSet::new();
    for b in &c.buildings.buildings {
        assert!(keys.insert(b.key.as_str()), "duplicate building {}", b.key);
        assert!(
            BUILDING_GROUPS.contains(&b.group.as_str()),
            "{}: group",
            b.key
        );
        assert!(!b.methods.is_empty(), "{} has no method", b.key);
        for m in &b.methods {
            for (good, qty) in m.inputs.iter().chain(&m.outputs) {
                let from = since
                    .get(good.as_str())
                    .unwrap_or_else(|| panic!("{}/{}: unknown good {good}", b.key, m.key));
                assert!(
                    *from <= m.available_from,
                    "{}/{} ({}) uses {good}, available from {from}",
                    b.key,
                    m.key,
                    m.available_from
                );
                assert!(*qty > 0.0, "{}/{}: {good}", b.key, m.key);
            }
            assert!(!m.outputs.is_empty(), "{}/{} makes nothing", b.key, m.key);
            for (p, n) in &m.jobs {
                assert!(
                    PROFESSIONS.contains(&p.as_str()),
                    "{}/{}: profession {p}",
                    b.key,
                    m.key
                );
                assert!(*n > 0);
            }
        }
    }
    for n in &c.needs.needs {
        assert!((1..=4).contains(&n.tier), "{}", n.key);
        for (good, w) in &n.goods {
            assert!(
                since.contains_key(good.as_str()),
                "need {}: unknown good {good}",
                n.key
            );
            assert!(*w > 0.0);
        }
    }
}

#[test]
fn every_land_use_and_deposit_has_its_building() {
    let c = catalogue();
    for farm in FARMS {
        let n = c
            .buildings
            .buildings
            .iter()
            .filter(|b| b.farm.as_deref() == Some(farm))
            .count();
        assert_eq!(n, 1, "land use {farm}");
    }
    for res in RESOURCES {
        let n = c
            .buildings
            .buildings
            .iter()
            .filter(|b| b.resource.as_deref() == Some(res))
            .count();
        assert!(n >= 1, "deposit {res} has no building");
    }
}

#[test]
fn every_good_is_made_and_used() {
    let c = catalogue();
    let made: BTreeSet<&str> = c
        .buildings
        .buildings
        .iter()
        .flat_map(|b| &b.methods)
        .flat_map(|m| &m.outputs)
        .map(|(g, _)| g.as_str())
        .collect();
    let used: BTreeSet<&str> = c
        .buildings
        .buildings
        .iter()
        .flat_map(|b| &b.methods)
        .flat_map(|m| &m.inputs)
        .map(|(g, _)| g.as_str())
        .chain(
            c.needs
                .needs
                .iter()
                .flat_map(|n| &n.goods)
                .map(|(g, _)| g.as_str()),
        )
        .collect();
    // Armies and the state consume these from the war and politics phases on; the state buys
    // what public services make, and construction is bought by the queue of works.
    let consumed_later = ["military", "capital", "public"];
    for g in &c.goods.goods {
        assert!(made.contains(g.key.as_str()), "nothing makes {}", g.key);
        if !consumed_later.contains(&g.category.as_str()) {
            assert!(used.contains(g.key.as_str()), "nothing uses {}", g.key);
        }
    }
}
