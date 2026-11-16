//! Sea zones: the water of `provinces.png` cut into zones by the Natural Earth marine polygons
//! and `data/map/sea_zones.ron`, with the contacts between zones, the ports (coastal states and
//! the zones they touch) and the declared straits and canals. Run with
//! `cargo run -p bf-mapgen --release -- seas`; reads the province texture already in
//! `assets/map/`, so the provinces do not change. `-- seas --survey` lists the marine polygons
//! instead, to write the zone rules.
//!
//! Method and decisions: `docs/map-pipeline.pt-BR.md`, "Zonas marítimas".

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use bf_sim::scenario::MapFile;
use bf_sim::sea::{
    EdgeDef, HolderDef, Ice, PassageKind, PortDef, SeaMeta, SeasFile, StraitDef, ZoneDef,
};
use geo::{BooleanOps, Coord, LineString, MultiPolygon, Polygon};
use serde::Deserialize;

use crate::projection::Frame;
use crate::{raster, sources, states};

/// The hand-written rules.
#[derive(Debug, Deserialize)]
pub struct SeaZonesFile {
    pub zones: Vec<ZoneRule>,
    pub straits: Vec<StraitRule>,
}

#[derive(Debug, Deserialize)]
pub struct ZoneRule {
    pub key: String,
    pub name: String,
    pub parts: Vec<Part>,
    pub ice: Ice,
    /// Source of the ice class; required unless the zone is ice-free.
    #[serde(default)]
    pub ice_source: String,
    /// A sea with no way out (the Caspian): exempt from the every-port-reaches-every-port rule.
    #[serde(default)]
    pub isolated: bool,
}

/// A Natural Earth marine polygon, by name, optionally clipped to a lon/lat box.
#[derive(Debug, Deserialize)]
pub struct Part {
    pub feature: String,
    #[serde(default)]
    pub lon: Option<(f64, f64)>,
    #[serde(default)]
    pub lat: Option<(f64, f64)>,
}

#[derive(Debug, Deserialize)]
pub struct StraitRule {
    pub key: String,
    pub name: String,
    pub kind: PassageKind,
    pub zones: (String, String),
    /// States on its shores; who controls it is the owner of these states at the time.
    pub shores: Vec<String>,
    #[serde(default)]
    pub opened: Option<String>,
    /// Treaty that governs passage (reference for phase 4; no behaviour yet).
    #[serde(default)]
    pub regime: Option<String>,
    /// Countries that hold the passage by treaty or occupation, before the shores' owners.
    #[serde(default)]
    pub holders: Vec<HolderDef>,
    pub source: String,
}

fn read_png16(path: &Path) -> Result<(u32, u32, Vec<u16>), String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut reader = png::Decoder::new(std::io::BufReader::new(file))
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut bytes = vec![0; reader.output_buffer_size().ok_or("png too large")?];
    let info = reader.next_frame(&mut bytes).map_err(|e| e.to_string())?;
    if info.bit_depth != png::BitDepth::Sixteen {
        return Err(format!("{}: not a 16-bit image", path.display()));
    }
    let buf = bytes[..info.buffer_size()]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes(*c))
        .collect();
    Ok((info.width, info.height, buf))
}

fn project(frame: &Frame, geometry: &MultiPolygon<f64>) -> Vec<Vec<raster::Ring>> {
    let ring = |ls: &LineString<f64>| -> raster::Ring {
        ls.0.iter().map(|c| frame.to_pixel(c.x, c.y)).collect()
    };
    geometry
        .0
        .iter()
        .map(|p| {
            std::iter::once(p.exterior())
                .chain(p.interiors())
                .map(ring)
                .collect()
        })
        .collect()
}

