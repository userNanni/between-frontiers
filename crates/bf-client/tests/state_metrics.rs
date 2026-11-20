//! Metrics by state: a month in, every state lens colours the player's states, and the economy
//! window's states tab lists them; a line selects the state and closes the window.

mod common;

use bf_client::app::Transition;
use bf_client::map::lens::Lens;
use bf_client::saves;
use bf_client::screens::Screen;
use common::layout::{app, root};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;

fn steps(h: &mut Harness<'_, bf_client::App>) {
    for _ in 0..8 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
}

#[test]
fn the_states_have_their_metrics() {
    let user = std::env::temp_dir().join(format!("bf-state-metrics-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    // Sweden a month and a half in: the states' figures of the month are read.
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "SWE".into() });
    while session.clock().date < Date::new(1925, 2, 15) {
        session.sim.step();
    }
    let mut save =
        saves::create(&user.join("saves"), "Estados", 1925, Date::new(1925, 1, 1)).unwrap();
    save.meta.player = Some("SWE".into());
    save.meta.date = session.clock().date;
    std::fs::write(
        save.dir.join(saves::WORLD_FILE),
        session.save("test", "Estados").unwrap(),
    )
    .unwrap();
    saves::write(&save).unwrap();
    let mut h = app(user.clone(), "en", egui::vec2(1600.0, 900.0));
    h.ctx.options_mut(|o| o.warn_on_id_clash = true);
    h.run_steps(4);
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    let started = std::time::Instant::now();
    while h
        .state()
        .shared
        .world
        .as_ref()
        .is_none_or(|w| w.live_states.is_none())
        && started.elapsed().as_secs() < 30
    {
        steps(&mut h);
    }
    let world = h.state().shared.world.as_ref().expect("the world");
    assert!(world.live_states.is_some(), "the month's states");
    let swe = world
        .countries
        .iter()
        .position(|c| c.tag() == "SWE")
        .unwrap();
    let paper = bf_client::map::lens::palette(world, Lens::Political, 1925);
    for m in 0..bf_client::state_metric::METRICS.len() {
        let pal = bf_client::map::lens::palette(world, Lens::State(m), 1925);
        let coloured = world.countries[swe]
            .provinces
            .iter()
            .filter(|&&p| pal[usize::from(p)] != paper[usize::from(p)])
            .count();
        assert!(coloured > 0, "metric {m} colours Sweden's states");
    }
    // The economy window's states tab: Sweden's states, a line selects one.
    h.key_press(egui::Key::N);
    steps(&mut h);
    h.query_all(egui_kittest::kittest::by().label("States"))
        .next()
        .expect("the states tab")
        .click();
    steps(&mut h);
    let first = {
        let w = h.state().shared.world.as_ref().unwrap();
        let s = w.countries[swe].states[0];
        bf_client::text::state_name(&h.state().shared.i18n, &w.states[s].def)
    };
    h.query_all(egui_kittest::kittest::by().label(first.as_str()))
        .next()
        .unwrap_or_else(|| panic!("the line of {first}"))
        .click();
    steps(&mut h);
    match &h.state().screen {
        Screen::Game(g) => {
            assert!(g.selected_state().is_some(), "a state selected");
            assert!(!g.open_windows().contains(&"economy"), "the window closed");
        }
        _ => panic!("not in a campaign"),
    }
    let _ = std::fs::remove_dir_all(&user);
}
