//! A warhead costs the ore of its core (U2, U3; the owner, 2026-11-19; `data/military/units.ron`,
//! "nuclear_warhead"; `data/economy/buildings.ron`, "weapon_core_plant"): one fission core by
//! high enrichment takes the ore of a Little Boy core (64 kg at about 80 %: 14.1 to 28.3 t of U3O8
//! with tails of 0.3 to 0.5 %; here 14.7 t, by the 1945 plant of the data), by the production reactor that of a Fat Man
//! core (6.15 kg of plutonium at about 0.9 kg a tonne of natural uranium: about 8 t of U3O8); the
//! thermonuclear generation takes less of either and yields more.

use std::path::PathBuf;

use bf_sim::data::GameData;

#[test]
fn a_core_costs_the_ore_of_its_route_and_each_generation_less() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_forces();
    let mil = &d.war.as_ref().unwrap().military;
    let cat = &d.catalogue;
    let cores = cat.good("weapon_cores").unwrap();
    let ore = cat.good("uranium_ore").unwrap();
    let plant = &cat.buildings[cat.building("weapon_core_plant").unwrap()];
    let per_core = |m: usize| {
        let m = &plant.methods[m];
        let input = m.inputs.iter().find(|(g, _)| *g == ore).unwrap().1;
        let output = m.outputs.iter().find(|(g, _)| *g == cores).unwrap().1;
        input.to_f64_lossy() / output.to_f64_lossy() / 1000.0
    };
    let (heu, pu) = (per_core(0), per_core(1));
    eprintln!("t of U3O8 a core: high enrichment {heu:.1}, production reactor {pu:.1}");
    assert!((12.0..=16.0).contains(&heu), "{heu}");
    assert!((7.0..=9.0).contains(&pu), "{pu}");
    assert!(pu < heu / 1.5, "the plutonium route saves ore");
    let ty = mil.unit("nuclear_warhead").unwrap();
    let take = |g: usize| {
        mil.spec(ty, g)
            .equipment
            .iter()
            .find(|(x, _)| *x == cores)
            .unwrap()
            .1
            .to_f64_lossy()
    };
    let (s0, s1) = (mil.spec(ty, 0), mil.spec(ty, 1));
    eprintln!(
        "cores a warhead: {} then {}; equivalent megatons per mille: {} then {}",
        take(0),
        take(1),
        s0.emt_mille(),
        s1.emt_mille()
    );
    assert_eq!(take(0), 1.0);
    assert!(take(1) < take(0), "each generation takes less");
    // 21 kt is 0.076 equivalent megatons; 1.4 Mt is 1.25.
    assert_eq!(s0.emt_mille(), 76);
    assert_eq!(s1.emt_mille(), 1251);
}
