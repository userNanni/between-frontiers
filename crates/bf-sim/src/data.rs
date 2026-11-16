//! Reads the game data from disk: the economy catalogue and the 1925 scenario. Kept out of the
//! engine (which does no I/O); the client, the server and the benchmark all start here.

use std::path::Path;
use std::sync::Arc;

use crate::econ::{Catalogue, setup};
use crate::economy::{BuildingsFile, GoodsFile, NeedsFile};
use crate::game::GameWorld;
use crate::initial::{IndustryFile, PopulationFile};
use crate::research::{ResearchState, TechTree, TechnologiesFile};
use crate::resources::{DepositsFile, LandFile};
use crate::scenario::{CountriesFile, MapFile};

pub struct GameData {
    pub catalogue: Arc<Catalogue>,
    pub countries: CountriesFile,
    pub map: MapFile,
    pub deposits: Option<DepositsFile>,
    pub land: Option<LandFile>,
    pub population: Option<PopulationFile>,
    pub industry: Option<IndustryFile>,
    /// The technology tree, when `data/research/technologies.ron` exists.
    pub tree: Option<Arc<TechTree>>,
    /// Who lives where, when `data/scenario/1925/communities.ron` exists. Not in the tick yet.
    pub communities: Option<crate::communities::Communities>,
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
        let catalogue = Catalogue::build(&goods, &buildings, &needs).map_err(|e| e.to_string())?;
        let tree = optional::<TechnologiesFile>(&data.join("research/technologies.ron"))?
            .map(|file| TechTree::build(&file, &catalogue).map(Arc::new))
            .transpose()
            .map_err(|e| format!("research/technologies.ron: {e}"))?;
        let map: MapFile = read(&root.join("assets/map/provinces.ron"))?;
        let communities = crate::communities::load(&data, &map)?;
        Ok(Self {
            tree,
            catalogue: Arc::new(catalogue),
            countries: read(&scenario.join("countries.ron"))?,
            map,
            communities,
            deposits: optional(&scenario.join("deposits.ron"))?,
            land: optional(&scenario.join("land.ron"))?,
            population: optional(&scenario.join("population.ron"))?,
            industry: optional(&scenario.join("industry.ron"))?,
        })
    }

    /// The rules the schedule runs with.
    pub fn rules(&self) -> crate::game::Rules {
        crate::game::Rules {
            catalogue: Arc::clone(&self.catalogue),
            tree: self.tree.clone(),
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
        if let Some(tree) = &self.tree {
            world.research = ResearchState::initial(
                tree,
                self.countries.countries.iter().map(|c| c.tag.as_str()),
            );
        }
        world
    }
}
