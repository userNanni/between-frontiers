//! The HUD edit mode driven by pointer events, without a GPU: a panel dragged near the screen
//! edge snaps to it and the layout keeps the new place, written and read back; the same layout
//! on a smaller screen keeps every panel on screen and apart.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bf_client::hud::{self, Anchor, GAP, Hud, Layout, PanelId};
use bf_client::screens::hud_edit::{self, Edit};
use bf_i18n::{I18n, Locale};
use egui::{Pos2, Rect, Vec2, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The panels' sizes as the campaign screen asks for them on a screen `w` wide.
fn sizes(w: f32, h: f32) -> BTreeMap<PanelId, Vec2> {
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(w, h));
    [
        (
            PanelId::Country,
            bf_client::screens::top_bar::corner_size(15),
        ),
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

/// A harness with the edit mode on and the country figures' block editor open.
fn block_harness() -> Harness<'static, State> {
    let i18n = I18n::load(&root().join("assets/i18n"), Locale::PtBr).expect("i18n");
    let mut edit = Edit::default();
    hud_edit::edit_blocks(&mut edit, hud_edit::Blocks::Country);
    let state = State {
        i18n,
        hud: Hud::default(),
        edit,
        saved: false,
        screen: Rect::NOTHING,
        themed: false,
    };
    let config = std::env::temp_dir().join(format!("bf-hud-blocks-{}", std::process::id()));
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
                    &config,
                    &mut s.hud,
                    &sizes,
                    &mut s.edit,
                );
                s.saved |= r.save;
            },
            state,
        );
    h.run_steps(2);
    h
}

fn pointer_drag(h: &mut Harness<'_, State>, from: Pos2, to: Pos2) {
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
}

fn button_rect(h: &Harness<'_, State>, label: &str) -> Rect {
    h.get_all(egui_kittest::kittest::by().label(label))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .unwrap_or_else(|| panic!("block {label}"))
        .rect()
}

#[test]
fn blocks_are_put_in_moved_and_taken_out_by_dragging() {
    let mut h = block_harness();
    assert_eq!(
        h.state().hud.layout.tiles.as_ref(),
        None,
        "the standard set"
    );
    // The public debt from the tray to the head of the large row.
    let debt = button_rect(&h, "Dívida pública");
    let first = button_rect(&h, "Qualidade de vida");
    pointer_drag(&mut h, debt.center(), first.left_center() + vec2(4.0, 0.0));
    let tiles = h.state().hud.layout.tiles();
    assert_eq!(tiles[0], "debt", "{tiles:?}");
    assert_eq!(tiles.len(), 16);
    assert!(h.state().saved);
    // Inflation clicked in from the tray: it goes at the end.
    let inflation = button_rect(&h, "Inflação");
    h.hover_at(inflation.center());
    h.run_steps(1);
    h.get_all(egui_kittest::kittest::by().label("Inflação"))
        .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .unwrap()
        .click();
    h.run_steps(2);
    assert_eq!(h.state().hud.layout.tiles().last(), Some(&"inflation"));
    // The subjects out to the tray.
    let subjects = button_rect(&h, "Súditos");
    let tray = button_rect(&h, "Caixa do Tesouro");
    pointer_drag(&mut h, subjects.center(), tray.center());
    let tiles = h.state().hud.layout.tiles();
    assert!(!tiles.contains(&"subjects"), "{tiles:?}");
    assert_eq!(tiles.len(), 16);
    // The panel takes a row more for the 16th and 17th blocks; back to standard undoes all.
    assert!(
        bf_client::screens::top_bar::corner_size(16).y
            > bf_client::screens::top_bar::corner_size(15).y
    );
    hud_edit::reset_blocks(&mut h.state_mut().hud, hud_edit::Blocks::Country);
    assert_eq!(h.state().hud.layout.tiles, None);
    // Written and read back with the file.
    let mut l = Layout::default();
    l.set_tiles(vec!["debt".into(), "inflation".into(), "gone".into()]);
    let back = Layout::parse(&ron::to_string(&l).unwrap()).unwrap();
    assert_eq!(back.tiles(), vec!["debt", "inflation"]);
}

/// The open picker's size for a height, as a fresh context measures it.
fn picker_size(lens: bf_client::map::lens::Lens) -> impl Fn(f32) -> Vec2 {
    let ctx = egui::Context::default();
    move |max_h| bf_client::screens::lens_panel::size(&ctx, lens, max_h)
}

