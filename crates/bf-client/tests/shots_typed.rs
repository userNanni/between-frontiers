//! Prints of the typed numbers: a stepper's value turned into a field.
//! `cargo test -p bf-client --release --test shots_typed -- --ignored`; images in
//! `target/screenshots/typed/`.

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
        .join("typed");
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
        saves::create(&user.join("saves"), "Tarifas", 1925, Date::new(1925, 1, 1)).unwrap();
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

/// Types the key tip of element `id`, the leader key first; scrolls down to find it.
fn act(h: &mut Harness<'_, App>, id: &str) {
    for _ in 0..8 {
        if !h.state().shared.keys.hint_mode() {
            h.key_press(egui::Key::AltLeft);
            h.run_steps(3);
        }
        if let Some(code) = h.state().shared.keys.code_of(id).map(str::to_owned) {
            for c in code.chars() {
                h.key_press(egui::Key::from_name(&c.to_string()).expect("a key for the code"));
                h.run_steps(4);
            }
            settle(h);
            if h.state().shared.keys.hint_mode() {
                h.key_press(egui::Key::Escape);
                h.run_steps(2);
            }
            return;
        }
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
        h.run_steps(4);
    }
    panic!("no key tip for {id}");
}

/// The rate of a good typed: Britain in February 1926, the trade tab with steel fixed, its
/// rate's field open with 37 typed; then 250 typed and refused, with the limits said.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn typed_tariff() {
    let user = std::env::temp_dir().join(format!("bf-shots-typed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "GBR", Date::new(1926, 2, 1));
    for (locale, size, tag, tip) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "De 0 % a 200 %"),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "From 0% to 200%",
        ),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::T);
        settle(&mut h);
        act(&mut h, "gov.tariff.cat.industrial");
        act(&mut h, "gov.tariff.good.steel.fixed");
        let value = h
            .query_all(egui_kittest::kittest::by().label_contains(tip))
            .last()
            .unwrap_or_else(|| panic!("the steel's rate"));
        value.scroll_to_me();
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(tip))
            .last()
            .unwrap()
            .click();
        settle(&mut h);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text("37".into()));
        settle(&mut h);
        save_shot(&mut h, &format!("292-typed-{tag}"));
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text("250".into()));
        h.run_steps(1);
        h.key_press(egui::Key::Enter);
        settle(&mut h);
        save_shot(&mut h, &format!("293-typed-refused-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
