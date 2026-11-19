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
    econ: crate::econ::compat::EconomyBeforeTrade,
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

/// The foreign state of versions 6 to 11 (#109): stakes and expropriations, before the works
/// paid from abroad and the reinvested earnings.
#[derive(Default, Deserialize)]
pub struct ForeignV7 {
    stakes: Vec<crate::foreign_stakes::Stake>,
    expropriations: Vec<crate::foreign_stakes::Expropriation>,
}

/// Version 6, foreign capital (`GameWorld::foreign`), before the influence state.
#[derive(Deserialize)]
pub struct V6 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::compat::EconomyBeforeTrade,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: ForeignV7,
}

/// Version 7, the influence state (`GameWorld::influence`), before trade by pair.
#[derive(Deserialize)]
pub struct V7 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::compat::EconomyBeforeTrade,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: ForeignV7,
    influence: crate::influence::InfluenceState,
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
    econ: crate::econ::compat::EconomyBeforeTrade,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: ForeignV7,
    influence: crate::influence::InfluenceState,
    #[serde(deserialize_with = "treaties_before_agreements")]
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

/// Version 9, the armed forces (`GameWorld::war`, #115), before trade by pair.
#[derive(Deserialize)]
pub struct V9 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    econ: crate::econ::compat::EconomyBeforeTrade,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: ForeignV7,
    influence: crate::influence::InfluenceState,
    #[serde(deserialize_with = "treaties_before_agreements")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeOrders,
}

/// 9 → 10: trade by pair (`econ::trade::TradeState`) starts empty and stays out of the hash;
/// with the switch on it fills in the first week (the preferences start at the cost of
/// arrival).
pub fn v9_to_v10(w: V9) -> V10 {
    V10 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ.into(),
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
        foreign: w.foreign,
        influence: w.influence,
        treaties: w.treaties,
        war: w.war,
    }
}

/// Version 10: trade by pair (`Economy::trade`, #113), before schooling by cohort.
#[derive(Deserialize)]
pub struct V10 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    #[serde(deserialize_with = "econ_before_obligations")]
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: ForeignV7,
    influence: crate::influence::InfluenceState,
    #[serde(deserialize_with = "treaties_before_agreements")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeOrders,
}

/// 10 → 11: schooling by cohort and level (`GameWorld::education`), empty. With the rule
/// `schooling` off it stays empty; on, the demography builds it from 1925's data and the
/// state's literacy at its next month. An empty schooling hashes as nothing.
pub fn v10_to_v11(w: V10) -> V11 {
    V11 {
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
        war: w.war,
        education: Default::default(),
    }
}

/// Version 11: schooling by cohort (#152), before capital crossed borders. Versions 12 and 13
/// are taken by the orders and independence PRs.
#[derive(Deserialize)]
pub struct V11 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    #[serde(deserialize_with = "econ_before_obligations")]
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: ForeignV7,
    influence: crate::influence::InfluenceState,
    #[serde(deserialize_with = "treaties_before_agreements")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeOrders,
    education: crate::demography::education::EducationState,
}

/// 11 → 14: capital crosses borders (#117); no work was paid from abroad and no earnings were
/// kept, so the two new lists are empty and the hash stays.
pub fn v11_to_v14(w: V11) -> V14 {
    V14 {
        scenario: w.scenario,
        player: w.player,
        countries: w.countries,
        census: w.census,
        econ: w.econ,
        research: w.research,
        politics: w.politics,
        demography: w.demography,
        borders: w.borders,
        foreign: {
            let mut f = crate::foreign_stakes::ForeignState {
                expropriations: w.foreign.expropriations,
                ..Default::default()
            };
            for s in w.foreign.stakes {
                f.add(s);
            }
            f
        },
        influence: w.influence,
        treaties: w.treaties,
        war: w.war,
        education: w.education,
    }
}

/// Version 14: capital crosses borders (#117), before the orders of forces (slice A2).
#[derive(Deserialize)]
pub struct V14 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    #[serde(deserialize_with = "econ_before_obligations")]
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: crate::foreign_stakes::ForeignState,
    influence: crate::influence::InfluenceState,
    #[serde(deserialize_with = "treaties_before_agreements")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeOrders,
    education: crate::demography::education::EducationState,
}

