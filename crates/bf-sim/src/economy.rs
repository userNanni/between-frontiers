//! Economy catalogue as stored in `data/economy/*.ron`: goods, buildings with their production
//! methods, professions and needs. Names live in the Fluent files: `good-<key>`,
//! `good-category-<key>`, `building-<key>`, `method-<key>`, `profession-<key>`, `need-<key>`.
//!
//! Quantities are per building level per year, in each good's unit; a unit is priced at
//! `base_price` 2011 international dollars before the market moves it.

use serde::{Deserialize, Serialize};

/// Markets a good belongs to, for the interface and for trade rules.
pub const GOOD_CATEGORIES: &[&str] = &[
    "staple_food",
    "food",
    "agricultural",
    "mineral",
    "energy",
    "industrial",
    "consumer",
    "capital",
    "military",
    "service",
    "public",
];

/// Occupations of the population table (state × community × profession).
pub const PROFESSIONS: &[&str] = &[
    "peasant",
    "farm_laborer",
    "laborer",
    "skilled_worker",
    "clerk",
    "engineer",
    "professional",
    "manager",
    "capitalist",
    "landowner",
    "civil_servant",
    "clergy",
    "soldier",
];

pub const BUILDING_GROUPS: &[&str] = &[
    "agriculture",
    "extraction",
    "heavy_industry",
    "light_industry",
    "energy",
    "infrastructure",
    "services",
    "military_industry",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoodsFile {
    pub goods: Vec<GoodDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoodDef {
    pub key: String,
    /// One of [`GOOD_CATEGORIES`].
    pub category: String,
    /// 2011 international dollars per unit, before supply and demand.
    pub base_price: u32,
    /// First year the good can exist; the technology that unlocks it may come later.
    pub available_from: i32,
    /// False for goods consumed where they are made (electricity, local services).
    pub tradable: bool,
    /// Made and used in the state itself (transport, electricity before the national grid):
    /// it clears region by region, a share of each surplus reaching the neighbouring states.
    #[serde(default)]
    pub local: bool,
    /// Technology whose industrial stage in a country lets the good flow across all of it (the
    /// national grid for electricity).
    #[serde(default)]
    pub national_with: Option<String>,
    /// What one unit is, for tooltips and calibration, e.g. "1 t of grain".
    pub unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingsFile {
    pub buildings: Vec<BuildingDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingDef {
    pub key: String,
    /// One of [`BUILDING_GROUPS`].
    pub group: String,
    /// Deposit it works (a key of `resources::RESOURCES`), for mines and wells.
    #[serde(default)]
    pub resource: Option<String>,
    /// Land use it needs (a key of `resources::FARMS`), for farms, forestry and fisheries.
    #[serde(default)]
    pub farm: Option<String>,
    /// Public service (school, hospital): the state builds it and buys all it makes, and its
    /// value added goes entirely to wages.
    #[serde(default)]
    pub public: bool,
    /// Urban self-employment (street trade, domestic service, workshops): nobody builds it; the
    /// labour market sizes it to the town's jobless each month, and all its value added is the
    /// pay of those who work it.
    #[serde(default)]
    pub informal: bool,
    /// Production methods, the oldest first. A building runs one at a time.
    pub methods: Vec<MethodDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MethodDef {
    /// Shared key, named in `method-<key>`.
    pub key: String,
    /// First year it can be adopted; the research phase will gate it by technology.
    pub available_from: i32,
    pub inputs: Vec<(String, f32)>,
    pub outputs: Vec<(String, f32)>,
    /// Workers per level, by profession.
    pub jobs: Vec<(String, u32)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeedsFile {
    pub needs: Vec<NeedDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeedDef {
    pub key: String,
    /// 1 survival, 2 basic, 3 comfort, 4 affluence. Lower tiers are bought first.
    pub tier: u8,
    /// Units per person per year at the reference income of the tier.
    pub per_capita: f32,
    /// Only the urban professions have it (`Some(true)`), only the rural ones (`Some(false)`),
    /// or everyone (`None`).
    #[serde(default)]
    pub urban: Option<bool>,
    /// Goods that satisfy the need and how well (1 = fully), so the population substitutes
    /// when prices move.
    pub goods: Vec<(String, f32)>,
    /// Tiers 3 and 4: weight of the need in the money left once every tier is met.
    #[serde(default)]
    pub spare: f32,
}

/// Format of `data/economy/purchases.ron`: what the state, the investment pool and the builders
/// buy, as shares per mille of the money spent, and the freight of each building group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PurchasesFile {
    /// What the state buys with its goods budget.
    pub state: Vec<(String, i64)>,
    /// What the investment pool spends on upkeep and housing.
    pub upkeep: Vec<(String, i64)>,
    /// Goods of a construction basket that are the building itself, delivered by the building
    /// trade as construction points.
    pub building_work: Vec<String>,
    /// One entry per building group.
    pub groups: Vec<GroupPurchases>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupPurchases {
    /// One of [`BUILDING_GROUPS`].
    pub group: String,
    /// Goods of one new level, per mille of its cost.
    pub construction: Vec<(String, i64)>,
    /// Transport bought per unit of output value at base prices, per mille (0: none).
    #[serde(default)]
    pub freight: i64,
    /// Research and development the group's firms want, per mille of the value it adds
    /// (0: none). Paid out of the investment pool (`docs/research.pt-BR.md`).
    #[serde(default)]
    pub research: i64,
}
