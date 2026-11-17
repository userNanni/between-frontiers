//! The sea lens' data: the sea zones of `assets/map/seas.png` (a zone id per water pixel) and
//! the route graph of `assets/map/seas.ron` (`bf_sim::sea`), with what the interface draws from
//! them, worked out once: a tinted overlay with the zones' outlines and their ice, and where each
//! strait or canal sits on the map.
//!
//! Nothing here is part of the game: the graph is not in the tick yet, and a route drawn on the
//! lens closes nothing.

use std::path::Path;
use std::sync::Arc;

use bf_sim::sea::{Ice, SeaGraph, SeasFile};
use egui::{Color32, ColorImage, Pos2, TextureHandle, TextureOptions};

/// The overlay is the zone map shrunk by this factor on each side: outlines and hatching read
/// at any zoom the camera allows, at a quarter of the memory.
pub const SHRINK: u32 = 2;

/// The sea zones and the graph, as the lens draws them.
pub struct SeaLayer {
    pub graph: Arc<SeaGraph>,
    /// Zone id per texel of the map (0: land or lake), full size.
    ids: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Tints, outlines and ice, `SHRINK` times smaller than the map.
    pub overlay: ColorImage,
    /// Where each strait or canal sits, in map texels, by strait id.
    pub straits: Vec<Pos2>,
}

/// Why the layer could not load.
pub fn load(assets: &Path) -> Result<SeaLayer, String> {
    let text = std::fs::read_to_string(assets.join("map/seas.ron"))
        .map_err(|e| format!("seas.ron: {e}"))?;
    let file: SeasFile = ron::from_str(&text).map_err(|e| format!("seas.ron: {e}"))?;
    let graph = SeaGraph::new(file)?;
    let (width, height, ids) = read_zone_ids(&assets.join("map/seas.png"))?;
    Ok(SeaLayer::new(graph, width, height, ids, |_| None))
}

fn read_zone_ids(path: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::IDENTITY);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let info = reader.info();
    if info.bit_depth != png::BitDepth::Eight || info.color_type != png::ColorType::Grayscale {
        return Err(format!(
            "{}: expected 8-bit grayscale, got {:?} {:?}",
            path.display(),
            info.bit_depth,
            info.color_type
        ));
    }
    let (w, h) = (info.width, info.height);
    let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
    reader
        .next_frame(&mut buf)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    buf.truncate((w * h) as usize);
    Ok((w, h, buf))
}

/// A quiet tint per zone, by its id: neighbours differ, no zone shouts.
pub fn zone_tint(id: u8) -> Color32 {
    const TINTS: [[u8; 3]; 6] = [
        [70, 120, 160],
        [60, 140, 150],
        [90, 110, 170],
        [60, 130, 130],
        [100, 130, 165],
        [75, 105, 145],
    ];
    let t = TINTS[(id as usize * 7) % TINTS.len()];
    Color32::from_rgba_unmultiplied(t[0], t[1], t[2], 70)
}

impl SeaLayer {
    /// `state_center` gives a state's label anchor in texels (`None` when unknown); the layer
    /// uses it to put the straits beside their shores.
    pub fn new(
        graph: SeaGraph,
        width: u32,
        height: u32,
        ids: Vec<u8>,
        state_center: impl Fn(u16) -> Option<Pos2>,
    ) -> Self {
        let overlay = overlay(&graph, width, height, &ids);
        let mut layer = Self {
            graph: Arc::new(graph),
            ids,
            width,
            height,
            overlay,
            straits: Vec::new(),
        };
        layer.place_straits(state_center);
        layer
    }

