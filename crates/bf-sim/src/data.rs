//! Reads the game data from disk: the economy catalogue and the 1925 scenario. Kept out of the
//! engine (which does no I/O); the client, the server and the benchmark all start here.

use std::path::Path;
use std::sync::Arc;

use crate::diplomacy::DiplomacyFile;
use crate::econ::{Catalogue, setup};
use crate::economy::{BuildingsFile, GoodsFile, NeedsFile, PurchasesFile};
use crate::game::GameWorld;
use crate::initial::{IndustryFile, PopulationFile};
use crate::politics::{
    InterestGroupsFile, LawsFile, LawsInForce, MovementsFile, Politics, ScenarioLawsFile,
};
use crate::research::{ResearchState, TechTree, TechnologiesFile};
use crate::resources::{DepositsFile, LandFile};
use crate::scenario::{CountriesFile, MapFile};

pub struct GameData {
    /// Rules a PR added switched off (`data/rules.ron`).
    pub switches: RuleSwitches,
    pub catalogue: Arc<Catalogue>,
    pub countries: CountriesFile,
    pub map: MapFile,
    /// Sea zones, straits and ports, when `assets/map/seas.ron` exists.
    pub seas: Option<crate::sea::SeaGraph>,
    /// Distances between countries for the freight of trade (`econ::trade::Geography`), built
    /// the first time trade by pair is switched on (`trade_geography`).
    trade_geography: std::sync::OnceLock<Arc<crate::econ::trade::Geography>>,
    pub deposits: Option<DepositsFile>,
    pub land: Option<LandFile>,
    pub population: Option<PopulationFile>,
    pub industry: Option<IndustryFile>,
    /// Organizations, treaties, monetary regimes and tariffs (phase 4; not simulated yet).
    pub diplomacy: Option<DiplomacyFile>,
    /// The technology tree, when `data/research/technologies.ron` exists.
    pub tree: Option<Arc<TechTree>>,
    /// Who lives where, when `data/scenario/1925/communities.ron` exists.
    pub communities: Option<crate::communities::Communities>,
    /// Schooling of 1925 by unit (`scenario/1925/education.ron`) and the adult literacy
    /// campaign (`economy/literacy_campaign.ron`), when they exist.
    pub education_1925: Option<crate::demography::education::EducationFile>,
    pub literacy_campaign: Option<crate::demography::education::CampaignRules>,
    /// What the simulation reads of them, built once.
    pub community_table: Option<Arc<crate::demography::communities::CommunityTable>>,
    /// Laws, interest groups and movements, when `data/politics/laws.ron` exists.
    pub politics: Option<Arc<Politics>>,
    /// Laws in force on the start date, when `data/scenario/1925/laws.ron` exists too.
    pub laws_1925: Option<LawsInForce>,
    /// Units, combat rules and the 1925 forces (phase 5; not simulated yet).
    pub war: Option<crate::war::War>,
    /// Countries that may be born after 1925, when `data/borders/candidates.ron` exists (border
    /// changes; not simulated yet).
    pub candidates: Option<crate::borders::CandidatesFile>,
    /// The clause catalogue of treaties, when `data/diplomacy/clauses.ron` exists (phase 4; not
    /// simulated yet).
    pub clauses: Option<crate::treaty::ClauseCatalogue>,
    /// Debts between governments on the start date, when `war_debts.ron` exists.
    pub war_debts: Option<crate::treaty::WarDebtsFile>,
    /// Rules of influence (`data/diplomacy/influence.ron`; phase 4, not simulated yet).
    pub influence: Option<crate::influence::InfluenceRules>,
    /// What the border operations read (map, communities, catalogue, the 1925 subjects).
    pub border_rules: Arc<crate::borders::BorderRules>,
    /// Who did not recognise whom on the start date, when `recognition.ron` exists.
    pub recognition: Option<crate::treaty::RecognitionFile>,
}

/// Rules a PR added switched off (`data/rules.ron`); a missing file or field is off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct RuleSwitches {
    /// Schooling by cohort and level (`demography::education`).
    pub schooling: bool,
}

impl RuleSwitches {
    /// Switches one rule on by name; `false` for an unknown name.
    pub fn switch_on(&mut self, name: &str) -> bool {
        match name {
            "schooling" => self.schooling = true,
            _ => return false,
        }
        true
    }
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn optional<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    if path.is_file() {
        read(path).map(Some)
    } else {
        Ok(None)
    }
}