fn clip(geometry: &MultiPolygon<f64>, part: &Part) -> MultiPolygon<f64> {
    if part.lon.is_none() && part.lat.is_none() {
        return geometry.clone();
    }
    let (x0, x1) = part.lon.unwrap_or((-180.0, 180.0));
    let (y0, y1) = part.lat.unwrap_or((-90.0, 90.0));
    let rect = Polygon::new(
        LineString::new(vec![
            Coord { x: x0, y: y0 },
            Coord { x: x1, y: y0 },
            Coord { x: x1, y: y1 },
            Coord { x: x0, y: y1 },
            Coord { x: x0, y: y0 },
        ]),
        vec![],
    );
    geometry.intersection(&MultiPolygon::new(vec![rect]))
}

/// Great-circle distance in km between two lon/lat points (mean Earth radius).
fn great_circle_km(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (l1, p1) = (a.0.to_radians(), a.1.to_radians());
    let (l2, p2) = (b.0.to_radians(), b.1.to_radians());
    let h = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * ((l2 - l1) / 2.0).sin().powi(2);
    2.0 * 6371.0 * h.sqrt().min(1.0).asin()
}

fn round_to(v: f64, step: f64) -> f64 {
    (v * step).round() / step
}

/// The four neighbours of pixel `i`, wrapping across the date line.
fn neighbours(i: usize, w: usize, h: usize) -> impl Iterator<Item = usize> {
    let (x, y) = (i % w, i / w);
    let left = if x == 0 { i + w - 1 } else { i - 1 };
    let right = if x + 1 == w { i + 1 - w } else { i + 1 };
    let up = (y > 0).then(|| i - w);
    let down = (y + 1 < h).then(|| i + w);
    [Some(left), Some(right), up, down].into_iter().flatten()
}

