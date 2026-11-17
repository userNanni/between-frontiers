//! Read-only view of the scenario for the client: countries, provinces and derived figures.
//!
//! Built from the scenario files; during a campaign the simulation's figures are laid over the
//! scenario's (`World::live`), refreshed once a month.

use std::path::Path;

use bf_sim::indicators::{QualityOfLife, quality_of_life};
use bf_sim::scenario::{CountriesFile, CountryDef, MapFile, ProvinceDef, StateDef, Status};
use laplace_schelling::calendar::Date;

use crate::live::LiveFigures;

pub struct Province {
    pub def: ProvinceDef,
    /// Index into [`World::countries`].
    pub owner: Option<usize>,
    /// Index into [`World::states`]; `None` for maps written before states existed.
    pub state: Option<usize>,
}

/// A state: the unit of the simulation, made of provinces of one owner.
pub struct State {
    pub def: StateDef,
    /// Index into [`World::countries`].
    pub owner: Option<usize>,
}

pub struct Country {
    pub def: CountryDef,
    /// Metropole provinces.
    pub provinces: Vec<u16>,
    pub colonial_provinces: Vec<u16>,
    /// Metropole and colonial states (indices into [`World::states`]).
    pub states: Vec<usize>,
    pub colonial_states: Vec<usize>,
    /// Metropole area.
    pub area_km2: f64,
    pub colonial_area_km2: f64,
    pub overlord: Option<usize>,
    pub subjects: Vec<usize>,
    /// Province holding the capital, when the map has one under it.
    pub capital_province: Option<u16>,
}

impl Country {
    pub fn tag(&self) -> &str {
        &self.def.tag
    }

    /// Whether the country holds colonies, by the scenario or on the map.
    pub fn has_colonies(&self) -> bool {
        self.def.colonies.is_some() || !self.colonial_provinces.is_empty()
    }

    pub fn is_subject(&self) -> bool {
        matches!(self.def.status, Status::Subject { .. })
    }
}

/// Where a table of figures comes from. The two count money in different units, never mixed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    /// The 1925 scenario: GDP in 2011 international dollars (Maddison Project).
    Scenario,
    /// The simulation, read once a month: GDP in dollars at the simulation's base prices, of the
    /// year `gdp_year` (`None` while the first year runs on the 1925 calibration).
    Live { gdp_year: Option<i32> },
}

/// One country's figures, metropole and colonies apart.
#[derive(Clone, Debug, Default)]
pub struct Figures {
    pub population: f64,
    pub colonial_population: f64,
    /// In the unit of the table's [`Source`].
    pub gdp: f64,
    pub colonial_gdp: f64,
    /// Metropole, 0–1.
    pub literacy: Option<f64>,
    pub urbanization: Option<f64>,
    /// Years. Always the scenario's: the simulation does not model it yet.
    pub life_expectancy: Option<f64>,
    /// Present when literacy and life expectancy are known.
    pub quality: Option<QualityOfLife>,
    /// Metropole population.
    pub rank_population: usize,
    /// GDP with the colonies: weight in the world, not welfare.
    pub rank_gdp: usize,
    pub rank_quality: Option<usize>,
}

impl Figures {
    fn new(
        metropole: (f64, f64),
        colonies: (f64, f64),
        literacy: Option<f64>,
        urbanization: Option<f64>,
        life_expectancy: Option<f64>,
    ) -> Self {
        let mut f = Self {
            population: metropole.0,
            gdp: metropole.1,
            colonial_population: colonies.0,
            colonial_gdp: colonies.1,
            literacy,
            urbanization,
            life_expectancy,
            ..Self::default()
        };
        f.quality = match (life_expectancy, literacy) {
            (Some(life), Some(lit)) => Some(quality_of_life(f.gdp_per_capita(), life, lit)),
            _ => None,
        };
        f
    }

    /// Metropole.
    pub fn gdp_per_capita(&self) -> f64 {
        if self.population > 0.0 {
            self.gdp / self.population
        } else {
            0.0
        }
    }

    pub fn total_population(&self) -> f64 {
        self.population + self.colonial_population
    }

    pub fn total_gdp(&self) -> f64 {
        self.gdp + self.colonial_gdp
    }
}

/// Figures of every country (index as in [`World::countries`]) with the world totals.
#[derive(Clone, Debug)]
pub struct FigureTable {
    pub source: Source,
    pub countries: Vec<Figures>,
    pub world_population: f64,
    pub world_gdp: f64,
}

