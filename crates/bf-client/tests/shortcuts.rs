//! Shortcuts you can find: F1 opens the sheet of keys over any window and Escape closes the
//! sheet alone; the sheet's search narrows it; the main windows open with one key.

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

#[test]
fn the_keys_are_found_and_answer() {
    let user = std::env::temp_dir().join(format!("bf-shortcuts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut h = app(user.clone(), "en", egui::vec2(1600.0, 900.0));
    h.run_steps(4);
    let mut save = saves::create(
        &user.join("saves"),
        "Keys",
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some("SWE".into());
    saves::write(&save).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    let started = std::time::Instant::now();
    while h.state().shared.world.is_none() && started.elapsed().as_secs() < 20 {
        steps(&mut h);
    }
    steps(&mut h);
    // One key for each main window.
    for (key, window) in [
        (egui::Key::N, "economy"),
        (egui::Key::K, "war"),
        (egui::Key::I, "influence"),
        (egui::Key::G, "government"),
    ] {
        h.key_press(key);
        steps(&mut h);
        assert_eq!(open(&h), vec![window], "{key:?} opens {window}");
    }
    // F1 over the government: the sheet, with every key; the search narrows it.
    h.key_press(egui::Key::F1);
    steps(&mut h);
    assert_eq!(open(&h), vec!["government", "keys-sheet"]);
    assert!(
        h.query_by_label("Pause or resume").is_some(),
        "the pause on the sheet"
    );
    h.event(egui::Event::Text("research".into()));
    steps(&mut h);
    assert!(
        h.query_by_label("Pause or resume").is_none(),
        "the search narrows"
    );
    // Escape closes the sheet alone.
    h.key_press(egui::Key::Escape);
    steps(&mut h);
    assert_eq!(
        open(&h),
        vec!["government"],
        "Escape closes the sheet, not the window"
    );
    let _ = std::fs::remove_dir_all(&user);
}
