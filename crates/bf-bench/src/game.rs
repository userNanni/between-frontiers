//! `--game`: the real campaign, not the toy world. Loads the 1925 scenario, runs the game on
//! the engine for the requested years and checks what exists so far:
//!
//! - determinism across thread counts (yearly state hashes);
//! - save and load in the middle of the run: the reloaded campaign must end on the same hash
//!   as the one that never stopped;
//! - the yearly census and the player's command made it into the state and the history;
//! - cost per tick and size of a save.

use std::path::PathBuf;
use std::time::Instant;

use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::scenario::CountriesFile;
use laplace_schelling::{Date, StateHash};
use serde::Serialize;

#[derive(Serialize)]
pub struct GameReport {
    pub years: u32,
    pub countries: usize,
    pub ticks: u64,
    pub threads: usize,
    pub total_ms: f64,
    pub slowest_tick_ms: f64,
    pub ticks_per_second: f64,
    pub final_hash: StateHash,
    pub yearly_hashes: Vec<StateHash>,
    pub census_records: usize,
    pub world_population_1925: i64,
    pub world_population_end: i64,
    pub world_gdp_1925: i64,
    pub player: Option<String>,
    pub history_len: usize,
    pub save_bytes: usize,
    pub save_load_mid_run_ok: bool,
    pub determinism_ok: Option<bool>,
}

fn scenario_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/scenario/1925/countries.ron")
}

fn new_session(file: &CountriesFile, seed: u64, threads: usize) -> Session {
    let world = GameWorld::from_scenario("1925", &file.countries);
    let mut s = Session::new(world, seed, Date::new(1925, 1, 1), threads);
    s.submit(GameCommand::ChoosePlayer { tag: "BRA".into() });
    s
}

struct Run {
    session: Session,
    yearly: Vec<StateHash>,
    total_ms: f64,
    slowest_ms: f64,
    ticks: u64,
}

fn run(mut session: Session, until: Date) -> Run {
    let mut yearly = Vec::new();
    let (mut total, mut slowest) = (0.0f64, 0.0f64);
    let mut ticks = 0;
    while session.clock().date < until {
        let t = Instant::now();
        session.sim.step();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        total += ms;
        slowest = slowest.max(ms);
        ticks += 1;
        let d = session.clock().date;
        if d.month == 1 && d.day == 1 {
            yearly.push(session.state_hash());
        }
    }
    Run {
        session,
        yearly,
        total_ms: total,
        slowest_ms: slowest,
        ticks,
    }
}

pub fn main(years: u32, seed: u64, threads: usize, check: bool, out: &str) -> Result<(), String> {
    let path = scenario_path();
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: CountriesFile =
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let end = Date::new(1925 + years as i32, 1, 1);
    let mid = Date::new(1925 + (years / 2) as i32, 1, 1);

    let main = run(new_session(&file, seed, threads), end);

    // Save in the middle, reload, continue: the end must not change.
    let half = run(new_session(&file, seed, threads), mid);
    let bytes = half
        .session
        .save(env!("CARGO_PKG_VERSION"), "bench")
        .map_err(|e| e.to_string())?;
    let reloaded = Session::load(&bytes, threads).map_err(|e| e.to_string())?;
    let resumed = run(reloaded, end);
    let save_load_ok = resumed.session.state_hash() == main.session.state_hash()
        && resumed.session.history().len() == main.session.history().len();

    let determinism = if check {
        let other = if threads == 1 { 0 } else { 1 };
        let alt = run(new_session(&file, seed, other), end);
        Some(alt.yearly == main.yearly && alt.session.state_hash() == main.session.state_hash())
    } else {
        None
    };

    let world = main.session.world();
    let census = &world.census;
    let report = GameReport {
        years,
        countries: world.countries.len(),
        ticks: main.ticks,
        threads,
        total_ms: main.total_ms,
        slowest_tick_ms: main.slowest_ms,
        ticks_per_second: main.ticks as f64 / (main.total_ms / 1000.0).max(1e-9),
        final_hash: main.session.state_hash(),
        yearly_hashes: main.yearly.clone(),
        census_records: census.len(),
        world_population_1925: census.first().map_or(0, |c| c.population),
        world_population_end: census.last().map_or(0, |c| c.population),
        world_gdp_1925: census.first().map_or(0, |c| c.gdp),
        player: world.player_tag().map(str::to_owned),
        history_len: main.session.history().len(),
        save_bytes: bytes.len(),
        save_load_mid_run_ok: save_load_ok,
        determinism_ok: determinism,
    };

    println!(
        "bf-bench --game 1925 → {}: {} ticks, {} countries, seed {seed}",
        end.year, report.ticks, report.countries
    );
    println!(
        "simulation {:.0} ms ({:.0} ticks/s), slowest tick {:.3} ms",
        report.total_ms, report.ticks_per_second, report.slowest_tick_ms
    );
    println!(
        "census: {} records, world population {} → {}, world GDP 1925 ${}",
        report.census_records,
        report.world_population_1925,
        report.world_population_end,
        report.world_gdp_1925
    );
    println!(
        "player {:?}, {} command(s) in the history, save {} bytes",
        report.player, report.history_len, report.save_bytes
    );
    println!(
        "save at {} and reload: {}",
        mid.year,
        if save_load_ok {
            "same final hash"
        } else {
            "DIVERGED"
        }
    );
    match determinism {
        Some(true) => println!("determinism OK across thread counts"),
        Some(false) => println!("determinism FAILED across thread counts"),
        None => {}
    }
    println!("final hash {}", report.final_hash);

    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    std::fs::write(out, json).map_err(|e| format!("{out}: {e}"))?;

    let mut failures = Vec::new();
    if !save_load_ok {
        failures.push("save/load changed the outcome");
    }
    if determinism == Some(false) {
        failures.push("hash differs across thread counts");
    }
    if report.census_records != years as usize {
        failures.push("census did not run once a year");
    }
    if report.player.as_deref() != Some("BRA") {
        failures.push("the player's command was not applied");
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}
