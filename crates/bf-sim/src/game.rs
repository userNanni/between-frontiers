//! The game on the Laplace-Schelling engine: world state, commands, systems, sessions.
//!
//! Phase 0 has no economy yet. The world holds the scenario's countries as the simulation will
//! see them, the player's choice, and a yearly census of world totals. That is enough to run the
//! real tick, hash, save and replay path from the first campaign on.

use laplace_schelling::command::{CommandLog, Issuer, Stamped};
use laplace_schelling::hash::{HashState, StateHash, StateHasher, hash_of};
use laplace_schelling::save::{SaveError, SaveFile, SaveHeader, decode_snapshot, encode_snapshot};
use laplace_schelling::schedule::{Access, Cadence, Clock, ResourceId, Schedule, System, TickCtx};
use laplace_schelling::{Command, Date, Fixed, Id, Simulation, Table};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::econ::Catalogue;
use crate::scenario::CountryDef;

/// Identifier written in every save; a save of another game is refused.
pub const GAME_ID: &str = "between-frontiers";
/// Issuer of commands from the local player.
pub const PLAYER: Issuer = 0;

/// Marker for country ids.
pub struct Country;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryState {
    pub tag: String,
    /// Metropole population.
    pub population: i64,
    /// Metropole GDP per capita, 2011 international dollars.
    pub gdp_per_capita: i64,
    pub colonial_population: i64,
    pub colonial_gdp_per_capita: i64,
}

impl CountryState {
    pub fn from_def(def: &CountryDef) -> Self {
        let (cp, cg) = def.colonies.as_ref().map_or((0, 0), |c| {
            (c.population as i64, i64::from(c.gdp_per_capita))
        });
        Self {
            tag: def.tag.clone(),
            population: def.population as i64,
            gdp_per_capita: i64::from(def.gdp_per_capita),
            colonial_population: cp,
            colonial_gdp_per_capita: cg,
        }
    }

    /// GDP of metropole and colonies, in dollars.
    pub fn total_gdp(&self) -> i64 {
        self.population * self.gdp_per_capita
            + self.colonial_population * self.colonial_gdp_per_capita
    }
}

impl HashState for CountryState {
    fn hash_state(&self, h: &mut StateHasher) {
        h.write_str(&self.tag);
        h.write_i64(self.population);
        h.write_i64(self.gdp_per_capita);
        h.write_i64(self.colonial_population);
        h.write_i64(self.colonial_gdp_per_capita);
    }
}

/// World totals taken on January 1st.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CensusRecord {
    pub date: Date,
    pub population: i64,
    pub gdp: i64,
}

