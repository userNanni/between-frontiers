//! Reads the game data from disk: the economy catalogue and the 1925 scenario. Kept out of the
//! engine (which does no I/O); the client, the server and the benchmark all start here.

use std::collections::BTreeMap;

use laplace_schelling::Fixed;
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
    /// The rule switches the game starts with (`data/rules.ron`, then `BF_RULES`).
    pub switches: crate::game::Switches,
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
    /// Independence by movement (`data/borders/independence.ron`).
    pub independence: Option<crate::borders::independence::IndependenceRules>,
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
        // The conduct of the war names interest groups and laws: checked here, at load.
        if let Some(c) = war.as_ref().and_then(|w| w.conduct.clone()) {
            crate::war::support::Conduct::resolve(c, politics.clone())?;
        }
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
        let data = Self {
            trade_geography: std::sync::OnceLock::new(),
            switches: optional::<crate::game::Switches>(&data.join("rules.ron"))?
                .unwrap_or_default()
                .with(&std::env::var("BF_RULES").unwrap_or_default())
                .map_err(|e| format!("BF_RULES: {e}"))?,
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
            independence: optional(&data.join("borders/independence.ron"))?,
            war_debts: optional(&scenario.join("war_debts.ron"))?,
            recognition: optional(&scenario.join("recognition.ron"))?,
        };
        Ok(data)
    }

    /// The terrain of each map state for the war on land: climate, mountain and marsh, forest
    /// and town (`war::state_terrain`).
    fn state_terrains(&self) -> BTreeMap<u16, crate::war::TerrainMix> {
        let (Some(w), Some(land)) = (&self.war, &self.land) else {
            return BTreeMap::new();
        };
        let rules = &w.military.rules;
        let ids = &self.border_rules.state_ids;
        let pop: BTreeMap<&str, (u64, f32)> = self
            .population
            .as_ref()
            .map(|p| {
                p.states
                    .iter()
                    .map(|s| (s.state.as_str(), (s.population, s.urban)))
                    .collect()
            })
            .unwrap_or_default();
        let area: BTreeMap<u16, f32> = self.map.states.iter().map(|s| (s.id, s.area_km2)).collect();
        land.states
            .iter()
            .filter_map(|s| {
                let id = *ids.get(&s.state)?;
                let a = area.get(&id).copied().unwrap_or(1.0).max(1.0);
                let urban = pop.get(s.state.as_str()).map_or(Fixed::ZERO, |(p, u)| {
                    crate::war::urban_share(rules, *p, *u, a)
                });
                let forest = (s.forest_km2 / a).clamp(0.0, 1.0);
                let terrain = w.terrain.as_ref().and_then(|t| t.state(id));
                Some((
                    id,
                    crate::war::state_terrain(rules, terrain, &s.climate, forest, urban),
                ))
            })
            .collect()
    }

    /// Big rivers along the borders between states, by (lower, higher) state id.
    fn river_borders(&self) -> BTreeMap<(u16, u16), Fixed> {
        let Some(t) = self.war.as_ref().and_then(|w| w.terrain.as_ref()) else {
            return BTreeMap::new();
        };
        let mut out = BTreeMap::new();
        for (a, near) in self.border_rules.neighbours.iter() {
            for b in near.iter().filter(|b| *b > a) {
                let r = t.river_share(*a, *b);
                if r.is_positive() {
                    out.insert((*a, *b), r);
                }
            }
        }
        out
    }

    /// The state of each country's capital on the start date: of its metropolitan states, the
    /// one whose centre lies nearest the capital (`capital_lon_lat`).
    fn capital_states(&self) -> BTreeMap<String, u16> {
        let meta = &self.map.meta;
        let miller = |lat: f64| {
            1.25 * (std::f64::consts::FRAC_PI_4 + 0.4 * lat.to_radians())
                .tan()
                .ln()
        };
        let (top, bottom) = (miller(meta.lat_top), miller(meta.lat_bottom));
        let lon_lat = |c: (f32, f32)| {
            let lon = f64::from(c.0) / f64::from(meta.width.max(1)) * 360.0 - 180.0;
            let y = top - f64::from(c.1) / f64::from(meta.height.max(1)) * (top - bottom);
            let lat = ((y / 1.25).exp().atan() - std::f64::consts::FRAC_PI_4) / 0.4;
            (lon, lat.to_degrees())
        };
        let great_circle = |a: (f64, f64), b: (f64, f64)| {
            let (la1, la2) = (a.1.to_radians(), b.1.to_radians());
            let dl = (b.0 - a.0).to_radians();
            let h = ((la2 - la1) / 2.0).sin().powi(2)
                + la1.cos() * la2.cos() * (dl / 2.0).sin().powi(2);
            2.0 * 6371.0 * h.sqrt().asin()
        };
        self.countries
            .countries
            .iter()
            .filter_map(|c| {
                self.map
                    .states
                    .iter()
                    .filter(|s| s.owner == c.tag && !s.colony)
                    .map(|s| (great_circle(lon_lat(s.center), c.capital_lon_lat), s.id))
                    .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                    .map(|(_, id)| (c.tag.clone(), id))
            })
            .collect()
    }

    /// States whose climate is mostly cold (continental, subarctic and polar): winter slows
    /// the front there from December to February.
    fn cold_states(&self) -> std::collections::BTreeSet<u16> {
        let Some(land) = &self.land else {
            return Default::default();
        };
        let ids = &self.border_rules.state_ids;
        land.states
            .iter()
            .filter(|s| {
                let cold: f32 = s
                    .climate
                    .iter()
                    .filter(|(k, _)| {
                        matches!(
                            k.as_str(),
                            "humid_continental" | "subarctic" | "tundra" | "ice_cap"
                        )
                    })
                    .map(|(_, v)| v)
                    .sum();
                cold >= 0.5
            })
            .filter_map(|s| ids.get(&s.state).copied())
            .collect()
    }

    /// Switches the forces in peace on (`Switches::forces`), whatever `data/rules.ron` says.
    pub fn enable_forces(&mut self) {
        self.switches.forces = true;
    }

    /// Switches crises and declaration on (`switches.crisis`), with the forces they need.
    pub fn enable_crisis(&mut self) {
        self.switches.forces = true;
        self.switches.crisis = true;
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

    /// Each country's gap between its 1925 tariff (`diplomacy.ron`, `tariffs`) and the tariff of
    /// its trade law of 1925, in market order (`econ::customs::Duties`); none without
    /// `trade.customs.rates_1925`.
    pub fn duties(&self) -> crate::econ::customs::Duties {
        use laplace_schelling::Fixed;
        let n = self.countries.countries.len();
        if !self.catalogue.trade.customs.rates_1925 {
            return crate::econ::customs::Duties {
                offset: vec![Fixed::ZERO; n],
            };
        }
        let rates: Vec<Option<Fixed>> = self
            .countries
            .countries
            .iter()
            .map(|c| {
                self.diplomacy
                    .as_ref()
                    .and_then(|d| d.tariff(&c.tag))
                    .map(|t| Fixed::from_f64_lossy(f64::from(t.rate)))
            })
            .collect();
        let tariff = crate::politics::effect_index("tariff");
        let law: Vec<Fixed> = (0..n)
            .map(|ci| match (&self.politics, &self.laws_1925, tariff) {
                (Some(p), Some(laws), Some(e)) => laws.laws.get(ci).map_or(Fixed::ZERO, |set| {
                    let v = p.effects(set, crate::politics::Scope::Metropole)[e];
                    Fixed::from_ratio(i64::from(v), 1000)
                }),
                _ => Fixed::ZERO,
            })
            .collect();
        crate::econ::customs::Duties::new(&rates, &law)
    }

    /// The rules the schedule runs with, with the switches of `data/rules.ron` (and `BF_RULES`).
    pub fn rules(&self) -> crate::game::Rules {
        self.rules_with(self.switches)
    }

    /// The rules the schedule runs with, with these switches.
    pub fn rules_with(&self, switches: crate::game::Switches) -> crate::game::Rules {
        let influence = self.influence.clone().map(|rules| {
            Arc::new(crate::influence::InfluenceSetup {
                rules: crate::influence::InfluenceRules {
                    live_ties: switches.influence_ties,
                    ..rules
                },
                table: self.community_table.clone(),
                colonial: self
                    .politics
                    .as_ref()
                    .map(|p| p.groups.iter().map(|g| g.colonial).collect())
                    .unwrap_or_default(),
                catalogue: Some(Arc::clone(&self.catalogue)),
                borders: Some(Arc::clone(&self.border_rules)),
            })
        });
        let independence = match (&self.independence, &self.politics) {
            (Some(rules), Some(politics)) => {
                Some(Arc::new(crate::borders::independence::IndependenceSetup {
                    rules: crate::borders::independence::IndependenceRules {
                        enabled: switches.independence,
                        ..rules.clone()
                    },
                    borders: Arc::clone(&self.border_rules),
                    politics: Arc::clone(politics),
                    cat: Arc::clone(&self.catalogue),
                    clauses: self.clauses.clone().map(Arc::new),
                    influence: influence.clone(),
                    trusteeship: politics.law("trusteeship").map(|l| l as u16),
                    services: (
                        self.catalogue.building("school"),
                        self.catalogue.building("hospital"),
                    ),
                }))
            }
            _ => None,
        };
        let war = self.war_rules(switches.forces);
        // Crises follow the forces' switch: built from the war rules already made.
        let crisis = self
            .war
            .as_ref()
            .and_then(|w| w.crisis.clone())
            .filter(|_| switches.crisis && war.is_some())
            .map(|rules| {
                Arc::new(crate::war::CrisisSetup {
                    rules,
                    clauses: self.clauses.clone(),
                    state_ids: self.border_rules.state_ids.clone(),
                })
            });
        let fight = war.as_ref().filter(|_| switches.war).map(|_| {
            Arc::new(crate::war::fight::FightSetup {
                neighbours: Arc::new(self.border_rules.neighbours.clone()),
                depth_km: self
                    .map
                    .states
                    .iter()
                    .map(|s| {
                        let side = f64::from(s.area_km2.max(1.0)).sqrt();
                        (s.id, laplace_schelling::Fixed::from_f64_lossy(side / 2.0))
                    })
                    .collect(),
                terrain: self.state_terrains(),
                river: self.river_borders(),
                cold: self.cold_states(),
                width_km: self
                    .war
                    .as_ref()
                    .and_then(|w| w.borders.as_ref())
                    .map(|b| {
                        b.borders
                            .iter()
                            .map(|x| ((x.a, x.b), Fixed::from_int(i64::from(x.km))))
                            .collect()
                    })
                    .unwrap_or_default(),
                capitals: self.capital_states(),
                conduct: self
                    .war
                    .as_ref()
                    .and_then(|w| w.conduct.clone())
                    .and_then(|c| {
                        crate::war::support::Conduct::resolve(c, self.politics.clone()).ok()
                    })
                    .map(Arc::new),
                politics: self.politics.clone(),
                vehicles: self
                    .war
                    .as_ref()
                    .and_then(|w| w.peace.as_ref()?.composition.as_ref())
                    .map(|r| crate::war::composition::vehicle_goods(r, &self.catalogue))
                    .unwrap_or_default(),
                doctrine_tech: self.war.as_ref().and_then(|w| {
                    self.tree
                        .as_ref()?
                        .tech(&w.military.rules.command.doctrine_tech)
                        .map(|t| t as u16)
                }),
            })
        });
        crate::game::Rules {
            catalogue: Arc::clone(&self.catalogue),
            tree: self.tree.clone(),
            politics: self.politics.clone(),
            communities: self.community_table.clone(),
            borders: Some(Arc::clone(&self.border_rules)),
            influence,
            war,
            crisis,
            fight,
            trade: switches.trade_by_pair.then(|| self.trade_geography()),
            duties: (switches.customs_duties && switches.trade_by_pair)
                .then(|| Arc::new(self.duties())),
            treaties: (switches.clause_tests || switches.reputation || switches.negotiation).then(
                || {
                    Arc::new(crate::treaty_system::TreatySetup {
                        decay: self
                            .clauses
                            .as_ref()
                            .map_or(laplace_schelling::Fixed::ZERO, |c| {
                                c.weights.reputation_decay
                            }),
                        government: self
                            .politics
                            .as_ref()
                            .and_then(|p| {
                                let g = p.group(crate::politics::GOVERNMENT_GROUP)?;
                                Some(p.groups[g].laws.clone())
                            })
                            .unwrap_or_default(),
                        clauses: self.clauses.clone().map(Arc::new),
                    })
                },
            ),
            switches,
            education: match (&self.education_1925, &self.literacy_campaign) {
                (Some(start), Some(campaign)) => {
                    Some(Arc::new(crate::demography::education::EducationRules {
                        start: start.clone(),
                        campaign: campaign.clone(),
                        enabled: switches.schooling,
                        kinds: [
                            self.catalogue.building("school"),
                            self.catalogue.building("secondary_school"),
                            self.catalogue.building("university"),
                        ],
                    }))
                }
                _ => None,
            },
            independence,
        }
    }

    /// The unit catalogue and the rules in peace, when the forces are on (the switch, or
    /// `peace.ron` with `soldiers: Forces`) and the 1925 forces are loaded.
    fn war_rules(
        &self,
        forces: bool,
    ) -> Option<(Arc<crate::war::Military>, Arc<crate::war::PeaceRules>)> {
        let w = self.war.as_ref()?;
        let peace = w.peace.as_ref()?;
        let on = forces || peace.soldiers == crate::war::SoldierRule::Forces;
        (on && w.forces_1925.is_some()).then(|| {
            let mut peace = peace.clone();
            peace.soldiers = crate::war::SoldierRule::Forces;
            (Arc::new(w.military.clone()), Arc::new(peace))
        })
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
        // The record of each line (`Switches::ledger_record`, off): kept by the ledger itself,
        // so that a line written in play records too.
        world.econ.ledger.record = self.switches.ledger_record;
        // The reputation of breaches is kept (`Switches::reputation`, off).
        world.treaties.reputation_on = self.switches.reputation;
        // The treaties of 1925 in force as agreements (`Switches::treaties_in_force`, shipped on).
        if self.switches.treaties_in_force
            && let (Some(d), Some(cat)) = (&self.diplomacy, &self.clauses)
        {
            world.treaties.seed_1925(d, cat);
        }
        // Recognition withheld on the start date (owner's decision 4).
        if let (Some(cat), Some(rec)) = (&self.clauses, &self.recognition) {
            world.treaties.recognition = rec.start(&self.countries, &cat.policy);
        }
        if let (Some(w), Some(_)) = (&self.war, self.war_rules(self.switches.forces)) {
            let forces = w.forces_1925.as_ref().expect("checked by war_rules");
            world.war =
                crate::war::WarState::initial(&w.military, forces, &self.countries.countries);
            crate::war::station_soldiers(&w.military, &mut world);
        }
        world
    }
}
