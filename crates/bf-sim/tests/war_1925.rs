//! Invariants of the war data: `data/military/units.ron` and `rules.ron` against the economy
//! catalogue and the technology tree, and `data/scenario/1925/forces.ron` against them and the
//! countries. Each rule of `ForcesFile::validate` is also shown to refuse a broken copy, and the
//! pure functions are run on the real catalogue.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::war::{
    Activity, Domain, ForcesFile, LandField, Military, Posture, RulesFile, Segment, Terrain,
    TerrainMix, UnitsFile, allocate, land_step,
};
use laplace_schelling::Fixed;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| GameData::load(&repo_root()).unwrap_or_else(|e| panic!("{e}")))
}

fn military() -> &'static Military {
    &data()
        .war
        .as_ref()
        .expect("data/military/units.ron")
        .military
}

fn forces() -> &'static ForcesFile {
    data()
        .war
        .as_ref()
        .and_then(|w| w.forces_1925.as_ref())
        .expect("data/scenario/1925/forces.ron")
}

fn read<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Errors of `validate` on a copy of the forces changed by `edit`.
fn errors_after(edit: impl FnOnce(&mut ForcesFile)) -> Vec<String> {
    let mut file = forces().clone();
    edit(&mut file);
    let d = data();
    file.validate(military(), d.tree.as_ref().unwrap(), &d.countries)
        .err()
        .unwrap_or_default()
}

fn assert_refused(errors: &[String], needle: &str) {
    assert!(
        errors.iter().any(|e| e.contains(needle)),
        "expected an error containing {needle:?}, got {errors:?}"
    );
}

#[test]
fn the_catalogue_has_the_designed_types() {
    let m = military();
    let counts: Vec<usize> = Domain::ALL.iter().map(|d| m.count(*d)).collect();
    assert_eq!(counts, vec![11, 9, 8, 5], "land, sea, air, strategic");
    for t in &m.types {
        // Every later generation needs strictly more technologies than the one before.
        for w in t.generations.windows(2) {
            assert!(w[1].requires.len() > w[0].requires.len(), "{}", t.key);
        }
    }
}

#[test]
fn broken_catalogues_are_refused() {
    let d = data();
    let units: UnitsFile = read("data/military/units.ron");
    let rules: RulesFile = read("data/military/rules.ron");
    let build = |u: &UnitsFile, r: &RulesFile| {
        Military::build(u, r, &d.catalogue, d.tree.as_ref().unwrap())
            .err()
            .unwrap_or_default()
    };
    let mut u = units.clone();
    u.types[0].equipment[0].0 = "laser_rifles".into();
    assert_refused(&build(&u, &rules), "unknown good laser_rifles");
    let mut u = units.clone();
    u.types[2].requires = vec!["warp_drive".into()];
    assert_refused(&build(&u, &rules), "unknown technology warp_drive");
    let mut u = units.clone();
    u.types[2].men = 900;
    assert_refused(&build(&u, &rules), "a land element is 1,000 men");
    let mut u = units.clone();
    u.types[1].upkeep.push(("coffee".into(), Fixed::ONE));
    assert_refused(&build(&u, &rules), "no weight in tonnes_per_unit");
    let mut r = rules.clone();
    r.terrain.pop();
    assert_refused(&build(&units, &r), "every terrain class");
}

