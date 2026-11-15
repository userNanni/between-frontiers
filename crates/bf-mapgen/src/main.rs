//! Map preprocessing tool: builds `assets/map/provinces.png` and `assets/map/provinces.ron` from
//! Natural Earth admin-1 polygons and the 1925 ownership file. Deterministic: the same inputs
//! give byte-identical outputs. See `docs/map-pipeline.pt-BR.md`.

mod geom;
mod owners;
mod preview;
mod projection;
mod raster;
mod relief;
mod sources;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use bf_sim::scenario::{MapFile, MapMeta, ProvinceDef};
use geo::orient::Direction;
use geo::{
    Area, BooleanOps, BoundingRect, GeodesicArea, Intersects, LineString, MultiPolygon, Orient,
    Polygon,
};

use crate::owners::{OwnersFile, TAGS_1925};
use crate::projection::Frame;
use crate::raster::Ring;

const START_DATE: &str = "1925-01-01";
const WIDTH: u32 = 8192;
const LAT_TOP: f64 = 84.0;
const LAT_BOTTOM: f64 = -58.0;
/// Key prefix for admin-0 units that have no admin-1 feature (e.g. Bir Tawil).
const ADM0_KEY_PREFIX: &str = "ADM0-";

struct Province {
    key: String,
    name: String,
    owner: String,
    /// Natural Earth `adm0_a3` of the unit the province comes from (of the parent for a carve).
    adm0: String,
    geometry: MultiPolygon<f64>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn repo_root() -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    dir.canonicalize().unwrap_or(dir)
}

