//! The game on the Laplace-Schelling engine: world state, commands, systems, sessions.
//!
//! Phase 0 has no economy yet. The world holds the scenario's countries as the simulation will
//! see them, the player's choice, and a yearly census of world totals. That is enough to run the
//! real tick, hash, save and replay path from the first campaign on.
//!
//! **Adding a field to `GameWorld` or to anything in it changes the save's format.** Raise
//! `save_format::WORLD_VERSION`, freeze the old layout in `save_format::compat`, write the
//! step from it, add it to the test of every version and bless the new schema: the steps are
//! in [`crate::save_format`] and in `docs/saves.pt-BR.md`.

use laplace_schelling::command::{CommandLog, Issuer, Stamped};
use laplace_schelling::hash::{HashState, StateHash, StateHasher, hash_of};
use laplace_schelling::save::{SaveError, SaveFile, SaveHeader};
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
    /// Border operations queued and run, and the subjects' overlords once one ran.
    #[serde(default)]
    pub borders: crate::borders::BordersState,
    /// Plants built by foreign funds, by cohort, and the expropriations; empty until foreign
    /// capital moves (`docs/foreign-capital.pt-BR.md`).
    #[serde(default)]
    pub foreign: crate::foreign_stakes::ForeignState,
    /// Goodwill between countries and the influence map of the month; empty without the rules of
    /// influence.
    #[serde(default)]
    pub influence: crate::influence::InfluenceState,
    /// Recognition between states (and, as they enter the tick, agreements and breaches);
    /// empty without the treaty data.
    #[serde(default)]
    pub treaties: crate::treaty::TreatyState,
    /// Armed forces of every country; empty without the military data (`docs/war.pt-BR.md`,
    /// "Forças em paz").
    #[serde(default)]
    pub war: crate::war::WarState,
    /// Schooling by cohort and level and the adult literacy campaigns (`demography::education`).
    #[serde(default)]
    pub education: crate::demography::education::EducationState,
    /// What each government fixed over the budget's rule (`econ::levers`); empty while every
    /// country follows the rule. Each market carries a copy (`Market::levers`).
    #[serde(default)]
    pub levers: crate::econ::levers::LeversState,
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
        // Worlds where no border operation ever ran keep their hash.
        if !self.borders.is_empty() {
            h.section("borders");
            self.borders.hash_state(h);
        }
        // Worlds where no foreign capital ever moved keep their hash.
        if !self.foreign.is_empty() {
            h.section("foreign");
            self.foreign.hash_state(h);
        }
        // Worlds where nobody holds influence keep their hash.
        if !self.influence.is_empty() {
            h.section("influence");
            self.influence.hash_state(h);
        }
        // Worlds without treaty data keep their hash.
        if !self.treaties.is_empty() {
            h.section("treaties");
            self.treaties.hash_state(h);
        }
        // Worlds without forces keep their hash.
        if !self.war.is_empty() {
            h.section("war");
            self.war.hash_state(h);
        }
        // Worlds without schooling keep their hash.
        if !self.education.is_empty() {
            h.section("education");
            self.education.hash_state(h);
        }
        // Worlds where every country follows the budget's rule keep their hash.
        if !self.levers.is_empty() {
            h.section("levers");
            self.levers.hash_state(h);
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
            borders: crate::borders::BordersState::default(),
            foreign: Default::default(),
            influence: Default::default(),
            education: Default::default(),
            levers: Default::default(),
            demography: crate::demography::DemographyState::default(),
            treaties: Default::default(),
            war: crate::war::WarState::default(),
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
    /// Calls up the country's reserve (`on`) or sends it home; the forces system carries it out
    /// over the weeks of `reserve.call_up_weeks`.
    Mobilize { tag: String, on: bool },
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
    /// The issuer plays `tag` from now on (any time, any country of the world: the one born of
    /// its own revolt, a subject it set free, an heir). Session state: the world does not change
    /// and the hash neither; the command is in the log so that a save and a replay know who
    /// played what and when. An unknown tag is refused by the session.
    SwitchCountry { tag: String },
    /// A border operation (`borders::op`): run at the end of the day by the `borders` system,
    /// refused there with a reason when it does not apply. Issued by tests, tools and, later,
    /// treaties and peace; a player only frees its own land ([`GameCommand::Liberate`]).
    Borders { op: crate::borders::Operation },
    /// Country `by` frees some of its states as a new country (the catalogue's tag when the
    /// states are its core, `tag` when given, a generic one otherwise), independent or its subject.
    /// Who may issue it for `by` is the session's business: the simulation checks only that the
    /// states are `by`'s and that `by` keeps at least one.
    Liberate {
        by: String,
        states: Vec<String>,
        tag: Option<String>,
        subject: bool,
    },
    /// Starts (`on`) or stops the adult literacy campaign of a country, a budget programme of
    /// `data/economy/literacy_campaign.ron`'s years and cost (`demography::education`). The
    /// screen asks the player to confirm. Unknown tags are no-ops. Last in the list: saved
    /// command logs keep their variants' places.
    LiteracyCampaign { tag: String, on: bool },
    /// Fixes the servants and the clergy per thousand people (`econ::levers`, D1), cut to their
    /// floor and ceiling; `None` hands the payroll back to the rule. Unknown tags are no-ops.
    SetPayroll {
        tag: String,
        per_mille: Option<[u16; 2]>,
    },
    /// Fixes public pay over the private wage, in thousandths (the rule is 700; D2).
    SetPublicPay { tag: String, per_mille: Option<u16> },
    /// Fixes the weeks of revenue the treasury keeps and what its surplus is for (D4).
    SetTreasury {
        tag: String,
        reserve_weeks: Option<u16>,
        surplus: Option<crate::econ::levers::SurplusUse>,
    },
    /// Fixes the year's deficit for purchases, in thousandths of GDP (D5).
    SetDeficitCap {
        tag: String,
        per_mille_gdp: Option<u16>,
    },
}

