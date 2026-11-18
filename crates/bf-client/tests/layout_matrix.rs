//! The interface fits every logical screen it supports, without a GPU: the app runs headless at
//! each size, in both languages, on each screen, and every element it reports for
//! accessibility lies on the screen and, across, inside the window or panel that holds it.
//! Down, an element may pass the screen's foot only inside an area that scrolls.
//!
//! The interface scale only changes the logical screen (`ui_scale` tests the arithmetic), so
//! the sizes here are logical, from the supported floor of 960 × 540 to 4K at 100 %.

use bf_client::App;
use bf_client::app::Transition;
use bf_client::saves;
use bf_client::screens::Screen;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

mod common;
use common::layout::*;

/// The HUD's panels never cover each other.
fn check_hud(h: &Harness<'_, App>, case: &str) -> Vec<String> {
    check_hud_with(h, case, &["country-figures", "time", "country-panel"])
}

/// [`check_hud`] wanting the panels `want` on screen.
fn check_hud_with(h: &Harness<'_, App>, case: &str, want: &[&str]) -> Vec<String> {
    const HUD: [&str; 5] = [
        "country-figures",
        "time",
        "overview",
        "country-panel",
        "population",
    ];
    let frames: Vec<Node> = nodes(h)
        .into_iter()
        .filter(|n| n.role == egui::accesskit::Role::Window && HUD.contains(&n.label.as_str()))
        .collect();
    let mut out = Vec::new();
    for want in want {
        if !frames.iter().any(|f| f.label == *want) {
            out.push(format!("{case}: no {want} on screen"));
        }
    }
    for (i, a) in frames.iter().enumerate() {
        for b in &frames[i + 1..] {
            if a.rect.intersects(b.rect) && a.rect.intersect(b.rect).area() > 1.0 {
                out.push(format!(
                    "{case}: {} {:?} over {} {:?}",
                    a.label, a.rect, b.label, b.rect
                ));
            }
        }
    }
    out
}

/// Opens the player's country panel (the country command; the banner may have moved).
fn open_country(h: &mut Harness<'_, App>) {
    h.key_press(egui::Key::C);
    steps(h);
}

/// Sets the map's lens.
fn set_lens(h: &mut Harness<'_, App>, lens: bf_client::map::lens::Lens) {
    let app = h.state_mut();
    if let (Some(view), Some(world)) = (&mut app.shared.map_view, &app.shared.world) {
        bf_client::screens::lens_panel::choose(view, world, lens, 1925);
    }
    steps(h);
}

fn steps(h: &mut Harness<'_, App>) {
    h.run_steps(4);
}

