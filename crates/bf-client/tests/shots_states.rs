//! Prints of the metrics by state: a state lens with its legend and a state's value on hover,
//! and the economy window's states tab sorted by a column.
//! `cargo test -p bf-client --release --test shots_states -- --ignored`; images in
//! `target/screenshots/states/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "states";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// Brazil in February 1926: the density lens kept open with its legend, a state's density on
/// hover; the literacy lens; then the states tab sorted by population. Portuguese at
/// 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn states_metrics() {
    use bf_client::map::lens::Lens;
    let user = std::env::temp_dir().join(format!("bf-shots-states-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = session(
        "BRA",
        Date::new(1926, 2, 1),
        bf_sim::data::GameData::load(&root())
            .expect("game data")
            .switches,
    );
    for (locale, size, tag, tab, population) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Estados",
            "População",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "States",
            "Population",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Estados", "BRA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        let choose = |h: &mut egui_kittest::Harness<'_, bf_client::App>, lens: Lens| {
            let app = h.state_mut();
            app.shared.hud.layout.map_modes_open = true;
            if let (Some(view), Some(world)) = (&mut app.shared.map_view, &app.shared.world) {
                bf_client::screens::lens_panel::choose(view, world, lens, 1926);
            }
        };
        choose(&mut h, Lens::State(1));
        settle(&mut h);
        h.hover_at(egui::pos2(size.x * 0.33, size.y * 0.78));
        settle(&mut h);
        shot(&mut h, THEME, &format!("311-state-density-{tag}"));
        choose(&mut h, Lens::State(3));
        settle(&mut h);
        h.hover_at(egui::pos2(size.x * 0.33, size.y * 0.78));
        settle(&mut h);
        shot(&mut h, THEME, &format!("312-state-literacy-{tag}"));
        h.state_mut().shared.hud.layout.map_modes_open = false;
        h.hover_at(egui::pos2(size.x * 0.5, size.y * 0.5));
        settle(&mut h);
        h.key_press(egui::Key::N);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label(tab))
            .next()
            .expect("the states tab")
            .click();
        settle(&mut h);
        h.query_all(
            egui_kittest::kittest::by()
                .role(egui::accesskit::Role::ColumnHeader)
                .label(population),
        )
        .next()
        .expect("the population column")
        .click();
        settle(&mut h);
        shot(&mut h, THEME, &format!("313-states-table-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