fn run() -> Result<(), String> {
    let root = repo_root();
    let raw = root.join("data/raw");
    let admin1_shp = sources::fetch(&sources::ADMIN1, &raw)?;
    let admin0_shp = sources::fetch(&sources::ADMIN0, &raw)?;

    let owners_path = root.join("data/scenario/1925/province_owners.ron");
    let owners_text =
        fs::read_to_string(&owners_path).map_err(|e| format!("{}: {e}", owners_path.display()))?;
    let owners = OwnersFile::parse(&owners_text)?;
    owners.check_tags()?;
    if owners.start_date != START_DATE {
        return Err(format!(
            "province_owners.ron is for {}, expected {START_DATE}",
            owners.start_date
        ));
    }

    let admin1 = sources::read_features(
        &admin1_shp,
        &["adm1_code", "adm0_a3", "name", "name_en", "admin"],
    )?;
    let admin0 = sources::read_features(&admin0_shp, &["ADM0_A3", "ADMIN"])?;
    let mut provinces = collect_provinces(&owners, admin1, admin0)?;
    apply_carves(&owners, &mut provinces)?;
    provinces.sort_by(|a, b| a.key.cmp(&b.key));
    let colony = mark_colonies(&owners, &provinces)?;
    if provinces.len() >= usize::from(u16::MAX) {
        return Err(format!(
            "{} provinces do not fit in 16 bits",
            provinces.len()
        ));
    }

    let frame = Frame::new(WIDTH, LAT_TOP, LAT_BOTTOM);
    let projected: Vec<Vec<Vec<Ring>>> = provinces
        .iter()
        .map(|p| project(&frame, &p.geometry))
        .collect();

    let mut buf = vec![0u16; frame.width as usize * frame.height as usize];
    for (i, polygons) in projected.iter().enumerate() {
        let id = (i + 1) as u16;
        for rings in polygons {
            raster::fill_polygon(&mut buf, frame.width, frame.height, rings, id);
        }
    }
    let mut pixels = vec![0u32; provinces.len() + 1];
    for &id in &buf {
        pixels[id as usize] += 1;
    }

    let defs: Vec<ProvinceDef> = provinces
        .iter()
        .zip(&projected)
        .enumerate()
        .map(|(i, (p, polygons))| {
            let id = i + 1;
            let largest = polygons
                .iter()
                .max_by(|a, b| geom::polygon_area(a).total_cmp(&geom::polygon_area(b)))
                .expect("every province has a polygon");
            let (cx, cy) = geom::label_point(largest, 0.5);
            ProvinceDef {
                id: id as u16,
                key: p.key.clone(),
                name: p.name.clone(),
                owner: p.owner.clone(),
                center: (round_to(cx, 100.0) as f32, round_to(cy, 100.0) as f32),
                area_km2: round_to(p.geometry.geodesic_area_unsigned() / 1e6, 10.0) as f32,
                pixels: pixels[id],
                colony: colony[i],
            }
        })
        .collect();

    // A wrongly wound ring yields the area of the rest of the globe; the largest admin-1 unit
    // (Sakha) is about 3.1 million km2.
    if let Some(p) = defs.iter().find(|p| p.area_km2 > 5.0e6) {
        return Err(format!("{} has an area of {} km2", p.key, p.area_km2));
    }

    let map = MapFile {
        meta: MapMeta {
            width: frame.width,
            height: frame.height,
            projection: "miller".to_string(),
            lat_top: round_to(frame.lat_top(), 1e9),
            lat_bottom: round_to(frame.lat_bottom(), 1e9),
            sources: vec![
                format!(
                    "Natural Earth {} 1:10m Admin 1 states and provinces with large lakes \
                     ({}), public domain, https://www.naturalearthdata.com",
                    sources::NATURAL_EARTH_VERSION,
                    sources::ADMIN1.stem
                ),
                format!(
                    "Natural Earth {} 1:10m Admin 0 countries with large lakes ({}), public \
                     domain, https://www.naturalearthdata.com",
                    sources::NATURAL_EARTH_VERSION,
                    sources::ADMIN0.stem
                ),
                "Owners on 1925-01-01: data/scenario/1925/province_owners.ron, CC BY-SA 4.0"
                    .to_string(),
            ],
        },
        provinces: defs,
    };

    let out_dir = root.join("assets/map");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let png_path = out_dir.join("provinces.png");
    write_id_png(&png_path, &frame, &buf)?;

    let relief_tif = sources::fetch_file(&sources::SHADED_RELIEF, &raw, "SR_50M.tif")?;
    let relief_source = relief::read_tiff(&relief_tif)?;
    let (relief_frame, relief_pixels) = relief::build(
        &relief_source,
        &frame,
        &buf,
        frame.lat_top(),
        frame.lat_bottom(),
    );
    relief::write_png(&out_dir.join("relief.png"), &relief_frame, &relief_pixels)?;
    write_ron(&out_dir.join("provinces.ron"), &map)?;
    fs::write(out_dir.join("ATTRIBUTION.md"), ATTRIBUTION).map_err(|e| e.to_string())?;

    write_previews(&root.join("target-preview"), &frame, &buf, &map)?;
    print_summary(&map, &png_path);
    Ok(())
}

fn round_to(v: f64, factor: f64) -> f64 {
    (v * factor).round() / factor
}

