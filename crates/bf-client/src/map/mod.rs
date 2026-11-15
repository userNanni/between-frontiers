//! The political map: data prepared on the CPU, drawn by one shader, plus camera, picking and
//! country labels painted by egui on top.

pub mod gpu;

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use egui::{Align2, Color32, Pos2, Rect, Sense, Ui, Vec2};

use crate::world::World;

/// Quarter-resolution distance fields for coast and border shading.
const FIELD_DIVISOR: u32 = 4;
/// Distance, in field cells, that maps to the field's full value.
const FIELD_RANGE: f32 = 40.0;

/// Map data kept on the CPU: the id texture for picking, adjacency for labels.
pub struct MapData {
    pub width: u32,
    pub height: u32,
    pub ids: Vec<u16>,
    pub province_count: u32,
    /// Owner index of each province id (index 0 = water). Owner index is country index + 1.
    pub owner_of: Vec<u16>,
    /// Whether each province id is a colony of its owner.
    pub colony_of: Vec<bool>,
    /// Political palette, by province id.
    pub palette: Vec<[u8; 4]>,
    pub field_width: u32,
    pub field_height: u32,
    /// RG8: r = sea distance to land, g = land distance to a country border.
    pub fields: Vec<u8>,
    pub labels: Vec<Label>,
}

/// A country name placed over one contiguous block of its provinces.
#[derive(Clone, Debug)]
pub struct Label {
    pub country: usize,
    /// Anchor in texels.
    pub at: Pos2,
    /// Rough extent in texels, for sizing the text.
    pub extent: Vec2,
}

#[derive(Debug)]
pub enum LoadError {
    Io(String),
    Format(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) | LoadError::Format(e) => f.write_str(e),
        }
    }
}

/// Reads a 16-bit grayscale PNG into ids.
pub fn read_ids(path: &Path) -> Result<(u32, u32, Vec<u16>), LoadError> {
    let file =
        std::fs::File::open(path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|e| LoadError::Format(format!("{}: {e}", path.display())))?;
    let info = reader.info();
    if info.bit_depth != png::BitDepth::Sixteen || info.color_type != png::ColorType::Grayscale {
        return Err(LoadError::Format(format!(
            "{}: expected 16-bit grayscale, got {:?} {:?}",
            path.display(),
            info.bit_depth,
            info.color_type
        )));
    }
    let (w, h) = (info.width, info.height);
    let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
    reader
        .next_frame(&mut buf)
        .map_err(|e| LoadError::Format(format!("{}: {e}", path.display())))?;
    let ids = buf
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_be_bytes(*b))
        .collect();
    Ok((w, h, ids))
}

impl MapData {
    pub fn build(world: &World, width: u32, height: u32, ids: Vec<u16>) -> Self {
        let province_count = world.provinces.len() as u32;
        let mut owner_of = vec![0u16; province_count as usize + 1];
        let mut colony_of = vec![false; province_count as usize + 1];
        for p in &world.provinces {
            owner_of[p.def.id as usize] = p.owner.map_or(0, |c| c as u16 + 1);
            colony_of[p.def.id as usize] = p.def.colony;
        }
        let palette = political_palette(world);
        let (field_width, field_height, fields) = distance_fields(width, height, &ids, &owner_of);
        let labels = place_labels(world, width, height, &ids, &owner_of);
        Self {
            width,
            height,
            ids,
            province_count,
            owner_of,
            colony_of,
            palette,
            field_width,
            field_height,
            fields,
            labels,
        }
    }

    pub fn id_at(&self, texel: Pos2) -> u16 {
        if texel.x < 0.0 || texel.y < 0.0 {
            return 0;
        }
        let (x, y) = (texel.x as u32, texel.y as u32);
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.ids[(y * self.width + x) as usize]
    }
}

