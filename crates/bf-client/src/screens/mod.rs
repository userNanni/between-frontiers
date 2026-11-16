//! Screens. Each one draws with `bf_ui_kit` components only and returns a transition.

pub mod budget;
pub mod country_panel;
pub mod game;
pub mod government;
pub mod laws;
pub mod lens_panel;
pub mod menu;
pub mod money;
pub mod pick;
pub mod research;
pub mod saves;
pub mod settings;
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
