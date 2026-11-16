//! Arranging the HUD: every panel gets an outline and a handle, moves by dragging and snaps to
//! the screen edges and to the other panels; a toolbar shows and hides panels, picks a preset,
//! marks the free area, exports, imports and resets. Outside this mode nothing moves.

use std::collections::BTreeMap;

use bf_i18n::{Arg, I18n};
use bf_ui_kit::theme::sans_semibold;
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};

use crate::hud::{self, Area, GAP, Hud, PanelId, Placement, Preset, SNAP};

/// The edit mode's own state.
#[derive(Clone, Debug, Default)]
pub struct Edit {
    pub on: bool,
    /// The panel being dragged and where the pointer holds it.
    drag: Option<(PanelId, Vec2)>,
    /// Marking the free area: where the drag started.
    marking: bool,
    mark_from: Option<Pos2>,
    /// Where the toolbar was dragged from the centre.
    toolbar: Vec2,
}

/// Turns the edit mode on or off (a command calls this).
pub fn toggle(edit: &mut Edit) {
    *edit = Edit {
        on: !edit.on,
        ..Edit::default()
    };
}

#[derive(Default)]
pub struct Response {
    /// The layout changed and should be written.
    pub save: bool,
    /// A line for the toast.
    pub message: Option<String>,
}

const TOOLBAR_W: f32 = 560.0;
const HANDLE_H: f32 = 24.0;

/// Where each panel's handle goes: above the panel when there is room and it touches no
/// other panel nor handle, inside the top of the panel otherwise, so that every handle stays
/// clickable.
pub fn handles(rects: &BTreeMap<PanelId, Rect>, screen: Rect) -> BTreeMap<PanelId, Rect> {
    let mut out: BTreeMap<PanelId, Rect> = BTreeMap::new();
    for (&p, &r) in rects {
        let above =
            Rect::from_min_size(r.min - vec2(0.0, HANDLE_H + 2.0), vec2(r.width(), HANDLE_H));
        let clear = |h: Rect| {
            screen.contains_rect(h)
                && !rects.iter().any(|(o, b)| *o != p && b.intersects(h))
                && !out.values().any(|o| o.intersects(h))
        };
        let mut h = if clear(above) {
            above
        } else {
            Rect::from_min_size(r.min, vec2(r.width(), HANDLE_H))
        };
        // An inside handle under another one moves down the panel.
        while let Some(o) = out.values().find(|o| o.intersects(h))
            && h.bottom() + HANDLE_H <= r.bottom()
        {
            h = h.translate(vec2(0.0, o.bottom() - h.top() + 2.0));
        }
        out.insert(p, h);
    }
    out
}

/// Moves the free area by its body and resizes it by its corners; the new rectangle, and
/// whether the gesture ended.
fn free_area_grips(ui: &mut Ui, free: Rect, screen: Rect) -> Option<(Rect, bool)> {
    const GRIP: f32 = 16.0;
    const MIN: f32 = 80.0;
    let body = ui
        .interact(free, ui.make_persistent_id("hud-free-move"), Sense::drag())
        .on_hover_cursor(egui::CursorIcon::Grab);
    let mut out = None;
    if body.dragged() || body.drag_stopped() {
        let r = hud::clamp_into(free.translate(body.drag_delta()), screen);
        out = Some((r, body.drag_stopped()));
    }
    let corners = [
        (
            free.left_top(),
            free.right_bottom(),
            egui::CursorIcon::ResizeNwSe,
        ),
        (
            free.right_top(),
            free.left_bottom(),
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            free.left_bottom(),
            free.right_top(),
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            free.right_bottom(),
            free.left_top(),
            egui::CursorIcon::ResizeNwSe,
        ),
    ];
    for (k, (corner, opposite, cursor)) in corners.into_iter().enumerate() {
        let g = Rect::from_center_size(corner, Vec2::splat(GRIP));
        let resp = ui
            .interact(
                g,
                ui.make_persistent_id(("hud-free-grip", k)),
                Sense::drag(),
            )
            .on_hover_cursor(cursor);
        ui.painter().rect(
            g.shrink(3.0),
            radius::SM,
            color::POSITIVE,
            Stroke::new(1.0, Color32::from_black_alpha(160)),
            egui::StrokeKind::Inside,
        );
        if resp.dragged() || resp.drag_stopped() {
            let to = corner + resp.drag_delta();
            let mut r = Rect::from_two_pos(opposite, to).intersect(screen);
            if r.width() < MIN || r.height() < MIN {
                r = Rect::from_min_size(r.min, r.size().max(Vec2::splat(MIN)));
            }
            out = Some((r, resp.drag_stopped()));
        }
    }
    out
}

