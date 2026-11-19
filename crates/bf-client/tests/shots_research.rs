//! Prints of the research screen: what a technology opens, and the way back from a unit.
//! `cargo test -p bf-client --release --test shots_research -- --ignored`; images in
//! `target/screenshots/research/`.
use std::path::PathBuf;

use bf_client::app::Transition;
use bf_client::paths::Paths;
use bf_client::{App, saves};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn out_dir() -> PathBuf {
    let dir = std::env::var_os("BF_SCREENSHOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/screenshots"))
        .join("research");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn harness(user: PathBuf, locale: &str, size: egui::Vec2) -> Harness<'static, App> {
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        format!("(locale: \"{locale}\", ui_scale: 100, map_labels: true, autosave: Off)"),
    )
    .unwrap();
    let paths = Paths::new(&root(), user.clone(), user);
    let mut h = Harness::builder()
        .with_size(size)
        .wgpu()
        .build_eframe(move |cc| App::new(cc, paths));
    h.set_render_cursor(false);
    h
}

/// A campaign of `tag` run from 1925 to `until` with the shipped rules, written under `user`.
fn campaign(user: &std::path::Path, tag: &str, until: Date) -> saves::Save {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: tag.into() });
    while session.clock().date < until {
        session.sim.step();
    }
    let mut save =
        saves::create(&user.join("saves"), "Pesquisa", 1925, Date::new(1925, 1, 1)).unwrap();
    save.meta.player = Some(tag.into());
    save.meta.date = session.clock().date;
    std::fs::write(
        save.dir.join(saves::WORLD_FILE),
        session.save("test", &save.meta.name).unwrap(),
    )
    .unwrap();
    saves::write(&save).unwrap();
    save
}

fn settle(h: &mut Harness<'_, App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

fn save_shot(h: &mut Harness<'_, App>, name: &str) {
    h.run_steps(4);
    let path = out_dir().join(format!("{name}.png"));
    h.render().expect("render").save(&path).unwrap();
    println!("screenshot: {}", path.display());
}

/// The way back: Britain in February 1926 searches for the armour; the tree lights the tank
/// and centres it, and its tip says what it opens (the armour), what it leads to, its gains and
/// what Britain still has to research first; then the atomic bomb, far ahead, with the
/// technologies missing on the way. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn research_unlocks() {
    let user = std::env::temp_dir().join(format!("bf-shots-research-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "GBR", Date::new(1926, 2, 1));
    for (locale, size, tag, query, tank, bomb) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "blindados",
            "Carro de combate",
            "Bomba atômica",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "armour",
            "Tank",
            "Atomic bomb",
        ),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        h.key_press(egui::Key::R);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
            .next()
            .expect("the search field")
            .click();
        h.run_steps(2);
        h.event(egui::Event::Text(query.into()));
        settle(&mut h);
        let at = h
            .query_all(egui_kittest::kittest::by().label(tank))
            .next()
            .expect("the tank's card")
            .rect()
            .center();
        h.hover_at(at);
        settle(&mut h);
        save_shot(&mut h, &format!("294-research-{tag}"));
        // A technology far ahead: what Britain still has to research first.
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text(bomb.into()));
        settle(&mut h);
        let at = h
            .query_all(egui_kittest::kittest::by().label(bomb))
            .next()
            .expect("the bomb's card")
            .rect()
            .center();
        h.hover_at(at);
        settle(&mut h);
        save_shot(&mut h, &format!("295-research-path-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
