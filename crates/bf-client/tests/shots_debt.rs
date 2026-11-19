//! Prints of the public debt in the HUD.
//! `cargo test -p bf-client --release --test shots_debt -- --ignored`; images in
//! `target/screenshots/debt/`.

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
        .join("debt");
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
        saves::create(&user.join("saves"), "Dívida", 1925, Date::new(1925, 1, 1)).unwrap();
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

/// The public debt in sight: Britain in February 1926, the block "Public debt" beside the GDP in
/// the HUD's figures, with the share of GDP, who holds it and the interest of a year on hover.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn debt_in_sight() {
    let user = std::env::temp_dir().join(format!("bf-shots-debt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "GBR", Date::new(1926, 2, 1));
    for (locale, size, tag, title) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "Dívida pública: "),
        ("en", egui::vec2(1280.0, 720.0), "en-1280", "Public debt: "),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(title))
            .next()
            .expect("the debt's block in the HUD")
            .hover();
        settle(&mut h);
        h.run_steps(4);
        let path = out_dir().join(format!("290-debt-{tag}.png"));
        h.render().expect("render").save(&path).unwrap();
        println!("screenshot: {}", path.display());
    }
    let _ = std::fs::remove_dir_all(&user);
}