/// The left (or top) edge that snaps `lo..lo+len` to the screen edges and to `others`.
fn snap_axis(lo: f32, len: f32, edges: &[(f32, f32)], min: f32, max: f32) -> f32 {
    let mut candidates = vec![min + GAP, max - GAP - len];
    for &(a, b) in edges {
        // Aligned with the other panel's edges, or beside it.
        candidates.extend([a, b - len, b + GAP, a - GAP - len]);
    }
    candidates
        .into_iter()
        .filter(|c| (c - lo).abs() <= SNAP)
        .min_by(|a, b| (a - lo).abs().total_cmp(&(b - lo).abs()))
        .unwrap_or(lo)
}

pub fn snap(r: Rect, others: &[Rect], screen: Rect) -> Rect {
    let xs: Vec<(f32, f32)> = others.iter().map(|o| (o.left(), o.right())).collect();
    let ys: Vec<(f32, f32)> = others.iter().map(|o| (o.top(), o.bottom())).collect();
    let x = snap_axis(r.left(), r.width(), &xs, screen.left(), screen.right());
    let y = snap_axis(r.top(), r.height(), &ys, screen.top(), screen.bottom());
    hud::clamp_into(Rect::from_min_size(pos2(x, y), r.size()), screen)
}

fn hatch(ui: &Ui, r: Rect, tint: Color32) {
    let p = ui.painter().with_clip_rect(r);
    let step = 14.0;
    let mut x = r.left() - r.height();
    while x < r.right() {
        p.line_segment(
            [pos2(x, r.bottom()), pos2(x + r.height(), r.top())],
            Stroke::new(1.0, tint),
        );
        x += step;
    }
    ui.painter().rect_stroke(
        r,
        radius::MD,
        Stroke::new(2.0, tint),
        egui::StrokeKind::Inside,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    config: &std::path::Path,
    hud_state: &mut Hud,
    sizes: &BTreeMap<PanelId, Vec2>,
    edit: &mut Edit,
) -> Response {
    let mut out = Response::default();
    let resolved = hud_state.resolved(screen, sizes).clone();
    // A veil over the map; it takes every click the panels and the toolbar do not.
    ui.painter()
        .rect_filled(screen, 0, Color32::from_black_alpha(110));
    let veil = ui.interact(
        screen,
        ui.make_persistent_id("hud-edit-veil"),
        Sense::click_and_drag(),
    );

    // The free area, and marking a new one by dragging over the map.
    if edit.marking {
        if veil.drag_started() {
            edit.mark_from = veil.interact_pointer_pos();
        }
        if let (Some(a), Some(b)) = (edit.mark_from, ui.input(|i| i.pointer.interact_pos())) {
            let r = Rect::from_two_pos(a, b);
            hatch(ui, r, color::POSITIVE);
            if veil.drag_stopped() {
                if r.width() >= 40.0 && r.height() >= 40.0 {
                    hud::set_free_area(hud_state, Some(Area::of_rect(r, screen)));
                    out.save = true;
                }
                edit.marking = false;
                edit.mark_from = None;
            }
        }
    }
    if let Some(free) = resolved.free {
        hatch(ui, free, color::POSITIVE.gamma_multiply(0.8));
        ui.painter().text(
            free.center(),
            Align2::CENTER_CENTER,
            i18n.tr("hud-free-label"),
            sans_semibold(size::BODY),
            color::POSITIVE,
        );
        if !edit.marking
            && let Some(moved) = free_area_grips(ui, free, screen)
        {
            hud::set_free_area(hud_state, Some(Area::of_rect(moved.0, screen)));
            out.save |= moved.1;
        }
    }

    // Panels: outline, handle, drag.
    let handles = handles(&resolved.rects, screen);
    for (&p, &r) in &resolved.rects {
        let id = ui.make_persistent_id(("hud-edit-panel", p.key()));
        let handle = handles[&p];
        let resp = ui.interact(r.union(handle), id, Sense::drag());
        let dragging = edit.drag.is_some_and(|(d, _)| d == p);
        let outline = if dragging || resp.hovered() {
            color::ACCENT_HOVER
        } else {
            color::ACCENT
        };
        ui.painter()
            .rect_filled(r, radius::LG, Color32::from_black_alpha(70));
        ui.painter().rect_stroke(
            r,
            radius::LG,
            Stroke::new(2.0, outline),
            egui::StrokeKind::Outside,
        );
        ui.painter()
            .rect_filled(handle, radius::MD, color::ACCENT.gamma_multiply(0.9));
        let name = i18n.tr(p.label());
        ui.painter().text(
            handle.left_center() + vec2(space::SM, 0.0),
            Align2::LEFT_CENTER,
            format!(":: {name}"),
            sans_semibold(size::SMALL),
            color::ON_ACCENT,
        );
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name.clone()));
        // Hide, at the right end of the handle.
        let hide_text = i18n.tr("hud-hide");
        let galley = ui.painter().layout_no_wrap(
            hide_text.clone(),
            sans_semibold(size::SMALL),
            color::ON_ACCENT,
        );
        let hide = Rect::from_min_size(
            pos2(
                handle.right() - galley.size().x - space::MD * 2.0,
                handle.top() + 2.0,
            ),
            vec2(galley.size().x + space::MD * 2.0, HANDLE_H - 4.0),
        );
        let hide_resp = ui.interact(hide, id.with("hide"), Sense::click());
        hide_resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{hide_text} {name}"))
        });
        ui.painter().rect_filled(
            hide,
            radius::SM,
            if hide_resp.hovered() {
                Color32::from_black_alpha(60)
            } else {
                Color32::from_black_alpha(30)
            },
        );
        ui.painter().galley(
            hide.center() - galley.size() / 2.0,
            galley,
            color::ON_ACCENT,
        );
        if hide_resp.clicked() {
            hud::toggle_panel(hud_state, p);
            out.save = true;
        }
        let resp = resp.on_hover_cursor(egui::CursorIcon::Grab);
        if resp.drag_started()
            && let Some(at) = resp.interact_pointer_pos()
        {
            edit.drag = Some((p, at - r.min));
        }
        if let Some((d, grab)) = edit.drag
            && d == p
        {
            if let Some(at) = ui.input(|i| i.pointer.interact_pos()) {
                let size = sizes.get(&p).copied().unwrap_or(r.size());
                let others: Vec<Rect> = resolved
                    .rects
                    .iter()
                    .filter(|(o, _)| **o != p)
                    .map(|(_, r)| *r)
                    .chain(resolved.free)
                    .collect();
                let moved = snap(Rect::from_min_size(at - grab, size), &others, screen);
                hud_state
                    .layout
                    .set_place(p, Placement::of_rect(moved, screen, true));
                hud_state.changed();
            }
            if resp.drag_stopped() || !ui.input(|i| i.pointer.any_down()) {
                edit.drag = None;
                out.save = true;
            }
        }
    }

    toolbar(ui, screen, i18n, config, hud_state, edit, &mut out);
    out
}

