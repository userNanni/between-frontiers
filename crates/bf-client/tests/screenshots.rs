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
    let size = egui::vec2(1600.0, 900.0);
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        format!("(locale: \"{locale}\", ui_scale: 100, map_labels: true, autosave: Yearly)"),
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
    lens(&mut h, "Infra");
    h.run_steps(3);
    shot(&mut h, "06j-modo-transporte");
    lens(&mut h, "Político");
    h.run_steps(2);
    h.run_steps(2);
    h.get_by_label("Menu (F10)").click();
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
    lens(&mut h, "Recursos");
    h.run_steps(2);
    lens(&mut h, "Cobre");
    h.run_steps(3);
    shot(&mut h, "06f-modo-cobre");
    lens(&mut h, "Bauxita");
    h.run_steps(3);
    shot(&mut h, "06g-modo-bauxita");
    lens(&mut h, "Clima");
    h.run_steps(3);
    shot(&mut h, "06h-modo-clima");
    lens(&mut h, "Agro");
    h.run_steps(2);
    lens(&mut h, "Cafezal");
    h.run_steps(3);
    shot(&mut h, "06i-modo-cafe");
    lens(&mut h, "Político");
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

/// Drags with the left button from `from` to `to`, in steps, as a hand would.
fn drag(h: &mut Harness<'_, App>, from: egui::Pos2, to: egui::Pos2) {
    h.hover_at(from);
    h.run_steps(1);
    h.drag_at(from);
    h.run_steps(1);
    for k in 1..=6 {
        let t = k as f32 / 6.0;
        h.hover_at(from + (to - from) * t);
        h.run_steps(1);
    }
    h.drop_at(to);
    h.run_steps(2);
}

