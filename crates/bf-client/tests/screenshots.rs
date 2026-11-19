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
    advanced_session_of("BRA", until)
}

/// [`advanced_session`] for the country `tag`.
fn advanced_session_of(
    tag: &str,
    until: laplace_schelling::calendar::Date,
) -> bf_sim::game::Session {
    advanced_session_with(tag, until, bf_sim::game::Switches::default())
}

/// [`advanced_session_of`] with rules of `data/rules.ron` switched on.
fn advanced_session_with(
    tag: &str,
    until: laplace_schelling::calendar::Date,
    switches: bf_sim::game::Switches,
) -> bf_sim::game::Session {
    let mut data = bf_sim::data::GameData::load(&root()).expect("game data");
    data.switches = switches;
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: tag.into() });
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
        military: rules.war.as_ref().map(|(m, _)| m.as_ref()),
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

/// Fails when a part of the open map mode picker leaves its frame: the families, the pin and
/// every item of the list in view.
fn assert_picker_fits(h: &Harness<'_, App>, title: &str, families: &[String], case: &str) {
    let pane = h
        .query_all(egui_kittest::kittest::by().label(title))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Pane)
        .unwrap_or_else(|| panic!("{case}: no open picker"))
        .rect();
    for f in families {
        let r = h
            .query_all(egui_kittest::kittest::by().label(f.as_str()))
            .map(|n| n.rect())
            .find(|r| pane.intersects(*r))
            .unwrap_or_else(|| panic!("{case}: family {f} not in the picker"));
        assert!(
            pane.contains_rect(r),
            "{case}: family {f} {r:?} out of {pane:?}"
        );
    }
    // Items of the list: those in view sit inside the frame across.
    for n in h.query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::Button)) {
        let r = n.rect();
        if pane.contains(r.center()) {
            assert!(
                r.left() >= pane.left() && r.right() <= pane.right(),
                "{case}: {:?} {r:?} out of {pane:?}",
                n.accesskit_node().label()
            );
        }
    }
}

