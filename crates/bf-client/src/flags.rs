//! Country flags: PNG files in `assets/flags`, loaded once as textures.

use std::collections::HashMap;
use std::path::Path;

use bf_ui_kit::tokens::{color, radius};
use egui::{Align2, Color32, Rect, Stroke, StrokeKind, TextureHandle, Ui};

use crate::world::{Country, World};

#[derive(Default)]
pub struct Flags {
    by_tag: HashMap<String, TextureHandle>,
}

impl Flags {
    /// Loads `assets/flags/<file>` for every country; the file is the country's `flag` field or
    /// `<TAG>.png`. Missing or unreadable files are skipped and drawn as a colour field instead.
    pub fn load(ctx: &egui::Context, dir: &Path, world: &World) -> Self {
        let mut by_tag = HashMap::new();
        for c in &world.countries {
            let file = c
                .def
                .flag
                .clone()
                .unwrap_or_else(|| format!("{}.png", c.tag()));
            let path = dir.join(&file);
            let Ok(img) = image::open(&path) else {
                continue;
            };
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            let handle = ctx.load_texture(
                format!("flag-{}", c.tag()),
                image,
                egui::TextureOptions::LINEAR,
            );
            by_tag.insert(c.tag().to_owned(), handle);
        }
        Self { by_tag }
    }

    pub fn get(&self, tag: &str) -> Option<&TextureHandle> {
        self.by_tag.get(tag)
    }

    pub fn len(&self) -> usize {
        self.by_tag.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_tag.is_empty()
    }
}

/// Paints a country's flag stretched into `rect` (flags are 3:2), with a thin frame. Without an
/// image, the map colour with the tag.
pub fn paint_flag(ui: &Ui, rect: Rect, flags: &Flags, country: &Country, frame: Color32) {
    let painter = ui.painter();
    match flags.get(country.tag()) {
        Some(tex) => {
            painter.image(
                tex.id(),
                rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            let (r, g, b) = country.def.color;
            painter.rect_filled(rect, radius::SM, Color32::from_rgb(r, g, b));
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                country.tag(),
                bf_ui_kit::theme::serif((rect.height() * 0.38).max(9.0)),
                Color32::from_rgba_unmultiplied(255, 248, 230, 210),
            );
        }
    }
    // Light sheen at the top and shade at the bottom, so flat flags read as cloth.
    let mut mesh = egui::Mesh::default();
    let top = Color32::from_white_alpha(22);
    let bottom = Color32::from_black_alpha(40);
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
    painter.rect_stroke(
        rect,
        radius::SM,
        Stroke::new(1.0, frame),
        StrokeKind::Inside,
    );
}

/// Flag widget of a given height in a layout.
pub fn flag(ui: &mut Ui, flags: &Flags, country: &Country, height: f32) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(height * 1.5, height), egui::Sense::hover());
    paint_flag(ui, rect, flags, country, color::BORDER_STRONG);
    response
}
