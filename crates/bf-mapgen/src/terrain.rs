//! Terrain of every state, for the land combat step: the shares of mountain and marsh, the mean
//! elevation when a grid is available, and the big rivers on the borders between states. Run with
//! `cargo run -p bf-mapgen --release -- terrain`; reads the province texture already in
//! `assets/map/`, so the provinces do not change, and writes `assets/map/terrain.ron`.
//!
//! - Mountain and elevation: NOAA ETOPO5. A cell is mountain where the elevation varies by
//!   [`MOUNTAIN_RELIEF_M`] or more (standard deviation over 3 × 3 cells); the mean elevation of
//!   the state's cells is recorded apart, since a high plateau (Tibet) is not rough.
//! - Marsh: GLWD v2, the dominant wetland class of each 15″ cell where wetlands cover more than
//!   half of it, for the classes in [`MARSH_CLASSES`].
//! - Rivers: Natural Earth river centrelines of scale rank up to [`RIVER_MAX_SCALERANK`].
//!
//! Method and decisions: `docs/map-pipeline.pt-BR.md`, "Terreno".

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::Path;

use bf_sim::scenario::MapFile;
use bf_sim::war::{RiverBorder, StateTerrain, TerrainFile, TerrainMeta};
use shapefile::dbase::FieldValue;

use crate::projection::Frame;
use crate::sources;

/// Local relief (standard deviation of the ETOPO5 elevation over 3 × 3 cells, in metres) from
/// which a cell counts as mountain. Measured medians: Alps 275, Caucasus 225, Himalaya 181, Andes
/// 139, Massif Central 104, Tibet 102, Carpathians 96, Appalachians 53; North European Plain 8,
/// Kazakh and Pontic steppes 12, pampas 0.
pub const MOUNTAIN_RELIEF_M: u16 = 100;
/// Scale rank (Natural Earth) up to which a river counts as big: the Rhine, the Dnieper and the
/// Oder are rank 4.
pub const RIVER_MAX_SCALERANK: u8 = 5;
/// GLWD v2 classes that count as marsh: lacustrine wetlands (8–9), riverine and palustrine
/// wetlands that flood (10–13, 16–17), peatlands (22–27), mangrove, saltmarsh, river deltas and
/// other coastal wetlands (28–31). Left out: open water (1–7), wetlands only seasonally saturated
/// (14–15, 18–19: the waterlogged soils of farmed floodplains, which made the Dutch polders, the
/// Thames and the pampas look like marsh), ephemeral wetlands (20–21), salt pans (32) and rice
/// paddies (33).
pub const MARSH_CLASSES: [u8; 18] = [
    8, 9, 10, 11, 12, 13, 16, 17, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
];
const GLWD_MEMBER: &str = "GLWD_v2_0_combined_classes/GLWD_v2_0_main_class_50pct.tif";
/// GLWD v2 grid: 15″ cells from 180° W and 84° N.
const GLWD_CELLS_PER_DEGREE: f64 = 240.0;
const GLWD_LAT_TOP: f64 = 84.0;

pub(crate) fn read_png16(path: &Path) -> Result<(u32, u32, Vec<u16>), String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut reader = png::Decoder::new(std::io::BufReader::new(file))
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut bytes = vec![0; reader.output_buffer_size().ok_or("png too large")?];
    let info = reader.next_frame(&mut bytes).map_err(|e| e.to_string())?;
    let buf = bytes[..info.buffer_size()]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes(*c))
        .collect();
    Ok((info.width, info.height, buf))
}

/// The map's state at a lon/lat, or 0 for water and off the map.
struct StateLookup<'a> {
    frame: &'a Frame,
    land: &'a [u16],
    state_of_province: Vec<u16>,
}

impl StateLookup<'_> {
    fn row(&self, lat: f64) -> Option<usize> {
        let (_, y) = self.frame.to_pixel(0.0, lat);
        (y >= 0.0 && y < f64::from(self.frame.height)).then_some(y as usize)
    }

    fn column(&self, lon: f64) -> usize {
        let (x, _) = self.frame.to_pixel(lon, 0.0);
        (x.max(0.0) as usize).min(self.frame.width as usize - 1)
    }

    fn state(&self, column: usize, row: usize) -> u16 {
        self.state_of_province[self.land[row * self.frame.width as usize + column] as usize]
    }
}

/// ETOPO5 grid: 5-minute cells, 2160 rows from 90° N, 4320 columns from 0° E.
const ETOPO_ROWS: usize = 2160;
const ETOPO_COLUMNS: usize = 4320;

