//! `--game`: the real campaign on the 1925 scenario, economy included. Checks determinism
//! across thread counts, a save and reload in the middle of the run, and the health of the
//! world by decade (population, GDP, needs, prices stuck at their limits).

use std::path::PathBuf;
use std::time::Instant;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::{Date, Fixed, StateHash};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Decade {
    pub year: i32,
    pub population: i64,
    /// GDP of the last full year at 1925 base prices.
    pub gdp: i64,
    pub gdp_per_capita: i64,
    pub basic_needs: f64,
    pub literacy: f64,
    pub urban: f64,
    pub plant_levels: i64,
    /// Market × good cells with trade whose price sits at the floor or the ceiling.
    pub prices_at_limit: usize,
    pub priced_cells: usize,
    pub ms: f64,
}

#[derive(Serialize)]
pub struct GameReport {
    pub years: u32,
    pub markets: usize,
    pub regions: usize,
    pub ticks: u64,
    pub threads: usize,
    pub total_ms: f64,
    pub slowest_tick_ms: f64,
    pub final_hash: StateHash,
    pub yearly_hashes: Vec<StateHash>,
    pub decades: Vec<Decade>,
    pub save_bytes: usize,
    pub save_load_mid_run_ok: bool,
    pub determinism_ok: Option<bool>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn new_session(data: &GameData, seed: u64, threads: usize) -> Session {
    let mut s = Session::new(
        data.new_world(),
        seed,
        Date::new(1925, 1, 1),
        threads,
        Some(data.catalogue.clone()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "BRA".into() });
    s
}

fn snapshot(data: &GameData, s: &Session, ms: f64) -> Decade {
    let w = s.world();
    let e = &w.econ;
    let population: i64 = e.regions.iter().map(|r| r.population()).sum();
    let gdp: i64 = e
        .markets
        .iter()
        .map(|m| m.gdp_last_year.to_int_round())
        .sum();
    let weighted = |f: &dyn Fn(&bf_sim::econ::Region) -> Fixed| -> f64 {
        e.regions
            .iter()
            .map(|r| f(r).to_f64_lossy() * r.population() as f64)
            .sum::<f64>()
            / population.max(1) as f64
    };
    let needs = weighted(&|r| {
        let n = r.population().max(1);
        r.pops
            .iter()
            .map(|p| (p.satisfaction[0] + p.satisfaction[1]) * p.size)
            .sum::<Fixed>()
            / (n * 2)
    });
    let cat = &data.catalogue;
    let (mut at_limit, mut cells) = (0, 0);
    for m in &e.markets {
        for (gi, good) in cat.goods.iter().enumerate() {
            if m.supply[gi] <= Fixed::ZERO && m.demand[gi] <= Fixed::ZERO {
                continue;
            }
            cells += 1;
            let r = m.price[gi] / good.base_price;
            if r >= Fixed::from_raw(3_900) || r <= Fixed::from_raw(260) {
                at_limit += 1;
            }
        }
    }
    Decade {
        year: s.clock().date.year,
        population,
        gdp,
        gdp_per_capita: gdp / population.max(1),
        basic_needs: needs,
        literacy: weighted(&|r| r.literacy),
        urban: weighted(&|r| r.urban_share()),
        plant_levels: e
            .regions
            .iter()
            .flat_map(|r| r.plants.iter())
            .map(|p| p.levels)
            .sum(),
        prices_at_limit: at_limit,
        priced_cells: cells,
        ms,
    }
}

struct Run {
    session: Session,
    yearly: Vec<StateHash>,
    decades: Vec<Decade>,
    total_ms: f64,
    slowest_ms: f64,
    ticks: u64,
}

fn run(data: &GameData, mut session: Session, until: Date, report: bool) -> Run {
    let (mut yearly, mut decades) = (Vec::new(), Vec::new());
    let (mut total, mut slowest, mut since) = (0.0f64, 0.0f64, 0.0f64);
    let mut ticks = 0;
    if report {
        decades.push(snapshot(data, &session, 0.0));
    }
    while session.clock().date < until {
        let t = Instant::now();
        session.sim.step();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        total += ms;
        since += ms;
        slowest = slowest.max(ms);
        ticks += 1;
        let d = session.clock().date;
        if d.month == 1 && d.day == 1 {
            yearly.push(session.state_hash());
            let every_year = std::env::var_os("BF_VERBOSE").is_some();
            if report && (d.year % 10 == 5 || every_year) {
                let snap = snapshot(data, &session, since);
                println!(
                    "{}: pop {:>5.0} M  GDP {:>7.0} bn  per head {:>6}  needs {:.2}  literacy {:.2}  urban {:.2}  levels {:>7}  prices at limit {}/{}  ({:.0} ms)",
                    snap.year,
                    snap.population as f64 / 1e6,
                    snap.gdp as f64 / 1e9,
                    snap.gdp_per_capita,
                    snap.basic_needs,
                    snap.literacy,
                    snap.urban,
                    snap.plant_levels,
                    snap.prices_at_limit,
                    snap.priced_cells,
                    since
                );
                decades.push(snap);
                since = 0.0;
            }
        }
    }
    Run {
        session,
        yearly,
        decades,
        total_ms: total,
        slowest_ms: slowest,
        ticks,
    }
}

pub fn main(years: u32, seed: u64, threads: usize, check: bool, out: &str) -> Result<(), String> {
    let data = GameData::load(&root())?;
    let end = Date::new(1925 + years as i32, 1, 1);
    let mid = Date::new(1925 + (years / 2) as i32, 1, 1);
    println!(
        "bf-bench --game: {} markets, {} regions, data: deposits {}, land {}, population {}, industry {}",
        data.countries.countries.len(),
        data.map.states.len(),
        data.deposits.is_some(),
        data.land.is_some(),
        data.population.is_some(),
        data.industry.is_some()
    );
    let main = run(&data, new_session(&data, seed, threads), end, true);

    let half = run(&data, new_session(&data, seed, threads), mid, false);
    let bytes = half
        .session
        .save(env!("CARGO_PKG_VERSION"), "bench")
        .map_err(|e| e.to_string())?;
    let reloaded =
        Session::load(&bytes, threads, Some(data.catalogue.clone())).map_err(|e| e.to_string())?;
    let resumed = run(&data, reloaded, end, false);
    let save_load_ok = resumed.session.state_hash() == main.session.state_hash();

    let determinism = if check {
        let other = if threads == 1 { 0 } else { 1 };
        let alt = run(&data, new_session(&data, seed, other), end, false);
        Some(alt.yearly == main.yearly)
    } else {
        None
    };
    let report = GameReport {
        years,
        markets: main.session.world().econ.markets.len(),
        regions: main.session.world().econ.regions.len(),
        ticks: main.ticks,
        threads,
        total_ms: main.total_ms,
        slowest_tick_ms: main.slowest_ms,
        final_hash: main.session.state_hash(),
        yearly_hashes: main.yearly.clone(),
        decades: main.decades.clone(),
        save_bytes: bytes.len(),
        save_load_mid_run_ok: save_load_ok,
        determinism_ok: determinism,
    };
    println!(
        "simulation {:.1} s, slowest tick {:.1} ms, save {:.1} MB at {}, reload → {}",
        report.total_ms / 1000.0,
        report.slowest_tick_ms,
        bytes.len() as f64 / 1e6,
        mid.year,
        if save_load_ok {
            "same final hash"
        } else {
            "DIVERGED"
        }
    );
    if let Some(ok) = determinism {
        println!(
            "determinism across thread counts: {}",
            if ok { "OK" } else { "FAILED" }
        );
    }
    println!("final hash {}", report.final_hash);
    std::fs::write(
        out,
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{out}: {e}"))?;

    let mut failures = Vec::new();
    if !save_load_ok {
        failures.push("save/load changed the outcome".to_owned());
    }
    if determinism == Some(false) {
        failures.push("hash differs across thread counts".to_owned());
    }
    if report
        .decades
        .last()
        .is_some_and(|last| last.population <= 0)
    {
        failures.push("the world is empty".to_owned());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}