/// Changes the levers of the country `tag` (`econ::levers`); unknown tags are no-ops.
fn set_levers(world: &mut GameWorld, tag: &str, f: impl FnOnce(&mut crate::econ::levers::Levers)) {
    let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == tag) else {
        return;
    };
    world.levers.set(&mut world.econ.markets, ci, f);
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
            GameCommand::Mobilize { tag, on } => {
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                if let Some(c) = world.war.countries.get_mut(ci) {
                    c.mobilized = *on;
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
            GameCommand::SwitchCountry { .. } => {}
            GameCommand::SetPayroll { tag, per_mille } => {
                let v = *per_mille;
                set_levers(world, tag, |l| l.payroll = v);
            }
            GameCommand::SetPublicPay { tag, per_mille } => {
                let v = *per_mille;
                set_levers(world, tag, |l| l.public_pay = v);
            }
            GameCommand::SetTreasury {
                tag,
                reserve_weeks,
                surplus,
            } => {
                let (w, s) = (*reserve_weeks, *surplus);
                set_levers(world, tag, |l| {
                    l.reserve_weeks = w;
                    l.surplus = s;
                });
            }
            GameCommand::SetDeficitCap { tag, per_mille_gdp } => {
                let v = *per_mille_gdp;
                set_levers(world, tag, |l| l.deficit_cap = v);
            }
            GameCommand::LiteracyCampaign { tag, on } => {
                // Without schooling (the rule off) there is nothing to teach with.
                if world.education.regions.is_empty() {
                    return;
                }
                let Some(ci) = world.countries.iter().position(|(_, c)| c.tag == *tag) else {
                    return;
                };
                let c = &mut world.education.campaigns;
                if c.len() <= ci {
                    c.resize(ci + 1, 0);
                }
                // The demography turns the start mark into the campaign's years.
                c[ci] = if *on {
                    crate::demography::education::CAMPAIGN_STARTS
                } else {
                    0
                };
            }
            GameCommand::Borders { op } => world.borders.pending.push(op.clone()),
            GameCommand::Liberate {
                by,
                states,
                tag,
                subject,
            } => world
                .borders
                .pending
                .push(crate::borders::Operation::Secede {
                    from: by.clone(),
                    states: states.clone(),
                    tag: tag.clone().unwrap_or_default(),
                    overlord: subject.then(|| by.clone()),
                    // Freeing one's own land: the old owner signs the creation.
                    signatories: vec![by.clone()],
                    terms: Default::default(),
                }),
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
    /// The community catalogue; without it the population carries no communities.
    pub communities: Option<Arc<crate::demography::communities::CommunityTable>>,
    /// Map, catalogue of countries and scenario the border operations read; without them border
    /// commands wait in the queue.
    pub borders: Option<Arc<crate::borders::BorderRules>>,
    /// Rules of influence (`data/diplomacy/influence.ron`); without them nobody holds any.
    pub influence: Option<Arc<crate::influence::InfluenceSetup>>,
    /// The unit catalogue and how the forces sit in peace; without them no forces run.
    pub war: Option<(Arc<crate::war::Military>, Arc<crate::war::PeaceRules>)>,
    /// Distances for the freight of trade; without them trade pays no freight.
    pub trade: Option<Arc<crate::econ::trade::Geography>>,
    /// Rules that change everyone's results, each off by default until a PR of its own
    /// switches it on.
    pub switches: Switches,
    /// Schooling of 1925 and the literacy campaign; without them literacy follows the yearly rule
    /// of the economy.
    pub education: Option<Arc<crate::demography::education::EducationRules>>,
}

/// Rules that can be switched on, all off by default: a rule enters the main switched off, with
/// the world's hash unchanged, and a later PR switches it on and is judged on its own. The
/// values the game starts with are in `data/rules.ron` (`GameData::switches`); a field missing
/// there is off. `BF_RULES` switches some on over the file when the data load, and
/// `bf-bench --rule NAME` for a run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Switches {
    /// Trade by pair (`econ::trade`, docs/trade.pt-BR.md): each buyer splits what it seeks among
    /// the sellers by the cost of arrival. Off, the world pool shares trade pro rata.
    pub trade_by_pair: bool,
    /// Jobs follow use (`econ::monthly::LAYOFF_PER_MONTH`): a plant that sells less than it
    /// could make lays off part of its crew each month. Off, every level employs its full crew
    /// whatever it sells.
    pub jobs_follow_use: bool,
    /// The wage floor of a small market (`econ::monthly::MIN_HANDS_FOR_A_WAGE`): a market with
    /// too few plant workers to have a wage of its own reads the lowest of the others.
    pub small_market_wage_floor: bool,
    /// Capital crosses borders (`econ::monthly::abroad`, docs/foreign-capital.pt-BR.md): the
    /// funds build abroad with what they have left at home; the stakes and the weekly
    /// remittance follow from what they build.
    pub foreign_flow: bool,
    /// Schooling by cohort and level and the adult literacy campaign (`demography::education`,
    /// docs/population.pt-BR.md, "Ensino por coorte"). Off, literacy follows the economy's yearly
    /// rule; a world that already has schooling keeps it.
    pub schooling: bool,
    /// Armed forces in peace (`war`, docs/war.pt-BR.md, "Forças em paz"): the soldiers are the
    /// men of the formations. Off, the old rule of `data/military/peace.ron` (`soldiers`).
    pub forces: bool,
    /// The servants' capacity to collect (`econ::levers::collection`, owner's decision O1): a
    /// region with fewer servants than the rule asks for collects that much less tax.
    pub fiscal_capacity: bool,
}

