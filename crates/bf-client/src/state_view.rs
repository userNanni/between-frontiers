//! The figures of every state the simulation runs, for the state panel and the state lens: its
//! people, jobs, household income, schooling and towns, the buildings it has and the works
//! under way there. Read once a month, for every state at once (some hundreds, a few microseconds
//! each), so a click on the map needs no round trip to the simulation.

use bf_sim::econ::Catalogue;
use bf_sim::game::GameWorld;
use laplace_schelling::calendar::Date;

const PEASANT: usize = bf_sim::econ::prof::PEASANT;

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
    /// A state firm: its levels are the state's.
    pub state_firm: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StateFigures {
    /// The simulation runs this state (a region of a market).
    pub known: bool,
    /// Index of the market (country) it trades in, in the simulation's order.
    pub market: u16,
    pub colony: bool,
    pub population: i64,
    /// The labour force: each profession's working share (`econ::WORKER_SHARE`, 40 % of its
    /// people, dependents being the rest).
    pub workforce: i64,
    /// Of it, those at work in a plant or for the state (servants, soldiers, clergy).
    pub employed: i64,
    /// Peasants of the labour force with no job in a plant: they live off their own plot
    /// (`week::subsistence_food`).
    pub subsistence: i64,
    /// The rest of the labour force (every profession but peasants), and the jobless in it:
    /// the unemployment the simulation has.
    pub others: i64,
    pub jobless: i64,
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
    /// Share of the labour force at work in a plant or for the state.
    pub fn employed_share(&self) -> Option<f64> {
        (self.workforce > 0).then(|| self.employed as f64 / self.workforce as f64)
    }

    /// Jobless share of the labour force outside the peasantry.
    pub fn unemployment(&self) -> Option<f64> {
        (self.others > 0).then(|| self.jobless as f64 / self.others as f64)
    }

    pub fn income_per_head(&self) -> Option<f64> {
        (self.population > 0).then(|| self.income / self.population as f64)
    }

    pub fn levels(&self) -> i64 {
        self.plants.iter().map(|p| p.levels).sum()
    }

    /// Levels of state firms here.
    pub fn state_levels(&self) -> i64 {
        self.plants
            .iter()
            .filter(|p| p.state_firm)
            .map(|p| p.levels)
            .sum()
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
                    state_firm: b.state_firm,
                })
            })
            .collect();
        plants.sort_by(|a, b| b.levels.cmp(&a.levels).then(a.key.cmp(&b.key)));
        let mut works = [0usize; 2];
        for p in m.queue.iter().filter(|p| p.region as usize == ri) {
            works[usize::from(!p.public)] += 1;
        }
        let population = r.population();
        // Each profession's labour force and those of it at work; the state's servants,
        // soldiers and clergy work for it, not in a plant.
        let workforce: Vec<i64> = r
            .pops
            .iter()
            .map(|p| bf_sim::econ::workers(p.size))
            .collect();
        let at_work: Vec<i64> = r
            .pops
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if GOVERNMENT.contains(&i) {
                    workforce[i]
                } else {
                    p.employed.min(workforce[i])
                }
            })
            .collect();
        *slot = StateFigures {
            known: true,
            market: r.market,
            colony: r.colony,
            population,
            workforce: workforce.iter().sum(),
            employed: at_work.iter().sum(),
            subsistence: workforce[PEASANT] - at_work[PEASANT],
            others: workforce.iter().sum::<i64>() - workforce[PEASANT],
            jobless: (0..workforce.len())
                .filter(|&i| i != PEASANT)
                .map(|i| workforce[i] - at_work[i])
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
