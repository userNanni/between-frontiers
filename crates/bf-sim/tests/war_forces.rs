//! The forces in peace (phase 5, slice A) on the real scenario: the soldiers of the pops are the
//! men of the formations, people are conserved when they are stationed, the reserve reports and
//! goes home on command, the size in peace is measured, and the old rule comes back by data.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::{prof, workers};
use bf_sim::game::{GameCommand, Session};
use bf_sim::war::{SoldierRule, men, station_soldiers};
use laplace_schelling::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut d = GameData::load(&root()).unwrap_or_else(|e| panic!("{e}"));
        d.enable_forces();
        d
    })
}

fn population(w: &bf_sim::game::GameWorld) -> i64 {
    w.econ.regions.iter().map(|r| r.population()).sum()
}

fn index(w: &bf_sim::game::GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
}

#[test]
fn the_soldiers_are_the_men_of_the_formations() {
    let d = data();
    let w = d.new_world();
    assert!(!w.war.is_empty());
    let mil = &d.war.as_ref().unwrap().military;
    let formations: i64 = w
        .war
        .countries
        .iter()
        .flat_map(|c| &c.formations)
        .map(|f| men(mil, f))
        .sum();
    assert_eq!(w.war.soldiers.iter().sum::<i64>(), formations);
    // Every region holds its soldiers, to the rounding of a pop into workers.
    for (r, want) in w.econ.regions.iter().zip(&w.war.soldiers) {
        assert!((workers(r.pops[prof::SOLDIER].size) - want).abs() <= 1);
    }
    // France keeps its colonial army in its colonies.
    let fra = index(&w, "FRA");
    let colonial: i64 = w
        .econ
        .regions
        .iter()
        .zip(&w.war.soldiers)
        .filter(|(r, _)| r.market as usize == fra && r.colony)
        .map(|(_, s)| *s)
        .sum();
    assert_eq!(colonial, 107_157);
}

#[test]
fn stationing_the_soldiers_conserves_people() {
    let d = data();
    let mut w = d.new_world();
    let before = population(&w);
    for c in &mut w.war.countries {
        for f in &mut c.formations {
            f.elements = f.elements / 2;
        }
    }
    station_soldiers(&d.war.as_ref().unwrap().military, &mut w);
    assert_eq!(population(&w), before);
    for (r, want) in w.econ.regions.iter().zip(&w.war.soldiers) {
        assert!((workers(r.pops[prof::SOLDIER].size) - want).abs() <= 1);
    }
}

#[test]
fn the_reserve_reports_and_goes_home() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    let che = index(s.world(), "CHE");
    let mil = &d.war.as_ref().unwrap().military;
    let total = |w: &bf_sim::game::GameWorld| {
        let c = &w.war.countries[che];
        c.reserve as i64
            + c.formations
                .iter()
                .filter(|f| f.called_up)
                .map(|f| men(mil, f))
                .sum::<i64>()
    };
    assert_eq!(s.world().war.countries[che].reserve, 500_000);
    s.submit(GameCommand::Mobilize {
        tag: "CHE".into(),
        on: true,
    });
    s.sim.run_until(Date::new(1925, 2, 15));
    let w = s.world();
    assert!(w.war.countries[che].reserve < 500_000);
    assert!(w.war.countries[che].formations.iter().any(|f| f.called_up));
    assert!((total(w) - 500_000).abs() <= 1);
    s.submit(GameCommand::Mobilize {
        tag: "CHE".into(),
        on: false,
    });
    s.sim.run_until(Date::new(1925, 2, 25));
    let w = s.world();
    assert!(!w.war.countries[che].formations.iter().any(|f| f.called_up));
    assert!((w.war.countries[che].reserve as i64 - 500_000).abs() <= 1);
}

#[test]
fn the_size_in_peace_is_measured_the_first_january() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1926, 1, 15));
    let w = s.world();
    for tag in ["FRA", "GBR", "USA"] {
        let c = &w.war.countries[index(w, tag)];
        assert!(c.peace_share.is_positive(), "{tag}");
        assert!(c.law_then > 0, "{tag}");
    }
    // Germany under Versailles keeps a small army.
    let ger = &w.war.countries[index(w, "GER")];
    let fra = &w.war.countries[index(w, "FRA")];
    assert!(ger.peace_share < fra.peace_share);
}

