//! Print of the mass media in the country panel, with the switch `media` on.
//! `cargo test -p bf-client --release --test shots_media -- --ignored`; images in
//! `target/screenshots/media/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "media";
const RULES: &str = "media";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The United States in February 1926, a January after the start: newspapers, radio sets,
/// television and internet (not yet) per thousand, the reach and the state's share, in the
/// country panel the banner opens. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn media_in_the_country_panel() {
    let user = std::env::temp_dir().join(format!("bf-shots-media-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches
        .with(RULES)
        .expect("switches");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let s = session("USA", Date::new(1926, 2, 2), switches);
    for (locale, size, tag, line) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "Jornais"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280", "Newspapers"),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Mídia", "USA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        let banner = egui::pos2(54.0, 42.0);
        h.hover_at(banner);
        h.run_steps(1);
        h.drag_at(banner);
        h.run_steps(1);
        h.drop_at(banner);
        h.hover_at(egui::pos2(size.x / 2.0, size.y - 100.0));
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(line))
            .next()
            .expect("the media's line")
            .scroll_to_me();
        settle(&mut h);
        shot(&mut h, THEME, &format!("332-media-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