impl HashState for CensusRecord {
    fn hash_state(&self, h: &mut StateHasher) {
        self.date.hash_state(h);
        h.write_i64(self.population);
        h.write_i64(self.gdp);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameWorld {
    pub scenario: String,
    pub player: Option<Id<Country>>,
    pub countries: Table<Country, CountryState>,
    pub census: Vec<CensusRecord>,
    /// Markets, regions, pops and plants; empty when the game runs without an economy.
    #[serde(default)]
    pub econ: crate::econ::Economy,
    /// Stage of every technology in every country; empty without the research data.
    #[serde(default)]
    pub research: crate::research::ResearchState,
    /// Laws in force, interest groups and movements of every country; empty without the
    /// politics data.
    #[serde(default)]
    pub politics: crate::polity::PoliticsState,
    /// Age pyramids, vital rates and migration links; empty without an economy.
    #[serde(default)]
    pub demography: crate::demography::DemographyState,
}

impl HashState for GameWorld {
    fn hash_state(&self, h: &mut StateHasher) {
        h.section("scenario");
        h.write_str(&self.scenario);
        h.section("player");
        self.player.hash_state(h);
        h.section("countries");
        self.countries.hash_state(h);
        h.section("census");
        self.census.hash_state(h);
        h.section("economy");
        self.econ.hash_state(h);
        h.section("research");
        self.research.hash_state(h);
        // Old worlds without politics keep their hash.
        if !self.politics.is_empty() {
            h.section("politics");
            self.politics.hash_state(h);
        }
        // Worlds without an economy keep their hash.
        if !self.demography.is_empty() {
            h.section("demography");
            self.demography.hash_state(h);
        }
    }
}

impl GameWorld {
    pub fn from_scenario<'a>(
        scenario: &str,
        countries: impl IntoIterator<Item = &'a CountryDef>,
    ) -> Self {
        Self {
            scenario: scenario.to_owned(),
            player: None,
            countries: Table::from_vec(countries.into_iter().map(CountryState::from_def).collect()),
            census: Vec::new(),
            econ: crate::econ::Economy::default(),
            research: crate::research::ResearchState::default(),
            politics: crate::polity::PoliticsState::default(),
            demography: crate::demography::DemographyState::default(),
        }
    }

    pub fn country_by_tag(&self, tag: &str) -> Option<Id<Country>> {
        self.countries
            .iter()
            .find(|(_, c)| c.tag == tag)
            .map(|(id, _)| id)
    }

    pub fn player_tag(&self) -> Option<&str> {
        self.player
            .and_then(|id| self.countries.get(id))
            .map(|c| c.tag.as_str())
    }
}

/// What players (and later the AI) can do. Phase 0: choose the country.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameCommand {
    /// Takes control of a country; an unknown tag is a no-op.
    ChoosePlayer { tag: String },
    /// Points the country's research at a technology (index in the tree), or back to choosing
    /// by itself with `None`. Unknown tags and indices are no-ops.
    SetResearch { tag: String, tech: Option<u16> },
    /// Sets the income tax of each class (lower, middle, upper) in thousandths, capped at
    /// `econ::public::MAX_TAX`; `None` hands the rates back to the historical rule.
    SetTaxRates {
        tag: String,
        rates: Option<[u16; crate::econ::state::TAX_CLASSES]>,
    },
    /// Weights of public investment (infrastructure, education, health, urban housing), each
    /// capped at `econ::public::MAX_PRIORITY`. All zero stops new public works.
    SetPublicPriority {
        tag: String,
        weights: [u8; crate::econ::state::PUBLIC_CATEGORIES],
    },
    /// Opens or restricts moving house inside the country (internal passports).
    SetMigrationLaw {
        tag: String,
        law: crate::econ::migration::MigrationLaw,
    },
    /// Fixes the central bank's rate, in thousandths a year (50 is 5 %), capped at
    /// `econ::money::MAX_RATE`; `None` hands it back to the Taylor rule.
    SetPolicyRate { tag: String, rate: Option<u16> },
    /// Who the central bank answers to: the historical rule, its own rule, or the treasury.
    SetCentralBank {
        tag: String,
        bank: crate::econ::money::CentralBank,
    },
    /// Asks for a law (index in `politics::Politics::laws`) in place of the one in force in its
    /// group. The politics system enacts it on the same day; unknown tags and laws, and colonial
    /// laws for a country without colonies, are no-ops.
    SetLaw { tag: String, law: u16 },
}

impl Command<GameWorld> for GameCommand {
    fn apply(&self, ctx: &TickCtx, world: &mut GameWorld) {
        match self {
            GameCommand::ChoosePlayer { tag } => {
                if let Some(id) = world.country_by_tag(tag) {
                    world.player = Some(id);
                }
            }
            GameCommand::SetResearch { tag, tech } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                let r = &mut world.research;
                let known = r.stage.get(ci).map_or(0, Vec::len);
                if ci < r.focus.len() && tech.is_none_or(|t| usize::from(t) < known) {
                    r.focus[ci] = *tech;
                }
            }
            GameCommand::SetTaxRates { tag, rates } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                let Some(m) = world.econ.markets.get_mut(ci) else {
                    return;
                };
                let b = &mut m.budget;
                match rates {
                    Some(rates) => {
                        b.tax_auto = false;
                        for (r, v) in b.tax_rates.iter_mut().zip(rates) {
                            *r = Fixed::from_raw(i64::from(*v)).min(crate::econ::public::MAX_TAX);
                        }
                        crate::econ::public::cap_taxes(m);
                    }
                    None => {
                        b.tax_auto = true;
                        let base = crate::econ::public::historical_rates(ctx.date.year);
                        for (r, v) in b.tax_rates.iter_mut().zip(base) {
                            *r = (v * b.adjust).min(crate::econ::public::MAX_TAX);
                        }
                        crate::econ::public::cap_taxes(m);
                    }
                }
            }
            GameCommand::SetMigrationLaw { tag, law } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                if let Some(m) = world.econ.markets.get_mut(ci) {
                    m.laws.migration = *law;
                }
            }
            GameCommand::SetPolicyRate { tag, rate } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                if let Some(m) = world.econ.markets.get_mut(ci) {
                    m.money.rate_command = rate.map(|r| {
                        crate::econ::money::pct(i64::from(r)).min(crate::econ::money::MAX_RATE)
                    });
                }
            }
            GameCommand::SetCentralBank { tag, bank } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                if let Some(m) = world.econ.markets.get_mut(ci) {
                    m.money.bank = *bank;
                }
            }
            GameCommand::SetLaw { tag, law } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                if ci < world.politics.laws.len() {
                    world.politics.request(ci as u16, *law);
                }
            }
            GameCommand::SetPublicPriority { tag, weights } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                if let Some(m) = world.econ.markets.get_mut(ci) {
                    for (p, w) in m.budget.priority.iter_mut().zip(weights) {
                        *p = (*w).min(crate::econ::public::MAX_PRIORITY);
                    }
                }
            }
        }
    }
}

