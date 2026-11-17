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

/// Release of the marine polygons.
pub const MARINE_VERSION: &str = "5.1.0";

/// Natural Earth release the hashes above belong to.
pub const NATURAL_EARTH_VERSION: &str = "5.1.1";

/// Natural Earth 1:10m marine polygons: oceans, seas, gulfs, bays and straits, with names.
/// Version [`MARINE_VERSION`], as its `VERSION.txt` says.
pub const MARINE: Source = Source {
    stem: "ne_10m_geography_marine_polys",
    url: "https://naciscdn.org/naturalearth/10m/physical/ne_10m_geography_marine_polys.zip",
    blake3: "5ad0125cbe8d5a79d3a13a8614e0b46888d0896c788a9a2ca84210f670b65a19",
};

/// Natural Earth 1:10m rivers and lake centrelines, with names and a scale rank.
pub const RIVERS: Source = Source {
    stem: "ne_10m_rivers_lake_centerlines",
    url: "https://naciscdn.org/naturalearth/10m/physical/ne_10m_rivers_lake_centerlines.zip",
    blake3: "2a5def415e8de8e16185dc6b26c014066367b6ee4ade3cac51be2cdc1b166096",
};

/// Release of the river centrelines, as their `VERSION.txt` says.
pub const RIVERS_VERSION: &str = "5.0.0";

/// Global Lakes and Wetlands Database v2 (Lehner et al. 2025), combined classes, GeoTIFF, 15
/// arc-seconds, CC BY 4.0: https://doi.org/10.6084/m9.figshare.28519994. 925 MB: read by
/// [`fetch_member`], which hashes the archive as a stream and unpacks one file.
pub const GLWD: Source = Source {
    stem: "GLWD_v2_0_combined_classes_tif",
    url: "https://ndownloader.figshare.com/files/54001814",
    blake3: "28744db34bfd465f1e49aaf9f2123b321ac2e11162c7124af371362dd601a01e",
};

/// NOAA ETOPO5: elevation of the land and the sea floor on a 5-minute grid, 2160 rows (from
/// 90° N southwards) × 4320 columns (from 0° E eastwards), big-endian 16-bit metres, no header.
/// Free to use and redistribute, as stated by the PacIOOS mirror (the NOAA record did not open).
pub const ETOPO5: Source = Source {
    stem: "ETOPO5",
    url: "https://www.ngdc.noaa.gov/mgg/global/relief/ETOPO5/TOPO/ETOPO5/ETOPO5.DAT",
    blake3: "f4a2874fdc175d2eca90f568ae137a7bcd0409548530a6ec69e6b20bedd39189",
};

/// Downloads a file that is not an archive into `raw_dir/<stem>/<file name of the url>` and
/// checks its hash.
pub fn fetch_plain(source: &Source, raw_dir: &Path) -> Result<PathBuf, String> {
    let name = source
        .url
        .rsplit('/')
        .next()
        .ok_or("url without a file name")?;
    let dir = raw_dir.join(source.stem);
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(name);
    if !path.exists() {
        eprintln!("downloading {}", source.url);
        let partial = path.with_extension("part");
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
        fs::rename(&partial, &path).map_err(|e| e.to_string())?;
    }
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    if hash != source.blake3 {
        return Err(format!(
            "{} has BLAKE3 {hash}, expected {}. Delete it to download again, or update the hash \
             in sources.rs if the source published a new release on purpose.",
            path.display(),
            source.blake3
        ));
    }
    Ok(path)
}

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

/// Like [`fetch_file`] for a large archive: the hash is computed as a stream and only `member`
/// (its path inside the archive) is unpacked, to `raw_dir/<stem>/<file name>`. The archive can be
/// deleted after a run; the next run downloads it again.
pub fn fetch_member(source: &Source, raw_dir: &Path, member: &str) -> Result<PathBuf, String> {
    let dir = raw_dir.join(source.stem);
    let name = Path::new(member)
        .file_name()
        .ok_or("member without a file name")?;
    let out = dir.join(name);
    let stamp = dir.join(format!("{}.blake3", name.to_string_lossy()));
    // Unpacked before from an archive with the pinned hash: no need for the archive.
    if out.exists() && fs::read_to_string(&stamp).is_ok_and(|h| h.trim() == source.blake3) {
        return Ok(out);
    }
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
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
    let mut hasher = blake3::Hasher::new();
    let mut file = fs::File::open(&archive).map_err(|e| format!("{}: {e}", archive.display()))?;
    io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    let hash = hasher.finalize().to_hex().to_string();
    if hash != source.blake3 {
        return Err(format!(
            "{} has BLAKE3 {hash}, expected {}. Delete it to download again, or update the hash \
             in sources.rs if the source published a new release on purpose.",
            archive.display(),
            source.blake3
        ));
    }
    let file = fs::File::open(&archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(io::BufReader::new(file)).map_err(|e| e.to_string())?;
    let mut entry = zip
        .by_name(member)
        .map_err(|e| format!("{}: {member}: {e}", archive.display()))?;
    let mut target = fs::File::create(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    io::copy(&mut entry, &mut target).map_err(|e| e.to_string())?;
    fs::write(&stamp, source.blake3).map_err(|e| e.to_string())?;
    Ok(out)
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
