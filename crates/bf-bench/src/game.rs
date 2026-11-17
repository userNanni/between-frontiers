//! `--game`: the real campaign on the 1925 scenario, economy included. Checks determinism
//! across thread counts, a save and reload in the middle of the run, and the health of the
//! world by decade (population, GDP, needs, prices stuck at their limits).

use std::path::PathBuf;
use std::time::Instant;

use bf_sim::data::GameData;
use bf_sim::game::Session;
use bf_sim::research::Stage;
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
    /// Technologies made at scale somewhere.
    pub techs_industrial: usize,
    /// The states' budgets, as shares of world GDP: taxes and spending (payroll, goods, public
    /// services, public works) over the last week times 52, and the treasuries' balance.
    #[serde(default)]
    pub taxes: f64,
    #[serde(default)]
    pub spending: f64,
    #[serde(default)]
    pub treasury: f64,
    /// Countries whose treasury is below zero.
    #[serde(default)]
    pub in_deficit: usize,
    /// Works waiting in the queues, and the world price of construction against its base.
    #[serde(default)]
    pub works_queued: usize,
    #[serde(default)]
    pub construction_price: f64,
    /// Money (slice 3c): yearly inflation of the decade, GDP-weighted mean and median, in
    /// percent; countries above 25 % in the last year; public debt, the central banks' share
    /// of it, interest and new money lent to treasuries, as shares of world GDP; defaults so
    /// far; the GDP-weighted policy rate.
    #[serde(default)]
    pub inflation: f64,
    #[serde(default)]
    pub inflation_median: f64,
    #[serde(default)]
    pub high_inflation: usize,
    #[serde(default)]
    pub debt: f64,
    #[serde(default)]
    pub debt_bank: f64,
    #[serde(default)]
    pub interest: f64,
    #[serde(default)]
    pub monetized: f64,
    #[serde(default)]
    pub defaults: u32,
    #[serde(default)]
    pub defaults_microstates: u32,
    #[serde(default)]
    pub policy_rate: f64,
    /// The central banks' loans to banks, as a share of world GDP.
    #[serde(default)]
    pub credit: f64,
    #[serde(default)]
    pub countries: Vec<CountryMoney>,
    pub ms: f64,
}

/// Money and debt of one country at a decade mark.
#[derive(Serialize, Clone)]
pub struct CountryMoney {
    pub tag: String,
    /// Mean yearly inflation of the decade, percent.
    pub inflation: f64,
    pub policy_rate: f64,
    pub bond_rate: f64,
    /// Debt over GDP, and the shares held by the central bank and in foreign currency.
    pub debt: f64,
    pub bank_share: f64,
    pub fx_share: f64,
    /// Debt to other governments by treaty and claims on them, over GDP (`treaty::Ledger`).
    #[serde(default)]
    pub treaty_owed: f64,
    #[serde(default)]
    pub treaty_claims: f64,
    pub defaults: u16,
}

/// GDP below which a market counts as a microstate in the report (a billion a year).
const MICROSTATE_GDP: Fixed = Fixed::from_int(1_000_000_000);

/// Countries the report follows by name.
const WATCHED: [&str; 6] = ["USA", "GBR", "JAP", "BRA", "ARG", "GER"];

/// Yearly inflation by market, from the price levels a year apart, and its decade sums.
#[derive(Default)]
struct InflationLog {
    /// Price level and currency reforms of each market a year ago.
    last_levels: Vec<(Fixed, u16)>,
    sums: Vec<f64>,
    world_mean: f64,
    world_median: f64,
    years: u32,
    high: usize,
}

impl InflationLog {
    fn year(&mut self, w: &bf_sim::game::GameWorld) {
        let ms = &w.econ.markets;
        let levels: Vec<(Fixed, u16)> = ms
            .iter()
            .map(|m| (m.money.price_level, m.money.redenominations))
            .collect();
        if self.last_levels.len() == levels.len() {
            let mut rows: Vec<(f64, f64)> = ms
                .iter()
                .zip(&self.last_levels)
                .map(|(m, (last, reforms))| {
                    // A currency reform cuts three zeros: count them back in.
                    let cut = 1000f64.powi(i32::from(m.money.redenominations - reforms));
                    let now = m.money.price_level.to_f64_lossy() * cut;
                    let rate = (now / last.to_f64_lossy().max(1e-9) - 1.0) * 100.0;
                    (rate, m.gdp_last_year.to_f64_lossy())
                })
                .collect();
            if self.sums.len() != rows.len() {
                self.sums = vec![0.0; rows.len()];
            }
            for (s, (r, _)) in self.sums.iter_mut().zip(&rows) {
                *s += r;
            }
            let gdp: f64 = rows.iter().map(|r| r.1).sum::<f64>().max(1.0);
            self.world_mean += rows.iter().map(|(r, g)| r * g).sum::<f64>() / gdp;
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut acc = 0.0;
            let median = rows
                .iter()
                .find(|(_, g)| {
                    acc += g;
                    acc >= gdp / 2.0
                })
                .map_or(0.0, |r| r.0);
            self.world_median += median;
            self.high = rows.iter().filter(|r| r.0 > 25.0).count();
            self.years += 1;
        }
        self.last_levels = levels;
    }

