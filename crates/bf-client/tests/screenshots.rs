//! Renders every screen offscreen with wgpu and writes PNGs, walking the real flow: main menu,
//! settings, campaigns, a new campaign, choosing a country with a click, starting the campaign.
//!
//! Run with `cargo test -p bf-client --release --test screenshots -- --ignored --nocapture`. Images go to
//! `BF_SCREENSHOT_DIR` (default `target/screenshots`). Needs a GPU or a software Vulkan driver.

use std::path::PathBuf;

use bf_client::app::Transition;
use bf_client::paths::Paths;
use bf_client::screens::Screen;
use bf_client::{App, saves};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn out_dir() -> PathBuf {
    let dir = std::env::var_os("BF_SCREENSHOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/screenshots"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn shot(harness: &mut Harness<'_, App>, name: &str) {
    harness.run_steps(4);
    let image = harness.render().expect("render");
    let path = out_dir().join(format!("{name}.png"));
    image.save(&path).unwrap();
    println!("screenshot: {}", path.display());
}

fn harness(user: PathBuf, locale: &str) -> Harness<'static, App> {
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        format!("(locale: \"{locale}\", ui_scale: 100, map_labels: true, autosave: Yearly)"),
    )
    .unwrap();
    let paths = Paths::new(&root(), user.clone(), user);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 900.0))
        .wgpu()
        .build_eframe(move |cc| App::new(cc, paths));
    h.set_render_cursor(false);
    h
}

