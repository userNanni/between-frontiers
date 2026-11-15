//! 100-year headless benchmark.
//!
//! Simulates 1925-01-01 to 2025-01-01 one day per tick with a fixed seed, hashes the state
//! every tick (folded into a chain) and at the end of every year, measures the cost of each
//! system, of each decade and of the slowest tick, and writes a JSON report. With
//! `--check-determinism` it runs a second time on a different thread count and fails if any
//! hash differs. The world is a synthetic toy until the game's systems exist.

mod game;
mod world;

use std::process::ExitCode;
use std::time::{Duration, Instant};

use laplace_schelling::command::Command;
use laplace_schelling::hash::{HashState, StateHash, StateHasher, hash_of};
use laplace_schelling::schedule::{Cadence, TickCtx};
use laplace_schelling::{Date, Simulation};
use serde::Serialize;

use crate::world::World;

const USAGE: &str = "\
Usage: bf-bench [options]

  --years N             years to simulate from 1925-01-01 (default 100)
  --threads N           worker threads, 0 = one per core (default 0)
  --seed S              world seed (default 1925)
  --check-determinism   run again on another thread count; fail on any hash divergence
  --budget-ms MS        fail when the total simulation time exceeds MS
  --budget-tick-ms MS   fail when the slowest tick exceeds MS
  --out PATH            JSON report path (default bf-bench-report.json)
  --game                run the real campaign on the 1925 scenario instead of the toy world
";

struct Options {
    years: u32,
    threads: usize,
    seed: u64,
    check_determinism: bool,
    budget_ms: Option<f64>,
    budget_tick_ms: Option<f64>,
    out: String,
    game: bool,
}

fn parse_args() -> Result<Options, String> {
    let mut options = Options {
        years: 100,
        threads: 0,
        seed: 1925,
        check_determinism: false,
        budget_ms: None,
        budget_tick_ms: None,
        out: "bf-bench-report.json".to_owned(),
        game: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        let number_error = |name: &str| format!("{name} needs a number");
        match flag.as_str() {
            "--years" => {
                options.years = value("--years")?
                    .parse()
                    .map_err(|_| number_error("--years"))?
            }
            "--threads" => {
                options.threads = value("--threads")?
                    .parse()
                    .map_err(|_| number_error("--threads"))?
            }
            "--seed" => {
                options.seed = value("--seed")?
                    .parse()
                    .map_err(|_| number_error("--seed"))?
            }
            "--check-determinism" => options.check_determinism = true,
            "--game" => options.game = true,
            "--budget-ms" => {
                options.budget_ms = Some(
                    value("--budget-ms")?
                        .parse()
                        .map_err(|_| number_error("--budget-ms"))?,
                )
            }
            "--budget-tick-ms" => {
                options.budget_tick_ms = Some(
                    value("--budget-tick-ms")?
                        .parse()
                        .map_err(|_| number_error("--budget-tick-ms"))?,
                )
            }
            "--out" => options.out = value("--out")?,
            "-h" | "--help" => return Err(String::new()),
            other => return Err(format!("unknown option {other}")),
        }
    }
    if options.years == 0 {
        return Err("--years must be at least 1".into());
    }
    Ok(options)
}

/// The toy world takes no commands.
#[derive(Clone)]
enum NoCommand {}

impl Command<World> for NoCommand {
    fn apply(&self, _ctx: &TickCtx, _world: &mut World) {
        match *self {}
    }
}

#[derive(Serialize)]
struct SystemReport {
    name: &'static str,
    cadence: String,
    runs: u64,
    compute_ms: f64,
    apply_ms: f64,
    total_ms: f64,
    mean_us: f64,
    slowest_ms: f64,
}

#[derive(Serialize)]
struct YearReport {
    year: i32,
    hash: StateHash,
    population: i64,
}

#[derive(Serialize)]
struct DecadeReport {
    decade: i32,
    ticks: u64,
    total_ms: f64,
    slowest_tick_ms: f64,
}

#[derive(Serialize)]
struct SlowestTick {
    tick: u64,
    date: String,
    ms: f64,
}

#[derive(Serialize)]
struct RunReport {
    threads: usize,
    ticks: u64,
    /// Simulation time: commands plus systems, without hashing.
    total_ms: f64,
    /// Time spent hashing the state every tick.
    hash_ms: f64,
    ticks_per_second: f64,
    slowest_tick: SlowestTick,
    systems: Vec<SystemReport>,
    decades: Vec<DecadeReport>,
    years: Vec<YearReport>,
    final_hash: StateHash,
    /// Hash chain over every tick's state hash.
    tick_chain_hash: StateHash,
    final_population: i64,
    markets_at_price_limit: u32,
}