const COUNTRIES: ResourceId = ResourceId(0);
const CENSUS: ResourceId = ResourceId(1);

/// Records world population and GDP every January 1st.
pub struct Census;

impl System<GameWorld> for Census {
    type Changes = CensusRecord;

    fn name(&self) -> &'static str {
        "census"
    }

    fn cadence(&self) -> Cadence {
        Cadence::Yearly { month: 1, day: 1 }
    }

    fn access(&self) -> Access {
        Access::new()
            .read(COUNTRIES)
            .read(crate::econ::week::ECON)
            .write(CENSUS)
    }

    fn compute(&self, ctx: &TickCtx, world: &GameWorld) -> CensusRecord {
        let (population, gdp) = if world.econ.regions.is_empty() {
            world.countries.iter().fold((0i64, 0i64), |(p, g), (_, c)| {
                (p + c.population + c.colonial_population, g + c.total_gdp())
            })
        } else {
            (
                world.econ.regions.iter().map(|r| r.population()).sum(),
                world
                    .econ
                    .markets
                    .iter()
                    .map(|m| m.gdp_last_year.to_int_round())
                    .sum(),
            )
        };
        CensusRecord {
            date: ctx.date,
            population,
            gdp,
        }
    }

    fn apply(&self, _ctx: &TickCtx, record: CensusRecord, world: &mut GameWorld) {
        world.census.push(record);
    }
}

/// The systems of the game. Without a catalogue only the census runs (tools and old tests).
/// What the rules need besides the world: the economy catalogue and the technology tree.
#[derive(Clone)]
pub struct Rules {
    pub catalogue: Arc<Catalogue>,
    pub tree: Option<Arc<crate::research::TechTree>>,
    /// Laws, interest groups and movements; without them politics stands still.
    pub politics: Option<Arc<crate::politics::Politics>>,
}

pub fn schedule(threads: usize, rules: Option<Rules>) -> Schedule<GameWorld> {
    let mut s = Schedule::new(threads);
    s.add(Census);
    if let Some(Rules {
        catalogue: cat,
        tree,
        politics,
    }) = rules
    {
        if let Some(p) = &politics {
            s.add(crate::polity::PoliticsSystem::new(
                Arc::clone(p),
                Arc::clone(&cat),
            ));
        }
        s.add(crate::econ::week::Week {
            cat: Arc::clone(&cat),
            tree: tree.clone(),
        });
        s.add(crate::econ::monthly::Labour {
            cat: Arc::clone(&cat),
        });
        s.add(crate::demography::Demography::new(
            Arc::clone(&cat),
            politics,
        ));
        s.add(crate::econ::monthly::Investment {
            cat: Arc::clone(&cat),
            tree: tree.clone(),
        });
        if let Some(tree) = &tree {
            s.add(crate::research_system::Research {
                cat: Arc::clone(&cat),
                tree: Arc::clone(tree),
            });
        }
        s.add(crate::econ::yearly::Yearly { cat, tree });
    }
    s
}

pub type GameSim = Simulation<GameWorld, GameCommand>;

/// A running campaign: the simulation and the commands applied before it was last loaded.
pub struct Session {
    pub sim: GameSim,
    earlier: Vec<Stamped<GameCommand>>,
}

