//! `--diag`: the 1925 economy before the first tick and after a few weeks, for calibration.

use bf_sim::data::GameData;
use bf_sim::game::Session;
use laplace_schelling::{Date, Fixed};

pub fn main() -> Result<(), String> {
    let root = crate::data_root::root();
    let data = GameData::load(&root)?;
    let cat = data.catalogue.clone();
    let world = data.new_world();
    let e = &world.econ;
    let mut va = 0.0;
    for r in &e.regions {
        let tfp = r.tfp.to_f64_lossy();
        for p in &r.plants {
            let m = &cat.buildings[p.kind as usize].methods[p.method as usize];
            let out: f64 = m
                .outputs
                .iter()
                .map(|(g, q)| q.to_f64_lossy() * cat.goods[*g as usize].base_price.to_f64_lossy())
                .sum();
            let inp: f64 = m
                .inputs
                .iter()
                .map(|(g, q)| q.to_f64_lossy() * cat.goods[*g as usize].base_price.to_f64_lossy())
                .sum();
            va += (out - inp) * tfp * 52.0 * p.levels as f64;
        }
    }
    let pop: i64 = e.regions.iter().map(|r| r.population()).sum();
    let tfps: Vec<f64> = e.markets.iter().map(|m| m.tfp.to_f64_lossy()).collect();
    let at_cap = tfps.iter().filter(|t| **t >= 2.99).count();
    println!(
        "1925: pop {:.0} M, plant VA {:.0} bn/yr, markets with tfp at cap {}/{}",
        pop as f64 / 1e6,
        va / 1e9,
        at_cap,
        tfps.len()
    );
    let mut by_group = std::collections::BTreeMap::new();
    for r in &e.regions {
        for p in &r.plants {
            *by_group
                .entry(cat.buildings[p.kind as usize].group.clone())
                .or_insert(0i64) += p.levels;
        }
    }
    println!("levels by group: {by_group:?}");
    let workers: i64 = e
        .regions
        .iter()
        .flat_map(|r| r.pops.iter())
        .map(|p| p.employed)
        .sum();
    println!("employed workers {:.0} M", workers as f64 / 1e6);
    // How much of a household budget the needs plan spends at base prices.
    let base: Vec<Fixed> = cat.goods.iter().map(|g| g.base_price).collect();
    for (tag, pop, gdppc) in [
        ("USA", 115_000_000i64, 11_150.0f64),
        ("BRA", 30_000_000, 1_298.0),
        ("RAJ", 330_000_000, 1_119.0),
    ] {
        let budget = Fixed::from_f64_lossy(pop as f64 * gdppc * 0.7 / 52.0);
        let plan = bf_sim::econ::week::plan_pop_public(&cat, &base, 1925, pop, budget, Fixed::ZERO);
        let spent: f64 = plan
            .iter()
            .map(|(g, q)| q.to_f64_lossy() * base[*g as usize].to_f64_lossy())
            .sum();
        println!(
            "{tag}: budget/wk {:.2} bn, plan spends {:.2} bn ({:.0}%)",
            budget.to_f64_lossy() / 1e9,
            spent / 1e9,
            spent / budget.to_f64_lossy() * 100.0
        );
    }
    let mut s = Session::new(
        world,
        1,
        Date::new(1925, 1, 1),
        0,
        Some(bf_sim::game::Rules {
            catalogue: cat.clone(),
            tree: None,
            politics: None,
            communities: None,
            borders: None,
            influence: None,
            war: None,
        }),
    );
    let world_supply = |e: &bf_sim::econ::Economy| -> Vec<f64> {
        (0..cat.goods.len())
            .map(|gi| {
                e.markets
                    .iter()
                    .map(|m| m.supply[gi].to_f64_lossy() - m.net_import[gi].to_f64_lossy().max(0.0))
                    .sum()
            })
            .collect()
    };
    let world_demand = |e: &bf_sim::econ::Economy| -> Vec<f64> {
        (0..cat.goods.len())
            .map(|gi| {
                e.markets
                    .iter()
                    .map(|m| m.demand[gi].to_f64_lossy() + m.net_import[gi].to_f64_lossy().min(0.0))
                    .sum()
            })
            .collect()
    };
    let mut first: Option<(Vec<f64>, Vec<f64>)> = None;
    for week in 0..6 {
        for _ in 0..7 {
            s.sim.step();
        }
        if week == 0 {
            first = Some((world_supply(&s.world().econ), world_demand(&s.world().econ)));
        }
        let e = &s.world().econ;
        let gdp: f64 = e.markets.iter().map(|m| m.gdp_running.to_f64_lossy()).sum();
        let staffing: f64 = e
            .regions
            .iter()
            .flat_map(|r| r.plants.iter())
            .map(|p| p.staffing.to_f64_lossy() * p.levels as f64)
            .sum::<f64>()
            / e.regions
                .iter()
                .flat_map(|r| r.plants.iter())
                .map(|p| p.levels as f64)
                .sum::<f64>()
                .max(1.0);
        let input: f64 = e
            .regions
            .iter()
            .flat_map(|r| r.plants.iter())
            .map(|p| p.input_ratio.to_f64_lossy() * p.levels as f64)
            .sum::<f64>()
            / e.regions
                .iter()
                .flat_map(|r| r.plants.iter())
                .map(|p| p.levels as f64)
                .sum::<f64>()
                .max(1.0);
        let pool: f64 = e.markets.iter().map(|m| m.pool.to_f64_lossy()).sum();
        let treasury: f64 = e.markets.iter().map(|m| m.treasury.to_f64_lossy()).sum();
        let savings: f64 = e
            .regions
            .iter()
            .flat_map(|r| r.pops.iter())
            .map(|p| p.savings.to_f64_lossy())
            .sum();
        let income: f64 = e
            .regions
            .iter()
            .flat_map(|r| r.pops.iter())
            .map(|p| p.income.to_f64_lossy())
            .sum();
        let needs: f64 = e
            .regions
            .iter()
            .flat_map(|r| r.pops.iter())
            .map(|p| p.satisfaction[0].to_f64_lossy() * p.size as f64)
            .sum::<f64>()
            / pop as f64;
        println!(
            "week {}: GDP so far {:.0} bn, staffing {:.2}, inputs {:.2}, income/wk {:.0} bn, savings {:.0} bn, pool {:.0} bn, treasury {:.0} bn, tier1 {:.2}",
            week + 1,
            gdp / 1e9,
            staffing,
            input,
            income / 1e9,
            savings / 1e9,
            pool / 1e9,
            treasury / 1e9,
            needs
        );
    }
    let years: i32 = std::env::var("BF_DIAG_YEARS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if years > 0 {
        s.sim.run_until(Date::new(1925 + years, 1, 1));
        let e = &s.world().econ;
        println!(
            "after {years} years, markets with trade where each good sits at the floor / the ceiling:"
        );
        for (gi, g) in cat.goods.iter().enumerate() {
            let (mut floor, mut ceil, mut active) = (0, 0, 0);
            for m in &e.markets {
                if m.supply[gi] <= Fixed::ZERO && m.demand[gi] <= Fixed::ZERO {
                    continue;
                }
                active += 1;
                let r = m.price[gi] / g.base_price;
                if r <= Fixed::from_raw(260) {
                    floor += 1;
                } else if r >= Fixed::from_raw(3_900) {
                    ceil += 1;
                }
            }
            if floor + ceil > active / 3 {
                println!(
                    "  {:<22} floor {:>2}  ceiling {:>2}  of {:>2}",
                    g.key, floor, ceil, active
                );
            }
        }
        return Ok(());
    }
    let e = &s.world().econ;
    if let Some((s1, d1)) = &first {
        let s6 = world_supply(e);
        let d6 = world_demand(e);
        println!("supply and demand, week 1 → week 6 (largest supply falls):");
        let mut rows: Vec<(f64, usize)> = (0..cat.goods.len())
            .filter(|&g| s1[g] > 0.0)
            .map(|g| (s6[g] / s1[g], g))
            .collect();
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (r, g) in rows.iter().take(15) {
            println!(
                "  {:<22} supply ×{:.2}  demand ×{:.2}",
                cat.goods[*g].key,
                r,
                d6[*g] / d1[*g].max(1e-9)
            );
        }
    }
    println!("world shortages (buy < 0.8) after 6 weeks:");
    for (gi, g) in cat.goods.iter().enumerate() {
        let (mut sup, mut dem) = (0.0, 0.0);
        for m in &e.markets {
            sup += m.supply[gi].to_f64_lossy();
            dem += m.demand[gi].to_f64_lossy();
        }
        if dem > 0.0 && sup / dem < 0.8 {
            println!("  {:<22} S/D {:.2}", g.key, sup / dem);
        }
    }
    let usa = data
        .countries
        .countries
        .iter()
        .position(|c| c.tag == "USA")
        .unwrap();
    let m = &e.markets[usa];
    println!("USA prices / base, buy, sell:");
    for (gi, g) in cat.goods.iter().enumerate() {
        if m.demand[gi] > Fixed::ZERO || m.supply[gi] > Fixed::ZERO {
            println!(
                "  {:<22} p/base {:>5.2}  S {:>14.0}  D {:>14.0}  buy {:.2} sell {:.2}",
                g.key,
                (m.price[gi] / g.base_price).to_f64_lossy(),
                m.supply[gi].to_f64_lossy(),
                m.demand[gi].to_f64_lossy(),
                m.buy_ratio[gi].to_f64_lossy(),
                m.sell_ratio[gi].to_f64_lossy()
            );
        }
    }
    Ok(())
}