#[derive(Serialize)]
struct Determinism {
    checked: bool,
    ok: bool,
    compared_threads: Vec<usize>,
    /// Total simulation time of the comparison run.
    compared_total_ms: Option<f64>,
    first_divergent_year: Option<i32>,
}

#[derive(Serialize)]
struct Report {
    engine_version: &'static str,
    os: &'static str,
    arch: &'static str,
    seed: u64,
    start: String,
    end: String,
    years: u32,
    regions: u32,
    goods: usize,
    budget_ms: Option<f64>,
    budget_tick_ms: Option<f64>,
    within_budget: bool,
    run: RunReport,
    determinism: Determinism,
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn cadence_label(cadence: Cadence) -> String {
    match cadence {
        Cadence::Daily => "daily".into(),
        Cadence::Weekly(day) => format!("weekly ({day:?})"),
        Cadence::Monthly { day } => format!("monthly (day {day})"),
        Cadence::Yearly { month, day } => format!("yearly ({month:02}-{day:02})"),
    }
}

fn run(seed: u64, threads: usize, end: Date) -> RunReport {
    let world = World::generate(seed);
    let schedule = world::schedule(threads);
    let threads = schedule.threads();
    let mut sim: Simulation<World, NoCommand> =
        Simulation::new(world, seed, world::START, schedule);

    let mut total = Duration::ZERO;
    let mut hashing = Duration::ZERO;
    let mut slowest = (0, world::START, Duration::ZERO);
    let mut decades: Vec<DecadeReport> = Vec::new();
    let mut years = Vec::new();
    let mut chain = StateHash::default();

    while sim.clock().date < end {
        let clock = sim.clock();
        let started = Instant::now();
        sim.step();
        let tick = started.elapsed();
        total += tick;
        if tick > slowest.2 {
            slowest = (clock.tick, clock.date, tick);
        }
        let decade = (clock.date.year - world::START.year).div_euclid(10) * 10 + world::START.year;
        if decades.last().is_none_or(|d| d.decade != decade) {
            decades.push(DecadeReport {
                decade,
                ticks: 0,
                total_ms: 0.0,
                slowest_tick_ms: 0.0,
            });
        }
        let current = decades.last_mut().expect("pushed above");
        current.ticks += 1;
        current.total_ms += ms(tick);
        current.slowest_tick_ms = current.slowest_tick_ms.max(ms(tick));

        let hashed = Instant::now();
        let state = hash_of(sim.world());
        let mut link = StateHasher::new();
        chain.hash_state(&mut link);
        state.hash_state(&mut link);
        chain = link.finish();
        let next = sim.clock().date;
        if next.month == 1 && next.day == 1 {
            years.push(YearReport {
                year: clock.date.year,
                hash: state,
                population: sim.world().population_total(),
            });
        }
        hashing += hashed.elapsed();
    }

    let systems = sim
        .schedule()
        .stats()
        .into_iter()
        .map(|s| SystemReport {
            name: s.name,
            cadence: cadence_label(s.cadence),
            runs: s.runs,
            compute_ms: ms(s.compute),
            apply_ms: ms(s.apply),
            total_ms: ms(s.compute + s.apply),
            mean_us: if s.runs == 0 {
                0.0
            } else {
                (s.compute + s.apply).as_secs_f64() * 1e6 / s.runs as f64
            },
            slowest_ms: ms(s.slowest),
        })
        .collect();
    let ticks = sim.clock().tick;
    RunReport {
        threads,
        ticks,
        total_ms: ms(total),
        hash_ms: ms(hashing),
        ticks_per_second: ticks as f64 / total.as_secs_f64().max(1e-9),
        slowest_tick: SlowestTick {
            tick: slowest.0,
            date: slowest.1.to_string(),
            ms: ms(slowest.2),
        },
        systems,
        decades,
        years,
        final_hash: hash_of(sim.world()),
        tick_chain_hash: chain,
        final_population: sim.world().population_total(),
        markets_at_price_limit: sim.world().stats.markets_at_price_limit,
    }
}

fn print_summary(report: &Report) {
    let run = &report.run;
    println!(
        "bf-bench {} → {}: {} ticks, seed {}, {} threads, {} regions × {} goods",
        report.start, report.end, run.ticks, report.seed, run.threads, report.regions, report.goods
    );
    println!(
        "total {:.0} ms ({:.0} ticks/s), slowest tick {:.3} ms on {} (tick {}), hashing {:.0} ms",
        run.total_ms,
        run.ticks_per_second,
        run.slowest_tick.ms,
        run.slowest_tick.date,
        run.slowest_tick.tick,
        run.hash_ms
    );
    println!(
        "{:<12} {:<20} {:>6} {:>11} {:>9} {:>9} {:>11}",
        "system", "cadence", "runs", "compute ms", "apply ms", "mean µs", "slowest ms"
    );
    for s in &run.systems {
        println!(
            "{:<12} {:<20} {:>6} {:>11.1} {:>9.1} {:>9.1} {:>11.3}",
            s.name, s.cadence, s.runs, s.compute_ms, s.apply_ms, s.mean_us, s.slowest_ms
        );
    }
    println!(
        "world: population {:.2} billion, {} markets at a price limit",
        run.final_population as f64 / 1e9,
        run.markets_at_price_limit
    );
    println!("final hash {}", run.final_hash);
    println!("tick chain {}", run.tick_chain_hash);
    let d = &report.determinism;
    if d.checked {
        let threads: Vec<String> = d.compared_threads.iter().map(ToString::to_string).collect();
        if d.ok {
            println!(
                "determinism OK: {} threads give identical hashes (comparison run {:.0} ms)",
                threads.join(" vs "),
                d.compared_total_ms.unwrap_or_default()
            );
        } else {
            println!(
                "determinism FAILED: {} threads diverge (first divergent year: {:?})",
                threads.join(" vs "),
                d.first_divergent_year
            );
        }
    }
}

fn main() -> ExitCode {
    let options = match parse_args() {
        Ok(options) => options,
        Err(message) => {
            if !message.is_empty() {
                eprintln!("bf-bench: {message}\n");
            }
            eprint!("{USAGE}");
            return ExitCode::from(if message.is_empty() { 0 } else { 64 });
        }
    };
    if options.game {
        return match game::main(
            options.years,
            options.seed,
            options.threads,
            options.check_determinism,
            &options.out,
        ) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("bf-bench --game: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let end = Date::new(world::START.year + options.years as i32, 1, 1);
    let primary = run(options.seed, options.threads, end);

    let mut determinism = Determinism {
        checked: options.check_determinism,
        ok: true,
        compared_threads: vec![primary.threads],
        compared_total_ms: None,
        first_divergent_year: None,
    };
    if options.check_determinism {
        let all = std::thread::available_parallelism().map_or(2, |n| n.get());
        let other_threads = if primary.threads == 1 { all.max(2) } else { 1 };
        let other = run(options.seed, other_threads, end);
        determinism.compared_threads.push(other.threads);
        determinism.compared_total_ms = Some(other.total_ms);
        determinism.first_divergent_year = primary
            .years
            .iter()
            .zip(&other.years)
            .find(|(a, b)| a.hash != b.hash)
            .map(|(a, _)| a.year);
        determinism.ok = primary.final_hash == other.final_hash
            && primary.tick_chain_hash == other.tick_chain_hash
            && determinism.first_divergent_year.is_none();
    }

    let over_total = options.budget_ms.is_some_and(|b| primary.total_ms > b);
    let over_tick = options
        .budget_tick_ms
        .is_some_and(|b| primary.slowest_tick.ms > b);
    let report = Report {
        engine_version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        seed: options.seed,
        start: world::START.to_string(),
        end: end.to_string(),
        years: options.years,
        regions: world::REGIONS,
        goods: world::GOODS,
        budget_ms: options.budget_ms,
        budget_tick_ms: options.budget_tick_ms,
        within_budget: !over_total && !over_tick,
        run: primary,
        determinism,
    };
    print_summary(&report);
    let json = serde_json::to_string_pretty(&report).expect("report serializes");
    if let Err(error) = std::fs::write(&options.out, json + "\n") {
        eprintln!("bf-bench: cannot write {}: {error}", options.out);
        return ExitCode::FAILURE;
    }
    println!("report written to {}", options.out);

    if !report.determinism.ok {
        return ExitCode::from(2);
    }
    if over_total {
        eprintln!(
            "bf-bench: total {:.0} ms exceeds the budget of {:.0} ms",
            report.run.total_ms,
            options.budget_ms.unwrap_or_default()
        );
        return ExitCode::FAILURE;
    }
    if over_tick {
        eprintln!(
            "bf-bench: slowest tick {:.3} ms exceeds the budget of {:.3} ms",
            report.run.slowest_tick.ms,
            options.budget_tick_ms.unwrap_or_default()
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use laplace_schelling::Date;

    #[test]
    fn toy_world_is_identical_on_one_and_four_threads() {
        let end = Date::new(1925, 4, 1);
        let one = super::run(7, 1, end);
        let four = super::run(7, 4, end);
        assert_eq!(one.ticks, 90);
        assert_eq!(one.tick_chain_hash, four.tick_chain_hash);
        assert_eq!(one.final_hash, four.final_hash);
        assert_ne!(super::run(8, 4, end).final_hash, four.final_hash);
    }
}