    /// Decade averages, and a fresh start for the next decade.
    fn take(&mut self) -> (f64, f64, Vec<f64>) {
        let n = f64::from(self.years.max(1));
        let out = (
            self.world_mean / n,
            self.world_median / n,
            self.sums.iter().map(|s| s / n).collect(),
        );
        self.world_mean = 0.0;
        self.world_median = 0.0;
        self.years = 0;
        self.sums.iter_mut().for_each(|s| *s = 0.0);
        out
    }
}

/// When a technology first worked and was first made at scale, in history and in the run.
#[derive(Clone, Serialize)]
pub struct TechTiming {
    pub key: String,
    pub demonstrated: i32,
    pub industrialized: i32,
    pub prototype: Option<i32>,
    pub industrial: Option<i32>,
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
    /// Cost of each system in the main run.
    pub systems: Vec<crate::SystemReport>,
    pub technologies: Vec<TechTiming>,
    /// Law changes of the campaign against the dated reference changes of 1925–1994.
    #[serde(default)]
    pub politics: Option<PoliticsReport>,
    /// With `--borders-history`: the reference changes submitted, applied, refused, and any whose
    /// totals changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub borders: Option<crate::borders::Log>,
}

/// One dated law change of the reference as a closed question for the history scorecard
/// (`tools/history/`): did it happen, and when?
#[derive(Serialize, Clone)]
pub struct LawEvent {
    /// `law:<TAG>:<law>`.
    pub id: String,
    pub question: String,
    pub tag: String,
    pub law: String,
    /// Reference date, `YYYY-MM-DD`, and the window counted as on time (years, inclusive).
    pub reference: String,
    pub window: (i32, i32),
    pub happened: bool,
    /// `YYYY-MM-DD` of the first simulated enactment, if any.
    pub simulated: Option<String>,
    /// Simulated year minus reference year.
    pub delay: Option<i32>,
    /// `on_time`, `early`, `late` or `never`.
    pub verdict: &'static str,
}

#[derive(Serialize, Clone)]
pub struct PoliticsReport {
    pub reference: usize,
    /// Within [`WINDOW`] years of the reference, earlier by more, later by more, never.
    pub on_time: usize,
    pub early: usize,
    pub late: usize,
    pub never: usize,
    pub events: Vec<LawEvent>,
    /// (tag, law, date) of simulated changes the reference does not list.
    pub extra: Vec<(String, String, String)>,
    /// Every change: (date, tag, from, to, by).
    pub changes: Vec<(String, String, String, String, String)>,
}

/// Years either side of the reference date that count as on time.
const WINDOW: i32 = 5;