fn collect_provinces(
    owners: &OwnersFile,
    admin1: Vec<sources::Feature>,
    admin0: Vec<sources::Feature>,
) -> Result<Vec<Province>, String> {
    let excluded: BTreeSet<&str> = owners
        .excluded_countries
        .iter()
        .map(String::as_str)
        .collect();
    let mut missing_country = BTreeSet::new();
    let mut seen_countries = BTreeSet::new();
    let mut admin0_only = BTreeSet::new();
    let mut provinces = Vec::new();

    for f in &admin1 {
        let adm0 = f.get("adm0_a3");
        seen_countries.insert(adm0.to_string());
        if excluded.contains(adm0) {
            continue;
        }
        let key = f.get("adm1_code").to_string();
        let owner = match owners.provinces.get(&key).or(owners.countries.get(adm0)) {
            Some(owner) => owner.clone(),
            None => {
                missing_country.insert(adm0.to_string());
                continue;
            }
        };
        let name = [f.get("name"), f.get("name_en")]
            .into_iter()
            .find(|n| !n.is_empty())
            .map_or_else(|| format!("{} ({key})", f.get("admin")), str::to_string);
        provinces.push(Province {
            key,
            name,
            owner,
            adm0: adm0.to_string(),
            geometry: f.geometry.clone(),
        });
    }

    for f in &admin0 {
        let adm0 = f.get("ADM0_A3");
        if seen_countries.contains(adm0) || excluded.contains(adm0) {
            continue;
        }
        let Some(owner) = owners.countries.get(adm0) else {
            missing_country.insert(adm0.to_string());
            continue;
        };
        admin0_only.insert(adm0.to_string());
        // Some of these units (Bir Tawil, the Cyprus buffer zone) are already inside admin-1
        // polygons: keep only the part no admin-1 province covers, unless that is only slivers
        // (compared in planar degrees, where a sliver cannot wind into a globe-sized area).
        let original = f.geometry.unsigned_area();
        let mut rest = f.geometry.clone();
        let bbox = f.geometry.bounding_rect();
        for p in &provinces {
            if let (Some(a), Some(b)) = (bbox, p.geometry.bounding_rect())
                && a.intersects(&b)
            {
                rest = rest.difference(&p.geometry);
            }
        }
        if rest.0.is_empty() || rest.unsigned_area() < original * 0.01 {
            eprintln!("{ADM0_KEY_PREFIX}{adm0}: covered by admin-1 provinces, skipped");
            continue;
        }
        provinces.push(Province {
            key: format!("{ADM0_KEY_PREFIX}{adm0}"),
            name: f.get("ADMIN").to_string(),
            owner: owner.clone(),
            adm0: adm0.to_string(),
            geometry: rest.orient(Direction::Default),
        });
    }

    if !missing_country.is_empty() {
        return Err(format!(
            "no default owner for these adm0_a3 codes: {}",
            missing_country.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    let keys: BTreeSet<&str> = provinces.iter().map(|p| p.key.as_str()).collect();
    if keys.len() != provinces.len() {
        return Err("duplicate province keys in the source".to_string());
    }
    let unknown: Vec<&str> = owners
        .provinces
        .keys()
        .map(String::as_str)
        .filter(|k| !keys.contains(k))
        .collect();
    if !unknown.is_empty() {
        return Err(format!(
            "overrides for unknown adm1_code: {}",
            unknown.join(", ")
        ));
    }
    let unused: Vec<&str> = owners
        .countries
        .keys()
        .map(String::as_str)
        .filter(|c| !seen_countries.contains(*c) && !admin0_only.contains(*c))
        .collect();
    if !unused.is_empty() {
        return Err(format!(
            "country defaults never used: {}",
            unused.join(", ")
        ));
    }
    Ok(provinces)
}

fn apply_carves(owners: &OwnersFile, provinces: &mut Vec<Province>) -> Result<(), String> {
    for carve in &owners.carves {
        let key = carve.key();
        let Some(i) = provinces.iter().position(|p| p.key == carve.from) else {
            return Err(format!("carve {key}: no province {}", carve.from));
        };
        let mask = Polygon::new(LineString::from(carve.mask.clone()), vec![]);
        let piece = provinces[i].geometry.intersection(&mask);
        let rest = provinces[i].geometry.difference(&mask);
        if piece.0.is_empty() || rest.0.is_empty() {
            return Err(format!(
                "carve {key}: the mask must cut the province in two"
            ));
        }
        provinces[i].geometry = rest.orient(Direction::Default);
        let adm0 = provinces[i].adm0.clone();
        provinces.push(Province {
            key,
            name: carve.name.clone(),
            owner: carve.owner.clone(),
            adm0,
            geometry: piece.orient(Direction::Default),
        });
    }
    Ok(())
}

/// Colony flag per province, in the order of `provinces`. Checks that every entry of
/// `metropoles` matches the map: listed countries and keys must hold provinces of that owner,
/// and every listed owner must end up with both metropole and colonies.
fn mark_colonies(owners: &OwnersFile, provinces: &[Province]) -> Result<Vec<bool>, String> {
    let mut errors = Vec::new();
    for (tag, metropole) in &owners.metropoles {
        let owned: Vec<&Province> = provinces.iter().filter(|p| &p.owner == tag).collect();
        for country in &metropole.countries {
            if !owned.iter().any(|p| &p.adm0 == country) {
                errors.push(format!("{tag}: no province of {country}"));
            }
        }
        for key in metropole.provinces.iter().chain(&metropole.colonies) {
            match owned.iter().find(|p| &p.key == key) {
                None => errors.push(format!("{tag}: {key} is not a province of {tag}")),
                Some(p) => {
                    let in_countries = metropole.countries.contains(&p.adm0);
                    let as_colony = metropole.colonies.contains(key);
                    if in_countries != as_colony {
                        let list = if as_colony { "colonies" } else { "provinces" };
                        errors.push(format!("{tag}: {key} ({}) is redundant in {list}", p.adm0));
                    }
                }
            }
        }
        let metropole_count = owned
            .iter()
            .filter(|p| metropole.contains(&p.key, &p.adm0))
            .count();
        if metropole_count == 0 || metropole_count == owned.len() {
            errors.push(format!(
                "{tag}: {metropole_count} of {} provinces in the metropole",
                owned.len()
            ));
        }
    }
    if !errors.is_empty() {
        return Err(format!("metropoles: {}", errors.join("; ")));
    }
    Ok(provinces
        .iter()
        .map(|p| {
            owners
                .metropoles
                .get(&p.owner)
                .is_some_and(|m| !m.contains(&p.key, &p.adm0))
        })
        .collect())
}

/// Polygons in texture pixels; each polygon is its exterior followed by its holes.
fn project(frame: &Frame, geometry: &MultiPolygon<f64>) -> Vec<Vec<Ring>> {
    let ring =
        |ls: &LineString<f64>| -> Ring { ls.0.iter().map(|c| frame.to_pixel(c.x, c.y)).collect() };
    geometry
        .0
        .iter()
        .map(|poly| {
            std::iter::once(poly.exterior())
                .chain(poly.interiors())
                .map(ring)
                .collect()
        })
        .collect()
}

fn write_id_png(path: &Path, frame: &Frame, buf: &[u16]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), frame.width, frame.height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Sixteen);
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = buf.iter().flat_map(|v| v.to_be_bytes()).collect();
    writer.write_image_data(&bytes).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}

fn write_ron(path: &Path, map: &MapFile) -> Result<(), String> {
    // One province per line keeps diffs readable.
    let config = ron::ser::PrettyConfig::new().depth_limit(2);
    let body = ron::ser::to_string_pretty(map, config).map_err(|e| e.to_string())?;
    let text = format!(
        "// Generated by `cargo run -p bf-mapgen --release`; do not edit by hand.\n\
         // Ids are pixel values in provinces.png. Format: bf_sim::scenario::MapFile.\n{body}\n"
    );
    let back: MapFile =
        ron::from_str(&text).map_err(|e| format!("provinces.ron does not read back: {e}"))?;
    if back.provinces.len() != map.provinces.len() {
        return Err("provinces.ron does not read back".to_string());
    }
    fs::write(path, text).map_err(|e| e.to_string())
}

fn write_previews(dir: &Path, frame: &Frame, buf: &[u16], map: &MapFile) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tag_index: BTreeMap<&str, usize> =
        TAGS_1925.iter().enumerate().map(|(i, t)| (*t, i)).collect();
    let mut owner_of = vec![0usize; map.provinces.len() + 1];
    for p in &map.provinces {
        owner_of[p.id as usize] = tag_index[p.owner.as_str()];
    }
    // Countries that touch on the texture, so the preview can give them different colours.
    let mut touching = BTreeSet::new();
    let w = frame.width as usize;
    for (i, &id) in buf.iter().enumerate() {
        if id == 0 {
            continue;
        }
        let neighbours = [
            (i % w + 1 < w).then(|| i + 1),
            buf.get(i + w).map(|_| i + w),
        ];
        for j in neighbours.into_iter().flatten() {
            let other = buf[j];
            if other != 0 && owner_of[other as usize] != owner_of[id as usize] {
                let (a, b) = (owner_of[id as usize], owner_of[other as usize]);
                touching.insert((a.min(b), a.max(b)));
            }
        }
    }
    let colors = preview::country_colors(TAGS_1925.len(), &touching);
    let full = (0, 0, frame.width, frame.height);
    preview::write(
        &dir.join("owners-preview.png"),
        &preview::Grid::sample(buf, frame.width, full, 1),
        &owner_of,
        &colors,
    )?;
    preview::write(
        &dir.join("owners-preview-small.png"),
        &preview::Grid::sample(buf, frame.width, full, 4),
        &owner_of,
        &colors,
    )?;
    // Close-ups for checking borders: (name, west, north, east, south, step); a step of 0 means
    // a 4x zoom.
    let windows: [(&str, f64, f64, f64, f64, u32); 12] = [
        ("europe", -12.0, 71.0, 42.0, 34.0, 1),
        ("east-europe", 12.0, 62.0, 42.0, 43.0, 1),
        ("middle-east", 25.0, 42.0, 62.0, 11.0, 1),
        ("east-asia", 95.0, 55.0, 150.0, 15.0, 2),
        ("africa", -20.0, 38.0, 55.0, -36.0, 2),
        ("americas", -120.0, 50.0, -30.0, -56.0, 2),
        ("south-asia", 60.0, 40.0, 100.0, 5.0, 2),
        ("danzig", 15.0, 56.0, 24.0, 52.5, 0),
        ("aegean", 19.0, 42.0, 30.5, 34.5, 0),
        ("cameroon", 2.0, 14.0, 17.0, 1.0, 0),
        ("kwantung", 118.0, 42.0, 126.0, 37.0, 0),
        ("sakhalin", 138.0, 55.0, 158.0, 42.0, 0),
    ];
    for (name, west, north, east, south, step) in windows {
        let (x0, y0) = frame.to_pixel(west, north);
        let (x1, y1) = frame.to_pixel(east, south);
        let window = (
            x0 as u32,
            y0.max(0.0) as u32,
            x1 as u32,
            (y1 as u32).min(frame.height),
        );
        let grid = if step == 0 {
            preview::Grid::sample(buf, frame.width, window, 1).zoom(4)
        } else {
            preview::Grid::sample(buf, frame.width, window, step)
        };
        preview::write(
            &dir.join(format!("owners-{name}.png")),
            &grid,
            &owner_of,
            &colors,
        )?;
    }
    Ok(())
}

