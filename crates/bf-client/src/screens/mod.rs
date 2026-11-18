//! Screens. Each one draws with `bf_ui_kit` components only and returns a transition.

pub mod budget;
pub mod country_panel;
pub mod demand;
pub mod forces;
pub mod game;
pub mod gov_ui;
pub mod government;
pub mod hud_edit;
pub mod influence;
pub mod journal;
pub mod keys_editor;
pub mod laws;
pub mod ledger;
pub mod lens_panel;
pub mod liberate;
pub mod menu;
pub mod money;
pub mod outliner;
pub mod pick;
pub mod population;
pub mod research;
pub mod saves;
pub mod sea_lens;
pub mod settings;
pub mod tiles;
pub mod top_bar;

use egui::{Rect, Ui, UiBuilder};

use crate::app::{Shared, Transition};
use crate::map::MapMode;

pub enum Screen {
    Menu,
    Settings(Box<Screen>),
    Saves(saves::State),
    Pick(pick::State),
    Game(Box<game::State>),
}

pub fn show(
    screen: &mut Screen,
    shared: &mut Shared,
    ui: &mut Ui,
    rect: Rect,
) -> Option<Transition> {
    match screen {
        Screen::Menu => menu::show(shared, ui, rect),
        Screen::Settings(back) => {
            // Settings open over whatever was behind them; only the map backdrop is redrawn.
            let backdrop = !matches!(**back, Screen::Game(_));
            if backdrop {
                backdrop_map(shared, ui, rect, 0.0);
            }
            let done = settings::show(shared, ui, rect);
            if done {
                let back = std::mem::replace(&mut **back, Screen::Menu);
                *screen = back;
                if let Some(v) = &mut shared.map_view {
                    v.camera = None;
                }
            }
            None
        }
        Screen::Saves(state) => saves::show(state, shared, ui, rect),
        Screen::Pick(state) => pick::show(state, shared, ui, rect),
        Screen::Game(state) => game::show(state, shared, ui, rect),
    }
}

/// The world map drifting behind menus, darkened toward the left where the menu sits.
pub fn backdrop_map(shared: &mut Shared, ui: &mut Ui, rect: Rect, extra_dim: f32) {
    let i18n = &shared.i18n;
    if let Some((map, view, world)) = match (&shared.map, &mut shared.map_view, &shared.world) {
        (Some(m), Some(v), Some(w)) => Some((m, v, w)),
        _ => None,
    } {
        let names = |c: usize| crate::text::country_name(i18n, world.countries[c].tag());
        view.show(ui, rect, map, world, &names, MapMode::Backdrop);
    }
    // Left-to-right shade so text on the left stays readable over any country colour.
    let mut mesh = egui::Mesh::default();
    let dark = bf_ui_kit::tokens::color::BACKDROP;
    let a = |alpha: f32| {
        egui::Color32::from_rgba_unmultiplied(dark.r(), dark.g(), dark.b(), (alpha * 255.0) as u8)
    };
    let mid = rect.left() + rect.width() * 0.62;
    for (x0, x1, c0, c1) in [
        (rect.left(), mid, a(0.92), a(0.25 + extra_dim)),
        (mid, rect.right(), a(0.25 + extra_dim), a(0.15 + extra_dim)),
    ] {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(egui::pos2(x0, rect.top()), c0);
        mesh.colored_vertex(egui::pos2(x1, rect.top()), c1);
        mesh.colored_vertex(egui::pos2(x1, rect.bottom()), c1);
        mesh.colored_vertex(egui::pos2(x0, rect.bottom()), c0);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    ui.painter().add(mesh);
}

/// A child Ui confined to `rect`.
pub fn region<R>(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope_builder(UiBuilder::new().max_rect(rect), add).inner
}

/// Rectangle of `size` centred in `outer`.
pub fn centered(outer: Rect, size: egui::Vec2) -> Rect {
    Rect::from_center_size(
        outer.center(),
        size.min(outer.size() - egui::vec2(32.0, 32.0)),
    )
}

/// Height of a window's title strip, the part it is moved by.
const WINDOW_TITLE_H: f32 = 64.0;
const WINDOW_GRIP: f32 = 18.0;

/// A window the player moves by its title strip: paints its background, blocks the map under
/// it and returns its rectangle. Call [`window_grip`] after the window's content.
pub fn window_frame(
    ui: &mut Ui,
    key: &str,
    default: Rect,
    screen: Rect,
    place: &mut crate::hud::WindowPlace,
    min: egui::Vec2,
) -> (Rect, bool) {
    let rect = place.rect(default, screen, min);
    crate::keys::cover(ui, rect);
    frame(ui, rect, key);
    ui.interact(
        rect,
        ui.make_persistent_id((key, "block")),
        egui::Sense::click_and_drag(),
    );
    let title = Rect::from_min_size(rect.min, egui::vec2(rect.width(), WINDOW_TITLE_H));
    let moved = ui
        .interact(
            title,
            ui.make_persistent_id((key, "move")),
            egui::Sense::drag(),
        )
        .on_hover_cursor(egui::CursorIcon::Grab);
    if moved.dragged() {
        let d = moved.drag_delta();
        place.offset = [place.offset[0] + d.x, place.offset[1] + d.y];
    }
    (rect, moved.drag_stopped())
}

/// The corner a window is resized from; true when a resize just ended.
pub fn window_grip(
    ui: &mut Ui,
    key: &str,
    rect: Rect,
    place: &mut crate::hud::WindowPlace,
    min: egui::Vec2,
) -> bool {
    let grip = Rect::from_min_max(rect.max - egui::Vec2::splat(WINDOW_GRIP), rect.max);
    let r = ui
        .interact(
            grip,
            ui.make_persistent_id((key, "resize")),
            egui::Sense::drag(),
        )
        .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
    let tint = if r.hovered() || r.dragged() {
        bf_ui_kit::tokens::color::ACCENT
    } else {
        bf_ui_kit::tokens::color::BORDER_STRONG
    };
    for k in [5.0, 10.0] {
        ui.painter().line_segment(
            [
                egui::pos2(rect.right() - 4.0 - k, rect.bottom() - 4.0),
                egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0 - k),
            ],
            egui::Stroke::new(1.2, tint),
        );
    }
    if r.dragged() {
        let size = (rect.size() + r.drag_delta()).max(min);
        place.size = Some([size.x, size.y]);
    }
    r.drag_stopped()
}

/// Paints an overlay panel and names it for accessibility as a window, so that checks (and
/// screen readers) know which elements it holds. `name` is stable, not shown.
pub fn frame(ui: &mut Ui, rect: Rect, name: &str) {
    bf_ui_kit::paint_overlay(ui, rect);
    mark_frame(ui, rect, name);
}

/// Names `rect` as a window for accessibility, for frames painted another way.
pub fn mark_frame(ui: &mut Ui, rect: Rect, name: &str) {
    let r = ui.interact(
        rect,
        ui.make_persistent_id(("bf-frame", name)),
        egui::Sense::hover(),
    );
    let name = name.to_owned();
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Window, true, name.clone()));
}
