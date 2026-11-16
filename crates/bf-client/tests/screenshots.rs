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
use egui_kittest::kittest::{NodeT, Queryable};

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
    // A tick in flight (the weekly market clearing takes longest) finishes before the pause.
    let paused_at = std::time::Instant::now();
    while !game_snapshot(&h).paused && paused_at.elapsed().as_secs() < 10 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(1);
    }
    h.run_steps(2);
    let before = game_snapshot(&h);
    assert!(
        before.paused && before.tick >= 20,
        "simulation did not run: {before:?}"
    );
    assert_eq!(before.player.as_deref(), Some("BRA"));

    // 6r. Research screen: R opens it on the era with open technologies; Esc closes it.
    h.key_press(egui::Key::R);
    h.run_steps(3);
    shot(&mut h, "06r-pesquisa");
    // Hovering a card lights up what it needs and what it leads to.
    h.hover_at(egui::pos2(612.0, 540.0));
    h.run_steps(2);
    shot(&mut h, "06s-pesquisa-caminho");
    h.get_by_label("Ajustar").click();
    h.run_steps(2);
    shot(&mut h, "06t-pesquisa-inteira");
    h.get_by_label("● Militar").click();
    h.run_steps(2);
    shot(&mut h, "06u-pesquisa-militar");
    h.get_by_label("Todas as áreas").click();
    h.run_steps(2);
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    // Transport met in each state, live from the simulation.
    h.get_by_label("Infra").click();
    h.run_steps(3);
    shot(&mut h, "06j-modo-transporte");
    h.get_by_label("Político").click();
    h.run_steps(2);
    h.run_steps(2);
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

    // 6f. Map modes: copper deposits, climate, coffee suitability, then back to political.
    {
        let app = h.state_mut();
        let view = app.shared.map_view.as_mut().unwrap();
        view.camera = None;
    }
    h.run_steps(2);
    h.get_by_label("Recursos").click();
    h.run_steps(2);
    h.get_by_label("Cobre").click();
    h.run_steps(3);
    shot(&mut h, "06f-modo-cobre");
    h.get_by_label("Bauxita").click();
    h.run_steps(3);
    shot(&mut h, "06g-modo-bauxita");
    h.get_by_label("Clima").click();
    h.run_steps(3);
    shot(&mut h, "06h-modo-clima");
    h.get_by_label("Agro").click();
    h.run_steps(2);
    h.get_by_label("Cafezal").click();
    h.run_steps(3);
    shot(&mut h, "06i-modo-cafe");
    h.get_by_label("Político").click();
    h.run_steps(2);

    // 6k. A campaign fifteen years in: the HUD and the country panel read the simulation, not the
    // 1925 scenario. The engine runs here, off the interface, and the campaign opens from its
    // save.
    let mut later = saves::create(
        &user.join("saves"),
        "Quinze anos depois",
        1925,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    later.meta.player = Some("BRA".into());
    {
        let data = bf_sim::data::GameData::load(&root()).expect("game data");
        let mut session = bf_sim::game::Session::new(
            data.new_world(),
            later.meta.seed,
            later.meta.date,
            0,
            Some(data.rules()),
        );
        session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "BRA".into() });
        while session.clock().date < laplace_schelling::calendar::Date::new(1940, 3, 1) {
            session.sim.step();
        }
        later.meta.date = session.clock().date;
        std::fs::write(
            later.dir.join(saves::WORLD_FILE),
            session.save("test", &later.meta.name).unwrap(),
        )
        .unwrap();
    }
    saves::write(&later).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(later), &ctx);
    h.run_steps(3);
    {
        let world = h.state().shared.world.as_ref().unwrap();
        let bra = world
            .countries
            .iter()
            .position(|c| c.tag() == "BRA")
            .unwrap();
        let live = world.live.as_ref().expect("live figures in a campaign");
        assert_eq!(
            live.source,
            bf_client::world::Source::Live {
                gdp_year: Some(1939)
            }
        );
        let (now, then) = (&live.countries[bra], &world.scenario.countries[bra]);
        assert_ne!(now.population, then.population);
        assert_ne!(now.gdp, then.gdp);
        println!(
            "BRA population {:.0} -> {:.0}, GDP {:.0} -> {:.0}",
            then.population, now.population, then.gdp, now.gdp
        );
    }
    shot(&mut h, "06k-campanha-1940");
    let gdp = h
        .get_all(egui_kittest::kittest::by().label_contains("PIB:"))
        .next()
        .expect("GDP figure")
        .rect();
    h.hover_at(gdp.center());
    h.run_steps(8);
    shot(&mut h, "06l-pib-simulado");
    let quality = h
        .get_all(egui_kittest::kittest::by().label_contains("Qualidade de vida"))
        .next()
        .expect("quality figure")
        .rect();
    h.hover_at(quality.center());
    h.run_steps(8);
    shot(&mut h, "06m-qualidade-simulada");
    // The banner opens the country panel, with the same figures.
    let banner = egui::pos2(54.0, 42.0);
    h.hover_at(banner);
    h.run_steps(1);
    h.drag_at(banner);
    h.run_steps(1);
    h.drop_at(banner);
    h.hover_at(egui::pos2(800.0, 600.0));
    h.run_steps(3);
    shot(&mut h, "06n-painel-pais-simulado");

    // 6o. Government window (G): the budget fifteen years in, then a tooltip with the sum behind
    // a figure, then money and debt.
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    h.key_press(egui::Key::G);
    h.run_steps(4);
    shot(&mut h, "06o-governo-orcamento");
    let hover = |h: &mut Harness<'_, App>, label: &str| {
        let r = h
            .get_all(egui_kittest::kittest::by().label(label))
            .next()
            .unwrap_or_else(|| panic!("{label} in the government window"))
            .rect();
        h.hover_at(r.center());
        h.run_steps(8);
    };
    hover(&mut h, "Classe média");
    shot(&mut h, "06p-governo-imposto-tooltip");
    hover(&mut h, "Saldo");
    shot(&mut h, "06q-governo-saldo-tooltip");
    hover(&mut h, "Compras");
    shot(&mut h, "06r2-governo-compras-tooltip");
    h.hover_at(egui::pos2(800.0, 880.0));
    h.get_by_label("Moeda e dívida").click();
    h.run_steps(4);
    shot(&mut h, "06v-governo-moeda");
    hover(&mut h, "Taxa básica");
    shot(&mut h, "06w-governo-taxa-tooltip");
    hover(&mut h, "Exterior, em dólar");
    shot(&mut h, "06x-governo-divida-tooltip");
    hover(&mut h, "Moeda fora da regra");
    shot(&mut h, "06x2-governo-moeda-fora-da-regra");
    h.hover_at(egui::pos2(800.0, 880.0));
    // The laws tab, still empty: the laws screen fills it.
    // A tax raised while paused shows at once, says it waits for the clock, and the next tick
    // applies it.
    h.get_by_label("Orçamento").click();
    h.run_steps(2);
    let rule_rate = game_snapshot(&h).government.unwrap().budget.tax_rates[0];
    h.get_all(egui_kittest::kittest::by().label("+"))
        .next()
        .expect("tax stepper")
        .click();
    h.run_steps(3);
    shot(&mut h, "06z-governo-imposto-pendente");
    h.key_press(egui::Key::Space);
    let tick = game_snapshot(&h).tick;
    let started = std::time::Instant::now();
    while game_snapshot(&h).tick <= tick + 1 && started.elapsed().as_secs() < 20 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
    h.key_press(egui::Key::Space);
    let paused_at = std::time::Instant::now();
    while !game_snapshot(&h).paused && paused_at.elapsed().as_secs() < 10 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(1);
    }
    let budget = game_snapshot(&h).government.unwrap().budget.clone();
    assert!(
        !budget.tax_auto,
        "the tax command did not reach the simulation"
    );
    println!(
        "lower-class tax {:.4} -> {:.4}",
        rule_rate.to_f64_lossy(),
        budget.tax_rates[0].to_f64_lossy()
    );
    assert!(budget.tax_rates[0] > rule_rate);
    h.get_by_label("Leis").click();
    h.run_steps(3);
    shot(&mut h, "06y-governo-leis");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);

    // 7. Back to the main menu: "Continue" now offers the new campaign.
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Menu, &ctx);
    // Outside a campaign the scenario's figures stand again.
    assert!(h.state().shared.world.as_ref().unwrap().live.is_none());
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
    // Two weeks of play, so the government window has a week of accounts to show.
    let en_snapshot = |h: &Harness<'_, App>| match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    };
    en.key_press(egui::Key::Space);
    let started = std::time::Instant::now();
    while en_snapshot(&en).tick < 15 && started.elapsed().as_secs() < 60 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        en.run_steps(1);
    }
    en.key_press(egui::Key::Space);
    let paused_at = std::time::Instant::now();
    while !en_snapshot(&en).paused && paused_at.elapsed().as_secs() < 10 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        en.run_steps(1);
    }
    en.key_press(egui::Key::G);
    en.run_steps(4);
    shot(&mut en, "10-government-budget-en");
    en.get_by_label("Money and debt").click();
    en.run_steps(4);
    shot(&mut en, "11-government-money-en");

    let _ = std::fs::remove_dir_all(&user);
}

