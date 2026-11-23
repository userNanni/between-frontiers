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
    /// Kept by the market when no buyer takes it (`econ::stock`, `Switches::market_stock`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stock: Option<StockDef>,
}

/// How a storable good is kept (`econ::stock`).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockDef {
    /// Most a market keeps, in weeks of its own output of the good.
    pub weeks: u16,
    /// Lowest price, in thousandths of the base price.
    pub floor: u32,
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
    /// A state firm (the weapon core plant, U2): its whole profit goes to the treasury, so its
    /// product costs the country what it took to make (inputs, wages, the works), whatever the
    /// price.
    #[serde(default)]
    pub state_firm: bool,
    /// Urban self-employment (street trade, domestic service, workshops): nobody builds it; the
    /// labour market sizes it to the town's jobless each month, and all its value added is the
    /// pay of those who work it.
    #[serde(default)]
    pub informal: bool,
    /// Traditional transport and the like (`Switches::rail_by_line`): laid only by the 1925
    /// setup; nobody builds new levels, and the old ones wear and close like any private plant.
    #[serde(default)]
    pub traditional: bool,
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
    /// Capital of a level relative to a level of the building's first method, where the
    /// building declares it for every method (and 1 for the first). Without it a level costs
    /// `CAPITAL_YEARS` of the value it adds, as every building did before
    /// (docs/economy.pt-BR.md, "Capital por método").
    #[serde(default)]
    pub capital: Option<f32>,
    /// Inputs that fade as a technology spreads in the country: `(good, technology)`. The use of
    /// the good falls with the technology's stage there (`econ::fade`): a substitute taken up
    /// gradually, with no decision (the radium of the hospitals and the cobalt of 1951).
    #[serde(default)]
    pub fading: Vec<(String, String)>,
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
    /// Trade with a partner (`docs/trade.pt-BR.md`).
    #[serde(default)]
    pub trade: TradeDef,
}

/// The parameters of trade with a partner, all design values.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeDef {
    pub reach: f32,
    pub sigma: u8,
    pub monthly_step: f32,
    pub reference_km: u32,
    pub sea_per_land: f32,
    pub land_detour: f32,
    /// How much a buyer leans toward its own product against a foreign one at the same cost,
    /// with gross trade (`game::Switches::trade_gross`).
    #[serde(default = "one")]
    pub home_bias: f32,
    /// Customs duties (`econ::customs`, `Switches::customs_duties`).
    #[serde(default)]
    pub customs: CustomsDef,
}

fn one() -> f32 {
    1.0
}

/// The data of customs duties (`econ::customs`): each of the owner's decisions a datum.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomsDef {
    /// The duty goes to the treasury (CT3).
    pub revenue: bool,
    /// How much less a country seeks abroad for a higher tariff, `1 / (1 + ε t)` (CT4; zero,
    /// none).
    pub elasticity: f32,
    /// Each country's 1925 rate of `diplomacy.ron`, moved by its trade law from there (CT5).
    pub rates_1925: bool,
}

impl Default for CustomsDef {
    fn default() -> Self {
        Self {
            revenue: true,
            elasticity: 1.0,
            rates_1925: true,
        }
    }
}

impl Default for TradeDef {
    fn default() -> Self {
        Self {
            reach: 0.85,
            sigma: 5,
            monthly_step: 1.0 / 6.0,
            reference_km: 1000,
            sea_per_land: 1.0 / 3.0,
            land_detour: 1.2,
            home_bias: 1.0,
            customs: CustomsDef::default(),
        }
    }
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