impl Switches {
    /// The names `switch_on` knows, in the order of the fields.
    pub const NAMES: [&'static str; 7] = [
        "trade_by_pair",
        "jobs_follow_use",
        "small_market_wage_floor",
        "foreign_flow",
        "schooling",
        "forces",
        "fiscal_capacity",
    ];

    /// Switches one rule on by name; unknown names are refused.
    pub fn switch_on(&mut self, name: &str) -> Result<(), String> {
        match name {
            "trade_by_pair" => self.trade_by_pair = true,
            "jobs_follow_use" => self.jobs_follow_use = true,
            "small_market_wage_floor" => self.small_market_wage_floor = true,
            "foreign_flow" => self.foreign_flow = true,
            "schooling" => self.schooling = true,
            "forces" => self.forces = true,
            "fiscal_capacity" => self.fiscal_capacity = true,
            other => {
                return Err(format!(
                    "unknown rule switch {other} (known: {})",
                    Self::NAMES.join(", ")
                ));
            }
        }
        Ok(())
    }

    /// These switches with the ones named in a comma-separated list on.
    pub fn with(mut self, names: &str) -> Result<Self, String> {
        for name in names.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            self.switch_on(name)?;
        }
        Ok(self)
    }

    /// The switches named in a comma-separated list on, the others off.
    pub fn parse(names: &str) -> Result<Self, String> {
        Self::default().with(names)
    }
}

pub fn schedule(threads: usize, rules: Option<Rules>) -> Schedule<GameWorld> {
    let mut s = Schedule::new(threads);
    s.add(Census);
    if let Some(Rules {
        catalogue: cat,
        tree,
        politics,
        communities,
        borders,
        influence,
        war,
        trade,
        switches,
        education,
    }) = rules
    {
        if let Some(p) = &politics {
            s.add(crate::polity::PoliticsSystem::new(
                Arc::clone(p),
                Arc::clone(&cat),
            ));
        }
        if let Some((military, peace)) = war {
            s.add(crate::war::Forces {
                cat: Arc::clone(&cat),
                military,
                peace,
                politics: politics.clone(),
            });
        }
        s.add(crate::econ::week::Week {
            cat: Arc::clone(&cat),
            tree: tree.clone(),
            geo: trade.filter(|_| switches.trade_by_pair),
            by_pair: switches.trade_by_pair,
            wage_floor: switches.small_market_wage_floor,
            foreign_flow: switches.foreign_flow,
            fiscal_capacity: switches.fiscal_capacity,
        });
        s.add(crate::econ::monthly::Labour {
            cat: Arc::clone(&cat),
            jobs_follow_use: switches.jobs_follow_use,
        });
        s.add(
            crate::demography::Demography::new(Arc::clone(&cat), politics)
                .with_communities(communities)
                .with_education(education),
        );
        s.add(crate::econ::monthly::Investment {
            cat: Arc::clone(&cat),
            tree: tree.clone(),
            wage_floor: switches.small_market_wage_floor,
            foreign_flow: switches.foreign_flow,
        });
        if let Some(tree) = &tree {
            s.add(crate::research_system::Research {
                cat: Arc::clone(&cat),
                tree: Arc::clone(tree),
                wage_floor: switches.small_market_wage_floor,
            });
        }
        s.add(crate::econ::yearly::Yearly {
            cat,
            tree,
            wage_floor: switches.small_market_wage_floor,
        });
        // Last in the day: an operation sees the day's results and the next day sees its own.
        if let Some(setup) = influence {
            s.add(crate::influence::InfluenceSystem { setup });
        }
        if let Some(rules) = borders {
            s.add(crate::borders::op::Borders { rules });
        }
    }
    s
}