/// Opens a campaign for `tag` straight from a new save.
fn open_campaign(h: &mut Harness<'_, App>, user: &std::path::Path, name: &str, tag: &str) {
    let mut save = saves::create(
        &user.join("saves"),
        name,
        1925,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some(tag.into());
    saves::write(&save).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    h.run_steps(3);
}

/// Runs the simulation until `done` holds or `secs` pass, then pauses it.
fn run_until(
    h: &mut Harness<'_, App>,
    secs: u64,
    done: impl Fn(&bf_client::sim::Snapshot) -> bool,
) {
    let snap = |h: &Harness<'_, App>| match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    };
    h.key_press(egui::Key::Space);
    let started = std::time::Instant::now();
    while !done(&snap(h)) && started.elapsed().as_secs() < secs {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(1);
    }
    h.key_press(egui::Key::Space);
    let paused_at = std::time::Instant::now();
    while !snap(h).paused && paused_at.elapsed().as_secs() < 10 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h.run_steps(1);
    }
    h.run_steps(3);
}

/// Laws tab of the government window: Brazil, a law compared with the one in force and
/// proposed, the law in force after the simulation enacts it; then France, with colonies.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn laws_screen() {
    let user = std::env::temp_dir().join(format!("bf-shots-laws-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut h = harness(user.clone(), "pt-BR");
    open_campaign(&mut h, &user, "Leis do Brasil", "BRA");
    // A few days in, so the interest groups' clout is measured.
    run_until(&mut h, 20, |s| s.tick >= 3);

    h.key_press(egui::Key::G);
    h.run_steps(2);
    h.get_by_label("Leis").click();
    h.run_steps(3);
    shot(&mut h, "10a-leis-brasil");

    // Taxation, and a law not in force: what changes and who backs it.
    h.get_all(egui_kittest::kittest::by().label_contains("Tributação:"))
        .next()
        .expect("taxation row")
        .click();
    h.run_steps(2);
    click_card(&mut h, "Imposto de renda progressivo");
    h.run_steps(3);
    shot(&mut h, "10b-leis-brasil-comparacao");

    // Propose it; the simulation enacts it within a day.
    h.get_by_label("Propor lei").click();
    h.run_steps(3);
    shot(&mut h, "10c-leis-brasil-proposta");
    let enacted = |s: &bf_client::sim::Snapshot| {
        s.laws
            .as_ref()
            .is_some_and(|l| l.pending.is_empty() && !l.changes.is_empty())
    };
    run_until(&mut h, 20, enacted);
    let snap = match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => unreachable!(),
    };
    assert!(enacted(&snap), "the law was not enacted: {:?}", snap.laws);
    shot(&mut h, "10d-leis-brasil-em-vigor");

    // France: the colonial groups hold.
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    open_campaign(&mut h, &user, "Leis da França", "FRA");
    run_until(&mut h, 20, |s| s.tick >= 3);
    h.key_press(egui::Key::G);
    h.run_steps(2);
    h.get_by_label("Leis").click();
    h.run_steps(2);
    h.get_all(egui_kittest::kittest::by().label_contains("Trabalho colonial:"))
        .next()
        .expect("colonial labour row")
        .scroll_to_me();
    h.run_steps(3);
    h.get_all(egui_kittest::kittest::by().label_contains("Trabalho colonial:"))
        .next()
        .expect("colonial labour row")
        .click();
    h.run_steps(2);
    shot(&mut h, "10e-leis-franca-colonias-grupo");
    click_card(&mut h, "Trabalho livre");
    h.run_steps(3);
    shot(&mut h, "10f-leis-franca-trabalho-livre");

    let _ = std::fs::remove_dir_all(&user);
}

