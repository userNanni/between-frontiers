//! Development preview: provinces coloured by owner, with coast, country and province borders.
//! Written to `target-preview/`, never committed.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const WATER: [u8; 3] = [178, 196, 214];
const COAST: [u8; 3] = [70, 80, 90];
const COUNTRY_BORDER: [u8; 3] = [15, 15, 15];

const PALETTE_SIZE: usize = 24;

/// Palette entry `k`: hues a golden angle apart, cycling through three saturation and value
/// levels.
fn palette(k: usize) -> [u8; 3] {
    let hue = (k as f64 * 0.618_033_988_75).fract() * 360.0;
    let sat = [0.55, 0.80, 0.40][k % 3];
    let val = [0.92, 0.72, 0.82][(k / 3) % 3];
    hsv_to_rgb(hue, sat, val)
}

/// One colour per country, greedy in tag order, never the colour of a touching country already
/// coloured. `touching` holds pairs of tag indices `(low, high)`.
pub fn country_colors(count: usize, touching: &BTreeSet<(usize, usize)>) -> Vec<[u8; 3]> {
    let mut chosen: Vec<usize> = Vec::with_capacity(count);
    for t in 0..count {
        let used: BTreeSet<usize> = (0..t)
            .filter(|&o| touching.contains(&(o, t)))
            .map(|o| chosen[o])
            .collect();
        let start = (t * 7) % PALETTE_SIZE;
        let k = (0..PALETTE_SIZE)
            .map(|d| (start + d) % PALETTE_SIZE)
            .find(|k| !used.contains(k))
            .unwrap_or(start);
        chosen.push(k);
    }
    chosen.into_iter().map(palette).collect()
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to_u8 = |f: f64| ((f + m) * 255.0).round() as u8;
    [to_u8(r), to_u8(g), to_u8(b)]
}

/// Id grid sampled from the full texture: the window `x0..x1`, `y0..y1`, every `step` pixels.
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub ids: Vec<u16>,
}

impl Grid {
    pub fn sample(buf: &[u16], buf_width: u32, window: (u32, u32, u32, u32), step: u32) -> Self {
        let (x0, y0, x1, y1) = window;
        let width = (x1 - x0) / step;
        let height = (y1 - y0) / step;
        let mut ids = Vec::with_capacity(width as usize * height as usize);
        for j in 0..height {
            let row = (y0 + j * step) as usize * buf_width as usize;
            for i in 0..width {
                ids.push(buf[row + (x0 + i * step) as usize]);
            }
        }
        Self { width, height, ids }
    }

    /// Each pixel repeated `factor` times in both directions, for small close-ups.
    pub fn zoom(self, factor: u32) -> Self {
        let f = factor as usize;
        let (w, h) = (self.width as usize, self.height as usize);
        let mut ids = Vec::with_capacity(w * h * f * f);
        for y in 0..h * f {
            for x in 0..w * f {
                ids.push(self.ids[(y / f) * w + x / f]);
            }
        }
        Self {
            width: self.width * factor,
            height: self.height * factor,
            ids,
        }
    }
}

/// `owner_of[id]` is an index into `colors`; id 0 is water.
pub fn write(
    path: &Path,
    grid: &Grid,
    owner_of: &[usize],
    colors: &[[u8; 3]],
) -> Result<(), String> {
    let (w, h) = (grid.width as usize, grid.height as usize);
    let mut rgb = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let id = grid.ids[y * w + x];
            let color = if id == 0 {
                WATER
            } else {
                let own = owner_of[id as usize];
                let mut color = colors[own];
                for (nx, ny) in [
                    (x + 1, y),
                    (x.wrapping_sub(1), y),
                    (x, y + 1),
                    (x, y.wrapping_sub(1)),
                ] {
                    if nx >= w || ny >= h {
                        continue;
                    }
                    let other = grid.ids[ny * w + nx];
                    if other == id {
                        continue;
                    }
                    if other == 0 {
                        color = COAST;
                        break;
                    }
                    if owner_of[other as usize] != own {
                        color = COUNTRY_BORDER;
                        break;
                    }
                    color = colors[own].map(|c| (f64::from(c) * 0.78) as u8);
                }
                color
            };
            rgb[(y * w + x) * 3..(y * w + x) * 3 + 3].copy_from_slice(&color);
        }
    }
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), grid.width, grid.height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&rgb).map_err(|e| e.to_string())
}
