//! What the screenshot tests share, one file per theme (`shots_<theme>.rs`), so that two
//! threads adding prints never append to the same file. A theme's file includes this one with
//! `#[path = "common/shots.rs"] mod shots;` and writes its images to
//! `target/screenshots/<theme>/` (or `BF_SCREENSHOT_DIR/<theme>/`).
//!
//! Run a theme with `cargo test -p bf-client --release --test shots_<theme> -- --ignored`.
//! Needs a GPU or a software Vulkan driver.
#![allow(dead_code)]

use std::path::PathBuf;

use bf_client::paths::Paths;
use bf_client::{App, saves};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Where a theme's images go.
pub fn out_dir(theme: &str) -> PathBuf {
    let dir = std::env::var_os("BF_SCREENSHOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/screenshots"))
        .join(theme);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Renders the harness and writes `<theme>/<name>.png`.
pub fn shot(h: &mut Harness<'_, App>, theme: &str, name: &str) {
    h.run_steps(4);
    let image = h.render().expect("render");
    let path = out_dir(theme).join(format!("{name}.png"));
    image.save(&path).unwrap();
    println!("screenshot: {}", path.display());
}

/// The app at `size`, in `locale`, with its user folder at `user`; autosave off.
pub fn harness(user: PathBuf, locale: &str, size: egui::Vec2) -> Harness<'static, App> {
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

/// A campaign of `tag` run from 1925 to `until`, with rules of `data/rules.ron` switched on.
pub fn session(tag: &str, until: Date, switches: bf_sim::game::Switches) -> bf_sim::game::Session {
    let mut data = bf_sim::data::GameData::load(&root()).expect("game data");
    data.switches = switches;
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
    session
}

/// Writes `session` as a campaign of `tag` named `name` under `user`.
pub fn write_campaign(
    user: &std::path::Path,
    session: &bf_sim::game::Session,
    name: &str,
    tag: &str,
) -> saves::Save {
    let mut save = saves::create(&user.join("saves"), name, 1925, Date::new(1925, 1, 1)).unwrap();
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

/// Opens `save` in the harness.
pub fn open(h: &mut Harness<'_, App>, save: saves::Save) {
    let ctx = h.ctx.clone();
    h.state_mut()
        .apply(bf_client::app::Transition::Game(save), &ctx);
    h.run_steps(4);
}

/// Clicks the first element whose label contains `part`; false when there is none. A click
/// reaches only what is on screen.
pub fn click_containing(h: &mut Harness<'_, App>, part: &str) -> bool {
    let found = match h
        .query_all(egui_kittest::kittest::by().label_contains(part))
        .next()
    {
        Some(n) => {
            n.click();
            true
        }
        None => false,
    };
    if found {
        h.run_steps(4);
    }
    found
}
