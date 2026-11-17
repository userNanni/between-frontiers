//! The world as each older format wrote it, frozen, and the step from each to the next.
//!
//! Postcard is not self-describing: a field added to the world, or to anything in it, does not
//! read as its default. So every change to what [`GameWorld`] serialises raises
//! [`super::WORLD_VERSION`], and the layout it replaces stays here as a struct that never
//! changes again, with a function that turns it into the next version.
//!
//! A frozen struct holds the current type of each part that has not changed since. When a part
//! changes (the economy gained its ledger, the demography its communities), its old layout is
//! frozen too, beside the part (`econ::state::EconomyBeforeLedger`,
//! `demography::DemographyBeforeCommunities`), and the structs here that held the part switch
//! to it.

use laplace_schelling::{Id, Table};
use serde::Deserialize;

use crate::game::{CensusRecord, Country, CountryState, GameWorld};

/// Version 0, before the communities in the population (2026-11-17); no border state and no
/// ledger either.
#[derive(Deserialize)]
pub struct V0 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::state::EconomyBeforeLedger,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyBeforeCommunities,
}

/// Version 1, the communities in the population; still no border state and no ledger.
#[derive(Deserialize)]
pub struct V1 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::state::EconomyBeforeLedger,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
}

/// Version 2, the border state (`borders`); no ledger of debts between states yet.
#[derive(Deserialize)]
pub struct V2 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::state::EconomyBeforeLedger,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
}

/// 0 → 1: the communities are filled on the next month.
pub fn v0_to_v1(w: V0) -> V1 {
    V1 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography.into(),
    }
}

/// 1 → 2: no border operation had run.
pub fn v1_to_v2(w: V1) -> V2 {
    V2 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: Default::default(),
    }
}

/// Versions 3 and 4 (4 only added enum variants at the end): the ledger; public works in four
/// categories and law effects without a research priority, as before research that costs.
#[derive(Deserialize)]
pub struct V4 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::compat::EconomyBeforeResearch,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
}

/// Version 5: research that costs; no foreign capital yet.
#[derive(Deserialize)]
pub struct V5 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
}

/// 4 → 5: research that costs; at the default research weight and no priority the world
/// hashes as it did.
pub fn v4_to_v5(w: V4) -> V5 {
    V5 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ.into(),
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
    }
}

/// 2 → 5: an empty ledger, which stays out of the hash; the economy of version 2 converts
/// straight to the one of version 5 (`EconomyBeforeLedger`).
pub fn v2_to_v5(w: V2) -> V5 {
    V5 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ.into(),
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
    }
}

/// 5 → 6: no foreign capital had moved; an empty `foreign` stays out of the hash.
pub fn v5_to_v6(w: V5) -> V6 {
    V6 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
        foreign: Default::default(),
    }
}

/// Version 6, foreign capital (`GameWorld::foreign`), before the influence state.
#[derive(Deserialize)]
pub struct V6 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: crate::foreign_stakes::ForeignState,
}

/// 6 → 7: the influence state (`GameWorld::influence`), empty, which hashes as nothing.
pub fn v6_to_v7(w: V6) -> V7 {
    V7 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
        foreign: w.foreign,
        influence: Default::default(),
    }
}

/// Version 7: the world with the influence state (#110), before the treaty state.
#[derive(Deserialize)]
pub struct V7 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: crate::foreign_stakes::ForeignState,
    influence: crate::influence::InfluenceState,
}

/// 7 → 8: the treaty state (recognition between states); empty, it stays out of the hash.
pub fn v7_to_v8(w: V7) -> V8 {
    V8 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
        foreign: w.foreign,
        influence: w.influence,
        treaties: Default::default(),
    }
}

/// Version 8: the treaty state (#145), before the forces.
#[derive(Deserialize)]
pub struct V8 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: crate::foreign_stakes::ForeignState,
    influence: crate::influence::InfluenceState,
    treaties: crate::treaty::TreatyState,
}

/// 8 → 9: the armed forces (`GameWorld::war`), empty: a world without forces hashes as before.
pub fn v8_to_v9(w: V8) -> V9 {
    V9 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
        foreign: w.foreign,
        influence: w.influence,
        treaties: w.treaties,
        war: Default::default(),
    }
}

/// Version 9: the armed forces, before the orders (slice A2).
#[derive(Deserialize)]
pub struct V9 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: crate::foreign_stakes::ForeignState,
    influence: crate::influence::InfluenceState,
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeOrders,
}

/// 9 → 10: the orders of forces (`CountryWar::queue`, `Formation::raised`), empty: a world where
/// nobody ordered anything hashes as before.
pub fn v9_to_v10(w: V9) -> GameWorld {
    GameWorld {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
        foreign: w.foreign,
        influence: w.influence,
        treaties: w.treaties,
        war: w.war.into(),
    }
}
