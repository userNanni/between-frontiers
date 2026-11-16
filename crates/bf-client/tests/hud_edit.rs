//! The HUD edit mode driven by pointer events, without a GPU: a panel dragged near the screen
//! edge snaps to it and the layout keeps the new place, written and read back; the same layout
//! on a smaller screen keeps every panel on screen and apart.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bf_client::hud::{self, Anchor, GAP, Hud, Layout, PanelId};
use bf_client::screens::hud_edit::{self, Edit};
use bf_i18n::{I18n, Locale};
use egui::{Pos2, Rect, Vec2, vec2};
use egui_kittest::Harness;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The panels' sizes as the campaign screen asks for them on a screen `w` wide.
fn sizes(w: f32, h: f32) -> BTreeMap<PanelId, Vec2> {
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(w, h));
    [
        (PanelId::Country, bf_client::screens::top_bar::corner_size()),
        (
            PanelId::Time,
            bf_client::screens::top_bar::time_size(screen),
        ),
        (PanelId::Outliner, vec2(300.0, 420.0)),
        (PanelId::MapModes, vec2(400.0, 122.0)),
        (PanelId::CountryPanel, vec2(380.0, h - 96.0)),
    ]
    .into_iter()
    .collect()
}

struct State {
    i18n: I18n,
    hud: Hud,
    edit: Edit,
    saved: bool,
    screen: Rect,
    themed: bool,
}

#[test]
fn a_dragged_panel_snaps_to_the_edge_and_keeps_its_place() {
    let i18n = I18n::load(&root().join("assets/i18n"), Locale::PtBr).expect("i18n");
    let mut edit = Edit::default();
    hud_edit::toggle(&mut edit);
    let state = State {
        i18n,
        hud: Hud::default(),
        edit,
        saved: false,
        screen: Rect::NOTHING,
        themed: false,
    };
    let config = std::env::temp_dir().join(format!("bf-hud-edit-{}", std::process::id()));
    let config2 = config.clone();
    let mut h = Harness::builder()
        .with_size(vec2(1600.0, 900.0))
        .build_ui_state(
            move |ui, s: &mut State| {
                if !s.themed {
                    bf_ui_kit::theme::install(ui.ctx());
                    s.themed = true;
                }
                let screen = ui.ctx().content_rect();
                s.screen = screen;
                let sizes = sizes(screen.width(), screen.height());
                let r = hud_edit::show(
                    ui,
                    screen,
                    &s.i18n,
                    &config2,
                    &mut s.hud,
                    &sizes,
                    &mut s.edit,
                );
                s.saved |= r.save;
            },
            state,
        );
    h.run_steps(2);
    let screen = h.state().screen;
    let sizes = sizes(screen.width(), screen.height());
    let before = h
        .state_mut()
        .hud
        .resolved(screen, &sizes)
        .rect(PanelId::Outliner)
        .unwrap();
    // The country panel holds the right edge in the edit mode. Grab the overview by its
    // middle and drop it with its right edge 5 px short of the gap beside that panel.
    let country = h
        .state_mut()
        .hud
        .resolved(screen, &sizes)
        .rect(PanelId::CountryPanel)
        .unwrap();
    let from = before.center();
    let target_left = country.left() - GAP - before.width() - 5.0;
    let to = from + vec2(target_left - before.left(), 40.0);
    h.hover_at(from);
    h.run_steps(1);
    h.drag_at(from);
    h.run_steps(1);
    for k in 1..=8 {
        h.hover_at(from + (to - from) * (k as f32 / 8.0));
        h.run_steps(1);
    }
    h.drop_at(to);
    h.run_steps(2);

    assert!(
        h.state().saved,
        "the drop asks for the layout to be written"
    );
    let after = h
        .state_mut()
        .hud
        .resolved(screen, &sizes)
        .rect(PanelId::Outliner)
        .unwrap();
    assert_eq!(
        after.right(),
        country.left() - GAP,
        "snapped beside the country panel: {after:?}"
    );
    assert!(
        (after.top() - (before.top() + 40.0)).abs() <= 10.0,
        "{after:?}"
    );
    // Kept as an anchor and an offset that give the same rectangle back.
    let place = h.state().hud.layout.place(PanelId::Outliner);
    assert_eq!(place.anchor, Anchor::Center, "{place:?}");
    assert_eq!(place.rect(screen, sizes[&PanelId::Outliner]), after);

    // Written and read back as it is.
    let file = hud::file(&config);
    h.state().hud.layout.save(&file).unwrap();
    assert_eq!(Layout::load(&file), h.state().hud.layout);
    let _ = std::fs::remove_dir_all(&config);

    // The other panels did not move.
    let standard = hud::resolve(&Layout::default(), screen, &sizes);
    let now = hud::resolve(&h.state().hud.layout, screen, &sizes);
    for p in [PanelId::Country, PanelId::Time, PanelId::MapModes] {
        assert_eq!(now.rect(p), standard.rect(p), "{p:?}");
    }

    // From 1600 × 900 to 1280 × 720: on screen and apart.
    let small = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0));
    let r = hud::resolve(&h.state().hud.layout, small, &self::sizes(1280.0, 720.0));
    let rects: Vec<_> = r.rects.iter().collect();
    for (i, (p, a)) in rects.iter().enumerate() {
        assert!(small.contains_rect(**a), "{p:?} {a:?} off 1280×720");
        for (q, b) in &rects[i + 1..] {
            assert!(
                !a.intersects(**b) || a.intersect(**b).area() <= 1.0,
                "{p:?} {a:?} over {q:?} {b:?}"
            );
        }
    }
}

#[test]
fn every_preset_fits_both_screens() {
    for preset in hud::Preset::CHOICES {
        let layout = Layout::preset(preset);
        for (w, h) in [(1600.0, 900.0), (1280.0, 720.0)] {
            let screen = Rect::from_min_size(Pos2::ZERO, vec2(w, h));
            let r = hud::resolve(&layout, screen, &sizes(w, h));
            // The country figures keep their corner in every preset.
            assert_eq!(
                r.rect(PanelId::Country).map(|c| c.min),
                Some(egui::pos2(GAP, GAP)),
                "{preset:?} at {w}×{h}"
            );
            let rects: Vec<_> = r.rects.iter().collect();
            for (i, (p, a)) in rects.iter().enumerate() {
                assert!(
                    screen.contains_rect(**a),
                    "{preset:?} {p:?} {a:?} off {w}×{h}"
                );
                if let Some(free) = r.free {
                    assert!(
                        !a.intersects(free) || a.intersect(free).area() <= 1.0,
                        "{preset:?} {p:?} in the free area"
                    );
                }
                for (q, b) in &rects[i + 1..] {
                    assert!(
                        !a.intersects(**b) || a.intersect(**b).area() <= 1.0,
                        "{preset:?} {p:?} {a:?} over {q:?} {b:?} at {w}×{h}"
                    );
                }
            }
        }
    }
}
