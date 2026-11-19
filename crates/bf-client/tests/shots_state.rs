//! Prints of the state panel: a state clicked on the map, with its country above it.
//! `cargo test -p bf-client --release --test shots_state -- --ignored`; images in
//! `target/screenshots/state/`.

#[path = "common/shots.rs"]
mod shots;

use laplace_schelling::calendar::Date;
use shots::*;

/// Brazil in March 1926 (a year done, so the GDP share shows): the capital's state, then the
/// country panel it leads up to. Portuguese at 1600 × 900, English at 1280 × 720, Portuguese
/// at the floor.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn state_panel() {
    let user = std::env::temp_dir().join(format!("bf-shots-state-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "BRA",
        Date::new(1926, 3, 1),
        bf_sim::game::Switches::default(),
    );
    for (locale, size, name) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
        ("pt-BR", egui::vec2(960.0, 540.0), "pt-960"),
    ] {
        let home = user.join(name);
        let save = write_campaign(&home, &s, "Estado", "BRA");
        let mut h = harness(home.clone(), locale, size);
        open(&mut h, save);
        assert!(
            select_capital_state(&mut h, "BRA").is_some(),
            "no capital state"
        );
        shot(&mut h, "state", &format!("200-state-{name}"));
        // Up to the country.
        h.key_press(egui::Key::Escape);
        h.run_steps(4);
        shot(&mut h, "state", &format!("201-state-up-{name}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Selects the state that holds `tag`'s capital, as a click on it does; its index.
fn select_capital_state(
    h: &mut egui_kittest::Harness<'_, bf_client::App>,
    tag: &str,
) -> Option<usize> {
    let app = h.state_mut();
    let world = app.shared.world.as_ref()?;
    let c = world.countries.iter().find(|c| c.tag() == tag)?;
    let state = world.provinces[usize::from(c.capital_province?) - 1].state?;
    if let bf_client::screens::Screen::Game(g) = &mut app.screen {
        g.select_state(world, state);
    }
    h.run_steps(4);
    Some(state)
}
