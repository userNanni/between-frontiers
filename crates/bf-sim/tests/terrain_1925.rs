//! The terrain of the states (`assets/map/terrain.ron`) against what the map should show: the
//! Alps, the Himalaya and the Andes are mountain, the North European Plain and the steppes are
//! not, Tibet is high, Polesie and the Bengal delta are marsh, London is town, and the Rhine runs
//! between Alsace and Baden.

use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::war::{Terrain, TerrainFile, state_terrain, urban_share};
use laplace_schelling::Fixed;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        GameData::load(&root).unwrap_or_else(|e| panic!("{e}"))
    })
}

fn terrain() -> &'static TerrainFile {
    data()
        .war
        .as_ref()
        .and_then(|w| w.terrain.as_ref())
        .expect("assets/map/terrain.ron")
}

fn id(key: &str) -> u16 {
    data()
        .map
        .states
        .iter()
        .find(|s| s.key == key)
        .unwrap_or_else(|| panic!("no state {key}"))
        .id
}

fn mountain(key: &str) -> u16 {
    terrain().state(id(key)).unwrap().mountain
}

fn marsh(key: &str) -> u16 {
    terrain().state(id(key)).unwrap().marsh
}

#[test]
fn every_state_has_a_row_in_id_order() {
    let t = terrain();
    assert_eq!(t.states.len(), data().map.states.len());
    for (i, s) in t.states.iter().enumerate() {
        assert_eq!(s.state as usize, i + 1);
        assert_eq!(s.key, data().map.states[i].key);
        assert!(s.mountain <= 1000 && s.marsh <= 1000);
    }
}

#[test]
fn the_great_ranges_are_mountain_and_the_plains_are_not() {
    for key in [
        "CHE-TICINO",
        "CHE-OSTSCHWEIZ",
        "AUT-7",
        "NPL",
        "IND-KASHMIR",
        "PER-CENTRO",
        "CHL-NORTE",
    ] {
        assert!(mountain(key) >= 500, "{key}: {}", mountain(key));
    }
    for key in [
        "DEU-BB",
        "POL-WARSZAWA",
        "POL-LODZ",
        "KAZ-NORTH",
        "KAZ-WEST",
        "ARG-BUENOS-AIRES",
    ] {
        assert!(mountain(key) <= 50, "{key}: {}", mountain(key));
    }
}

#[test]
fn tibet_is_high_and_the_low_countries_are_not() {
    let t = terrain();
    let height = |key: &str| t.state(id(key)).unwrap().elevation_m.unwrap();
    assert!(height("CHN-XZ") > 4000);
    assert!(height("NPL") > 1500);
    assert!(height("NLD-ZH") < 50);
    assert!(height("DEU-BB") < 200);
}

#[test]
fn marshes_and_towns_show_up() {
    assert!(marsh("POL-POLESIE") >= 150, "{}", marsh("POL-POLESIE"));
    assert!(marsh("BGD-DACCA") >= 300, "{}", marsh("BGD-DACCA"));
    assert!(marsh("KAZ-NORTH") <= 50);
    // London: town takes a large share of the state; the Kazakh steppe almost none.
    let d = data();
    let rules = &d.war.as_ref().unwrap().military.rules;
    let pop = d.population.as_ref().unwrap();
    let urban = |key: &str| {
        let p = pop.states.iter().find(|s| s.state == key).unwrap();
        let area = d.map.states[id(key) as usize - 1].area_km2;
        urban_share(rules, p.population, p.urban, area)
    };
    assert!(
        urban("GBR-LONDON") > Fixed::from_ratio(1, 10),
        "{}",
        urban("GBR-LONDON")
    );
    assert!(urban("KAZ-NORTH") < Fixed::from_ratio(1, 1000));
    let land = d.land.as_ref().unwrap();
    let london = land
        .states
        .iter()
        .find(|s| s.state == "GBR-LONDON")
        .unwrap();
    let mix = state_terrain(
        rules,
        terrain().state(id("GBR-LONDON")),
        &london.climate,
        0.0,
        urban("GBR-LONDON"),
    );
    assert!(
        mix.0
            .iter()
            .any(|(t, s)| *t == Terrain::Urban && *s == urban("GBR-LONDON"))
    );
}

#[test]
fn the_rhine_runs_between_alsace_and_baden() {
    let t = terrain();
    let (a, b) = (id("FRA-ALSACE"), id("DEU-BW"));
    assert_eq!(t.river_name(a, b), Some("Rhine"));
    assert!(t.river_share(a, b) > Fixed::from_ratio(9, 10));
    assert_eq!(t.river_share(a, b), t.river_share(b, a));
}