fn print_summary(map: &MapFile, png_path: &Path) {
    let mut per_tag: BTreeMap<&str, (usize, u64)> = BTreeMap::new();
    let mut colonies: BTreeMap<&str, usize> = BTreeMap::new();
    for p in &map.provinces {
        let e = per_tag.entry(p.owner.as_str()).or_default();
        e.0 += 1;
        e.1 += u64::from(p.pixels);
        if p.colony {
            *colonies.entry(p.owner.as_str()).or_default() += 1;
        }
    }
    let zero = map.provinces.iter().filter(|p| p.pixels == 0).count();
    let size = fs::metadata(png_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "{} provinces ({} without pixels), {}x{}, provinces.png {} bytes",
        map.provinces.len(),
        zero,
        map.meta.width,
        map.meta.height,
        size
    );
    let without: Vec<&str> = TAGS_1925
        .iter()
        .copied()
        .filter(|t| !per_tag.contains_key(t))
        .collect();
    println!("tags without provinces: {without:?}");
    let invisible: Vec<&str> = per_tag
        .iter()
        .filter(|(_, v)| v.1 == 0)
        .map(|(k, _)| *k)
        .collect();
    println!("tags without pixels: {invisible:?}");
    println!("colonial provinces per owner: {colonies:?}");
}