impl FigureTable {
    fn new(source: Source, mut countries: Vec<Figures>) -> Self {
        let n = countries.len();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| countries[b].population.total_cmp(&countries[a].population));
        for (rank, &i) in order.iter().enumerate() {
            countries[i].rank_population = rank + 1;
        }
        order.sort_by(|&a, &b| {
            countries[b]
                .total_gdp()
                .total_cmp(&countries[a].total_gdp())
        });
        for (rank, &i) in order.iter().enumerate() {
            countries[i].rank_gdp = rank + 1;
        }
        let q = |f: &Figures| f.quality.map(|q| q.index);
        let mut with_quality: Vec<usize> = (0..n).filter(|&i| q(&countries[i]).is_some()).collect();
        with_quality.sort_by(|&a, &b| {
            q(&countries[b])
                .unwrap_or(0.0)
                .total_cmp(&q(&countries[a]).unwrap_or(0.0))
        });
        for (rank, &i) in with_quality.iter().enumerate() {
            countries[i].rank_quality = Some(rank + 1);
        }
        Self {
            source,
            world_population: countries.iter().map(Figures::total_population).sum(),
            world_gdp: countries.iter().map(Figures::total_gdp).sum(),
            countries,
        }
    }

    /// The 1925 scenario.
    pub fn scenario(defs: &[CountryDef]) -> Self {
        let countries = defs
            .iter()
            .map(|def| {
                let (cp, cg) = def.colonies.as_ref().map_or((0.0, 0.0), |c| {
                    (
                        c.population as f64,
                        c.population as f64 * c.gdp_per_capita as f64,
                    )
                });
                let pop = def.population as f64;
                Figures::new(
                    (pop, pop * def.gdp_per_capita as f64),
                    (cp, cg),
                    def.literacy.map(f64::from),
                    def.urbanization.map(f64::from),
                    def.life_expectancy.map(f64::from),
                )
            })
            .collect();
        Self::new(Source::Scenario, countries)
    }

    /// The simulation's figures, matched to the countries by tag. Life expectancy stays the
    /// scenario's; a country the simulation does not know counts as empty.
    pub fn live(countries: &[Country], live: &LiveFigures) -> Self {
        let rows = countries
            .iter()
            .map(|c| {
                let life = c.def.life_expectancy.map(f64::from);
                match live.countries.iter().find(|l| l.tag == c.def.tag) {
                    Some(l) => Figures::new(
                        (l.metropole.population, l.metropole.gdp),
                        (l.colonies.population, l.colonies.gdp),
                        l.metropole.literacy,
                        l.metropole.urbanization,
                        life,
                    ),
                    None => Figures::new((0.0, 0.0), (0.0, 0.0), None, None, life),
                }
            })
            .collect();
        Self::new(
            Source::Live {
                gdp_year: live.gdp_year,
            },
            rows,
        )
    }
}

pub struct World {
    pub start: Date,
    /// Deposits per state (index into [`World::states`]); empty when the data is missing.
    pub deposits: Vec<Vec<bf_sim::resources::Deposit>>,
    /// Land per state; `None` when the data is missing.
    pub land: Vec<Option<bf_sim::resources::StateLand>>,
    /// Figures on the start date.
    pub scenario: FigureTable,
    /// Figures from the simulation during a campaign, refreshed once a month; `None` outside
    /// one, where the scenario's stand.
    pub live: Option<FigureTable>,
    pub countries: Vec<Country>,
    pub provinces: Vec<Province>,
    /// Sorted by id: `states[i].def.id == i + 1`.
    pub states: Vec<State>,
    pub map: MapFile,
    /// Live from the simulation: share of the demand for transport and power met in each state
    /// (index into [`World::states`]), `-1` where unknown. Empty outside a game.
    pub live_local: std::sync::Arc<Vec<[f32; 2]>>,
    /// Border changes of the simulation already applied to the owners (`apply_owners`).
    pub owners_seen: usize,
    /// Countries that may be born (`data/borders/candidates.ron`); empty when the file is missing.
    pub candidates: Vec<bf_sim::borders::Candidate>,
    /// Countries of the scenario; those after them were born in the game.
    pub scenario_countries: usize,
    /// Status of every country of the scenario, to undo a campaign's border changes.
    scenario_status: Vec<Status>,
    /// The clause catalogue of treaties (`data/diplomacy/clauses.ron`): what a subject yields.
    pub clauses: Option<bf_sim::treaty::ClauseCatalogue>,
}

