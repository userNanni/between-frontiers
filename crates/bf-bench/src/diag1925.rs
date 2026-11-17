//! `--diag-1925`: the first year of the economy against the scenario, for calibration.
//!
//! Runs 1925 with the game's rules and prints, by country and for the world, the GDP the
//! scenario asks for, the GDP the market measured, what the plants could make at base prices,
//! the money that stayed in stocks (pool, treasury, savings), and the 1925 output of the goods
//! `industry.ron` gives against what the plants made.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::{Economy, plant_tfp};
use bf_sim::game::Session;
use laplace_schelling::Date;

const GOODS: &[(&str, &str, f64)] = &[
    // (catalogue key, industry.ron key, catalogue units per industry unit)
    ("steel", "steel", 1000.0),
    ("cement", "cement", 1000.0),
    ("electricity", "electricity", 1000.0),
    ("fuel", "fuel", 1000.0),
    ("paper", "paper", 1000.0),
    ("chemicals", "chemicals", 1000.0),
    ("fertilizer", "fertilizer", 1000.0),
    ("aluminium", "aluminium", 1000.0),
    ("copper", "copper", 1000.0),
    ("textiles", "textiles", 1000.0),
    // A vehicle of 1925 is 8.79 lots of $1,000 (docs/economy.pt-BR.md, "Produção de 1925").
    ("automobiles", "automobiles", 8787.0),
    ("coal", "", 0.0),
    ("crude_oil", "", 0.0),
    ("natural_gas", "", 0.0),
    ("iron_ore", "", 0.0),
    ("grain", "", 0.0),
];

fn stocks(e: &Economy) -> (f64, f64, f64) {
    let pool = e.markets.iter().map(|m| m.pool.to_f64_lossy()).sum();
    let treasury = e.markets.iter().map(|m| m.treasury.to_f64_lossy()).sum();
    let savings = e
        .regions
        .iter()
        .flat_map(|r| r.pops.iter())
        .map(|p| p.savings.to_f64_lossy())
        .sum();
    (pool, treasury, savings)
}