/// Opens a campaign for Turkey and lets a week run, so the windows have figures. Turkey's
/// budget has every source of revenue from the start: income tax, tax on consumption (its
/// taxation law caps the income tax) and profit of state firms.
fn open_campaign(h: &mut Harness<'_, App>, user: &std::path::Path) {
    let mut save = saves::create(
        &user.join("saves"),
        "Matriz",
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some("TUR".into());
    saves::write(&save).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    steps(h);
    let snap = |h: &Harness<'_, App>| match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    };
    h.key_press(egui::Key::Space);
    let started = std::time::Instant::now();
    while snap(h).tick < 8 && started.elapsed().as_secs() < 180 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
    h.key_press(egui::Key::Space);
    let paused = std::time::Instant::now();
    while !snap(h).paused && paused.elapsed().as_secs() < 30 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
    steps(h);
}

fn click(h: &mut Harness<'_, App>, label: &str) {
    h.get_all(egui_kittest::kittest::by().label(label))
        .next()
        .unwrap_or_else(|| panic!("no {label}"))
        .click();
    steps(h);
}

#[test]
fn every_screen_fits_every_supported_logical_size() {
    let mut failures = Vec::new();
    for (locale, t) in [("pt-BR", PT), ("en", EN)] {
        let user = std::env::temp_dir().join(format!("bf-matrix-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        steps(&mut h);
        // Out of a campaign: the menu, the settings, the keyboard editor.
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh}");
            failures.extend(check(&h, &format!("{case} menu")));
            click(&mut h, t.settings);
            failures.extend(check(&h, &format!("{case} settings")));
            click(&mut h, t.keys);
            failures.extend(check(&h, &format!("{case} keys")));
            h.key_press(egui::Key::Escape);
            steps(&mut h);
            if h.query_all(egui_kittest::kittest::by().label(t.done))
                .next()
                .is_some()
            {
                click(&mut h, t.done);
            }
        }
        // In a campaign: the HUD, each tab of the government, the agendas, research, the
        // HUD's edit mode and block editor, the open lens picker.
        h.set_size(egui::vec2(1600.0, 900.0));
        open_campaign(&mut h, &user);
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh}");
            failures.extend(check(&h, &format!("{case} map")));
            open_country(&mut h);
            failures.extend(check(&h, &format!("{case} map with country")));
            failures.extend(check_hud(&h, &format!("{case} map with country")));
            h.key_press(egui::Key::Y);
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} population")));
            // Side by side where both fit; else the population panel takes the country
            // panel's place.
            let want: &[&str] = if w < bf_client::screens::population::SIDE_BY_SIDE {
                &["country-figures", "time", "population"]
            } else {
                &["country-figures", "time", "country-panel", "population"]
            };
            failures.extend(check_hud_with(&h, &format!("{case} population"), want));
            h.key_press(egui::Key::Y);
            steps(&mut h);
            if nodes(&h).iter().any(|n| n.label == "population") {
                failures.push(format!("{case}: population panel did not close"));
            }
            // The country panel again, as Escape below expects.
            open_country(&mut h);
            h.key_press(egui::Key::Escape);
            steps(&mut h);
            h.key_press(egui::Key::G);
            steps(&mut h);
            for tab in t.tabs {
                click(&mut h, tab);
                failures.extend(check(&h, &format!("{case} government {tab}")));
            }
            h.key_press(egui::Key::G);
            steps(&mut h);
            h.key_press(egui::Key::J);
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} agendas")));
            h.key_press(egui::Key::J);
            steps(&mut h);
            h.key_press(egui::Key::R);
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} research")));
            h.key_press(egui::Key::R);
            steps(&mut h);
            h.key_press(egui::Key::F8);
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} hud edit")));
            click(&mut h, t.edit_tiles);
            failures.extend(check(&h, &format!("{case} blocks")));
            click(&mut h, t.done);
            click(&mut h, t.done);
            h.get_all(egui_kittest::kittest::by().label_contains(t.lens_button))
                .next()
                .expect("map mode button")
                .click();
            steps(&mut h);
            click(&mut h, t.resources);
            failures.extend(check(&h, &format!("{case} lens picker")));
            h.key_press(egui::Key::Escape);
            steps(&mut h);
            // The sea lens, with the route Bombay–Kent and Suez closed in the preview.
            set_lens(&mut h, bf_client::map::lens::Lens::Sea);
            failures.extend(check(&h, &format!("{case} sea lens")));
            if let Screen::Game(g) = &mut h.state_mut().screen {
                let v = g.sea_view_mut();
                v.from = Some(358);
                v.to = Some(276);
                v.closed.insert(11);
                v.focus = Some(0);
            }
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} sea route")));
            if let Screen::Game(g) = &mut h.state_mut().screen {
                g.sea_view_mut().clear();
            }
            set_lens(&mut h, bf_client::map::lens::Lens::Political);
        }
        // The streaming preset, and the middle band on wide screens.
        for (layout, name) in [
            (
                bf_client::hud::Layout::preset(bf_client::hud::Preset::Streaming),
                "streaming",
            ),
            (
                bf_client::hud::Layout {
                    center_band: true,
                    ..bf_client::hud::Layout::default()
                },
                "band",
            ),
        ] {
            h.state_mut().shared.hud = bf_client::hud::Hud::new(layout);
            for (w, hh) in SIZES {
                h.set_size(egui::vec2(w, hh));
                steps(&mut h);
                open_country(&mut h);
                let case = format!("{locale} {w}×{hh} {name}");
                failures.extend(check(&h, &case));
                failures.extend(check_hud(&h, &case));
                h.key_press(egui::Key::Escape);
                steps(&mut h);
            }
        }
        // Restore points: a quick save, the in-game menu, the dialog to save with a name; then
        // the campaigns screen with the campaign's points and its "…" menu.
        h.state_mut().shared.hud = bf_client::hud::Hud::new(bf_client::hud::Layout::default());
        h.set_size(egui::vec2(1600.0, 900.0));
        steps(&mut h);
        h.key_press(egui::Key::F5);
        let quick = user.join("saves/matriz/points/quick.ron");
        let started = std::time::Instant::now();
        while !quick.is_file() && started.elapsed().as_secs() < 60 {
            std::thread::sleep(std::time::Duration::from_millis(20));
            h.run_steps(1);
        }
        assert!(quick.is_file(), "the quick save wrote no point");
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh}");
            h.key_press(egui::Key::F10);
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} game menu")));
            h.key_press(egui::Key::Escape);
            steps(&mut h);
            h.key_press_modifiers(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            );
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} save with a name")));
            if !nodes(&h).iter().any(|n| n.label == "save-named") {
                failures.push(format!("{case}: no dialog to save with a name"));
            }
            h.key_press(egui::Key::Escape);
            steps(&mut h);
        }
        let ctx = h.ctx.clone();
        h.state_mut().shared.refresh_saves();
        h.state_mut().apply(Transition::Saves, &ctx);
        steps(&mut h);
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh}");
            failures.extend(check(&h, &format!("{case} campaigns")));
            h.get_all(
                egui_kittest::kittest::by().label_contains(MENU_LABEL[usize::from(locale == "en")]),
            )
            .next()
            .expect("options button")
            .click();
            steps(&mut h);
            failures.extend(check(&h, &format!("{case} campaign menu")));
            if !nodes(&h).iter().any(|n| n.label == "campaign-menu") {
                failures.push(format!("{case}: no campaign menu"));
            }
            h.key_press(egui::Key::Escape);
            steps(&mut h);
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(
        failures.is_empty(),
        "{} elements out of place:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Labels the walk clicks, per language.
struct Labels {
    settings: &'static str,
    keys: &'static str,
    done: &'static str,
    tabs: [&'static str; 4],
    edit_tiles: &'static str,
    lens_button: &'static str,
    resources: &'static str,
}

const PT: Labels = Labels {
    settings: "Configurações",
    keys: "Editar atalhos",
    done: "Concluir",
    tabs: [
        "Orçamento",
        "Moeda e dívida",
        "Dívidas entre governos",
        "Leis",
    ],
    edit_tiles: "Editar os números do país",
    lens_button: "Modo do mapa:",
    resources: "Recursos",
};

const EN: Labels = Labels {
    settings: "Settings",
    keys: "Edit shortcuts",
    done: "Done",
    tabs: [
        "Budget",
        "Money and debt",
        "Debts between governments",
        "Laws",
    ],
    edit_tiles: "Edit the country figures",
    lens_button: "Map mode:",
    resources: "Resources",
};

/// A name of 60 characters.
const LONG_NAME: &str = "Campanha com um nome bem comprido para testar onde ele corta";

/// Twenty campaigns (one with a 60-character name, one whose folder passes a thousand MB, the
/// longest country name and a long date) and thirty restore points with long names.
fn crowded_saves(user: &std::path::Path) {
    use bf_client::points::{self, Kind, Point};
    assert_eq!(LONG_NAME.chars().count(), 60);
    let dir = user.join("saves");
    let date = laplace_schelling::calendar::Date::new(1999, 9, 28);
    for n in 0..20 {
        let name = if n == 0 {
            LONG_NAME.to_owned()
        } else {
            format!("Campanha {n}")
        };
        let mut save = saves::create(&dir, &name, 7, date).unwrap();
        // The longest country name of the scenario, in both languages.
        save.meta.player = Some("YUG".into());
        save.meta.last_played = 1_758_000_000 + n;
        saves::write(&save).unwrap();
        if n == 0 {
            // A folder of more than a thousand MB, without writing them: a sparse file.
            let big = std::fs::File::create(save.dir.join("world.lss")).unwrap();
            big.set_len(1_234_000_000).unwrap();
            for k in 0..30u64 {
                let id = format!("p{k}");
                std::fs::create_dir_all(points::dir(&save)).unwrap();
                std::fs::write(points::save_path(&save, &id), b"x").unwrap();
                points::record(
                    &save,
                    Point {
                        id,
                        kind: if k % 3 == 0 { Kind::Manual } else { Kind::Auto },
                        name: (k % 3 == 0).then(|| format!("{LONG_NAME} {k}")),
                        date,
                        written: 1_758_000_000 + k,
                        bytes: 0,
                    },
                )
                .unwrap();
            }
        }
    }
}

/// The campaigns screen holds long names, a big folder, twenty campaigns and thirty points at
/// the floor, 720p, 900p and 4K, with the interface at 100, 125 and 150 %, in both languages;
/// and the campaign menu stays in the window.
#[test]
fn the_campaigns_screen_holds_long_and_many() {
    let mut failures = Vec::new();
    for locale in ["pt-BR", "en"] {
        let user =
            std::env::temp_dir().join(format!("bf-matrix-crowd-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        crowded_saves(&user);
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        steps(&mut h);
        let ctx = h.ctx.clone();
        h.state_mut().shared.refresh_saves();
        h.state_mut().apply(Transition::Saves, &ctx);
        steps(&mut h);
        for (w, hh) in [
            (960.0, 540.0),
            (1280.0, 720.0),
            (1600.0, 900.0),
            (3840.0, 2160.0),
        ] {
            for scale in [100u32, 125, 150] {
                let ppp = bf_client::ui_scale::pixels_per_point(
                    bf_client::ui_scale::Choice::Fixed(scale),
                    (w, hh),
                    1.0,
                );
                let size = egui::vec2(w / ppp, hh / ppp);
                h.set_size(size);
                steps(&mut h);
                let case = format!("{locale} {w}×{hh} at {scale} % ({}×{})", size.x, size.y);
                failures.extend(check(&h, &format!("{case} campaigns")));
                // The menu of the first campaign, then of the last one (scrolled to it).
                for which in [0usize, 19] {
                    let label = MENU_LABEL[usize::from(locale == "en")];
                    let Some(b) = options_button(&h, label, which != 0) else {
                        failures.push(format!("{case}: no options button"));
                        continue;
                    };
                    b.scroll_to_me();
                    steps(&mut h);
                    if let Some(b) = options_button(&h, label, which != 0) {
                        b.click();
                    }
                    steps(&mut h);
                    failures.extend(check(&h, &format!("{case} menu of campaign {which}")));
                    let screen = h.ctx.content_rect();
                    match nodes(&h).into_iter().find(|n| n.label == "campaign-menu") {
                        Some(m) if !screen.contains_rect(m.rect) => failures
                            .push(format!("{case}: the menu {:?} leaves the window", m.rect)),
                        Some(_) => {}
                        None => {
                            failures.push(format!("{case}: no campaign menu"));
                            continue;
                        }
                    }
                    h.key_press(egui::Key::Escape);
                    steps(&mut h);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(
        failures.is_empty(),
        "{} elements out of place:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Name of the options button of a campaign, per language.
const MENU_LABEL: [&str; 2] = ["Opções da campanha", "Campaign options"];

/// The options button of the first campaign, or of the last.
fn options_button<'a>(
    h: &'a Harness<'_, App>,
    label: &'a str,
    last: bool,
) -> Option<egui_kittest::Node<'a>> {
    let mut all: Vec<_> = h
        .query_all(egui_kittest::kittest::by().label_contains(label))
        .collect();
    if last {
        all.pop()
    } else {
        all.into_iter().next()
    }
}

/// Thirty debts of Turkey, owed and owed to, with the longest country name of the scenario and
/// every term switched on: the debts tab holds them at the floor, 720p, 900p and 4K, in both
/// languages.
#[test]
fn the_debts_tab_holds_many_and_long() {
    use bf_sim::treaty::{Currency, InterstateDebt, Pledge};
    use laplace_schelling::Fixed;
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut world = data.new_world();
    let ix = |tag: &str| {
        world
            .countries
            .iter()
            .position(|(_, c)| c.tag == tag)
            .unwrap() as u16
    };
    let (tur, others) = (
        ix("TUR"),
        [ix("YUG"), ix("ITA"), ix("GBR"), ix("FRA"), ix("USA")],
    );
    for k in 0..30u32 {
        let other = others[k as usize % others.len()];
        let (debtor, creditor) = if k % 2 == 0 {
            (tur, other)
        } else {
            (other, tur)
        };
        world.econ.ledger.create(InterstateDebt {
            id: 0,
            debtor,
            creditor,
            treaty: format!("a_treaty_with_a_very_long_key_for_the_test_{k}"),
            currency: Currency::Creditor,
            principal: Fixed::from_int(987_654_321_000),
            rate: Fixed::from_ratio(55, 1000),
            annuity: Fixed::from_int(12_345_678_900),
            years_left: 62,
            deferrable: Fixed::from_ratio(1, 4),
            transfer_cap: Some(Fixed::from_ratio(3, 100)),
            pledge: Some(Pledge::ConsumptionTaxes),
            contingent: Fixed::from_int(500_000_000_000),
            arrears: if k % 3 == 0 {
                Fixed::from_int(1_000_000_000)
            } else {
                Fixed::ZERO
            },
            suspended_until: if k % 5 == 0 { 1932 } else { 0 },
            paid: Fixed::ZERO,
            written_off: Fixed::ZERO,
            status: Default::default(),
        });
    }
    let mut session = bf_sim::game::Session::new(
        world,
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
        1,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "TUR".into() });
    let mut failures = Vec::new();
    for (locale, t) in [("pt-BR", PT), ("en", EN)] {
        let user =
            std::env::temp_dir().join(format!("bf-matrix-debts-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save = saves::create(
            &user.join("saves"),
            "Dívidas",
            7,
            laplace_schelling::calendar::Date::new(1925, 1, 1),
        )
        .unwrap();
        save.meta.player = Some("TUR".into());
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "debts").unwrap(),
        )
        .unwrap();
        saves::write(&save).unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        steps(&mut h);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        steps(&mut h);
        h.key_press(egui::Key::G);
        steps(&mut h);
        click(&mut h, t.tabs[2]);
        let mut sizes = Vec::new();
        for (w, hh) in [
            (960.0, 540.0),
            (1280.0, 720.0),
            (1600.0, 900.0),
            (3840.0, 2160.0),
        ] {
            for scale in [100u32, 125, 150] {
                let ppp = bf_client::ui_scale::pixels_per_point(
                    bf_client::ui_scale::Choice::Fixed(scale),
                    (w, hh),
                    1.0,
                );
                sizes.push((w, hh, scale, w / ppp, hh / ppp));
            }
        }
        for (w, hh, scale, lw, lh) in sizes {
            h.set_size(egui::vec2(lw, lh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh} at {scale} % ({lw}×{lh}) debts");
            failures.extend(check(&h, &case));
            failures.extend(no_overlapping_text(&h, &case));
            failures.extend(close_beside_tabs(
                &h,
                &case,
                if locale == "en" { "Close" } else { "Fechar" },
            ));
            if h.query_all(egui_kittest::kittest::by().label_contains("Serbs"))
                .next()
                .is_none()
                && h.query_all(egui_kittest::kittest::by().label_contains("Sérvios"))
                    .next()
                    .is_none()
            {
                failures.push(format!("{case}: no line of the long-named country"));
            }
        }
        // The influence window: the thirty debts are credit, so the lists fill.
        h.key_press(egui::Key::Escape);
        steps(&mut h);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::I);
        steps(&mut h);
        for side in [
            None,
            Some(if locale == "en" {
                "Who weighs on us"
            } else {
                "Quem pesa sobre nós"
            }),
        ] {
            if let Some(label) = side {
                click(&mut h, label);
            }
            for (w, hh) in [(960.0, 540.0), (1280.0, 720.0), (3840.0, 2160.0)] {
                h.set_size(egui::vec2(w, hh));
                steps(&mut h);
                let case = format!("{locale} {w}×{hh} influence {side:?}");
                failures.extend(check(&h, &case));
                failures.extend(no_overlapping_text(&h, &case));
                if !nodes(&h).iter().any(|n| n.label == "influence") {
                    failures.push(format!("{case}: no influence window"));
                }
            }
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(
        failures.is_empty(),
        "{} elements out of place:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The window's close button (an X) sits on the line of its tabs, centred with them, clear of
/// them and inside the window.
fn close_beside_tabs(h: &Harness<'_, App>, case: &str, close: &str) -> Vec<String> {
    let all = nodes(h);
    let Some(x) = all
        .iter()
        .find(|n| n.role == egui::accesskit::Role::Button && n.label == close)
    else {
        return vec![format!("{case}: no close button")];
    };
    let tabs: Vec<&Node> = all
        .iter()
        .filter(|n| n.role == egui::accesskit::Role::RadioButton)
        .collect();
    let mut out = Vec::new();
    if x.rect.width() < 31.0 || x.rect.height() < 31.0 {
        out.push(format!(
            "{case}: close button {:?} smaller than a button",
            x.rect
        ));
    }
    for t in &tabs {
        if (t.rect.center().y - x.rect.center().y).abs() > 2.0 {
            out.push(format!(
                "{case}: close {:?} not on the line of tab {:?} {:?}",
                x.rect, t.label, t.rect
            ));
        }
        if t.rect.intersects(x.rect) {
            out.push(format!("{case}: close {:?} over tab {:?}", x.rect, t.label));
        }
    }
    out
}

/// The country choice by keyboard: Ctrl+F opens the list of countries, typing filters it, Enter
/// takes the first; the list and the sidebar fit every size.
#[test]
fn the_country_is_chosen_by_keyboard() {
    let mut failures = Vec::new();
    for locale in ["pt-BR", "en"] {
        let user =
            std::env::temp_dir().join(format!("bf-matrix-pick-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let save = saves::create(
            &user.join("saves"),
            "Escolha",
            7,
            laplace_schelling::calendar::Date::new(1925, 1, 1),
        )
        .unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        steps(&mut h);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Pick(save), &ctx);
        steps(&mut h);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::F);
        steps(&mut h);
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh} country list");
            failures.extend(check(&h, &case));
            if !nodes(&h).iter().any(|n| n.label == "pick-countries") {
                failures.push(format!("{case}: no list of countries"));
            }
        }
        h.set_size(egui::vec2(1280.0, 720.0));
        h.event(egui::Event::Text("BRA".into()));
        steps(&mut h);
        h.key_press(egui::Key::Enter);
        steps(&mut h);
        let start = if locale == "en" {
            "Start campaign"
        } else {
            "Começar campanha"
        };
        if h.query_all(egui_kittest::kittest::by().label_contains(start))
            .next()
            .is_none()
        {
            failures.push(format!("{locale}: Enter did not choose Brazil"));
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