fn iso(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

fn politics_report(data: &GameData, s: &Session) -> Option<PoliticsReport> {
    let p = data.politics.as_ref()?;
    let laws = data.laws_1925.as_ref()?;
    let w = s.world();
    let tags: Vec<&str> = w.countries.iter().map(|(_, c)| c.tag.as_str()).collect();
    let board = bf_sim::polity::score_against_history(p, &laws.history, &tags, &w.politics);
    let events: Vec<LawEvent> = board
        .rows
        .iter()
        .map(|r| {
            let year: i32 = r.reference[..4].parse().unwrap_or(0);
            let delay = r.delay();
            LawEvent {
                id: format!("law:{}:{}", r.tag, r.law),
                question: format!("Did {} enact {} near {}?", r.tag, r.law, r.reference),
                tag: r.tag.clone(),
                law: r.law.clone(),
                reference: r.reference.clone(),
                window: (year - WINDOW, year + WINDOW),
                happened: r.simulated.is_some(),
                simulated: r.simulated.map(iso),
                delay,
                verdict: match delay {
                    None => "never",
                    Some(d) if d < -WINDOW => "early",
                    Some(d) if d > WINDOW => "late",
                    Some(_) => "on_time",
                },
            }
        })
        .collect();
    let count = |v: &str| events.iter().filter(|e| e.verdict == v).count();
    Some(PoliticsReport {
        reference: events.len(),
        on_time: count("on_time"),
        early: count("early"),
        late: count("late"),
        never: count("never"),
        extra: board
            .extra
            .iter()
            .map(|(t, l, d)| (t.clone(), l.clone(), iso(*d)))
            .collect(),
        changes: w
            .politics
            .changes
            .iter()
            .map(|r| {
                (
                    iso(r.date),
                    tags[usize::from(r.country)].to_owned(),
                    p.laws[usize::from(r.from)].key.clone(),
                    p.laws[usize::from(r.to)].key.clone(),
                    match r.by {
                        bf_sim::polity::Enactor::Command => "command".to_owned(),
                        bf_sim::polity::Enactor::Groups => "groups".to_owned(),
                        bf_sim::polity::Enactor::Movement(k) => {
                            p.movements[usize::from(k)].key.clone()
                        }
                    },
                )
            })
            .collect(),
        events,
    })
}

fn root() -> PathBuf {
    crate::data_root::root()
}

fn new_session(data: &GameData, seed: u64, threads: usize) -> Session {
    // No player: every country runs its own politics, so the law changes can be scored
    // against history everywhere.
    let mut world = data.new_world();
    // `BF_PERTURB=K`: K dollars more in every country's investment pool on 1 January 1925, an
    // amount with no economic weight. Runs that differ only by K measure how far the model's
    // own sensitivity moves a result (`docs/history.pt-BR.md`, "Faixa de ruído").
    if let Some(k) = std::env::var("BF_PERTURB")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
    {
        for m in &mut world.econ.markets {
            m.pool += laplace_schelling::Fixed::from_int(k);
        }
    }
    // `BF_PERTURB_PERMILLE=N`: every pool N per mille larger (or smaller), a nudge the size of
    // a real change of rule or data: the reshuffling band.
    if let Some(n) = std::env::var("BF_PERTURB_PERMILLE")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
    {
        for m in &mut world.econ.markets {
            m.pool = m.pool.mul_ratio(1000 + n, 1000);
        }
    }
    Session::new(
        world,
        seed,
        Date::new(1925, 1, 1),
        threads,
        Some(rules(data)),
    )
}

fn snapshot(data: &GameData, s: &Session, ms: f64, log: &mut InflationLog) -> Decade {
    let w = s.world();
    let techs_industrial = w
        .research
        .firsts
        .iter()
        .filter(|f| f[Stage::Industrial as usize] > 0)
        .count();
    let e = &w.econ;
    let population: i64 = e.regions.iter().map(|r| r.population()).sum();
    // Snapshots are taken on 1 January before that day's tick, so the year just closed is still
    // in `gdp_running` (`gdp_last_year` is the year before it, and before the first tick the
    // scenario's figure, which nothing measured).
    let gdp: i64 = e
        .markets
        .iter()
        .map(|m| {
            if s.clock().tick > 0 {
                m.gdp_running
            } else {
                m.gdp_last_year
            }
            .to_int_round()
        })
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
    let gdp_f = (gdp as f64).max(1.0);
    let sum = |f: &dyn Fn(&bf_sim::econ::Market) -> Fixed| -> f64 {
        e.markets.iter().map(|m| f(m).to_f64_lossy()).sum::<f64>()
    };
    let cg = cat.good("construction").map(usize::from);
    let (inflation, inflation_median, by_market) = log.take();
    let countries = WATCHED
        .iter()
        .filter_map(|tag| {
            let c = w.countries.iter().position(|(_, x)| x.tag == *tag)?;
            let m = e.markets.get(c)?;
            let money = &m.money;
            let g = m.gdp_last_year.to_f64_lossy().max(1.0);
            let debt = money.total_debt().to_f64_lossy();
            Some(CountryMoney {
                tag: (*tag).to_owned(),
                inflation: by_market.get(c).copied().unwrap_or(0.0),
                policy_rate: money.policy_rate.to_f64_lossy(),
                bond_rate: money.bond_rate.to_f64_lossy(),
                debt: debt / g,
                bank_share: money.debt_bank.to_f64_lossy() / debt.max(1.0),
                fx_share: money.debt_fx_local().to_f64_lossy() / debt.max(1.0),
                treaty_owed: e.ledger.owed_by(c as u16).to_f64_lossy() / g,
                treaty_claims: e.ledger.claims_of(c as u16).to_f64_lossy() / g,
                defaults: money.defaults,
            })
        })
        .collect();
    Decade {
        inflation,
        inflation_median,
        high_inflation: log.high,
        debt: sum(&|m| m.money.total_debt()) / gdp_f,
        debt_bank: sum(&|m| m.money.debt_bank) / gdp_f,
        interest: sum(&|m| m.budget.interest) * 52.0 / gdp_f,
        monetized: sum(&|m| m.budget.issued) * 52.0 / gdp_f,
        // Microstates (San Marino, Andorra, Liechtenstein, Monaco) restructure almost every
        // year: their economy is tiny and mostly idle. Counted apart.
        defaults: e
            .markets
            .iter()
            .filter(|m| m.gdp_last_year >= MICROSTATE_GDP)
            .map(|m| u32::from(m.money.defaults))
            .sum(),
        defaults_microstates: e
            .markets
            .iter()
            .filter(|m| m.gdp_last_year < MICROSTATE_GDP)
            .map(|m| u32::from(m.money.defaults))
            .sum(),
        credit: sum(&|m| m.money.credit) / gdp_f,
        policy_rate: sum(&|m| m.money.policy_rate.mul_div(m.gdp_last_year, Fixed::ONE)) / gdp_f,
        countries,
        taxes: sum(&|m| m.budget.revenue) * 52.0 / gdp_f,
        spending: sum(&|m| {
            m.budget.payroll
                + m.budget.goods
                + m.budget.services
                + m.budget.works
                + m.budget.interest
        }) * 52.0
            / gdp_f,
        treasury: sum(&|m| m.treasury) / gdp_f,
        in_deficit: e
            .markets
            .iter()
            .filter(|m| m.treasury < Fixed::ZERO)
            .count(),
        works_queued: e.markets.iter().map(|m| m.queue.len()).sum(),
        construction_price: cg.map_or(0.0, |g| {
            (e.world_price[g] / cat.goods[g].base_price).to_f64_lossy()
        }),
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
        techs_industrial,
        ms,
    }
}

struct Run {
    session: Session,
    borders: crate::borders::Log,
    yearly: Vec<StateHash>,
    decades: Vec<Decade>,
    total_ms: f64,
    slowest_ms: f64,
    ticks: u64,
}

/// Invariants checked at every decade of the main run, so that a broken rule stops the run in
/// its first years instead of after the century:
///
/// - the world keeps people and output;
/// - with `BF_BAND=BASE.json[:PCT]` (a `game.json` of the base, e.g. the stored baseline of
///   `tools/bench/baseline.sh`), world population and GDP stay within PCT % (default 50) of the
///   base in the same year. The band is wide on purpose: it catches a world gone wrong, not
///   the effect of a rule.
fn check_decade(snap: &Decade) -> Result<(), String> {
    if snap.population <= 0 || snap.gdp <= 0 {
        return Err(format!(
            "{}: the world is empty (population {}, GDP {})",
            snap.year, snap.population, snap.gdp
        ));
    }
    let Some((base, pct)) = band() else {
        return Ok(());
    };
    let Some(b) = base.iter().find(|b| b.0 == snap.year) else {
        return Ok(());
    };
    for (what, now, then) in [("population", snap.population, b.1), ("GDP", snap.gdp, b.2)] {
        let off = (now as f64 / then.max(1) as f64 - 1.0) * 100.0;
        if off.abs() > *pct {
            return Err(format!(
                "{}: world {what} {now} is {off:+.0} % from the base ({then}), beyond ±{pct:.0} % (BF_BAND)",
                snap.year
            ));
        }
    }
    Ok(())
}

/// `BF_BAND`: (year, population, GDP) of the base decades and the tolerance in percent.
type Band = (Vec<(i32, i64, i64)>, f64);

fn band() -> Option<&'static Band> {
    static BAND: std::sync::OnceLock<Option<Band>> = std::sync::OnceLock::new();
    BAND.get_or_init(|| {
        let spec = std::env::var("BF_BAND").ok()?;
        let (path, pct) = match spec.rsplit_once(':') {
            Some((p, n)) if n.parse::<f64>().is_ok() => (p.to_owned(), n.parse().unwrap()),
            _ => (spec.clone(), 50.0),
        };
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("BF_BAND: {path}: {e}"));
        let json: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("BF_BAND: {path}: {e}"));
        let decades = json["decades"]
            .as_array()
            .unwrap_or_else(|| panic!("BF_BAND: {path} has no decades"))
            .iter()
            .filter_map(|d| {
                Some((
                    d["year"].as_i64()? as i32,
                    d["population"].as_i64()?,
                    d["gdp"].as_i64()?,
                ))
            })
            .collect();
        Some((decades, pct))
    })
    .as_ref()
}