/// A country born during the game: the catalogue's colour, capital and flag; a grey one with no
/// capital for a country history did not make.
fn new_country_def(tag: &str, candidates: &[bf_sim::borders::Candidate]) -> CountryDef {
    let c = candidates.iter().find(|c| c.tag == tag);
    CountryDef {
        tag: tag.to_owned(),
        color: c.map_or((150, 150, 140), |c| c.color),
        status: Status::Sovereign,
        government: c
            .and_then(|c| c.government.clone())
            .unwrap_or_else(|| "presidential_republic".into()),
        capital_lon_lat: c.map_or((0.0, 0.0), |c| c.capital_lon_lat),
        population: 0,
        gdp_per_capita: 0,
        source: String::new(),
        literacy: None,
        life_expectancy: None,
        urbanization: None,
        social_source: String::new(),
        colonies: None,
        flag: c.and_then(|c| c.flag.clone()).map(|f| format!("{f}.png")),
    }
}

#[derive(Debug)]
pub struct WorldError(pub String);

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn read_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, WorldError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| WorldError(format!("{}: {e}", path.display())))?;
    ron::from_str(&text).map_err(|e| WorldError(format!("{}: {e}", path.display())))
}

fn parse_date(s: &str) -> Option<Date> {
    let mut it = s.split('-');
    let y = it.next()?.parse().ok()?;
    let m = it.next()?.parse().ok()?;
    let d = it.next()?.parse().ok()?;
    Some(Date::new(y, m, d))
}

impl World {
    pub fn load(map_ron: &Path, countries_ron: &Path) -> Result<Self, WorldError> {
        let map: MapFile = read_ron(map_ron)?;
        let countries: CountriesFile = read_ron(countries_ron)?;
        Ok(Self::build(map, countries))
    }

    pub fn build(map: MapFile, file: CountriesFile) -> Self {
        let start = parse_date(&file.start_date).unwrap_or(Date::new(1925, 1, 1));
        let index_of = |tag: &str| file.countries.iter().position(|c| c.tag == tag);
        let mut provinces: Vec<Province> = map
            .provinces
            .iter()
            .map(|def| Province {
                owner: index_of(&def.owner),
                state: None,
                def: def.clone(),
            })
            .collect();
        provinces.sort_by_key(|p| p.def.id);
        let mut states: Vec<State> = map
            .states
            .iter()
            .map(|def| State {
                owner: index_of(&def.owner),
                def: def.clone(),
            })
            .collect();
        states.sort_by_key(|s| s.def.id);
        for p in &mut provinces {
            p.state = usize::from(p.def.state)
                .checked_sub(1)
                .filter(|&i| states.get(i).is_some_and(|s| s.def.id == p.def.state));
        }

        let countries: Vec<Country> = file
            .countries
            .iter()
            .map(|def| Country {
                def: def.clone(),
                provinces: Vec::new(),
                colonial_provinces: Vec::new(),
                states: Vec::new(),
                colonial_states: Vec::new(),
                area_km2: 0.0,
                colonial_area_km2: 0.0,
                overlord: None,
                subjects: Vec::new(),
                capital_province: None,
            })
            .collect();
        let scenario = FigureTable::scenario(&file.countries);
        let deposits = vec![Vec::new(); states.len()];
        let land = vec![None; states.len()];
        let mut world = Self {
            start,
            deposits,
            land,
            scenario,
            live: None,
            countries,
            provinces,
            states,
            map,
            live_local: std::sync::Arc::default(),
            owners_seen: 0,
            candidates: Vec::new(),
            scenario_countries: 0,
            scenario_status: Vec::new(),
            clauses: None,
        };
        world.scenario_countries = world.countries.len();
        world.scenario_status = world
            .countries
            .iter()
            .map(|c| c.def.status.clone())
            .collect();
        world.index_countries();
        world
    }

    /// Back to the scenario's owners: a campaign's border changes leave with it.
    pub fn reset_owners(&mut self) {
        if self.owners_seen == 0 && self.countries.len() == self.scenario_countries {
            return;
        }
        self.countries.truncate(self.scenario_countries);
        for (c, s) in self.countries.iter_mut().zip(&self.scenario_status) {
            c.def.status = s.clone();
        }
        let index_of = |tag: &str| self.countries.iter().position(|c| c.def.tag == tag);
        let originals: Vec<(Option<usize>, bool)> = self
            .states
            .iter()
            .map(|st| {
                self.map
                    .states
                    .iter()
                    .find(|d| d.id == st.def.id)
                    .map_or((None, st.def.colony), |d| (index_of(&d.owner), d.colony))
            })
            .collect();
        for (st, (owner, colony)) in self.states.iter_mut().zip(originals) {
            st.owner = owner;
            st.def.colony = colony;
        }
        let provinces: Vec<(Option<usize>, bool)> = self
            .provinces
            .iter()
            .map(|p| {
                self.map
                    .provinces
                    .iter()
                    .find(|d| d.id == p.def.id)
                    .map_or((p.owner, p.def.colony), |d| (index_of(&d.owner), d.colony))
            })
            .collect();
        for (p, (owner, colony)) in self.provinces.iter_mut().zip(provinces) {
            p.owner = owner;
            p.def.colony = colony;
        }
        for table in std::iter::once(&mut self.scenario).chain(self.live.as_mut()) {
            table.countries.truncate(self.scenario_countries);
        }
        self.owners_seen = 0;
        self.index_countries();
    }

