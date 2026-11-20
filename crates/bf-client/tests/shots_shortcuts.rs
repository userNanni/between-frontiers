//! Prints of the shortcuts you can find: the keys on the buttons and tabs, and the sheet (F1).
//! `cargo test -p bf-client --release --test shots_shortcuts -- --ignored`; images in
//! `target/screenshots/shortcuts/`.

#[path = "common/shots.rs"]
mod shots;

use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "shortcuts";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// Britain in February 1926: the government open, each button of the top bar and each tab with
/// its own key in a corner; then the sheet of keys (F1) over it, and the sheet searched.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn shortcuts_in_sight() {
    let user = std::env::temp_dir().join(format!("bf-shots-shortcuts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "GBR",
        Date::new(1926, 2, 1),
        bf_sim::data::GameData::load(&root())
            .expect("game data")
            .switches,
    );
    for (locale, size, tag, query) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "governo"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280", "government"),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Atalhos", "GBR");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        shot(&mut h, THEME, &format!("296-keys-on-buttons-{tag}"));
        h.key_press(egui::Key::F1);
        settle(&mut h);
        shot(&mut h, THEME, &format!("297-keys-sheet-{tag}"));
        h.event(egui::Event::Text(query.into()));
        settle(&mut h);
        shot(&mut h, THEME, &format!("298-keys-sheet-search-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The first campaign, Brazil in January 1925: the key tips lit by themselves, with the banner
/// saying what they are and the shortcuts' chip below; then the chip alone once they go out.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn first_campaign_tips() {
    let user = std::env::temp_dir().join(format!("bf-shots-first-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "BRA",
        Date::new(1925, 1, 8),
        bf_sim::data::GameData::load(&root())
            .expect("game data")
            .switches,
    );
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Primeira", "BRA");
        let mut h = harness(home, locale, size);
        h.state_mut().shared.first_tips = true;
        open(&mut h, save);
        settle(&mut h);
        shot(&mut h, THEME, &format!("299-first-tips-{tag}"));
        h.key_press(egui::Key::Escape);
        settle(&mut h);
        shot(&mut h, THEME, &format!("300-shortcuts-chip-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
