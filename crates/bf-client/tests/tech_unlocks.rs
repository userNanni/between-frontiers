//! What a technology opens and what it takes, read from the shipped data: the tank brings the
//! armour into service, a search for a unit or a building finds the technology behind it, and
//! a late technology lists what a 1925 country still lacks, from the first.

use bf_client::screens::tech_unlocks::{leads_to, missing, search, units};
use bf_i18n::{I18n, Locale};
use bf_sim::research::Stage;

#[test]
fn the_shipped_tree_says_what_it_opens() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let data = bf_sim::data::GameData::load(&root).expect("game data");
    let rules = data.rules();
    let tree = rules.tree.as_ref().expect("the tree");
    let (military, _) = rules.war.as_ref().expect("the forces");
    let ix = |k: &str| tree.techs.iter().position(|t| t.key == k).expect(k);
    let tank = ix("tank");
    assert!(
        units(military, tank)
            .iter()
            .any(|(k, g)| k == "armor" && *g == 0),
        "the tank brings the armour into service"
    );
    let i18n = I18n::load(&root.join("assets/i18n"), Locale::En).unwrap();
    let found = search(&i18n, tree, &rules.catalogue, Some(military), "armour");
    assert!(
        found.contains(&tank),
        "a search for the unit finds the tank"
    );
    let pt = I18n::load(&root.join("assets/i18n"), Locale::PtBr).unwrap();
    let found = search(&pt, tree, &rules.catalogue, Some(military), "blindados");
    assert!(
        found.contains(&tank),
        "accents and case aside, in Portuguese too"
    );
    assert!(search(&i18n, tree, &rules.catalogue, None, "  ").is_empty());
    // Every requirement leads to what requires it.
    for (t, tech) in tree.techs.iter().enumerate() {
        for &r in &tech.requires {
            assert!(leads_to(tree, r as usize).contains(&t));
        }
    }
    // A late technology from nothing: all its ancestors, the earliest first, none known.
    let stages = vec![Stage::Unknown; tree.techs.len()];
    let last = (0..tree.techs.len())
        .max_by_key(|&t| tree.techs[t].level)
        .unwrap();
    let path = missing(tree, &stages, last);
    assert!(path.len() > 5, "a late technology needs many");
    assert!(
        path.windows(2)
            .all(|w| tree.techs[w[0]].level <= tree.techs[w[1]].level)
    );
    // With every requirement known, nothing is missing.
    let known = vec![Stage::Science; tree.techs.len()];
    assert!(missing(tree, &known, last).is_empty());
}
