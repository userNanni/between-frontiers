//! The first campaign: the key tips light by themselves once, with a banner; the shortcuts'
//! chip shows from then on and goes after a few uses, unless pinned.

mod common;

use bf_client::app::Transition;
use bf_client::saves;
use common::layout::{app, nodes};
use egui_kittest::Harness;

fn steps(h: &mut Harness<'_, bf_client::App>) {
    for _ in 0..8 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
}

fn chip(h: &Harness<'_, bf_client::App>) -> bool {
    nodes(h).iter().any(|n| n.label == "shortcuts")
}

#[test]
fn the_first_campaign_lights_the_tips_once() {
    let user = std::env::temp_dir().join(format!("bf-first-tips-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut h = app(user.clone(), "en", egui::vec2(1600.0, 900.0));
    h.run_steps(4);
    let mut save = saves::create(
        &user.join("saves"),
        "First",
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some("SWE".into());
    saves::write(&save).unwrap();
    // As the country choice leaves it.
    h.state_mut().shared.first_tips = true;
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    let started = std::time::Instant::now();
    while h.state().shared.world.is_none() && started.elapsed().as_secs() < 20 {
        steps(&mut h);
    }
    steps(&mut h);
    let s = &h.state().shared;
    assert!(s.keys.hint_mode(), "the tips lit by themselves");
    assert!(s.settings.first_tips_done);
    assert_eq!(s.settings.shortcut_uses, Some(0));
    assert!(chip(&h), "the chip shows");
    // Each time the player lights the tips counts; after five the chip goes.
    for n in 1..=5 {
        h.key_press(egui::Key::Escape);
        steps(&mut h);
        h.key_press(egui::Key::AltLeft);
        steps(&mut h);
        assert_eq!(h.state().shared.settings.shortcut_uses, Some(n));
    }
    h.key_press(egui::Key::Escape);
    steps(&mut h);
    assert!(!chip(&h), "the chip went after five uses");
    // Pinned, it stays.
    h.state_mut().shared.settings.shortcut_chip_pinned = true;
    steps(&mut h);
    assert!(chip(&h), "pinned, the chip stays");
    // The settings file keeps it all.
    let saved = std::fs::read_to_string(user.join("settings.ron")).unwrap();
    assert!(saved.contains("first_tips_done: true"), "{saved}");
    let _ = std::fs::remove_dir_all(&user);
}
