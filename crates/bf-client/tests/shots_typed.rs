//! Prints of the typed numbers: a stepper's value turned into a field.
//! `cargo test -p bf-client --release --test shots_typed -- --ignored`; images in
//! `target/screenshots/typed/`.

#[path = "common/shots.rs"]
mod shots;

use bf_client::App;
use bf_client::app::Transition;
use bf_client::saves;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "typed";

/// A campaign of `tag` run from 1925 to `until` with the shipped rules, written under `user`.
fn campaign(user: &std::path::Path, tag: &str, until: Date) -> saves::Save {
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    write_campaign(user, &session(tag, until, switches), "Tarifas", tag)
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

/// Types the key tip of element `id`, the leader key first; scrolls down to find it.
fn act(h: &mut Harness<'_, App>, id: &str) {
    for _ in 0..8 {
        if !h.state().shared.keys.hint_mode() {
            h.key_press(egui::Key::AltLeft);
            h.run_steps(3);
        }
        if let Some(code) = h.state().shared.keys.code_of(id).map(str::to_owned) {
            for c in code.chars() {
                h.key_press(egui::Key::from_name(&c.to_string()).expect("a key for the code"));
                h.run_steps(4);
            }
            settle(h);
            if h.state().shared.keys.hint_mode() {
                h.key_press(egui::Key::Escape);
                h.run_steps(2);
            }
            return;
        }
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
        h.run_steps(4);
    }
    panic!("no key tip for {id}");
}

/// The rate of a good typed: Britain in February 1926, the trade tab with steel fixed, its
/// rate's field open with 37 typed; then 250 typed and refused, with the limits said.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn typed_tariff() {
    let user = std::env::temp_dir().join(format!("bf-shots-typed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "GBR", Date::new(1926, 2, 1));
    for (locale, size, tag, tip) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "De 0 % a 200 %"),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "From 0% to 200%",
        ),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::T);
        settle(&mut h);
        act(&mut h, "gov.tariff.cat.industrial");
        act(&mut h, "gov.tariff.good.steel.fixed");
        let value = h
            .query_all(egui_kittest::kittest::by().label_contains(tip))
            .last()
            .unwrap_or_else(|| panic!("the steel's rate"));
        value.scroll_to_me();
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(tip))
            .last()
            .unwrap()
            .click();
        settle(&mut h);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text("37".into()));
        settle(&mut h);
        save_shot(&mut h, &format!("292-typed-{tag}"));
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.event(egui::Event::Text("250".into()));
        h.run_steps(1);
        h.key_press(egui::Key::Enter);
        settle(&mut h);
        save_shot(&mut h, &format!("293-typed-refused-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