fn run(
    data: &GameData,
    mut session: Session,
    until: Date,
    report: bool,
    mut history: Option<&mut crate::history::History>,
    saves: &mut [(Date, Option<Vec<u8>>)],
) -> Run {
    let (mut yearly, mut decades) = (Vec::new(), Vec::new());
    let (mut total, mut slowest, mut since) = (0.0f64, 0.0f64, 0.0f64);
    let mut ticks = 0;
    let mut log = InflationLog::default();
    log.year(session.world());
    if report {
        decades.push(snapshot(data, &session, 0.0, &mut log));
    }
    let mut borders = crate::borders::Log::default();
    while session.clock().date < until {
        if crate::borders::on() {
            crate::borders::submit(data, &mut session, &mut borders);
        }
        let t = Instant::now();
        session.sim.step();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        total += ms;
        since += ms;
        slowest = slowest.max(ms);
        ticks += 1;
        if let Some(h) = history.as_deref_mut() {
            h.tick(data, &session);
        }
        let d = session.clock().date;
        // Saved on the way: the save/load check resumes from here instead of simulating the
        // first half again.
        for (at, bytes) in saves.iter_mut() {
            if bytes.is_none() && *at == d {
                *bytes = session.save(env!("CARGO_PKG_VERSION"), "bench").ok();
            }
        }
        if d.month == 1 && d.day == 1 {
            yearly.push(session.state_hash());
            if report {
                log.year(session.world());
            }
            let every_year = std::env::var_os("BF_VERBOSE").is_some();
            if report && (d.year % 10 == 5 || every_year) {
                let snap = snapshot(data, &session, since, &mut log);
                println!(
                    "{}: pop {:>5.0} M  GDP {:>7.0} bn  per head {:>6}  needs {:.2}  literacy {:.2}  urban {:.2}  levels {:>7}  prices at limit {}/{}  techs {}  ({:.0} ms)",
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
                    snap.techs_industrial,
                    since
                );
                println!(
                    "      state: taxes {:.1}% of GDP, spending {:.1}%, treasuries {:+.1}%, {} in deficit; works queued {}, construction price {:.2}",
                    snap.taxes * 100.0,
                    snap.spending * 100.0,
                    snap.treasury * 100.0,
                    snap.in_deficit,
                    snap.works_queued,
                    snap.construction_price
                );
                {
                    // Local goods: share of the states' demand met, weighted by demand, and
                    // how many states get less than 80 % of it.
                    let w = session.world();
                    let cat = &data.catalogue;
                    let mut parts = Vec::new();
                    for (li, &g) in cat.local.iter().enumerate() {
                        let name = &cat.goods[g as usize].key;
                        let world_s: f64 = w
                            .econ
                            .markets
                            .iter()
                            .map(|m| m.supply[g as usize].to_f64_lossy())
                            .sum();
                        let world_d: f64 = w
                            .econ
                            .markets
                            .iter()
                            .map(|m| m.demand[g as usize].to_f64_lossy())
                            .sum();
                        let (mut met, mut want, mut short) = (0.0f64, 0.0f64, 0usize);
                        for r in &w.econ.regions {
                            if let Some(l) = r.local.get(li) {
                                let d = l.demand.to_f64_lossy();
                                met += d * l.buy.to_f64_lossy();
                                want += d;
                                if d > 0.0 && l.buy.to_f64_lossy() < 0.8 {
                                    short += 1;
                                }
                            }
                        }
                        if want > 0.0 {
                            parts.push(format!(
                                "{name} {:.0}% met, {short} states under 80% (supply/demand {:.2})",
                                met / want * 100.0,
                                world_s / world_d.max(1.0)
                            ));
                        }
                    }
                    if !parts.is_empty() {
                        println!("      local: {}", parts.join("; "));
                    }
                    if std::env::var_os("BF_LOCAL").is_some() {
                        let mut worst: Vec<(f64, usize)> = w
                            .econ
                            .regions
                            .iter()
                            .enumerate()
                            .filter_map(|(ri, r)| {
                                let li = std::env::var("BF_LOCAL")
                                    .ok()?
                                    .parse::<usize>()
                                    .unwrap_or(0);
                                let l = r.local.get(li)?;
                                (l.demand.to_f64_lossy() > 0.0).then(|| (l.buy.to_f64_lossy(), ri))
                            })
                            .collect();
                        worst.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                        for (buy, ri) in worst.iter().take(8) {
                            let r = &w.econ.regions[*ri];
                            let li = std::env::var("BF_LOCAL")
                                .ok()
                                .and_then(|v| v.parse::<usize>().ok())
                                .unwrap_or(0);
                            let l = &r.local[li];
                            let tag = w
                                .countries
                                .iter()
                                .nth(r.market as usize)
                                .map_or("?".to_string(), |(_, c)| c.tag.clone());
                            println!(
                                "        {tag} region {ri}: met {:.0}%, demand {:.0}, price {:.0}, {} neighbours, pop {} k",
                                buy * 100.0,
                                l.demand.to_f64_lossy(),
                                l.price.to_f64_lossy(),
                                r.neighbours.len(),
                                r.population() / 1000
                            );
                        }
                    }
                }
                println!(
                    "      money: inflation {:.1}% (median {:.1}%), {} above 25%; debt {:.0}% of GDP ({:.0}% central banks), interest {:.1}%, lent to treasuries {:.1}%, central bank credit {:.0}%, policy rate {:.1}%, defaults {} (+{} by microstates)",
                    snap.inflation,
                    snap.inflation_median,
                    snap.high_inflation,
                    snap.debt * 100.0,
                    snap.debt_bank * 100.0,
                    snap.interest * 100.0,
                    snap.monetized * 100.0,
                    snap.credit * 100.0,
                    snap.policy_rate,
                    snap.defaults,
                    snap.defaults_microstates
                );
                for c in &snap.countries {
                    println!(
                        "        {}: inflation {:>6.1}%  rate {:>5.1}%  bonds {:>5.1}%  debt {:>4.0}% of GDP (bank {:>3.0}%, foreign currency {:>3.0}%)  defaults {}  treaty debt {:>3.0}%, claims {:>3.0}%",
                        c.tag,
                        c.inflation,
                        c.policy_rate,
                        c.bond_rate,
                        c.debt * 100.0,
                        c.bank_share * 100.0,
                        c.fx_share * 100.0,
                        c.defaults,
                        c.treaty_owed * 100.0,
                        c.treaty_claims * 100.0
                    );
                }
                foreign_report(session.world());
                if std::env::var_os("BF_TECH").is_some() {
                    let w = session.world();
                    let mut e = [0.0f64; 5];
                    let mut total = 0.0;
                    for (c, eff) in w.research.effects.iter().enumerate() {
                        let pop: i64 = w
                            .econ
                            .regions
                            .iter()
                            .filter(|r| r.market as usize == c)
                            .map(|r| r.population())
                            .sum();
                        total += pop as f64;
                        for k in 0..5 {
                            e[k] += eff[k].to_f64_lossy() * pop as f64;
                        }
                    }
                    let (mut modern, mut all) = (0i64, 0i64);
                    for r in &w.econ.regions {
                        for p in &r.plants {
                            all += p.levels;
                            if data.catalogue.buildings[p.kind as usize].methods[p.method as usize]
                                .available_from
                                > 1925
                                || data.tree.as_ref().is_some_and(|t| {
                                    t.method_tech[p.kind as usize][p.method as usize].is_some()
                                })
                            {
                                modern += p.levels;
                            }
                        }
                    }
                    println!(
                        "      levels on tech-gated methods: {:.2}",
                        modern as f64 / all.max(1) as f64
                    );
                    for tag in ["USA", "GBR", "BRA", "CHN", "RAJ", "SOV"] {
                        if let Some(c) = w.countries.iter().position(|(_, x)| x.tag == tag) {
                            let tfp: Vec<String> = w.econ.markets[c]
                                .tech_tfp
                                .iter()
                                .map(|v| format!("{:.2}", v.to_f64_lossy()))
                                .collect();
                            let ind = w.research.stage[c]
                                .iter()
                                .filter(|s| **s >= Stage::Industrial)
                                .count();
                            println!(
                                "      {tag}: tech tfp {} industrial {ind} points {:.0}",
                                tfp.join(" "),
                                w.research.points[c].to_f64_lossy()
                            );
                        }
                    }
                    println!(
                        "      effects (pop-weighted): mortality {:.2} fertility {:.2} literacy {:.2} research {:.2} trade {:.2}",
                        e[0] / total,
                        e[1] / total,
                        e[2] / total,
                        e[3] / total,
                        e[4] / total
                    );
                }
                if let Err(e) = check_decade(&snap) {
                    // Out early: the rest of the century would be spent on a broken world.
                    eprintln!("bf-bench --game: {e}");
                    std::process::exit(3);
                }
                decades.push(snap);
                since = 0.0;
            }
        }
    }
    crate::borders::close(session.world(), &mut borders);
    Run {
        session,
        borders,
        yearly,
        decades,
        total_ms: total,
        slowest_ms: slowest,
        ticks,
    }
}