/// The open map mode picker, every family, from the four corners, at 1600 × 900 and
/// 1280 × 720, in Portuguese and English: nothing of it leaves its frame or the screen.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn lens_picker_fits_everywhere() {
    let user = std::env::temp_dir().join(format!("bf-shots-lensfit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(laplace_schelling::calendar::Date::new(1925, 3, 1));
    let bra = write_campaign(&user, &session, "Brasil", "BRA", &[]);
    use bf_client::hud::{Anchor, Layout, PanelId, Placement};
    let corners = [
        ("bl", Anchor::BottomLeft, [12.0, -12.0]),
        ("br", Anchor::BottomRight, [-12.0, -12.0]),
        ("tr", Anchor::TopRight, [-12.0, 80.0]),
        ("tl", Anchor::TopLeft, [12.0, 150.0]),
    ];
    for (locale, button, title) in [
        ("pt-BR", "Modo do mapa:", "Modo do mapa"),
        ("en", "Map mode:", "Map mode"),
    ] {
        for (w, h_) in [(1600.0, 900.0), (1280.0, 720.0)] {
            for (corner, anchor, offset) in corners {
                let dir = user.join(format!("{locale}-{w}-{corner}"));
                std::fs::create_dir_all(&dir).unwrap();
                let mut layout = Layout::default();
                layout.set_place(
                    PanelId::MapModes,
                    Placement {
                        anchor,
                        offset,
                        visible: true,
                    },
                );
                layout.save(&bf_client::hud::file(&dir)).unwrap();
                let mut h = harness_sized(dir, locale, egui::vec2(w, h_));
                let ctx = h.ctx.clone();
                h.state_mut().apply(Transition::Game(bra.clone()), &ctx);
                h.run_steps(4);
                let families =
                    bf_client::screens::lens_panel::family_labels(&h.state().shared.i18n);
                for (k, family) in families.clone().iter().enumerate() {
                    if h.query_all(egui_kittest::kittest::by().label(title))
                        .all(|n| n.accesskit_node().role() != egui::accesskit::Role::Pane)
                    {
                        h.get_all(egui_kittest::kittest::by().label_contains(button))
                            .next()
                            .expect("map mode button")
                            .click();
                        h.run_steps(3);
                    }
                    h.get_by_label(family).click();
                    h.run_steps(4);
                    // A family without a list folds the picker: open it again to look.
                    if h.query_all(egui_kittest::kittest::by().label(title))
                        .all(|n| n.accesskit_node().role() != egui::accesskit::Role::Pane)
                    {
                        h.get_all(egui_kittest::kittest::by().label_contains(button))
                            .next()
                            .unwrap()
                            .click();
                        h.run_steps(4);
                    }
                    let case = format!("{locale} {w}×{h_} {corner} {family}");
                    assert_picker_fits(&h, title, &families, &case);
                    if corner == "br" || (w < 1300.0 && k == 1) {
                        shot(
                            &mut h,
                            &format!("56-lens-{locale}-{}-{corner}-{k}", w as i32),
                        );
                    }
                    h.key_press(egui::Key::Escape);
                    h.run_steps(2);
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

fn harness_scaled(
    user: PathBuf,
    locale: &str,
    size: egui::Vec2,
    scale: &str,
) -> Harness<'static, App> {
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        format!("(locale: \"{locale}\", {scale}, map_labels: true, autosave: Off)"),
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

/// The interface scale: the settings, 4K at 100 %, at 200 % and as a stream, the Steam Deck's
/// 1280 × 800, an ultrawide 3440 × 1440 with the HUD in the middle band, and the floor of
/// 960 × 540 with panels folded.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn interface_scale() {
    let user = std::env::temp_dir().join(format!("bf-shots-scale-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(laplace_schelling::calendar::Date::new(1930, 3, 1));
    let bra = write_campaign(&user, &session, "Brasil 1930", "BRA", &["works-large"]);
    let open = |h: &mut Harness<'_, App>| {
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(bra.clone()), &ctx);
        h.run_steps(6);
    };
    let report = |h: &Harness<'_, App>, name: &str| {
        println!(
            "{name}: {:.0}% logical {:?}",
            h.ctx.pixels_per_point() * 100.0,
            h.ctx.content_rect().size()
        );
    };

    // The settings, with the trial bar after a change.
    let mut s = harness_scaled(
        user.join("s"),
        "pt-BR",
        egui::vec2(1600.0, 900.0),
        "ui_scale: 0",
    );
    s.run_steps(4);
    s.get_by_label("Configurações").click();
    s.run_steps(4);
    shot(&mut s, "60-escala-configuracoes");
    s.get_by_label("Fixa").click();
    s.run_steps(4);
    // The automatic scale is 100 % here: one step up, so the bar that asks to keep it shows.
    s.get_all(egui_kittest::kittest::by().label("+"))
        .next()
        .expect("scale stepper")
        .click();
    s.run_steps(6);
    shot(&mut s, "61-escala-fixa-reverter");
    drop(s);

    for (dir, size, scale, name) in [
        ("4k100", (3840.0, 2160.0), "ui_scale: 100", "62-4k-100"),
        ("4k200", (3840.0, 2160.0), "ui_scale: 200", "63-4k-200"),
        (
            "4kstream",
            (3840.0, 2160.0),
            "ui_scale: 0, ui_stream: Some(Hd)",
            "64-4k-transmissao",
        ),
        ("deck", (1280.0, 800.0), "ui_scale: 0", "65-1280x800-auto"),
        (
            "deck100",
            (1280.0, 800.0),
            "ui_scale: 100",
            "66-1280x800-100",
        ),
        ("uw", (3440.0, 1440.0), "ui_scale: 0", "67-3440x1440-auto"),
        (
            "floor",
            (960.0, 540.0),
            "ui_scale: 100",
            "69-960x540-transmissao-recolhido",
        ),
    ] {
        let mut h = harness_scaled(user.join(dir), "pt-BR", egui::vec2(size.0, size.1), scale);
        if dir == "floor" {
            let layout = bf_client::hud::Layout::preset(bf_client::hud::Preset::Streaming);
            layout.save(&bf_client::hud::file(&user.join(dir))).unwrap();
            h = harness_scaled(user.join(dir), "pt-BR", egui::vec2(size.0, size.1), scale);
        }
        open(&mut h);
        h.key_press(egui::Key::C);
        h.run_steps(4);
        report(&h, name);
        shot(&mut h, name);
        if dir == "uw" {
            // The HUD in the middle 16:9 band.
            h.state_mut().shared.hud.layout.center_band = true;
            h.state_mut().shared.hud.changed();
            h.run_steps(4);
            shot(&mut h, "68-3440x1440-faixa-central");
        }
        if dir == "floor" {
            h.get_by_label("Acompanhamento").click();
            h.run_steps(4);
            shot(&mut h, "69b-960x540-acompanhamento-aberto");
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The budget's revenue by source: income tax by class, tax on consumption and profit of state
/// firms, with the causes in their tooltips; Turkey, which has all three from the start.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn budget_revenue_sources() {
    let user = std::env::temp_dir().join(format!("bf-shots-revenue-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "TUR".into() });
    while session.clock().date < laplace_schelling::calendar::Date::new(1925, 3, 1) {
        session.sim.step();
    }
    let tur = write_campaign(&user, &session, "Turquia 1925", "TUR", &[]);
    for (locale, size, revenue, indirect, firms, name) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "Orçamento",
            "Imposto sobre consumo",
            "Lucro de estatais",
            "70-orcamento-receitas",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "Budget",
            "Tax on consumption",
            "Profit of state firms",
            "72-budget-revenues-en-1280",
        ),
    ] {
        let mut h = harness_sized(user.join(locale), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(tur.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(revenue).click();
        h.run_steps(4);
        // The sources add up to the total the window shows.
        let view = match &h.state().screen {
            Screen::Game(g) => g.snapshot().government.expect("budget"),
            _ => panic!("not in a campaign"),
        };
        let [income, consumption, state, customs] = view.revenue_sources();
        println!(
            "{locale}: income {income:.0} consumption {consumption:.0} firms {state:.0} customs {customs:.0}"
        );
        assert!(consumption > 0.0 && state > 0.0, "Turkey has both sources");
        let sum = income + consumption + state + customs;
        assert!((sum - view.budget.revenue.to_f64_lossy()).abs() < 1.0);
        let hover = |h: &mut Harness<'_, App>, label: &str| {
            let r = h
                .get_all(egui_kittest::kittest::by().label(label))
                .next()
                .unwrap_or_else(|| panic!("{label} in the budget"))
                .rect();
            h.hover_at(r.center());
            h.run_steps(8);
        };
        hover(&mut h, indirect);
        shot(&mut h, name);
        hover(&mut h, firms);
        shot(&mut h, &format!("{name}-estatais"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Research in the budget: the laboratories' bill (the treasury's legal share, the firms' fund
/// pays the rest) and the public works category "Research" whose weight the player sets; the
/// United States, three months into 1925.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn budget_research() {
    let user = std::env::temp_dir().join(format!("bf-shots-research-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "USA".into() });
    while session.clock().date < laplace_schelling::calendar::Date::new(1925, 4, 1) {
        session.sim.step();
    }
    let usa = write_campaign(&user, &session, "EUA 1925", "USA", &[]);
    for (locale, size, tab, line, category, name) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "Orçamento",
            "…dos quais pesquisa",
            "Pesquisa",
            "90-orcamento-pesquisa",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "Budget",
            "…of which research",
            "Research",
            "91-budget-research-en-1280",
        ),
    ] {
        let mut h = harness_sized(user.join(locale), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(usa.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(tab).click();
        h.run_steps(4);
        let view = match &h.state().screen {
            Screen::Game(g) => g.snapshot().government.expect("budget"),
            _ => panic!("not in a campaign"),
        };
        println!(
            "{locale}: research state {:.0} firms {:.0} share {:.2}",
            view.research[0], view.research[1], view.research_state_share
        );
        assert!(view.research[0] > 0.0 && view.research[1] > 0.0, "both pay");
        let hover = |h: &mut Harness<'_, App>, label: &str| {
            let r = h
                .get_all(egui_kittest::kittest::by().label_contains(label))
                .next()
                .unwrap_or_else(|| panic!("{label} in the budget"))
                .rect();
            h.hover_at(r.center());
            h.run_steps(8);
        };
        hover(&mut h, line);
        shot(&mut h, name);
        // The works list's own label: the top bar has a "Research (R)" button too.
        let r = h
            .get_all(egui_kittest::kittest::by().label_contains(&format!("{category} (")))
            .last()
            .expect("the research category among the public works")
            .rect();
        h.hover_at(r.center());
        h.run_steps(8);
        shot(&mut h, &format!("{name}-categoria"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The sea lens: the zones with their ice and names, the Mediterranean with Suez and Gibraltar,
/// the Baltic, and the route Bombay–Kent (London's state has no port in the graph) with Suez open and closed in the preview; in
/// Portuguese at 1600 × 900 and in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn sea_lens() {
    use bf_client::map::lens::Lens;
    let user = std::env::temp_dir().join(format!("bf-shots-sea-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(laplace_schelling::calendar::Date::new(1925, 6, 1));
    let save = write_campaign(&user, &session, "Mar 1925", "BRA", &[]);
    let (bombay, london, suez, gibraltar, oresund) = (358u16, 276u16, 11u16, 0u16, 2u16);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        let set_lens = |h: &mut Harness<'_, App>| {
            let app = h.state_mut();
            if let (Some(view), Some(world)) = (&mut app.shared.map_view, &app.shared.world) {
                bf_client::screens::lens_panel::choose(view, world, Lens::Sea, 1925);
            }
            h.run_steps(6);
        };
        set_lens(&mut h);
        shot(&mut h, &format!("80-sea-overview-{tag}"));
        let look = |h: &mut Harness<'_, App>, lon: f64, lat: f64, zoom: f32| {
            let app = h.state_mut();
            let meta = app.shared.world.as_ref().unwrap().map.meta.clone();
            let (x, y) = bf_client::world::project(&meta, lon, lat).unwrap();
            let view = app.shared.map_view.as_mut().unwrap();
            view.camera
                .as_mut()
                .unwrap()
                .look_at(egui::pos2(x, y), zoom);
            h.run_steps(4);
        };
        let marker = |h: &Harness<'_, App>, id: u16| {
            let app = h.state();
            let layer = app.shared.sea.as_ref().unwrap().as_ref().unwrap();
            let cam = app.shared.map_view.as_ref().unwrap().camera.unwrap();
            let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
            cam.to_screen(screen, layer.straits[id as usize])
        };
        let set = |h: &mut Harness<'_, App>,
                   f: &dyn Fn(&mut bf_client::screens::sea_lens::SeaView)| {
            if let Screen::Game(g) = &mut h.state_mut().screen {
                f(g.sea_view_mut());
            }
            h.run_steps(4);
        };
        // The Mediterranean: Suez's card kept open, Gibraltar's under the pointer.
        look(&mut h, 15.0, 36.0, size.x / 1600.0 * 1.0);
        set(&mut h, &|v| v.focus = Some(suez));
        let g = marker(&h, gibraltar);
        h.hover_at(g);
        h.run_steps(8);
        shot(&mut h, &format!("81-sea-mediterranean-{tag}"));
        // The Baltic, with the Øresund's card.
        h.hover_at(egui::pos2(5.0, 5.0));
        set(&mut h, &|v| v.focus = Some(oresund));
        look(&mut h, 18.0, 58.0, size.x / 1600.0 * 1.8);
        shot(&mut h, &format!("82-sea-baltic-{tag}"));
        // Bombay to London, Suez open, then closed in the preview.
        let ctx = h.ctx.clone();
        if let Some(v) = h.state_mut().shared.map_view.as_mut() {
            v.camera = None;
        }
        let _ = ctx;
        h.run_steps(2);
        set(&mut h, &|v| {
            v.focus = None;
            v.from = Some(bombay);
            v.to = Some(london);
        });
        shot(&mut h, &format!("83-sea-route-suez-open-{tag}"));
        set(&mut h, &|v| {
            v.closed.insert(suez);
        });
        shot(&mut h, &format!("84-sea-route-suez-closed-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The population panel: Brazil in 1925 and in 1960 (the whole country, then its largest
/// state), and Italy, a country of emigrants, in 1930; top and bottom of the panel, in
/// Portuguese at 1600 × 900 and in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn population_panel() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-pop-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let bra_1925 = advanced_session(Date::new(1925, 7, 1));
    let bra_1960 = advanced_session(Date::new(1960, 3, 1));
    let ita_1930 = advanced_session_of("ITA", Date::new(1930, 3, 1));
    let saves = [
        (
            write_campaign(&user, &bra_1925, "Brasil 1925", "BRA", &[]),
            "bra-1925",
            false,
        ),
        (
            write_campaign(&user, &bra_1960, "Brasil 1960", "BRA", &[]),
            "bra-1960",
            true,
        ),
        (
            write_campaign(&user, &ita_1930, "Itália 1930", "ITA", &[]),
            "ita-1930",
            false,
        ),
    ];
    for (locale, size, tag, bottom, classes, scale) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Ondas abertas",
            "Filhos por mulher por classe: País inteiro",
            "ui_scale: 100",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Open waves",
            "Children per woman by class: Whole country",
            "ui_scale: 100",
        ),
        (
            "pt-BR",
            egui::vec2(1280.0, 720.0),
            "pt-1280-125",
            "Ondas abertas",
            "Filhos por mulher por classe: País inteiro",
            "ui_scale: 125",
        ),
    ] {
        for (i, (save, name, state)) in saves.iter().enumerate() {
            let mut h = harness_scaled(user.join(format!("{tag}-{name}")), locale, size, scale);
            let ctx = h.ctx.clone();
            h.state_mut().apply(Transition::Game(save.clone()), &ctx);
            h.run_steps(4);
            // Children per woman by class is an output kept out of the save: it is back on the
            // 15th, at the first demography month after the load.
            run_until(&mut h, 120, |s| {
                s.population.as_ref().is_some_and(|p| !p.classes.is_empty())
            });
            h.key_press(egui::Key::Y);
            h.run_steps(4);
            let n = 90 + i * 2;
            shot(&mut h, &format!("{n}-population-{name}-{tag}"));
            // Children per woman by class, of the whole country.
            h.get_by_label(classes).scroll_to_me();
            h.run_steps(4);
            shot(&mut h, &format!("{n}-population-{name}-classes-{tag}"));
            // Communities, of the whole country and then of the state the pyramid shows.
            let communities = if locale == "pt-BR" {
                "Comunidades:"
            } else {
                "Communities:"
            };
            let scroll_to_communities = |h: &mut egui_kittest::Harness<'_, bf_client::App>| {
                h.get_all(egui_kittest::kittest::by().label_contains(communities))
                    .next()
                    .expect("communities heading")
                    .scroll_to_me();
                h.run_steps(4);
            };
            scroll_to_communities(&mut h);
            shot(&mut h, &format!("{n}-population-{name}-communities-{tag}"));
            if *state {
                h.get_by_label("›").click();
                h.run_steps(3);
                shot(&mut h, &format!("{n}-population-{name}-state-{tag}"));
                scroll_to_communities(&mut h);
                shot(
                    &mut h,
                    &format!("{n}-population-{name}-state-communities-{tag}"),
                );
            }
            h.get_by_label(bottom).scroll_to_me();
            h.run_steps(4);
            shot(&mut h, &format!("{}-population-{name}-bottom-{tag}", n + 1));
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Restore points: the settings, the in-game menu with the quick save, the dialog to save with
/// a name, and the campaigns screen with a campaign's points and its "…" menu; in Portuguese at
/// 1600 × 900 and in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn restore_points() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-points-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(Date::new(1925, 3, 1));
    for (locale, size, tag, name) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "Antes da reforma"),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Before the reform",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &session, "Brasil", "BRA", &[]);
        let mut h = harness_sized(home.clone(), locale, size);
        let ctx = h.ctx.clone();
        // A campaign with every kind of point.
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        let points_dir = save.dir.join("points");
        let count = |n: usize, h: &mut Harness<'_, App>| {
            let started = std::time::Instant::now();
            loop {
                let have = std::fs::read_dir(&points_dir)
                    .map(|d| {
                        d.filter_map(Result::ok)
                            .filter(|e| e.path().extension().is_some_and(|x| x == "ron"))
                            .count()
                    })
                    .unwrap_or(0);
                if have >= n || started.elapsed().as_secs() > 120 {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
                h.run_steps(1);
            }
        };
        h.key_press(egui::Key::F5);
        count(1, &mut h);
        // Two months with a monthly point each.
        h.state_mut().shared.settings.autosave = bf_client::settings::Autosave::Monthly;
        h.key_press(egui::Key::Space);
        count(3, &mut h);
        h.key_press(egui::Key::Space);
        h.run_steps(4);
        {
            let app = h.state_mut();
            if let Screen::Game(g) = &mut app.screen {
                g.restore_point_before(&app.shared, bf_client::points::Reason::Law);
            }
        }
        count(4, &mut h);
        h.key_press(egui::Key::F10);
        h.run_steps(4);
        shot(&mut h, &format!("101-game-menu-points-{tag}"));
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::S,
        );
        h.run_steps(4);
        let field = h
            .get_all(egui_kittest::kittest::by().role(egui::accesskit::Role::TextInput))
            .next()
            .expect("name field");
        field.focus();
        h.run_steps(1);
        h.event(egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        });
        h.event(egui::Event::Text(name.into()));
        h.run_steps(3);
        shot(&mut h, &format!("102-save-named-{tag}"));
        h.key_press(egui::Key::Enter);
        count(5, &mut h);
        h.run_steps(4);
        // Quick load: back to March, with where the player was kept as a point.
        h.key_press(egui::Key::F9);
        h.run_steps(6);
        match &h.state().screen {
            Screen::Game(g) => assert_eq!(g.snapshot().date, Date::new(1925, 3, 1)),
            _ => panic!("quick load left the campaign"),
        }
        count(6, &mut h);

        // The campaigns screen: the points, then the campaign's menu.
        h.state_mut().shared.refresh_saves();
        h.state_mut().apply(Transition::Saves, &ctx);
        h.run_steps(4);
        shot(&mut h, &format!("103-campaign-points-{tag}"));
        let options = if locale == "en" {
            "Campaign options"
        } else {
            "Opções da campanha"
        };
        h.get_all(egui_kittest::kittest::by().label_contains(options))
            .next()
            .expect("options button")
            .click();
        h.run_steps(3);
        shot(&mut h, &format!("104-campaign-menu-{tag}"));
        h.key_press(egui::Key::Escape);
        h.run_steps(2);

        // The settings, with what a point costs from the save just written.
        h.state_mut().apply(Transition::Settings, &ctx);
        h.run_steps(4);
        shot(&mut h, &format!("100-settings-points-{tag}"));
        h.get_all(egui_kittest::kittest::by().label_contains("MB · "))
            .next()
            .expect("cost line")
            .scroll_to_me();
        h.run_steps(4);
        shot(&mut h, &format!("100-settings-points-cost-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Borders (`docs/borders.pt-BR.md`): the Soviet Union freed Ukraine and Germany annexed Austria;
/// the map follows the owners. Freeing land is a decision in the journal window, confirmed in a
/// step of its own; playing another country is on the campaigns screen, behind "⋯".
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn borders_and_switching_country() {
    use bf_sim::borders::{Operation, Terms};
    use bf_sim::game::GameCommand;
    let user = std::env::temp_dir().join(format!("bf-shots-borders-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let ukraine: Vec<String> = [
        "05", "09", "12", "14", "18", "23", "35", "48", "51", "53", "59", "63", "65", "68", "71",
        "74", "KYIV",
    ]
    .iter()
    .map(|s| format!("UKR-{s}"))
    .collect();
    let mut session =
        advanced_session_of("SOV", laplace_schelling::calendar::Date::new(1925, 1, 5));
    session.submit(GameCommand::Liberate {
        by: "SOV".into(),
        states: ukraine,
        tag: Some("UKR".into()),
        subject: false,
    });
    session.submit(GameCommand::Borders {
        op: Operation::Annex {
            from: "AUT".into(),
            into: "GER".into(),
            colony: false,
            terms: Terms::default(),
        },
    });
    while session.clock().date < laplace_schelling::calendar::Date::new(1925, 2, 2) {
        session.sim.step();
    }
    assert_eq!(session.world().borders.changes.len(), 2);
    for (locale, suffix, size) in [
        ("pt-BR", "", egui::vec2(1600.0, 900.0)),
        ("en", "-en", egui::vec2(1280.0, 720.0)),
    ] {
        let dir = user.join(locale);
        let save = write_campaign(&dir, &session, "Fronteiras", "SOV", &[]);
        let mut h = harness_sized(dir.clone(), locale, size);
        let tr = |h: &Harness<'_, App>, key: &str| h.state().shared.i18n.tr(key);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(6);
        {
            let app = h.state_mut();
            let meta = app.shared.world.as_ref().unwrap().map.meta.clone();
            let (x, y) = bf_client::world::project(&meta, 30.0, 51.0).unwrap();
            let view = app.shared.map_view.as_mut().unwrap();
            view.camera.as_mut().unwrap().look_at(egui::pos2(x, y), 3.0);
        }
        h.run_steps(4);
        {
            let world = h.state().shared.world.as_ref().unwrap();
            let ukr = world
                .countries
                .iter()
                .position(|c| c.tag() == "UKR")
                .unwrap();
            assert!(world.countries[ukr].states.len() >= 17);
        }
        shot(&mut h, &format!("70-fronteiras-mapa{suffix}"));

        // The journal window: the decisions, below the agendas.
        h.key_press(egui::Key::J);
        h.run_steps(4);
        h.get_by_label(&tr(&h, "liberate-title")).click();
        h.run_steps(4);
        shot(&mut h, &format!("71-decisao-libertar{suffix}"));
        // Belarus, as a subject.
        let blr = bf_client::text::country_name(&h.state().shared.i18n, "BLR");
        h.get(egui_kittest::kittest::By::new().label_contains(&blr))
            .click();
        h.run_steps(2);
        h.get_by_label(&tr(&h, "liberate-subject")).click();
        h.run_steps(4);
        shot(&mut h, &format!("72-decisao-consequencias{suffix}"));
        h.get_by_label(&tr(&h, "liberate-review")).click();
        h.run_steps(4);
        shot(&mut h, &format!("73-decisao-confirmar{suffix}"));
        h.get_by_label(&tr(&h, "liberate-yes")).click();
        run_until(&mut h, 20, |s| {
            s.owners
                .as_ref()
                .is_some_and(|o| o.tags.iter().any(|t| t == "BLR"))
        });
        h.run_steps(4);
        {
            let world = h.state().shared.world.as_ref().unwrap();
            let blr = world
                .countries
                .iter()
                .position(|c| c.tag() == "BLR")
                .expect("Belarus");
            assert!(!world.countries[blr].states.is_empty());
            let sov = world
                .countries
                .iter()
                .position(|c| c.tag() == "SOV")
                .unwrap();
            assert_eq!(world.countries[blr].overlord, Some(sov));
        }

        // Out to the campaigns: "⋯ → play another country", Ukraine, and back in as Ukraine.
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Saves, &ctx);
        h.run_steps(4);
        h.get_by_label("…").click();
        h.run_steps(3);
        shot(&mut h, &format!("74-campanhas-mais{suffix}"));
        h.get_by_label(&tr(&h, "saves-menu-play-other")).click();
        h.run_steps(6);
        let kyiv = {
            let app = h.state();
            let world = app.shared.world.as_ref().unwrap();
            let (x, y) = bf_client::world::project(&world.map.meta, 30.52, 50.45).unwrap();
            let rect = h.ctx.content_rect();
            app.shared
                .map_view
                .as_ref()
                .unwrap()
                .camera
                .unwrap()
                .to_screen(rect, egui::pos2(x, y))
        };
        h.hover_at(kyiv);
        h.run_steps(1);
        h.drag_at(kyiv);
        h.run_steps(1);
        h.drop_at(kyiv);
        h.run_steps(4);
        shot(&mut h, &format!("75-escolher-outro-pais{suffix}"));
        h.get_by_label(&tr(&h, "pick-switch")).click();
        h.run_steps(4);
        assert!(matches!(h.state().screen, Screen::Saves(_)));
        let save = saves::list(&dir.join("saves"))
            .into_iter()
            .find(|s| s.meta.name == "Fronteiras")
            .unwrap();
        assert_eq!(save.meta.player.as_deref(), Some("UKR"));
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(8);
        let player = match &h.state().screen {
            Screen::Game(g) => g.snapshot().player,
            _ => None,
        };
        assert_eq!(player.as_deref(), Some("UKR"));
        shot(&mut h, &format!("76-jogando-ucrania{suffix}"));
    }
}

/// The campaign cards and their menu after the redesign: menu closed and open, a 60-character
/// name with a folder past a thousand MB at the floor of 960 × 540, and 1280 × 720 at 125 %.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn campaign_cards() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-cards-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(Date::new(1925, 2, 1));
    let long = "Campanha com um nome bem comprido para testar onde ele corta";
    let fill = |home: &std::path::Path| {
        write_campaign(home, &session, "Brasil", "BRA", &[]);
        write_campaign(home, &session, "Iugoslávia", "YUG", &[]);
        let big = write_campaign(home, &session, long, "YUG", &[]);
        std::fs::File::create(big.dir.join("filler.bin"))
            .unwrap()
            .set_len(1_234_000_000)
            .unwrap();
        write_campaign(home, &session, "União Soviética", "SOV", &[]);
    };
    let menu_label = |locale: &str| {
        if locale == "en" {
            "Campaign options"
        } else {
            "Opções da campanha"
        }
    };
    for (locale, size, tag, scale) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt", "ui_scale: 100"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280", "ui_scale: 100"),
        ("pt-BR", egui::vec2(960.0, 540.0), "pt-960", "ui_scale: 100"),
        (
            "pt-BR",
            egui::vec2(1280.0, 720.0),
            "pt-1280-125",
            "ui_scale: 125",
        ),
    ] {
        let home = user.join(tag);
        fill(&home);
        let mut h = harness_scaled(home.clone(), locale, size, scale);
        let ctx = h.ctx.clone();
        h.state_mut().shared.refresh_saves();
        h.state_mut().apply(Transition::Saves, &ctx);
        h.run_steps(4);
        shot(&mut h, &format!("106-campaigns-{tag}-v2"));
        h.get_all(egui_kittest::kittest::by().label_contains(menu_label(locale)))
            .next()
            .expect("options button")
            .click();
        h.run_steps(3);
        shot(&mut h, &format!("107-campaign-menu-{tag}-v2"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
/// The campaigns screen with a campaign written by a newer version of the game and one whose
/// file is damaged: each says why it does not open, and cannot be loaded.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn unreadable_campaigns() {
    use laplace_schelling::calendar::Date;
    use laplace_schelling::save::SaveFile;
    let user = std::env::temp_dir().join(format!("bf-shots-unreadable-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session(Date::new(1925, 2, 1));
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let home = user.join(tag);
        write_campaign(&home, &session, "Brasil", "BRA", &[]);
        // A newer world format: the same save with the next version in the world's header.
        let newer = write_campaign(&home, &session, "Do futuro", "BRA", &[]);
        let path = newer.dir.join(saves::WORLD_FILE);
        let mut file: SaveFile<bf_sim::game::GameCommand> =
            SaveFile::decode(&std::fs::read(&path).unwrap(), bf_sim::game::GAME_ID).unwrap();
        file.header.game_version = "0.2.0+f00dcafe1".into();
        let next = bf_sim::save_format::WORLD_VERSION + 1;
        file.snapshot[4..8].copy_from_slice(&next.to_le_bytes());
        std::fs::write(&path, file.encode().unwrap()).unwrap();
        // A damaged file: one byte flipped; its restore point is whole.
        let damaged = write_campaign(&home, &session, "Danificada", "BRA", &[]);
        let path = damaged.dir.join(saves::WORLD_FILE);
        std::fs::create_dir_all(bf_client::points::dir(&damaged)).unwrap();
        std::fs::copy(&path, bf_client::points::save_path(&damaged, "p1")).unwrap();
        bf_client::points::record(
            &damaged,
            bf_client::points::Point {
                id: "p1".into(),
                kind: bf_client::points::Kind::Auto,
                name: None,
                date: session.clock().date,
                written: saves::now(),
                bytes: 0,
            },
        )
        .unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        std::fs::write(&path, bytes).unwrap();
        assert!(matches!(
            saves::readable(&newer),
            saves::Readable::Newer { found, .. } if found == next
        ));
        assert!(matches!(
            saves::readable(&damaged),
            saves::Readable::Damaged(_)
        ));

        let mut h = harness_sized(home.clone(), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().shared.refresh_saves();
        h.state_mut().apply(Transition::Saves, &ctx);
        h.run_steps(4);
        shot(&mut h, &format!("105-campaigns-unreadable-{tag}"));
        // The damaged one selected: no load, and why beside, above its restore points.
        h.get(
            egui_kittest::kittest::by()
                .role(egui::accesskit::Role::Button)
                .label("Danificada"),
        )
        .click();
        h.run_steps(3);
        shot(&mut h, &format!("105-campaigns-damaged-{tag}"));
        h.get(
            egui_kittest::kittest::by()
                .role(egui::accesskit::Role::Button)
                .label("Do futuro"),
        )
        .click();
        h.run_steps(3);
        shot(&mut h, &format!("105-campaigns-newer-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The debts between governments tab: Britain in 1925 (it owes the United States and is owed
/// by Germany), in Portuguese at 1600 × 900 and in English at 1280 × 720, and Germany (owes four
/// governments) at the floor of 960 × 540.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn debts_tab() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-debts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    for (tag, locale, size, name, row) in [
        (
            "GBR",
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "gbr-pt",
            "Alemanha",
        ),
        (
            "GBR",
            "en",
            egui::vec2(1280.0, 720.0),
            "gbr-en-1280",
            "United States",
        ),
        (
            "GER",
            "pt-BR",
            egui::vec2(960.0, 540.0),
            "ger-pt-960",
            "França",
        ),
    ] {
        let session = advanced_session_of(tag, Date::new(1925, 3, 1));
        let home = user.join(name);
        let save = write_campaign(&home, &session, "Dívidas", tag, &[]);
        let mut h = harness_sized(home.clone(), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(3);
        let tab = if locale == "en" {
            "Debts between governments"
        } else {
            "Dívidas entre governos"
        };
        h.get_all(egui_kittest::kittest::by().label(tab))
            .next()
            .expect("debts tab")
            .click();
        h.run_steps(4);
        shot(&mut h, &format!("110-debts-{name}"));
        // Another line selected.
        let clicked = match h
            .query_all(
                egui_kittest::kittest::by()
                    .role(egui::accesskit::Role::Button)
                    .label(row),
            )
            .next()
        {
            Some(r) => {
                r.click();
                true
            }
            None => false,
        };
        if clicked {
            h.run_steps(3);
            shot(&mut h, &format!("111-debts-{name}-selected"));
        }
        // The other windows' X, beside the government's.
        if name == "gbr-pt" {
            h.key_press(egui::Key::G);
            h.key_press(egui::Key::J);
            h.run_steps(3);
            shot(&mut h, "112-close-x-agendas-pt");
            h.key_press(egui::Key::J);
            h.key_press(egui::Key::R);
            h.run_steps(4);
            shot(&mut h, "113-close-x-research-pt");
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The influence window: Britain in 1925 (Germany owes it, it owes the United States), both
/// sides, in Portuguese at 1600 × 900 and in English at 1280 × 720, and at the floor.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn influence_window() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-influence-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session_of("GBR", Date::new(1925, 3, 1));
    for (locale, size, name, other) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Quem pesa sobre nós",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Who weighs on us",
        ),
        (
            "pt-BR",
            egui::vec2(960.0, 540.0),
            "pt-960",
            "Quem pesa sobre nós",
        ),
    ] {
        let home = user.join(name);
        let save = write_campaign(&home, &session, "Influência", "GBR", &[]);
        let mut h = harness_sized(home.clone(), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(4);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::I);
        h.run_steps(4);
        shot(&mut h, &format!("120-influence-over-{name}"));
        let clicked = match h.query_all(egui_kittest::kittest::by().label(other)).next() {
            Some(b) => {
                b.click();
                true
            }
            None => false,
        };
        if clicked {
            h.run_steps(4);
            shot(&mut h, &format!("121-influence-on-{name}"));
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The armed forces (phase 5, slice A): the government's forces tab of Britain in peace, and the
/// journal's agenda with the decision to mobilize, asked and confirmed in two steps; then the
/// mobilization under way, with the reservists who reported. Portuguese at 1600 × 900, English
/// at 1280 × 720. Shots also go to `target/screenshots/war/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn armed_forces() {
    use laplace_schelling::calendar::Date;
    // The forces ship off; the session and the client read them on through `BF_RULES`.
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`), and no other
    // thread reads the environment while this one writes it.
    unsafe { std::env::set_var("BF_RULES", "forces") };
    let user = std::env::temp_dir().join(format!("bf-shots-forces-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let peace = advanced_session_of("GBR", Date::new(1926, 2, 1));
    let mut called = advanced_session_of("GBR", Date::new(1926, 2, 1));
    called.submit(bf_sim::game::GameCommand::Mobilize {
        tag: "GBR".into(),
        on: true,
    });
    while called.clock().date < Date::new(1926, 3, 2) {
        called.sim.step();
    }
    let fra = write_campaign(&user, &peace, "Reino Unido 1926", "GBR", &[]);
    let fra_called = write_campaign(&user, &called, "Reino Unido mobilizando", "GBR", &[]);
    let war_dir = root().join("target/screenshots/war");
    std::fs::create_dir_all(&war_dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, war_dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tab, agendas, ask, confirm, tag) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "Forças",
            "Forças armadas em paz",
            "Mobilizar a reserva…",
            "Confirmar a mobilização",
            "pt",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "Forces",
            "Armed forces in peace",
            "Mobilize the reserve…",
            "Confirm the mobilization",
            "en-1280",
        ),
    ] {
        let mut h = harness_sized(user.join(locale), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(fra.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.key_press(egui::Key::F);
        h.run_steps(4);
        if !matches!(&h.state().screen, Screen::Game(g)
            if g.government_tab() == Some(bf_client::screens::government::Tab::Forces))
        {
            h.get_by_label(tab).click();
            h.run_steps(4);
        }
        let name = format!("100-forces-tab-{tag}");
        shot(&mut h, &name);
        keep(&name);
        // The journal: the forces' agenda and its decision, asked and then waiting to confirm.
        h.key_press(egui::Key::G);
        h.run_steps(2);
        h.key_press(egui::Key::J);
        h.run_steps(4);
        h.get_by_label(agendas).scroll_to_me();
        h.run_steps(4);
        h.get_by_label(agendas).click();
        h.run_steps(4);
        let name = format!("101-forces-agenda-{tag}");
        shot(&mut h, &name);
        keep(&name);
        h.get_by_label(ask).scroll_to_me();
        h.run_steps(4);
        h.get_by_label(ask).click();
        h.run_steps(4);
        h.get_by_label(confirm);
        let name = format!("102-forces-agenda-confirm-{tag}");
        shot(&mut h, &name);
        keep(&name);
        // A month into the mobilization: the reservists who reported, in the tab.
        let mut h = harness_sized(user.join(format!("{locale}-called")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(fra_called.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(tab).click();
        h.run_steps(4);
        let name = format!("103-forces-mobilizing-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Schooling (`demography::education`): the population panel's section with the pyramid by level,
/// for Brazil in 1930 (the country, then its largest state), and the adult literacy campaign in
/// the budget, before and at its confirmation step; in Portuguese at 1600 × 900 and in English
/// at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn schooling_screens() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-school-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let bra = advanced_session_with(
        "BRA",
        Date::new(1930, 3, 1),
        bf_sim::game::Switches {
            schooling: true,
            ..Default::default()
        },
    );
    let save = write_campaign(&user, &bra, "Brasil 1930", "BRA", &[]);
    for (locale, size, tag, schooling, budget, campaign, start, yes) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Educação:",
            "Orçamento",
            "Campanha de alfabetização de adultos",
            "Começar campanha…",
            "Confirmar",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Schooling:",
            "Budget",
            "Adult literacy campaign",
            "Start a campaign…",
            "Confirm",
        ),
    ] {
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        run_until(&mut h, 120, |s| {
            s.population.as_ref().is_some_and(|p| p.schooling.is_some())
        });
        h.key_press(egui::Key::Y);
        h.run_steps(4);
        let to = |h: &mut Harness<'_, App>, text: &str| {
            h.get_all(egui_kittest::kittest::by().label_contains(text))
                .next()
                .unwrap_or_else(|| panic!("{text}"))
                .scroll_to_me();
            h.run_steps(4);
        };
        to(&mut h, schooling);
        shot(&mut h, &format!("96-schooling-bra-1930-{tag}"));
        h.get_by_label("›").click();
        h.run_steps(3);
        shot(
            &mut h,
            &format!("96-schooling-bra-1930-pyramid-state-{tag}"),
        );
        to(&mut h, schooling);
        shot(&mut h, &format!("96-schooling-bra-1930-state-{tag}"));
        h.key_press(egui::Key::Y);
        h.run_steps(3);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(budget).click();
        h.run_steps(4);
        to(&mut h, campaign);
        shot(&mut h, &format!("97-campaign-bra-1930-{tag}"));
        h.get_by_label(start).click();
        h.run_steps(4);
        assert!(
            h.get_all(egui_kittest::kittest::by().label(yes))
                .next()
                .is_some(),
            "the campaign's confirmation step"
        );
        to(&mut h, campaign);
        shot(&mut h, &format!("97-campaign-bra-1930-confirm-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The influence and blocs lenses (`docs/influence.pt-BR.md`, section 8): Britain's influence
/// over the others and the others' over Britain, the military blocs of 1925 with the customs
/// hatching, and the subjects. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn influence_and_bloc_lenses() {
    use bf_client::map::lens::Lens;
    let user = std::env::temp_dir().join(format!("bf-shots-lenses-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session_of("GBR", laplace_schelling::calendar::Date::new(1925, 3, 1));
    let save = write_campaign(&user, &session, "Lentes 1925", "GBR", &[]);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        let set_lens = |h: &mut Harness<'_, App>, lens: Lens| {
            let app = h.state_mut();
            if let (Some(view), Some(world)) = (&mut app.shared.map_view, &app.shared.world) {
                bf_client::screens::lens_panel::choose(view, world, lens, 1925);
            }
            // The simulation thread reads the lens's country, then the map recolours.
            for _ in 0..40 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        let look = |h: &mut Harness<'_, App>, lon: f64, lat: f64, zoom: f32| {
            let app = h.state_mut();
            let meta = app.shared.world.as_ref().unwrap().map.meta.clone();
            let (x, y) = bf_client::world::project(&meta, lon, lat).unwrap();
            let view = app.shared.map_view.as_mut().unwrap();
            view.camera
                .as_mut()
                .unwrap()
                .look_at(egui::pos2(x, y), zoom);
            h.run_steps(4);
        };
        set_lens(
            &mut h,
            Lens::Influence {
                issue: 0,
                on: false,
            },
        );
        h.key_press(egui::Key::V);
        h.run_steps(6);
        shot(&mut h, &format!("150-lens-influence-over-{tag}"));
        set_lens(&mut h, Lens::Influence { issue: 0, on: true });
        shot(&mut h, &format!("151-lens-influence-on-{tag}"));
        set_lens(&mut h, Lens::Blocs(0));
        look(&mut h, 18.0, 47.0, size.x / 1600.0 * 2.2);
        shot(&mut h, &format!("152-lens-blocs-military-{tag}"));
        set_lens(&mut h, Lens::Blocs(2));
        look(&mut h, 20.0, 15.0, size.x / 1600.0 * 0.9);
        shot(&mut h, &format!("153-lens-blocs-subjects-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Orders of forces (phase 5, slice A2): Britain two months after ordering a battleship, four
/// destroyers and six thousand infantry: the queue with what each delivered, a battleship picked
/// with what it costs to build and every year after, the large order's second click and a
/// cancellation asked. Portuguese at 1600 × 900, English at 1280 × 720; copies in
/// `target/screenshots/war/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn force_orders() {
    use bf_sim::game::GameCommand;
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces") };
    let user = std::env::temp_dir().join(format!("bf-shots-orders-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let forces = bf_sim::game::Switches::parse("forces").expect("switches");
    let mut session = advanced_session_with("GBR", Date::new(1925, 1, 2), forces);
    for (unit, elements) in [("battleship", 1), ("destroyer", 4), ("infantry", 6)] {
        session.submit(GameCommand::OrderForces {
            tag: "GBR".into(),
            unit: unit.into(),
            elements,
            colonial: false,
        });
    }
    while session.clock().date < Date::new(1925, 3, 2) {
        session.sim.step();
    }
    let save = write_campaign(&user, &session, "Encomendas", "GBR", &[]);
    let war_dir = root().join("target/screenshots/war");
    std::fs::create_dir_all(&war_dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, war_dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let mut h = harness_sized(user.join(locale), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(if en { "Forces" } else { "Forças" }).click();
        h.run_steps(4);
        let title = if en { "Orders" } else { "Encomendas" };
        h.get_by_label(title).scroll_to_me();
        h.run_steps(4);
        let name = format!("140-orders-queue-{tag}");
        shot(&mut h, &name);
        keep(&name);
        let ship = || {
            egui_kittest::kittest::by()
                .role(egui::accesskit::Role::Button)
                .label(if en { "Battleships" } else { "Encouraçados" })
        };
        h.query_all(ship())
            .next()
            .expect("the battleship button")
            .scroll_to_me();
        h.run_steps(4);
        h.query_all(ship()).next().unwrap().click();
        h.run_steps(4);
        h.get_by_label(if en { "Order" } else { "Encomendar" })
            .scroll_to_me();
        h.run_steps(4);
        let name = format!("141-orders-battleship-{tag}");
        shot(&mut h, &name);
        keep(&name);
        h.get_by_label(if en { "Order" } else { "Encomendar" })
            .click();
        h.run_steps(4);
        h.get_by_label(if en {
            "Place the order"
        } else {
            "Fazer a encomenda"
        })
        .scroll_to_me();
        h.run_steps(4);
        let name = format!("142-orders-confirm-{tag}");
        shot(&mut h, &name);
        keep(&name);
        let cancel = || egui_kittest::kittest::by().label(if en { "Cancel" } else { "Cancelar" });
        h.get_all(cancel()).next().expect("cancel").scroll_to_me();
        h.run_steps(4);
        h.get_all(cancel()).next().unwrap().click();
        h.run_steps(4);
        h.get_by_label(if en {
            "Cancel the order"
        } else {
            "Cancelar a encomenda"
        })
        .scroll_to_me();
        h.run_steps(4);
        let name = format!("143-orders-cancel-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Disbanding forces (phase 5, slice A2): Britain five months after ordering six thousand
/// infantry. The forces tab with a "Disband…" on each row; the first land type asked, with what
/// goes back to the depot and where the men go; the last type of the table (aircraft, which
/// leave service for good) asked; and the table after its confirming click and a day. Portuguese at
/// 1600 × 900, English at 1280 × 720; copies in `target/screenshots/disband/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn disband_forces() {
    use bf_sim::game::GameCommand;
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces") };
    let user = std::env::temp_dir().join(format!("bf-shots-disband-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let forces = bf_sim::game::Switches::parse("forces").expect("switches");
    let mut session = advanced_session_with("GBR", Date::new(1925, 1, 2), forces);
    session.submit(GameCommand::OrderForces {
        tag: "GBR".into(),
        unit: "infantry".into(),
        elements: 6,
        colonial: false,
    });
    while session.clock().date < Date::new(1925, 6, 1) {
        session.sim.step();
    }
    let save = write_campaign(&user, &session, "Dissolver", "GBR", &[]);
    let dir = root().join("target/screenshots/disband");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let mut h = harness_sized(user.join(locale), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(if en { "Forces" } else { "Forças" }).click();
        h.run_steps(4);
        let ask = || {
            egui_kittest::kittest::by()
                .role(egui::accesskit::Role::Button)
                .label(if en { "Disband…" } else { "Dissolver…" })
        };
        let name = format!("150-disband-table-{tag}");
        shot(&mut h, &name);
        keep(&name);
        h.query_all(ask()).next().expect("a disband button").click();
        h.run_steps(4);
        let yes = || {
            egui_kittest::kittest::by()
                .role(egui::accesskit::Role::Button)
                .label(if en { "Disband" } else { "Dissolver" })
        };
        h.query_all(yes())
            .next()
            .expect("the confirming button")
            .scroll_to_me();
        h.run_steps(4);
        let name = format!("151-disband-land-{tag}");
        shot(&mut h, &name);
        keep(&name);
        h.query_all(ask())
            .last()
            .expect("a disband button")
            .scroll_to_me();
        h.run_steps(4);
        h.query_all(ask()).last().unwrap().click();
        h.run_steps(4);
        h.query_all(yes())
            .next()
            .expect("the confirming button")
            .scroll_to_me();
        h.run_steps(4);
        let name = format!("152-disband-service-{tag}");
        shot(&mut h, &name);
        keep(&name);
        // The last type confirmed: the next tick carries it out and the table reads it anew.
        h.query_all(yes()).next().unwrap().click();
        h.run_steps(2);
        let snap = |h: &Harness<'_, App>| match &h.state().screen {
            Screen::Game(g) => g.snapshot(),
            _ => panic!("not in a campaign"),
        };
        let before = snap(&h).forces.expect("forces").stock;
        h.key_press(egui::Key::Space);
        let started = std::time::Instant::now();
        while snap(&h).forces.expect("forces").stock == before && started.elapsed().as_secs() < 30 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            h.run_steps(1);
        }
        h.key_press(egui::Key::Space);
        let paused_at = std::time::Instant::now();
        while !snap(&h).paused && paused_at.elapsed().as_secs() < 10 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            h.run_steps(1);
        }
        h.run_steps(4);
        let after = snap(&h).forces.expect("forces");
        assert_ne!(after.stock, before, "the disband was carried out");
        assert!(
            after.stock.2 <= before.2 - laplace_schelling::Fixed::ONE,
            "one element less: {before:?} → {:?}",
            after.stock
        );
        h.query_all(ask()).last().unwrap().scroll_to_me();
        h.run_steps(4);
        let name = format!("153-disband-done-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The negotiation table (treaties, section 13): Britain drafting a proposal to France picked
/// on the map (a payment of its own, a tribute of France's), with each side's account and the
/// likely answer; then a Britain whose proposal France refused and to which France proposed
/// non-aggression for a payment: the Agendas with the proposal waiting and the refusal on the
/// record, the proposal received with the three answers, and the counter on the table. Portuguese at 1600 × 900, English at 1280 × 720; copies in
/// `target/screenshots/treaty-table/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn negotiation_table() {
    use bf_sim::game::GameCommand;
    use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
    use laplace_schelling::Fixed;
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "negotiation") };
    let user = std::env::temp_dir().join(format!("bf-shots-table-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::game::Switches::parse("negotiation").expect("switches");
    let drafting = advanced_session_with("GBR", Date::new(1925, 2, 1), switches);
    let mut countered = advanced_session_with("GBR", Date::new(1925, 2, 1), switches);
    let gdp = |s: &bf_sim::game::Session, tag: &str| {
        let w = s.world();
        let i = w.countries.iter().position(|(_, c)| c.tag == tag).unwrap();
        w.econ.markets[i].gdp_last_year
    };
    let clause = |kind, mine: bool| Clause {
        kind,
        giver: if mine { Side::Senior } else { Side::Junior },
        receiver: if mine { Side::Junior } else { Side::Senior },
        until: None,
        note: None,
    };
    let proposal = Proposal {
        side_a: vec!["GBR".into()],
        side_b: vec!["FRA".into()],
        clauses: vec![
            clause(
                ClauseKind::Payment {
                    amount: gdp(&countered, "GBR") * Fixed::from_ratio(1, 100),
                },
                true,
            ),
            clause(
                ClauseKind::Tribute {
                    yearly: gdp(&countered, "FRA") * Fixed::from_ratio(2, 100),
                },
                false,
            ),
        ],
        term_years: Some(10),
        notice_years: 2,
    };
    // France proposes the mirror of it to Britain: non-aggression for a payment.
    let from_france = Proposal {
        side_a: vec!["FRA".into()],
        side_b: vec!["GBR".into()],
        clauses: vec![
            clause(ClauseKind::NonAggression, true),
            clause(
                ClauseKind::Payment {
                    amount: gdp(&countered, "GBR") * Fixed::from_ratio(1, 200),
                },
                false,
            ),
        ],
        term_years: Some(20),
        notice_years: 2,
    };
    countered.submit(GameCommand::ProposeTreaty {
        tag: "GBR".into(),
        to: "FRA".into(),
        proposal,
    });
    countered.submit(GameCommand::ProposeTreaty {
        tag: "FRA".into(),
        to: "GBR".into(),
        proposal: from_france,
    });
    while countered.clock().date < Date::new(1925, 2, 12) {
        countered.sim.step();
    }
    let save_draft = write_campaign(&user.join("draft"), &drafting, "Mesa", "GBR", &[]);
    let save_counter = write_campaign(
        &user.join("counter"),
        &countered,
        "Contraproposta",
        "GBR",
        &[],
    );
    let dir = root().join("target/screenshots/treaty-table");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let wait = |h: &mut Harness<'_, App>| {
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        // Scrolled into view first, then clicked where it is now: a click at a stale place
        // would land on the map and change the country picked.
        let press = |h: &mut Harness<'_, App>, text: &str, nth: usize| {
            let query = || {
                egui_kittest::kittest::by()
                    .label(text)
                    .role(egui::accesskit::Role::Button)
            };
            h.query_all(query())
                .nth(nth)
                .unwrap_or_else(|| panic!("button {text}"))
                .scroll_to_me();
            h.run_steps(4);
            h.query_all(query()).nth(nth).unwrap().click();
        };
        let contains = |h: &mut Harness<'_, App>, text: &str| {
            let b = h
                .query_all(egui_kittest::kittest::by().label_contains(text))
                .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
                .unwrap_or_else(|| panic!("button with {text}"));
            b.click();
        };
        // Drafting: France picked on the map.
        let mut h = harness_sized(user.join(format!("draft-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_draft.clone()), &ctx);
        wait(&mut h);
        let started = std::time::Instant::now();
        while h.state().shared.world.is_none() && started.elapsed().as_secs() < 20 {
            wait(&mut h);
        }
        let france = h
            .state()
            .shared
            .world
            .as_ref()
            .and_then(|w| w.countries.iter().position(|c| c.tag() == "FRA"));
        assert!(france.is_some(), "the map is loaded");
        if let Screen::Game(g) = &mut h.state_mut().screen {
            g.select(france);
        }
        h.key_press(egui::Key::J);
        wait(&mut h);
        contains(
            &mut h,
            if en {
                "Propose an agreement"
            } else {
                "Propor acordo"
            },
        );
        wait(&mut h);
        let add = if en {
            "Add a clause…"
        } else {
            "Acrescentar cláusula…"
        };
        press(&mut h, add, 0);
        wait(&mut h);
        let name = format!("180-table-picker-{tag}");
        shot(&mut h, &name);
        keep(&name);
        press(&mut h, if en { "Payment" } else { "Pagamento" }, 0);
        wait(&mut h);
        press(&mut h, add, 1);
        wait(&mut h);
        press(&mut h, if en { "Tribute" } else { "Tributo" }, 0);
        wait(&mut h);
        wait(&mut h);
        let name = format!("181-table-draft-{tag}");
        shot(&mut h, &name);
        keep(&name);
        let go = if en { "Propose" } else { "Propor" };
        h.query_all(egui_kittest::kittest::by().label(go))
            .next()
            .expect("the propose button")
            .scroll_to_me();
        wait(&mut h);
        let name = format!("182-table-account-{tag}");
        shot(&mut h, &name);
        keep(&name);
        // Countered: the Agendas, the proposal received, the counter.
        let mut h = harness_sized(user.join(format!("counter-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_counter.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::J);
        wait(&mut h);
        let name = format!("183-table-agendas-{tag}");
        shot(&mut h, &name);
        keep(&name);
        contains(
            &mut h,
            if en {
                "proposes an agreement"
            } else {
                "propõe um acordo"
            },
        );
        wait(&mut h);
        let name = format!("184-table-received-{tag}");
        shot(&mut h, &name);
        keep(&name);
        press(&mut h, if en { "Counter…" } else { "Contrapropor…" }, 0);
        wait(&mut h);
        wait(&mut h);
        let name = format!("185-table-counter-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// "Renegotiate" in the debts tab (treaties, section 13): Britain's debt to the United States,
/// the button, and the negotiation table it opens in the Agendas with the revision of that debt
/// on it, from today's terms. Portuguese at 1600 × 900, English at 1280 × 720; copies in
/// `target/screenshots/renegotiate/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn renegotiate_debt() {
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "negotiation") };
    let user = std::env::temp_dir().join(format!("bf-shots-renegotiate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::game::Switches::parse("negotiation").expect("switches");
    let session = advanced_session_with("GBR", Date::new(1925, 3, 1), switches);
    let save = write_campaign(&user.join("save"), &session, "Renegociar", "GBR", &[]);
    let dir = root().join("target/screenshots/renegotiate");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let wait = |h: &mut Harness<'_, App>| {
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        // Scrolled into view first, then clicked where it is now.
        let press = |h: &mut Harness<'_, App>, text: &str| {
            let query = || {
                egui_kittest::kittest::by()
                    .label(text)
                    .role(egui::accesskit::Role::Button)
            };
            h.query_all(query())
                .next()
                .unwrap_or_else(|| panic!("button {text}"))
                .scroll_to_me();
            h.run_steps(4);
            h.query_all(query()).next().unwrap().click();
        };
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::G);
        wait(&mut h);
        h.get_all(egui_kittest::kittest::by().label(if en {
            "Debts between governments"
        } else {
            "Dívidas entre governos"
        }))
        .next()
        .expect("debts tab")
        .click();
        wait(&mut h);
        press(
            &mut h,
            if en {
                "United States"
            } else {
                "Estados Unidos"
            },
        );
        wait(&mut h);
        let renegotiate = if en { "Renegotiate" } else { "Renegociar" };
        h.query_all(egui_kittest::kittest::by().label(renegotiate))
            .next()
            .expect("the renegotiate button")
            .scroll_to_me();
        wait(&mut h);
        let name = format!("190-renegotiate-button-{tag}");
        shot(&mut h, &name);
        keep(&name);
        press(&mut h, renegotiate);
        wait(&mut h);
        wait(&mut h);
        let name = format!("191-renegotiate-table-{tag}");
        shot(&mut h, &name);
        keep(&name);
        h.query_all(egui_kittest::kittest::by().label(if en { "Propose" } else { "Propor" }))
            .next()
            .expect("the propose button")
            .scroll_to_me();
        wait(&mut h);
        let name = format!("192-renegotiate-account-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The war window (phase 5, slice C): Hungary at war with Czechoslovakia, attacking since the
/// declaration, on the week of its first state occupied: each side's war support with its
/// groups and its speed, the occupation with what it yields, the devastation; the front with the
/// posture, the forces, the week's losses and advance; the segments opened, with the advance
/// into each state, the width, the men per km and the entrenchment; and peace, which opens the
/// negotiation table with Czechoslovakia fixed. Portuguese at 1600 × 900, English at 1280 × 720; copies in
/// `target/screenshots/war-window/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn war_window() {
    use bf_sim::game::GameCommand;
    use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces,crisis,war,negotiation") };
    let switches =
        bf_sim::game::Switches::parse("forces,crisis,war,negotiation").expect("switches");
    let user = std::env::temp_dir().join(format!("bf-shots-war-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut s = advanced_session_with("HUN", Date::new(1925, 3, 2), switches);
    s.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
            clauses: vec![Clause {
                kind: ClauseKind::Union,
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
        weeks: None,
    });
    while s.clock().date < Date::new(1925, 3, 20) {
        s.sim.step();
    }
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.submit(GameCommand::SetPosture {
        tag: "HUN".into(),
        enemy: "CZE".into(),
        attack: true,
    });
    // The first week of combat with its support moved and the segments still: since #217 the
    // war ends on 31 March without a peace signed (reported to the war's thread), so the
    // occupation and the devastation show empty here.
    while s.clock().date < Date::new(1925, 3, 30) {
        s.sim.step();
    }
    assert!(
        !s.world().war.fighting.last_week.is_empty() && !s.world().war.fighting.support.is_empty(),
        "the war has a front and a support"
    );
    let save = write_campaign(&user.join("save"), &s, "Guerra", "HUN", &[]);
    let dir = root().join("target/screenshots/war-window");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let wait = |h: &mut Harness<'_, App>| {
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        let press = |h: &mut Harness<'_, App>, text: &str| {
            let query = || {
                egui_kittest::kittest::by()
                    .label_contains(text)
                    .role(egui::accesskit::Role::Button)
            };
            h.query_all(query())
                .next()
                .unwrap_or_else(|| panic!("button {text}"))
                .scroll_to_me();
            h.run_steps(4);
            h.query_all(query()).next().unwrap().click();
        };
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        wait(&mut h);
        // The Agendas list the war; its button opens the window.
        h.key_press(egui::Key::J);
        wait(&mut h);
        let name = format!("210-war-agendas-{tag}");
        shot(&mut h, &name);
        keep(&name);
        press(&mut h, if en { "Hungary" } else { "Hungria" });
        wait(&mut h);
        wait(&mut h);
        let name = format!("211-war-window-{tag}");
        shot(&mut h, &name);
        keep(&name);
        press(&mut h, if en { "Show " } else { "Ver " });
        wait(&mut h);
        let name = format!("212-war-segments-{tag}");
        shot(&mut h, &name);
        keep(&name);
        press(&mut h, if en { "Propose peace" } else { "Propor paz" });
        wait(&mut h);
        wait(&mut h);
        let name = format!("213-war-peace-table-{tag}");
        shot(&mut h, &name);
        keep(&name);
        // The war lens over Central Europe: Hungary and Czechoslovakia in their colours, the
        // states each holds of the other hatched in the occupier's, the rest faded.
        let mut h = harness_sized(user.join(format!("lens-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        wait(&mut h);
        {
            let app = h.state_mut();
            if let (Some(view), Some(world)) = (&mut app.shared.map_view, &app.shared.world) {
                bf_client::screens::lens_panel::choose(
                    view,
                    world,
                    bf_client::map::lens::Lens::War,
                    1925,
                );
            }
        }
        wait(&mut h);
        h.key_press(egui::Key::V);
        wait(&mut h);
        {
            let app = h.state_mut();
            let meta = app.shared.world.as_ref().unwrap().map.meta.clone();
            let (x, y) = bf_client::world::project(&meta, 19.0, 48.5).unwrap();
            let view = app.shared.map_view.as_mut().unwrap();
            view.camera
                .as_mut()
                .unwrap()
                .look_at(egui::pos2(x, y), size.x / 1600.0 * 3.0);
        }
        wait(&mut h);
        let name = format!("214-war-lens-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Trade by partner in the country panel: Britain in February 1926 (the first full year
/// closed), with its largest partners and the agreements' terms on the tariff between them, and
/// Belgium, in a customs union with Luxembourg. Portuguese at 1600 × 900, English at
/// 1280 × 720; copies in `target/screenshots/trade/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn trade_partners() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-trade-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    // The rules as the game ships them (`data/rules.ron`: trade by pair, agreements in force).
    let shipped = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    let session = advanced_session_with("GBR", Date::new(1926, 2, 1), shipped);
    let save = write_campaign(&user.join("save"), &session, "Comércio", "GBR", &[]);
    let dir = root().join("target/screenshots/trade");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let wait = |h: &mut Harness<'_, App>| {
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        wait(&mut h);
        let started = std::time::Instant::now();
        while h.state().shared.world.is_none() && started.elapsed().as_secs() < 20 {
            wait(&mut h);
        }
        for (country, n) in [("GBR", 200), ("BEL", 201)] {
            let i = h
                .state()
                .shared
                .world
                .as_ref()
                .and_then(|w| w.countries.iter().position(|c| c.tag() == country));
            assert!(i.is_some(), "{country} on the map");
            if let Screen::Game(g) = &mut h.state_mut().screen {
                g.select(i);
            }
            wait(&mut h);
            // The top of the panel: the headline numbers on one line each.
            if country == "GBR" {
                let name = format!("202-country-top-{tag}");
                shot(&mut h, &name);
                keep(&name);
            }
            h.query_all(egui_kittest::kittest::by().label(if en {
                "LARGEST PARTNERS"
            } else {
                "PRINCIPAIS PARCEIROS"
            }))
            .next()
            .expect("the partners")
            .scroll_to_me();
            wait(&mut h);
            let name = format!("{n}-trade-{}-{tag}", country.to_lowercase());
            shot(&mut h, &name);
            keep(&name);
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Activates the element `id` by its key tip, scrolling the open window down until its tip
/// shows (`late_screens_layout::act`); whether it was found.
fn act_by_tip(h: &mut Harness<'_, App>, id: &str) -> bool {
    let steps = |h: &mut Harness<'_, App>| h.run_steps(4);
    if !h.state().shared.keys.hint_mode() {
        h.key_press(egui::Key::AltLeft);
        h.run_steps(3);
    }
    let mut scrolled = 0;
    let code = loop {
        if let Some(code) = h.state().shared.keys.code_of(id).map(str::to_owned) {
            break code;
        }
        if scrolled == 8 {
            h.key_press(egui::Key::Escape);
            h.run_steps(2);
            return false;
        }
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
        steps(h);
        h.key_press(egui::Key::AltLeft);
        h.run_steps(3);
        scrolled += 1;
    };
    for c in code.chars() {
        h.key_press(egui::Key::from_name(&c.to_string()).expect("a key for the code"));
        steps(h);
    }
    steps(h);
    if h.state().shared.keys.hint_mode() {
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
    }
    true
}

/// The government's trade tab (tariff by good and partner, owner's decisions CT1 and CT2):
/// Britain in February 1926, as the game ships (the tariff that counts off, said in one line),
/// with steel fixed at 30 %, then a category open. Portuguese and English at 1280 × 720; copies
/// in `target/screenshots/tariffs/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn trade_tab() {
    use bf_sim::game::GameCommand;
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-tariffs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let data_switches = || data.switches;
    let steel = data.catalogue.good("steel").expect("steel");
    let mut s = advanced_session_with("GBR", Date::new(1926, 1, 25), data_switches());
    s.submit(GameCommand::SetGoodsTariff {
        tag: "GBR".into(),
        goods: vec![steel],
        per_mille: Some(300),
    });
    s.sim.run_until(Date::new(1926, 2, 1));
    let save = write_campaign(&user.join("save"), &s, "Tarifas", "GBR", &[]);
    let dir = root().join("target/screenshots/tariffs");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1280.0, 720.0), "pt-1280"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
        ("pt-BR", egui::vec2(960.0, 540.0), "pt-960"),
        ("en", egui::vec2(960.0, 540.0), "en-960"),
    ] {
        let en = locale == "en";
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::T);
        h.run_steps(8);
        let name = format!("210-trade-tab-{tag}");
        shot(&mut h, &name);
        keep(&name);
        // The industrial inputs open: steel fixed at 30 %, the group's controls.
        let category = if en {
            "Industrial goods (13)"
        } else {
            "Insumos industriais (13)"
        };
        h.get_by_label(category).click();
        h.run_steps(8);
        h.get_by_label(category).scroll_to_me();
        h.run_steps(4);
        let name = format!("211-trade-tab-category-{tag}");
        shot(&mut h, &name);
        keep(&name);
        // Below: the partners, their agreements, the points and the search.
        h.get_by_label(if en { "BY PARTNER" } else { "POR PARCEIRO" })
            .scroll_to_me();
        h.run_steps(4);
        let name = format!("212-trade-tab-partners-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    // Belgium, in a customs union with Luxembourg: a surcharge on Luxembourg asks first, the
    // breach of the union and the general reputation in words, a restore point on confirming.
    let bel = advanced_session_with("BEL", Date::new(1926, 2, 1), data_switches());
    let save = write_campaign(&user.join("save-bel"), &bel, "Tarifas", "BEL", &[]);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1280.0, 720.0), "pt-1280"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let mut h = harness_sized(user.join(format!("bel-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::T);
        h.run_steps(8);
        assert!(
            act_by_tip(&mut h, "gov.tariff.partner.LUX.up"),
            "Luxembourg's surcharge"
        );
        h.run_steps(8);
        let name = format!("213-trade-tab-breach-{tag}");
        shot(&mut h, &name);
        keep(&name);
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The country's agreements (treaties, sections 9 and 13): Britain in March 1925, after
/// denouncing without notice one of its agreements of 1925 (a breach on the record, the
/// agreement ended, its reputation weighed): the agreements in force with their clauses, the
/// breach, the reputations, and the two steps of denouncing another without notice. Portuguese
/// at 1600 × 900, English at 1280 × 720; copies in `target/screenshots/agreements/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn agreements_screen() {
    use bf_sim::game::GameCommand;
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "negotiation,reputation") };
    let user = std::env::temp_dir().join(format!("bf-shots-agreements-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    switches = switches.with("negotiation,reputation").expect("switches");
    let mut s = advanced_session_with("GBR", Date::new(1925, 2, 1), switches);
    let party =
        |a: &bf_sim::treaty::Agreement| a.side_a.iter().chain(&a.side_b).any(|t| t == "GBR");
    let id = s
        .world()
        .treaties
        .agreements
        .iter()
        .find(|a| party(a) && a.notice_years == 0 && !a.kept.is_empty())
        .map(|a| a.id)
        .expect("an agreement of Britain's without notice");
    s.submit(GameCommand::Denounce {
        tag: "GBR".into(),
        agreement: id,
    });
    while s.clock().date < Date::new(1925, 3, 2) {
        s.sim.step();
    }
    assert!(
        !s.world().treaties.breaches.is_empty(),
        "the denunciation without notice is on the record"
    );
    let save = write_campaign(&user.join("save"), &s, "Acordos", "GBR", &[]);
    let dir = root().join("target/screenshots/agreements");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |name: &str| {
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let wait = |h: &mut Harness<'_, App>| {
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        let press = |h: &mut Harness<'_, App>, text: &str| {
            let query = || {
                egui_kittest::kittest::by()
                    .label_contains(text)
                    .role(egui::accesskit::Role::Button)
            };
            h.query_all(query())
                .next()
                .unwrap_or_else(|| panic!("button {text}"))
                .scroll_to_me();
            h.run_steps(4);
            h.query_all(query()).next().unwrap().click();
        };
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::J);
        wait(&mut h);
        press(
            &mut h,
            if en {
                "The country's agreements"
            } else {
                "Acordos do país"
            },
        );
        wait(&mut h);
        let name = format!("220-agreements-{tag}");
        shot(&mut h, &name);
        keep(&name);
        h.query_all(egui_kittest::kittest::by().label(if en { "Breaches" } else { "Quebras" }))
            .next()
            .expect("the breaches")
            .scroll_to_me();
        wait(&mut h);
        let name = format!("221-agreements-breaches-{tag}");
        shot(&mut h, &name);
        keep(&name);
        press(&mut h, if en { "Denounce…" } else { "Denunciar…" });
        wait(&mut h);
        let name = format!("222-agreements-denounce-{tag}");
        shot(&mut h, &name);
        keep(&name);
        let next = if en {
            "I want to break the agreement"
        } else {
            "Quero quebrar o acordo"
        };
        if h.query_all(egui_kittest::kittest::by().label_contains(next))
            .next()
            .is_some()
        {
            press(&mut h, next);
            wait(&mut h);
            let name = format!("223-agreements-break-{tag}");
            shot(&mut h, &name);
            keep(&name);
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The path from a demand to the peace, from the keyboard only (key tips: the leader key, then
/// the code each element shows): Hungary demands Czechoslovakia whole (the target found by name,
/// the union ticked, reviewed, sent); after the refusal it declares; in the war window it sets
/// the front to attack; then it opens the peace table, puts a payment on its side and proposes.
/// Portuguese at 1600 × 900; copies in `target/screenshots/keyboard-war/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn crisis_to_peace_by_keyboard() {
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces,crisis,war,negotiation") };
    let switches =
        bf_sim::game::Switches::parse("forces,crisis,war,negotiation").expect("switches");
    let user = std::env::temp_dir().join(format!("bf-shots-kbwar-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = advanced_session_with("HUN", Date::new(1925, 3, 2), switches);
    let save = write_campaign(&user.join("save"), &s, "Teclado", "HUN", &[]);
    let dir = root().join("target/screenshots/keyboard-war");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |h: &mut Harness<'_, App>, name: &str| {
        shot(h, name);
        let from = out_dir().join(format!("{name}.png"));
        std::fs::copy(&from, dir.join(format!("{name}.png"))).unwrap();
    };
    let wait = |h: &mut Harness<'_, App>| {
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            h.run_steps(1);
        }
    };
    let snap = |h: &Harness<'_, App>| match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    };
    // Types the code of element `id`: the leader key first when the tips are off.
    let act = |h: &mut Harness<'_, App>, id: &str| {
        if !h.state().shared.keys.hint_mode() {
            leader(h);
        }
        h.run_steps(2);
        let code = h
            .state()
            .shared
            .keys
            .code_of(id)
            .unwrap_or_else(|| panic!("no key tip for {id}"))
            .to_owned();
        for c in code.chars() {
            let key = egui::Key::from_name(&c.to_string()).expect("a key for the code");
            tip(h, key);
        }
        wait(h);
    };
    // Runs the clock until `done`, then pauses.
    let run = |h: &mut Harness<'_, App>, done: &dyn Fn(&bf_client::sim::Snapshot) -> bool| {
        h.key_press(egui::Key::Space);
        let started = std::time::Instant::now();
        while !done(&snap(h)) && started.elapsed().as_secs() < 60 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            h.run_steps(1);
        }
        h.key_press(egui::Key::Space);
        let paused = std::time::Instant::now();
        while !snap(h).paused && paused.elapsed().as_secs() < 10 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            h.run_steps(1);
        }
        wait(h);
        assert!(done(&snap(h)), "the clock ran to the step");
    };
    let mut h = harness_sized(user.join("pt"), "pt-BR", egui::vec2(1600.0, 900.0));
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save.clone()), &ctx);
    wait(&mut h);
    // 1. The demand: J, the decision, the target by name, the union, review, send.
    h.key_press(egui::Key::J);
    wait(&mut h);
    act(&mut h, "journal.decision.exigency");
    act(&mut h, "crisis.demand.find");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    h.event(egui::Event::Text("Tchec".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    wait(&mut h);
    keep(&mut h, "230-keyboard-demand-target");
    act(&mut h, "crisis.demand.union");
    act(&mut h, "crisis.demand.review");
    keep(&mut h, "231-keyboard-demand-review");
    act(&mut h, "crisis.demand.send");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    // 2. The refusal, then the declaration.
    run(&mut h, &|s| {
        s.crises.as_ref().is_some_and(|c| {
            c.crises
                .iter()
                .any(|k| k.stage == bf_sim::war::Stage::Refused)
        })
    });
    let id = snap(&h).crises.unwrap().crises[0].id;
    // The Agendas stayed open on the demand: back to the list, then the crisis.
    act(&mut h, "journal.decision.back");
    act(&mut h, &format!("journal.crisis.{id}"));
    keep(&mut h, "232-keyboard-crisis-refused");
    act(&mut h, "crisis.yes");
    act(&mut h, "crisis.confirm");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    run(&mut h, &|s| {
        s.war.as_ref().is_some_and(|w| !w.wars.is_empty())
    });
    // 3. The war window: the front to attack.
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::W);
    wait(&mut h);
    act(&mut h, "war.posture.CZE");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    run(&mut h, &|s| {
        s.war
            .as_ref()
            .is_some_and(|w| w.wars.iter().any(|x| x.fronts.iter().any(|f| f.attack)))
    });
    keep(&mut h, "233-keyboard-war-attack");
    // 4. Peace: the table with Czechoslovakia, a payment of ours, propose.
    act(&mut h, "war.peace");
    act(&mut h, "treaty.add.true");
    act(&mut h, "treaty.pick.payment");
    wait(&mut h);
    keep(&mut h, "234-keyboard-peace-table");
    // The button is below the fold: the page scrolls by key, then its tip shows.
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    for _ in 0..3 {
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
        h.run_steps(3);
    }
    act(&mut h, "treaty.propose");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    wait(&mut h);
    run(&mut h, &|s| {
        s.treaty_table
            .as_ref()
            .is_some_and(|t| !t.outgoing.is_empty() || !t.closed.is_empty())
    });
    let _ = std::fs::remove_dir_all(&user);
}

/// Answering a demand of independence: Nigeria's demand pending on the United Kingdom, from the
/// red button in the Agendas to the screen with the four answers, the refusal chosen and the
/// review before confirming; in Portuguese at 1600 × 900 and in English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn demand_screen() {
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "independence") };
    use bf_sim::borders::independence::{Account, Answer, Demand, units_of};
    use laplace_schelling::Fixed;
    use laplace_schelling::calendar::Date;
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let setup = rules.independence.clone().expect("independence rules");
    let mut world = data.new_world();
    let ix = |w: &bf_sim::game::GameWorld, tag: &str| {
        w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
    };
    let mut units = units_of(&setup, &world, &[]);
    let top: Vec<(u16, Fixed)> = ["USA", "SOV", "YUG", "TUR", "ITA"]
        .iter()
        .map(|t| (ix(&world, t), Fixed::from_ratio(3, 1000)))
        .collect();
    let u = units.iter_mut().find(|u| u.key == "NGA").expect("NGA");
    u.movement.support = 1000;
    u.mobilized = 640;
    u.demand = Some(Demand {
        since: Date::new(1925, 1, 2),
        answer: Answer::Pending,
        account: Account {
            value: Fixed::from_ratio(12, 1000),
            legitimacy: Fixed::from_ratio(4, 1000),
            international: Fixed::from_ratio(9, 1000),
            material: Fixed::from_ratio(2, 1000),
            top,
        },
    });
    world.independence.units = units;
    let mut session = bf_sim::game::Session::new(world, 7, Date::new(1925, 1, 2), 1, Some(rules));
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "GBR".into() });
    let user = std::env::temp_dir().join(format!("bf-shots-demand-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    for (locale, size, name) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let home = user.join(name);
        let save = write_campaign(&home, &session, "Exigência", "GBR", &[]);
        let mut h = harness_sized(home.clone(), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::J);
        h.run_steps(3);
        shot(&mut h, &format!("130-demand-agendas-{name}"));
        let title = if locale == "en" {
            "Nigeria"
        } else {
            "Nigéria"
        };
        h.query_all(egui_kittest::kittest::by().label_contains(title))
            .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
            .expect("the demand in the Agendas")
            .click();
        h.run_steps(4);
        shot(&mut h, &format!("131-demand-screen-{name}"));
        let refuse = if locale == "en" { "Refuse" } else { "Recusar" };
        h.get_all(egui_kittest::kittest::by().label_contains(refuse))
            .next()
            .expect("refuse")
            .click();
        h.run_steps(4);
        shot(&mut h, &format!("132-demand-refuse-{name}"));
        let review = if locale == "en" { "Review" } else { "Revisar" };
        let clicked = match h
            .query_all(egui_kittest::kittest::by().label_contains(review))
            .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        {
            Some(b) => {
                b.click();
                true
            }
            None => false,
        };
        if clicked {
            h.run_steps(4);
            shot(&mut h, &format!("133-demand-review-{name}"));
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The budget levers (D1, D2, D4, D5) in the budget tab: Britain in 1925 with the payroll and
/// the deficit cap fixed, public pay and the treasury on the historical rule. Portuguese at
/// 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn budget_levers() {
    use bf_sim::game::GameCommand;
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-levers-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut session = advanced_session_of("GBR", Date::new(1925, 2, 1));
    session.submit(GameCommand::SetPayroll {
        tag: "GBR".into(),
        per_mille: Some([10, 5]),
    });
    session.submit(GameCommand::SetStatePurchases {
        tag: "GBR".into(),
        goods: Some(900),
        military: Some(500),
    });
    session.submit(GameCommand::SetDeficitCap {
        tag: "GBR".into(),
        per_mille_gdp: Some(30),
    });
    while session.clock().date < Date::new(1925, 3, 2) {
        session.sim.step();
    }
    let save = write_campaign(&user, &session, "Alavancas", "GBR", &[]);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(if en { "Budget" } else { "Orçamento" })
            .click();
        h.run_steps(4);
        let heading = if en {
            "Payroll and treasury"
        } else {
            "Folha e Tesouro"
        };
        h.get_by_label(heading).scroll_to_me();
        h.run_steps(4);
        shot(&mut h, &format!("160-levers-{tag}"));
        // The treasury fixed: its reserve and the use of the excess show.
        let treasury = if en {
            "Treasury reserve"
        } else {
            "Reserva do Tesouro"
        };
        h.get_by_label(treasury).scroll_to_me();
        h.run_steps(4);
        let fixed = if en { "Fixed" } else { "Fixado" };
        if let Some(b) = h.query_all(egui_kittest::kittest::by().label(fixed)).nth(2) {
            b.click();
        }
        h.run_steps(4);
        shot(&mut h, &format!("161-levers-treasury-{tag}"));
        // The purchases (D3): the goods budget and its military share, fixed.
        let military = if en {
            "Military share of purchases"
        } else {
            "Parte militar das compras"
        };
        h.get_by_label(military).scroll_to_me();
        h.run_steps(4);
        shot(&mut h, &format!("162-levers-purchases-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Crisis and declaration (phase 5, slice B): Greece answering Italy's demand of the Aegean
/// islands and a payment, the refusal's second step; Italy drafting a demand of Albania with
/// how Albania would weigh it; and Italy, refused by Greece, choosing between war and backing
/// down. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn crisis_screens() {
    use bf_sim::game::GameCommand;
    use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces,crisis") };
    let switches = bf_sim::game::Switches::parse("forces,crisis").expect("switches");
    let user = std::env::temp_dir().join(format!("bf-shots-crisis-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let give = |kind| Clause {
        kind,
        giver: Side::Junior,
        receiver: Side::Senior,
        until: None,
        note: None,
    };
    let demand = |kinds: Vec<ClauseKind>| Proposal {
        side_a: vec!["ITA".into()],
        side_b: vec!["GRE".into()],
        clauses: kinds.into_iter().map(give).collect(),
        term_years: None,
        notice_years: 0,
    };
    // Greece, target of Italy's demand.
    let mut greece = advanced_session_with("GRE", Date::new(1925, 2, 1), switches);
    greece.submit(GameCommand::Demand {
        tag: "ITA".into(),
        target: "GRE".into(),
        demand: demand(vec![ClauseKind::CedeStates {
            states: vec!["GRC-AIGAIO".into()],
        }]),
        weeks: None,
    });
    while greece.clock().date < Date::new(1925, 2, 3) {
        greece.sim.step();
    }
    // Italy, refused by Greece over an annexation.
    let mut italy = advanced_session_with("ITA", Date::new(1925, 2, 1), switches);
    italy.submit(GameCommand::Demand {
        tag: "ITA".into(),
        target: "GRE".into(),
        demand: demand(vec![ClauseKind::Union]),
        weeks: None,
    });
    italy.submit(GameCommand::AnswerCrisis {
        tag: "GRE".into(),
        crisis: 0,
        accept: false,
    });
    while italy.clock().date < Date::new(1925, 2, 20) {
        italy.sim.step();
    }
    let save_greece = write_campaign(&user.join("grc"), &greece, "Crise grega", "GRE", &[]);
    let save_italy = write_campaign(&user.join("ita"), &italy, "Crise italiana", "ITA", &[]);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let wait = |h: &mut Harness<'_, App>| {
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        let button = |h: &mut Harness<'_, App>, text: &str| {
            let found = h
                .query_all(egui_kittest::kittest::by().label_contains(text))
                .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button);
            match found {
                Some(b) => {
                    b.click();
                    true
                }
                None => false,
            }
        };
        // Greece: the crisis in the Agendas, then its screen and the refusal's second step.
        let mut h = harness_sized(user.join(format!("grc-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_greece.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::J);
        wait(&mut h);
        shot(&mut h, &format!("170-crisis-agendas-{tag}"));
        button(&mut h, if en { "Italy demands" } else { "Itália exige" });
        wait(&mut h);
        shot(&mut h, &format!("171-crisis-target-{tag}"));
        if button(&mut h, if en { "Refuse" } else { "Recusar" }) {
            wait(&mut h);
            shot(&mut h, &format!("172-crisis-refuse-confirm-{tag}"));
        }
        // Italy: drafting a demand of Albania, picked on the map.
        let mut h = harness_sized(user.join(format!("ita-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_italy.clone()), &ctx);
        wait(&mut h);
        let albania = h
            .state()
            .shared
            .world
            .as_ref()
            .and_then(|w| w.countries.iter().position(|c| c.tag() == "ALB"));
        if let Screen::Game(g) = &mut h.state_mut().screen {
            g.select(albania);
        }
        h.key_press(egui::Key::J);
        wait(&mut h);
        button(&mut h, if en { "Demand of" } else { "Exigir d" });
        wait(&mut h);
        if let Some(b) = h
            .query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::CheckBox))
            .nth(1)
        {
            b.click();
        }
        wait(&mut h);
        shot(&mut h, &format!("173-crisis-demand-draft-{tag}"));
        // Italy, refused: war or backing down.
        // The back button of the decision, not the overview's link to the Agendas.
        button(&mut h, "←");
        wait(&mut h);
        if button(&mut h, if en { "Italy demands" } else { "Itália exige" }) {
            wait(&mut h);
            shot(&mut h, &format!("174-crisis-declare-{tag}"));
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The public works' queue (D8): Britain in 1925 with its public works, one moved to the top of
/// the government's order and a new school asked for; the region list and its buildings.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn works_tab() {
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-works-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let session = advanced_session_of("GBR", Date::new(1925, 3, 1));
    let save = write_campaign(&user, &session, "Obras", "GBR", &[]);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(if en { "Works" } else { "Obras" }).click();
        h.run_steps(4);
        shot(&mut h, &format!("180-works-{tag}"));
        // The third work moved up: the order becomes the government's.
        if let Some(b) = h.query_all(egui_kittest::kittest::by().label("↑")).nth(1) {
            b.click();
        }
        h.run_steps(4);
        shot(&mut h, &format!("181-works-moved-{tag}"));
        // A new work: the first region, its buildings and costs.
        let new = if en { "New work" } else { "Nova obra" };
        h.get_by_label(new).scroll_to_me();
        h.run_steps(4);
        // London's buildings and their costs.
        let london = if en { "London" } else { "Londres" };
        let found = match h
            .query_all(egui_kittest::kittest::by().label(london))
            .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        {
            Some(b) => {
                b.click();
                true
            }
            None => false,
        };
        if found {
            h.run_steps(4);
        }
        if let Some(n) = h
            .query_all(egui_kittest::kittest::by().label_contains(" · "))
            .rfind(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        {
            n.scroll_to_me();
        }
        h.run_steps(4);
        shot(&mut h, &format!("182-works-new-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Third parties in a crisis (W4): Hungary, refused by Czechoslovakia over Subcarpathian Ruthenia, sees who
/// would join before declaring (the Little Entente and France's alliance of 1924); with the war
/// declared, France, called by its alliance, answers. Portuguese at 1600 × 900, English at
/// 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn third_parties() {
    use bf_sim::game::GameCommand;
    use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
    use laplace_schelling::calendar::Date;
    let switches =
        bf_sim::game::Switches::parse("forces,crisis,treaties_in_force").expect("switches");
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces,crisis,treaties_in_force") };
    let user = std::env::temp_dir().join(format!("bf-shots-third-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let demand = Proposal {
        side_a: vec!["HUN".into()],
        side_b: vec!["CZE".into()],
        clauses: vec![Clause {
            kind: ClauseKind::CedeStates {
                states: vec!["CZE-PODKARPATSKA-RUS".into()],
            },
            giver: Side::Junior,
            receiver: Side::Senior,
            until: None,
            note: None,
        }],
        term_years: None,
        notice_years: 0,
    };
    let refused = |tag: &str| {
        let mut s = advanced_session_with(tag, Date::new(1925, 2, 1), switches);
        s.submit(GameCommand::Demand {
            tag: "HUN".into(),
            target: "CZE".into(),
            demand: demand.clone(),
            weeks: None,
        });
        while s.clock().date < Date::new(1925, 2, 12) {
            s.sim.step();
        }
        s
    };
    let hungary = refused("HUN");
    // France as the player: Hungary is the AI then, and an AI refused backs down in the week; the
    // demand, the refusal and the declaration go in the same day.
    let mut france = advanced_session_with("FRA", Date::new(1925, 2, 1), switches);
    let crisis = france.world().war.conflicts.next_id;
    france.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: demand.clone(),
        weeks: None,
    });
    france.submit(GameCommand::AnswerCrisis {
        tag: "CZE".into(),
        crisis,
        accept: false,
    });
    france.submit(GameCommand::Declare {
        tag: "HUN".into(),
        crisis,
        war: true,
    });
    while france.clock().date < Date::new(1925, 2, 4) {
        france.sim.step();
    }
    let save_hun = write_campaign(&user.join("hun"), &hungary, "Hungria", "HUN", &[]);
    let save_fra = write_campaign(&user.join("fra"), &france, "França", "FRA", &[]);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let wait = |h: &mut Harness<'_, App>| {
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                h.run_steps(1);
            }
        };
        let button = |h: &mut Harness<'_, App>, text: &str| {
            let found = h
                .query_all(egui_kittest::kittest::by().label_contains(text))
                .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button);
            match found {
                Some(b) => {
                    b.click();
                    true
                }
                None => false,
            }
        };
        let mut h = harness_sized(user.join(format!("hun-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_hun.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::J);
        wait(&mut h);
        if button(
            &mut h,
            if en {
                "Hungary demands"
            } else {
                "Hungria exige"
            },
        ) {
            wait(&mut h);
            shot(&mut h, &format!("190-third-who-would-join-{tag}"));
        }
        let mut h = harness_sized(user.join(format!("fra-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_fra.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::J);
        wait(&mut h);
        shot(&mut h, &format!("191-third-call-agendas-{tag}"));
        if button(&mut h, if en { "calls:" } else { "chama:" }) {
            wait(&mut h);
            shot(&mut h, &format!("192-third-call-{tag}"));
            if button(&mut h, if en { "Do not honour" } else { "Não honrar" }) {
                wait(&mut h);
                shot(&mut h, &format!("193-third-call-refuse-{tag}"));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// Where the treasury issues (D9): Brazil in 1925 with its new bonds split between home, abroad
/// in its own currency and abroad in dollars, a month in: what each market bought and left
/// unsold, and the warning when lenders abroad take less in milréis than asked. Portuguese at
/// 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn issuance() {
    use bf_sim::game::GameCommand;
    use laplace_schelling::calendar::Date;
    let user = std::env::temp_dir().join(format!("bf-shots-issue-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut session = advanced_session_of("BRA", Date::new(1925, 2, 1));
    session.submit(GameCommand::SetIssuance {
        tag: "BRA".into(),
        shares: Some([300, 500, 200]),
    });
    while session.clock().date < Date::new(1925, 3, 2) {
        session.sim.step();
    }
    let save = write_campaign(&user, &session, "Emissão", "BRA", &[]);
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let en = locale == "en";
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.get_by_label(if en {
            "Money and debt"
        } else {
            "Moeda e dívida"
        })
        .click();
        h.run_steps(4);
        h.get_by_label(if en {
            "Sold abroad, in dollars"
        } else {
            "Vendido fora, em dólar"
        })
        .scroll_to_me();
        h.run_steps(4);
        shot(&mut h, &format!("163-issuance-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The composition of the forces in the forces tab: Germany in February 1926 with the war rule
/// on, the target fixed by key and stepped a point up on wheels, with today's shares, the rule's
/// target and the conversions. Portuguese at 1600 × 900, English at 1280 × 720; copies in
/// `target/screenshots/composition/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn composition_section() {
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces,war") };
    let mut switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches;
    switches = switches.with("forces,war").expect("switches");
    let user = std::env::temp_dir().join(format!("bf-shots-comp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = advanced_session_with("GER", Date::new(1926, 2, 1), switches);
    let save = write_campaign(&user.join("save"), &s, "Composição", "GER", &[]);
    let dir = root().join("target/screenshots/composition");
    std::fs::create_dir_all(&dir).unwrap();
    for (locale, size, tag, heading, last) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Composição das forças",
            "Em conversão",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Composition of the forces",
            "Converting to",
        ),
    ] {
        let mut h = harness_sized(user.join(tag), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            h.run_steps(1);
        }
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.key_press(egui::Key::T);
        h.run_steps(8);
        h.get_by_label(heading).scroll_to_me();
        h.run_steps(8);
        // Fixed by key, then a point up on wheels: K F, K 2.
        for code in [
            [egui::Key::K, egui::Key::F],
            [egui::Key::K, egui::Key::Num2],
        ] {
            leader(&mut h);
            for k in code {
                tip(&mut h, k);
            }
            h.key_press(egui::Key::Escape);
            h.run_steps(4);
        }
        // The section's foot in view: the steppers and the conversions.
        h.get_all(egui_kittest::kittest::by().label_contains(last))
            .last()
            .expect("a conversion")
            .scroll_to_me();
        h.run_steps(8);
        let name = format!("241-composition-{tag}");
        shot(&mut h, &name);
        std::fs::copy(
            out_dir().join(format!("{name}.png")),
            dir.join(format!("{name}.png")),
        )
        .unwrap();
    }
    let _ = std::fs::remove_dir_all(&user);
}

/// The budget and the works at the floor of 960 × 540, in both languages: the budget's top, its
/// levers all fixed, and the works in the player's order. Copies in
/// `target/screenshots/budget-floor/`.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn budget_and_works_at_the_floor() {
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces") };
    let switches = bf_sim::game::Switches::parse("forces").expect("switches");
    let user = std::env::temp_dir().join(format!("bf-shots-floor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let s = advanced_session_with("HUN", Date::new(1926, 2, 1), switches);
    let save = write_campaign(&user.join("save"), &s, "Piso", "HUN", &[]);
    let dir = root().join("target/screenshots/budget-floor");
    std::fs::create_dir_all(&dir).unwrap();
    let keep = |h: &mut Harness<'_, App>, name: &str| {
        shot(h, name);
        std::fs::copy(
            out_dir().join(format!("{name}.png")),
            dir.join(format!("{name}.png")),
        )
        .unwrap();
    };
    let act = |h: &mut Harness<'_, App>, id: &str| {
        for _ in 0..10 {
            leader(h);
            if let Some(code) = h.state().shared.keys.code_of(id).map(str::to_owned) {
                for c in code.chars() {
                    tip(h, egui::Key::from_name(&c.to_string()).expect("key"));
                }
                h.key_press(egui::Key::Escape);
                h.run_steps(3);
                return;
            }
            h.key_press(egui::Key::Escape);
            h.run_steps(2);
            h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
            h.run_steps(4);
        }
    };
    let pages = |h: &mut Harness<'_, App>, key: egui::Key, n: usize| {
        for _ in 0..n {
            h.key_press_modifiers(egui::Modifiers::SHIFT, key);
            h.run_steps(3);
        }
    };
    for (locale, tag) in [("pt-BR", "pt"), ("en", "en")] {
        let mut h = harness_sized(user.join(tag), locale, egui::vec2(960.0, 540.0));
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save.clone()), &ctx);
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            h.run_steps(1);
        }
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.key_press(egui::Key::B);
        h.run_steps(8);
        keep(&mut h, &format!("250-budget-top-{tag}"));
        for lever in [
            "payroll",
            "pay",
            "treasury",
            "deficit",
            "purchases",
            "military",
        ] {
            act(&mut h, &format!("gov.lever.{lever}.fixed"));
        }
        pages(&mut h, egui::Key::PageUp, 12);
        for k in 1..=6 {
            pages(&mut h, egui::Key::PageDown, 1);
            keep(&mut h, &format!("251-budget-page{k}-{tag}"));
        }
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::B);
        h.run_steps(8);
        act(&mut h, "works.order.mine");
        pages(&mut h, egui::Key::PageUp, 12);
        keep(&mut h, &format!("252-works-top-{tag}"));
        pages(&mut h, egui::Key::PageDown, 2);
        keep(&mut h, &format!("253-works-down-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