/// Clicks the law card named `name` (the card, not the name printed on it).
fn click_card(h: &mut Harness<'_, App>, name: &str) {
    let find = |h: &Harness<'_, App>| {
        h.get_all(egui_kittest::kittest::by().label(name))
            .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
            .is_some()
    };
    assert!(find(h), "card {name}");
    h.get_all(egui_kittest::kittest::by().label(name))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .unwrap()
        .scroll_to_me();
    h.run_steps(3);
    h.get_all(egui_kittest::kittest::by().label(name))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .unwrap_or_else(|| panic!("card {name}"))
        .click();
}

/// Runs a new world for Brazil from 1925 to `until` on two threads and returns the session.
fn advanced_session(until: laplace_schelling::calendar::Date) -> bf_sim::game::Session {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "BRA".into() });
    while session.clock().date < until {
        session.sim.step();
    }
    session
}

/// Writes `session` as a campaign of `tag` named `name`, with `pinned` journal entries.
fn write_campaign(
    user: &std::path::Path,
    session: &bf_sim::game::Session,
    name: &str,
    tag: &str,
    pinned: &[&str],
) -> saves::Save {
    let mut save = saves::create(
        &user.join("saves"),
        name,
        1925,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some(tag.into());
    save.meta.date = session.clock().date;
    save.meta.pinned = pinned.iter().map(|p| (*p).to_owned()).collect();
    std::fs::write(
        save.dir.join(saves::WORLD_FILE),
        session.save("test", &save.meta.name).unwrap(),
    )
    .unwrap();
    saves::write(&save).unwrap();
    save
}

fn harness_sized(user: PathBuf, locale: &str, size: egui::Vec2) -> Harness<'static, App> {
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        format!("(locale: \"{locale}\", ui_scale: 100, map_labels: true, autosave: Off)"),
    )
    .unwrap();
    let paths = Paths::new(&root(), user.clone(), user);
    let mut h = Harness::builder()
        .with_size(size)
        .wgpu()
        .build_eframe(move |cc| App::new(cc, paths));
    h.set_render_cursor(false);
    h
}

