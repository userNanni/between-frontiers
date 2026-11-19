//! The economy window's methods tab fits every supported logical screen, in both languages.

mod common;
use common::layout::*;

use bf_client::app::Transition;
use bf_client::saves;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;

#[test]
fn the_methods_tab_fits() {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let mut session =
        bf_sim::game::Session::new(data.new_world(), 7, Date::new(1925, 1, 1), 1, Some(rules));
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "GBR".into() });
    while session.clock().date < Date::new(1925, 2, 3) {
        session.sim.step();
    }
    let mut failures = Vec::new();
    for (locale, tab, rule) in [
        ("pt-BR", "Métodos", "Os donos escolheriam hoje"),
        ("en", "Methods", "Owners would pick today"),
    ] {
        let user =
            std::env::temp_dir().join(format!("bf-matrix-methods-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save =
            saves::create(&user.join("saves"), "Métodos", 7, Date::new(1925, 1, 1)).unwrap();
        save.meta.player = Some("GBR".into());
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "methods").unwrap(),
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
        let found = match h.query_all(egui_kittest::kittest::by().label(tab)).next() {
            Some(n) => {
                n.click();
                true
            }
            None => false,
        };
        h.run_steps(4);
        if !found {
            failures.push(format!("{locale}: no methods tab"));
        }
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            h.run_steps(4);
            let case = format!("{locale} {w}×{hh} methods");
            if !nodes(&h).iter().any(|n| n.label.contains(rule)) {
                failures.push(format!("{case}: no owners' pick"));
            }
            failures.extend(check(&h, &case));
            failures.extend(no_overlapping_text(&h, &case));
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
