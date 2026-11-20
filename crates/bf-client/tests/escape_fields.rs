//! Escape in a text field leaves the field, not the window around it: the first Escape lets go
//! of the field, the second closes the window. The partners' search on the trade tab and the
//! research screen's search.

mod common;

use bf_client::app::Transition;
use bf_client::saves;
use bf_client::screens::Screen;
use common::layout::app;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn steps(h: &mut Harness<'_, bf_client::App>) {
    for _ in 0..8 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
}

fn open(h: &Harness<'_, bf_client::App>) -> Vec<&'static str> {
    match &h.state().screen {
        Screen::Game(g) => g.open_windows(),
        _ => Vec::new(),
    }
}

/// Types into the first text field on screen, then Escape twice.
fn escape_twice(h: &mut Harness<'_, bf_client::App>, window: &str) {
    h.query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
        .next()
        .unwrap_or_else(|| panic!("a text field in {window}"))
        .click();
    steps(h);
    h.event(egui::Event::Text("swe".into()));
    steps(h);
    assert!(
        h.ctx.egui_wants_keyboard_input(),
        "{window}: typing in the field"
    );
    h.key_press(egui::Key::Escape);
    steps(h);
    assert!(
        !h.ctx.egui_wants_keyboard_input(),
        "{window}: the field let go"
    );
    assert!(
        open(h).contains(&window),
        "{window}: still open after one Escape"
    );
    h.key_press(egui::Key::Escape);
    steps(h);
    assert!(
        !open(h).contains(&window),
        "{window}: closed by the second Escape"
    );
}

#[test]
fn escape_leaves_the_field_first() {
    let user = std::env::temp_dir().join(format!("bf-escape-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut h = app(user.clone(), "en", egui::vec2(1600.0, 900.0));
    h.ctx.options_mut(|o| o.warn_on_id_clash = true);
    h.run_steps(4);
    let mut save = saves::create(
        &user.join("saves"),
        "Escape",
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some("GBR".into());
    saves::write(&save).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    let started = std::time::Instant::now();
    while h.state().shared.world.is_none() && started.elapsed().as_secs() < 20 {
        steps(&mut h);
    }
    steps(&mut h);
    // The trade tab's search of partners by name.
    h.key_press(egui::Key::G);
    steps(&mut h);
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::T);
    steps(&mut h);
    let find = h
        .query_all(egui_kittest::kittest::by().label_contains("Choose a country by name"))
        .next();
    match find {
        Some(b) => b.click(),
        None => panic!("the partners' search on the trade tab"),
    }
    steps(&mut h);
    escape_twice(&mut h, "government");
    // The research screen's search.
    h.key_press(egui::Key::R);
    steps(&mut h);
    escape_twice(&mut h, "research");
    let _ = std::fs::remove_dir_all(&user);
}