pub fn political_palette(world: &World) -> Vec<[u8; 4]> {
    let mut palette = vec![[0, 0, 0, 255]; world.provinces.len() + 1];
    for p in &world.provinces {
        let c = p
            .owner
            .map(|c| world.countries[c].def.color)
            .unwrap_or((120, 112, 98));
        palette[p.def.id as usize] = [c.0, c.1, c.2, 255];
    }
    palette
}

/// Two-pass chamfer distance on a downsampled grid.
fn distance_fields(width: u32, height: u32, ids: &[u16], owner_of: &[u16]) -> (u32, u32, Vec<u8>) {
    let fw = width.div_ceil(FIELD_DIVISOR);
    let fh = height.div_ceil(FIELD_DIVISOR);
    let n = (fw * fh) as usize;
    let owner_at = |fx: u32, fy: u32| -> u16 {
        let x = (fx * FIELD_DIVISOR + FIELD_DIVISOR / 2).min(width - 1);
        let y = (fy * FIELD_DIVISOR + FIELD_DIVISOR / 2).min(height - 1);
        let id = ids[(y * width + x) as usize];
        if id == 0 {
            return 0;
        }
        match owner_of.get(id as usize).copied().unwrap_or(0) {
            0 => u16::MAX,
            o => o,
        }
    };
    let mut owner = vec![0u16; n];
    for fy in 0..fh {
        for fx in 0..fw {
            owner[(fy * fw + fx) as usize] = owner_at(fx, fy);
        }
    }
    // Seeds: sea cells next to land (for r) and land cells next to another owner (for g).
    let big = f32::MAX / 4.0;
    let mut sea = vec![big; n];
    let mut border = vec![big; n];
    for fy in 0..fh {
        for fx in 0..fw {
            let i = (fy * fw + fx) as usize;
            let me = owner[i];
            if me == 0 {
                // Sea distance counts from land cells.
                continue;
            }
            sea[i] = 0.0;
            let mut edge = false;
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (fx as i32 + dx, fy as i32 + dy);
                if nx < 0 || ny < 0 || nx >= fw as i32 || ny >= fh as i32 {
                    continue;
                }
                if owner[(ny as u32 * fw + nx as u32) as usize] != me {
                    edge = true;
                }
            }
            if edge {
                border[i] = 0.0;
            }
        }
    }
    chamfer(&mut sea, fw, fh);
    chamfer(&mut border, fw, fh);
    let mut out = vec![0u8; n * 2];
    for i in 0..n {
        let q = |d: f32| ((d / FIELD_RANGE).clamp(0.0, 1.0) * 255.0).round() as u8;
        out[i * 2] = if owner[i] == 0 { q(sea[i]) } else { 0 };
        out[i * 2 + 1] = if owner[i] == 0 { 255 } else { q(border[i]) };
    }
    (fw, fh, out)
}

fn chamfer(d: &mut [f32], w: u32, h: u32) {
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    let w = w as usize;
    let h = h as usize;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut v = d[i];
            if x > 0 {
                v = v.min(d[i - 1] + a);
            }
            if y > 0 {
                v = v.min(d[i - w] + a);
                if x > 0 {
                    v = v.min(d[i - w - 1] + b);
                }
                if x + 1 < w {
                    v = v.min(d[i - w + 1] + b);
                }
            }
            d[i] = v;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            let mut v = d[i];
            if x + 1 < w {
                v = v.min(d[i + 1] + a);
            }
            if y + 1 < h {
                v = v.min(d[i + w] + a);
                if x + 1 < w {
                    v = v.min(d[i + w + 1] + b);
                }
                if x > 0 {
                    v = v.min(d[i + w - 1] + b);
                }
            }
            d[i] = v;
        }
    }
}