/// 14 → 15: the orders of forces (`CountryWar::queue`, `Formation::raised`), empty: a world
/// where nobody ordered anything hashes as before.
pub fn v14_to_v15(w: V14) -> V15 {
    V15 {
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
        education: w.education,
    }
}

/// Formats 10 to 15 wrote the ledger without the record of each line: read the economy in that
/// layout and convert (every line open, nothing recorded as paid or written off).
fn econ_before_obligations<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<crate::econ::Economy, D::Error> {
    crate::econ::compat::EconomyBeforeObligations::deserialize(d).map(Into::into)
}

/// Formats 8 to 15 wrote the treaty state with recognition only.
fn treaties_before_agreements<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<crate::treaty::TreatyState, D::Error> {
    crate::treaty::TreatyStateBeforeAgreements::deserialize(d).map(Into::into)
}

/// Version 15: the orders of forces (#151); the ledger without the record of each line and the
/// treaty state with recognition only.
#[derive(Deserialize)]
pub struct V15 {
    scenario: String,
    player: Option<Id<Country>>,
    countries: Table<Country, CountryState>,
    census: Vec<CensusRecord>,
    #[serde(deserialize_with = "econ_before_obligations")]
    econ: crate::econ::Economy,
    research: crate::research::ResearchState,
    politics: crate::polity::PoliticsState,
    demography: crate::demography::DemographyState,
    borders: crate::borders::BordersState,
    foreign: crate::foreign_stakes::ForeignState,
    influence: crate::influence::InfluenceState,
    #[serde(deserialize_with = "treaties_before_agreements")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeCrises,
    education: crate::demography::education::EducationState,
}

/// 15 → 16: each line of the ledger keeps what was paid and written off and its standing (open
/// for every line read), closed lines apart; the agreements signed in play (none). Without the
/// record (`Switches::ledger_record`, off) the world hashes as it did.
pub fn v15_to_v16(w: V15) -> GameWorld {
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
        education: w.education,
        independence: Default::default(),
        levers: Default::default(),
    }
}

/// Version 16: the record of each line of the ledger and the agreements (#158), before the
/// independence state.
#[derive(Deserialize)]
pub struct V16 {
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
    #[serde(deserialize_with = "treaties_before_breaches")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeCrises,
    education: crate::demography::education::EducationState,
}

/// 16 → 17: the independence state (`GameWorld::independence`), empty, which hashes as nothing.
pub fn v16_to_v17(w: V16) -> GameWorld {
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
        education: w.education,
        independence: Default::default(),
        levers: Default::default(),
    }
}

/// Version 17: independence by movement (#136), before the budget's levers.
#[derive(Deserialize)]
pub struct V17 {
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
    #[serde(deserialize_with = "treaties_before_breaches")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeCrises,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
}

/// 17 → 18: the budget's levers (`GameWorld::levers`), empty: every country on the rule, and
/// the world hashes as before.
pub fn v17_to_v18(w: V17) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: Default::default(),
    }
}

/// Version 18: the budget's levers (#165), before the crises (slice B). 19 was held and is
/// never written.
#[derive(Deserialize)]
pub struct V18 {
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
    #[serde(deserialize_with = "treaties_before_breaches")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeCrises,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: LeversV18,
}

/// 18 → 20: crises and wars (`WarState::conflicts`), empty: a world in peace hashes as before.
pub fn v18_to_v20(w: V18) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers.into(),
    }
}

/// The budget's levers as versions 18 and 20 saved them, before the order of the public works.
#[derive(Deserialize)]
pub struct LeversV18 {
    countries: Vec<crate::econ::levers::Levers>,
}

impl From<LeversV18> for crate::econ::levers::LeversState {
    fn from(l: LeversV18) -> Self {
        Self {
            countries: l.countries,
            ..Default::default()
        }
    }
}

/// Version 20: crises and wars (#157), before the order of the public works.
#[derive(Deserialize)]
pub struct V20 {
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
    #[serde(deserialize_with = "treaties_before_breaches")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeCalls,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: LeversV18,
}

