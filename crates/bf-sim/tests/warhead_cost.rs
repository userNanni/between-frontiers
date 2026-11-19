//! A warhead costs the ore it took (`data/military/units.ron`, "nuclear_warhead";
//! `docs/war.pt-BR.md`, "Arma nuclear na escada de crise"): a Little Boy core took 12 to 24 t of
//! natural uranium, 14.1 to 28.3 t of U3O8.

use std::path::PathBuf;

use bf_sim::data::GameData;

#[test]
fn a_warhead_costs_the_ore_of_a_little_boy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_forces();
    let mil = &d.war.as_ref().unwrap().military;
    let cat = &d.catalogue;
    let fuel = cat.good("nuclear_fuel").unwrap();
    let ore = cat.good("uranium_ore").unwrap();
    // The ore per kg enriched of the enrichment plant's first method.
    let plant = &cat.buildings[cat.building("nuclear_fuel_plant").unwrap()];
    let m = &plant.methods[0];
    let input = m.inputs.iter().find(|(g, _)| *g == ore).unwrap().1;
    let output = m.outputs.iter().find(|(g, _)| *g == fuel).unwrap().1;
    let ratio = input.to_f64_lossy() / output.to_f64_lossy();
    let ty = mil.unit("nuclear_warhead").unwrap();
    for generation in 0..mil.types[ty as usize].generations.len() {
        let enriched = mil
            .spec(ty, generation)
            .equipment
            .iter()
            .find(|(g, _)| *g == fuel)
            .unwrap()
            .1
            .to_f64_lossy();
        let t = enriched * ratio / 1000.0;
        eprintln!(
            "generation {generation}: {enriched} kg enriched, {t:.1} t of U3O8 (ratio {ratio:.2})"
        );
        assert!((14.0..=28.5).contains(&t), "{t} t of U3O8");
    }
}
