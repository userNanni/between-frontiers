//! The views read the country played now: a campaign begun with Britain and switched to Germany
//! ("Play another country") shows Germany's agreements and tariffs, not Britain's: the views read
//! the country played now (`Session::player`), which the world follows since the switch moves the
//! control.

use bf_client::app::Transition;
use bf_client::saves;
use bf_client::screens::Screen;
use bf_sim::game::GameCommand;
use laplace_schelling::calendar::Date;

mod common;
use common::layout::*;

#[test]
fn the_views_follow_the_country_switched_to() {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut s = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "GBR".into() });
    while s.clock().date < Date::new(1925, 1, 10) {
        s.sim.step();
    }
    s.submit(GameCommand::SwitchCountry { tag: "GER".into() });
    s.sim.step();
    assert_eq!(s.player(), Some("GER"));
    assert_eq!(
        s.world().player_tag(),
        Some("GER"),
        "the world follows the switch (the control moves with it)"
    );
    let user = std::env::temp_dir().join(format!("bf-played-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut save = saves::create(&user.join("saves"), "Troca", 7, Date::new(1925, 1, 1)).unwrap();
    save.meta.player = Some("GER".into());
    save.meta.date = s.clock().date;
    std::fs::write(
        save.dir.join(saves::WORLD_FILE),
        s.save("test", "switch").unwrap(),
    )
    .unwrap();
    saves::write(&save).unwrap();
    let mut h = app(user.clone(), "pt-BR", egui::vec2(1600.0, 900.0));
    h.run_steps(4);
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    let snap = |h: &egui_kittest::Harness<'_, bf_client::App>| match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    };
    let started = std::time::Instant::now();
    while (snap(&h).agreements.is_none() || snap(&h).tariffs.is_none())
        && started.elapsed().as_secs() < 30
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(1);
    }
    let s = snap(&h);
    assert_eq!(s.agreements.as_ref().map(|a| a.me.as_str()), Some("GER"));
    assert_eq!(s.tariffs.as_ref().map(|t| t.tag.as_str()), Some("GER"));
    let _ = std::fs::remove_dir_all(&user);
}
