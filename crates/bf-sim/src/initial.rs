//! Initial conditions of the 1925 economy, as stored in the scenario files. Contract between the
//! research data (`data/scenario/1925/population.ron`, `industry.ron`) and the economy, which
//! turns them into pops and building levels on the start date.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PopulationFile {
    pub states: Vec<StatePopulation>,
    /// Labour force structure per country (metropole and colonies apart).
    pub workforce: Vec<Workforce>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatePopulation {
    /// State key from `assets/map/provinces.ron`.
    pub state: String,
    /// People on 1925-01-01.
    pub population: u64,
    /// Share living in towns, 0 to 1.
    pub urban: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workforce {
    pub country: String,
    /// `false` for the metropole, `true` for its colonies taken together.
    pub colonies: bool,
    /// Share of the population that works or seeks work, 0 to 1.
    pub participation: f32,
    /// Share of the labour force in agriculture, industry (mining, manufacturing, construction,
    /// utilities) and services; sums to 1.
    pub agriculture: f32,
    pub industry: f32,
    pub services: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndustryFile {
    /// Unit of each quantity, e.g. `("steel", "kt")`, `("railways_km", "km")`.
    pub units: Vec<(String, String)>,
    pub countries: Vec<CountryIndustry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountryIndustry {
    pub country: String,
    /// Output in 1925 of goods from `data/economy/goods.ron` (steel, cement, textiles, paper,
    /// electricity, fuel, chemicals, fertilizer, aluminium, copper, automobiles…) and
    /// infrastructure stocks (`railways_km`, `merchant_fleet_grt`, `telephones`), in `units`.
    pub output: Vec<(String, f32)>,
    /// Free-text source note.
    pub source: String,
}