#[test]
fn the_lens_picker_opens_toward_the_free_side() {
    use bf_client::map::lens::Lens;
    use bf_client::screens::game::lens_popup;
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 900.0));
    let size = picker_size(Lens::Climate);
    // Bottom left: up, aligned on the left.
    let b = Rect::from_min_size(egui::pos2(12.0, 848.0), vec2(200.0, 40.0));
    let p = lens_popup(b, &size, screen, None);
    assert!(p.bottom() <= b.top() && p.left() == b.left(), "{p:?}");
    // Bottom right: up and to the left.
    let b = Rect::from_min_size(egui::pos2(1388.0, 848.0), vec2(200.0, 40.0));
    let p = lens_popup(b, &size, screen, None);
    assert!(p.bottom() <= b.top() && p.right() == b.right(), "{p:?}");
    // Top right: down and to the left.
    let b = Rect::from_min_size(egui::pos2(1388.0, 80.0), vec2(200.0, 40.0));
    let p = lens_popup(b, &size, screen, None);
    assert!(p.top() >= b.bottom() && p.right() == b.right(), "{p:?}");
    assert!(screen.contains_rect(p));
}

/// Every lens, from a button in each corner and the middle of each edge, on the screens the
/// game supports: the open picker stays on the screen and out of the free area, and every part
/// of it inside its frame.
#[test]
fn the_open_lens_picker_fits_its_frame_and_the_screen() {
    use bf_client::screens::game::lens_popup;
    use bf_client::screens::lens_panel;
    let ctx = egui::Context::default();
    for (w, h) in [(1600.0, 900.0), (1280.0, 720.0), (960.0, 540.0)] {
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(w, h));
        let button = vec2(200.0, 40.0);
        let corners = [
            pos2(12.0, 12.0 + 140.0),
            pos2(w - 212.0, 80.0),
            pos2(12.0, h - 52.0),
            pos2(w - 212.0, h - 52.0),
            pos2(w / 2.0 - 100.0, h - 52.0),
        ];
        // The streaming preset's free corner, top right.
        let free = hud::Layout::preset(hud::Preset::Streaming)
            .free
            .map(|a| hud::clamp_into(a.rect(screen), screen));
        for at in corners {
            let b = Rect::from_min_size(at, button);
            for lens in lens_panel::all_lenses() {
                for free in [None, free] {
                    let p = lens_popup(b, picker_size(lens), screen, free);
                    assert!(screen.contains_rect(p), "{lens:?} {p:?} off {w}×{h}");
                    if let Some(f) = free
                        && !f.intersects(b)
                    {
                        assert!(!f.intersects(p), "{lens:?} {p:?} in the free area {f:?}");
                    }
                    let lay = lens_panel::layout(&ctx, lens, p);
                    for (part, r) in [
                        ("header", Some(lay.header)),
                        ("families", Some(lay.families)),
                        ("list", lay.list),
                        ("legend", lay.legend),
                    ] {
                        if let Some(r) = r {
                            assert!(
                                p.contains_rect(r),
                                "{lens:?} {part} {r:?} out of {p:?} at {w}×{h}"
                            );
                        }
                    }
                    // The list keeps rows in view; the legend never folds under it.
                    if let (Some(l), Some(g)) = (lay.list, lay.legend) {
                        assert!(
                            l.height() >= lay.list_h.min(48.0) - 0.5 && l.bottom() <= g.top(),
                            "{lens:?} {l:?} {g:?}"
                        );
                    }
                }
            }
        }
    }
}

/// The five families fit across the panel in both languages, as the kit draws them.
#[test]
fn the_lens_families_fit_across_the_panel() {
    use bf_client::screens::lens_panel;
    for locale in [Locale::PtBr, Locale::En] {
        let i18n = I18n::load(&root().join("assets/i18n"), locale).expect("i18n");
        let labels = lens_panel::family_labels(&i18n);
        let width = lens_panel::inner_width();
        let mut right = 0.0f32;
        let mut h = Harness::builder()
            .with_size(vec2(800.0, 200.0))
            .build_ui(|ui| {
                bf_ui_kit::theme::install(ui.ctx());
                let area = Rect::from_min_size(pos2(10.0, 10.0), vec2(width, 60.0));
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area));
                let options: Vec<(usize, String)> = labels.iter().cloned().enumerate().collect();
                let mut v = 0usize;
                let (_, rects) = bf_ui_kit::segmented_rects(&mut child, &mut v, &options);
                right = rects.iter().map(|r| r.right()).fold(0.0, f32::max) - 10.0;
            });
        h.run_steps(3);
        drop(h);
        assert!(
            right <= width,
            "{locale:?}: families {right} wide in {width}"
        );
    }
}
