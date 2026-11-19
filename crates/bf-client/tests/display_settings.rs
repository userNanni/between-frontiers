//! The window survives closing and opening the game: a mode and a size kept from the settings,
//! and full screen toggled by F11 and by Alt+Enter, are written and read again by a new app.

use bf_client::display::{Display, Mode};
use bf_client::settings::Settings;

mod common;
use common::layout::*;

#[test]
fn the_window_survives_closing_and_opening() {
    let user = std::env::temp_dir().join(format!("bf-display-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let file = user.join("settings.ron");
    {
        let mut h = app(user.clone(), "pt-BR", egui::vec2(1600.0, 900.0));
        h.run_steps(4);
        let chosen = Display {
            mode: Mode::Windowed,
            size: Some([1366, 768]),
        };
        let now = h.ctx.input(|i| i.time);
        h.state_mut().shared.try_display(chosen, now);
        // On trial, the file keeps the window before it.
        assert_eq!(Settings::load(&file).display, Display::default());
        h.state_mut().shared.keep_display();
        assert_eq!(Settings::load(&file).display, chosen);
        // F11: full screen, written at once.
        h.key_press(egui::Key::F11);
        h.run_steps(4);
        assert_eq!(Settings::load(&file).display.mode, Mode::Fullscreen);
        // Alt+Enter: back to the window, its size kept.
        h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::Enter);
        h.run_steps(4);
        assert_eq!(Settings::load(&file).display, chosen);
        h.key_press(egui::Key::F11);
        h.run_steps(4);
    }
    // A new app reads it back (without `app`, which writes a fresh settings file).
    let paths = bf_client::paths::Paths::new(&root(), user.clone(), user.clone());
    let h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1600.0, 900.0))
        .build_eframe(move |cc| bf_client::App::new(cc, paths));
    assert_eq!(h.state().shared.settings.display.mode, Mode::Fullscreen);
    assert_eq!(h.state().shared.settings.display.size, Some([1366, 768]));
    let _ = std::fs::remove_dir_all(&user);
}