pub fn main() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let data = GameData::load(&root)?;
    let cat = data.catalogue.clone();
    let world = data.new_world();
    let nm = world.econ.markets.len();
    let target: Vec<f64> = world
        .econ
        .markets
        .iter()
        .map(|m| m.gdp_last_year.to_f64_lossy())
        .collect();
    let base = |g: usize| cat.goods[g].base_price.to_f64_lossy();
    // Value added at base prices with every level staffed and running.
    let potential = |e: &Economy, full: bool| -> Vec<f64> {
        let mut out = vec![0.0; nm];
        for r in &e.regions {
            let m = &e.markets[r.market as usize];
            for p in &r.plants {
                let b = &cat.buildings[p.kind as usize];
                let meth = &b.methods[p.method as usize];
                let va: f64 = meth
                    .outputs
                    .iter()
                    .map(|(g, q)| q.to_f64_lossy() * base(*g as usize))
                    .sum::<f64>()
                    - meth
                        .inputs
                        .iter()
                        .map(|(g, q)| q.to_f64_lossy() * base(*g as usize))
                        .sum::<f64>();
                let mut scale = p.levels as f64 * plant_tfp(r, m, b).to_f64_lossy();
                if !full {
                    scale *= p.staffing.to_f64_lossy() * p.utilization.to_f64_lossy();
                }
                out[r.market as usize] += va * scale;
            }
        }
        out
    };
    let pot0 = potential(&world.econ, true);
    let s0 = stocks(&world.econ);
    let idx: Vec<Option<usize>> = GOODS
        .iter()
        .map(|(k, _, _)| cat.good(k).map(usize::from))
        .collect();
    let mut s = Session::new(world, 1, Date::new(1925, 1, 1), 0, Some(data.rules()));
    // Counterfactual laws held all century: `BF_LAWS="CHN=planned_economy+closed_capital+autarky"`
    // (several entries separated by `;`); `CHN@1980=...` holds them from that year on, over the
    // earlier ones of the same group. Applied on the first day and every 1 January after, so
    // that no movement undoes them.
    let forced = forced_laws(&data)?;
    force(&mut s, &forced, 1925);
    // Energy in a later year: demand by source, prices and the power plants' methods.
    if let Some(year) = std::env::var("BF_DIAG_ENERGY")
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
    {
        s.sim.run_until(Date::new(year, 7, 2));
        let e = &s.world().econ;
        for key in [
            "coal",
            "crude_oil",
            "natural_gas",
            "fuel",
            "electricity",
            "steel",
        ] {
            let Some(g) = cat.good(key) else { continue };
            let g = g as usize;
            let (mut sup, mut dem, mut pv) = (0.0, 0.0, 0.0);
            for m in &e.markets {
                sup += m.supply[g].to_f64_lossy();
                dem += m.demand[g].to_f64_lossy();
                pv += (m.price[g] / cat.goods[g].base_price).to_f64_lossy()
                    * m.demand[g].to_f64_lossy();
            }
            let mut by = vec![0.0f64; cat.buildings.len()];
            for r in &e.regions {
                let m = &e.markets[r.market as usize];
                for pl in &r.plants {
                    let b = &cat.buildings[pl.kind as usize];
                    let meth = &b.methods[pl.method as usize];
                    let sc = pl.levels as f64
                        * pl.staffing.to_f64_lossy()
                        * plant_tfp(r, m, b).to_f64_lossy()
                        * pl.utilization.to_f64_lossy();
                    for (gi, q) in &meth.inputs {
                        if *gi as usize == g {
                            by[pl.kind as usize] += q.to_f64_lossy() * sc;
                        }
                    }
                }
            }
            let plants: f64 = by.iter().sum();
            let mut top: Vec<usize> = (0..by.len()).collect();
            top.sort_by(|a, b| by[*b].total_cmp(&by[*a]));
            let tops: Vec<String> = top
                .iter()
                .take(5)
                .filter(|k| by[**k] > 0.0)
                .map(|k| {
                    format!(
                        "{} {:.0}%",
                        cat.buildings[*k].key,
                        by[*k] / dem.max(1.0) * 100.0
                    )
                })
                .collect();
            println!(
                "{year} {key}: supply {:.3e}/yr demand {:.3e}/yr price/base {:.2}; plants {:.0}% [{}]",
                sup * 52.0,
                dem * 52.0,
                pv / dem.max(1.0),
                plants / dem.max(1.0) * 100.0,
                tops.join(", ")
            );
        }
        if let Some(k) = cat.building("power_plant") {
            let mut by = vec![0i64; cat.buildings[k].methods.len()];
            for r in &e.regions {
                for p in r.plants.iter().filter(|p| p.kind as usize == k) {
                    by[p.method as usize] += p.levels;
                }
            }
            let names: Vec<String> = cat.buildings[k]
                .methods
                .iter()
                .zip(&by)
                .map(|(m, n)| format!("{} {n}", m.key))
                .collect();
            println!("{year} power plants: {}", names.join(", "));
        }
        return Ok(());
    }
    let mut made = vec![vec![0.0f64; GOODS.len()]; nm];
    let mut run = vec![0.0f64; nm];
    let mut util_sum = 0.0;
    let mut lost = vec![0.0f64; cat.buildings.len()];
    let mut vab = vec![0.0f64; cat.buildings.len()];
    let mut parts = [0.0f64; 4]; // VA-weighted staffing, utilization, input ratio, weight
    let mut weeks = 0;
    let mut gdp_end = vec![0.0; nm];
    // income, taxes, payroll, state goods, public services, works, subsistence
    let mut flows = [0.0f64; 7];
    let mut unsold = vec![0.0f64; cat.goods.len()];
    let mut short = vec![0.0f64; cat.goods.len()];
    let mut traded = vec![0.0f64; cat.goods.len()];
    loop {
        s.sim.step();
        let date = s.clock().date;
        if date.year > 1925 {
            break;
        }
        if date == Date::new(1925, 12, 31) {
            gdp_end = s
                .world()
                .econ
                .markets
                .iter()
                .map(|m| m.gdp_running.to_f64_lossy())
                .collect();
        }
        // The week ran on Monday; Tuesday holds its outcome.
        if date.weekday() != laplace_schelling::Weekday::Tuesday {
            continue;
        }
        weeks += 1;
        let e = &s.world().econ;
        let p = potential(e, false);
        for (a, b) in run.iter_mut().zip(&p) {
            *a += b;
        }
        let full = potential(e, true);
        flows[0] += e
            .regions
            .iter()
            .flat_map(|r| r.pops.iter())
            .map(|p| p.income.to_f64_lossy())
            .sum::<f64>();
        for m in &e.markets {
            flows[1] += m.budget.revenue.to_f64_lossy();
            flows[2] += m.budget.payroll.to_f64_lossy();
            flows[3] += m.budget.goods.to_f64_lossy();
            flows[4] += m.budget.services.to_f64_lossy();
            flows[5] += m.budget.works.to_f64_lossy();
            for g in 0..cat.goods.len() {
                let (sp, d) = (m.supply[g].to_f64_lossy(), m.demand[g].to_f64_lossy());
                let t = sp.min(d);
                unsold[g] += (sp - t) * base(g);
                short[g] += (d - t) * base(g);
                traded[g] += t * base(g);
            }
        }
        if let Some(gr) = cat.good("grain") {
            for r in &e.regions {
                let m = &e.markets[r.market as usize];
                flows[6] += bf_sim::econ::week::subsistence_food(&cat, r, m).to_f64_lossy()
                    * base(gr as usize);
            }
        }
        util_sum += p.iter().sum::<f64>() / full.iter().sum::<f64>().max(1.0);
        for r in &e.regions {
            let m = &e.markets[r.market as usize];
            for pl in &r.plants {
                let bb = &cat.buildings[pl.kind as usize];
                let mm = &bb.methods[pl.method as usize];
                let va = mm
                    .value_added(|g| cat.goods[g].base_price)
                    .to_f64_lossy()
                    .max(0.0)
                    * pl.levels as f64;
                parts[0] += va * pl.staffing.to_f64_lossy();
                parts[1] += va * pl.utilization.to_f64_lossy();
                parts[2] += va * pl.input_ratio.to_f64_lossy();
                parts[3] += va;
                lost[pl.kind as usize] += va
                    * (1.0
                        - pl.utilization.to_f64_lossy()
                            * pl.staffing.to_f64_lossy()
                            * pl.input_ratio.to_f64_lossy());
                vab[pl.kind as usize] += va;
                let b = &cat.buildings[pl.kind as usize];
                let meth = &b.methods[pl.method as usize];
                let scale = pl.levels as f64
                    * pl.staffing.to_f64_lossy()
                    * plant_tfp(r, m, b).to_f64_lossy()
                    * pl.utilization.to_f64_lossy()
                    * pl.input_ratio.to_f64_lossy();
                for (gi, (g, _)) in meth.outputs.iter().enumerate() {
                    let _ = gi;
                    if let Some(k) = idx.iter().position(|x| *x == Some(*g as usize)) {
                        made[r.market as usize][k] += meth.outputs[gi].1.to_f64_lossy() * scale;
                    }
                }
            }
        }
    }
    let e = &s.world().econ;
    let s1 = stocks(e);
    // Demand of a few goods by source in the last week: plant inputs by building, the rest.
    for key in [
        "cement",
        "steel",
        "automobiles",
        "electricity",
        "fuel",
        "crude_oil",
        "natural_gas",
        "coal",
        "consumer_electronics",
        "appliances",
        "machinery",
        "transportation",
    ] {
        let Some(g) = cat.good(key) else { continue };
        let g = g as usize;
        let total: f64 = e
            .markets
            .iter()
            .map(|m| m.demand[g].to_f64_lossy() + m.net_import[g].to_f64_lossy().min(0.0))
            .sum();
        let mut by = vec![0.0f64; cat.buildings.len()];
        for r in &e.regions {
            let m = &e.markets[r.market as usize];
            for pl in &r.plants {
                let b = &cat.buildings[pl.kind as usize];
                let meth = &b.methods[pl.method as usize];
                let scale = pl.levels as f64
                    * pl.staffing.to_f64_lossy()
                    * plant_tfp(r, m, b).to_f64_lossy()
                    * pl.utilization.to_f64_lossy();
                for (gi, q) in &meth.inputs {
                    if *gi as usize == g {
                        by[pl.kind as usize] += q.to_f64_lossy() * scale;
                    }
                }
            }
        }
        let plants: f64 = by.iter().sum();
        let upkeep: f64 = e
            .markets
            .iter()
            .map(|m| {
                let money = m.pool.to_f64_lossy().max(0.0) * 0.03;
                cat.upkeep_basket
                    .iter()
                    .filter(|(gg, _)| *gg as usize == g)
                    .map(|(_, pm)| money * *pm as f64 / 1000.0 / m.price[g].to_f64_lossy())
                    .sum::<f64>()
            })
            .sum();
        let constr: f64 = e
            .markets
            .iter()
            .map(|m| m.construction[g].to_f64_lossy() / 8.0)
            .sum();
        let mut top: Vec<usize> = (0..by.len()).collect();
        top.sort_by(|a, b| by[*b].total_cmp(&by[*a]));
        let tops: Vec<String> = top
            .iter()
            .take(4)
            .filter(|k| by[**k] > 0.0)
            .map(|k| format!("{} {:.0}%", cat.buildings[*k].key, by[*k] / total * 100.0))
            .collect();
        println!(
            "demand {key}: {:.3e}/yr; plants {:.0}% [{}], upkeep {:.0}%, construction {:.0}%, rest (households, state) {:.0}%",
            total * 52.0,
            plants / total * 100.0,
            tops.join(", "),
            upkeep / total * 100.0,
            constr / total * 100.0,
            (total - plants - upkeep - constr) / total * 100.0
        );
    }
    let tags: Vec<String> = data
        .countries
        .countries
        .iter()
        .map(|c| c.tag.clone())
        .collect();
    let sum = |v: &[f64]| v.iter().sum::<f64>();
    println!(
        "world 1925: target GDP {:.2} tri, measured {:.2} tri ({:.0}%), plant VA full {:.2} tri, running {:.2} tri/yr, mean use {:.2}",
        sum(&target) / 1e12,
        sum(&gdp_end) / 1e12,
        sum(&gdp_end) / sum(&target) * 100.0,
        sum(&pot0) * 52.0 / 1e12,
        sum(&run) / weeks as f64 * 52.0 / 1e12,
        util_sum / weeks as f64
    );
    println!(
        "VA-weighted staffing {:.3}, utilization {:.3}, input ratio {:.3}",
        parts[0] / parts[3],
        parts[1] / parts[3],
        parts[2] / parts[3]
    );
    println!(
        "stocks change over 1925 (bn): pool {:+.0}, treasury {:+.0}, savings {:+.0}",
        (s1.0 - s0.0) / 1e9,
        (s1.1 - s0.1) / 1e9,
        (s1.2 - s0.2) / 1e9
    );
    let mut kb: Vec<usize> = (0..cat.buildings.len()).collect();
    kb.sort_by(|a, b| lost[*b].total_cmp(&lost[*a]));
    println!("VA lost by building (bn/yr at tfp 1): lost / full");
    for &k in kb.iter().take(15) {
        println!(
            "  {:<24} {:>7.1} / {:>8.1}",
            cat.buildings[k].key,
            lost[k] * 52.0 / weeks as f64 / 1e9,
            vab[k] * 52.0 / weeks as f64 / 1e9
        );
    }
    let names = [
        "household income",
        "taxes",
        "payroll",
        "state goods",
        "public services",
        "works",
        "subsistence",
    ];
    for (n, v) in names.iter().zip(flows) {
        println!(
            "  {n:<18} {:>8.0} bn/yr ({:.0}% of measured GDP)",
            v / weeks as f64 * 52.0 / 1e9,
            v / weeks as f64 * 52.0 / sum(&gdp_end) * 100.0
        );
    }
    let mut rows: Vec<usize> = (0..cat.goods.len()).collect();
    rows.sort_by(|a, b| unsold[*b].total_cmp(&unsold[*a]));
    println!("goods by unsold value (bn/yr at base): unsold, short, traded");
    for &g in rows.iter().take(25) {
        let k = 52.0 / weeks as f64 / 1e9;
        println!(
            "  {:<22} {:>7.1} {:>7.1} {:>8.1}",
            cat.goods[g].key,
            unsold[g] * k,
            short[g] * k,
            traded[g] * k
        );
    }
    let mut order: Vec<usize> = (0..nm).collect();
    order.sort_by(|a, b| target[*b].total_cmp(&target[*a]));
    println!(
        "{:<5} {:>9} {:>9} {:>6} {:>9}",
        "tag", "target bn", "measured", "ratio", "VA full"
    );
    for &mi in order.iter().take(25) {
        println!(
            "{:<5} {:>9.1} {:>9.1} {:>6.2} {:>9.1}",
            tags[mi],
            target[mi] / 1e9,
            gdp_end[mi] / 1e9,
            gdp_end[mi] / target[mi],
            pot0[mi] * 52.0 / 1e9
        );
    }
    println!("output in 1925, world (catalogue units): simulated / industry.ron");
    for (k, (key, ind, per)) in GOODS.iter().enumerate() {
        let sim: f64 = made.iter().map(|m| m[k]).sum();
        let real: f64 = if ind.is_empty() {
            f64::NAN
        } else {
            data.industry.as_ref().map_or(f64::NAN, |f| {
                f.countries
                    .iter()
                    .filter_map(|c| c.output.iter().find(|(g, _)| g == ind))
                    .map(|(_, v)| f64::from(*v) * per)
                    .sum()
            })
        };
        println!(
            "  {key:<14} {:>16.0} {:>16.0} {:>6.2}",
            sim,
            real,
            sim / real
        );
    }
    // Growth after 1925: what grows, year by year.
    let years: i32 = std::env::var("BF_DIAG_YEARS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let levels_va = |e: &Economy| -> (f64, f64, f64) {
        let (mut at1, mut at_tfp, mut wsum) = (0.0, 0.0, 0.0);
        for r in &e.regions {
            let m = &e.markets[r.market as usize];
            for p in &r.plants {
                let b = &cat.buildings[p.kind as usize];
                let va = b.methods[p.method as usize]
                    .value_added(|g| cat.goods[g].base_price)
                    .to_f64_lossy()
                    * p.levels as f64
                    * 52.0;
                at1 += va * r.tfp_start.to_f64_lossy();
                at_tfp += va * plant_tfp(r, m, b).to_f64_lossy();
                wsum += b.methods[p.method as usize].workers as f64 * p.levels as f64;
            }
        }
        (at1, at_tfp, wsum)
    };
    // Why a country's industry does or does not grow: every 10 years.
    if let Ok(tag) = std::env::var("BF_DIAG_COUNTRY") {
        let mi = tags.iter().position(|t| *t == tag).ok_or("tag")?;
        for y in (1930..=2000).step_by(10) {
            s.sim.run_until(Date::new(y, 1, 2));
            let e = &s.world().econ;
            let m = &e.markets[mi];
            let mut by: std::collections::BTreeMap<String, (f64, f64, i64)> = Default::default();
            let (mut idle, mut people) = (0i64, 0i64);
            for r in e.regions.iter().filter(|r| r.market as usize == mi) {
                let p = &r.pops[0];
                idle += bf_sim::econ::workers(p.size) - p.employed;
                people += r.population();
                for pl in &r.plants {
                    let b = &cat.buildings[pl.kind as usize];
                    let x = by.entry(b.group.clone()).or_default();
                    x.0 += pl.utilization.to_f64_lossy() * pl.levels as f64;
                    x.1 += pl.levels as f64;
                    x.2 += b.methods[pl.method as usize].workers * pl.levels;
                }
            }
            let gdp = m.gdp_last_year.to_f64_lossy();
            println!(
                "{y}: gdp/head {:.0}, pool {:.1}% of GDP, idle peasant workers {:.0} M of {:.0} M people, hurdle {:.3}, wage/wk {:.1}",
                gdp / people as f64,
                m.pool.to_f64_lossy() / gdp * 100.0,
                idle as f64 / 1e6,
                people as f64 / 1e6,
                bf_sim::econ::monthly::hurdle(m).to_f64_lossy(),
                bf_sim::econ::monthly::market_wages(e)[mi].to_f64_lossy()
            );
            for (g, (u, l, w)) in &by {
                print!(
                    "  {g}: use {:.2} jobs {:.1} M;",
                    u / l.max(1.0),
                    *w as f64 / 1e6
                );
            }
            println!();
            let mut imp: Vec<(f64, String)> = (0..cat.goods.len())
                .filter(|&g| m.demand[g] > laplace_schelling::Fixed::ZERO)
                .map(|g| {
                    let d = m.demand[g].to_f64_lossy();
                    (m.net_import[g].to_f64_lossy() / d, cat.goods[g].key.clone())
                })
                .collect();
            imp.sort_by(|a, b| b.0.total_cmp(&a.0));
            println!(
                "  most imported (share of demand): {}",
                imp.iter()
                    .take(8)
                    .map(|(x, k)| format!("{k} {:.0}%", x * 100.0))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        return Ok(());
    }
    // Colonies against poor sovereigns: plant value added per head split into market jobs per
    // head, the catalogue's value added per job (the plant mix), base productivity and
    // technology, every 10 years.
    if std::env::var_os("BF_DIAG_COLONIES").is_some() {
        let units: [(&str, bool); 17] = [
            ("GBR", true),
            ("FRA", true),
            ("POR", true),
            ("USA", true),
            ("BEL", true),
            ("NLD", true),
            ("JAP", true),
            ("RAJ", false),
            ("EGY", false),
            ("BRA", false),
            ("CHN", false),
            ("TUR", false),
            ("SIA", false),
            ("GBR", false),
            ("NLD", false),
            ("ESP", false),
            ("ESP", true),
        ];
        let last: i32 = std::env::var("BF_DIAG_UNTIL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2025);
        let mut y = 1926;
        while y <= last + 1 {
            s.sim.run_until(Date::new(y, 1, 2));
            force(&mut s, &forced, y);
            let e = &s.world().econ;
            println!("{}:", y - 1);
            for (tag, colony) in units {
                let Some(mi) = tags.iter().position(|t| *t == tag) else {
                    continue;
                };
                let m = &e.markets[mi];
                let (mut va, mut v0, mut vb, mut w, mut pop, mut t0) =
                    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
                for r in e
                    .regions
                    .iter()
                    .filter(|r| r.market as usize == mi && r.colony == colony)
                {
                    pop += r.population() as f64;
                    t0 += r.tfp_start.to_f64_lossy() * r.population() as f64;
                    for p in &r.plants {
                        let b = &cat.buildings[p.kind as usize];
                        let me = &b.methods[p.method as usize];
                        let run = p.levels as f64
                            * p.staffing.to_f64_lossy()
                            * p.utilization.to_f64_lossy();
                        let at1 =
                            me.value_added(|g| cat.goods[g].base_price).to_f64_lossy() * run * 52.0;
                        v0 += at1;
                        vb += at1 * r.tfp_for(b).to_f64_lossy();
                        va += at1 * plant_tfp(r, m, b).to_f64_lossy();
                        w += me.workers as f64 * p.levels as f64 * p.staffing.to_f64_lossy();
                    }
                }
                if std::env::var_os("BF_DIAG_MIX").is_some() {
                    let mut by: std::collections::BTreeMap<String, (f64, f64)> = Default::default();
                    for r in e
                        .regions
                        .iter()
                        .filter(|r| r.market as usize == mi && r.colony == colony)
                    {
                        for p in &r.plants {
                            let b = &cat.buildings[p.kind as usize];
                            let me = &b.methods[p.method as usize];
                            let st = p.levels as f64 * p.staffing.to_f64_lossy();
                            let x = by.entry(b.key.clone()).or_default();
                            x.0 += me.workers as f64 * st;
                            x.1 += me.value_added(|g| cat.goods[g].base_price).to_f64_lossy()
                                * st
                                * p.utilization.to_f64_lossy()
                                * 52.0;
                        }
                    }
                    let mut v: Vec<_> = by.into_iter().collect();
                    v.sort_by(|a, b| b.1.1.total_cmp(&a.1.1));
                    let line: Vec<String> = v
                        .iter()
                        .take(8)
                        .map(|(k, (w2, x))| {
                            format!(
                                "{k} va {:.0}% jobs {:.0}% @1 {:.0}",
                                x / v0.max(1.0) * 100.0,
                                w2 / w.max(1.0) * 100.0,
                                x / w2.max(1.0)
                            )
                        })
                        .collect();
                    println!("    {}", line.join("; "));
                }
                if std::env::var_os("BF_DIAG_USE").is_some() && tag == "NLD" {
                    // Kinds with the most levels: levels, utilization, staffing, input ratio,
                    // share of levels on the newest method the block runs, losing weeks.
                    let mut by: std::collections::BTreeMap<String, [f64; 6]> = Default::default();
                    for r in e
                        .regions
                        .iter()
                        .filter(|r| r.market as usize == mi && r.colony == colony)
                    {
                        for p in &r.plants {
                            let b = &cat.buildings[p.kind as usize];
                            if b.public || b.informal {
                                continue;
                            }
                            let l = p.levels as f64;
                            let x = by.entry(b.key.clone()).or_default();
                            x[0] += l;
                            x[1] += l * p.utilization.to_f64_lossy();
                            x[2] += l * p.staffing.to_f64_lossy();
                            x[3] += l * p.input_ratio.to_f64_lossy();
                            x[4] += l * f64::from(p.method);
                            x[5] += l * f64::from(p.losing_weeks);
                        }
                    }
                    let mut v: Vec<_> = by.into_iter().collect();
                    v.sort_by(|a, b| b.1[0].total_cmp(&a.1[0]));
                    for (k, x) in v.iter().take(12) {
                        let l = x[0].max(1.0);
                        println!(
                            "    use {k:<22} levels {:>8.0} util {:.2} staff {:.2} inputs {:.2} method {:.1} losing {:.1}",
                            x[0],
                            x[1] / l,
                            x[2] / l,
                            x[3] / l,
                            x[4] / l,
                            x[5] / l
                        );
                    }
                }
                if std::env::var_os("BF_DIAG_SITES").is_some() && tag == "NLD" && !colony {
                    use bf_sim::econ::{monthly, prof, workers};
                    let cat: &bf_sim::econ::Catalogue = &cat;
                    let w = s.world();
                    let tree = data.tree.as_deref();
                    let unlocks = bf_sim::econ::Unlocks::new(cat, y - 1, tree, &w.research);
                    let national =
                        bf_sim::econ::local::national(cat, tree, &w.research, e.markets.len());
                    let wage = monthly::market_wages(e)[mi];
                    let rs: Vec<_> = e
                        .regions
                        .iter()
                        .filter(|r| r.market as usize == mi && !r.colony)
                        .collect();
                    let idle: i64 = rs
                        .iter()
                        .map(|r| {
                            let i =
                                |pi: usize| (workers(r.pops[pi].size) - r.pops[pi].employed).max(0);
                            let inf: i64 = cat.informal.map_or(0, |k| {
                                r.plants
                                    .iter()
                                    .filter(|p| p.kind == k)
                                    .map(|p| {
                                        p.levels * cat.buildings[k as usize].methods[0].workers
                                    })
                                    .sum()
                            });
                            i(prof::PEASANT) + i(prof::LABORER) + inf
                        })
                        .sum();
                    println!(
                        "    sites NLD: idle hands {:.2} M, hurdle {:.3}, wage/wk {:.0}, pool {:.1}% of GDP",
                        idle as f64 / 1e6,
                        monthly::hurdle(m).to_f64_lossy(),
                        wage.to_f64_lossy(),
                        m.pool.to_f64_lossy() / m.gdp_last_year.to_f64_lossy().max(1.0) * 100.0
                    );
                    let r = rs.iter().max_by_key(|r| r.population()).unwrap();
                    let view = bf_sim::econ::local::View::last_week(cat, m, r, &national[mi]);
                    for key in [
                        "machinery_plant",
                        "pharmaceutical_plant",
                        "electronics_plant",
                        "construction_sector",
                        "chemical_plant",
                        "steel_mill",
                    ] {
                        let Some(kind) = cat.building(key) else {
                            continue;
                        };
                        let b = &cat.buildings[kind];
                        let (mut used, mut built) = (0.0, 0.0);
                        for r2 in &rs {
                            for p in r2.plants.iter().filter(|p| p.kind as usize == kind) {
                                used += p.levels as f64 * p.utilization.to_f64_lossy();
                                built += p.levels as f64;
                            }
                        }
                        let tfp = bf_sim::econ::plant_tfp(r, m, b);
                        let best = monthly::best_method(
                            cat,
                            kind,
                            &unlocks,
                            (mi, false),
                            &view.price,
                            &view.buy,
                            tfp,
                            wage,
                        );
                        let line = match best {
                            Some((mt, profit)) => {
                                let cap = monthly::level_capital(cat, kind, 0, tfp);
                                let me = &b.methods[mt as usize];
                                let worst = me
                                    .inputs
                                    .iter()
                                    .map(|(g, _)| {
                                        (
                                            view.buy[*g as usize].to_f64_lossy(),
                                            (m.price[*g as usize]
                                                / cat.goods[*g as usize].base_price)
                                                .to_f64_lossy(),
                                            cat.goods[*g as usize].key.clone(),
                                        )
                                    })
                                    .min_by(|a, b| a.0.total_cmp(&b.0));
                                let out = me.outputs.first().map_or(0.0, |(g, _)| {
                                    (m.price[*g as usize] / cat.goods[*g as usize].base_price)
                                        .to_f64_lossy()
                                });
                                format!(
                                    "method {mt} return {:.3} (va {:.0} wages {:.0}/wk per level) out price x{out:.2}, scarcest input {:?}",
                                    (profit * 52
                                        / cap.max(laplace_schelling::Fixed::from_ratio(1, 1000)))
                                    .to_f64_lossy(),
                                    (me.value_added(|g| view.price[g]) * tfp).to_f64_lossy(),
                                    (wage * me.workers).to_f64_lossy(),
                                    worst
                                )
                            }
                            None => "no method (inputs short or locked)".to_string(),
                        };
                        println!(
                            "    site {key:<22} levels {built:>6.0} use {:.2} {line}",
                            used / built.max(1.0)
                        );
                    }
                }
                if !colony {
                    let w = s.world();
                    let st = w.research.stages(mi, false).cloned().unwrap_or_default();
                    let ind = st
                        .iter()
                        .filter(|x| **x >= bf_sim::research::Stage::Industrial)
                        .count();
                    let dif = st
                        .iter()
                        .filter(|x| **x >= bf_sim::research::Stage::Diffused)
                        .count();
                    let col = w.research.stages(mi, true).map_or(0, |c| {
                        c.iter()
                            .filter(|x| **x >= bf_sim::research::Stage::Industrial)
                            .count()
                    });
                    println!(
                        "    research {tag}: points/month {:.2}, industrial {ind}, diffused {dif}, colonies industrial {col}",
                        w.research.points.get(mi).map_or(0.0, |p| p.to_f64_lossy())
                    );
                }
                let wages = bf_sim::econ::monthly::block_wages(e)[mi];
                println!(
                    "  {tag}{} wage/yr {:>6.0} (metropole {:>6.0}) va/head {:>6.0} jobs/head {:>5.1}% va/job@1 {:>6.0} base {:.2} (1925 {:.2}) tech {:.2}",
                    if colony { "/col" } else { "    " },
                    wages[usize::from(colony)].to_f64_lossy() * 52.0,
                    wages[0].to_f64_lossy() * 52.0,
                    va / pop.max(1.0),
                    w / pop.max(1.0) * 100.0,
                    v0 / w.max(1.0),
                    vb / v0.max(1.0),
                    t0 / pop.max(1.0),
                    va / vb.max(1.0)
                );
            }
            y += 10;
        }
        return Ok(());
    }
    // Productivity by decade for a few countries: base (regions) and technology (by group).
    if std::env::var_os("BF_DIAG_TFP").is_some() {
        let groups = bf_sim::economy::BUILDING_GROUPS;
        let gi = |k: &str| groups.iter().position(|g| *g == k).unwrap_or(0);
        for y in (1935..=2025).step_by(10) {
            for year in (y - 9)..=y {
                s.sim.run_until(Date::new(year, 1, 2));
                force(&mut s, &forced, year);
            }
            let w = s.world();
            let e = &w.econ;
            print!("{y}:");
            for tag in [
                "USA", "GBR", "GER", "FRA", "JAP", "SOV", "BRA", "MEX", "CHN", "RAJ",
            ] {
                let Some(mi) = tags.iter().position(|t| t == tag) else {
                    continue;
                };
                let m = &e.markets[mi];
                let rs: Vec<_> = e
                    .regions
                    .iter()
                    .filter(|r| r.market as usize == mi && !r.colony)
                    .collect();
                let pop: i64 = rs.iter().map(|r| r.population()).sum();
                let base: f64 = rs
                    .iter()
                    .map(|r| r.tfp.to_f64_lossy() * r.population() as f64)
                    .sum::<f64>()
                    / pop.max(1) as f64;
                let t = |k: &str| m.tech_tfp.get(gi(k)).map_or(1.0, |x| x.to_f64_lossy());
                let jobs = crate::history::sector_shares(&data, w, mi);
                // Technologies after 1925 at each stage or past it: science, prototype,
                // industry, diffused.
                let mut at = [0usize; 4];
                if let Some(tree) = &data.tree {
                    for (t, tech) in tree.techs.iter().enumerate() {
                        if tech.era == 0 {
                            continue;
                        }
                        let st = w.research.stage[mi][t] as usize;
                        for (k, n) in at.iter_mut().enumerate() {
                            if st > k {
                                *n += 1;
                            }
                        }
                    }
                }
                print!(
                    " {tag} gdp {:.0} base {:.2} heavy {:.2} serv {:.2} agr {:.2} jobs {:.0}/{:.0}/{:.0} st {}/{}/{}/{} |",
                    m.gdp_last_year.to_f64_lossy() / pop.max(1) as f64,
                    base,
                    t("heavy_industry"),
                    t("services"),
                    t("agriculture"),
                    jobs[0] * 100.0,
                    jobs[1] * 100.0,
                    jobs[2] * 100.0,
                    at[0],
                    at[1],
                    at[2],
                    at[3]
                );
            }
            println!();
        }
        return Ok(());
    }
    for y in 1..=years {
        s.sim.run_until(Date::new(1926 + y, 1, 2));
        let e = &s.world().econ;
        let gdp: f64 = e
            .markets
            .iter()
            .map(|m| m.gdp_last_year.to_f64_lossy())
            .sum();
        let (a, b, w) = levels_va(e);
        let pool: f64 = e.markets.iter().map(|m| m.pool.to_f64_lossy()).sum();
        let queue: usize = e.markets.iter().map(|m| m.queue.len()).sum();
        println!(
            "{}: GDP {:.2} tri, VA of levels at 1925 tfp {:.2}, at today's tfp {:.2}, jobs {:.0} M, pool {:.0} bn, queue {}",
            1925 + y,
            gdp / 1e12,
            a / 1e12,
            b / 1e12,
            w / 1e6,
            pool / 1e9,
            queue
        );
    }
    Ok(())
}

/// `BF_LAWS`: (from year, tag, law index) to hold, in the order given.
fn forced_laws(data: &GameData) -> Result<Vec<(i32, String, u16)>, String> {
    let Ok(spec) = std::env::var("BF_LAWS") else {
        return Ok(Vec::new());
    };
    let p = data.politics.as_ref().ok_or("BF_LAWS: no politics data")?;
    let mut out = Vec::new();
    for part in spec.split(';').filter(|x| !x.trim().is_empty()) {
        let (who, laws) = part
            .split_once('=')
            .ok_or_else(|| format!("BF_LAWS: {part:?} is not TAG[@year]=law+law"))?;
        let (tag, from) = match who.split_once('@') {
            Some((t, y)) => (
                t,
                y.trim()
                    .parse::<i32>()
                    .map_err(|_| format!("BF_LAWS: bad year in {who:?}"))?,
            ),
            None => (who, 1925),
        };
        for law in laws.split('+') {
            let i = p
                .law(law.trim())
                .ok_or_else(|| format!("BF_LAWS: unknown law {law:?}"))?;
            out.push((from, tag.trim().to_string(), i as u16));
        }
    }
    Ok(out)
}

fn force(s: &mut Session, laws: &[(i32, String, u16)], year: i32) {
    for (_, tag, law) in laws.iter().filter(|l| l.0 <= year) {
        s.submit(bf_sim::game::GameCommand::SetLaw {
            tag: tag.clone(),
            law: *law,
        });
    }
}