/// Rules of `data/rules.ron` switched on by `--rule`.
pub static RULES_ON: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

pub fn main(
    years: u32,
    seed: u64,
    threads: usize,
    check: bool,
    out: &str,
    history_out: Option<&str>,
    save_at: &[Date],
) -> Result<(), String> {
    let mut data = GameData::load(&root())?;
    for name in RULES_ON.lock().map_err(|e| e.to_string())?.iter() {
        data.switches
            .switch_on(name)
            .map_err(|e| format!("--rule {name}: {e}"))?;
    }
    // `BF_INFLUENCE_WEIGHT=N`: the `proposal_weight` of the rules of influence (0 checks that a
    // world where nobody holds influence keeps the hash of a build without the system).
    if let (Some(rules), Some(w)) = (
        data.influence.as_mut(),
        std::env::var("BF_INFLUENCE_WEIGHT")
            .ok()
            .and_then(|v| v.parse::<u32>().ok()),
    ) {
        rules.proposal_weight = w;
    }
    let data = data;
    let end = Date::new(1925 + years as i32, 1, 1);
    let mid = Date::new(1925 + (years / 2) as i32, 1, 1);
    let save_points = if save_at.is_empty() {
        vec![mid]
    } else {
        save_at.to_vec()
    };
    println!(
        "bf-bench --game: {} markets, {} regions, data: deposits {}, land {}, population {}, industry {}",
        data.countries.countries.len(),
        data.map.states.len(),
        data.deposits.is_some(),
        data.land.is_some(),
        data.population.is_some(),
        data.industry.is_some()
    );
    let session = new_session(&data, seed, threads);
    let mut history = history_out.map(|_| crate::history::History::new(&data, &session));
    let mut saves: Vec<(Date, Option<Vec<u8>>)> =
        save_points.iter().map(|&at| (at, None)).collect();
    let main = run(&data, session, end, true, history.as_mut(), &mut saves);
    if let (Some(h), Some(path)) = (history, history_out) {
        crate::history::write(h, path)?;
    }

    // Save at each point, reload and finish: the end must not change.
    let mut reloads = Vec::new();
    for (at, saved) in saves {
        // A point the main run did not pass (before the start or after the end): simulate to it.
        let bytes = match saved {
            Some(bytes) => bytes,
            None => run(
                &data,
                new_session(&data, seed, threads),
                at,
                false,
                None,
                &mut [],
            )
            .session
            .save(env!("CARGO_PKG_VERSION"), "bench")
            .map_err(|e| e.to_string())?,
        };
        let reloaded =
            Session::load(&bytes, threads, Some(rules(&data))).map_err(|e| e.to_string())?;
        let resumed = run(&data, reloaded, end, false, None, &mut []);
        let ok = resumed.session.state_hash() == main.session.state_hash();
        reloads.push((at, bytes.len(), ok));
    }
    let save_load_ok = reloads.iter().all(|r| r.2);
    let bytes_len = reloads.first().map_or(0, |r| r.1);

    let determinism = if check {
        let other = if threads == 1 { 0 } else { 1 };
        let alt = run(
            &data,
            new_session(&data, seed, other),
            end,
            false,
            None,
            &mut [],
        );
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
        save_bytes: bytes_len,
        save_load_mid_run_ok: save_load_ok,
        determinism_ok: determinism,
        systems: crate::system_reports(main.session.sim.schedule().stats()),
        technologies: timings(&data, &main.session),
        politics: politics_report(&data, &main.session),
        borders: crate::borders::on().then(|| main.borders.clone()),
    };
    if let Some(b) = &report.borders {
        println!(
            "borders: {} reference changes submitted, {} applied, {} refused, {} not conserved",
            b.submitted,
            b.applied,
            b.refused.len(),
            b.not_conserved.len()
        );
        for r in b.refused.iter().take(20) {
            println!("  refused {r}");
        }
        for r in &b.not_conserved {
            println!("  NOT CONSERVED {r}");
        }
    }
    if let Some(pr) = &report.politics {
        println!(
            "politics: {} reference changes: {} within {WINDOW} years, {} early, {} late, {} never; {} changes not in the reference",
            pr.reference,
            pr.on_time,
            pr.early,
            pr.late,
            pr.never,
            pr.extra.len()
        );
    }
    print_independence(&main.session);
    print_timings(&report.technologies);
    crate::print_systems(&report.systems);
    println!(
        "simulation {:.1} s, slowest tick {:.1} ms",
        report.total_ms / 1000.0,
        report.slowest_tick_ms
    );
    for (at, len, ok) in &reloads {
        println!(
            "save {:.1} MB at {at}, reload → {}",
            *len as f64 / 1e6,
            if *ok { "same final hash" } else { "DIVERGED" }
        );
    }
    if let Some(ok) = determinism {
        println!(
            "determinism across thread counts: {}",
            if ok { "OK" } else { "FAILED" }
        );
    }
    {
        let h: Vec<f64> = bf_sim::econ::monthly::ABROAD_HELD
            .iter()
            .map(|a| a.load(std::sync::atomic::Ordering::Relaxed) as f64)
            .collect();
        let pct = |x: f64| 100.0 * x / h[1].max(1.0);
        println!(
            "abroad: offered {:.0} bn, placed {:.0} bn; placed at sites full by idle hands {:.1}%, by the host's room {:.1}%; left unbuilt by hands {:.1}%, by room {:.1}%",
            h[0] / 1e9,
            h[1] / 1e9,
            pct(h[2]),
            pct(h[3]),
            pct(h[4]),
            pct(h[5])
        );
    }
    println!("final hash {}", report.final_hash);
    std::fs::write(
        out,
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{out}: {e}"))?;

    let mut failures = Vec::new();
    if report
        .borders
        .as_ref()
        .is_some_and(|b| !b.not_conserved.is_empty())
    {
        failures.push("a border operation changed the world's totals".to_owned());
    }
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

fn timings(data: &GameData, s: &Session) -> Vec<TechTiming> {
    let Some(tree) = &data.tree else {
        return Vec::new();
    };
    let firsts = &s.world().research.firsts;
    tree.techs
        .iter()
        .enumerate()
        .map(|(t, tech)| {
            let at = |stage: Stage| firsts.get(t).map(|f| f[stage as usize]).filter(|y| *y > 0);
            TechTiming {
                key: tech.key.clone(),
                demonstrated: tech.demonstrated,
                industrialized: tech.industrialized,
                prototype: at(Stage::Prototype),
                industrial: at(Stage::Industrial),
            }
        })
        .collect()
}

/// Run against history: how many technologies arrived, and how early or late (era 0 left out).
fn print_timings(t: &[TechTiming]) {
    let later: Vec<&TechTiming> = t.iter().filter(|x| x.demonstrated > 1925).collect();
    if later.is_empty() {
        return;
    }
    let stat = |pick: &dyn Fn(&TechTiming) -> (Option<i32>, i32)| {
        let lags: Vec<i32> = later
            .iter()
            .filter_map(|x| {
                let (sim, hist) = pick(x);
                sim.map(|y| y - hist)
            })
            .collect();
        let n = lags.len().max(1) as f64;
        let mean = lags.iter().sum::<i32>() as f64 / n;
        let abs = lags.iter().map(|l| l.abs()).sum::<i32>() as f64 / n;
        (lags.len(), mean, abs)
    };
    let (np, mp, ap) = stat(&|x| (x.prototype, x.demonstrated));
    let (ni, mi, ai) = stat(&|x| (x.industrial, x.industrialized));
    let due_p = later.iter().filter(|x| x.demonstrated <= 2025).count();
    println!(
        "technologies after 1925: prototype {np}/{due_p} (lag {mp:+.1} y, |lag| {ap:.1}), industrial {ni}/{due_p} (lag {mi:+.1} y, |lag| {ai:.1})"
    );
    if std::env::var_os("BF_TECH").is_some() {
        for x in t {
            println!(
                "  {:<28} proto {:>4} sim {:>6}   industry {:>4} sim {:>6}",
                x.key,
                x.demonstrated,
                x.prototype.map_or("-".into(), |y| y.to_string()),
                x.industrialized,
                x.industrial.map_or("-".into(), |y| y.to_string())
            );
        }
    }
}

/// The game's rules, with the switches of `data/rules.ron`, `BF_RULES` and `--rule`.
pub fn rules(data: &GameData) -> bf_sim::game::Rules {
    data.rules()
}

/// Foreign stakes: levels owned abroad, the largest hosts (share of their private levels) and the
/// largest owners (`docs/foreign-capital.pt-BR.md`).
fn foreign_report(w: &bf_sim::game::GameWorld) {
    let f = &w.foreign;
    let nm = w.econ.markets.len();
    let tag = |mi: usize| {
        w.countries
            .iter()
            .nth(mi)
            .map_or_else(|| format!("#{mi}"), |c| c.1.tag.to_string())
    };
    let mut hosted = vec![0i64; nm];
    let mut owned = vec![0i64; nm];
    for s in f.stakes() {
        hosted[w.econ.regions[s.region as usize].market as usize] += s.levels;
        owned[s.owner as usize] += s.levels;
    }
    let mut levels = vec![0i64; nm];
    for r in &w.econ.regions {
        for p in &r.plants {
            levels[r.market as usize] += p.levels;
        }
    }
    let total: i64 = hosted.iter().sum();
    let all: i64 = levels.iter().sum();
    let top = |v: &[i64], share: bool| {
        let mut ix: Vec<usize> = (0..nm).filter(|&i| v[i] > 0).collect();
        ix.sort_by(|a, b| v[*b].cmp(&v[*a]).then(a.cmp(b)));
        ix.iter()
            .take(6)
            .map(|&i| {
                if share {
                    format!(
                        "{} {} ({:.1}%)",
                        tag(i),
                        v[i],
                        100.0 * v[i] as f64 / levels[i].max(1) as f64
                    )
                } else {
                    format!("{} {}", tag(i), v[i])
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!(
        "      foreign: {} levels ({:.2}% of all), {} building; hosts {}; owners {}",
        total,
        100.0 * total as f64 / all.max(1) as f64,
        f.building_count(),
        top(&hosted, true),
        top(&owned, false)
    );
}

/// Independence by movement: the units at the end, the independences by decade and how, and the
/// checks of the broken-rule signals (`docs/borders.pt-BR.md`, section 11).
fn print_independence(session: &bf_sim::game::Session) {
    use bf_sim::borders::independence::How;
    let w = session.world();
    let st = &w.independence;
    if st.is_empty() {
        return;
    }
    let tag = |c: u16| {
        w.countries
            .iter()
            .nth(usize::from(c))
            .map_or("?", |(_, x)| x.tag.as_str())
            .to_owned()
    };
    let mut by_decade: std::collections::BTreeMap<i32, [u32; 3]> = Default::default();
    for r in &st.history {
        let e = by_decade.entry(r.date.year / 10 * 10).or_default();
        e[match r.how {
            How::Agreement => 0,
            How::Dominion => 1,
            How::Revolt => 2,
        }] += 1;
    }
    println!(
        "independence: {} units at the end, {} independences",
        st.units.len(),
        st.history.len()
    );
    for (d, [a, dm, rv]) in &by_decade {
        println!("  {d}s: {a} by agreement, {dm} as dominion, {rv} by revolt");
    }
    for r in &st.history {
        println!(
            "  {} {} from {} ({:?})",
            r.date,
            r.unit,
            tag(r.owner),
            r.how
        );
    }
    let mut top: Vec<_> = st.units.iter().collect();
    top.sort_by(|a, b| b.mobilized.cmp(&a.mobilized).then(a.key.cmp(&b.key)));
    for u in top.iter().take(10) {
        println!(
            "  unit {} of {}: support {} mobilized {} pressure {} weakest {} answer {:?}",
            u.key,
            tag(u.owner),
            u.movement.support,
            u.mobilized,
            u.movement.pressure,
            u.movement.weakest,
            u.demand.as_ref().map(|d| d.answer)
        );
    }
}
