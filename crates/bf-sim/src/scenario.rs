//! Scenario data as stored in text files. These types are the contract between the data
//! pipeline (`bf-mapgen`, `data/scenario/*`) and the game.
//!
//! - `assets/map/provinces.png`: 16-bit grayscale PNG. Each pixel holds a province id;
//!   0 is water or land outside any province. Projection and extent are in [`MapMeta`].
//! - `assets/map/provinces.ron`: [`MapFile`], with the provinces and the states they form.
//! - `data/scenario/1925/countries.ron`: [`CountriesFile`].
//! - Names live in the Fluent files, never here: `country-<TAG>` (with an `.article`
//!   attribute), `country-<TAG>-capital`, `government-<key>`.

use serde::{Deserialize, Serialize};

/// Country tag: three upper-case ASCII letters, e.g. `BRA`.
pub type Tag = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapFile {
    pub meta: MapMeta,
    pub provinces: Vec<ProvinceDef>,
    /// States on the start date: groups of provinces with one owner, the unit of the
    /// simulation. Provinces are only drawn (and, later, fought over).
    #[serde(default)]
    pub states: Vec<StateDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapMeta {
    /// Texture size in pixels.
    pub width: u32,
    pub height: u32,
    /// Projection name, e.g. `"miller"`. The texture spans longitude -180..180.
    pub projection: String,
    /// Latitude of the top and bottom edges of the texture, in degrees.
    pub lat_top: f64,
    pub lat_bottom: f64,
    /// Where the geometry came from and under which licence.
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvinceDef {
    /// Pixel value in `provinces.png`, starting at 1, contiguous.
    pub id: u16,
    /// Stable key from the source data (Natural Earth `adm1_code`).
    pub key: String,
    /// Display name as in the source (proper noun, not translated).
    pub name: String,
    /// Owner on the scenario start date.
    pub owner: Tag,
    /// Label anchor in texture pixels.
    pub center: (f32, f32),
    pub area_km2: f32,
    /// Pixels in the texture; 0 when the province is too small to be drawn.
    pub pixels: u32,
    /// Colony of the owner on the start date: ruled directly but not part of the metropole
    /// (Victoria 3's unincorporated states). Its people count in the owner's colonial figures,
    /// never in the national ones.
    #[serde(default)]
    pub colony: bool,
    /// Id of the [`StateDef`] the province belongs to; 0 in maps written before states existed.
    #[serde(default)]
    pub state: u16,
}

/// A state: provinces of one owner, all colonial or all metropolitan, grouped by the rules in
/// `data/scenario/1925/states.ron`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateDef {
    /// Starting at 1, contiguous, in key order.
    pub id: u16,
    /// Stable key, e.g. `BRA-SP`: Natural Earth `adm0_a3`, a dash and the ISO 3166-2 suffix of
    /// a single admin-1 unit, or a code chosen in the rules for a group.
    pub key: String,
    /// Display name, a proper noun as in the sources (not translated).
    pub name: String,
    pub owner: Tag,
    /// Whether the state is a colony of its owner (see [`ProvinceDef::colony`]).
    #[serde(default)]
    pub colony: bool,
    /// Label anchor in texture pixels, inside the largest part of the state.
    pub center: (f32, f32),
    pub area_km2: f32,
    /// Province ids, ascending.
    pub provinces: Vec<u16>,
    /// States sharing a land border with this one (ids, ascending): what a railway or a power
    /// line can reach without the sea.
    #[serde(default)]
    pub neighbours: Vec<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountriesFile {
    /// ISO date, e.g. `"1925-01-01"`.
    pub start_date: String,
    pub countries: Vec<CountryDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountryDef {
    pub tag: Tag,
    /// Map colour, sRGB.
    pub color: (u8, u8, u8),
    pub status: Status,
    /// Key of `government-<key>` in the Fluent files.
    pub government: String,
    /// Capital position, for locating the capital province.
    pub capital_lon_lat: (f64, f64),
    /// Population of the metropole (non-colonial provinces) on the start date.
    pub population: u64,
    /// GDP per capita of the metropole, in 2011 international dollars (Maddison Project 2023
    /// unit).
    pub gdp_per_capita: u32,
    /// Free-text source note for `population` and `gdp_per_capita`.
    pub source: String,
    /// Share of adults (15 and older) in the metropole who can read and write, 0 to 1. `None`
    /// when unknown.
    #[serde(default)]
    pub literacy: Option<f32>,
    /// Life expectancy at birth, in years. `None` when unknown.
    #[serde(default)]
    pub life_expectancy: Option<f32>,
    /// Share of the population living in urban areas, 0 to 1. `None` when unknown.
    #[serde(default)]
    pub urbanization: Option<f32>,
    /// Free-text source note for `literacy`, `life_expectancy` and `urbanization`.
    #[serde(default)]
    pub social_source: String,
    /// Figures of the colonies taken together; `None` for a country without colonies.
    #[serde(default)]
    pub colonies: Option<ColonialStats>,
    /// Flag image under `assets/flags/`, e.g. `"BRA.png"`; `None` until one is drawn.
    #[serde(default)]
    pub flag: Option<String>,
}

/// Aggregate figures of a country's colonies on the start date.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColonialStats {
    pub population: u64,
    /// 2011 international dollars.
    pub gdp_per_capita: u32,
    #[serde(default)]
    pub literacy: Option<f32>,
    #[serde(default)]
    pub life_expectancy: Option<f32>,
    #[serde(default)]
    pub urbanization: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Sovereign,
    /// Self-governing or protected subject of another country (dominion, mandate, protectorate,
    /// satellite). Colonies under direct rule are provinces of the overlord instead.
    Subject {
        overlord: Tag,
        kind: SubjectKind,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectKind {
    Dominion,
    Mandate,
    Protectorate,
    Satellite,
    /// British Raj style: a separate administration under the overlord's crown.
    CrownColony,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MapFile {
        MapFile {
            meta: MapMeta {
                width: 8,
                height: 4,
                projection: "miller".to_string(),
                lat_top: 84.0,
                lat_bottom: -58.0,
                sources: vec!["test".to_string()],
            },
            provinces: vec![ProvinceDef {
                id: 1,
                key: "BRA-627".to_string(),
                name: "Rio de Janeiro".to_string(),
                owner: "BRA".to_string(),
                center: (1.5, 2.5),
                area_km2: 43_750.0,
                pixels: 3,
                colony: false,
                state: 1,
            }],
            states: vec![StateDef {
                id: 1,
                key: "BRA-RJ".to_string(),
                name: "Rio de Janeiro".to_string(),
                owner: "BRA".to_string(),
                colony: false,
                center: (1.5, 2.5),
                area_km2: 43_750.0,
                provinces: vec![1],
                neighbours: Vec::new(),
            }],
        }
    }

    #[test]
    fn map_file_round_trips_through_ron() {
        let map = sample();
        let text = ron::to_string(&map).unwrap();
        let back: MapFile = ron::from_str(&text).unwrap();
        assert_eq!(back.states, map.states);
        assert_eq!(back.provinces[0].state, 1);
    }

    #[test]
    fn maps_without_states_still_parse() {
        let text = r#"(
            meta: (width: 8, height: 4, projection: "miller", lat_top: 84.0, lat_bottom: -58.0,
                sources: []),
            provinces: [(id: 1, key: "BRA-627", name: "Rio de Janeiro", owner: "BRA",
                center: (1.5, 2.5), area_km2: 43750.0, pixels: 3)],
        )"#;
        let map: MapFile = ron::from_str(text).unwrap();
        assert!(map.states.is_empty());
        assert_eq!(map.provinces[0].state, 0);
        assert!(!map.provinces[0].colony);
    }
}