    /// A country born in the game has no 1925 figures of its own: its life expectancy (the
    /// scenario's until health is simulated) is that of the 1925 owners of its states, metropole
    /// or colonies, weighted by the people of each state now.
    fn inherit_social(&mut self, view: &bf_sim::borders::op::OwnersView) {
        let first_new = self.scenario.countries.len().min(self.scenario_countries);
        let tag_ix = |tag: &str| {
            self.countries[..first_new]
                .iter()
                .position(|c| c.def.tag == tag)
        };
        let mut sums = vec![(0.0f64, 0.0f64); self.countries.len()];
        for (i, st) in self.map.states.iter().enumerate() {
            let Some(&(owner, _)) = view.owners.get(i) else {
                continue;
            };
            let owner = usize::from(owner);
            if owner < first_new || owner >= sums.len() {
                continue;
            }
            let Some(old) = tag_ix(&st.owner) else {
                continue;
            };
            let def = &self.countries[old].def;
            let life = if st.colony {
                def.colonies.as_ref().and_then(|c| c.life_expectancy)
            } else {
                def.life_expectancy
            };
            let people = view.people.get(i).copied().unwrap_or(0) as f64;
            if let Some(l) = life {
                sums[owner].0 += f64::from(l) * people;
                sums[owner].1 += people;
            }
        }
        for (c, (lp, p)) in self.countries.iter_mut().zip(sums).skip(first_new) {
            c.def.life_expectancy = (p > 0.0).then(|| (lp / p) as f32);
        }
    }

    /// Each country's provinces, states, areas, overlord and subjects, from the owners of the
    /// provinces and states.
    fn index_countries(&mut self) {
        let index_of =
            |countries: &[Country], tag: &str| countries.iter().position(|c| c.def.tag == tag);
        for c in &mut self.countries {
            c.provinces.clear();
            c.colonial_provinces.clear();
            c.states.clear();
            c.colonial_states.clear();
            c.area_km2 = 0.0;
            c.colonial_area_km2 = 0.0;
            c.overlord = None;
            c.subjects.clear();
        }
        let countries = &mut self.countries;
        for p in &self.provinces {
            if let Some(o) = p.owner {
                let c = &mut countries[o];
                if p.def.colony {
                    c.colonial_provinces.push(p.def.id);
                    c.colonial_area_km2 += p.def.area_km2 as f64;
                } else {
                    c.provinces.push(p.def.id);
                    c.area_km2 += p.def.area_km2 as f64;
                }
            }
        }
        for (i, st) in self.states.iter().enumerate() {
            if let Some(o) = st.owner {
                if st.def.colony {
                    countries[o].colonial_states.push(i);
                } else {
                    countries[o].states.push(i);
                }
            }
        }
        for i in 0..countries.len() {
            if let Status::Subject { overlord, .. } = &countries[i].def.status
                && let Some(o) = index_of(countries, overlord)
            {
                countries[i].overlord = Some(o);
                countries[o].subjects.push(i);
            }
        }
    }

