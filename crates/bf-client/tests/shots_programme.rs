//! Prints of the programme-only technologies on the research screen.
//! `cargo test -p bf-client --release --test shots_programme -- --ignored`; images in
//! `target/screenshots/research/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "research";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// Britain in February 1926, without a nuclear programme: the production reactor searched and
/// hovered, closed with the reason. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn programme_only() {
    let user = std::env::temp_dir().join(format!("bf-shots-programme-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "GBR",
        Date::new(1926, 2, 1),
        bf_sim::data::GameData::load(&root())
            .expect("game data")
            .switches,
    );
    for (locale, size, tag, reactor) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Reator de produção",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Production reactor",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Pesquisa", "GBR");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::R);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
            .next()
            .expect("the search field")
            .click();
        h.run_steps(2);
        h.event(egui::Event::Text(reactor.into()));
        settle(&mut h);
        let at = h
            .query_all(egui_kittest::kittest::by().label(reactor))
            .next()
            .expect("the reactor's card")
            .rect()
            .center();
        h.hover_at(at);
        settle(&mut h);
        shot(&mut h, THEME, &format!("301-research-programme-only-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
