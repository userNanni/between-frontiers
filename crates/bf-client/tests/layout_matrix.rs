//! The interface fits every logical screen it supports, without a GPU: the app runs headless at
//! each size, in both languages, on each screen, and every element it reports for
//! accessibility lies on the screen and, across, inside the window or panel that holds it.
//! Down, an element may pass the screen's foot only inside an area that scrolls.
//!
//! The interface scale only changes the logical screen (`ui_scale` tests the arithmetic), so
//! the sizes here are logical, from the supported floor of 960 × 540 to 4K at 100 %.

use std::path::PathBuf;

use bf_client::App;
use bf_client::app::Transition;
use bf_client::paths::Paths;
use bf_client::saves;
use bf_client::screens::Screen;
use egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Logical screens: the floor, 4:3, 16:9, 16:10 (the Steam Deck), 21:9 and 4K at 100 %.
const SIZES: [(f32, f32); 10] = [
    (960.0, 540.0),
    (1024.0, 768.0),
    (1280.0, 720.0),
    (1280.0, 800.0),
    (1600.0, 900.0),
    (1920.0, 1080.0),
    (1920.0, 1200.0),
    (2560.0, 1080.0),
    (3440.0, 1440.0),
    (3840.0, 2160.0),
];

fn app(user: PathBuf, locale: &str, size: egui::Vec2) -> Harness<'static, App> {
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        format!("(locale: \"{locale}\", ui_scale: 100, map_labels: true, autosave: Off)"),
    )
    .unwrap();
    let paths = Paths::new(&root(), user.clone(), user);
    Harness::builder()
        .with_size(size)
        .build_eframe(move |cc| App::new(cc, paths))
}

struct Node {
    role: egui::accesskit::Role,
    label: String,
    rect: Rect,
}

fn nodes(h: &Harness<'_, App>) -> Vec<Node> {
    let screen = h.ctx.content_rect();
    h.query_all(egui_kittest::kittest::by())
        .filter_map(|n| {
            let a = n.accesskit_node();
            let b = a.bounding_box()?;
            Some(Node {
                role: a.role(),
                label: a.label().unwrap_or_default(),
                rect: Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                ),
            })
        })
        // Veils and catchers the size of the screen hold nothing to read.
        .filter(|n| {
            n.rect.width() > 0.0 && n.rect.height() > 0.0 && n.rect.area() < screen.area() * 0.9
        })
        .collect()
}

/// What failed on one screen, as lines.
fn check(h: &Harness<'_, App>, case: &str) -> Vec<String> {
    use egui::accesskit::Role;
    let screen = h.ctx.content_rect();
    let all = nodes(h);
    let is_frame = |n: &Node| matches!(n.role, Role::Window | Role::Pane);
    let scrolls: Vec<Rect> = all
        .iter()
        .filter(|n| n.role == Role::ScrollBar)
        .map(|n| n.rect)
        .collect();
    let tol = 0.5;
    let mut out = Vec::new();
    for (i, n) in all.iter().enumerate() {
        let r = n.rect;
        if is_frame(n) {
            if !screen.expand(tol).contains_rect(r) {
                out.push(format!("{case}: frame {} {r:?} off the screen", n.label));
            }
            continue;
        }
        if n.role == Role::ScrollBar {
            continue;
        }
        // Across: on the screen, and inside the frame drawn last before it under its middle
        // (the window it belongs to; frames drawn later cover it).
        if r.left() < screen.left() - tol || r.right() > screen.right() + tol {
            out.push(format!(
                "{case}: {:?} {:?} {r:?} off the screen across",
                n.role, n.label
            ));
            continue;
        }
        if let Some(f) = all[..i]
            .iter()
            .rev()
            .find(|f| is_frame(f) && f.rect.contains(r.center()))
            && (r.left() < f.rect.left() - tol || r.right() > f.rect.right() + tol)
        {
            out.push(format!(
                "{case}: {:?} {:?} {r:?} out of {} {:?}",
                n.role, n.label, f.label, f.rect
            ));
        }
        // Down: on the screen, unless an area that scrolls holds it.
        if r.top() < screen.top() - tol || r.bottom() > screen.bottom() + tol {
            let scrolled = scrolls
                .iter()
                .any(|s| s.right() >= r.right() - tol && s.top() <= r.top() + tol);
            if !scrolled {
                out.push(format!(
                    "{case}: {:?} {:?} {r:?} off the screen down",
                    n.role, n.label
                ));
            }
        }
    }
    out
}

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
            click(&mut h, "…");
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
    tabs: [&'static str; 3],
    edit_tiles: &'static str,
    lens_button: &'static str,
    resources: &'static str,
}

const PT: Labels = Labels {
    settings: "Configurações",
    keys: "Editar atalhos",
    done: "Concluir",
    tabs: ["Orçamento", "Moeda e dívida", "Leis"],
    edit_tiles: "Editar os números do país",
    lens_button: "Modo do mapa:",
    resources: "Recursos",
};

const EN: Labels = Labels {
    settings: "Settings",
    keys: "Edit shortcuts",
    done: "Done",
    tabs: ["Budget", "Money and debt", "Laws"],
    edit_tiles: "Edit the country figures",
    lens_button: "Map mode:",
    resources: "Resources",
};
