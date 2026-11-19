//! Prints of the research screen: what a technology opens, and the way back from a unit.
//! `cargo test -p bf-client --release --test shots_research -- --ignored`; images in
//! `target/screenshots/research/`.
#[path = "common/shots.rs"]
mod shots;

use bf_client::App;
use bf_client::app::Transition;
use bf_client::saves;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "research";

/// A campaign of `tag` run from 1925 to `until` with the shipped rules, written under `user`.
fn campaign(user: &std::path::Path, tag: &str, until: Date) -> saves::Save {
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    write_campaign(user, &session(tag, until, switches), "Pesquisa", tag)
}

fn settle(h: &mut Harness<'_, App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

fn save_shot(h: &mut Harness<'_, App>, name: &str) {
    shot(h, THEME, name);
}

/// The way back: Britain in February 1926 searches for the armour; the tree lights the tank
/// and centres it, and its tip says what it opens (the armour), what it leads to, its gains and
/// what Britain still has to research first; then the atomic bomb, far ahead, with the
/// technologies missing on the way. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn research_unlocks() {
    let user = std::env::temp_dir().join(format!("bf-shots-research-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "GBR", Date::new(1926, 2, 1));
    for (locale, size, tag, query, tank, bomb) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "blindados",
            "Carro de combate",
            "Bomba atômica",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "armour",
            "Tank",
            "Atomic bomb",
        ),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        h.key_press(egui::Key::R);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
            .next()
            .expect("the search field")
            .click();
        h.run_steps(2);
        h.event(egui::Event::Text(query.into()));
        settle(&mut h);
        let at = h
            .query_all(egui_kittest::kittest::by().label(tank))
            .next()
            .expect("the tank's card")
            .rect()
            .center();
        h.hover_at(at);
        settle(&mut h);
        save_shot(&mut h, &format!("294-research-{tag}"));
        // A technology far ahead: what Britain still has to research first.
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text(bomb.into()));
        settle(&mut h);
        let at = h
            .query_all(egui_kittest::kittest::by().label(bomb))
            .next()
            .expect("the bomb's card")
            .rect()
            .center();
        h.hover_at(at);
        settle(&mut h);
        save_shot(&mut h, &format!("295-research-path-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
