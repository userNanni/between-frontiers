//! Print of the state's offices with computers on the Budget tab, with the switches
//! `computer_efficiency` and `fiscal_capacity` on.
//! `cargo test -p bf-client --release --test shots_admin -- --ignored`; images in
//! `target/screenshots/admin/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "admin";
const RULES: &str = "fiscal_capacity,computer_efficiency";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The United States in March 1925 given the electronic computer at the industrial stage (by
/// hand: the technology comes in the 1950s), two weeks on: what a servant collects with the
/// mainframe, and the computers the state bought (not saved: the client runs a week). Portuguese at 1600 × 900, English at
/// 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn offices_with_computers() {
    let user = std::env::temp_dir().join(format!("bf-shots-admin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let switches = data.switches.with(RULES).expect("switches");
    let tech = data
        .rules()
        .tree
        .as_ref()
        .expect("tree")
        .techs
        .iter()
        .position(|t| t.key == "electronic_computer")
        .expect("electronic computer");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let mut s = session("USA", Date::new(1925, 3, 2), switches);
    {
        let w = s.sim.world_mut();
        let usa = w
            .countries
            .iter()
            .position(|(_, c)| c.tag == "USA")
            .unwrap();
        w.research.stage[usa][tech] = bf_sim::research::Stage::Industrial;
    }
    let until = Date::new(1925, 3, 16);
    while s.clock().date < until {
        s.sim.step();
    }
    for (locale, size, tag, line) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Cada servidor arrecada",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Each servant collects",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Repartições", "USA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press(egui::Key::B);
        settle(&mut h);
        // The offices' outputs are not saved: run until a week has passed, then stop.
        h.key_press(egui::Key::Space);
        let mut found = false;
        for _ in 0..2400 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            h.run_steps(1);
            if h.query_all(egui_kittest::kittest::by().label_contains(line))
                .next()
                .is_some()
            {
                found = true;
                break;
            }
        }
        h.key_press(egui::Key::Space);
        settle(&mut h);
        assert!(found, "the offices' line");
        h.query_all(egui_kittest::kittest::by().label_contains(line))
            .next()
            .expect("the offices' line")
            .scroll_to_me();
        settle(&mut h);
        shot(&mut h, THEME, &format!("330-admin-computers-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
