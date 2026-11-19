//! The state panel fits every supported logical screen, in both languages: a state picked as a
//! click picks it, its panel in the country panel's place, Esc up to the country.

mod common;
use common::layout::*;

use bf_client::app::Transition;
use bf_client::saves;
use bf_client::screens::Screen;
use laplace_schelling::calendar::Date;

#[test]
fn the_state_panel_fits() {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let mut session =
        bf_sim::game::Session::new(data.new_world(), 7, Date::new(1925, 1, 1), 1, Some(rules));
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "BRA".into() });
    while session.clock().date < Date::new(1925, 2, 3) {
        session.sim.step();
    }
    let mut failures = Vec::new();
    for (locale, unemployment) in [("pt-BR", "Sem trabalho"), ("en", "Without work")] {
        let user =
            std::env::temp_dir().join(format!("bf-matrix-state-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save =
            saves::create(&user.join("saves"), "Estado", 7, Date::new(1925, 1, 1)).unwrap();
        save.meta.player = Some("BRA".into());
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "state").unwrap(),
        )
        .unwrap();
        saves::write(&save).unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        h.run_steps(4);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(4);
        {
            let app = h.state_mut();
            let world = app.shared.world.as_ref().expect("world");
            let c = world.countries.iter().find(|c| c.tag() == "BRA").unwrap();
            let state = world.provinces[usize::from(c.capital_province.unwrap()) - 1]
                .state
                .unwrap();
            match &mut app.screen {
                Screen::Game(g) => g.select_state(world, state),
                _ => panic!("not in a campaign"),
            }
        }
        h.run_steps(4);
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            h.run_steps(4);
            let case = format!("{locale} {w}×{hh} state");
            if !nodes(&h)
                .iter()
                .any(|n| n.label.eq_ignore_ascii_case(unemployment))
            {
                failures.push(format!("{case}: no state panel"));
            }
            failures.extend(check(&h, &case));
            failures.extend(no_overlapping_text(&h, &case));
        }
        // Esc goes up to the country panel.
        h.key_press(egui::Key::Escape);
        h.run_steps(4);
        if nodes(&h)
            .iter()
            .any(|n| n.label.eq_ignore_ascii_case(unemployment))
        {
            failures.push(format!("{locale}: Esc did not go up to the country"));
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
