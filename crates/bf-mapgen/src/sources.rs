//! Source datasets: download into `data/raw/`, check the pinned hash, unzip, read.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use geo::orient::Direction;
use geo::{Coord, LineString, MultiPolygon, Orient, Polygon};
use shapefile::dbase::FieldValue;

pub struct Source {
    /// Archive file name, also the stem of the shapefile inside it.
    pub stem: &'static str,
    pub url: &'static str,
    /// BLAKE3 of the archive. A new Natural Earth release changes it: check the release notes,
    /// then update the hash and regenerate the map.
    pub blake3: &'static str,
}

pub const ADMIN1: Source = Source {
    stem: "ne_10m_admin_1_states_provinces_lakes",
    url: "https://naciscdn.org/naturalearth/10m/cultural/ne_10m_admin_1_states_provinces_lakes.zip",
    blake3: "eff2688c02e83981e852b70f763aa2fa4d5031df88e3b68a403d22557fb29062",
};

pub const ADMIN0: Source = Source {
    stem: "ne_10m_admin_0_countries_lakes",
    url: "https://naciscdn.org/naturalearth/10m/cultural/ne_10m_admin_0_countries_lakes.zip",
    blake3: "315631c01405c147d8efde108e7c82279143a8413036cce6cb6b776119cb7439",
};

/// Natural Earth release the hashes above belong to.
pub const NATURAL_EARTH_VERSION: &str = "5.1.1";

/// Natural Earth 1:50m shaded relief raster (10800×5400, 8-bit, equirectangular).
pub const SHADED_RELIEF: Source = Source {
    stem: "SR_50M",
    url: "https://naciscdn.org/naturalearth/50m/raster/SR_50M.zip",
    blake3: "403c983172414b9617fcd1164f200f3831e30a31bbc57f46e4277a43a7c6cb08",
};

/// Makes sure the archive is in `raw_dir`, matches its hash and is unpacked. Returns the path of
/// the `.shp` file.
pub fn fetch(source: &Source, raw_dir: &Path) -> Result<PathBuf, String> {
    fetch_file(source, raw_dir, &format!("{}.shp", source.stem))
}

/// Like [`fetch`], for an archive whose useful file is `inner` (relative to the unpacked folder).
pub fn fetch_file(source: &Source, raw_dir: &Path, inner: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(raw_dir).map_err(|e| format!("{}: {e}", raw_dir.display()))?;
    let archive = raw_dir.join(format!("{}.zip", source.stem));
    if !archive.exists() {
        eprintln!("downloading {}", source.url);
        let partial = archive.with_extension("zip.part");
        let status = Command::new("curl")
            .args(["--fail", "--location", "--silent", "--show-error"])
            .args(["--retry", "3", "--output"])
            .arg(&partial)
            .arg(source.url)
            .status()
            .map_err(|e| format!("could not run curl: {e}"))?;
        if !status.success() {
            return Err(format!("curl failed for {}", source.url));
        }
        fs::rename(&partial, &archive).map_err(|e| e.to_string())?;
    }

    let bytes = fs::read(&archive).map_err(|e| format!("{}: {e}", archive.display()))?;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    if hash != source.blake3 {
        return Err(format!(
            "{} has BLAKE3 {hash}, expected {}. Delete it to download again, or, if Natural Earth \
             published a new release on purpose, update the hash in sources.rs.",
            archive.display(),
            source.blake3
        ));
    }

    let dir = raw_dir.join(source.stem);
    let shp = dir.join(inner);
    if !shp.exists() {
        let mut zip = zip::ZipArchive::new(io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        zip.extract(&dir)
            .map_err(|e| format!("unzip {}: {e}", archive.display()))?;
    }
    Ok(shp)
}

/// One source feature with the attributes the pipeline uses.
pub struct Feature {
    pub fields: Vec<(String, String)>,
    pub geometry: MultiPolygon<f64>,
}

impl Feature {
    pub fn get(&self, name: &str) -> &str {
        self.fields
            .iter()
            .find(|(k, _)| k == name)
            .map_or("", |(_, v)| v.as_str())
    }
}

/// Reads a polygon shapefile keeping only the listed text fields.
pub fn read_features(shp: &Path, fields: &[&str]) -> Result<Vec<Feature>, String> {
    let mut reader =
        shapefile::Reader::from_path(shp).map_err(|e| format!("{}: {e}", shp.display()))?;
    let mut out = Vec::new();
    for item in reader.iter_shapes_and_records_as::<shapefile::Polygon, shapefile::dbase::Record>()
    {
        let (shape, record) = item.map_err(|e| format!("{}: {e}", shp.display()))?;
        let fields = fields
            .iter()
            .map(|&name| {
                let value = match record.get(name) {
                    Some(FieldValue::Character(Some(s))) => s.trim().to_string(),
                    _ => String::new(),
                };
                (name.to_string(), value)
            })
            .collect();
        out.push(Feature {
            fields,
            geometry: to_multipolygon(shape),
        });
    }
    Ok(out)
}

fn ring_to_linestring(points: &[shapefile::Point]) -> LineString<f64> {
    LineString::new(points.iter().map(|p| Coord { x: p.x, y: p.y }).collect())
}

fn ring_contains(ring: &LineString<f64>, p: Coord<f64>) -> bool {
    let pts = &ring.0;
    let mut inside = false;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if (a.y <= p.y) != (b.y <= p.y) && p.x < a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y) {
            inside = !inside;
        }
    }
    inside
}

/// Shapefile rings to polygons: each hole goes to the exterior that contains it. Shapefiles wind
/// exteriors clockwise; the result follows the geo convention (exteriors counter-clockwise), which
/// the geodesic area needs.
fn to_multipolygon(shape: shapefile::Polygon) -> MultiPolygon<f64> {
    let mut exteriors: Vec<LineString<f64>> = Vec::new();
    let mut holes: Vec<LineString<f64>> = Vec::new();
    for ring in shape.rings() {
        match ring {
            shapefile::PolygonRing::Outer(points) => exteriors.push(ring_to_linestring(points)),
            shapefile::PolygonRing::Inner(points) => holes.push(ring_to_linestring(points)),
        }
    }
    let mut interiors: Vec<Vec<LineString<f64>>> = vec![Vec::new(); exteriors.len()];
    for hole in holes {
        // A hole may touch its exterior at a vertex, so vote with several of its vertices.
        let probes: Vec<Coord<f64>> = hole.0.iter().copied().take(16).collect();
        let best = exteriors
            .iter()
            .enumerate()
            .map(|(i, e)| (probes.iter().filter(|&&p| ring_contains(e, p)).count(), i))
            .filter(|&(votes, _)| votes > 0)
            .max_by_key(|&(votes, i)| (votes, std::cmp::Reverse(i)));
        match best {
            Some((_, i)) => interiors[i].push(hole),
            None => eprintln!("warning: dropped a hole outside every exterior ring"),
        }
    }
    MultiPolygon::new(
        exteriors
            .into_iter()
            .zip(interiors)
            .map(|(e, i)| Polygon::new(e, i))
            .collect(),
    )
    .orient(Direction::Default)
}
