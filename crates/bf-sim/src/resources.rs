//! Natural resources by state, as stored in the scenario files. Contract between the research
//! data (`data/scenario/1925/deposits.ron`, `land.ron`) and the economy.
//!
//! Keys are stable strings shared with `data/economy/*.ron` and the Fluent files
//! (`resource-<key>`, `farm-<key>`, `climate-<key>`).

use serde::{Deserialize, Serialize};

/// Mineral and energy resources. A deposit with `discovered` after the scenario start exists
/// in the ground but is hidden until it is found (prospecting, technology, or the year).
pub const RESOURCES: &[&str] = &[
    "coal",
    "iron_ore",
    "copper",
    "bauxite",
    "oil",
    "natural_gas",
    "gold",
    "lead_zinc",
    "tin",
    "nickel",
    "manganese",
    "chromium",
    "tungsten",
    "sulfur",
    "phosphate",
    "potash",
    "uranium",
    "rare_earths",
];

/// Kinds of farm and other land use. Each needs certain climates (see `docs/economy.pt-BR.md`).
pub const FARMS: &[&str] = &[
    "grain_farm",
    "rice_paddy",
    "maize_farm",
    "livestock_ranch",
    "dairy_farm",
    "cotton_plantation",
    "sugarcane_plantation",
    "sugar_beet_farm",
    "coffee_plantation",
    "tea_plantation",
    "cocoa_plantation",
    "rubber_plantation",
    "tobacco_plantation",
    "fruit_plantation",
    "fiber_plantation",
    "vineyard",
    "forestry",
    "fishery",
];

/// Köppen–Geiger main climates, plus the subgroups that decide crops.
pub const CLIMATES: &[&str] = &[
    "tropical_rainforest", // Af
    "tropical_monsoon",    // Am
    "tropical_savanna",    // Aw/As
    "hot_desert",          // BWh
    "cold_desert",         // BWk
    "hot_steppe",          // BSh
    "cold_steppe",         // BSk
    "mediterranean",       // Csa/Csb/Csc
    "humid_subtropical",   // Cfa/Cwa
    "oceanic",             // Cfb/Cfc/Cwb/Cwc
    "humid_continental",   // Dfa/Dfb/Dwa/Dwb/Dsa/Dsb
    "subarctic",           // Dfc/Dfd/Dwc/Dwd/Dsc/Dsd
    "tundra",              // ET
    "ice_cap",             // EF
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositsFile {
    /// Unit of `output_1925` and `potential` for each resource, e.g. `("copper", "kt Cu content")`.
    pub units: Vec<(String, String)>,
    pub states: Vec<StateDeposits>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateDeposits {
    /// State key from `assets/map/provinces.ron`.
    pub state: String,
    pub deposits: Vec<Deposit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deposit {
    /// One of [`RESOURCES`].
    pub resource: String,
    /// 1 small … 5 world class, judged against the world's deposits of that resource over
    /// the century.
    pub size: u8,
    /// Year the deposit was known well enough to mine; ≤ 1925 means known at the start.
    pub discovered: i32,
    /// Output in 1925, in the resource's unit; 0 when not mined yet.
    pub output_1925: f32,
    /// The most the deposit yielded in a year over the century: the long-run ceiling. How much
    /// of it a mine reaches in a given year depends on its production method.
    pub potential: f32,
    /// Named field or mine, e.g. "Katanga Copperbelt"; short source note.
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LandFile {
    pub states: Vec<StateLand>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateLand {
    pub state: String,
    /// Share of the state's land in each climate of [`CLIMATES`], summing to 1.
    pub climate: Vec<(String, f32)>,
    /// Cropland in use around 1925.
    pub cropland_km2: f32,
    /// Pasture and rangeland in use around 1925.
    pub pasture_km2: f32,
    /// Forest cover around 1925.
    pub forest_km2: f32,
    /// Whether the state has a sea coast (fisheries, ports).
    pub coast: bool,
    /// Farm kinds the state's climate and soils allow, with a suitability from 0 to 1.
    pub farms: Vec<(String, f32)>,
}
