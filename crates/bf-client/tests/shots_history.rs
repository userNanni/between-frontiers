//! Prints of the figures' charts: the closed years of a figure, on hover of the HUD and of the
//! country panel.
//! `cargo test -p bf-client --release --test shots_history -- --ignored`; images in
//! `target/screenshots/history/`.

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
        .join("history");
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
        saves::create(&user.join("saves"), "Séries", 1925, Date::new(1925, 1, 1)).unwrap();
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

/// The figures' charts: Sweden (no colonies) in February 1928, three years closed. The HUD's
/// GDP and literacy with the chart of their closed years, the lowest, the highest and the
/// change of the last year; then the country panel's population, with the same chart.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn figure_history() {
    let user = std::env::temp_dir().join(format!("bf-shots-history-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "SWE", Date::new(1928, 2, 1));
    for (locale, size, tag, blocks, population) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            ["PIB: ", "Alfabetização: "],
            "POPULAÇÃO",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            ["GDP: ", "Literacy: "],
            "POPULATION",
        ),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        for (k, block) in blocks.iter().enumerate() {
            h.query_all(egui_kittest::kittest::by().label_contains(block))
                .next()
                .unwrap_or_else(|| panic!("the block {block}"))
                .hover();
            settle(&mut h);
            save_shot(&mut h, &format!("291-history-{k}-{tag}"));
        }
        // C opens the country panel; its population, on hover.
        h.key_press(egui::Key::C);
        settle(&mut h);
        let at = h
            .query_all(egui_kittest::kittest::by().label(population))
            .last()
            .expect("the country panel's population")
            .rect()
            .center();
        h.hover_at(at);
        settle(&mut h);
        save_shot(&mut h, &format!("291-history-2-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
