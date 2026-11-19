//! Prints of the economy window's methods tab.
//! `cargo test -p bf-client --release --test shots_methods -- --ignored`; images in
//! `target/screenshots/methods/`.

#[path = "common/shots.rs"]
mod shots;

use laplace_schelling::calendar::Date;
use shots::*;

/// Brazil in March 1925: the methods of its first building, then of the second. Portuguese at
/// 1600 × 900, English at 1280 × 720, Portuguese at the floor.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn methods_tab() {
    let user = std::env::temp_dir().join(format!("bf-shots-methods-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "BRA",
        Date::new(1925, 3, 1),
        bf_sim::game::Switches::default(),
    );
    for (locale, size, name, tab) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "Métodos"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280", "Methods"),
        ("pt-BR", egui::vec2(960.0, 540.0), "pt-960", "Métodos"),
    ] {
        let home = user.join(name);
        let save = write_campaign(&home, &s, "Métodos", "BRA");
        let mut h = harness(home.clone(), locale, size);
        open(&mut h, save);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::E);
        h.run_steps(4);
        assert!(click_containing(&mut h, tab), "no methods tab");
        shot(&mut h, "methods", &format!("170-methods-{name}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
