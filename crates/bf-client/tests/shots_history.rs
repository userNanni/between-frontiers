//! Prints of the figures' charts: the closed years of a figure, on hover of the HUD and of the
//! country panel.
//! `cargo test -p bf-client --release --test shots_history -- --ignored`; images in
//! `target/screenshots/history/`.

#[path = "common/shots.rs"]
mod shots;

use bf_client::App;
use bf_client::app::Transition;
use bf_client::saves;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "history";

/// A campaign of `tag` run from 1925 to `until` with the shipped rules, written under `user`.
fn campaign(user: &std::path::Path, tag: &str, until: Date) -> saves::Save {
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    write_campaign(user, &session(tag, until, switches), "Séries", tag)
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

/// The figures' charts: Sweden (no colonies) in February 1928, three years closed. The HUD's
/// GDP and literacy with the chart of their closed years, the lowest, the highest and the
/// change of the last year; then the country panel's population, with the same chart.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn figure_history() {
    let user = std::env::temp_dir().join(format!("bf-shots-history-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let save = campaign(&user.join("save"), "SWE", Date::new(1928, 2, 1));
    for (locale, size, tag, blocks, population) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            ["PIB: ", "Alfabetização: "],
            "POPULAÇÃO",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            ["GDP: ", "Literacy: "],
            "POPULATION",
        ),
    ] {
        let mut h = harness(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        settle(&mut h);
        for (k, block) in blocks.iter().enumerate() {
            h.query_all(egui_kittest::kittest::by().label_contains(block))
                .next()
                .unwrap_or_else(|| panic!("the block {block}"))
                .hover();
            settle(&mut h);
            save_shot(&mut h, &format!("291-history-{k}-{tag}"));
        }
        // C opens the country panel; its population, on hover.
        h.key_press(egui::Key::C);
        settle(&mut h);
        let at = h
            .query_all(egui_kittest::kittest::by().label(population))
            .last()
            .expect("the country panel's population")
            .rect()
            .center();
        h.hover_at(at);
        settle(&mut h);
        save_shot(&mut h, &format!("291-history-2-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The yearly series of the simulation (save 45): Sweden in February 1928 with the charts of its
/// public debt and life expectancy; Britain, with colonies, with the chart of its metropole's
/// GDP alone. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn series_history() {
    let user = std::env::temp_dir().join(format!("bf-shots-series-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let sweden = campaign(&user.join("swe"), "SWE", Date::new(1928, 2, 1));
    let britain = campaign(&user.join("gbr"), "GBR", Date::new(1928, 2, 1));
    for (locale, size, tag, blocks, gdp) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            ["Dívida pública: ", "Expectativa de vida: "],
            "PIB: ",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            ["Public debt: ", "Life expectancy: "],
            "GDP: ",
        ),
    ] {
        let mut h = harness(user.join(format!("swe-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(sweden.clone()), &ctx);
        settle(&mut h);
        for (k, block) in blocks.iter().enumerate() {
            h.query_all(egui_kittest::kittest::by().label_contains(block))
                .next()
                .unwrap_or_else(|| panic!("the block {block}"))
                .hover();
            settle(&mut h);
            save_shot(&mut h, &format!("319-series-{k}-{tag}"));
        }
        let mut h = harness(user.join(format!("gbr-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(britain.clone()), &ctx);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(gdp))
            .next()
            .expect("the GDP block")
            .hover();
        settle(&mut h);
        save_shot(&mut h, &format!("319-series-metro-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