/// One label per contiguous block of a country's provinces that is large enough to read.
fn place_labels(
    world: &World,
    width: u32,
    height: u32,
    ids: &[u16],
    owner_of: &[u16],
) -> Vec<Label> {
    let pc = owner_of.len();
    // Adjacency between provinces of the same owner, from the texture.
    let mut adjacency: BTreeSet<(u16, u16)> = BTreeSet::new();
    let w = width as usize;
    for y in 0..height as usize {
        let row = &ids[y * w..(y + 1) * w];
        for x in 0..w {
            let a = row[x];
            if a == 0 {
                continue;
            }
            let mut link = |b: u16| {
                if b != 0 && b != a && owner_of[a as usize] == owner_of[b as usize] {
                    adjacency.insert((a.min(b), a.max(b)));
                }
            };
            if x + 1 < w {
                link(row[x + 1]);
            }
            if y + 1 < height as usize {
                link(ids[(y + 1) * w + x]);
            }
        }
    }
    // Union-find over provinces.
    let mut parent: Vec<usize> = (0..pc).collect();
    fn find(p: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while p[r] != r {
            p[r] = p[p[r]];
            r = p[r];
        }
        r
    }
    for (a, b) in adjacency {
        let (ra, rb) = (find(&mut parent, a as usize), find(&mut parent, b as usize));
        if ra != rb {
            parent[ra.max(rb)] = ra.min(rb);
        }
    }
    struct Acc {
        owner: u16,
        pixels: f64,
        sx: f64,
        sy: f64,
        min: Pos2,
        max: Pos2,
    }
    let mut groups: std::collections::BTreeMap<usize, Acc> = Default::default();
    for p in &world.provinces {
        let id = p.def.id as usize;
        let owner = owner_of[id];
        if owner == 0 || p.def.pixels == 0 {
            continue;
        }
        let root = find(&mut parent, id);
        let px = p.def.pixels as f64;
        let c = Pos2::new(p.def.center.0, p.def.center.1);
        let half = (px.sqrt() as f32) * 0.5;
        let e = groups.entry(root).or_insert(Acc {
            owner,
            pixels: 0.0,
            sx: 0.0,
            sy: 0.0,
            min: c,
            max: c,
        });
        e.pixels += px;
        e.sx += c.x as f64 * px;
        e.sy += c.y as f64 * px;
        e.min = e.min.min(c - Vec2::splat(half));
        e.max = e.max.max(c + Vec2::splat(half));
    }
    let mut labels: Vec<Label> = groups
        .into_values()
        .filter(|g| g.pixels >= 900.0)
        .map(|g| {
            let mut at = Pos2::new((g.sx / g.pixels) as f32, (g.sy / g.pixels) as f32);
            // A weighted centre can fall in the sea for curved shapes; nudge it onto the owner.
            if owner_of[ids_at(ids, width, height, at) as usize] != g.owner {
                at = Pos2::new((g.min.x + g.max.x) / 2.0, (g.min.y + g.max.y) / 2.0);
            }
            Label {
                country: g.owner as usize - 1,
                at,
                extent: (g.max - g.min).max(Vec2::splat((g.pixels.sqrt()) as f32)),
            }
        })
        .collect();
    labels.sort_by(|a, b| b.extent.x.total_cmp(&a.extent.x));
    labels
}

fn ids_at(ids: &[u16], width: u32, height: u32, at: Pos2) -> u16 {
    let x = (at.x.max(0.0) as u32).min(width - 1);
    let y = (at.y.max(0.0) as u32).min(height - 1);
    ids[(y * width + x) as usize]
}

/// Camera over the map texture.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Texel at the centre of the view.
    pub center: Pos2,
    /// Screen points per texel.
    pub zoom: f32,
}

impl Camera {
    pub fn fit(map: &MapData, view: Vec2) -> Self {
        Self {
            center: Pos2::new(map.width as f32 / 2.0, map.height as f32 * 0.42),
            zoom: (view.x / map.width as f32).max(view.y / map.height as f32),
        }
    }

    fn min_zoom(map: &MapData, view: Vec2) -> f32 {
        (view.x / map.width as f32).max(view.y / map.height as f32)
    }