#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn walk_through_the_screens() {
    let user = std::env::temp_dir().join(format!("bf-shots-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut h = harness(user.clone(), "pt-BR");

    // 1. Main menu, no campaign yet.
    shot(&mut h, "01-menu-vazio");

    // 2. Settings.
    h.get_by_label("Configurações").click();
    shot(&mut h, "02-configuracoes");
    h.get_by_label("Concluir").click();
    h.run_steps(2);

    // 3. Campaigns with an older campaign already saved.
    let old = saves::create(
        &user.join("saves"),
        "Ordem e Progresso",
        1889,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    let mut old = old;
    old.meta.player = Some("BRA".into());
    old.meta.date = laplace_schelling::calendar::Date::new(1931, 10, 24);
    old.meta.last_played -= 3 * 86_400;
    saves::write(&old).unwrap();
    h.get_by_label("Iniciar jogo").click();
    shot(&mut h, "03-campanhas");

    // 4. Create a campaign: lands on the map to choose a country.
    h.get_by_label("Criar e escolher país").click();
    h.run_steps(3);
    shot(&mut h, "04-escolha-do-pais");

    // 5. Click Brazil's capital on the map.
    let target = {
        let app = h.state();
        let world = app.shared.world.as_ref().expect("world loaded");
        let view = app.shared.map_view.as_ref().unwrap();
        let bra = world.countries.iter().find(|c| c.tag() == "BRA").unwrap();
        let (lon, lat) = bra.def.capital_lon_lat;
        let (x, y) = bf_client::world::project(&world.map.meta, lon - 4.0, lat + 4.0).unwrap();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 900.0));
        view.camera.unwrap().to_screen(rect, egui::pos2(x, y))
    };
    h.hover_at(target);
    h.run_steps(1);
    h.drag_at(target);
    h.run_steps(1);
    h.drop_at(target);
    shot(&mut h, "05-pais-selecionado");
    assert!(matches!(h.state().screen, Screen::Pick(_)));

    // 6. Start the campaign.
    h.get_by_label("Começar campanha").click();
    h.run_steps(3);
    assert!(matches!(h.state().screen, Screen::Game(_)));
    shot(&mut h, "06-campanha");

    // 6a. The engine runs: unpause, let days pass, pause, save from the menu, reopen with
    // "Continue" and find the same date and state hash.
    let game_snapshot = |h: &Harness<'_, App>| match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    };
    h.key_press(egui::Key::Space);
    let started = std::time::Instant::now();
    while game_snapshot(&h).tick < 20 && started.elapsed().as_secs() < 20 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(1);
    }
    h.key_press(egui::Key::Space);
    std::thread::sleep(std::time::Duration::from_millis(400));
    h.run_steps(2);
    let before = game_snapshot(&h);
    assert!(
        before.paused && before.tick >= 20,
        "simulation did not run: {before:?}"
    );
    assert_eq!(before.player.as_deref(), Some("BRA"));
    h.get_by_label("Menu (Esc)").click();
    h.run_steps(2);
    h.get_by_label("Salvar").click();
    h.run_steps(2);
    shot(&mut h, "06a-salvo");
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Menu, &ctx);
    h.run_steps(2);
    h.get_by_label("Continuar").click();
    h.run_steps(3);
    let after = game_snapshot(&h);
    assert_eq!(
        (after.date, after.tick, after.hash.clone()),
        (before.date, before.tick, before.hash.clone())
    );

    // 6b. Tooltip of the quality-of-life figure in the top bar.
    let quality = h
        .get_all(egui_kittest::kittest::by().label_contains("Qualidade de vida"))
        .next()
        .expect("quality figure in the top bar")
        .rect();
    h.hover_at(quality.center());
    h.run_steps(8);
    shot(&mut h, "06b-barra-tooltip");

    // 6c. Tooltip of GDP, in the compact rows.
    let gdp = h
        .get_all(egui_kittest::kittest::by().label_contains("PIB:"))
        .next()
        .expect("GDP figure")
        .rect();
    h.hover_at(gdp.center());
    h.run_steps(8);
    shot(&mut h, "06c-pib-tooltip");
    h.hover_at(egui::pos2(800.0, 500.0));

    // 6d. A colonial power: France, with the population split into metropole and colonies.
    let mut fra = saves::create(
        &user.join("saves"),
        "Terceira República",
        1925,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    fra.meta.player = Some("FRA".into());
    saves::write(&fra).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(fra), &ctx);
    h.run_steps(3);
    let population = h
        .get_all(egui_kittest::kittest::by().label_contains("População:"))
        .next()
        .expect("population figure")
        .rect();
    h.hover_at(population.center());
    h.run_steps(8);
    shot(&mut h, "06d-franca-colonias");
    // Every figure that has a colonial part splits it in its tooltip.
    for figure in ["População:", "PIB:", "Área:", "Estados:"] {
        h.hover_at(egui::pos2(800.0, 600.0));
        h.run_steps(4);
        let cell = h
            .get_all(egui_kittest::kittest::by().label_contains(figure))
            .next()
            .unwrap_or_else(|| panic!("{figure} in the HUD"))
            .rect();
        h.hover_at(cell.center());
        h.run_steps(8);
        assert!(
            h.get_all(egui_kittest::kittest::by().label("Colônias"))
                .next()
                .is_some(),
            "{figure} tooltip lacks the colonial line"
        );
    }
    h.hover_at(egui::pos2(800.0, 600.0));

    // 6e. Close-up of the Alps: pastel colours with the terrain shading under them.
    {
        let app = h.state_mut();
        let meta = app.shared.world.as_ref().unwrap().map.meta.clone();
        let (x, y) = bf_client::world::project(&meta, 10.5, 46.0).unwrap();
        let view = app.shared.map_view.as_mut().unwrap();
        view.camera.as_mut().unwrap().look_at(egui::pos2(x, y), 2.2);
    }
    h.run_steps(3);
    shot(&mut h, "06e-relevo-alpes");

    // 7. Back to the main menu: "Continue" now offers the new campaign.
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Menu, &ctx);
    shot(&mut h, "07-menu-continuar");

    // 8. Same menu in English.
    let mut en = harness(user.clone(), "en");
    shot(&mut en, "08-menu-en");

    // 9. A campaign with a long country name, in English.
    let mut yug = saves::create(
        &user.join("saves"),
        "Yugoslav test",
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    yug.meta.player = Some("YUG".into());
    saves::write(&yug).unwrap();
    let ctx = en.ctx.clone();
    en.state_mut().apply(Transition::Game(yug), &ctx);
    shot(&mut en, "09-campaign-en-long-name");

    let _ = std::fs::remove_dir_all(&user);
}
