//! The technology tree loads against the economy catalogue and every country gets a 1925 state.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::research::Stage;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn tree_builds_and_seeds_1925() {
    let data = GameData::load(&root()).unwrap();
    let tree = data.tree.as_ref().expect("data/research/technologies.ron");
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
