//! Prints of the buildings' search: Britain's colonies, then a name typed.
//! `cargo test -p bf-client --release --test shots_search -- --ignored`; images in
//! `target/screenshots/search/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn buildings_search() {
    let user = std::env::temp_dir().join(format!("bf-shots-search-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "GBR",
        Date::new(1925, 3, 1),
        bf_sim::game::Switches::default(),
    );
    for (locale, size, name, colonies, find) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Colônias",
            "algod",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Colonies",
            "cotton",
        ),
    ] {
        let home = user.join(name);
        let save = write_campaign(&home, &s, "Busca", "GBR");
        let mut h = harness(home.clone(), locale, size);
        open(&mut h, save);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::E);
        h.run_steps(4);
        assert!(click_containing(&mut h, colonies), "no colonies' filter");
        shot(&mut h, "search", &format!("210-colonies-{name}"));
        if let Some(n) = h
            .query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
            .next()
        {
            n.focus();
            n.type_text(find);
        }
        h.run_steps(4);
        click_containing(&mut h, "+ ");
        shot(&mut h, "search", &format!("211-search-{name}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
