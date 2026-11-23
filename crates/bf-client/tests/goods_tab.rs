//! The economy window's goods tab: a month in, the market's goods are listed; the search
//! narrows them; a good opens on who makes it, and "See in Buildings" goes to the buildings
//! tab on the maker.

mod common;

use bf_client::app::Transition;
use bf_client::saves;
use bf_client::screens::Screen;
use bf_client::screens::economy::Tab;
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
fn the_market_is_read_good_by_good() {
    let user = std::env::temp_dir().join(format!("bf-goods-tab-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
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
    let mut save = saves::create(&user.join("saves"), "Bens", 1925, Date::new(1925, 1, 1)).unwrap();
    save.meta.player = Some("SWE".into());
    save.meta.date = session.clock().date;
    std::fs::write(
        save.dir.join(saves::WORLD_FILE),
        session.save("test", "Bens").unwrap(),
    )
    .unwrap();
    saves::write(&save).unwrap();
    let mut h = app(user.clone(), "en", egui::vec2(1600.0, 900.0));
    h.ctx.options_mut(|o| o.warn_on_id_clash = true);
    h.run_steps(4);
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    let started = std::time::Instant::now();
    while h.state().shared.world.is_none() && started.elapsed().as_secs() < 30 {
        steps(&mut h);
    }
    steps(&mut h);
    h.key_press(egui::Key::N);
    steps(&mut h);
    h.get_by_label("Goods").click();
    steps(&mut h);
    // Iron ore is listed; the search narrows the list to it.
    assert!(
        h.query_by_label("Iron ore").is_some(),
        "the goods are listed"
    );
    h.query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
        .next()
        .expect("the search")
        .click();
    steps(&mut h);
    h.event(egui::Event::Text("iron".into()));
    steps(&mut h);
    assert!(h.query_by_label("Grain").is_none(), "the search narrows");
    h.get_by_label("Iron ore").click();
    steps(&mut h);
    h.query_all(egui_kittest::kittest::by().label_contains("see in Buildings"))
        .next()
        .expect("a maker of iron ore")
        .click();
    steps(&mut h);
    match &h.state().screen {
        Screen::Game(g) => {
            let (tab, open) = g.economy_tab().expect("the economy window");
            assert_eq!(tab, Tab::Buildings, "the buildings tab");
            assert!(open.is_some(), "on the maker");
        }
        _ => panic!("not in a campaign"),
    }
    let _ = std::fs::remove_dir_all(&user);
}
