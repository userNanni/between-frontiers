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
pub fn v5_to_v6(w: V5) -> GameWorld {
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
        foreign: Default::default(),
    }
}