    /// The zone under a texel, 0 for land.
    pub fn zone_at(&self, texel: Pos2) -> u8 {
        let (x, y) = (texel.x.floor() as i64, texel.y.floor() as i64);
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return 0;
        }
        self.ids[(y as u64 * self.width as u64 + x as u64) as usize]
    }

    /// Puts each strait on the water of its two zones nearest the middle of its shores, or the
    /// middle of the two zones' centres when the shores are not known.
    pub fn place_straits(&mut self, state_center: impl Fn(u16) -> Option<Pos2>) {
        let zones = &self.graph.file.zones;
        let center = |z: u16| {
            zones
                .get(z as usize - 1)
                .map_or(Pos2::ZERO, |z| Pos2::new(z.center.0, z.center.1))
        };
        self.straits = self
            .graph
            .file
            .straits
            .iter()
            .map(|s| {
                let shores: Vec<Pos2> =
                    s.shores.iter().filter_map(|&st| state_center(st)).collect();
                let target = if shores.is_empty() {
                    let (a, b) = (center(s.a), center(s.b));
                    Pos2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
                } else {
                    let n = shores.len() as f32;
                    Pos2::new(
                        shores.iter().map(|p| p.x).sum::<f32>() / n,
                        shores.iter().map(|p| p.y).sum::<f32>() / n,
                    )
                };
                self.nearest_water(target, [s.a as u8, s.b as u8])
                    .unwrap_or(target)
            })
            .collect();
    }

    /// The pixel of zone `a` or `b` nearest `at`, within a few hundred texels.
    fn nearest_water(&self, at: Pos2, zones: [u8; 2]) -> Option<Pos2> {
        let (cx, cy) = (at.x.round() as i64, at.y.round() as i64);
        let mut best: Option<(i64, Pos2)> = None;
        for r in (0..=320i64).step_by(2) {
            if best.is_some_and(|(d, _)| d <= r * r) {
                break;
            }
            for dy in (-r..=r).step_by(2) {
                for dx in [-r, r] {
                    for (x, y) in [(cx + dx, cy + dy), (cx + dy, cy + dx)] {
                        let z = self.zone_at(Pos2::new(x as f32, y as f32));
                        if z != 0 && zones.contains(&z) {
                            let d = (x - cx).pow(2) + (y - cy).pow(2);
                            if best.is_none_or(|(b, _)| d < b) {
                                best = Some((d, Pos2::new(x as f32 + 0.5, y as f32 + 0.5)));
                            }
                        }
                    }
                }
            }
        }
        best.map(|(_, p)| p)
    }

    /// The overlay as textures of `ctx`, made once: tiles no larger than the GPU takes, each
    /// with the part of the map it covers, in texels.
    pub fn textures(&self, ctx: &egui::Context) -> Vec<(TextureHandle, egui::Rect)> {
        let side = ctx.input(|i| i.max_texture_side).clamp(256, 4096);
        let [w, h] = self.overlay.size;
        let mut out = Vec::new();
        for y0 in (0..h).step_by(side) {
            for x0 in (0..w).step_by(side) {
                let (tw, th) = ((w - x0).min(side), (h - y0).min(side));
                let mut pixels = Vec::with_capacity(tw * th);
                for y in y0..y0 + th {
                    pixels.extend_from_slice(&self.overlay.pixels[y * w + x0..y * w + x0 + tw]);
                }
                let tile = ColorImage::new([tw, th], pixels);
                let tex = ctx.load_texture(
                    format!("sea-overlay-{x0}-{y0}"),
                    tile,
                    TextureOptions::LINEAR,
                );
                let s = SHRINK as f32;
                let rect = egui::Rect::from_min_size(
                    Pos2::new(x0 as f32 * s, y0 as f32 * s),
                    egui::vec2(tw as f32 * s, th as f32 * s),
                );
                out.push((tex, rect));
            }
        }
        out
    }
}

/// Tints, outlines between zones, and ice: diagonal hatching for seasonal ice, a pale wash with
/// denser hatching for permanent ice.
fn overlay(graph: &SeaGraph, width: u32, height: u32, ids: &[u8]) -> ColorImage {
    let (w, h) = ((width / SHRINK) as usize, (height / SHRINK) as usize);
    let at = |x: usize, y: usize| -> u8 {
        let (fx, fy) = (
            (x * SHRINK as usize + SHRINK as usize / 2).min(width as usize - 1),
            (y * SHRINK as usize + SHRINK as usize / 2).min(height as usize - 1),
        );
        ids[fy * width as usize + fx]
    };
    let ice = |z: u8| {
        graph
            .file
            .zones
            .get((z as usize).wrapping_sub(1))
            .map_or(Ice::Free, |z| z.ice)
    };
    let mut pixels = vec![Color32::TRANSPARENT; w * h];
    for y in 0..h {
        for x in 0..w {
            let z = at(x, y);
            if z == 0 {
                continue;
            }
            // An outline where another zone touches (land does not draw one: the coast does).
            let edge = [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dy)| {
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        return false;
                    }
                    let n = at(nx as usize, ny as usize);
                    n != 0 && n != z
                });
            let mut c = zone_tint(z);
            match ice(z) {
                Ice::Free => {}
                Ice::Seasonal => {
                    if (x + y) % 6 == 0 {
                        c = Color32::from_rgba_unmultiplied(225, 235, 245, 130);
                    }
                }
                Ice::Permanent => {
                    c = if (x + y) % 3 == 0 {
                        Color32::from_rgba_unmultiplied(235, 242, 250, 170)
                    } else {
                        Color32::from_rgba_unmultiplied(200, 215, 230, 110)
                    };
                }
            }
            if edge {
                c = Color32::from_rgba_unmultiplied(20, 35, 55, 200);
            }
            pixels[y * w + x] = c;
        }
    }
    ColorImage::new([w, h], pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layer_loads_and_every_strait_sits_on_its_water() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layer = load(&root).expect("sea layer");
        assert_eq!(layer.graph.file.zones.len(), 63);
        assert_eq!(layer.straits.len(), layer.graph.file.straits.len());
        for (s, p) in layer.graph.file.straits.iter().zip(&layer.straits) {
            let z = layer.zone_at(*p);
            assert!(
                z as u16 == s.a || z as u16 == s.b,
                "{} at {p:?} on zone {z}",
                s.key
            );
        }
        // Zone ids of the picture match the file.
        let biggest = layer
            .graph
            .file
            .zones
            .iter()
            .max_by_key(|z| z.pixels)
            .unwrap();
        assert!(biggest.pixels > 0);
    }
}