    pub fn clamp(&mut self, map: &MapData, view: Vec2) {
        self.zoom = self.zoom.clamp(Self::min_zoom(map, view), 6.0);
        let half = view / self.zoom / 2.0;
        self.center.x = self
            .center
            .x
            .clamp(half.x, (map.width as f32 - half.x).max(half.x));
        self.center.y = self
            .center
            .y
            .clamp(half.y, (map.height as f32 - half.y).max(half.y));
    }

    pub fn to_texel(&self, rect: Rect, screen: Pos2) -> Pos2 {
        self.center + (screen - rect.center()) / self.zoom
    }

    pub fn to_screen(&self, rect: Rect, texel: Pos2) -> Pos2 {
        rect.center() + (texel - self.center) * self.zoom
    }

    /// Moves to show `texel` at the centre at `zoom`.
    pub fn look_at(&mut self, texel: Pos2, zoom: f32) {
        self.center = texel;
        self.zoom = zoom;
    }
}

/// What happened on the map this frame.
#[derive(Default)]
pub struct MapResponse {
    pub hovered_province: Option<u16>,
    pub clicked_province: Option<u16>,
    pub clicked_sea: bool,
}

/// How the map is used on the current screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MapMode {
    /// Slowly drifting, darkened, no interaction: the main menu background.
    Backdrop,
    Interactive,
}

pub struct MapView {
    pub camera: Option<Camera>,
    pub hovered_owner: Option<usize>,
    pub selected_owner: Option<usize>,
    pub show_labels: bool,
    palette_dirty: Option<Arc<Vec<[u8; 4]>>>,
}

impl MapView {
    pub fn new(map: &MapData) -> Self {
        Self {
            camera: None,
            hovered_owner: None,
            selected_owner: None,
            show_labels: true,
            palette_dirty: Some(Arc::new(map.palette.clone())),
        }
    }

    pub fn set_palette(&mut self, palette: Vec<[u8; 4]>) {
        self.palette_dirty = Some(Arc::new(palette));
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        map: &MapData,
        world: &World,
        names: &dyn Fn(usize) -> String,
        mode: MapMode,
    ) -> MapResponse {
        let sense = match mode {
            MapMode::Backdrop => Sense::hover(),
            MapMode::Interactive => Sense::click_and_drag(),
        };
        let response = ui.interact(rect, ui.make_persistent_id("map"), sense);
        let view = rect.size();
        let camera = self.camera.get_or_insert_with(|| {
            let mut c = Camera::fit(map, view);
            if mode == MapMode::Backdrop {
                c.zoom *= 1.6;
                c.center = Pos2::new(map.width as f32 * 0.52, map.height as f32 * 0.33);
            } else {
                c.zoom *= 1.15;
            }
            c
        });
        let mut out = MapResponse::default();

        match mode {
            MapMode::Backdrop => {
                let t = ui.input(|i| i.time) as f32;
                camera.center.x += ui.input(|i| i.stable_dt).min(0.1) * 6.0;
                camera.center.y += (t * 0.05).sin() * 0.02;
                ui.ctx().request_repaint();
            }
            MapMode::Interactive => {
                if response.dragged() {
                    camera.center -= response.drag_delta() / camera.zoom;
                }
                if let Some(pointer) = response.hover_pos() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll != 0.0 {
                        let before = camera.to_texel(rect, pointer);
                        camera.zoom *= (scroll * 0.0025).exp();
                        camera.clamp(map, view);
                        let after = camera.to_texel(rect, pointer);
                        camera.center += before - after;
                    }
                    let id = map.id_at(camera.to_texel(rect, pointer));
                    if id != 0 {
                        out.hovered_province = Some(id);
                    }
                }
                let pan = ui.input(|i| {
                    let mut v = Vec2::ZERO;
                    if i.key_down(egui::Key::ArrowLeft) || i.key_down(egui::Key::A) {
                        v.x -= 1.0;
                    }
                    if i.key_down(egui::Key::ArrowRight) || i.key_down(egui::Key::D) {
                        v.x += 1.0;
                    }
                    if i.key_down(egui::Key::ArrowUp) || i.key_down(egui::Key::W) {
                        v.y -= 1.0;
                    }
                    if i.key_down(egui::Key::ArrowDown) || i.key_down(egui::Key::S) {
                        v.y += 1.0;
                    }
                    v * i.stable_dt.min(0.1) * 700.0
                });
                if pan != Vec2::ZERO && !ui.ctx().egui_wants_keyboard_input() {
                    camera.center += pan / camera.zoom;
                    ui.ctx().request_repaint();
                }
                if response.clicked()
                    && let Some(p) = response.interact_pointer_pos()
                {
                    let id = map.id_at(camera.to_texel(rect, p));
                    if id == 0 {
                        out.clicked_sea = true;
                    } else {
                        out.clicked_province = Some(id);
                    }
                }
            }
        }
        camera.clamp(map, view);
        let camera = *camera;

