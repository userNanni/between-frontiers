//! Read-only view of the scenario for the client: countries, provinces and derived figures.
//!
//! Phase 0 has no simulation state yet, so this is built straight from the scenario files.

use std::path::Path;

use bf_sim::indicators::{QualityOfLife, quality_of_life};
use bf_sim::scenario::{CountriesFile, CountryDef, MapFile, ProvinceDef, Status};
use laplace_schelling::calendar::Date;

pub struct Province {
    pub def: ProvinceDef,
    /// Index into [`World::countries`].
    pub owner: Option<usize>,
}

pub struct Country {
    pub def: CountryDef,
    pub provinces: Vec<u16>,
    pub area_km2: f64,
    /// GDP in 2011 international dollars.
    pub gdp: f64,
    pub rank_population: usize,
    pub rank_gdp: usize,
    /// Present when literacy and life expectancy are known.
    pub quality: Option<QualityOfLife>,
    pub rank_quality: Option<usize>,
    pub overlord: Option<usize>,
    pub subjects: Vec<usize>,
    /// Province holding the capital, when the map has one under it.
    pub capital_province: Option<u16>,
}

impl Country {
    pub fn tag(&self) -> &str {
        &self.def.tag
    }

    pub fn is_subject(&self) -> bool {
        matches!(self.def.status, Status::Subject { .. })
    }
}

pub struct World {
    pub start: Date,
    pub countries: Vec<Country>,
    pub provinces: Vec<Province>,
    pub map: MapFile,
}

#[derive(Debug)]
pub struct WorldError(pub String);

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn read_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, WorldError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| WorldError(format!("{}: {e}", path.display())))?;
    ron::from_str(&text).map_err(|e| WorldError(format!("{}: {e}", path.display())))
}

fn parse_date(s: &str) -> Option<Date> {
    let mut it = s.split('-');
    let y = it.next()?.parse().ok()?;
    let m = it.next()?.parse().ok()?;
    let d = it.next()?.parse().ok()?;
    Some(Date::new(y, m, d))
}

impl World {
    pub fn load(map_ron: &Path, countries_ron: &Path) -> Result<Self, WorldError> {
        let map: MapFile = read_ron(map_ron)?;
        let countries: CountriesFile = read_ron(countries_ron)?;
        Ok(Self::build(map, countries))
    }

    pub fn build(map: MapFile, file: CountriesFile) -> Self {
        let start = parse_date(&file.start_date).unwrap_or(Date::new(1925, 1, 1));
        let index_of = |tag: &str| file.countries.iter().position(|c| c.tag == tag);
        let mut provinces: Vec<Province> = map
            .provinces
            .iter()
            .map(|def| Province {
                owner: index_of(&def.owner),
                def: def.clone(),
            })
            .collect();
        provinces.sort_by_key(|p| p.def.id);

        let mut countries: Vec<Country> = file
            .countries
            .iter()
            .map(|def| Country {
                gdp: def.population as f64 * def.gdp_per_capita as f64,
                def: def.clone(),
                provinces: Vec::new(),
                area_km2: 0.0,
                rank_population: 0,
                rank_gdp: 0,
                quality: match (def.life_expectancy, def.literacy) {
                    (Some(life), Some(lit)) => Some(quality_of_life(
                        def.gdp_per_capita as f64,
                        life as f64,
                        lit as f64,
                    )),
                    _ => None,
                },
                rank_quality: None,
                overlord: None,
                subjects: Vec::new(),
                capital_province: None,
            })
            .collect();
        for p in &provinces {
            if let Some(o) = p.owner {
                countries[o].provinces.push(p.def.id);
                countries[o].area_km2 += p.def.area_km2 as f64;
            }
        }
        for i in 0..countries.len() {
            if let Status::Subject { overlord, .. } = &countries[i].def.status
                && let Some(o) = index_of(overlord)
            {
                countries[i].overlord = Some(o);
                countries[o].subjects.push(i);
            }
        }
        let mut order: Vec<usize> = (0..countries.len()).collect();
        order.sort_by(|&a, &b| {
            countries[b]
                .def
                .population
                .cmp(&countries[a].def.population)
        });
        for (rank, &i) in order.iter().enumerate() {
            countries[i].rank_population = rank + 1;
        }
        order.sort_by(|&a, &b| countries[b].gdp.total_cmp(&countries[a].gdp));
        for (rank, &i) in order.iter().enumerate() {
            countries[i].rank_gdp = rank + 1;
        }
        let mut with_quality: Vec<usize> = (0..countries.len())
            .filter(|&i| countries[i].quality.is_some())
            .collect();
        with_quality.sort_by(|&a, &b| {
            let q = |i: usize| countries[i].quality.map_or(0.0, |q| q.index);
            q(b).total_cmp(&q(a))
        });
        for (rank, &i) in with_quality.iter().enumerate() {
            countries[i].rank_quality = Some(rank + 1);
        }
        Self {
            start,
            countries,
            provinces,
            map,
        }
    }

    /// Finds the capital provinces from the capital coordinates, using the projection.
    pub fn locate_capitals(&mut self, ids: &[u16]) {
        let (w, h) = (self.map.meta.width, self.map.meta.height);
        for c in &mut self.countries {
            let (lon, lat) = c.def.capital_lon_lat;
            let Some((x, y)) = project(&self.map.meta, lon, lat) else {
                continue;
            };
            let (x, y) = ((x as u32).min(w - 1), (y as u32).min(h - 1));
            let id = ids[(y * w + x) as usize];
            if id != 0 {
                c.capital_province = Some(id);
            }
        }
    }
}

/// Longitude/latitude to texel, for the projections the map tool writes.
pub fn project(meta: &bf_sim::scenario::MapMeta, lon: f64, lat: f64) -> Option<(f32, f32)> {
    let y_of = |lat: f64| -> f64 {
        let phi = lat.to_radians();
        match meta.projection.as_str() {
            "miller" => 1.25 * (std::f64::consts::FRAC_PI_4 + 0.4 * phi).tan().ln(),
            "equirectangular" => phi,
            _ => phi,
        }
    };
    let (top, bottom) = (y_of(meta.lat_top), y_of(meta.lat_bottom));
    let x = (lon + 180.0) / 360.0 * meta.width as f64;
    let y = (top - y_of(lat)) / (top - bottom) * meta.height as f64;
    if !(0.0..meta.height as f64).contains(&y) {
        return None;
    }
    Some((x as f32, y as f32))
}
