//! The figures of every state the simulation runs, for the state panel and the state lens: its
//! people, jobs, household income, schooling and towns, the buildings it has and the works
//! under way there. Read once a month, for every state at once (some hundreds, a few microseconds
//! each), so a click on the map needs no round trip to the simulation.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use laplace_schelling::calendar::Date;

/// Professions the state employs directly.
const GOVERNMENT: [usize; 3] = [
    bf_sim::econ::prof::CIVIL_SERVANT,
    bf_sim::econ::prof::SOLDIER,
    bf_sim::econ::prof::CLERGY,
];

/// One building kind in the state.
#[derive(Clone, Debug, PartialEq)]
pub struct StatePlant {
    /// Building key (`building-<key>`).
    pub key: String,
    pub levels: i64,
    pub workers: f64,
    /// A year's output at this month's prices.
    pub output: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StateFigures {
    /// The simulation runs this state (a region of a market).
    pub known: bool,
    /// Index of the market (country) it trades in, in the simulation's order.
    pub market: u16,
    pub colony: bool,
    pub population: i64,
    /// People of working age (`econ::workers`) and those with work: in a plant, or for the
    /// state (servants, soldiers, clergy). Peasants without land work on no plant and count as
    /// without work, as in the simulation's subsistence (`week::unemployed_people`).
    pub workforce: i64,
    pub employed: i64,
    /// Households' income over a year, after tax, at last week's pace.
    pub income: f64,
    pub literacy: f64,
    pub urban: f64,
    /// The plants' output over a year at this month's prices.
    pub output: f64,
    /// The country's GDP of last year shared by the states' household income, as the country
    /// figures split metropole and colonies (`live::read`); `None` before the first year ends.
    pub gdp: Option<f64>,
    /// Building kinds here, most levels first.
    pub plants: Vec<StatePlant>,
    /// Works under way here: public, private.
    pub works: [usize; 2],
}

impl StateFigures {
    pub fn unemployment(&self) -> Option<f64> {
        (self.workforce > 0).then(|| 1.0 - self.employed as f64 / self.workforce as f64)
    }

    pub fn income_per_head(&self) -> Option<f64> {
        (self.population > 0).then(|| self.income / self.population as f64)
    }

    pub fn levels(&self) -> i64 {
        self.plants.iter().map(|p| p.levels).sum()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatesView {
    pub month: (i32, u8),
    /// By map state (`World::states` index, the state id less one).
    pub states: Vec<StateFigures>,
}

pub fn read(world: &GameWorld, cat: &Catalogue, date: Date) -> Option<StatesView> {
    let econ = &world.econ;
    if econ.regions.is_empty() {
        return None;
    }
    let n = econ
        .regions
        .iter()
        .map(|r| usize::from(r.state))
        .max()
        .unwrap_or(0);
    let mut states = vec![StateFigures::default(); n];
    for (ri, r) in econ.regions.iter().enumerate() {
        let Some(slot) = usize::from(r.state)
            .checked_sub(1)
            .and_then(|i| states.get_mut(i))
        else {
            continue;
        };
        let Some(m) = econ.markets.get(usize::from(r.market)) else {
            continue;
        };
        let mut plants: Vec<StatePlant> = r
            .plants
            .iter()
            .filter(|p| p.levels > 0)
            .filter_map(|p| {
                let b = cat.buildings.get(usize::from(p.kind))?;
                let method = b.methods.get(usize::from(p.method))?;
                let (workers, output) = crate::buildings_view::plant_figures(b, method, r, m, p);
                Some(StatePlant {
                    key: b.key.clone(),
                    levels: p.levels,
                    workers,
                    output,
                })
            })
            .collect();
        plants.sort_by(|a, b| b.levels.cmp(&a.levels).then(a.key.cmp(&b.key)));
        let mut works = [0usize; 2];
        for p in m.queue.iter().filter(|p| p.region as usize == ri) {
            works[usize::from(!p.public)] += 1;
        }
        let population = r.population();
        *slot = StateFigures {
            known: true,
            market: r.market,
            colony: r.colony,
            population,
            workforce: r.pops.iter().map(|p| bf_sim::econ::workers(p.size)).sum(),
            // The state's servants, soldiers and clergy work for it, not in a plant.
            employed: r
                .pops
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if GOVERNMENT.contains(&i) {
                        bf_sim::econ::workers(p.size)
                    } else {
                        p.employed
                    }
                })
                .sum(),
            income: r
                .pops
                .iter()
                .map(|p| p.income.to_f64_lossy().max(0.0))
                .sum::<f64>()
                * 52.0,
            literacy: r.literacy.to_f64_lossy(),
            urban: r.urban_share().to_f64_lossy(),
            output: plants.iter().map(|p| p.output).sum(),
            gdp: None,
            plants,
            works,
        };
    }
    // Last year's GDP of each country, shared by household income.
    let mut income = vec![0.0f64; econ.markets.len()];
    for f in states.iter().filter(|f| f.known) {
        if let Some(x) = income.get_mut(usize::from(f.market)) {
            *x += f.income;
        }
    }
    let year_done = !econ.history.is_empty();
    for f in states.iter_mut().filter(|f| f.known) {
        let total = income.get(usize::from(f.market)).copied().unwrap_or(0.0);
        let gdp = econ
            .markets
            .get(usize::from(f.market))
            .map_or(0.0, |m| m.gdp_last_year.to_f64_lossy());
        f.gdp = (year_done && total > 0.0).then(|| gdp * f.income / total);
    }
    Some(StatesView {
        month: (date.year, date.month),
        states,
    })
}