/// The forces ship off: `peace.ron` keeps the rule from before them, and the world is the one
/// of before the forces until they are switched on.
#[test]
fn the_forces_ship_off_and_switch_on() {
    let d = GameData::load(&root()).unwrap();
    assert_eq!(
        d.war.as_ref().unwrap().peace.as_ref().unwrap().soldiers,
        SoldierRule::PopulationShare(10)
    );
    let w = d.new_world();
    assert!(w.war.is_empty());
    assert!(d.rules().war.is_none());
    assert!(!data().new_world().war.is_empty());
    assert!(data().rules().war.is_some());
}

#[test]
fn the_whole_military_budget_is_spent() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1925, 3, 3));
    let w = s.world();
    let cat = &d.catalogue;
    let weight: i64 = cat
        .state_basket
        .iter()
        .filter(|(g, _)| cat.goods[*g as usize].category == "military")
        .map(|(_, w)| *w)
        .sum();
    for tag in ["GBR", "FRA", "USA", "JAP"] {
        let ci = index(w, tag);
        let m = &w.econ.markets[ci];
        let room = bf_sim::econ::public::goods_budget(m)
            .max(laplace_schelling::Fixed::ZERO)
            .mul_ratio(weight, 1000)
            .to_f64_lossy();
        let cost: f64 = w.war.countries[ci]
            .orders
            .iter()
            .map(|(g, q)| (*q * m.price[*g as usize]).to_f64_lossy())
            .sum();
        // Prices and the budget move between the week the orders were set and now.
        assert!((cost / room - 1.0).abs() < 0.05, "{tag}: {cost} of {room}");
    }
}

#[test]
fn a_force_moves_to_the_next_generation_a_share_at_a_time() {
    let d = data();
    let mut w = d.new_world();
    let tree = d.tree.as_ref().unwrap();
    let gbr = index(&w, "GBR");
    let t = tree.tech("arms_mass_production").unwrap();
    w.research.stage[gbr][t] = bf_sim::research::Stage::Industrial;
    let mil = &d.war.as_ref().unwrap().military;
    let inf = mil.unit("infantry").unwrap();
    let land = |w: &bf_sim::game::GameWorld, generation: u8| -> i64 {
        w.war.countries[gbr]
            .formations
            .iter()
            .filter(|f| f.unit == inf && f.generation == generation && !f.called_up)
            .map(|f| men(mil, f))
            .sum()
    };
    let before = land(&w, 0);
    let mut s = Session::new(w, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    let (old, new) = (land(w, 0), land(w, 1));
    assert!(new > 0, "some infantry moved");
    // Gradual: at most a tenth a year, so about 4 % in five months.
    assert!(new < before / 20, "{new} of {before}");
    assert!((old + new - before).abs() <= 2, "men are kept");
}

#[test]
fn equipment_beyond_the_needs_goes_to_the_depot_and_reservists_draw_on_it() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1926, 1, 6));
    let mil = &d.war.as_ref().unwrap().military;
    let gbr = index(s.world(), "GBR");
    let fill = |f: &bf_sim::war::Formation| {
        let spec = mil.spec(f.unit, f.generation as usize);
        let need: Vec<(u16, laplace_schelling::Fixed)> = spec
            .equipment
            .iter()
            .map(|(g, q)| (*g, *q * f.elements))
            .collect();
        mil.value_lots(&f.equipment).to_f64_lossy() / mil.value_lots(&need).to_f64_lossy()
    };
    {
        let c = &s.world().war.countries[gbr];
        assert!(!c.depot.is_empty(), "the rest of the budget is in store");
        for f in &c.formations {
            // At most the allowance: no rifles piled on the infantry.
            assert!(fill(f) <= 1.0001, "{f:?}");
        }
    }
    s.submit(GameCommand::Mobilize {
        tag: "GBR".into(),
        on: true,
    });
    s.sim.run_until(Date::new(1926, 2, 15));
    let c = &s.world().war.countries[gbr];
    let called = c.formations.iter().find(|f| f.called_up).unwrap();
    // The reservists took rifles from the depot as they reported, before any purchase arrived.
    assert!(fill(called) > 0.2, "{}", fill(called));
}