#[test]
fn the_1925_forces_are_valid_and_cover_the_powers() {
    let f = forces();
    let m = military();
    let tags: BTreeSet<&str> = f.countries.iter().map(|c| c.tag.as_str()).collect();
    // The great powers and the twenty largest economies of the scenario that the sources cover;
    // Switzerland's militia, Australia's and South Africa's part-time forces wait for a reserve rule
    // (docs/war.pt-BR.md, "Lacunas").
    for tag in [
        "USA", "GBR", "FRA", "GER", "ITA", "JAP", "SOV", "CHN", "RAJ", "POL", "ESP", "CAN", "ARG",
        "NLD", "CZE", "BEL", "MEX", "BRA", "AUT",
    ] {
        assert!(tags.contains(tag), "{tag} has no forces");
        assert!(f.total(m, tag, Domain::Land) > 0, "{tag} has no army");
    }
    for tag in ["USA", "GBR", "JAP", "FRA", "ITA"] {
        assert!(f.total(m, tag, Domain::Sea) > 0, "{tag} has no navy");
        assert!(f.total(m, tag, Domain::Air) > 0, "{tag} has no air force");
    }
    // Facts the sources state.
    assert_eq!(
        f.total(m, "GER", Domain::Air),
        0,
        "Versailles: no air force"
    );
    let ger = f.country("GER").unwrap();
    assert!(ger.rows.iter().all(|r| r.unit != "submarine"));
    let capital = |tag: &str| {
        f.country(tag)
            .unwrap()
            .rows
            .iter()
            .filter(|r| r.unit == "battleship")
            .map(|r| r.count)
            .sum::<u64>()
    };
    // Washington treaty, chapter II part 1: the capital ships retained.
    assert_eq!(capital("USA"), 18);
    assert_eq!(capital("GBR"), 22);
    assert_eq!(capital("JAP"), 10);
    let (estimated, sourced) = f.estimated_and_sourced();
    assert!(
        estimated * 4 < sourced,
        "{estimated} estimated rows against {sourced}"
    );
}

#[test]
fn broken_forces_are_refused() {
    assert_refused(
        &errors_after(|f| f.countries[0].rows[0].unit = "death_ray".into()),
        "unknown unit type",
    );
    // A drone in 1925 needs technologies nobody has.
    assert_refused(
        &errors_after(|f| f.countries[0].rows[0].unit = "drone".into()),
        "unknown in USA on the start date",
    );
    assert_refused(
        &errors_after(|f| f.countries[0].rows[0].count = 0),
        "count is zero",
    );
    assert_refused(
        &errors_after(|f| f.countries[0].rows[0].source.clear()),
        "no source",
    );
    assert_refused(
        &errors_after(|f| f.countries[0].tag = "XXX".into()),
        "not a country of the scenario",
    );
    assert_refused(
        &errors_after(|f| {
            let c = f.countries[0].clone();
            f.countries.push(c);
        }),
        "listed twice",
    );
    assert_refused(
        &errors_after(|f| {
            let c = f.countries.iter_mut().find(|c| c.tag == "CHN").unwrap();
            c.rows[0].station = bf_sim::war::Station::Colonies;
        }),
        "colonies the country does not have",
    );
    assert_refused(
        &errors_after(|f| {
            let c = f.countries.iter_mut().find(|c| c.tag == "GBR").unwrap();
            let row = c.rows.iter_mut().find(|r| r.unit == "battleship").unwrap();
            row.reserve = true;
        }),
        "only land forces have a reserve",
    );
    assert_refused(
        &errors_after(|f| f.start_date = "1926-01-01".into()),
        "start_date",
    );
}

#[test]
fn costs_on_the_real_catalogue() {
    let m = military();
    let inf = m.unit("infantry").unwrap();
    let raise = m.raise_cost(inf, 0, 10);
    assert_eq!(raise.men, 10_000);
    let ammo = m.rules.ammunition;
    let peace = m.upkeep(inf, 0, 10, Activity::Peace, false);
    let combat = m.upkeep(inf, 0, 10, Activity::Combat, false);
    let of = |list: &[(u16, Fixed)], g| list.iter().find(|(k, _)| *k == g).unwrap().1;
    assert_eq!(
        of(&combat, ammo),
        of(&peace, ammo) * m.rules.combat_ammunition
    );
    assert!(m.tonnes(&combat) > m.tonnes(&peace));
    // One element of 1925 infantry is the reference: its equipment term is one.
    let side = m.land_side(inf, 0, 1);
    assert_eq!(side.fire, m.rules.reference_fire);
    // Later generations carry more fire.
    let fire: Vec<Fixed> = (0..3).map(|g| m.land_side(inf, g, 1).fire).collect();
    assert!(fire[0] < fire[1] && fire[1] < fire[2]);
    // Armour protects, infantry does not.
    let armor = m.unit("armor").unwrap();
    assert!(m.land_side(armor, 0, 1).protection.is_positive());
    assert_eq!(side.protection, Fixed::ZERO);
}