/// Per state: ETOPO5 land cells, those rough enough to be mountain, and the sum of their
/// elevations. Roughness is the standard deviation of the elevation over a 3 × 3 window (about
/// 27 km), wrapping across the date line.
fn mountain(lookup: &StateLookup, etopo: &[i16], states: usize) -> (Vec<u64>, Vec<u64>, Vec<i64>) {
    let (w, h) = (ETOPO_COLUMNS, ETOPO_ROWS);
    let columns: Vec<usize> = (0..w)
        .map(|x| {
            let lon = x as f64 / 12.0;
            lookup.column(if lon >= 180.0 { lon - 360.0 } else { lon })
        })
        .collect();
    let t = i64::from(MOUNTAIN_RELIEF_M);
    let (mut total, mut rough, mut height) = (
        vec![0u64; states + 1],
        vec![0u64; states + 1],
        vec![0i64; states + 1],
    );
    for y in 0..h {
        let Some(row) = lookup.row(90.0 - y as f64 / 12.0) else {
            continue;
        };
        for x in 0..w {
            let s = lookup.state(columns[x], row) as usize;
            if s == 0 {
                continue;
            }
            let (mut sum, mut sq) = (0i64, 0i64);
            for dy in 0..3 {
                let yy = (y + dy).saturating_sub(1).min(h - 1);
                for dx in 0..3 {
                    let v = i64::from(etopo[yy * w + (x + w + dx - 1) % w]);
                    sum += v;
                    sq += v * v;
                }
            }
            // std ≥ t  ⇔  9·Σv² − (Σv)² ≥ t²·81
            total[s] += 1;
            height[s] += i64::from(etopo[y * w + x]);
            if 9 * sq - sum * sum >= t * t * 81 {
                rough[s] += 1;
            }
        }
    }
    (total, rough, height)
}

fn read_etopo(path: &Path) -> Result<Vec<i16>, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() != ETOPO_ROWS * ETOPO_COLUMNS * 2 {
        return Err(format!(
            "{}: {} bytes, not a 2160 x 4320 grid",
            path.display(),
            bytes.len()
        ));
    }
    Ok(bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| i16::from_be_bytes(*c))
        .collect())
}

/// Counts, per state, the GLWD cells and those whose dominant wetland is a marsh class. Reads the
/// GeoTIFF chunk by chunk, so the 3-gigapixel grid never sits in memory.
fn marsh(lookup: &StateLookup, tif: &Path, states: usize) -> Result<(Vec<u64>, Vec<u64>), String> {
    let file = File::open(tif).map_err(|e| format!("{}: {e}", tif.display()))?;
    let mut decoder = tiff::decoder::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| format!("{}: {e}", tif.display()))?
        .with_limits(tiff::decoder::Limits::unlimited());
    let (width, height) = decoder.dimensions().map_err(|e| e.to_string())?;
    let expected_w = (360.0 * GLWD_CELLS_PER_DEGREE) as u32;
    if width != expected_w {
        return Err(format!(
            "{}: {width} columns, expected {expected_w}",
            tif.display()
        ));
    }
    let mut is_marsh = [false; 256];
    for c in MARSH_CLASSES {
        is_marsh[c as usize] = true;
    }
    let columns: Vec<usize> = (0..width)
        .map(|x| lookup.column((f64::from(x) + 0.5) / GLWD_CELLS_PER_DEGREE - 180.0))
        .collect();
    let rows: Vec<Option<usize>> = (0..height)
        .map(|y| lookup.row(GLWD_LAT_TOP - (f64::from(y) + 0.5) / GLWD_CELLS_PER_DEGREE))
        .collect();
    let (cw, ch) = decoder.chunk_dimensions();
    let across = width.div_ceil(cw);
    let chunks = match decoder.get_chunk_type() {
        tiff::decoder::ChunkType::Tile => decoder.tile_count(),
        tiff::decoder::ChunkType::Strip => decoder.strip_count(),
    }
    .map_err(|e| e.to_string())?;
    let (mut total, mut wet) = (vec![0u64; states + 1], vec![0u64; states + 1]);
    for i in 0..chunks {
        let (dw, dh) = decoder.chunk_data_dimensions(i);
        let (x0, y0) = ((i % across) * cw, (i / across) * ch);
        let data = match decoder.read_chunk(i).map_err(|e| e.to_string())? {
            tiff::decoder::DecodingResult::U8(v) => v,
            _ => return Err(format!("{}: not an 8-bit grid", tif.display())),
        };
        for dy in 0..dh {
            let Some(row) = rows[(y0 + dy) as usize] else {
                continue;
            };
            for dx in 0..dw {
                let s = lookup.state(columns[(x0 + dx) as usize], row) as usize;
                if s == 0 {
                    continue;
                }
                total[s] += 1;
                if is_marsh[data[(dy * dw + dx) as usize] as usize] {
                    wet[s] += 1;
                }
            }
        }
    }
    Ok((total, wet))
}