fn toolbar(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    config: &std::path::Path,
    hud_state: &mut Hud,
    edit: &mut Edit,
    out: &mut Response,
) {
    let width = TOOLBAR_W.min(screen.width() - 32.0);
    let id = egui::Id::NULL.with("bf-hud-toolbar");
    let height: f32 = ui.ctx().data(|d| d.get_temp(id)).unwrap_or(260.0);
    // Centred at first; dragged by its background out of the way of a panel.
    let rect = hud::clamp_into(
        Rect::from_center_size(screen.center() + edit.toolbar, vec2(width, height)),
        screen,
    );
    bf_ui_kit::paint_overlay(ui, rect);
    let bg = ui
        .interact(rect, id.with("block"), Sense::click_and_drag())
        .on_hover_cursor(egui::CursorIcon::Grab);
    if bg.dragged() {
        edit.toolbar += bg.drag_delta();
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(space::LG)));
    let ui = &mut child;
    ui.spacing_mut().item_spacing = vec2(space::SM, space::SM);
    let top = ui.cursor().top();
    ui.horizontal(|ui| {
        bf_ui_kit::heading(ui, &i18n.tr("hud-edit-title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::primary(&i18n.tr("hud-done"))
                .size(ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                toggle(edit);
            }
        });
    });
    bf_ui_kit::small(ui, &i18n.tr("hud-edit-hint"));

    bf_ui_kit::caption(ui, &i18n.tr("hud-presets"));
    let mut preset = hud_state.layout.preset;
    let options: Vec<(Preset, String)> = Preset::CHOICES
        .iter()
        .map(|p| (*p, i18n.tr(p.label())))
        .collect();
    if bf_ui_kit::segmented(ui, &mut preset, &options) {
        hud::apply_preset(hud_state, preset);
        out.save = true;
    }
    if hud_state.layout.preset == Preset::Custom {
        bf_ui_kit::small(ui, &i18n.tr("hud-preset-custom"));
    }

    bf_ui_kit::caption(ui, &i18n.tr("hud-panels"));
    ui.horizontal_wrapped(|ui| {
        for p in PanelId::ALL {
            let shown = hud_state.layout.visible(p);
            let label = i18n.tr(p.label());
            let b = if shown {
                Button::secondary(&label)
            } else {
                Button::ghost(&label)
            };
            let tip = i18n.tr(if shown {
                "hud-tip-shown"
            } else {
                "hud-tip-hidden"
            });
            if b.size(ButtonSize::Small)
                .show(ui)
                .on_hover_text(bf_ui_kit::tooltip_text(&tip))
                .clicked()
            {
                hud::toggle_panel(hud_state, p);
                out.save = true;
            }
        }
    });

    bf_ui_kit::caption(ui, &i18n.tr("hud-free"));
    ui.horizontal_wrapped(|ui| {
        let mark = i18n.tr(if edit.marking {
            "hud-free-marking"
        } else {
            "hud-free-mark"
        });
        if Button::secondary(&mark)
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
        {
            edit.marking = !edit.marking;
            edit.mark_from = None;
        }
        if hud_state.layout.free.is_some()
            && Button::ghost(&i18n.tr("hud-free-clear"))
                .size(ButtonSize::Small)
                .show(ui)
                .clicked()
        {
            hud::set_free_area(hud_state, None);
            out.save = true;
        }
    });

    ui.horizontal_wrapped(|ui| {
        if Button::ghost(&i18n.tr("hud-export"))
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
        {
            out.message = Some(match hud::export(hud_state, config) {
                Ok(path) => i18n.tr_args(
                    "hud-exported",
                    &[("path", Arg::String(path.display().to_string()))],
                ),
                Err(e) => i18n.tr_args("hud-io-error", &[("error", Arg::String(e.to_string()))]),
            });
        }
        if Button::ghost(&i18n.tr("hud-import"))
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
        {
            out.message = Some(match hud::import(hud_state, config) {
                Ok(path) => {
                    out.save = true;
                    i18n.tr_args(
                        "hud-imported",
                        &[("path", Arg::String(path.display().to_string()))],
                    )
                }
                Err(e) => i18n.tr_args("hud-io-error", &[("error", Arg::String(e))]),
            });
        }
        if Button::ghost(&i18n.tr("hud-reset"))
            .size(ButtonSize::Small)
            .show(ui)
            .clicked()
        {
            hud::reset(hud_state);
            out.save = true;
        }
    });
    let used = ui.min_rect().bottom() - top + space::LG * 2.0;
    ui.ctx().data_mut(|d| d.insert_temp(id, used));
}