const ATTRIBUTION: &str = "\
# Map data attribution

`provinces.png`, `provinces.ron` and `relief.png` are generated by `bf-mapgen` from:

- **Natural Earth** 1:10m cultural vectors, version 5.1.1: *Admin 1 – States, Provinces* (with
  large lakes) and *Admin 0 – Countries* (with large lakes). Made with Natural Earth. Free vector
  and raster map data @ naturalearthdata.com. Natural Earth is in the public domain.
- **Natural Earth** 1:50m *Shaded Relief* raster, reprojected to the map's Miller frame and
  centred on flat ground for `relief.png`. Public domain.
- **Owners on 1925-01-01**: `data/scenario/1925/province_owners.ron`, written for Between
  Frontiers and released under CC BY-SA 4.0 with the rest of the game data.

Natural Earth shows modern boundaries; where they cannot express the 1925 border the owner is an
approximation, listed in `docs/map-pipeline.pt-BR.md`.
";

#[cfg(test)]
mod tests {
    use super::*;

    fn committed_map() -> MapFile {
        let path = repo_root().join("assets/map/provinces.ron");
        let text = fs::read_to_string(&path).expect("read assets/map/provinces.ron");
        ron::from_str(&text).expect("parse provinces.ron")
    }

    #[test]
    fn ids_are_contiguous_and_keys_sorted() {
        let map = committed_map();
        assert!(!map.provinces.is_empty());
        for (i, p) in map.provinces.iter().enumerate() {
            assert_eq!(usize::from(p.id), i + 1, "{}", p.key);
            assert!(owners::is_tag(&p.owner), "{} owned by {}", p.key, p.owner);
        }
        assert!(map.provinces.windows(2).all(|w| w[0].key < w[1].key));
    }