struct River {
    name: String,
    points: Vec<Vec<(f64, f64)>>,
}

fn read_rivers(shp: &Path) -> Result<Vec<River>, String> {
    let mut reader =
        shapefile::Reader::from_path(shp).map_err(|e| format!("{}: {e}", shp.display()))?;
    let mut out = Vec::new();
    for item in reader.iter_shapes_and_records_as::<shapefile::Polyline, shapefile::dbase::Record>()
    {
        let (shape, record) = item.map_err(|e| format!("{}: {e}", shp.display()))?;
        let text = |name: &str| match record.get(name) {
            Some(FieldValue::Character(Some(s))) => s
                .trim_matches(|c: char| c == '\0' || c.is_whitespace())
                .to_string(),
            _ => String::new(),
        };
        let rank = match record.get("scalerank") {
            Some(FieldValue::Numeric(Some(v))) => *v,
            Some(FieldValue::Integer(v)) => f64::from(*v),
            Some(FieldValue::Double(v)) => *v,
            _ => f64::INFINITY,
        };
        if text("featurecla") != "River" || rank > f64::from(RIVER_MAX_SCALERANK) {
            continue;
        }
        let name = text("name_en");
        let name = if name.is_empty() { text("name") } else { name };
        out.push(River {
            name,
            points: shape
                .parts()
                .iter()
                .map(|p| p.iter().map(|q| (q.x, q.y)).collect())
                .collect(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Marks the pixels the rivers cross with their index + 1 (the first river wins a pixel).
fn rasterize_rivers(frame: &Frame, rivers: &[River]) -> Vec<u16> {
    let (w, h) = (frame.width as usize, frame.height as usize);
    let mut buf = vec![0u16; w * h];
    for (ri, r) in rivers.iter().enumerate() {
        let id = (ri + 1) as u16;
        for part in &r.points {
            for seg in part.windows(2) {
                let (x0, y0) = frame.to_pixel(seg[0].0, seg[0].1);
                let (x1, y1) = frame.to_pixel(seg[1].0, seg[1].1);
                let steps = ((x1 - x0).abs().max((y1 - y0).abs()) * 2.0).ceil().max(1.0) as usize;
                for k in 0..=steps {
                    let t = k as f64 / steps as f64;
                    let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
                    if x < 0.0 || y < 0.0 || x >= w as f64 || y >= h as f64 {
                        continue;
                    }
                    let i = y as usize * w + x as usize;
                    if buf[i] == 0 {
                        buf[i] = id;
                    }
                }
            }
        }
    }
    buf
}

/// Border edges, those near a big river, and the hits of each river, for one pair of states.
type Tally = (u32, u32, BTreeMap<u16, u32>);

/// Borders between neighbouring states and the part of each within a pixel of a big river.
fn river_borders(
    frame: &Frame,
    land: &[u16],
    state_of_province: &[u16],
    rivers: &[River],
) -> Vec<RiverBorder> {
    let (w, h) = (frame.width as usize, frame.height as usize);
    let mask = rasterize_rivers(frame, rivers);
    let near = |i: usize| -> u16 {
        let (x, y) = ((i % w) as i64, (i / w) as i64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (nx, ny) = ((x + dx).rem_euclid(w as i64), y + dy);
                if ny < 0 || ny >= h as i64 {
                    continue;
                }
                let r = mask[ny as usize * w + nx as usize];
                if r != 0 {
                    return r;
                }
            }
        }
        0
    };
    let mut pairs: BTreeMap<(u16, u16), Tally> = BTreeMap::new();
    for i in 0..w * h {
        let s = state_of_province[land[i] as usize];
        if s == 0 {
            continue;
        }
        let x = i % w;
        let right = if x + 1 == w { i + 1 - w } else { i + 1 };
        for j in [Some(right), (i + w < w * h).then_some(i + w)]
            .into_iter()
            .flatten()
        {
            let t = state_of_province[land[j] as usize];
            if t == 0 || t == s {
                continue;
            }
            let e = pairs.entry((s.min(t), s.max(t))).or_default();
            e.0 += 1;
            let r = near(i).max(near(j));
            if r != 0 {
                e.1 += 1;
                *e.2.entry(r).or_default() += 1;
            }
        }
    }
    pairs
        .into_iter()
        .filter(|(_, (_, river, _))| *river > 0)
        .map(|((a, b), (border, river, hits))| {
            let best = hits
                .iter()
                .max_by(|x, y| x.1.cmp(y.1).then(y.0.cmp(x.0)))
                .map_or(0, |(r, _)| *r);
            RiverBorder {
                a,
                b,
                border,
                river,
                name: rivers[best as usize - 1].name.clone(),
            }
        })
        .collect()
}

fn per_mille(part: u64, whole: u64) -> u16 {
    (part * 1000).checked_div(whole).unwrap_or(0) as u16
}

pub fn run(root: &Path, frame: &Frame) -> Result<(), String> {
    let raw = root.join("data/raw");
    let (w, h, land) = read_png16(&root.join("assets/map/provinces.png"))?;
    if (w, h) != (frame.width, frame.height) {
        return Err(format!(
            "provinces.png is {w}x{h}, expected {}x{}",
            frame.width, frame.height
        ));
    }
    let map_path = root.join("assets/map/provinces.ron");
    let map: MapFile = ron::from_str(
        &fs::read_to_string(&map_path).map_err(|e| format!("{}: {e}", map_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", map_path.display()))?;
    let state_of_province: Vec<u16> = std::iter::once(0)
        .chain(map.provinces.iter().map(|p| p.state))
        .collect();
    let lookup = StateLookup {
        frame,
        land: &land,
        state_of_province: state_of_province.clone(),
    };
    let n = map.states.len();

    let etopo = read_etopo(&sources::fetch_plain(&sources::ETOPO5, &raw)?)?;
    let (etopo_total, etopo_rough, etopo_height) = mountain(&lookup, &etopo, n);
    drop(etopo);

    let glwd = sources::fetch_member(&sources::GLWD, &raw, GLWD_MEMBER)?;
    let (glwd_total, glwd_wet) = marsh(&lookup, &glwd, n)?;

    let rivers_shp = sources::fetch(&sources::RIVERS, &raw)?;
    let rivers = read_rivers(&rivers_shp)?;
    let borders = river_borders(frame, &land, &state_of_province, &rivers);

    let file = TerrainFile {
        meta: TerrainMeta {
            sources: vec![
                format!(
                    "NOAA ETOPO5 5-minute elevation, free to use and redistribute: mountain = \
                     standard deviation of the elevation over 3 x 3 cells >= {MOUNTAIN_RELIEF_M} m"
                ),
                "GLWD v2 (Lehner et al. 2025), main wetland class where wetlands exceed 50 %, \
                 CC BY 4.0, https://doi.org/10.6084/m9.figshare.28519994"
                    .to_string(),
                format!(
                    "Natural Earth {} 1:10m rivers ({}), public domain, scale rank <= \
                     {RIVER_MAX_SCALERANK}",
                    sources::RIVERS_VERSION,
                    sources::RIVERS.stem
                ),
            ],
            mountain_relief_m: MOUNTAIN_RELIEF_M,
            river_max_scalerank: RIVER_MAX_SCALERANK,
        },
        states: map
            .states
            .iter()
            .map(|s| {
                let i = s.id as usize;
                StateTerrain {
                    state: s.id,
                    key: s.key.clone(),
                    mountain: per_mille(etopo_rough[i], etopo_total[i]),
                    marsh: per_mille(glwd_wet[i], glwd_total[i]),
                    elevation_m: (etopo_total[i] > 0)
                        .then(|| (etopo_height[i] / etopo_total[i] as i64) as i32),
                }
            })
            .collect(),
        borders,
    };
    write_ron(&root.join("assets/map/terrain.ron"), &file)?;
    let mountainous = file.states.iter().filter(|s| s.mountain >= 500).count();
    let marshy = file.states.iter().filter(|s| s.marsh >= 100).count();
    println!(
        "{} states: {mountainous} at least half mountain, {marshy} with a tenth of marsh; {} \
         borders on big rivers",
        file.states.len(),
        file.borders.len()
    );
    Ok(())
}

fn write_ron(path: &Path, file: &TerrainFile) -> Result<(), String> {
    let config = ron::ser::PrettyConfig::new().depth_limit(2);
    let body = ron::ser::to_string_pretty(file, config).map_err(|e| e.to_string())?;
    let text = format!(
        "// Generated by `cargo run -p bf-mapgen --release -- terrain`; do not edit by hand.\n\
         // Format: bf_sim::war::TerrainFile. Method: docs/map-pipeline.pt-BR.md, \"Terreno\".\n{body}\n"
    );
    let _: TerrainFile =
        ron::from_str(&text).map_err(|e| format!("terrain.ron does not read back: {e}"))?;
    fs::write(path, text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 360-pixel-wide map: land in the middle third (state 1 west, state 2 east of 0°).
    fn world() -> (Frame, Vec<u16>) {
        let frame = Frame::new(360, 84.0, -58.0);
        let (w, h) = (frame.width as usize, frame.height as usize);
        let mut land = vec![0u16; w * h];
        for y in 0..h {
            for x in 120..240 {
                land[y * w + x] = if x < 180 { 1 } else { 2 };
            }
        }
        (frame, land)
    }

    #[test]
    fn rough_ground_counts_as_mountain_and_plateaus_keep_their_height() {
        let (frame, land) = world();
        let lookup = StateLookup {
            frame: &frame,
            land: &land,
            state_of_province: vec![0, 1, 2],
        };
        // West: a checkerboard of 0 and 1,000 m; east: a flat plateau at 4,000 m.
        let etopo: Vec<i16> = (0..ETOPO_ROWS * ETOPO_COLUMNS)
            .map(|i| {
                let (x, y) = (i % ETOPO_COLUMNS, i / ETOPO_COLUMNS);
                if x >= ETOPO_COLUMNS / 2 {
                    if (x + y) % 2 == 0 { 1000 } else { 0 }
                } else {
                    4000
                }
            })
            .collect();
        let (total, rough, height) = mountain(&lookup, &etopo, 2);
        assert!(total[1] > 0 && total[2] > 0);
        assert_eq!(per_mille(rough[1], total[1]), 1000, "west is rough");
        // Only the column at 0°, next to the checkerboard, is rough: under 1 % of the plateau.
        assert!(per_mille(rough[2], total[2]) < 10, "the plateau is flat");
        assert_eq!(height[2] / total[2] as i64, 4000);
        assert_eq!(
            mountain(&lookup, &etopo, 2),
            (total, rough, height),
            "deterministic"
        );
    }

    #[test]
    fn a_river_along_the_border_is_counted_with_its_name() {
        let (frame, land) = world();
        let river = River {
            name: "Test".into(),
            points: vec![vec![(0.0, 60.0), (0.0, -40.0)]],
        };
        let borders = river_borders(&frame, &land, &[0, 1, 2], &[river]);
        assert_eq!(borders.len(), 1);
        let b = &borders[0];
        assert_eq!((b.a, b.b, b.name.as_str()), (1, 2, "Test"));
        assert!(b.river > 0 && b.river < b.border, "{b:?}");
        let again = river_borders(
            &frame,
            &land,
            &[0, 1, 2],
            &[River {
                name: "Test".into(),
                points: vec![vec![(0.0, 60.0), (0.0, -40.0)]],
            }],
        );
        assert_eq!(borders, again);
        // No river: no border is listed.
        assert!(river_borders(&frame, &land, &[0, 1, 2], &[]).is_empty());
    }

    #[test]
    fn the_committed_terrain_matches_the_map() {
        let root = crate::repo_root();
        let map: MapFile =
            ron::from_str(&fs::read_to_string(root.join("assets/map/provinces.ron")).unwrap())
                .unwrap();
        let terrain: TerrainFile =
            ron::from_str(&fs::read_to_string(root.join("assets/map/terrain.ron")).unwrap())
                .unwrap();
        let keys: Vec<&str> = map.states.iter().map(|s| s.key.as_str()).collect();
        let generated: Vec<&str> = terrain.states.iter().map(|s| s.key.as_str()).collect();
        assert_eq!(
            keys, generated,
            "regenerate: cargo run -p bf-mapgen --release -- terrain"
        );
        assert_eq!(terrain.meta.mountain_relief_m, MOUNTAIN_RELIEF_M);
        assert_eq!(terrain.meta.river_max_scalerank, RIVER_MAX_SCALERANK);
        assert!(
            terrain
                .borders
                .iter()
                .all(|b| b.a < b.b && b.river <= b.border)
        );
    }
}