/// The side panel and the agendas window fifteen years in: Brazil's queue of works and
/// research, the agendas with real entries, one pinned to the panel; a country with a fiscal
/// alert or a political movement; and the same panel in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn outliner_and_journal() {
    let user = std::env::temp_dir().join(format!("bf-shots-journal-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut session = advanced_session(laplace_schelling::calendar::Date::new(1940, 3, 1));
    let bra = write_campaign(&user, &session, "Brasil 1940", "BRA", &[]);

    // A country with a fiscal alert or an active movement, read with the journal's own reader.
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let jr = bf_client::journal::Rules {
        catalogue: &rules.catalogue,
        tree: rules.tree.as_deref(),
        politics: rules.politics.as_deref(),
    };
    let date = session.clock().date;
    let world = session.world();
    let mut alerts: Vec<(String, i64, Vec<String>)> = world
        .countries
        .iter()
        .enumerate()
        .filter_map(|(c, (_, def))| {
            let j = bf_client::journal::read_country(world, c, date, jr)?;
            let ids: Vec<String> = j
                .entries
                .iter()
                .filter(|e| {
                    matches!(
                        e.kind,
                        bf_client::journal::Kind::Fiscal(_) | bf_client::journal::Kind::Movement(_)
                    )
                })
                .map(|e| e.id.clone())
                .collect();
            (!ids.is_empty()).then(|| (def.tag.clone(), def.population, ids))
        })
        .collect();
    // The most alerts in a country of more than a million people, then the largest.
    alerts.sort_by_key(|(tag, pop, ids)| {
        (
            *pop < 1_000_000,
            std::cmp::Reverse(ids.len()),
            std::cmp::Reverse(*pop),
            tag.clone(),
        )
    });
    let alerts: Vec<(String, Vec<String>)> =
        alerts.into_iter().map(|(t, _, ids)| (t, ids)).collect();
    println!("countries with alerts in {date:?}: {}", alerts.len());
    for (tag, ids) in alerts.iter().take(12) {
        println!("  {tag}: {ids:?}");
    }
    // The first alert of a country of more than a million people, and the country with the
    // most alerts of all.
    let mut picks: Vec<(String, Vec<String>)> = alerts.first().cloned().into_iter().collect();
    if let Some(most) = alerts
        .iter()
        .max_by_key(|(t, ids)| (ids.len(), std::cmp::Reverse(t.clone())))
        && picks.first().is_none_or(|p| p.0 != most.0)
    {
        picks.push(most.clone());
    }

    let mut h = harness_sized(user.clone(), "pt-BR", egui::vec2(1600.0, 900.0));
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(bra.clone()), &ctx);
    h.run_steps(4);
    let snap = match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    };
    let journal = snap.journal.clone().expect("journal in a campaign");
    println!(
        "BRA journal: {} entries, {} public / {} private works, bottlenecks {}",
        journal.entries.len(),
        journal.works.public,
        journal.works.private,
        journal.works.bottlenecks.len()
    );
    for e in &journal.entries {
        println!("  {}", e.id);
    }
    shot(&mut h, "20-painel-obras");

    // The agendas window, then an entry picked and pinned.
    h.key_press(egui::Key::J);
    h.run_steps(3);
    shot(&mut h, "21-agendas");
    let pick = journal
        .entries
        .iter()
        .find(|e| e.id == "works-large")
        .or_else(|| journal.entries.first())
        .unwrap();
    let title = bf_client::screens::journal::text(
        &h.state().shared.i18n,
        h.state().shared.world.as_ref(),
        &pick.title,
    );
    h.get_all(egui_kittest::kittest::by().label(title.as_str()))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .unwrap_or_else(|| panic!("entry {title}"))
        .click();
    h.run_steps(3);
    h.get_by_label("Fixar no painel").click();
    h.run_steps(3);
    shot(&mut h, "22-agenda-fixada");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    shot(&mut h, "23-painel-com-fixada");
    let pinned = saves::list(&user.join("saves"))
        .into_iter()
        .find(|s| s.meta.name == "Brasil 1940")
        .map(|s| s.meta.pinned)
        .unwrap_or_default();
    println!("pinned in the campaign file: {pinned:?}");
    assert_eq!(
        pinned,
        vec![pick.id.clone()],
        "the pin is kept with the campaign"
    );

    // A campaign with a fiscal alert or a movement, with them pinned.
    for (tag, ids) in &picks {
        session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: tag.clone() });
        session.sim.step();
        let ids: Vec<&str> = ids.iter().map(String::as_str).take(3).collect();
        let save = write_campaign(&user, &session, &format!("Alerta {tag}"), tag, &ids);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(4);
        shot(&mut h, &format!("24-painel-alerta-{tag}"));
        h.key_press(egui::Key::J);
        h.run_steps(3);
        let first = ids[0];
        let snap = match &h.state().screen {
            Screen::Game(g) => g.snapshot(),
            _ => panic!("not in a campaign"),
        };
        if let Some(e) = snap.journal.as_ref().and_then(|j| j.entry(first)) {
            let title = bf_client::screens::journal::text(
                &h.state().shared.i18n,
                h.state().shared.world.as_ref(),
                &e.title,
            );
            if let Some(n) = h
                .get_all(egui_kittest::kittest::by().label(title.as_str()))
                .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
            {
                n.click();
            }
            h.run_steps(3);
        }
        shot(&mut h, &format!("25-agendas-alerta-{tag}"));
    }

    // English, 1280 × 720: the panel with pinned entries, and the agendas.
    let en_pins: Vec<&str> = journal
        .entries
        .iter()
        .map(|e| e.id.as_str())
        .filter(|id| *id != "works")
        .take(2)
        .collect();
    let en_save = write_campaign(&user, &session_for(&bra), "Brazil 1940", "BRA", &en_pins);
    let mut en = harness_sized(user.clone(), "en", egui::vec2(1280.0, 720.0));
    let ctx = en.ctx.clone();
    en.state_mut().apply(Transition::Game(en_save), &ctx);
    en.run_steps(4);
    shot(&mut en, "26-panel-en-1280");
    en.key_press(egui::Key::J);
    en.run_steps(3);
    shot(&mut en, "27-agendas-en-1280");
    en.key_press(egui::Key::Escape);
    en.run_steps(2);
    // The country panel on the right and the side panel on the left, side by side.
    let banner = egui::pos2(54.0, 42.0);
    en.hover_at(banner);
    en.run_steps(1);
    en.drag_at(banner);
    en.run_steps(1);
    en.drop_at(banner);
    en.hover_at(egui::pos2(640.0, 500.0));
    en.run_steps(3);
    shot(&mut en, "28-panel-and-country-en-1280");

    let _ = std::fs::remove_dir_all(&user);
}

/// The engine save of a campaign, loaded again.
fn session_for(save: &saves::Save) -> bf_sim::game::Session {
    let bytes = std::fs::read(save.dir.join(saves::WORLD_FILE)).unwrap();
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    bf_sim::game::Session::load(&bytes, 2, Some(data.rules())).unwrap()
}