#[derive(Debug)]
pub enum LoadError {
    Save(SaveError),
    /// The snapshot decoded, but its hash is not the one recorded when it was written.
    HashMismatch {
        recorded: StateHash,
        actual: StateHash,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Save(e) => write!(f, "{e}"),
            LoadError::HashMismatch { recorded, actual } => {
                write!(
                    f,
                    "state hash {actual} differs from the recorded {recorded}"
                )
            }
        }
    }
}

impl std::error::Error for LoadError {}

impl Session {
    pub fn new(
        world: GameWorld,
        seed: u64,
        start: Date,
        threads: usize,
        rules: Option<Rules>,
    ) -> Self {
        Self {
            sim: Simulation::new(world, seed, start, schedule(threads, rules)),
            earlier: Vec::new(),
        }
    }

    pub fn world(&self) -> &GameWorld {
        self.sim.world()
    }

    pub fn clock(&self) -> Clock {
        self.sim.clock()
    }

    pub fn state_hash(&self) -> StateHash {
        hash_of(self.sim.world())
    }

    pub fn submit(&mut self, command: GameCommand) {
        self.sim.submit(PLAYER, command);
    }

    /// Every command applied in this campaign, oldest first.
    /// Every command of the campaign: applied before the last load, applied since, and queued
    /// for the next tick.
    pub fn history(&self) -> CommandLog<GameCommand> {
        let mut log = CommandLog::new();
        for entry in self
            .earlier
            .iter()
            .chain(self.sim.log().entries())
            .chain(self.sim.pending())
        {
            log.push(entry.clone());
        }
        log
    }

    /// Encodes a save: snapshot of the world now, plus the full command history.
    pub fn save(&self, game_version: &str, label: &str) -> Result<Vec<u8>, SaveError> {
        let mut header = SaveHeader::new(
            GAME_ID,
            game_version,
            self.sim.seed(),
            self.clock(),
            self.state_hash(),
        );
        header.label = label.to_owned();
        SaveFile {
            header,
            snapshot: encode_snapshot(self.sim.world())?,
            log: self.history(),
        }
        .encode()
    }