        self.hovered_owner = out
            .hovered_province
            .and_then(|id| world.provinces.get(id as usize - 1))
            .and_then(|p| p.owner);
        if out.hovered_province.is_some() && mode == MapMode::Interactive {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let ppp = ui.ctx().pixels_per_point();
        let view_size = view / camera.zoom;
        let origin = camera.center - view_size / 2.0;
        let uniforms = gpu::Uniforms {
            view_origin: [origin.x, origin.y],
            view_size: [view_size.x, view_size.y],
            map_size: [map.width as f32, map.height as f32],
            texel_per_px: 1.0 / (camera.zoom * ppp),
            time: ui.input(|i| i.time) as f32,
            hovered: self.hovered_owner.map_or(0, |o| o as u32 + 1),
            selected: self.selected_owner.map_or(0, |o| o as u32 + 1),
            dim: if mode == MapMode::Backdrop { 0.35 } else { 0.0 },
            _pad: 0.0,
        };
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            gpu::MapCallback {
                uniforms,
                palette: self.palette_dirty.take(),
            },
        ));

        if self.show_labels {
            self.paint_labels(ui, rect, &camera, map, names, mode);
        }
        out
    }

    fn paint_labels(
        &self,
        ui: &Ui,
        rect: Rect,
        camera: &Camera,
        map: &MapData,
        names: &dyn Fn(usize) -> String,
        mode: MapMode,
    ) {
        let painter = ui.painter().with_clip_rect(rect);
        let alpha = if mode == MapMode::Backdrop { 90 } else { 200 };
        let mut taken: Vec<Rect> = Vec::new();
        for label in &map.labels {
            let name = names(label.country).to_uppercase();
            let chars = name.chars().count().max(3) as f32;
            let width_px = label.extent.x * camera.zoom * 0.72;
            let size = (width_px / (chars * 0.78))
                .min(label.extent.y * camera.zoom * 0.45)
                .min(44.0);
            if size < 9.0 {
                continue;
            }
            let pos = camera.to_screen(rect, label.at);
            if !rect.expand(200.0).contains(pos) {
                continue;
            }
            let mut job = egui::text::LayoutJob::default();
            job.append(
                &name,
                0.0,
                egui::TextFormat {
                    font_id: bf_ui_kit::theme::serif(size),
                    color: Color32::from_rgba_unmultiplied(22, 17, 12, alpha),
                    extra_letter_spacing: size * 0.18,
                    ..Default::default()
                },
            );
            let galley = painter.layout_job(job);
            let r = Align2::CENTER_CENTER.anchor_size(pos, galley.size());
            if taken.iter().any(|t| t.intersects(r)) {
                continue;
            }
            taken.push(r);
            painter.galley(r.min, galley, Color32::BLACK);
        }
    }
}
