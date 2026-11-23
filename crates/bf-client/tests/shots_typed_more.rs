//! Prints of the typed numbers on the steppers that came later: the nuclear programme's share of
//! GDP typed on the forces tab, and refused past its limit.
//! `cargo test -p bf-client --release --test shots_typed_more -- --ignored`; images in
//! `target/screenshots/typed/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "typed";
const RULES: &str = "forces,crisis,war";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The United States in March 1925 with a programme of 0.5 % of GDP: the share's field open
/// with 1,2 typed; then 50 typed and refused, with the limits said. Portuguese at 1600 × 900,
/// English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn typed_programme_share() {
    let user = std::env::temp_dir().join(format!("bf-shots-typed-more-{}", std::process::id()));
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
    for (locale, size, tag, tip) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "1,2"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280", "1.2"),
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
        let label = if locale == "en" {
            "A tenth of a point of GDP a year"
        } else {
            "Um décimo de ponto do PIB por ano"
        };
        h.get_by_label(label).scroll_to_me();
        settle(&mut h);
        h.get_by_label(label).click();
        settle(&mut h);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text(tip.into()));
        settle(&mut h);
        shot(&mut h, THEME, &format!("314-typed-programme-{tag}"));
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text("50".into()));
        h.run_steps(1);
        h.key_press(egui::Key::Enter);
        settle(&mut h);
        shot(&mut h, THEME, &format!("315-typed-programme-refused-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