/// The HUD arranged by hand: the standard HUD of a player who never moved it, the edit mode
/// with its handles, panels dragged elsewhere, the streaming preset with its free corner, and
/// the same layouts in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn modular_hud() {
    let user = std::env::temp_dir().join(format!("bf-shots-hud-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(laplace_schelling::calendar::Date::new(1940, 3, 1));
    let bra = write_campaign(&user, &session, "Brasil 1940", "BRA", &["works-large"]);
    let open = |h: &mut Harness<'_, App>| {
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(bra.clone()), &ctx);
        h.run_steps(4);
    };
    let pt = user.join("pt");
    let mut h = harness_sized(pt.clone(), "pt-BR", egui::vec2(1600.0, 900.0));
    open(&mut h);
    assert!(
        !bf_client::hud::file(&pt).exists(),
        "a new player has no layout file"
    );
    shot(&mut h, "30-hud-padrao");

    // Edit mode, from the in-game menu.
    h.get_by_label("Menu (F10)").click();
    h.run_steps(2);
    h.get_by_label("Arrumar o HUD").click();
    h.run_steps(3);
    shot(&mut h, "31-hud-edicao");

    // The overview to the right edge, the map modes to the bottom middle.
    drag(&mut h, egui::pos2(60.0, 154.0), egui::pos2(1350.0, 300.0));
    drag(&mut h, egui::pos2(100.0, 868.0), egui::pos2(740.0, 868.0));
    shot(&mut h, "32-hud-edicao-arrastado");
    h.get_by_label("Concluir").click();
    h.run_steps(3);
    shot(&mut h, "33-hud-rearranjado");
    let layout = bf_client::hud::Layout::load(&bf_client::hud::file(&pt));
    assert_eq!(layout.preset, bf_client::hud::Preset::Custom, "{layout:?}");
    let rearranged = std::fs::read_to_string(bf_client::hud::file(&pt)).unwrap();

    // The streaming preset: a free corner the panels keep out of.
    h.key_press(egui::Key::F8);
    h.run_steps(2);
    h.get_by_label("Transmissão").click();
    h.run_steps(3);
    shot(&mut h, "34-hud-transmissao-edicao");
    let streaming = std::fs::read_to_string(bf_client::hud::file(&pt)).unwrap();
    // The free area grows by its lower left corner, by hand.
    let free = bf_client::hud::Layout::load(&bf_client::hud::file(&pt))
        .free
        .expect("free area")
        .rect(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1600.0, 900.0),
        ));
    drag(
        &mut h,
        free.left_bottom(),
        free.left_bottom() + egui::vec2(-60.0, 40.0),
    );
    shot(&mut h, "34b-hud-area-livre-redimensionada");
    h.get_by_label("Concluir").click();
    h.run_steps(2);
    let banner = egui::pos2(54.0, 42.0);
    h.hover_at(banner);
    h.run_steps(1);
    h.drag_at(banner);
    h.run_steps(1);
    h.drop_at(banner);
    h.hover_at(egui::pos2(800.0, 500.0));
    h.run_steps(3);
    shot(&mut h, "35-hud-transmissao");

    // The same layouts at 1280 × 720, in English.
    for (name, text) in [("rearranged", &rearranged), ("streaming", &streaming)] {
        let dir = user.join(format!("en-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(bf_client::hud::file(&dir), text).unwrap();
        let mut en = harness_sized(dir, "en", egui::vec2(1280.0, 720.0));
        open(&mut en);
        en.hover_at(banner);
        en.run_steps(1);
        en.drag_at(banner);
        en.run_steps(1);
        en.drop_at(banner);
        en.hover_at(egui::pos2(640.0, 400.0));
        en.run_steps(3);
        shot(&mut en, &format!("36-hud-{name}-en-1280"));
        en.key_press(egui::Key::F8);
        en.run_steps(3);
        shot(&mut en, &format!("37-hud-{name}-edit-en-1280"));
    }

    let _ = std::fs::remove_dir_all(&user);
}

/// The rect of the button labelled `label`.
fn button_at(h: &Harness<'_, App>, label: &str) -> egui::Rect {
    h.get_all(egui_kittest::kittest::by().label(label))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .unwrap_or_else(|| panic!("button {label}"))
        .rect()
}

/// The country figures arranged by blocks: the standard panel, the block editor with its tray,
/// fiscal figures dragged to the large row, a click on a block, the overview's sections, and the
/// same in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn country_figure_blocks() {
    let user = std::env::temp_dir().join(format!("bf-shots-tiles-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(laplace_schelling::calendar::Date::new(1940, 3, 1));
    let bra = write_campaign(&user, &session, "Brasil 1940", "BRA", &["works-large"]);
    let open = |h: &mut Harness<'_, App>| {
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(bra.clone()), &ctx);
        h.run_steps(4);
    };
    let pt = user.join("pt");
    let mut h = harness_sized(pt.clone(), "pt-BR", egui::vec2(1600.0, 900.0));
    open(&mut h);
    shot(&mut h, "40-numeros-padrao");

    // The editor, from the edit mode's toolbar.
    h.key_press(egui::Key::F8);
    h.run_steps(2);
    h.get_by_label("Editar os números do país").click();
    h.run_steps(3);
    shot(&mut h, "41-editor-de-blocos");

    // Debt, inflation and balance to the head of the large row, in that order.
    for (k, name) in ["Dívida pública", "Inflação", "Saldo anual"]
        .into_iter()
        .enumerate()
    {
        let from = button_at(&h, name).center();
        let used: Vec<egui::Rect> = {
            let layout = bf_client::hud::Layout::load(&bf_client::hud::file(&pt));
            let first = layout.tiles()[k];
            let label = h.state().shared.i18n.tr(&format!("tile-short-{first}"));
            vec![button_at(&h, &label)]
        };
        drag(&mut h, from, used[0].left_center() + egui::vec2(4.0, 0.0));
    }
    shot(&mut h, "42-editor-fiscal");
    let layout = bf_client::hud::Layout::load(&bf_client::hud::file(&pt));
    assert_eq!(&layout.tiles()[..3], &["debt", "inflation", "balance"]);
    h.get_by_label("Concluir").click();
    h.run_steps(2);

    // The overview's sections: pinned agendas first.
    h.get_by_label("Editar as seções do acompanhamento").click();
    h.run_steps(3);
    let pinned = button_at(&h, "Agendas fixadas").center();
    let works = button_at(&h, "Obras").left_center() + egui::vec2(4.0, 0.0);
    drag(&mut h, pinned, works);
    shot(&mut h, "43-editor-secoes");
    h.get_by_label("Concluir").click();
    h.run_steps(2);
    h.get_by_label("Concluir").click();
    h.run_steps(3);
    shot(&mut h, "44-numeros-fiscais");

    // A click on the debt opens money and debt.
    h.get_all(egui_kittest::kittest::by().label_contains("Dívida pública:"))
        .next()
        .expect("debt block")
        .click();
    h.run_steps(4);
    shot(&mut h, "45-clique-divida");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    let arranged = std::fs::read_to_string(bf_client::hud::file(&pt)).unwrap();

    // English, 1280 × 720.
    let dir = user.join("en");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(bf_client::hud::file(&dir), &arranged).unwrap();
    let mut en = harness_sized(dir, "en", egui::vec2(1280.0, 720.0));
    open(&mut en);
    shot(&mut en, "46-figures-fiscal-en-1280");
    en.key_press(egui::Key::F8);
    en.run_steps(2);
    en.get_by_label("Edit the country figures").click();
    en.run_steps(3);
    shot(&mut en, "47-blocks-editor-en-1280");

    let _ = std::fs::remove_dir_all(&user);
}

