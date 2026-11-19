//! No good is asked for before it exists: for every unit generation, the goods of its equipment
//! and upkeep are available (`available_from`) by the historical year of the technologies that
//! free it (the latest `industrialized` of its requirements, 1925 at the earliest); and for every
//! method a technology frees, its inputs are available by that technology's `industrialized`
//! year. A good asked for too early blocks what needs it (the fission warhead's electronic
//! components of 1950 kept every bomb from being built before then).

use std::path::PathBuf;

use bf_sim::data::GameData;

#[test]
fn no_good_is_asked_for_before_it_exists() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_forces();
    let cat = &d.catalogue;
    let tree = d.tree.as_ref().unwrap();
    let mil = &d.war.as_ref().unwrap().military;
    let mut bad = Vec::new();
    for (ti, ty) in mil.types.iter().enumerate() {
        for (g, spec) in ty.generations.iter().enumerate() {
            let year = spec
                .requires
                .iter()
                .map(|&t| tree.techs[t as usize].industrialized)
                .max()
                .unwrap_or(1925)
                .max(1925);
            let _ = ti;
            for (good, _) in spec.equipment.iter().chain(&spec.upkeep) {
                let from = cat.goods[*good as usize].available_from;
                if from > year {
                    bad.push(format!(
                        "unit {} generation {g} ({year}): {} from {from}",
                        ty.key, cat.goods[*good as usize].key
                    ));
                }
            }
        }
    }
    for t in &tree.techs {
        for &(b, m) in &t.methods {
            let building = &cat.buildings[b as usize];
            let method = &building.methods[m as usize];
            for (good, _) in &method.inputs {
                let from = cat.goods[*good as usize].available_from;
                if from > t.industrialized.max(1925) {
                    bad.push(format!(
                        "method {}.{} of {} ({}): {} from {from}",
                        building.key,
                        method.key,
                        t.key,
                        t.industrialized,
                        cat.goods[*good as usize].key
                    ));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "asked for before it exists:\n{}",
        bad.join("\n")
    );
}