    /// Restores a campaign and checks the snapshot against its recorded hash.
    pub fn load(bytes: &[u8], threads: usize, rules: Option<Rules>) -> Result<Self, LoadError> {
        let file: SaveFile<GameCommand> =
            SaveFile::decode(bytes, GAME_ID).map_err(LoadError::Save)?;
        let world: GameWorld = decode_snapshot(&file.snapshot).map_err(LoadError::Save)?;
        let actual = hash_of(&world);
        if actual != file.header.state_hash {
            return Err(LoadError::HashMismatch {
                recorded: file.header.state_hash,
                actual,
            });
        }
        let mut sim = Simulation::resume(
            world,
            file.header.seed,
            file.header.clock,
            schedule(threads, rules),
        );
        // Commands stamped for ticks not yet run go back into the queue.
        sim.replay(&file.log);
        let earlier = file
            .log
            .entries()
            .iter()
            .filter(|e| e.tick < file.header.clock.tick)
            .cloned()
            .collect();
        Ok(Self { sim, earlier })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::Status;

    fn def(tag: &str, population: u64) -> CountryDef {
        CountryDef {
            tag: tag.into(),
            color: (0, 0, 0),
            status: Status::Sovereign,
            government: "presidential_republic".into(),
            capital_lon_lat: (0.0, 0.0),
            population,
            gdp_per_capita: 1_000,
            source: String::new(),
            literacy: None,
            life_expectancy: None,
            urbanization: None,
            social_source: String::new(),
            colonies: None,
            flag: None,
        }
    }

    fn session(threads: usize) -> Session {
        let defs = [def("BRA", 30_000_000), def("ARG", 10_000_000)];
        let mut s = Session::new(
            GameWorld::from_scenario("1925", &defs),
            42,
            Date::new(1925, 1, 1),
            threads,
            None,
        );
        s.submit(GameCommand::ChoosePlayer { tag: "BRA".into() });
        s
    }

    #[test]
    fn choosing_the_player_and_counting_the_world() {
        let mut s = session(1);
        s.sim.run_until(Date::new(1927, 1, 2));
        assert_eq!(s.world().player_tag(), Some("BRA"));
        assert_eq!(s.world().census.len(), 3);
        assert_eq!(s.world().census[0].population, 40_000_000);
        assert_eq!(s.world().census[0].gdp, 40_000_000_000);
    }

    #[test]
    fn save_load_keeps_the_state_and_the_history() {
        let mut s = session(2);
        s.sim.run_until(Date::new(1926, 6, 1));
        let bytes = s.save("test", "BRA").unwrap();
        let mut loaded = Session::load(&bytes, 1, None).unwrap();
        assert_eq!(loaded.state_hash(), s.state_hash());
        assert_eq!(loaded.clock(), s.clock());
        assert_eq!(loaded.history().len(), 1);
        // Both continue identically.
        s.sim.run_until(Date::new(1930, 1, 2));
        loaded.sim.run_until(Date::new(1930, 1, 2));
        assert_eq!(loaded.state_hash(), s.state_hash());
    }

    /// A command issued before the next tick (the player's choice on the first day, a law
    /// changed while paused) is in the save and applies after the load.
    #[test]
    fn a_save_keeps_the_commands_not_yet_applied() {
        let mut s = session(2);
        let bytes = s.save("test", "first day").unwrap();
        let mut loaded = Session::load(&bytes, 1, None).unwrap();
        assert_eq!(loaded.history().len(), 1);
        s.sim.run_until(Date::new(1925, 3, 1));
        loaded.sim.run_until(Date::new(1925, 3, 1));
        assert_eq!(loaded.world().player_tag(), Some("BRA"));
        assert_eq!(loaded.state_hash(), s.state_hash());
    }

    #[test]
    fn taxes_public_priorities_and_money_by_command() {
        use crate::econ::{Market, public};
        let mut s = session(1);
        s.sim.run_until(Date::new(1925, 1, 2));
        let mut world = s.world().clone();
        world.econ.markets = vec![Market::empty(0), Market::empty(0)];
        let mut s = Session::new(world, 42, Date::new(1925, 1, 2), 1, None);
        s.submit(GameCommand::SetTaxRates {
            tag: "BRA".into(),
            rates: Some([30, 80, 900]),
        });
        s.submit(GameCommand::SetPublicPriority {
            tag: "ARG".into(),
            weights: [0, 9, 99, 1],
        });
        s.submit(GameCommand::SetMigrationLaw {
            tag: "ARG".into(),
            law: crate::econ::migration::MigrationLaw::Restricted,
        });
        s.submit(GameCommand::SetTaxRates {
            tag: "XXX".into(),
            rates: Some([1, 1, 1]),
        });
        s.submit(GameCommand::SetPolicyRate {
            tag: "BRA".into(),
            rate: Some(65_000),
        });
        s.submit(GameCommand::SetCentralBank {
            tag: "ARG".into(),
            bank: crate::econ::money::CentralBank::Autonomous,
        });
        s.sim.run_until(Date::new(1925, 1, 4));
        let bra = &s.world().econ.markets[0].budget;
        assert!(!bra.tax_auto);
        assert_eq!(
            bra.tax_rates,
            [Fixed::from_raw(30), Fixed::from_raw(80), public::MAX_TAX]
        );
        assert_eq!(
            s.world().econ.markets[1].budget.priority,
            [0, 9, public::MAX_PRIORITY, 1]
        );
        assert_eq!(
            s.world().econ.markets[1].laws.migration,
            crate::econ::migration::MigrationLaw::Restricted
        );
        assert_eq!(
            s.world().econ.markets[0].money.rate_command,
            Some(crate::econ::money::MAX_RATE)
        );
        assert_eq!(
            s.world().econ.markets[1].money.bank,
            crate::econ::money::CentralBank::Autonomous
        );
        s.submit(GameCommand::SetTaxRates {
            tag: "BRA".into(),
            rates: None,
        });
        s.sim.run_until(Date::new(1925, 1, 6));
        let bra = &s.world().econ.markets[0].budget;
        assert!(bra.tax_auto);
        assert_eq!(bra.tax_rates, public::historical_rates(1925));
    }

    #[test]
    fn refuses_a_tampered_snapshot() {
        let s = session(1);
        let mut bytes = s.save("test", "").unwrap();
        let n = bytes.len();
        bytes[n / 2] ^= 0xFF;
        assert!(Session::load(&bytes, 1, None).is_err());
    }
}
