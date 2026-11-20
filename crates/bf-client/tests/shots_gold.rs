//! Prints of the gold standard on the money tab: exchange and reserves with the rules on, the
//! second step of capital controls, and the tab as it was with them off.
//! `cargo test -p bf-client --release --test shots_gold -- --ignored`; images in
//! `target/screenshots/gold/`.

#[path = "common/shots.rs"]
mod shots;

use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "gold";
const RULES: &str = "payments_settle,gold_standard,capital_follows_rate";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The United States in February 1926, on gold: the money tab's exchange and reserves; capital
/// controls at their second step; and the same tab with the rules off, as it was. Portuguese
/// at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn gold_section() {
    let user = std::env::temp_dir().join(format!("bf-shots-gold-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let shipped = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    let on = session(
        "USA",
        Date::new(1926, 2, 1),
        shipped.with(RULES).expect("switches"),
    );
    let off = session("USA", Date::new(1926, 2, 1), shipped);
    for (locale, size, tag, ask) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Ligar o controle de capital",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Turn capital controls on",
        ),
    ] {
        // The client takes its switches from `data/rules.ron` and `BF_RULES`, not the save.
        // SAFETY: this file's only test, run on one thread.
        unsafe { std::env::set_var("BF_RULES", RULES) };
        let home = user.join(format!("on-{tag}"));
        let save = write_campaign(&home, &on, "Ouro", "USA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press(egui::Key::M);
        settle(&mut h);
        shot(&mut h, THEME, &format!("302-gold-top-{tag}"));
        let page = |h: &mut egui_kittest::Harness<'_, bf_client::App>, n: usize| {
            for _ in 0..n {
                h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
                h.run_steps(2);
            }
            settle(h);
        };
        page(&mut h, 1);
        shot(&mut h, THEME, &format!("303-gold-section-{tag}"));
        page(&mut h, 1);
        shot(&mut h, THEME, &format!("303-gold-reserves-{tag}"));
        page(&mut h, 6);
        assert!(click_containing(&mut h, ask), "the controls' first step");
        page(&mut h, 4);
        shot(&mut h, THEME, &format!("304-gold-controls-{tag}"));
        // The rules off: the tab as it was.
        // SAFETY: as above.
        unsafe { std::env::remove_var("BF_RULES") };
        let home = user.join(format!("off-{tag}"));
        let save = write_campaign(&home, &off, "Sem ouro", "USA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press(egui::Key::M);
        settle(&mut h);
        shot(&mut h, THEME, &format!("305-gold-off-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The Agendas' decision: the United States on gold, the choices to leave with what each means
/// this week, and the second step of floating; Britain (floating by the rule) with the choices
/// to adhere, the prewar parity's cover against the legal one. Portuguese at 1600 × 900,
/// English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn gold_decision() {
    let user = std::env::temp_dir().join(format!("bf-shots-gold-dec-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches
        .with(RULES)
        .expect("switches");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    for (tag_c, name, step) in [("USA", "306", true), ("GBR", "308", false)] {
        let s = session(tag_c, Date::new(1926, 2, 1), switches);
        for (locale, size, tag, open_label, float) in [
            (
                "pt-BR",
                egui::vec2(1600.0, 900.0),
                "pt",
                "padrão-ouro…",
                "Sair do ouro e flutuar",
            ),
            (
                "en",
                egui::vec2(1280.0, 720.0),
                "en-1280",
                "gold standard…",
                "Leave gold and float",
            ),
        ] {
            let home = user.join(format!("{tag_c}-{tag}"));
            let save = write_campaign(&home, &s, "Ouro", tag_c);
            let mut h = harness(home, locale, size);
            open(&mut h, save);
            settle(&mut h);
            h.key_press(egui::Key::J);
            settle(&mut h);
            assert!(click_containing(&mut h, open_label), "the gold decision");
            settle(&mut h);
            shot(&mut h, THEME, &format!("{name}-gold-decision-{tag}"));
            if step {
                assert!(click_containing(&mut h, float), "floating");
                settle(&mut h);
                shot(&mut h, THEME, &format!("307-gold-decision-confirm-{tag}"));
            }
        }
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