pub fn run(root: &Path, frame: &Frame, survey: bool) -> Result<(), String> {
    let shp = sources::fetch(&sources::MARINE, &root.join("data/raw"))?;
    let features = sources::read_features(&shp, &["name", "featurecla"])?;
    let (w, h, land) = read_png16(&root.join("assets/map/provinces.png"))?;
    if (w, h) != (frame.width, frame.height) {
        return Err(format!(
            "provinces.png is {w}x{h}, expected {}x{}",
            frame.width, frame.height
        ));
    }
    let (w, h) = (w as usize, h as usize);
    if survey {
        return print_survey(frame, &features, &land, w, h);
    }
    let map_path = root.join("assets/map/provinces.ron");
    let map: MapFile = ron::from_str(
        &fs::read_to_string(&map_path).map_err(|e| format!("{}: {e}", map_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", map_path.display()))?;
    let rules_path = root.join("data/map/sea_zones.ron");
    let rules: SeaZonesFile = ron::from_str(
        &fs::read_to_string(&rules_path).map_err(|e| format!("{}: {e}", rules_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", rules_path.display()))?;
    let seas = build(frame, &features, &land, w, h, &map, &rules)?;
    let out = root.join("assets/map");
    write_zone_png(&out.join("seas.png"), frame, &seas.1)?;
    write_ron(&out.join("seas.ron"), &seas.0)?;
    print_summary(&seas.0);
    Ok(())
}

fn print_survey(
    frame: &Frame,
    features: &[sources::Feature],
    land: &[u16],
    w: usize,
    h: usize,
) -> Result<(), String> {
    let mut claims = vec![0u8; w * h];
    let mut rows = Vec::new();
    for f in features {
        let mut count = 0u64;
        let mut mask = Vec::new();
        for rings in project(frame, &f.geometry) {
            rasterize_into(&mut mask, w, h, &rings);
        }
        mask.sort_unstable();
        mask.dedup();
        let mut sx = 0.0;
        let mut sy = 0.0;
        for &i in &mask {
            if land[i] == 0 {
                count += 1;
                claims[i] = claims[i].saturating_add(1);
                sx += (i % w) as f64;
                sy += (i / w) as f64;
            }
        }
        let (lon, lat) = if count > 0 {
            frame.to_lon_lat(sx / count as f64, sy / count as f64)
        } else {
            (f64::NAN, f64::NAN)
        };
        rows.push((
            f.get("name").to_string(),
            f.get("featurecla").to_string(),
            count,
            lon,
            lat,
        ));
    }
    rows.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    for (name, class, n, lon, lat) in &rows {
        println!("{n:>9} {class:<8} {name:<40} {lon:>8.1} {lat:>6.1}");
    }
    let water = land.iter().filter(|v| **v == 0).count();
    let covered = claims.iter().filter(|c| **c > 0).count();
    let overlap = claims.iter().filter(|c| **c > 1).count();
    println!("water pixels {water}, covered {covered}, overlapping {overlap}");
    Ok(())
}

/// The generated graph and the zone of every pixel (0 on land and on unreached water).
pub fn build(
    frame: &Frame,
    features: &[sources::Feature],
    land: &[u16],
    w: usize,
    h: usize,
    map: &MapFile,
    rules: &SeaZonesFile,
) -> Result<(SeasFile, Vec<u16>), String> {
    let mut errors = Vec::new();
    let mut by_name: BTreeMap<&str, Vec<&sources::Feature>> = BTreeMap::new();
    for f in features {
        by_name.entry(f.get("name")).or_default().push(f);
    }
    let mut keys = BTreeSet::new();
    for z in &rules.zones {
        if !keys.insert(z.key.as_str()) {
            errors.push(format!("zone {}: duplicate key", z.key));
        }
        if z.ice != Ice::Free && z.ice_source.trim().is_empty() {
            errors.push(format!("zone {}: ice {:?} without a source", z.key, z.ice));
        }
    }
    // 1. Rasterize the parts of every zone on water pixels; the first zone in the file wins.
    let mut zone = vec![0u16; w * h];
    let mut used_features = BTreeSet::new();
    for (zi, z) in rules.zones.iter().enumerate() {
        let id = (zi + 1) as u16;
        for part in &z.parts {
            let Some(found) = by_name.get(part.feature.as_str()) else {
                errors.push(format!(
                    "zone {}: no marine polygon named {:?}",
                    z.key, part.feature
                ));
                continue;
            };
            used_features.insert(part.feature.as_str());
            for f in found {
                let geometry = clip(&f.geometry, part);
                let mut scratch_rings = project(frame, &geometry);
                for rings in scratch_rings.drain(..) {
                    let mut mask = Vec::new();
                    rasterize_into(&mut mask, w, h, &rings);
                    for i in mask {
                        if land[i] == 0 && zone[i] == 0 {
                            zone[i] = id;
                        }
                    }
                }
            }
        }
    }
    // 2. Water no part covers goes to the nearest zone through water (breadth first, in pixel
    //    order); water not connected to any zone (lakes) stays out.
    let mut queue: VecDeque<usize> = (0..w * h)
        .filter(|&i| zone[i] != 0 && neighbours(i, w, h).any(|j| land[j] == 0 && zone[j] == 0))
        .collect();
    while let Some(i) = queue.pop_front() {
        for j in neighbours(i, w, h) {
            if land[j] == 0 && zone[j] == 0 {
                zone[j] = zone[i];
                queue.push_back(j);
            }
        }
    }
    let n = rules.zones.len();
    let mut pixels = vec![0u32; n + 1];
    for &z in &zone {
        pixels[z as usize] += 1;
    }
    for (zi, z) in rules.zones.iter().enumerate() {
        if pixels[zi + 1] == 0 {
            errors.push(format!("zone {}: no pixel", z.key));
        }
    }
    // 3. Centres (pole of inaccessibility on the texture), contacts and ports.
    let identity: Vec<u32> = (0..=n as u32).collect();
    let centers = states::centers(&zone, w as u32, h as u32, &identity, n);
    let state_of_province: Vec<u16> = std::iter::once(0)
        .chain(map.provinces.iter().map(|p| p.state))
        .collect();
    let mut contact: BTreeMap<(u16, u16), u32> = BTreeMap::new();
    let mut coast: BTreeMap<(u16, u16), u32> = BTreeMap::new();
    for i in 0..w * h {
        let zi = zone[i];
        if zi != 0 {
            // Each pair counted once per pixel edge: look right and down only.
            let x = i % w;
            let right = if x + 1 == w { i + 1 - w } else { i + 1 };
            for j in [Some(right), (i + w < w * h).then_some(i + w)]
                .into_iter()
                .flatten()
            {
                let zj = zone[j];
                if zj != 0 && zj != zi {
                    *contact.entry((zi.min(zj), zi.max(zj))).or_default() += 1;
                }
            }
        } else if land[i] != 0 {
            let s = state_of_province[land[i] as usize];
            for j in neighbours(i, w, h) {
                if zone[j] != 0 {
                    *coast.entry((s, zone[j])).or_default() += 1;
                }
            }
        }
    }
    let lon_lat = |c: (f32, f32)| frame.to_lon_lat(c.0 as f64, c.1 as f64);
    let zone_ll: Vec<(f64, f64)> = centers
        .iter()
        .map(|c| c.map_or((0.0, 0.0), lon_lat))
        .collect();
    let zones: Vec<ZoneDef> = rules
        .zones
        .iter()
        .enumerate()
        .map(|(zi, z)| {
            let c = centers[zi].unwrap_or((0.0, 0.0));
            let ll = zone_ll[zi];
            ZoneDef {
                id: (zi + 1) as u16,
                key: z.key.clone(),
                name: z.name.clone(),
                center: c,
                lon_lat: (round_to(ll.0, 100.0) as f32, round_to(ll.1, 100.0) as f32),
                pixels: pixels[zi + 1],
                ice: z.ice,
                ice_source: z.ice_source.clone(),
                isolated: z.isolated,
            }
        })
        .collect();
    let zone_id = |key: &str| zones.iter().find(|z| z.key == key).map(|z| z.id);
    let state_id = |key: &str| map.states.iter().find(|s| s.key == key).map(|s| s.id);
    // 4. Straits: declared edges that replace the natural contact between their zones.
    let mut straits = Vec::new();
    for (si, s) in rules.straits.iter().enumerate() {
        let a = zone_id(&s.zones.0);
        let b = zone_id(&s.zones.1);
        if a.is_none() || b.is_none() || a == b {
            errors.push(format!("strait {}: bad zones {:?}", s.key, s.zones));
            continue;
        }
        let mut shores = Vec::new();
        for k in &s.shores {
            // A state too small for the texture (Gibraltar) is a shore by declaration.
            let tiny = |id: u16| {
                map.states[id as usize - 1]
                    .provinces
                    .iter()
                    .all(|p| map.provinces[*p as usize - 1].pixels < 50)
            };
            match state_id(k) {
                Some(id) if tiny(id) || coast.keys().any(|(st, _)| *st == id) => shores.push(id),
                Some(_) => errors.push(format!("strait {}: shore {k} has no coast", s.key)),
                None => errors.push(format!("strait {}: unknown state {k}", s.key)),
            }
        }
        if shores.is_empty() {
            errors.push(format!("strait {}: no shore", s.key));
        }
        for h in &s.holders {
            if !crate::owners::is_tag(&h.tag)
                || h.source.trim().is_empty()
                || h.until.as_deref().is_some_and(|u| u <= h.from.as_str())
            {
                errors.push(format!(
                    "strait {}: holder {} unknown, without a source or with bad dates",
                    s.key, h.tag
                ));
            }
        }
        straits.push(StraitDef {
            id: si as u16,
            key: s.key.clone(),
            name: s.name.clone(),
            kind: s.kind,
            a: a.unwrap_or(0),
            b: b.unwrap_or(0),
            shores,
            opened: s.opened.clone(),
            regime: s.regime.clone(),
            holders: s.holders.clone(),
            source: s.source.clone(),
        });
    }
    let strait_pairs: BTreeSet<(u16, u16)> = straits
        .iter()
        .map(|s| (s.a.min(s.b), s.a.max(s.b)))
        .collect();
    let km = |a: u16, b: u16| {
        round_to(
            great_circle_km(zone_ll[a as usize - 1], zone_ll[b as usize - 1]),
            1.0,
        ) as f32
    };
    let mut edges: Vec<EdgeDef> = contact
        .iter()
        .filter(|(pair, _)| !strait_pairs.contains(pair))
        .map(|(&(a, b), &c)| EdgeDef {
            a,
            b,
            km: km(a, b),
            contact: c,
            strait: None,
        })
        .collect();
    for s in &straits {
        let (a, b) = (s.a.min(s.b), s.a.max(s.b));
        if a == 0 {
            continue;
        }
        edges.push(EdgeDef {
            a,
            b,
            km: km(a, b),
            contact: contact.get(&(a, b)).copied().unwrap_or(0),
            strait: Some(s.id),
        });
    }
    edges.sort_by_key(|e| (e.a, e.b, e.strait));
    let ports: Vec<PortDef> = coast
        .iter()
        .filter(|((s, _), _)| *s != 0)
        .map(|(&(s, z), &c)| {
            let st = &map.states[s as usize - 1];
            let ll = lon_lat(st.center);
            PortDef {
                state: s,
                zone: z,
                coast: c,
                km: round_to(great_circle_km(ll, zone_ll[z as usize - 1]), 1.0) as f32,
            }
        })
        .collect();
    let unused: Vec<&str> = by_name
        .keys()
        .filter(|k| !used_features.contains(*k))
        .copied()
        .collect();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let file = SeasFile {
        meta: SeaMeta {
            sources: vec![
                format!(
                    "Natural Earth {} 1:10m marine polygons ({}), public domain, \
                     https://www.naturalearthdata.com",
                    sources::MARINE_VERSION,
                    sources::MARINE.stem
                ),
                "Zones, straits and canals: data/map/sea_zones.ron, CC BY-SA 4.0".to_string(),
            ],
            unlisted_features: unused.len() as u32,
        },
        zones,
        edges,
        ports,
        straits,
    };
    Ok((file, zone))
}

/// Indices of the pixels whose centre is inside the polygon made of `rings`.
fn rasterize_into(out: &mut Vec<usize>, w: usize, h: usize, rings: &[raster::Ring]) {
    let (mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY);
    for r in rings {
        for p in r {
            y0 = y0.min(p.1);
            y1 = y1.max(p.1);
        }
    }
    if !y0.is_finite() {
        return;
    }
    let top = (y0.floor().max(0.0) as usize).min(h);
    let bottom = ((y1.ceil() + 1.0).max(0.0) as usize).min(h);
    if top >= bottom {
        return;
    }
    // Fill a band of the rows the polygon spans, then read it back.
    let band_h = bottom - top;
    let shifted: Vec<raster::Ring> = rings
        .iter()
        .map(|r| r.iter().map(|&(x, y)| (x, y - top as f64)).collect())
        .collect();
    let mut band = vec![0u16; w * band_h];
    raster::fill_polygon(&mut band, w as u32, band_h as u32, &shifted, 1);
    out.extend(
        band.iter()
            .enumerate()
            .filter(|(_, v)| **v == 1)
            .map(|(i, _)| i + top * w),
    );
}

fn write_zone_png(path: &Path, frame: &Frame, zone: &[u16]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), frame.width, frame.height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = zone.iter().map(|&z| z as u8).collect();
    writer.write_image_data(&bytes).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}

fn write_ron(path: &Path, seas: &SeasFile) -> Result<(), String> {
    let config = ron::ser::PrettyConfig::new().depth_limit(2);
    let body = ron::ser::to_string_pretty(seas, config).map_err(|e| e.to_string())?;
    let text = format!(
        "// Generated by `cargo run -p bf-mapgen --release -- seas`; do not edit by hand.\n\
         // Zone ids are pixel values in seas.png. Format: bf_sim::sea::SeasFile.\n{body}\n"
    );
    let _: SeasFile =
        ron::from_str(&text).map_err(|e| format!("seas.ron does not read back: {e}"))?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn print_summary(seas: &SeasFile) {
    println!(
        "{} zones, {} contacts, {} straits and canals, {} ports",
        seas.zones.len(),
        seas.edges.iter().filter(|e| e.strait.is_none()).count(),
        seas.straits.len(),
        seas.ports.len()
    );
    for z in &seas.zones {
        let degree = seas
            .edges
            .iter()
            .filter(|e| e.a == z.id || e.b == z.id)
            .count();
        let ports = seas.ports.iter().filter(|p| p.zone == z.id).count();
        println!(
            "  {:>3} {:<28} {:>9} px  {:>2} neighbours  {:>3} ports  {:?}",
            z.id, z.key, z.pixels, degree, ports, z.ice
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_sim::scenario::{MapMeta, ProvinceDef, StateDef};

    fn square(lon: (f64, f64), lat: (f64, f64)) -> MultiPolygon<f64> {
        let ring = LineString::new(vec![
            Coord { x: lon.0, y: lat.0 },
            Coord { x: lon.1, y: lat.0 },
            Coord { x: lon.1, y: lat.1 },
            Coord { x: lon.0, y: lat.1 },
            Coord { x: lon.0, y: lat.0 },
        ]);
        MultiPolygon::new(vec![Polygon::new(ring, vec![])])
    }

    fn feature(name: &str, lon: (f64, f64), lat: (f64, f64)) -> sources::Feature {
        sources::Feature {
            fields: vec![
                ("name".into(), name.into()),
                ("featurecla".into(), "sea".into()),
            ],
            geometry: square(lon, lat),
        }
    }

    /// A world of two seas split by a land strip (one state), with a gap no polygon covers and
    /// a lake; one declared strait between the seas.
    fn world() -> (
        Frame,
        Vec<sources::Feature>,
        Vec<u16>,
        MapFile,
        SeaZonesFile,
    ) {
        let frame = Frame::new(360, 84.0, -58.0);
        let (w, h) = (frame.width as usize, frame.height as usize);
        let mut land = vec![0u16; w * h];
        for y in 0..h {
            for x in 170..190 {
                land[y * w + x] = 1;
            }
        }
        // A lake inside the land strip.
        land[(h / 2) * w + 180] = 0;
        let map = MapFile {
            meta: MapMeta {
                width: frame.width,
                height: frame.height,
                projection: "miller".into(),
                lat_top: 84.0,
                lat_bottom: -58.0,
                sources: vec![],
            },
            provinces: vec![ProvinceDef {
                id: 1,
                key: "P-1".into(),
                name: "P".into(),
                owner: "AAA".into(),
                center: (180.0, (h / 2) as f32),
                area_km2: 1.0,
                pixels: (20 * h) as u32,
                colony: false,
                state: 1,
            }],
            states: vec![StateDef {
                id: 1,
                key: "A-1".into(),
                name: "A".into(),
                owner: "AAA".into(),
                colony: false,
                center: (180.0, (h / 2) as f32),
                area_km2: 1.0,
                provinces: vec![1],
                neighbours: vec![],
            }],
        };
        let features = vec![
            feature("West Sea", (-180.0, -60.0), (-60.0, 85.0)),
            feature("East Sea", (60.0, 180.0), (-60.0, 85.0)),
        ];
        let rules: SeaZonesFile = ron::from_str(
            r#"#![enable(implicit_some)]
            (
                zones: [
                    (key: "west", name: "West", parts: [(feature: "West Sea")], ice: Free),
                    (key: "east", name: "East", parts: [(feature: "East Sea")], ice: Free),
                ],
                straits: [
                    (key: "narrows", name: "Narrows", kind: Strait, zones: ("west", "east"),
                        shores: ["A-1"], regime: "none", source: "test"),
                ],
            )"#,
        )
        .unwrap();
        (frame, features, land, map, rules)
    }

    #[test]
    fn the_generator_is_deterministic_and_fills_the_open_water() {
        let (frame, features, land, map, rules) = world();
        let (w, h) = (frame.width as usize, frame.height as usize);
        let (a, za) = build(&frame, &features, &land, w, h, &map, &rules).unwrap();
        let (b, zb) = build(&frame, &features, &land, w, h, &map, &rules).unwrap();
        assert_eq!(a, b);
        assert_eq!(za, zb);
        // Every sea pixel has a zone; the lake inside the land does not.
        for i in 0..w * h {
            if land[i] == 0 {
                assert_eq!(za[i] == 0, i == (h / 2) * w + 180, "pixel {i}");
            }
        }
        // The two seas touch across the date line and along the uncovered middle, but the
        // declared strait replaces that contact.
        assert_eq!(a.edges.len(), 1);
        assert_eq!(a.edges[0].strait, Some(0));
        assert!(a.edges[0].contact > 0);
        // The land strip is a port on both seas.
        let zones: Vec<u16> = a.ports.iter().map(|p| p.zone).collect();
        assert_eq!(zones, vec![1, 2]);
        assert_eq!(a.straits[0].shores, vec![1]);
    }

    #[test]
    fn bad_rules_are_refused() {
        let (frame, features, land, map, mut rules) = world();
        let (w, h) = (frame.width as usize, frame.height as usize);
        rules.zones[1].parts[0].feature = "Nowhere".into();
        rules.zones[0].ice = Ice::Seasonal;
        rules.straits[0].shores = vec!["B-2".into()];
        let err = build(&frame, &features, &land, w, h, &map, &rules).unwrap_err();
        for needle in [
            "no marine polygon named \"Nowhere\"",
            "without a source",
            "unknown state B-2",
        ] {
            assert!(err.contains(needle), "{err}");
        }
    }

    #[test]
    fn the_committed_graph_matches_its_rules() {
        let root = crate::repo_root();
        let rules: SeaZonesFile =
            ron::from_str(&fs::read_to_string(root.join("data/map/sea_zones.ron")).unwrap())
                .unwrap();
        let seas: SeasFile =
            ron::from_str(&fs::read_to_string(root.join("assets/map/seas.ron")).unwrap()).unwrap();
        let keys: Vec<&str> = rules.zones.iter().map(|z| z.key.as_str()).collect();
        let generated: Vec<&str> = seas.zones.iter().map(|z| z.key.as_str()).collect();
        assert_eq!(
            keys, generated,
            "regenerate: cargo run -p bf-mapgen --release -- seas"
        );
        let straits: Vec<&str> = rules.straits.iter().map(|s| s.key.as_str()).collect();
        let generated: Vec<&str> = seas.straits.iter().map(|s| s.key.as_str()).collect();
        assert_eq!(straits, generated);
        // seas.png holds the zone of every pixel, as counted in seas.ron.
        let file = File::open(root.join("assets/map/seas.png")).unwrap();
        let mut reader = png::Decoder::new(std::io::BufReader::new(file))
            .read_info()
            .unwrap();
        let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut bytes).unwrap();
        let mut pixels = vec![0u32; seas.zones.len() + 1];
        for &z in &bytes[..info.buffer_size()] {
            pixels[z as usize] += 1;
        }
        for z in &seas.zones {
            assert_eq!(pixels[z.id as usize], z.pixels, "{}", z.key);
        }
    }
}
