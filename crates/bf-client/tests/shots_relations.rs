//! Prints of the relations' measures in the country panel, with the switch `relations` on.
//! `cargo test -p bf-client --release --test shots_relations -- --ignored`; images in
//! `target/screenshots/relations/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "relations";
const RULES: &str = "relations";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// Germany played in March 1925: its own power, role and threat, and the countries most hostile
/// to it, in the country panel the banner opens. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn relations_in_the_country_panel() {
    let user = std::env::temp_dir().join(format!("bf-shots-relations-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches
        .with(RULES)
        .expect("switches");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let s = session("GER", Date::new(1925, 3, 2), switches);
    for (locale, size, tag, line, source) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Poder (CINC)",
            "Fonte:",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Power (CINC)",
            "Source:",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Relações", "GER");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        let banner = egui::pos2(54.0, 42.0);
        h.hover_at(banner);
        h.run_steps(1);
        h.drag_at(banner);
        h.run_steps(1);
        h.drop_at(banner);
        h.hover_at(egui::pos2(size.x / 2.0, size.y - 100.0));
        settle(&mut h);
        // The measures come with the month's reading, below the country's figures: bring the
        // figures' last line into sight, then wait for them.
        for _ in 0..400 {
            if let Some(n) = h
                .query_all(egui_kittest::kittest::by().label_contains(source))
                .last()
            {
                n.scroll_to_me();
            }
            if h.query_all(egui_kittest::kittest::by().label_contains(line))
                .next()
                .is_some()
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
            h.run_steps(1);
        }
        h.query_all(egui_kittest::kittest::by().label_contains(line))
            .last()
            .expect("the relations' list")
            .scroll_to_me();
        settle(&mut h);
        shot(&mut h, THEME, &format!("334-relations-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
