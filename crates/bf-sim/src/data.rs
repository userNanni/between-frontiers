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
    pub catalogue: Arc<Catalogue>,
    pub countries: CountriesFile,
    pub map: MapFile,
    /// Sea zones, straits and ports, when `assets/map/seas.ron` exists (not simulated yet).
    pub seas: Option<crate::sea::SeaGraph>,
    pub deposits: Option<DepositsFile>,
    pub land: Option<LandFile>,
    pub population: Option<PopulationFile>,
    pub industry: Option<IndustryFile>,
    /// Organizations, treaties, monetary regimes and tariffs (phase 4; not simulated yet).
    pub diplomacy: Option<DiplomacyFile>,
    /// The technology tree, when `data/research/technologies.ron` exists.
    pub tree: Option<Arc<TechTree>>,
    /// Who lives where, when `data/scenario/1925/communities.ron` exists. Not in the tick yet.
    pub communities: Option<crate::communities::Communities>,
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
        Ok(Self {
            war,
            tree,
            politics: politics.map(Arc::new),
            laws_1925,
            catalogue: Arc::new(catalogue),
            countries,
            map,
            seas,
            communities,
            deposits: optional(&scenario.join("deposits.ron"))?,
            land: optional(&scenario.join("land.ron"))?,
            population: optional(&scenario.join("population.ron"))?,
            industry: optional(&scenario.join("industry.ron"))?,
            diplomacy: optional(&scenario.join("diplomacy.ron"))?,
            candidates: optional(&data.join("borders/candidates.ron"))?,
            clauses: optional(&data.join("diplomacy/clauses.ron"))?,
            war_debts: optional(&scenario.join("war_debts.ron"))?,
        })
    }

    /// The rules the schedule runs with.
    pub fn rules(&self) -> crate::game::Rules {
        crate::game::Rules {
            catalogue: Arc::clone(&self.catalogue),
            tree: self.tree.clone(),
            politics: self.politics.clone(),
        }
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
        world
    }
}