    /// The owners the simulation reports after border changes: countries born since the start
    /// join the list (their definition from the catalogue of possible countries, or a plain one),
    /// and every state and province takes its owner and status. `false` when nothing changed.
    pub fn apply_owners(&mut self, view: &bf_sim::borders::op::OwnersView) -> bool {
        if view.changes == self.owners_seen {
            return false;
        }
        self.owners_seen = view.changes;
        for tag in view.tags.iter().skip(self.countries.len()) {
            self.countries.push(Country {
                def: new_country_def(tag, &self.candidates),
                provinces: Vec::new(),
                colonial_provinces: Vec::new(),
                states: Vec::new(),
                colonial_states: Vec::new(),
                area_km2: 0.0,
                colonial_area_km2: 0.0,
                overlord: None,
                subjects: Vec::new(),
                capital_province: None,
            });
        }
        // Every figure table keeps a row per country; the new ones count as empty until the next
        // monthly reading.
        let n = self.countries.len();
        for table in std::iter::once(&mut self.scenario).chain(self.live.as_mut()) {
            table.countries.resize(n, Figures::default());
        }
        for (st, &(owner, colony)) in self.states.iter_mut().zip(&view.owners) {
            st.owner = (owner != u16::MAX).then_some(usize::from(owner));
            st.def.colony = colony;
        }
        for p in &mut self.provinces {
            if let Some(s) = p.state.and_then(|s| self.states.get(s)) {
                p.owner = s.owner;
                p.def.colony = s.def.colony;
            }
        }
        self.inherit_social(view);
        for (c, overlord) in self.countries.iter_mut().zip(&view.overlords) {
            c.def.status = match overlord {
                Some(o) => Status::Subject {
                    overlord: view.tags[usize::from(*o)].clone(),
                    kind: bf_sim::scenario::SubjectKind::Protectorate,
                },
                None => Status::Sovereign,
            };
        }
        self.index_countries();
        true
    }

    /// The figures on screen: the simulation's in a campaign, the scenario's elsewhere.
    pub fn figures(&self) -> &FigureTable {
        self.live.as_ref().unwrap_or(&self.scenario)
    }

    /// Leaves the campaign's live figures behind (back to the menu or the country choice).
    pub fn clear_live(&mut self) {
        self.live = None;
        self.live_local = std::sync::Arc::default();
    }

    /// Tooltip text for a province: the state name (the province name on maps without states)
    /// and the owner.
    pub fn place_name(
        &self,
        i18n: &bf_i18n::I18n,
        province: u16,
    ) -> Option<(String, Option<usize>)> {
        let p = self.provinces.get(usize::from(province).checked_sub(1)?)?;
        let name = p.state.map_or_else(
            || p.def.name.clone(),
            |s| crate::text::state_name(i18n, &self.states[s].def),
        );
        Some((name, p.owner))
    }

    /// Finds the capital provinces from the capital coordinates, using the projection.
    pub fn locate_capitals(&mut self, ids: &[u16]) {
        let (w, h) = (self.map.meta.width, self.map.meta.height);
        for c in &mut self.countries {
            let (lon, lat) = c.def.capital_lon_lat;
            let Some((x, y)) = project(&self.map.meta, lon, lat) else {
                continue;
            };
            let (x, y) = ((x as u32).min(w - 1), (y as u32).min(h - 1));
            let id = ids[(y * w + x) as usize];
            if id != 0 {
                c.capital_province = Some(id);
            }
        }
    }
}

/// Longitude/latitude to texel, for the projections the map tool writes.
pub fn project(meta: &bf_sim::scenario::MapMeta, lon: f64, lat: f64) -> Option<(f32, f32)> {
    let y_of = |lat: f64| -> f64 {
        let phi = lat.to_radians();
        match meta.projection.as_str() {
            "miller" => 1.25 * (std::f64::consts::FRAC_PI_4 + 0.4 * phi).tan().ln(),
            "equirectangular" => phi,
            _ => phi,
        }
    };
    let (top, bottom) = (y_of(meta.lat_top), y_of(meta.lat_bottom));
    let x = (lon + 180.0) / 360.0 * meta.width as f64;
    let y = (top - y_of(lat)) / (top - bottom) * meta.height as f64;
    if !(0.0..meta.height as f64).contains(&y) {
        return None;
    }
    Some((x as f32, y as f32))
}

impl World {
    /// Attaches deposits and land read from the scenario, matched to states by key.
    pub fn attach_resources(
        &mut self,
        deposits: Option<bf_sim::resources::DepositsFile>,
        land: Option<bf_sim::resources::LandFile>,
    ) {
        let index: std::collections::HashMap<String, usize> = self
            .states
            .iter()
            .enumerate()
            .map(|(i, s)| (s.def.key.clone(), i))
            .collect();
        if let Some(file) = deposits {
            for sd in file.states {
                if let Some(&i) = index.get(&sd.state) {
                    self.deposits[i].extend(sd.deposits);
                }
            }
        }
        if let Some(file) = land {
            for sl in file.states {
                if let Some(&i) = index.get(&sl.state) {
                    self.land[i] = Some(sl);
                }
            }
        }
    }

    /// Whether any deposit or land data was loaded.
    pub fn has_resources(&self) -> bool {
        self.deposits.iter().any(|d| !d.is_empty()) || self.land.iter().any(Option::is_some)
    }
}