pub type GameSim = Simulation<GameWorld, GameCommand>;

/// A running campaign: the simulation and the commands applied before it was last loaded.
pub struct Session {
    pub sim: GameSim,
    earlier: Vec<Stamped<GameCommand>>,
    /// The country each issuer plays, by [`GameCommand::ChoosePlayer`] and
    /// [`GameCommand::SwitchCountry`]: session state, not the world's (the simulation never
    /// reads it, so a switch does not change the hash). Rebuilt from the command log on load.
    players: std::collections::BTreeMap<Issuer, String>,
    /// Keys of the catalogues the rules index by position, written with every save.
    catalog: Option<crate::save_format::CatalogKeys>,
}

#[derive(Debug)]
pub enum LoadError {
    Save(SaveError),
    /// The save's technologies, goods or buildings no longer match the game's catalogues.
    Catalog(crate::save_format::CatalogError),
    /// The world was written in a newer format than this build reads.
    Newer {
        found: u32,
        supported: u32,
        /// Version of the game that wrote it.
        written_by: String,
    },
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
            LoadError::Catalog(e) => write!(f, "{e}"),
            LoadError::Newer {
                found,
                supported,
                written_by,
            } => write!(
                f,
                "written by version {written_by} in world format {found}; this build reads up to {supported}"
            ),
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

pub use crate::save_format::decode_world;

impl Session {
    pub fn new(
        world: GameWorld,
        seed: u64,
        start: Date,
        threads: usize,
        rules: Option<Rules>,
    ) -> Self {
        let catalog = rules.as_ref().map(crate::save_format::CatalogKeys::of);
        Self {
            sim: Simulation::new(world, seed, start, schedule(threads, rules)),
            earlier: Vec::new(),
            players: Default::default(),
            catalog,
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

    /// Issues `command` for the local player. Freeing land is refused (dropped) unless the
    /// land is the player's own country (decision 3 of `docs/borders.pt-BR.md`); the simulation
    /// itself only checks that the states are `by`'s.
    pub fn submit(&mut self, command: GameCommand) {
        self.submit_as(PLAYER, command);
    }

    /// Issues `command` for `issuer` (a player slot of a multiplayer server).
    pub fn submit_as(&mut self, issuer: Issuer, command: GameCommand) {
        match &command {
            GameCommand::Liberate { by, .. } if self.player_of(issuer) != Some(by.as_str()) => {
                return;
            }
            GameCommand::ChoosePlayer { tag } | GameCommand::SwitchCountry { tag } => {
                if self.world().country_by_tag(tag).is_none() {
                    return;
                }
                self.players.insert(issuer, tag.clone());
            }
            _ => {}
        }
        self.sim.submit(issuer, command);
    }

    /// The country the local player plays now.
    pub fn player(&self) -> Option<&str> {
        self.player_of(PLAYER)
    }

    /// The country `issuer` plays now; for a save older than the switch, the world's choice.
    pub fn player_of(&self, issuer: Issuer) -> Option<&str> {
        self.players.get(&issuer).map(String::as_str).or_else(|| {
            (issuer == PLAYER)
                .then(|| self.world().player_tag())
                .flatten()
        })
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
            snapshot: crate::save_format::encode_world(self.sim.world(), self.catalog.as_ref())?,
            log: self.history(),
        }
        .encode()
    }

    /// Restores a campaign and checks the snapshot against its recorded hash.
    pub fn load(bytes: &[u8], threads: usize, rules: Option<Rules>) -> Result<Self, LoadError> {
        let file: SaveFile<GameCommand> =
            SaveFile::decode(bytes, GAME_ID).map_err(LoadError::Save)?;
        let (mut world, saved) =
            crate::save_format::decode(&file.snapshot).map_err(|e| match e {
                crate::save_format::WorldError::Newer { found, supported } => LoadError::Newer {
                    found,
                    supported,
                    written_by: file.header.game_version.clone(),
                },
                crate::save_format::WorldError::Malformed(e) => LoadError::Save(e),
            })?;
        let actual = hash_of(&world);
        if actual != file.header.state_hash {
            return Err(LoadError::HashMismatch {
                recorded: file.header.state_hash,
                actual,
            });
        }
        // The world indexes technologies, goods and buildings by position: fit it to today's
        // catalogues (by key when the save has them) or refuse it here, never in play.
        if let Some(r) = &rules {
            crate::save_format::catalog::fit(&mut world, saved.as_ref(), r)
                .map_err(LoadError::Catalog)?;
        }
        let catalog = rules.as_ref().map(crate::save_format::CatalogKeys::of);
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
        let mut players = std::collections::BTreeMap::new();
        for e in file.log.entries() {
            if let GameCommand::ChoosePlayer { tag } | GameCommand::SwitchCountry { tag } =
                &e.command
            {
                players.insert(e.issuer, tag.clone());
            }
        }
        Ok(Self {
            sim,
            earlier,
            players,
            catalog,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::scenario::Status;
    use laplace_schelling::save::{decode_snapshot, encode_snapshot};

    /// A 1925 world written in every layout the saves had loads in this build with the hash it
    /// was saved with: this build; before the ledger; before the ledger and the border state;
    /// before the communities (and both). The ledger and the border state empty, as they were.
    #[test]
    fn every_version_loads_with_its_hash() {
        #[derive(Serialize)]
        struct OldEconomy<'a> {
            regions: &'a Vec<crate::econ::Region>,
            markets: &'a Vec<crate::econ::compat::MarketBeforeResearch>,
            world_price: &'a Vec<laplace_schelling::Fixed>,
            history: &'a Vec<crate::econ::YearRecord>,
        }
        /// The economy of before research that costs: the ledger, markets of then.
        #[derive(Serialize)]
        struct EconomyWithLedger<'a> {
            regions: &'a Vec<crate::econ::Region>,
            markets: &'a Vec<crate::econ::compat::MarketBeforeResearch>,
            world_price: &'a Vec<laplace_schelling::Fixed>,
            history: &'a Vec<crate::econ::YearRecord>,
            ledger: &'a crate::treaty::Ledger,
        }
        /// The economy of before trade by pair: this build's fields without `trade`.
        #[derive(Serialize)]
        struct EconomyNoTrade<'a> {
            regions: &'a Vec<crate::econ::Region>,
            markets: &'a Vec<crate::econ::Market>,
            world_price: &'a Vec<laplace_schelling::Fixed>,
            history: &'a Vec<crate::econ::YearRecord>,
            ledger: &'a crate::treaty::Ledger,
        }
        /// The foreign state of versions 6 to 11, before the works paid from abroad.
        #[derive(Serialize)]
        struct ForeignWithoutWorks {
            stakes: Vec<crate::foreign_stakes::Stake>,
            expropriations: Vec<crate::foreign_stakes::Expropriation>,
        }
        fn foreign_then(w: &GameWorld) -> ForeignWithoutWorks {
            ForeignWithoutWorks {
                stakes: w.foreign.stakes().collect(),
                expropriations: w.foreign.expropriations.clone(),
            }
        }
        /// Version 11: schooling by cohort, before capital crossed borders.
        #[derive(Serialize)]
        struct BeforeWorksAbroad<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: &'a crate::econ::Economy,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
            foreign: ForeignWithoutWorks,
            influence: &'a crate::influence::InfluenceState,
            treaties: &'a crate::treaty::TreatyState,
            war: &'a crate::war::WarState,
            education: &'a crate::demography::education::EducationState,
        }
        /// Version 9: this build's world with the economy of before trade by pair.
        #[derive(Serialize)]
        struct BeforeTrade<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: EconomyNoTrade<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
            foreign: ForeignWithoutWorks,
            influence: &'a crate::influence::InfluenceState,
            treaties: &'a crate::treaty::TreatyState,
            war: &'a crate::war::WarState,
        }
        /// Version 7: version 8 without the treaty state, its last field, and its economy.
        #[derive(Serialize)]
        struct BeforeTreaties<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: EconomyNoTrade<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
            foreign: ForeignWithoutWorks,
            influence: &'a crate::influence::InfluenceState,
        }
        /// The world of before the influence state: version 7 without its last field.
        #[derive(Serialize)]
        struct BeforeInfluence<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: EconomyNoTrade<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
            foreign: ForeignWithoutWorks,
        }
        /// The world of before the foreign stakes: version 6 without its last field.
        #[derive(Serialize)]
        struct BeforeForeign<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: EconomyNoTrade<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
        }
        /// The world of before the forces (version 8): this build's fields without the last one.
        #[derive(Serialize)]
        struct BeforeForces<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: EconomyNoTrade<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
            foreign: ForeignWithoutWorks,
            influence: &'a crate::influence::InfluenceState,
            treaties: &'a crate::treaty::TreatyState,
        }
        #[derive(Serialize)]
        struct BeforeResearch<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: EconomyWithLedger<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
        }
        /// `DemographyState` without `communities` (its last field).
        #[derive(Serialize)]
        struct OldDemography<'a> {
            ages: &'a Vec<[i64; crate::demography::AGE_GROUPS]>,
            schooling: &'a Vec<laplace_schelling::Fixed>,
            land_1925: &'a Vec<laplace_schelling::Fixed>,
            vital: &'a Vec<crate::demography::Vital>,
            territories: &'a Vec<crate::demography::Territory>,
            diaspora: &'a Vec<crate::demography::migration::Diaspora>,
            homes: &'a Vec<crate::demography::migration::Home>,
            waves: &'a Vec<crate::demography::migration::Wave>,
            waves_ended: &'a Vec<crate::demography::migration::WaveRecord>,
        }
        #[derive(Serialize)]
        struct WithBorders<'a, D: Serialize> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: OldEconomy<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: D,
            borders: &'a crate::borders::BordersState,
        }
        #[derive(Serialize)]
        struct NoBorders<'a, D: Serialize> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: OldEconomy<'a>,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: D,
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut data = crate::data::GameData::load(&root).unwrap();
        data.enable_forces();
        // This build, with the 1925 ledger and forces.
        let with_ledger = data.new_world();
        assert!(!with_ledger.econ.ledger.is_empty());
        assert!(!with_ledger.war.is_empty());
        let now = decode_world(&encode_snapshot(&with_ledger).unwrap()).unwrap();
        assert_eq!(hash_of(&now), hash_of(&with_ledger));
        // The same layouts behind the header with their version number go through the chain
        // of migrations instead of the guesses.
        let headed = |v: u32, bytes: &[u8]| {
            let mut b = crate::save_format::WORLD_MAGIC.to_vec();
            b.extend(v.to_le_bytes());
            // No catalogue keys.
            b.extend(0u32.to_le_bytes());
            b.extend_from_slice(bytes);
            decode_world(&b).unwrap_or_else(|e| panic!("version {v}: {e}"))
        };
        let now_headed = crate::save_format::encode_world(&with_ledger, None).unwrap();
        assert_eq!(
            hash_of(&decode_world(&now_headed).unwrap()),
            hash_of(&with_ledger)
        );
        assert_eq!(
            hash_of(&headed(15, &encode_snapshot(&with_ledger).unwrap())),
            hash_of(&with_ledger)
        );
        // Version 11, schooling by cohort, before the works paid from abroad and the reinvested
        // earnings.
        let before_works = encode_snapshot(&BeforeWorksAbroad {
            scenario: &with_ledger.scenario,
            player: &with_ledger.player,
            countries: &with_ledger.countries,
            census: &with_ledger.census,
            econ: &with_ledger.econ,
            research: &with_ledger.research,
            politics: &with_ledger.politics,
            demography: &with_ledger.demography,
            borders: &with_ledger.borders,
            foreign: foreign_then(&with_ledger),
            influence: &with_ledger.influence,
            treaties: &with_ledger.treaties,
            war: &with_ledger.war,
            education: &with_ledger.education,
        })
        .unwrap();
        assert_eq!(headed(11, &before_works), with_ledger);
        fn no_trade(w: &GameWorld) -> EconomyNoTrade<'_> {
            EconomyNoTrade {
                regions: &w.econ.regions,
                markets: &w.econ.markets,
                world_price: &w.econ.world_price,
                history: &w.econ.history,
                ledger: &w.econ.ledger,
            }
        }
        // Version 9, before trade by pair: the economy without its trade state.
        let before_trade = encode_snapshot(&BeforeTrade {
            scenario: &with_ledger.scenario,
            player: &with_ledger.player,
            countries: &with_ledger.countries,
            census: &with_ledger.census,
            econ: no_trade(&with_ledger),
            research: &with_ledger.research,
            politics: &with_ledger.politics,
            demography: &with_ledger.demography,
            borders: &with_ledger.borders,
            foreign: foreign_then(&with_ledger),
            influence: &with_ledger.influence,
            treaties: &with_ledger.treaties,
            war: &with_ledger.war,
        })
        .unwrap();
        assert_eq!(headed(9, &before_trade), with_ledger);
        // Version 8, before the forces: no forces then.
        // Version 14, before the budget's levers: this build's fields without `levers`.
        #[derive(Serialize)]
        struct BeforeLevers<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: &'a crate::econ::Economy,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
            foreign: &'a crate::foreign_stakes::ForeignState,
            influence: &'a crate::influence::InfluenceState,
            treaties: &'a crate::treaty::TreatyState,
            war: &'a crate::war::WarState,
            education: &'a crate::demography::education::EducationState,
        }
        let on_the_rule = data.new_world();
        assert!(on_the_rule.levers.is_empty());
        let before_levers = encode_snapshot(&BeforeLevers {
            scenario: &on_the_rule.scenario,
            player: &on_the_rule.player,
            countries: &on_the_rule.countries,
            census: &on_the_rule.census,
            econ: &on_the_rule.econ,
            research: &on_the_rule.research,
            politics: &on_the_rule.politics,
            demography: &on_the_rule.demography,
            borders: &on_the_rule.borders,
            foreign: &on_the_rule.foreign,
            influence: &on_the_rule.influence,
            treaties: &on_the_rule.treaties,
            war: &on_the_rule.war,
            education: &on_the_rule.education,
        })
        .unwrap();
        let loaded = headed(14, &before_levers);
        assert_eq!(hash_of(&loaded), hash_of(&on_the_rule));
        assert_eq!(loaded, on_the_rule, "version 14");
        // Version 10, before schooling by cohort: this build's fields without `education`.
        #[derive(Serialize)]
        struct BeforeEducation<'a> {
            scenario: &'a String,
            player: &'a Option<Id<Country>>,
            countries: &'a Table<Country, CountryState>,
            census: &'a Vec<CensusRecord>,
            econ: &'a crate::econ::Economy,
            research: &'a crate::research::ResearchState,
            politics: &'a crate::polity::PoliticsState,
            demography: &'a crate::demography::DemographyState,
            borders: &'a crate::borders::BordersState,
            foreign: ForeignWithoutWorks,
            influence: &'a crate::influence::InfluenceState,
            treaties: &'a crate::treaty::TreatyState,
            war: &'a crate::war::WarState,
        }
        let no_schooling = data.new_world();
        assert!(no_schooling.education.is_empty());
        let before_education = encode_snapshot(&BeforeEducation {
            scenario: &no_schooling.scenario,
            player: &no_schooling.player,
            countries: &no_schooling.countries,
            census: &no_schooling.census,
            econ: &no_schooling.econ,
            research: &no_schooling.research,
            politics: &no_schooling.politics,
            demography: &no_schooling.demography,
            borders: &no_schooling.borders,
            foreign: foreign_then(&no_schooling),
            influence: &no_schooling.influence,
            treaties: &no_schooling.treaties,
            war: &no_schooling.war,
        })
        .unwrap();
        let loaded = headed(10, &before_education);
        assert_eq!(hash_of(&loaded), hash_of(&no_schooling));
        assert_eq!(loaded, no_schooling, "version 10");
        let mut no_forces = data.new_world();
        no_forces.war = Default::default();
        assert!(!no_forces.treaties.is_empty());
        let before_forces = encode_snapshot(&BeforeForces {
            scenario: &no_forces.scenario,
            player: &no_forces.player,
            countries: &no_forces.countries,
            census: &no_forces.census,
            econ: no_trade(&no_forces),
            research: &no_forces.research,
            politics: &no_forces.politics,
            demography: &no_forces.demography,
            borders: &no_forces.borders,
            foreign: foreign_then(&no_forces),
            influence: &no_forces.influence,
            treaties: &no_forces.treaties,
        })
        .unwrap();
        assert!(decode_snapshot::<GameWorld>(&before_forces).is_err());
        assert_eq!(headed(8, &before_forces), no_forces);
        let loaded = decode_world(&before_forces).expect("before the forces");
        assert_eq!(hash_of(&loaded), hash_of(&no_forces));
        assert_eq!(loaded, no_forces);
        // Version 7, before the treaty state: no forces and no treaties then.
        let no_forces = {
            let mut w = no_forces;
            w.treaties = Default::default();
            w
        };
        let before_treaties = encode_snapshot(&BeforeTreaties {
            scenario: &no_forces.scenario,
            player: &no_forces.player,
            countries: &no_forces.countries,
            census: &no_forces.census,
            econ: no_trade(&no_forces),
            research: &no_forces.research,
            politics: &no_forces.politics,
            demography: &no_forces.demography,
            borders: &no_forces.borders,
            foreign: foreign_then(&no_forces),
            influence: &no_forces.influence,
        })
        .unwrap();
        let loaded = headed(7, &before_treaties);
        assert_eq!(hash_of(&loaded), hash_of(&no_forces));
        assert_eq!(loaded, no_forces);
        // Version 6, before the influence state: this build's fields without the last one,
        // behind the header and without it (a version 6 file may have been written either way).
        let before_influence = encode_snapshot(&BeforeInfluence {
            scenario: &no_forces.scenario,
            player: &no_forces.player,
            countries: &no_forces.countries,
            census: &no_forces.census,
            econ: no_trade(&no_forces),
            research: &no_forces.research,
            politics: &no_forces.politics,
            demography: &no_forces.demography,
            borders: &no_forces.borders,
            foreign: foreign_then(&no_forces),
        })
        .unwrap();
        assert_eq!(headed(6, &before_influence), no_forces);
        assert_eq!(decode_world(&before_influence).unwrap(), no_forces);
        // Older layouts: the world of then had no ledger, no treaty state and no forces.
        let mut world = data.new_world();
        world.econ.ledger = Default::default();
        world.treaties = Default::default();
        world.war = Default::default();
        let old_markets: Vec<_> = world
            .econ
            .markets
            .iter()
            .map(crate::econ::compat::MarketBeforeResearch::of)
            .collect();
        let econ = || OldEconomy {
            regions: &world.econ.regions,
            markets: &old_markets,
            world_price: &world.econ.world_price,
            history: &world.econ.history,
        };
        let check = |bytes: Vec<u8>, what: &str| {
            assert!(
                decode_snapshot::<GameWorld>(&bytes).is_err(),
                "{what}: the layout really differs"
            );
            let loaded = decode_world(&bytes).unwrap_or_else(|e| panic!("{what}: {e}"));
            assert_eq!(hash_of(&loaded), hash_of(&world), "{what}");
            loaded
        };
        // Before the foreign stakes (the main after research that costs).
        let before_foreign = encode_snapshot(&BeforeForeign {
            scenario: &world.scenario,
            player: &world.player,
            countries: &world.countries,
            census: &world.census,
            econ: no_trade(&world),
            research: &world.research,
            politics: &world.politics,
            demography: &world.demography,
            borders: &world.borders,
        })
        .unwrap();
        assert_eq!(headed(5, &before_foreign), world);
        assert_eq!(check(before_foreign, "before the foreign stakes"), world);
        // Before research that costs (the main after the ledger): markets with four public
        // works categories and no research priority.
        let before_research = encode_snapshot(&BeforeResearch {
            scenario: &world.scenario,
            player: &world.player,
            countries: &world.countries,
            census: &world.census,
            econ: EconomyWithLedger {
                regions: &world.econ.regions,
                markets: &old_markets,
                world_price: &world.econ.world_price,
                history: &world.econ.history,
                ledger: &world.econ.ledger,
            },
            research: &world.research,
            politics: &world.politics,
            demography: &world.demography,
            borders: &world.borders,
        })
        .unwrap();
        // Versions 3 and 4 (4 only added enum variants at the end) wrote this layout.
        assert_eq!(headed(3, &before_research), world);
        assert_eq!(headed(4, &before_research), world);
        assert_eq!(check(before_research, "before research that costs"), world);
        // Before the ledger (the main after the borders).
        let before_ledger = encode_snapshot(&WithBorders {
            scenario: &world.scenario,
            player: &world.player,
            countries: &world.countries,
            census: &world.census,
            econ: econ(),
            research: &world.research,
            politics: &world.politics,
            demography: &world.demography,
            borders: &world.borders,
        })
        .unwrap();
        assert_eq!(headed(2, &before_ledger), world);
        assert_eq!(check(before_ledger, "before the ledger"), world);
        // Before the border state too (the main after the communities).
        let before_borders = encode_snapshot(&NoBorders {
            scenario: &world.scenario,
            player: &world.player,
            countries: &world.countries,
            census: &world.census,
            econ: econ(),
            research: &world.research,
            politics: &world.politics,
            demography: &world.demography,
        })
        .unwrap();
        assert_eq!(headed(1, &before_borders), world);
        assert_eq!(check(before_borders, "before the borders"), world);
        // Before the communities: the demography without them, refilled on the next month.
        let d = &world.demography;
        let before_communities = encode_snapshot(&NoBorders {
            scenario: &world.scenario,
            player: &world.player,
            countries: &world.countries,
            census: &world.census,
            econ: econ(),
            research: &world.research,
            politics: &world.politics,
            demography: OldDemography {
                ages: &d.ages,
                schooling: &d.schooling,
                land_1925: &d.land_1925,
                vital: &d.vital,
                territories: &d.territories,
                diaspora: &d.diaspora,
                homes: &d.homes,
                waves: &d.waves,
                waves_ended: &d.waves_ended,
            },
        })
        .unwrap();
        for loaded in [
            decode_world(&before_communities).expect("before the communities"),
            headed(0, &before_communities),
        ] {
            assert!(loaded.demography.communities.is_empty());
            assert_eq!(loaded.econ, world.econ);
            assert_eq!(loaded.demography.ages, world.demography.ages);
        }
    }

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
            weights: [0, 9, 99, 1, 4],
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
            [0, 9, public::MAX_PRIORITY, 1, 4]
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