/// Taps the key-tips leader (Alt, pressed and released alone).
fn leader(h: &mut Harness<'_, App>) {
    h.key_press(egui::Key::AltLeft);
    h.run_steps(3);
}

/// Types a key tip and lets the element act.
fn tip(h: &mut Harness<'_, App>, key: egui::Key) {
    h.key_press(key);
    h.run_steps(4);
}

fn government_tab(h: &Harness<'_, App>) -> Option<bf_client::screens::government::Tab> {
    match &h.state().screen {
        Screen::Game(g) => g.government_tab(),
        _ => None,
    }
}

/// Keyboard: key tips over the map and over the government window, a sequence of tips opening
/// the laws, and the shortcut editor with its drawn keyboard (ABNT2, Ctrl layer, a conflict,
/// then English on ANSI at 1280 × 720).
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn keyboard_shortcuts() {
    use bf_client::screens::government::Tab;
    let user = std::env::temp_dir().join(format!("bf-shots-keys-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut h = harness(user.clone(), "pt-BR");
    open_campaign(&mut h, &user, "Atalhos", "BRA");
    run_until(&mut h, 20, |s| s.tick >= 3);

    // 1. Key tips over the map. AltGr types characters on ABNT2: it does not turn them on.
    h.key_press(egui::Key::AltRight);
    h.run_steps(3);
    assert!(
        !h.state().shared.keys.hint_mode(),
        "AltGr must not be the leader"
    );
    leader(&mut h);
    assert!(h.state().shared.keys.hint_mode());
    shot(&mut h, "12a-dicas-mapa");

    // 2. Leader, G, L: the government window on its laws, with the tips of the new screen.
    tip(&mut h, egui::Key::G);
    assert_eq!(government_tab(&h), Some(Tab::Budget));
    tip(&mut h, egui::Key::L);
    assert_eq!(government_tab(&h), Some(Tab::Laws));
    assert!(
        h.state().shared.keys.hint_mode(),
        "tips stay on for the next step"
    );
    shot(&mut h, "12b-dicas-governo-leis");
    // Esc leaves the tips first, then closes the window.
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(!h.state().shared.keys.hint_mode());
    assert_eq!(government_tab(&h), Some(Tab::Laws));
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert_eq!(government_tab(&h), None);
    // The plain shortcut does the same as the tip.
    h.key_press(egui::Key::L);
    h.run_steps(2);
    assert_eq!(government_tab(&h), Some(Tab::Laws));
    h.key_press(egui::Key::Escape);
    h.run_steps(2);

    // 3. The editor, from the settings of the main menu.
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Menu, &ctx);
    h.run_steps(2);
    h.get_by_label("Configurações").click();
    h.run_steps(2);
    h.get_by_label("Editar atalhos").click();
    h.run_steps(3);
    // Point at G so its command shows.
    let g = h
        .get_all(egui_kittest::kittest::by().label_contains("(KeyG)"))
        .next()
        .expect("drawn G key")
        .rect();
    h.hover_at(g.center());
    h.run_steps(4);
    shot(&mut h, "12c-teclado-abnt2");

    // 4. The Ctrl layer.
    h.hover_at(egui::pos2(10.0, 10.0));
    h.get_by_label("Ctrl").click();
    h.run_steps(3);
    shot(&mut h, "12d-teclado-ctrl");
    h.get_by_label("Ctrl").click();
    h.run_steps(2);

    // 5. Research asks for G, which the government window holds.
    list_button(&h, "Pesquisa").scroll_to_me();
    h.run_steps(3);
    list_button(&h, "Pesquisa").click();
    h.run_steps(2);
    h.get_all(egui_kittest::kittest::by().label_contains("(KeyG)"))
        .next()
        .unwrap()
        .click();
    h.run_steps(3);
    assert!(
        h.get_all(egui_kittest::kittest::by().label("Trocar"))
            .next()
            .is_some(),
        "conflict offers a swap"
    );
    shot(&mut h, "12e-conflito");
    h.get_by_label("Trocar").click();
    h.run_steps(2);
    {
        let keys = &h.state().shared.keys;
        let g = bf_client::keys::Chord::key(bf_client::keys::PhysKey::KeyG);
        assert_eq!(keys.holder(g), Some(bf_client::keys::id::RESEARCH));
    }
    h.get_by_label("Desfazer").click();
    h.run_steps(2);

    // 6. English, ANSI, 1280 × 720.
    let mut en = harness_sized(user.clone(), "en", egui::vec2(1280.0, 720.0));
    en.get_by_label("Settings").click();
    en.run_steps(2);
    en.get_by_label("Edit shortcuts").click();
    en.run_steps(2);
    en.get_by_label("ANSI").click();
    en.run_steps(2);
    list_button(&en, "Save the campaign").scroll_to_me();
    en.run_steps(3);
    list_button(&en, "Save the campaign").click();
    en.run_steps(3);
    shot(&mut en, "12f-editor-en-ansi-1280");
    en.get_by_label("Done").click();
    en.run_steps(2);
    en.get_by_label("Done").click();
    en.run_steps(2);

    // 7. Key tips at 1280 × 720 with the government window open.
    open_campaign(&mut en, &user, "Keys", "FRA");
    run_until(&mut en, 20, |s| s.tick >= 3);
    en.key_press(egui::Key::G);
    en.run_steps(3);
    leader(&mut en);
    shot(&mut en, "12g-tips-1280-government");
    leader(&mut en);
    en.key_press(egui::Key::Escape);
    en.run_steps(2);

    // 8. Arranging the HUD (F8): key tips on the handles and the toolbar.
    en.key_press(egui::Key::F8);
    en.run_steps(3);
    leader(&mut en);
    shot(&mut en, "12h-tips-1280-hud-edit");
    leader(&mut en);
    // The country figures' block editor (Shift+F8), with its tips.
    en.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F8);
    en.run_steps(3);
    leader(&mut en);
    shot(&mut en, "12i-tips-1280-blocks");
    leader(&mut en);
    en.key_press(egui::Key::F8);
    en.run_steps(2);
    // The lens picker opened with V, and its tips.
    en.key_press(egui::Key::V);
    en.run_steps(3);
    leader(&mut en);
    shot(&mut en, "12j-tips-1280-lens-picker");
    leader(&mut en);
    en.key_press(egui::Key::Escape);
    en.run_steps(2);

    let _ = std::fs::remove_dir_all(&user);
}

