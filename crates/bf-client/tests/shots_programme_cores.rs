//! Prints of the nuclear programme's cores and ore on the forces tab.
//! `cargo test -p bf-client --release --test shots_programme_cores -- --ignored`; images in
//! `target/screenshots/nuclear/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "nuclear";
const RULES: &str = "forces,crisis,war,market_stock";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The United States in March 1925 with a programme of 0.5 % of GDP: the core plants (none
/// yet), the cores on the market, and the uranium ore its mines dig and the market keeps.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn programme_cores() {
    let user = std::env::temp_dir().join(format!("bf-shots-cores-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches
        .with(RULES)
        .expect("switches");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let mut s = session("USA", Date::new(1925, 1, 8), switches);
    s.submit(bf_sim::game::GameCommand::NuclearProgramme {
        tag: "USA".into(),
        share: Some(5),
    });
    while s.clock().date < Date::new(1925, 3, 2) {
        s.sim.step();
    }
    for (locale, size, tag, ore) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Minério de urânio por ano",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Uranium ore a year",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Programa", "USA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press(egui::Key::T);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(ore))
            .next()
            .expect("the ore's line")
            .scroll_to_me();
        settle(&mut h);
        shot(&mut h, THEME, &format!("316-programme-cores-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
