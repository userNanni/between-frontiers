//! The harness keeps the size it was asked for: no window command of the app (a saved window
//! size, the display settings) resizes it, at start or in a campaign. Without this, the layout
//! tests would quietly run at the saved window's size.

mod common;

use bf_client::app::Transition;
use bf_client::saves;
use common::layout::app;

#[test]
fn the_harness_keeps_its_size() {
    for size in [egui::vec2(960.0, 540.0), egui::vec2(1280.0, 720.0)] {
        let user =
            std::env::temp_dir().join(format!("bf-harness-size-{}-{}", size.x, std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut h = app(user.clone(), "en", size);
        h.run_steps(4);
        assert_eq!(h.ctx.content_rect().size(), size, "at start");
        let mut save = saves::create(
            &user.join("saves"),
            "Tamanho",
            7,
            laplace_schelling::calendar::Date::new(1925, 1, 1),
        )
        .unwrap();
        save.meta.player = Some("TUR".into());
        saves::write(&save).unwrap();
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(8);
        assert_eq!(h.ctx.content_rect().size(), size, "in a campaign");
        let _ = std::fs::remove_dir_all(&user);
    }
}
