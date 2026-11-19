//! Prints of the public debt in the HUD.
//! `cargo test -p bf-client --release --test shots_debt -- --ignored`; images in
//! `target/screenshots/debt/`.

#[path = "common/shots.rs"]
mod shots;

use bf_client::App;
use bf_client::app::Transition;
use bf_client::saves;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "debt";

/// A campaign of `tag` run from 1925 to `until` with the shipped rules, written under `user`.
fn campaign(user: &std::path::Path, tag: &str, until: Date) -> saves::Save {
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    write_campaign(user, &session(tag, until, switches), "Dívida", tag)
}

fn settle(h: &mut Harness<'_, App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The public debt in sight: Britain in February 1926, the block "Public debt" beside the GDP in
/// the HUD's figures, with the share of GDP, who holds it and the interest of a year on hover.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn debt_in_sight() {
    let user = std::env::temp_dir().join(format!("bf-shots-debt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "GBR", Date::new(1926, 2, 1));
    for (locale, size, tag, title) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "Dívida pública: "),
        ("en", egui::vec2(1280.0, 720.0), "en-1280", "Public debt: "),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(title))
            .next()
            .expect("the debt's block in the HUD")
            .hover();
        settle(&mut h);
        shot(&mut h, THEME, &format!("290-debt-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