impl GameData {
    /// `root` holds `assets/` and `data/`.
    pub fn load(root: &Path) -> Result<Self, String> {
        let data = root.join("data");
        let scenario = data.join("scenario/1925");
        let goods: GoodsFile = read(&data.join("economy/goods.ron"))?;
        let buildings: BuildingsFile = read(&data.join("economy/buildings.ron"))?;
        let needs: NeedsFile = read(&data.join("economy/needs.ron"))?;
        let purchases: PurchasesFile = read(&data.join("economy/purchases.ron"))?;
        let catalogue =
            Catalogue::build(&goods, &buildings, &needs, &purchases).map_err(|e| e.to_string())?;
        let tree = optional::<TechnologiesFile>(&data.join("research/technologies.ron"))?
            .map(|file| TechTree::build(&file, &catalogue).map(Arc::new))
            .transpose()
            .map_err(|e| format!("research/technologies.ron: {e}"))?;
        let map: MapFile = read(&root.join("assets/map/provinces.ron"))?;
        let seas = optional::<crate::sea::SeasFile>(&root.join("assets/map/seas.ron"))?
            .map(crate::sea::SeaGraph::new)
            .transpose()
            .map_err(|e| format!("assets/map/seas.ron: {e}"))?;
        let communities = crate::communities::load(&data, &map)?;
        let education_1925 = optional(&scenario.join("education.ron"))?;
        let literacy_campaign = optional(&data.join("economy/literacy_campaign.ron"))?;
        let countries: CountriesFile = read(&scenario.join("countries.ron"))?;
        let politics = if data.join("politics/laws.ron").is_file() {
            Some(
                Politics::build(
                    &read::<LawsFile>(&data.join("politics/laws.ron"))?,
                    &read::<InterestGroupsFile>(&data.join("politics/interest_groups.ron"))?,
                    &read::<MovementsFile>(&data.join("politics/movements.ron"))?,
                )
                .map_err(|e| format!("data/politics: {e}"))?,
            )
        } else {
            None
        };
        let laws_1925 = match (
            &politics,
            optional::<ScenarioLawsFile>(&scenario.join("laws.ron"))?,
        ) {
            (Some(p), Some(file)) => Some(
                p.resolve(&file, &countries.countries)
                    .map_err(|e| format!("scenario/1925/laws.ron: {e}"))?,
            ),
            _ => None,
        };
        let war = crate::war::load(&data, &catalogue, tree.as_deref(), &countries)?;
        let candidates: Option<crate::borders::CandidatesFile> =
            optional(&data.join("borders/candidates.ron"))?;
        let politics = politics.map(Arc::new);
        let clauses: Option<crate::treaty::ClauseCatalogue> =
            optional(&data.join("diplomacy/clauses.ron"))?;
        let mut border_rules = crate::borders::BorderRules::new(
            Arc::new(map.clone()),
            communities.clone().map(Arc::new),
            candidates
                .clone()
                .unwrap_or(crate::borders::CandidatesFile {
                    countries: Vec::new(),
                }),
            &countries,
            tree.clone(),
            politics.as_deref(),
            laplace_schelling::Date::new(1925, 1, 1),
        );
        border_rules.recognition = clauses.as_ref().map(|c| c.policy.recognition_default);
        let border_rules = Arc::new(border_rules);
        let diplomacy: Option<DiplomacyFile> = optional(&scenario.join("diplomacy.ron"))?;
        let mut data = Self {
            trade_geography: std::sync::OnceLock::new(),
            switches: optional(&data.join("rules.ron"))?.unwrap_or_default(),
            border_rules,
            war,
            tree,
            politics,
            laws_1925,
            catalogue: Arc::new(catalogue),
            countries,
            map,
            seas,
            community_table: communities
                .as_ref()
                .map(|c| Arc::new(crate::demography::communities::CommunityTable::new(c))),
            communities,
            education_1925,
            literacy_campaign,
            deposits: optional(&scenario.join("deposits.ron"))?,
            land: optional(&scenario.join("land.ron"))?,
            population: optional(&scenario.join("population.ron"))?,
            industry: optional(&scenario.join("industry.ron"))?,
            diplomacy,
            candidates,
            influence: optional(&data.join("diplomacy/influence.ron"))?,
            clauses,
            war_debts: optional(&scenario.join("war_debts.ron"))?,
            recognition: optional(&scenario.join("recognition.ron"))?,
        };
        // `BF_FORCES=1` switches the forces on over `peace.ron` (bench runs and screenshots;
        // tests call `enable_forces`).
        if std::env::var("BF_FORCES").is_ok_and(|v| v == "1") {
            data.enable_forces();
        }
        Ok(data)
    }

    /// Switches the forces in peace on (`soldiers: Forces`), whatever `peace.ron` says. They
    /// ship off until their own PR switches them on (`docs/war.pt-BR.md`, "Chave").
    pub fn enable_forces(&mut self) {
        if let Some(p) = self.war.as_mut().and_then(|w| w.peace.as_mut()) {
            p.soldiers = crate::war::SoldierRule::Forces;
        }
    }

