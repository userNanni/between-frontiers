//! Searching the buildings: by a building's or a state's name, without accents, and by place
//! (metropole or colonies), in the economy window; and by name in the state panel. The filtered
//! kinds sum their states again.

use bf_client::app::Transition;
use bf_client::buildings_view::{KindRow, StateRow};
use bf_client::saves;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;

mod common;
use common::layout::*;

fn state(id: u16, colony: bool, levels: i64, utilization: f64) -> StateRow {
    StateRow {
        region: u32::from(id),
        state: id,
        colony,
        levels,
        building: 1,
        method: "m".into(),
        workers: 10.0 * levels as f64,
        output: 100.0 * levels as f64,
        profit: 5.0 * levels as f64,
        utilization,
        staffing: 1.0,
        inputs: 1.0,
    }
}

#[test]
fn a_kind_over_some_of_its_states_sums_them_again() {
    let k = KindRow {
        kind: 1,
        key: "steel_mill".into(),
        group: 2,
        public: false,
        state_firm: false,
        levels: 4,
        building: 2,
        workers: 40.0,
        output: 400.0,
        profit: 20.0,
        utilization: 0.75,
        staffing: 1.0,
        states: vec![state(1, false, 3, 1.0), state(2, true, 1, 0.0)],
    };
    let home = k.over(|s| !s.colony).unwrap();
    assert_eq!((home.levels, home.building, home.states.len()), (3, 1, 1));
    assert_eq!(home.output, 300.0);
    assert_eq!(home.utilization, 1.0);
    assert!(k.over(|s| s.state == 9).is_none());
    assert_eq!(bf_client::text::folded("São Paulo"), "sao paulo");
}

#[test]
fn the_buildings_are_found_by_name_and_place() {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let mut session =
        bf_sim::game::Session::new(data.new_world(), 7, Date::new(1925, 1, 1), 1, Some(rules));
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "GBR".into() });
    while session.clock().date < Date::new(1925, 2, 3) {
        session.sim.step();
    }
    let mut failures = Vec::new();
    for (locale, find, colonies, bank) in [
        ("pt-BR", "banco", "Colônias", "+ Banco"),
        ("en", "bank", "Colonies", "+ Bank"),
    ] {
        let user = std::env::temp_dir().join(format!("bf-search-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save =
            saves::create(&user.join("saves"), "Busca", 7, Date::new(1925, 1, 1)).unwrap();
        save.meta.player = Some("GBR".into());
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "search").unwrap(),
        )
        .unwrap();
        saves::write(&save).unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        h.run_steps(4);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(4);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::E);
        h.run_steps(4);
        // The colonies only.
        match h
            .query_all(egui_kittest::kittest::by().label(colonies))
            .next()
        {
            Some(n) => n.click(),
            None => failures.push(format!("{locale}: no colonies' filter")),
        }
        h.run_steps(4);
        // A name typed: only its kind stays.
        match h
            .query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
            .next()
        {
            Some(n) => {
                n.focus();
                n.type_text(find);
            }
            None => failures.push(format!("{locale}: no search field")),
        }
        h.run_steps(4);
        let rows: Vec<String> = nodes(&h)
            .into_iter()
            .map(|n| n.label)
            .filter(|l| l.starts_with("+ ") || l.starts_with("− "))
            .collect();
        if rows.is_empty() || rows.iter().any(|r| !r.starts_with(bank)) {
            failures.push(format!("{locale}: rows after the search {rows:?}"));
        }
        for (w, hh) in [(960.0, 540.0), (1280.0, 720.0), (3840.0, 2160.0)] {
            h.set_size(egui::vec2(w, hh));
            h.run_steps(4);
            let case = format!("{locale} {w}×{hh} search");
            failures.extend(check(&h, &case));
            failures.extend(no_overlapping_text(&h, &case));
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
