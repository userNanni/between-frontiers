//! Prints of the economy window's goods tab.
//! `cargo test -p bf-client --release --test shots_goods -- --ignored`; images in
//! `target/screenshots/goods/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "goods";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// Britain in February 1926: the market good by good sorted by what it makes; then steel
/// opened on who makes and who uses it. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn goods_tab() {
    let user = std::env::temp_dir().join(format!("bf-shots-goods-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "GBR",
        Date::new(1926, 2, 1),
        bf_sim::data::GameData::load(&root())
            .expect("game data")
            .switches,
    );
    for (locale, size, tag, tab, made, steel) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Bens",
            "Produção/ano",
            "Aço",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Goods",
            "Made/year",
            "Steel",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Bens", "GBR");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::N);
        settle(&mut h);
        h.get_by_label(tab).click();
        settle(&mut h);
        h.query_all(
            egui_kittest::kittest::by()
                .role(egui::accesskit::Role::ColumnHeader)
                .label(made),
        )
        .next()
        .expect("the column of what is made")
        .click();
        settle(&mut h);
        shot(&mut h, THEME, &format!("317-goods-{tag}"));
        h.get_by_label(steel).scroll_to_me();
        settle(&mut h);
        h.get_by_label(steel).click();
        settle(&mut h);
        shot(&mut h, THEME, &format!("318-goods-steel-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