    /// The distances of trade by pair, built on the first call: a game with trade by pair
    /// switched off never pays for them.
    pub fn trade_geography(&self) -> Arc<crate::econ::trade::Geography> {
        Arc::clone(self.trade_geography.get_or_init(|| {
            Arc::new(crate::econ::trade::Geography::build(
                &self.map,
                &self.countries.countries,
                self.seas.as_ref(),
                self.diplomacy.as_ref(),
                &self.catalogue.trade,
            ))
        }))
    }

    /// The rules the schedule runs with, every switch off.
    pub fn rules(&self) -> crate::game::Rules {
        self.rules_with(crate::game::Switches::default())
    }

    /// The rules the schedule runs with, with these switches.
    pub fn rules_with(&self, switches: crate::game::Switches) -> crate::game::Rules {
        crate::game::Rules {
            catalogue: Arc::clone(&self.catalogue),
            tree: self.tree.clone(),
            politics: self.politics.clone(),
            communities: self.community_table.clone(),
            borders: Some(Arc::clone(&self.border_rules)),
            influence: self.influence.clone().map(|rules| {
                Arc::new(crate::influence::InfluenceSetup {
                    rules,
                    table: self.community_table.clone(),
                    colonial: self
                        .politics
                        .as_ref()
                        .map(|p| p.groups.iter().map(|g| g.colonial).collect())
                        .unwrap_or_default(),
                })
            }),
            war: self.war_rules(),
            trade: switches.trade_by_pair.then(|| self.trade_geography()),
            switches,
            education: match (&self.education_1925, &self.literacy_campaign) {
                (Some(start), Some(campaign)) => {
                    Some(Arc::new(crate::demography::education::EducationRules {
                        start: start.clone(),
                        campaign: campaign.clone(),
                        enabled: self.switches.schooling,
                        kinds: [
                            self.catalogue.building("school"),
                            self.catalogue.building("secondary_school"),
                            self.catalogue.building("university"),
                        ],
                    }))
                }
                _ => None,
            },
        }
    }

    /// The unit catalogue and the rules in peace, when the forces are loaded (`peace.ron`
    /// with `soldiers: Forces` and the 1925 forces).
    fn war_rules(&self) -> Option<(Arc<crate::war::Military>, Arc<crate::war::PeaceRules>)> {
        let w = self.war.as_ref()?;
        let peace = w.peace.as_ref()?;
        (peace.soldiers == crate::war::SoldierRule::Forces && w.forces_1925.is_some())
            .then(|| (Arc::new(w.military.clone()), Arc::new(peace.clone())))
    }

    /// The world on the start date, economy included.
    pub fn new_world(&self) -> GameWorld {
        let mut world = GameWorld::from_scenario("1925", &self.countries.countries);
        world.econ = setup::build(
            &self.catalogue,
            &setup::Inputs {
                countries: &self.countries.countries,
                map: &self.map,
                deposits: self.deposits.as_ref(),
                land: self.land.as_ref(),
                population: self.population.as_ref(),
                industry: self.industry.as_ref(),
            },
        );
        world.demography = crate::demography::DemographyState {
            territories: crate::demography::territories(
                &world.econ,
                &self.map,
                self.communities.as_ref(),
            ),
            ..Default::default()
        };
        if let Some(c) = &self.communities {
            let start = crate::demography::migration::initial_diaspora(&world.econ, &self.map, c);
            world.demography.diaspora = start.diaspora;
            world.demography.homes = start.homes;
        }
        if let Some(tree) = &self.tree {
            world.research = ResearchState::initial(
                tree,
                self.countries.countries.iter().map(|c| c.tag.as_str()),
            );
        }
        if let (Some(p), Some(laws)) = (&self.politics, &self.laws_1925) {
            world.politics = crate::polity::PoliticsState::initial(p, laws);
        }
        // Debts between governments on the start date, when the switch of the clause catalogue
        // is on (owner's decision 6 of docs/treaties.pt-BR.md).
        if let (Some(cat), Some(debts)) = (&self.clauses, &self.war_debts)
            && !world.econ.markets.is_empty()
        {
            world.econ.ledger = crate::treaty::ledger_1925(debts, &cat.policy, &self.countries);
        }
        // Recognition withheld on the start date (owner's decision 4).
        if let (Some(cat), Some(rec)) = (&self.clauses, &self.recognition) {
            world.treaties.recognition = rec.start(&self.countries, &cat.policy);
        }
        if let (Some(w), Some(_)) = (&self.war, self.war_rules()) {
            let forces = w.forces_1925.as_ref().expect("checked by war_rules");
            world.war =
                crate::war::WarState::initial(&w.military, forces, &self.countries.countries);
            crate::war::station_soldiers(&w.military, &mut world);
        }
        world
    }
}
