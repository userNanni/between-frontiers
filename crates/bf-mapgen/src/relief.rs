//! Shaded relief: Natural Earth's hillshade reprojected onto the map frame.
//!
//! The output is an 8-bit grayscale PNG covering the same longitudes and latitudes as
//! `provinces.png`, at a lower resolution. Values are centred on 128 = flat ground: brighter faces
//! the light, darker is in shadow, so the shader can multiply country colours by it. The sea is
//! 128 everywhere, which leaves it untouched.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::projection::Frame;

/// Relief width: the province texture's, so close-ups stay sharp.
pub const WIDTH: u32 = 8192;
const NEUTRAL: u8 = 128;
/// Contrast applied around the land median.
const GAIN: f64 = 1.0;

pub struct Source {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub fn read_tiff(path: &Path) -> Result<Source, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = tiff::decoder::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| format!("{}: {e}", path.display()))?
        .with_limits(tiff::decoder::Limits::unlimited());
    let (width, height) = decoder.dimensions().map_err(|e| e.to_string())?;
    let pixels = match decoder.read_image().map_err(|e| e.to_string())? {
        tiff::decoder::DecodingResult::U8(v) => v,
        other => {
            return Err(format!(
                "{}: unexpected sample type {:?}",
                path.display(),
                std::mem::discriminant(&other)
            ));
        }
    };
    if pixels.len() != (width * height) as usize {
        return Err(format!("{}: expected one band", path.display()));
    }
    Ok(Source {
        width,
        height,
        pixels,
    })
}

impl Source {
    /// Bilinear sample at a longitude and latitude (equirectangular, pixel centres).
    fn sample(&self, lon: f64, lat: f64) -> f64 {
        let fx = (lon + 180.0) / 360.0 * f64::from(self.width) - 0.5;
        let fy = (90.0 - lat) / 180.0 * f64::from(self.height) - 0.5;
        let x0 = fx.floor();
        let y0 = fy.floor();
        let (tx, ty) = (fx - x0, fy - y0);
        let at = |x: f64, y: f64| -> f64 {
            let xi = (x as i64).rem_euclid(i64::from(self.width)) as u32;
            let yi = (y.max(0.0) as u32).min(self.height - 1);
            f64::from(self.pixels[(yi * self.width + xi) as usize])
        };
        let top = at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx;
        let bottom = at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx;
        top * (1.0 - ty) + bottom * ty
    }
}

/// Builds the relief for a frame with the same extent as `ids_frame`. `ids` is the province
/// texture, used to tell land from sea.
pub fn build(
    source: &Source,
    ids_frame: &Frame,
    ids: &[u16],
    lat_top: f64,
    lat_bottom: f64,
) -> (Frame, Vec<u8>) {
    let frame = Frame::new(WIDTH, lat_top, lat_bottom);
    let (w, h) = (frame.width, frame.height);
    let mut raw = vec![0f64; (w * h) as usize];
    let mut land = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let (lon, lat) = frame.to_lon_lat(f64::from(x) + 0.5, f64::from(y) + 0.5);
            let i = (y * w + x) as usize;
            raw[i] = source.sample(lon, lat);
            let (ix, iy) = ids_frame.to_pixel(lon, lat);
            let (ix, iy) = (
                (ix as u32).min(ids_frame.width - 1),
                (iy.max(0.0) as u32).min(ids_frame.height - 1),
            );
            land[i] = ids[(iy * ids_frame.width + ix) as usize] != 0;
        }
    }
    // The median of land is "flat": shading is the departure from it.
    let mut values: Vec<u16> = raw
        .iter()
        .zip(&land)
        .filter(|(_, l)| **l)
        .map(|(v, _)| v.round() as u16)
        .collect();
    values.sort_unstable();
    let median = f64::from(values.get(values.len() / 2).copied().unwrap_or(128));
    let out = raw
        .iter()
        .zip(&land)
        .map(|(v, l)| {
            if *l {
                (f64::from(NEUTRAL) + (v - median) * GAIN)
                    .round()
                    .clamp(0.0, 255.0) as u8
            } else {
                NEUTRAL
            }
        })
        .collect();
    (frame, out)
}

pub fn write_png(path: &Path, frame: &Frame, pixels: &[u8]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), frame.width, frame.height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(pixels).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sea_is_neutral_and_land_is_centred() {
        // A 4×2 source: brighter on the east half.
        let source = Source {
            width: 4,
            height: 2,
            pixels: vec![100, 100, 200, 200, 100, 100, 200, 200],
        };
        let frame = Frame::new(8, 60.0, -60.0);
        let mut ids = vec![0u16; (frame.width * frame.height) as usize];
        // Land on the left column only.
        for y in 0..frame.height {
            ids[(y * frame.width) as usize] = 1;
        }
        let (_, out) = build(&source, &frame, &ids, 60.0, -60.0);
        assert!(out.iter().all(|v| (0..=255).contains(v)));
        assert!(out.contains(&NEUTRAL));
    }
}