#[test]
fn the_1925_forces_cost_something_to_keep() {
    let m = military();
    let f = forces();
    let mut tonnes = Fixed::ZERO;
    for c in &f.countries {
        for r in &c.rows {
            let ty = m.unit(&r.unit).unwrap();
            tonnes += m.tonnes(&m.upkeep(ty, 0, r.elements(m), Activity::Peace, false));
        }
    }
    assert!(tonnes.is_positive());
}

#[test]
fn every_climate_of_the_map_has_a_terrain() {
    #[derive(serde::Deserialize)]
    struct Land {
        states: Vec<State>,
    }
    #[derive(serde::Deserialize)]
    struct State {
        climate: Vec<(String, f32)>,
    }
    let land: Land = read("data/scenario/1925/land.ron");
    let rules = &military().rules;
    let climates: BTreeSet<String> = land
        .states
        .iter()
        .flat_map(|s| s.climate.iter().map(|(k, _)| k.clone()))
        .collect();
    for c in climates {
        assert!(rules.climate(&c).is_some(), "climate {c} has no terrain");
    }
}

#[test]
fn a_1925_battle_on_the_real_rules() {
    let m = military();
    let inf = m.unit("infantry").unwrap();
    let mut attacker = m.land_side(inf, 0, 30);
    attacker.posture = Posture::Attack;
    let defender = m.land_side(inf, 0, 10);
    let field = LandField {
        terrain: TerrainMix::only(Terrain::Open),
        winter: false,
        river: Fixed::ZERO,
    };
    let o = land_step(&m.rules, &field, [&attacker, &defender]);
    assert!(o.advance_km.is_positive());
    assert!(o.losses[1] > 0 && o.losses[1] <= defender.men);
    let ratios = o.terms[0].over(&o.terms[1]);
    assert_eq!(ratios[0].1, Some(Fixed::from_int(3)), "number");
    // The same attack into mountains, against prepared defences, stops.
    let mut dug_in = defender.clone();
    dug_in.fortification = Fixed::ONE;
    let hills = LandField {
        terrain: TerrainMix::only(Terrain::Mountain),
        winter: true,
        river: Fixed::ZERO,
    };
    let o = land_step(&m.rules, &hills, [&attacker, &dug_in]);
    assert_eq!(o.advance_km, Fixed::ZERO);
}

#[test]
fn a_front_keeps_its_split_from_week_to_week() {
    let rules = &military().rules;
    let segments: Vec<Segment> = (0..40)
        .map(|i| Segment {
            length_km: Fixed::from_int(50 + (i * 37) % 120),
            enemy_power: Fixed::from_int(5 + (i * 13) % 40),
            objective: i % 9 == 0,
        })
        .collect();
    let first = allocate(rules, 1500, &segments);
    assert_eq!(first.iter().map(|s| s.elements).sum::<u64>(), 1500);
    for _ in 0..52 {
        assert_eq!(allocate(rules, 1500, &segments), first);
    }
}

#[test]
fn the_reserve_is_apart_from_the_army_in_service() {
    let f = forces();
    let m = military();
    // Switzerland: a militia of 500,000 mobilisable men and no standing army.
    assert_eq!(f.reserve("CHE"), 500_000);
    assert_eq!(f.total(m, "CHE", Domain::Land), 0);
    // The National Guard and the Organized Reserves do not count in the Regular Army.
    assert_eq!(f.reserve("USA"), 160_784 + 76_959);
    assert_eq!(f.total(m, "USA", Domain::Land), 133_264);
    assert_eq!(f.reserve("GBR"), 187_419 + 92_000);
    assert_eq!(f.reserve("GER"), 0);
    // Called up, it reports over the weeks of the rule, less trained than the army.
    let c = bf_sim::war::call_up(&m.rules, f.reserve("CHE"), 2);
    assert_eq!(c.reported + c.waiting, 500_000);
    assert!(c.reported > 0 && c.waiting > 0);
    assert!(c.training < Fixed::ONE);
}