/// 20 → 21: no order fixed for the public works and none asked for; the levers stay.
pub fn v20_to_v21(w: V20) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers.into(),
    }
}

/// Formats 16 to 21 wrote the treaty state without breaches nor causes.
fn treaties_before_breaches<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<crate::treaty::TreatyState, D::Error> {
    crate::treaty::TreatyStateBeforeBreaches::deserialize(d).map(Into::into)
}

/// Version 21: the order of the public works (#176), before the breaches of clauses.
#[derive(Deserialize)]
pub struct V21 {
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
    #[serde(deserialize_with = "treaties_before_breaches")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeCalls,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 21 → 22: the breaches of clauses and the causes they give (`TreatyState::breaches`,
/// `causes`), none: a world without breaches hashes as before.
pub fn v21_to_v22(w: V21) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}

/// Version 22: the breaches of clauses in force (#181), before the calls of third parties (W4).
#[derive(Deserialize)]
pub struct V22 {
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
    #[serde(deserialize_with = "treaties_before_reputation")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeCalls,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 22 → 23: the calls of third parties to war (`Conflicts::calls`), empty: a world without a
/// call hashes as before.
pub fn v22_to_v23(w: V22) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}

/// Format 22 wrote the treaty state without the reputation.
fn treaties_before_reputation<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<crate::treaty::TreatyState, D::Error> {
    crate::treaty::TreatyStateBeforeReputation::deserialize(d).map(Into::into)
}

/// Version 23: the calls of third parties to war (#184), before the reputation.
#[derive(Deserialize)]
pub struct V23 {
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
    #[serde(deserialize_with = "treaties_before_reputation")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeFighting,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 23 → 24: the reputation of breaches (`TreatyState::reputation`), empty and not kept: a world
/// without it hashes as before.
pub fn v23_to_v24(w: V23) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}

/// Format 24 wrote the treaty state without the negotiation table.
fn treaties_before_table<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<crate::treaty::TreatyState, D::Error> {
    crate::treaty::TreatyStateBeforeTable::deserialize(d).map(Into::into)
}

/// Version 24: the reputation of breaches, before the negotiation table.
#[derive(Deserialize)]
pub struct V24 {
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
    #[serde(deserialize_with = "treaties_before_table")]
    treaties: crate::treaty::TreatyState,
    war: crate::war::state::WarStateBeforeFighting,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 24 → 25: the negotiation table (`TreatyState::table`), empty: a world without a proposal
/// hashes as before.
pub fn v24_to_v25(w: V24) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}

/// Version 25: the negotiation table (#196), before the war on land (slice C).
#[derive(Deserialize)]
pub struct V25 {
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
    war: crate::war::state::WarStateBeforeFighting,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 25 → 26: the war on land (`WarState::fighting`), empty: a world in peace hashes as before.
pub fn v25_to_v26(w: V25) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}

/// Version 26: the war on land (#206), before the entrenchment of the fronts.
#[derive(Deserialize)]
pub struct V26 {
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
    war: crate::war::state::WarStateBeforeStill,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 26 → 27: the entrenchment of the fronts (`Fighting::still`), empty.
pub fn v26_to_v27(w: V26) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}

/// Version 27: the pace of the front (#224), before the war support and the occupation.
#[derive(Deserialize)]
pub struct V27 {
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
    war: crate::war::state::WarStateBeforeSupport,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 27 → 28: the war support, the occupation and the wrecks (`Fighting`), empty.
pub fn v27_to_v28(w: V27) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}

/// Version 28: the war support and the occupation (#217), before the composition of the forces.
#[derive(Deserialize)]
pub struct V28 {
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
    war: crate::war::state::WarStateBeforeComposition,
    education: crate::demography::education::EducationState,
    independence: crate::borders::independence::IndependenceState,
    levers: crate::econ::levers::LeversState,
}

/// 28 → 29: the composition of the forces (`WarState::composition`), empty.
pub fn v28_to_v29(w: V28) -> GameWorld {
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
        education: w.education,
        independence: w.independence,
        levers: w.levers,
    }
}