    #[test]
    fn colonies_belong_to_owners_with_a_metropole() {
        let map = committed_map();
        let text =
            fs::read_to_string(repo_root().join("data/scenario/1925/province_owners.ron")).unwrap();
        let owners = OwnersFile::parse(&text).unwrap();
        let with_colonies: BTreeSet<&str> = map
            .provinces
            .iter()
            .filter(|p| p.colony)
            .map(|p| p.owner.as_str())
            .collect();
        let listed: BTreeSet<&str> = owners.metropoles.keys().map(String::as_str).collect();
        assert_eq!(with_colonies, listed);
    }

    #[test]
    fn png_matches_the_province_list() {
        let map = committed_map();
        let file = File::open(repo_root().join("assets/map/provinces.png")).unwrap();
        let mut reader = png::Decoder::new(std::io::BufReader::new(file))
            .read_info()
            .unwrap();
        let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut bytes).unwrap();
        assert_eq!((info.width, info.height), (map.meta.width, map.meta.height));
        assert_eq!(info.color_type, png::ColorType::Grayscale);
        assert_eq!(info.bit_depth, png::BitDepth::Sixteen);
        let mut pixels = vec![0u32; map.provinces.len() + 1];
        for px in bytes.as_chunks::<2>().0 {
            let id = usize::from(u16::from_be_bytes(*px));
            assert!(
                id <= map.provinces.len(),
                "pixel value {id} has no province"
            );
            pixels[id] += 1;
        }
        for p in &map.provinces {
            assert_eq!(pixels[usize::from(p.id)], p.pixels, "{}", p.key);
        }
    }
}