/// Clicks the lens `label`, opening the folded map mode picker first when it is closed.
fn lens(h: &mut Harness<'_, App>, label: &str) {
    if h.query_all(egui_kittest::kittest::by().label(label))
        .next()
        .is_none()
    {
        h.get_all(egui_kittest::kittest::by().label_contains("Modo do mapa:"))
            .next()
            .expect("map mode button")
            .click();
        h.run_steps(2);
    }
    h.get_by_label(label).click();
}

/// The map mode picker folded into a button: the lens in use on it, open, open from another
/// corner, kept open, and in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn lens_picker() {
    let user = std::env::temp_dir().join(format!("bf-shots-lens-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(laplace_schelling::calendar::Date::new(1940, 3, 1));
    let bra = write_campaign(&user, &session, "Brasil 1940", "BRA", &["works-large"]);
    let open = |h: &mut Harness<'_, App>| {
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(bra.clone()), &ctx);
        h.run_steps(4);
    };
    let pt = user.join("pt");
    let mut h = harness_sized(pt.clone(), "pt-BR", egui::vec2(1600.0, 900.0));
    open(&mut h);
    // Copper chosen: the button names it, and folds back.
    lens(&mut h, "Recursos");
    h.run_steps(2);
    shot(&mut h, "50-lente-aberta");
    lens(&mut h, "Cobre");
    h.run_steps(3);
    shot(&mut h, "51-lente-botao-cobre");
    assert!(
        h.query_all(egui_kittest::kittest::by().label("Cobre"))
            .next()
            .is_none(),
        "the picker folded after the choice"
    );
    // Esc folds it without a change.
    lens(&mut h, "Clima");
    h.run_steps(2);
    lens(&mut h, "Político");
    h.run_steps(2);

    // The button in the lower right corner opens up and to the left.
    let mut layout = bf_client::hud::Layout::default();
    layout.set_place(
        bf_client::hud::PanelId::MapModes,
        bf_client::hud::Placement {
            anchor: bf_client::hud::Anchor::BottomRight,
            offset: [-12.0, -12.0],
            visible: true,
        },
    );
    let corner = user.join("corner");
    std::fs::create_dir_all(&corner).unwrap();
    layout.save(&bf_client::hud::file(&corner)).unwrap();
    let mut c = harness_sized(corner, "pt-BR", egui::vec2(1600.0, 900.0));
    open(&mut c);
    c.get_all(egui_kittest::kittest::by().label_contains("Modo do mapa:"))
        .next()
        .unwrap()
        .click();
    c.run_steps(3);
    shot(&mut c, "52-lente-canto-direito");
    c.key_press(egui::Key::Escape);
    c.run_steps(2);
    assert!(
        c.query_all(egui_kittest::kittest::by().label("Recursos"))
            .next()
            .is_none(),
        "Esc folds the picker"
    );
    // Kept open, as the panel always was.
    c.get_all(egui_kittest::kittest::by().label_contains("Modo do mapa:"))
        .next()
        .unwrap()
        .click();
    c.run_steps(2);
    c.get_by_label("Manter aberto").click();
    c.run_steps(3);
    shot(&mut c, "53-lente-mantida-aberta");

    // English, 1280 × 720.
    let mut en = harness_sized(user.join("en"), "en", egui::vec2(1280.0, 720.0));
    open(&mut en);
    shot(&mut en, "54-lens-button-en-1280");
    en.get_all(egui_kittest::kittest::by().label_contains("Map mode:"))
        .next()
        .unwrap()
        .click();
    en.run_steps(3);
    shot(&mut en, "55-lens-open-en-1280");

    let _ = std::fs::remove_dir_all(&user);
}

/// The button labelled `label` (a command in the editor's list, not its category caption).
fn list_button<'a>(h: &'a Harness<'_, App>, label: &'a str) -> egui_kittest::Node<'a> {
    h.get_all(egui_kittest::kittest::by().label(label))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .unwrap_or_else(|| panic!("{label} in the list"))
}
