//! `--diag-1925`: the first year of the economy against the scenario, for calibration.
//!
//! Runs 1925 with the game's rules and prints, by country and for the world, the GDP the
//! scenario asks for, the GDP the market measured, what the plants could make at base prices,
//! the money that stayed in stocks (pool, treasury, savings), and the 1925 output of the goods
//! `industry.ron` gives against what the plants made.

use bf_sim::data::GameData;
use bf_sim::econ::{Economy, payments, plant_tfp};
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
    let root = crate::data_root::root();
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
    let mut s = Session::new(
        world,
        1,
        Date::new(1925, 1, 1),
        0,
        Some(crate::game::rules(&data)),
    );
    // Counterfactual laws held all century: `BF_LAWS="CHN=planned_economy+closed_capital+autarky"`
    // (several entries separated by `;`); `CHN@1980=...` holds them from that year on, over the
    // earlier ones of the same group. Applied on the first day and every 1 January after, so
    // that no movement undoes them.
    let forced = forced_laws(&data)?;
    force(&mut s, &forced, 1925);
    // Tariffs fixed by good from the first week (`BF_TARIFF="GBR:textiles=1000,steel=300;..."`,
    // rates in thousandths), as the government's table would (`GameCommand::SetGoodsTariff`).
    if let Ok(spec) = std::env::var("BF_TARIFF") {
        for entry in spec.split(';').filter(|e| !e.trim().is_empty()) {
            let (tag, goods) = entry
                .split_once(':')
                .ok_or_else(|| format!("BF_TARIFF: no ':' in {entry}"))?;
            for kv in goods.split(',') {
                let (good, rate) = kv
                    .split_once('=')
                    .ok_or_else(|| format!("BF_TARIFF: no '=' in {kv}"))?;
                let g = cat
                    .good(good.trim())
                    .ok_or_else(|| format!("BF_TARIFF: unknown good {good}"))?;
                let rate: u16 = rate.trim().parse().map_err(|e| format!("BF_TARIFF: {e}"))?;
                s.submit(bf_sim::game::GameCommand::SetGoodsTariff {
                    tag: tag.trim().to_string(),
                    goods: vec![g],
                    per_mille: Some(rate),
                });
            }
        }
    }
    // Weekly prices and quantities (`BF_DIAG_WEEKLY=TAG,TAG[:weeks]`), to see whether trade
    // makes them oscillate: one CSV line per market, tradable good and week (price over base,
    // supply, demand and net imports at base prices).
    if let Ok(spec) = std::env::var("BF_DIAG_WEEKLY") {
        let (tags, weeks) = spec.split_once(':').unwrap_or((spec.as_str(), "104"));
        let weeks: i32 = weeks.parse().unwrap_or(104);
        let idx: Vec<(String, usize)> = tags
            .split(',')
            .filter_map(|t| {
                let t = t.trim();
                data.countries
                    .countries
                    .iter()
                    .position(|c| c.tag == t)
                    .map(|i| (t.to_string(), i))
            })
            .collect();
        println!("week,tag,good,price,supply,demand,net");
        let mut d = Date::new(1925, 1, 5);
        for w in 0..weeks {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            let e = &s.world().econ;
            for (tag, i) in &idx {
                let m = &e.markets[*i];
                for (gi, good) in cat.goods.iter().enumerate().filter(|(_, g)| g.tradable) {
                    let b = good.base_price;
                    println!(
                        "{w},{tag},{},{:.5},{:.6e},{:.6e},{:.6e}",
                        good.key,
                        (m.price[gi] / b).to_f64_lossy(),
                        (m.supply[gi] * b).to_f64_lossy(),
                        (m.demand[gi] * b).to_f64_lossy(),
                        (m.net_import[gi] * b).to_f64_lossy()
                    );
                }
            }
        }
        return Ok(());
    }
    // Trade's volume (`BF_DIAG_VOLUME=<years>`, docs/trade.pt-BR.md, section 14): by year, the
    // world's exports plus imports by pair over GDP, imports over the use of tradable goods, the
    // margin over trade, world GDP, and the trade over GDP of the five largest economies.
    if let Ok(years) = std::env::var("BF_DIAG_VOLUME") {
        let years: i32 = years.parse().unwrap_or(11);
        let end = Date::new(1925 + years, 1, 1);
        let mut d = Date::new(1925, 1, 5);
        let ng = cat.goods.len();
        let (mut year, mut weeks) = (1925, 0u32);
        let (mut used, mut margin, mut volume) = (vec![0f64; 300], 0f64, 0f64);
        let mut top: Vec<usize> = Vec::new();
        while d < end {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            let e = &s.world().econ;
            if used.len() != e.markets.len() {
                used = vec![0f64; e.markets.len()];
            }
            for (mi, m) in e.markets.iter().enumerate() {
                for gi in 0..ng {
                    if !cat.goods[gi].tradable {
                        continue;
                    }
                    let base = cat.goods[gi].base_price;
                    used[mi] +=
                        (m.demand[gi].max(laplace_schelling::Fixed::ZERO) * base).to_f64_lossy();
                    let n = m.net_import[gi];
                    if n > laplace_schelling::Fixed::ZERO {
                        volume += (n * base).to_f64_lossy();
                        margin += (n * m.sell_ratio[gi] * m.price[gi]).to_f64_lossy();
                    } else if n < laplace_schelling::Fixed::ZERO {
                        margin += (n * m.buy_ratio[gi] * m.price[gi]).to_f64_lossy();
                    }
                }
            }
            weeks += 1;
            if d.year != year {
                let n = e.markets.len();
                let t = &e.trade;
                let v = |a: usize, b: usize| {
                    t.last_year.get(a * n + b).map_or(0.0, |x| x.to_f64_lossy())
                };
                let gdp: Vec<f64> = e
                    .markets
                    .iter()
                    .map(|m| m.gdp_last_year.to_f64_lossy())
                    .collect();
                if top.is_empty() {
                    let mut o: Vec<usize> = (0..n).collect();
                    o.sort_by(|a, b| gdp[*b].total_cmp(&gdp[*a]));
                    o.truncate(5);
                    top = o;
                }
                let world: f64 = (0..n)
                    .flat_map(|a| (0..n).map(move |b| (a, b)))
                    .map(|(a, b)| v(a, b))
                    .sum();
                let wg: f64 = gdp.iter().sum();
                let imp_all: f64 = world;
                let use_all: f64 = used.iter().sum::<f64>() * 52.0 / f64::from(weeks);
                let mut line = format!(
                    "{} world: trade {:.2} % of GDP, imports {:.1} % of tradable use, margin {:.1} % of trade, GDP {:.4e};",
                    year - 1 + 1,
                    2.0 * world / wg * 100.0,
                    imp_all / use_all.max(1.0) * 100.0,
                    margin / volume.max(1.0) * 100.0,
                    wg
                );
                for &i in &top {
                    let x: f64 = (0..n).map(|b| v(i, b)).sum();
                    let m: f64 = (0..n).map(|a| v(a, i)).sum();
                    let tag = &data.countries.countries[i].tag;
                    line += &format!(" {tag} {:.1} %", (x + m) / gdp[i].max(1.0) * 100.0);
                }
                println!("{line}");
                used.iter_mut().for_each(|u| *u = 0.0);
                (margin, volume) = (0.0, 0.0);
                (year, weeks) = (d.year, 0);
            }
        }
        return Ok(());
    }
    // The balance of payments (`BF_DIAG_PAYMENTS=TAG,TAG[:years]`, `econ::payments`): by year,
    // each country's goods, services, income (bond interest, the ledger's interest,
    // remittances), transfers (trade's margins, treaties, occupation), financial account and
    // what is not settled, in % of its GDP; then the world's sums, which must be about zero.
    if let Ok(spec) = std::env::var("BF_DIAG_PAYMENTS") {
        let (tags, years) = spec.split_once(':').unwrap_or((spec.as_str(), "6"));
        let years: i32 = years.parse().unwrap_or(6);
        let idx: Vec<(String, usize)> = tags
            .split(',')
            .filter_map(|t| {
                let t = t.trim();
                data.countries
                    .countries
                    .iter()
                    .position(|c| c.tag == t)
                    .map(|i| (t.to_string(), i))
            })
            .collect();
        let mut d = Date::new(1925, 1, 5);
        let mut year = 1925;
        println!(
            "year tag: goods services | interest ledger remit | transfers | current | financial | not settled  (% of GDP)"
        );
        while year < 1925 + years {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            // The record closes its year in the first economy week of January.
            let w = s.world();
            let p = &w.payments;
            if p.year <= year {
                continue;
            }
            let pct = |v: laplace_schelling::Fixed, gdp: f64| v.to_f64_lossy() / gdp * 100.0;
            for (tag, i) in &idx {
                let Some(l) = p.markets.get(*i).map(|x| x.last_year) else {
                    continue;
                };
                let gdp = w.econ.markets[*i].gdp_last_year.to_f64_lossy().max(1.0);
                println!(
                    "{year} {tag}: {:+.2} {:+.2} | {:+.2} {:+.2} {:+.2} | {:+.2} | {:+.2} | {:+.2} | {:+.2}",
                    pct(l.goods(), gdp),
                    pct(l.services(), gdp),
                    pct(l.interest(), gdp),
                    pct(l.ledger_interest(), gdp),
                    pct(l.remittances(), gdp),
                    pct(l.transfers(), gdp),
                    pct(l.current_account(), gdp),
                    pct(l.financial_account(), gdp),
                    pct(l.not_settled(), gdp),
                );
            }
            // The world: each line's sum over every country, in % of world GDP, and what the
            // debtors kept in weeks when nobody held a claim abroad.
            let world_gdp: f64 = w
                .econ
                .markets
                .iter()
                .map(|m| m.gdp_last_year.to_f64_lossy())
                .sum::<f64>()
                .max(1.0);
            let mut t = payments::Lines::default();
            for m in &p.markets {
                t += m.last_year;
            }
            println!(
                "{year} world: {:+.4} {:+.4} | {:+.4} {:+.4} {:+.4} | {:+.4} | {:+.4} | {:+.4} | {:+.4}; exports {:.2} % of GDP, kept for want of a lender {:.6} % of GDP",
                pct(t.goods(), world_gdp),
                pct(t.services(), world_gdp),
                pct(t.interest(), world_gdp),
                pct(t.ledger_interest(), world_gdp),
                pct(t.remittances(), world_gdp),
                pct(t.transfers(), world_gdp),
                pct(t.current_account(), world_gdp),
                pct(t.financial_account(), world_gdp),
                pct(t.not_settled(), world_gdp),
                pct(t.exports, world_gdp),
                pct(
                    p.unreceived.last_year.interest_received
                        + p.unreceived.last_year.principal_received,
                    world_gdp
                ),
            );
            year = p.year;
        }
        return Ok(());
    }
    // The gold standard (`BF_DIAG_GOLD=TAG,TAG[:years]`, `econ::gold`, with `gold_standard` and
    // `payments_settle` on): by year and country, the regime and parity at the end of the year,
    // the reserves (gross, and net of what is owed abroad) in % of GDP, the cover against the
    // legal one, the central bank's rate against the Taylor rule (yearly averages of the weekly
    // values) and the year's inflation; with `capital_follows_rate` on, the year's private
    // capital (long and short, net inflow) and its income in % of GDP, and whether capital
    // controls are on; then every change of regime with its date.
    if let Ok(spec) = std::env::var("BF_DIAG_GOLD") {
        let (tags, years) = spec.split_once(':').unwrap_or((spec.as_str(), "8"));
        let years: i32 = years.parse().unwrap_or(8);
        let rules = data
            .gold
            .clone()
            .ok_or("BF_DIAG_GOLD: data/economy/gold.ron is not loaded")?;
        let idx: Vec<(String, usize)> = tags
            .split(',')
            .filter_map(|t| {
                let t = t.trim();
                data.countries
                    .countries
                    .iter()
                    .position(|c| c.tag == t)
                    .map(|i| (t.to_string(), i))
            })
            .collect();
        let f = |v: laplace_schelling::Fixed| v.to_f64_lossy();
        let mut d = Date::new(1925, 1, 5);
        let mut year = 1925;
        // Weekly sums: rate, its points over Taylor, cover; and the weeks counted.
        let mut sums = vec![(0.0f64, 0.0f64, 0.0f64); idx.len()];
        let mut weeks = 0.0f64;
        println!(
            "gold standard, by year: regime parity | reserves gross/net % GDP | money the bank issued % GDP | cover vs legal | rate (Taylor + cover's points), yearly mean | inflation of the year | private capital long/short net inflow and income, % GDP [controls]"
        );
        while year < 1925 + years {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            let w = s.world();
            if w.gold.is_empty() {
                continue;
            }
            if d.year > year {
                let net = w.reserves.net();
                for (k, (tag, i)) in idx.iter().enumerate() {
                    let m = &w.econ.markets[*i];
                    let st = &w.gold.countries[*i];
                    let gdp = f(m.gdp_last_year).max(1.0);
                    let gross = w.reserves.markets.get(*i).map_or(0.0, |r| f(r.total()));
                    let (rate, add, cover) = sums[k];
                    let n = weeks.max(1.0);
                    let l = w
                        .payments
                        .markets
                        .get(*i)
                        .map(|p| {
                            // The economy week of the new year may not have run yet.
                            if w.payments.year == year {
                                p.year
                            } else {
                                p.last_year
                            }
                        })
                        .unwrap_or_default();
                    let capital = if w.capital.is_empty() {
                        String::new()
                    } else {
                        format!(
                            " | capital {:+.2}/{:+.2}, income {:+.2}{}",
                            f(l.capital_long()) / gdp * 100.0,
                            f(l.capital_short()) / gdp * 100.0,
                            f(l.capital_income()) / gdp * 100.0,
                            if w.capital.controls.get(*i).copied().unwrap_or(false) {
                                " [controls]"
                            } else {
                                ""
                            }
                        )
                    };
                    println!(
                        "{year} {tag}: {:?} {:.1} | reserves {:.2}/{:.2} | issued {:.1} | cover {:.0} % (legal {:.0} %) | rate {:.2} = Taylor {:.2} + {:.2} | inflation {:+.1} %{capital}",
                        st.regime,
                        if st.regime == bf_sim::econ::gold::Regime::Float {
                            f(m.money.exchange_rate)
                        } else {
                            f(st.parity)
                        },
                        gross / gdp * 100.0,
                        f(net[*i]) / gdp * 100.0,
                        f(m.money.debt_bank + m.money.credit) / gdp * 100.0,
                        cover / n * 100.0,
                        f(rules.legal_cover[*i]) * 100.0,
                        rate / n,
                        (rate - add) / n,
                        add / n,
                        f(m.money.inflation_last_year),
                    );
                }
                sums.iter_mut().for_each(|x| *x = (0.0, 0.0, 0.0));
                weeks = 0.0;
                year = d.year;
            }
            // This week's values (the economy week ran on Monday).
            for (k, (_, i)) in idx.iter().enumerate() {
                let m = &w.econ.markets[*i];
                let st = &w.gold.countries[*i];
                let add = if st.regime == bf_sim::econ::gold::Regime::Gold {
                    ((f(rules.legal_cover[*i]) - f(st.cover)).max(0.0)
                        * 100.0
                        * f(rules.rate_per_point))
                    .min(f(rules.max_add))
                } else {
                    0.0
                };
                sums[k].0 += f(m.money.policy_rate);
                sums[k].1 += add;
                sums[k].2 += f(st.cover);
            }
            weeks += 1.0;
        }
        let w = s.world();
        let tag = |mi: usize| {
            data.countries
                .countries
                .get(mi)
                .map_or_else(|| format!("#{mi}"), |c| c.tag.to_string())
        };
        println!("changes of regime:");
        for e in &w.gold.events {
            println!(
                "{}-{:02} {}: {:?} -> {:?} at {:.1}",
                e.year,
                e.month,
                tag(usize::from(e.market)),
                e.change,
                e.regime,
                f(e.parity)
            );
        }
        return Ok(());
    }
    // Settlement in money and reserves (`BF_DIAG_SETTLE=TAG,TAG[:years]`,
    // `econ::payments::settle`). By year, in % of GDP:
    // - where each country's "not settled" sits: goods and services, trade's margins split into
    //   the market's own balance (its buyers' payment for its net imports given back to its own
    //   pool, less its sellers' pay for its net exports) and its share of the world's balance,
    //   and the rest (income, transfers between treasuries, financial account);
    // - with `payments_settle` on, the money that changed country (the world's sum of the
    //   balances settled and of the pools' moves, over world GDP), and each country's reserves
    //   at the start and end of the year and what moved them.
    if let Ok(spec) = std::env::var("BF_DIAG_SETTLE") {
        let (tags, years) = spec.split_once(':').unwrap_or((spec.as_str(), "6"));
        let years: i32 = years.parse().unwrap_or(6);
        let idx: Vec<(String, usize)> = tags
            .split(',')
            .filter_map(|t| {
                let t = t.trim();
                data.countries
                    .countries
                    .iter()
                    .position(|c| c.tag == t)
                    .map(|i| (t.to_string(), i))
            })
            .collect();
        let f = |v: laplace_schelling::Fixed| v.to_f64_lossy();
        let mut d = Date::new(1925, 1, 5);
        let mut year = 1925;
        let mut own = vec![0.0f64; 0];
        let mut share = vec![0.0f64; 0];
        let mut start: Vec<(f64, f64)> = Vec::new();
        println!(
            "settlement, % of GDP; residual: not settled = -(trade + margins + rest), margins = own balance - world share"
        );
        while year < 1925 + years {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            let w = s.world();
            let nm = w.econ.markets.len();
            own.resize(nm, 0.0);
            share.resize(nm, 0.0);
            let p = &w.payments;
            if p.year > year {
                let world_gdp: f64 = w
                    .econ
                    .markets
                    .iter()
                    .map(|m| f(m.gdp_last_year))
                    .sum::<f64>()
                    .max(1.0);
                let res = &w.reserves;
                let held = |mi: usize| -> (f64, f64) {
                    res.markets
                        .get(mi)
                        .map_or((0.0, 0.0), |r| (f(r.total()), f(res.owed(mi))))
                };
                for (tag, i) in &idx {
                    let Some(l) = p.markets.get(*i).map(|x| x.last_year) else {
                        continue;
                    };
                    let gdp = f(w.econ.markets[*i].gdp_last_year).max(1.0);
                    let pc = |v: f64| v / gdp * 100.0;
                    let rest = f(l.reserve_flow() - l.trade());
                    println!(
                        "{year} {tag}: residual: trade {:+.2}, margins {:+.2} (own {:+.2}, world share {:+.2}), rest {:+.2}; not settled {:+.2}",
                        pc(f(l.trade())),
                        pc(f(l.margins)),
                        pc(own[*i]),
                        pc(share[*i]),
                        pc(rest),
                        pc(f(l.not_settled())),
                    );
                    if !res.is_empty() {
                        let r = res.record.get(*i).map(|x| x.last_year).unwrap_or_default();
                        let (a, oa) = start.get(*i).copied().unwrap_or((0.0, 0.0));
                        let (b, ob) = held(*i);
                        println!(
                            "{year} {tag}: reserves {:.2} -> {:.2} ({:+.2}); settled: balance {:+.2} (pool {:+.2}), new gold {:+.2}, unpaid {:.2}; owed abroad {:.2} -> {:.2}",
                            pc(a),
                            pc(b),
                            pc(b - a),
                            pc(f(r.balance)),
                            pc(f(r.pool)),
                            pc(f(r.gold)),
                            pc(f(r.unpaid)),
                            pc(oa),
                            pc(ob),
                        );
                    }
                }
                let own_world: f64 = own.iter().sum();
                if res.is_empty() {
                    println!(
                        "{year} world: the markets' own balances add up to {:+.4} % of world GDP (money the clearing took out, given back as margins)",
                        own_world / world_gdp * 100.0
                    );
                } else {
                    let sum = |g: &dyn Fn(&bf_sim::econ::payments::Settled) -> f64| -> f64 {
                        res.record.iter().map(|x| g(&x.last_year)).sum()
                    };
                    let reserves: f64 = (0..nm).map(|mi| held(mi).0).sum();
                    println!(
                        "{year} world: money that changed country {:.2} % of world GDP (sum of |balance| {:.2} %, of |pool| {:.2} %); new gold {:.4} %, unpaid {:.3} %; reserves {:.2} % of world GDP; last week's mismatch {}",
                        sum(&|x| f(x.balance).abs()) / 2.0 / world_gdp * 100.0,
                        sum(&|x| f(x.balance).abs()) / world_gdp * 100.0,
                        sum(&|x| f(x.pool).abs()) / world_gdp * 100.0,
                        sum(&|x| f(x.gold)) / world_gdp * 100.0,
                        sum(&|x| f(x.unpaid)) / world_gdp * 100.0,
                        reserves / world_gdp * 100.0,
                        res.mismatch,
                    );
                }
                start = (0..nm).map(held).collect();
                own.iter_mut().for_each(|x| *x = 0.0);
                share.iter_mut().for_each(|x| *x = 0.0);
                year = p.year;
            } else if start.is_empty() && !w.reserves.is_empty() {
                start = (0..nm)
                    .map(|mi| {
                        let r = &w.reserves;
                        (
                            r.markets.get(mi).map_or(0.0, |x| f(x.total())),
                            f(r.owed(mi)),
                        )
                    })
                    .collect();
            }
            // This week's margins, split: each market's own balance and its share of the
            // world's.
            let (balance, _) = bf_sim::econ::week::trade_balances(&cat, &w.econ.markets);
            let t = bf_sim::econ::week::margin_transfers(&cat, &w.econ.markets);
            for mi in 0..nm {
                own[mi] += f(balance[mi]);
                share[mi] += f(balance[mi] - t[mi]);
            }
        }
        return Ok(());
    }
    // Customs revenue by country (`BF_DIAG_DUTIES=TAG,TAG[:years]`), with `customs_duties` on:
    // by year, the tariff in force (against the 1925 rate of `diplomacy.ron`), the duties over
    // GDP and the imports from every partner over GDP (gross trade included, at base prices).
    if let Ok(spec) = std::env::var("BF_DIAG_DUTIES") {
        let (tags, years) = spec.split_once(':').unwrap_or((spec.as_str(), "11"));
        let years: i32 = years.parse().unwrap_or(11);
        let duties = data.duties();
        let idx: Vec<(String, usize)> = tags
            .split(',')
            .filter_map(|t| {
                let t = t.trim();
                data.countries
                    .countries
                    .iter()
                    .position(|c| c.tag == t)
                    .map(|i| (t.to_string(), i))
            })
            .collect();
        let end = Date::new(1925 + years, 1, 1);
        let mut d = Date::new(1925, 1, 5);
        let mut year = 1925;
        while d < end {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            if d.year == year {
                continue;
            }
            // The year just ended: each country's imports from every partner (gross trade
            // included, at base prices) and the tariff in force at its end.
            let e = &s.world().econ;
            let n = e.markets.len();
            let imports = bf_sim::econ::customs::imports_by_buyer(&e.trade.last_year, n);
            for (tag, i) in &idx {
                let m = &e.markets[*i];
                let rate = duties.rate(*i, m);
                let gdp = m.gdp_last_year.to_f64_lossy().max(1.0);
                let bought = imports.get(*i).map_or(0.0, |v| v.to_f64_lossy());
                let r1925 = data
                    .diplomacy
                    .as_ref()
                    .and_then(|dp| dp.tariff(tag))
                    .map_or(f32::NAN, |t| t.rate);
                println!(
                    "{year} {tag}: tariff {:.1} % (1925 ref {:.1} %), duties {:.2} % of GDP; imports {:.2} % of GDP",
                    rate.to_f64_lossy() * 100.0,
                    f64::from(r1925) * 100.0,
                    rate.to_f64_lossy() * bought / gdp * 100.0,
                    bought / gdp * 100.0
                );
            }
            year = d.year;
        }
        return Ok(());
    }
    // What a tariff protects (`BF_DIAG_PROTECT=TAG[:years]`, docs/trade.pt-BR.md, section 13):
    // run with `BF_LAWS=TAG=free_trade` and then `protective_tariffs`, with and without
    // `customs_duties`. Printed by year, for the goods the country imported most in the first
    // weeks and for all goods: imports, domestic output and the price over the base price, as
    // weekly means.
    if let Ok(spec) = std::env::var("BF_DIAG_PROTECT") {
        let (tag, years) = spec.split_once(':').unwrap_or((spec.as_str(), "10"));
        let years: i32 = years.parse().unwrap_or(10);
        let mi = data
            .countries
            .countries
            .iter()
            .position(|c| c.tag == tag)
            .ok_or_else(|| format!("BF_DIAG_PROTECT: no country {tag}"))?;
        let ng = cat.goods.len();
        let end = Date::new(1925 + years, 1, 1);
        let mut d = Date::new(1925, 1, 5);
        let mut top: Vec<usize> = Vec::new();
        let (mut imp, mut out, mut price) = (vec![0f64; ng], vec![0f64; ng], vec![0f64; ng]);
        let (mut year, mut weeks) = (1925, 0u32);
        let rate =
            (data.switches.customs_duties && data.switches.trade_by_pair).then(|| data.duties());
        while d < end {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            let m = &s.world().econ.markets[mi];
            for gi in 0..ng {
                let n = m.net_import[gi].max(laplace_schelling::Fixed::ZERO);
                let base = cat.goods[gi].base_price;
                imp[gi] += (n * base).to_f64_lossy();
                out[gi] += ((m.supply[gi] - n) * base).to_f64_lossy();
                price[gi] += (m.price[gi] / base).to_f64_lossy();
            }
            weeks += 1;
            if d.year != year {
                let w = f64::from(weeks);
                if top.is_empty() {
                    let mut o: Vec<usize> = (0..ng).filter(|&g| imp[g] > 0.0).collect();
                    o.sort_by(|a, b| imp[*b].total_cmp(&imp[*a]));
                    o.truncate(6);
                    top = o;
                }
                let t = rate.as_ref().map_or(0.0, |r| r.rate(mi, m).to_f64_lossy());
                println!(
                    "{year} {tag} tariff {:.1} %: imports {:.4e}, output {:.4e} (all goods, base prices, a week)",
                    t * 100.0,
                    imp.iter().sum::<f64>() / w,
                    out.iter().sum::<f64>() / w
                );
                for &g in &top {
                    println!(
                        "    {:<16} imports {:.4e}  output {:.4e}  price/base {:.3}",
                        cat.goods[g].key,
                        imp[g] / w,
                        out[g] / w,
                        price[g] / w
                    );
                }
                (imp, out, price) = (vec![0f64; ng], vec![0f64; ng], vec![0f64; ng]);
                (year, weeks) = (d.year, 0);
            }
        }
        return Ok(());
    }
    // Demand left unmet (`BF_DIAG_MET=<years>`): by year, the world's tradable demand that found
    // no seller (demand times one less the share bought, at base prices) over all tradable
    // demand, the same for the goods a country imports on net, and the world's output of tradable
    // goods; to see whether a cut in imports moves demand to the producer at home or leaves it
    // unmet.
    if let Ok(years) = std::env::var("BF_DIAG_MET") {
        let years: i32 = years.parse().unwrap_or(5);
        let end = Date::new(1925 + years, 1, 1);
        let mut d = Date::new(1925, 1, 5);
        let (mut year, mut weeks) = (1925, 0u32);
        let (mut want, mut short, mut want_imp, mut short_imp, mut made) =
            (0f64, 0f64, 0f64, 0f64, 0f64);
        while d < end {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            let e = &s.world().econ;
            for m in &e.markets {
                for (gi, good) in cat.goods.iter().enumerate().filter(|(_, g)| g.tradable) {
                    let b = good.base_price;
                    let w = (m.demand[gi] * b).to_f64_lossy().max(0.0);
                    let met = m.buy_ratio[gi].to_f64_lossy().clamp(0.0, 1.0);
                    want += w;
                    short += w * (1.0 - met);
                    if m.net_import[gi] > laplace_schelling::Fixed::ZERO {
                        want_imp += w;
                        short_imp += w * (1.0 - met);
                    }
                    made += (m.supply[gi] * b).to_f64_lossy().max(0.0);
                }
            }
            weeks += 1;
            if d.year != year {
                println!(
                    "{year} world: unmet {:.2} % of tradable demand, {:.2} % where imported on net; tradable output {:.4e} a week",
                    short / want.max(1.0) * 100.0,
                    short_imp / want_imp.max(1.0) * 100.0,
                    made / f64::from(weeks)
                );
                (want, short, want_imp, short_imp, made) = (0.0, 0.0, 0.0, 0.0, 0.0);
                (year, weeks) = (d.year, 0);
            }
        }
        return Ok(());
    }
    // Conservation of trade, week by week (`BF_DIAG_CONSERVE=<years>`): for every tradable
    // good the world's net imports add up to zero, and the margins' transfers between the
    // pools (`week::margin_transfers`) add up to zero, so trade makes no money. Printed by
    // year: the largest residual of goods (at base prices) in a week, the money the transfers
    // would make in the year over GDP, trade over GDP and the margin over trade.
    if let Ok(years) = std::env::var("BF_DIAG_CONSERVE") {
        use laplace_schelling::Fixed;
        let years: i32 = years.parse().unwrap_or(5);
        let end = Date::new(1925 + years, 1, 1);
        let mut d = Date::new(1925, 1, 5);
        let (mut year, mut goods, mut made, mut volume, mut margin, mut weeks) =
            (1925, 0f64, 0f64, 0f64, 0f64, 0u32);
        // With customs duties on: what the treasuries collected over the value of the imports
        // from every partner, the world's average tariff as Clemens and Williamson measure it.
        let customs =
            (data.switches.customs_duties && data.switches.trade_by_pair).then(|| data.duties());
        let (mut collected, mut imported) = (0f64, 0f64);
        // The pairs' running totals of the year (`trade.year`), for the week's imports.
        let mut before: Vec<Fixed> = Vec::new();
        while d < end {
            d = Date::from_days(d.to_days() + 7);
            s.sim.run_until(d);
            let e = &s.world().econ;
            for gi in 0..cat.goods.len() {
                let base = cat.goods[gi].base_price;
                let mut net = Fixed::ZERO;
                for m in &e.markets {
                    let n = m.net_import[gi];
                    net += n;
                    if n > Fixed::ZERO {
                        volume += (n * base).to_f64_lossy();
                        margin += (n * m.sell_ratio[gi] * m.price[gi]).to_f64_lossy();
                    } else if n < Fixed::ZERO {
                        margin += (n * m.buy_ratio[gi] * m.price[gi]).to_f64_lossy();
                    }
                }
                goods = goods.max((net * base).abs().to_f64_lossy());
            }
            if let Some(dt) = &customs {
                let rates: Vec<Fixed> = e
                    .markets
                    .iter()
                    .enumerate()
                    .map(|(mi, m)| dt.rate(mi, m))
                    .collect();
                // The week's pair values: the year's totals less last week's (from zero when
                // the year turned over).
                let now = &e.trade.year;
                let turned =
                    before.len() != now.len() || now.iter().zip(&before).any(|(a, b)| a < b);
                let week: Vec<Fixed> = if turned {
                    now.clone()
                } else {
                    now.iter().zip(&before).map(|(a, b)| *a - *b).collect()
                };
                before = now.clone();
                let n = e.markets.len();
                let imports = bf_sim::econ::customs::imports_by_buyer(&week, n);
                let owed = |r: &[Fixed]| -> f64 {
                    bf_sim::econ::customs::duties(&cat, &e.markets, r, Some(&imports))
                        .iter()
                        .map(|t| t.to_f64_lossy())
                        .sum()
                };
                collected += owed(&rates);
                imported += owed(&vec![Fixed::ONE; rates.len()]);
            }
            made += bf_sim::econ::week::margin_transfers(&cat, &e.markets)
                .iter()
                .map(|t| t.to_f64_lossy())
                .sum::<f64>();
            weeks += 1;
            if d.year != year {
                let gdp: f64 = e
                    .markets
                    .iter()
                    .map(|m| m.gdp_last_year.to_f64_lossy())
                    .sum();
                println!(
                    "{year}: goods residual {goods:.3}, money made by the margins {:.4} % of GDP, trade {:.2} % of GDP, margin {:.1} % of trade",
                    made / gdp * 100.0,
                    volume * 52.0 / f64::from(weeks) / gdp * 100.0,
                    margin / volume.max(1.0) * 100.0
                );
                if customs.is_some() {
                    println!(
                        "{year}: duties {:.1} % of imports, {:.2} % of GDP",
                        collected / imported.max(1.0) * 100.0,
                        collected * 52.0 / f64::from(weeks) / gdp * 100.0
                    );
                }
                (year, goods, made, volume, margin, weeks) = (d.year, 0.0, 0.0, 0.0, 0.0, 0);
                (collected, imported) = (0.0, 0.0);
            }
        }
        return Ok(());
    }
    // Stocks by market (`BF_DIAG_STOCK=<years>`, `econ::stock`, with `market_stock` on): each
    // January, every storable good's stock in the world, in weeks of the world's output, and the
    // markets that hold most of it.
    if let Ok(years) = std::env::var("BF_DIAG_STOCK") {
        let years: i32 = years.parse().unwrap_or(16);
        for y in 1926..=1925 + years {
            s.sim.run_until(Date::new(y, 1, 2));
            let w = s.world();
            for (g, good) in cat
                .goods
                .iter()
                .enumerate()
                .filter(|(_, g)| g.stock.is_some())
            {
                let mut held: Vec<(usize, f64)> = (0..w.econ.markets.len())
                    .map(|mi| (mi, w.stocks.of(mi, g).to_f64_lossy()))
                    .filter(|x| x.1 > 0.0)
                    .collect();
                held.sort_by(|a, b| b.1.total_cmp(&a.1));
                let total: f64 = held.iter().map(|x| x.1).sum();
                let made: f64 = w
                    .econ
                    .markets
                    .iter()
                    .map(|m| m.supply[g].to_f64_lossy())
                    .sum();
                let top: Vec<String> = held
                    .iter()
                    .take(4)
                    .map(|(mi, q)| format!("{} {:.0}", data.countries.countries[*mi].tag, q))
                    .collect();
                println!(
                    "{y} {}: stock {total:.0} ({:.1} weeks of supply), price/base {:.2} [{}]",
                    good.key,
                    total / made.max(1e-9),
                    w.econ
                        .markets
                        .iter()
                        .map(|m| (m.price[g] / good.base_price).to_f64_lossy())
                        .fold(0.0, f64::max),
                    top.join(", ")
                );
            }
        }
        return Ok(());
    }
    // Railways against the scenario's network (`BF_DIAG_RAIL=TAG,TAG`): at the start, each
    // country's railway levels, its km of railway (`industry.ron`), levels per 1,000 km, the coal
    // the railways burn a year at full use, and the transport they make against the country's
    // demand for it.
    if let Ok(spec) = std::env::var("BF_DIAG_RAIL") {
        let w = s.world();
        let e = &w.econ;
        let rail = cat.building("railway").ok_or("no railway")?;
        let coal = cat.good("coal").ok_or("no coal")? as usize;
        let tr = cat.good("transportation").ok_or("no transportation")? as usize;
        let km = |tag: &str| {
            data.industry.as_ref().and_then(|f| {
                f.countries.iter().find(|c| c.country == tag).and_then(|c| {
                    c.output
                        .iter()
                        .find(|(k, _)| k == "railways_km")
                        .map(|(_, v)| *v as f64)
                })
            })
        };
        let all: Vec<String> = data
            .countries
            .countries
            .iter()
            .map(|c| c.tag.clone())
            .collect();
        let tags: Vec<String> = if spec == "all" {
            all.clone()
        } else {
            spec.split(',').map(str::to_string).collect()
        };
        let trad = cat.building("traditional_transport");
        println!(
            "tag,levels,km,levels_per_1000km,coal_Mt_year,trad_levels,trad_workers_M,workers_M"
        );
        for tag in tags {
            let Some(mi) = all.iter().position(|t| *t == tag) else {
                continue;
            };
            let levels: i64 = e
                .regions
                .iter()
                .filter(|r| r.market as usize == mi)
                .flat_map(|r| r.plants.iter())
                .filter(|p| p.kind as usize == rail)
                .map(|p| p.levels)
                .sum();
            let meth = &cat.buildings[rail].methods[0];
            let per_coal = meth
                .inputs
                .iter()
                .find(|(g, _)| *g as usize == coal)
                .map_or(0.0, |(_, q)| q.to_f64_lossy());
            let per_tr = meth
                .outputs
                .iter()
                .find(|(g, _)| *g as usize == tr)
                .map_or(1.0, |(_, q)| q.to_f64_lossy());
            let k = km(&tag).unwrap_or(0.0);
            let _ = per_tr;
            let tl: i64 = e
                .regions
                .iter()
                .filter(|r| r.market as usize == mi)
                .flat_map(|r| r.plants.iter())
                .filter(|p| Some(p.kind as usize) == trad)
                .map(|p| p.levels)
                .sum();
            let tw = trad.map_or(0, |t| cat.buildings[t].methods[0].workers) * tl;
            let workers: i64 = e
                .regions
                .iter()
                .filter(|r| r.market as usize == mi)
                .map(|r| {
                    r.pops
                        .iter()
                        .map(|p| bf_sim::econ::workers(p.size))
                        .sum::<i64>()
                })
                .sum();
            println!(
                "{tag},{levels},{k:.0},{:.1},{:.1},{tl},{:.2},{:.1}",
                levels as f64 / k.max(1.0) * 1000.0,
                levels as f64 * per_coal * 52.0 / 1e6,
                tw as f64 / 1e6,
                workers as f64 / 1e6
            );
        }
        return Ok(());
    }
    // The state's offices with computers (`BF_DIAG_ADMIN=TAG,TAG`, `econ::admin`, with
    // `computer_efficiency` and `fiscal_capacity`): every ten years, a servant's worth, the
    // computers ordered and found, and the taxes not collected over those collected.
    if let Ok(spec) = std::env::var("BF_DIAG_ADMIN") {
        let all: Vec<String> = data
            .countries
            .countries
            .iter()
            .map(|c| c.tag.clone())
            .collect();
        let idx: Vec<(String, usize)> = spec
            .split(',')
            .filter_map(|t| all.iter().position(|x| x == t).map(|i| (t.to_string(), i)))
            .collect();
        for y in (1935..=2025).step_by(10) {
            s.sim.run_until(Date::new(y, 7, 2));
            let e = &s.world().econ;
            for (tag, mi) in &idx {
                let m = &e.markets[*mi];
                let (want, got) = m.admin_computers.unwrap_or_default();
                let lost = m.uncollected.unwrap_or_default().to_f64_lossy();
                let taken = m.budget.revenue.to_f64_lossy();
                println!(
                    "{y} {tag}: servant worth {:.2}, computers {:.1} ordered {:.1} found, uncollected {:.1} % of taxes",
                    m.admin_efficiency.map_or(1.0, |x| x.to_f64_lossy()),
                    want.to_f64_lossy(),
                    got.to_f64_lossy(),
                    lost / (lost + taken).max(1e-9) * 100.0
                );
            }
        }
        return Ok(());
    }
    // The first year each method of a building runs in each country
    // (`BF_DIAG_ADOPTION=laboratory:USA,SOV,BRA,RAJ`): checked each January to 2025.
    if let Ok(spec) = std::env::var("BF_DIAG_ADOPTION") {
        let (kind, tags) = spec
            .split_once(':')
            .ok_or("BF_DIAG_ADOPTION=building:TAG,TAG")?;
        let k = cat.building(kind).ok_or("unknown building")?;
        let all: Vec<String> = data
            .countries
            .countries
            .iter()
            .map(|c| c.tag.clone())
            .collect();
        let idx: Vec<(String, usize)> = tags
            .split(',')
            .filter_map(|t| all.iter().position(|x| x == t).map(|i| (t.to_string(), i)))
            .collect();
        let nm = cat.buildings[k].methods.len();
        let mut first = vec![vec![0i32; nm]; idx.len()];
        for y in 1926..=2025 {
            s.sim.run_until(Date::new(y, 1, 2));
            let e = &s.world().econ;
            for (j, (_, mi)) in idx.iter().enumerate() {
                for r in e.regions.iter().filter(|r| r.market as usize == *mi) {
                    for p in r
                        .plants
                        .iter()
                        .filter(|p| p.kind as usize == k && p.levels > 0)
                    {
                        let f = &mut first[j][p.method as usize];
                        if *f == 0 {
                            *f = y;
                        }
                    }
                }
            }
        }
        for (j, (tag, _)) in idx.iter().enumerate() {
            let parts: Vec<String> = cat.buildings[k]
                .methods
                .iter()
                .zip(&first[j])
                .map(|(m, y)| {
                    format!(
                        "{} {}",
                        m.key,
                        if *y > 0 { y.to_string() } else { "-".into() }
                    )
                })
                .collect();
            println!("{tag} {kind}: {}", parts.join(", "));
        }
        return Ok(());
    }
    // Levels by method of some buildings, every ten years (`BF_DIAG_METHODS=laboratory,university`):
    // whether a new method is taken up, and how fast.
    if let Ok(spec) = std::env::var("BF_DIAG_METHODS") {
        let kinds: Vec<usize> = spec.split(',').filter_map(|k| cat.building(k)).collect();
        for y in (1935..=2025).step_by(10) {
            s.sim.run_until(Date::new(y, 1, 2));
            let e = &s.world().econ;
            for &k in &kinds {
                let b = &cat.buildings[k];
                let mut by = vec![0i64; b.methods.len()];
                for r in &e.regions {
                    for p in r.plants.iter().filter(|p| p.kind as usize == k) {
                        by[p.method as usize] += p.levels;
                    }
                }
                let parts: Vec<String> = b
                    .methods
                    .iter()
                    .zip(&by)
                    .map(|(m, n)| format!("{} {n}", m.key))
                    .collect();
                println!("{y} {}: {}", b.key, parts.join(", "));
            }
        }
        return Ok(());
    }
    // World trade over world GDP every ten years (data/history/trade_openness.csv): exports
    // plus imports at base prices, a week in July times 52, over the GDP of the last year.
    if std::env::var_os("BF_DIAG_TRADE").is_some() {
        let mut y = 1925;
        while y <= 2025 {
            s.sim.run_until(Date::new(y, 7, 2));
            let e = &s.world().econ;
            let mut traded = 0.0;
            let mut gdp = 0.0;
            for m in &e.markets {
                for (g, n) in m.net_import.iter().enumerate() {
                    traded += n.to_f64_lossy().abs() * cat.goods[g].base_price.to_f64_lossy();
                }
                gdp += m.gdp_last_year.to_f64_lossy();
            }
            println!(
                "{y} trade: {:.1} % of world GDP",
                traded * 52.0 / gdp.max(1.0) * 100.0
            );
            y += 10;
        }
        return Ok(());
    }
    // Power plants by method every ten years, as a share of the electricity they can make.
    if std::env::var_os("BF_DIAG_POWER").is_some()
        && let Some(k) = cat.building("power_plant")
    {
        let methods = &cat.buildings[k].methods;
        let mut y = 1925;
        while y <= 2025 {
            s.sim.run_until(Date::new(y, 7, 2));
            let mut out = vec![0.0f64; methods.len()];
            for r in &s.world().econ.regions {
                for p in r.plants.iter().filter(|p| p.kind as usize == k) {
                    let m = &methods[p.method as usize];
                    out[p.method as usize] +=
                        p.levels as f64 * m.outputs.first().map_or(0.0, |(_, q)| q.to_f64_lossy());
                }
            }
            let total: f64 = out.iter().sum::<f64>().max(1.0);
            let parts: Vec<String> = methods
                .iter()
                .zip(&out)
                .map(|(m, o)| format!("{} {:.0}%", m.key, o / total * 100.0))
                .collect();
            println!("{y} power: {}", parts.join(", "));
            y += 10;
        }
        return Ok(());
    }
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
        // By country (`BF_DIAG_ENERGY_TAGS=USA,FRA,...`): who makes and who uses steel and gas.
        let tags_of: Vec<String> = s
            .world()
            .countries
            .iter()
            .map(|(_, c)| c.tag.clone())
            .collect();
        for tag in std::env::var("BF_DIAG_ENERGY_TAGS")
            .unwrap_or_default()
            .split(',')
        {
            let Some(mi) = tags_of.iter().position(|t| t == tag) else {
                continue;
            };
            let m = &e.markets[mi];
            let gdp = m.gdp_last_year.to_f64_lossy();
            // The goods to break down (`BF_DIAG_ENERGY_GOODS=coal,timber`, by default steel, gas,
            // cement and machinery).
            let goods = std::env::var("BF_DIAG_ENERGY_GOODS")
                .unwrap_or_else(|_| "steel,natural_gas,cement,machinery".to_string());
            for key in goods.split(',') {
                let Some(g) = cat.good(key) else { continue };
                let g = g as usize;
                let dem = m.demand[g].to_f64_lossy() * 52.0;
                let sup = m.supply[g].to_f64_lossy() * 52.0;
                let mut by = vec![0.0f64; cat.buildings.len()];
                for r in e.regions.iter().filter(|r| r.market as usize == mi) {
                    for pl in &r.plants {
                        let b = &cat.buildings[pl.kind as usize];
                        let meth = &b.methods[pl.method as usize];
                        let sc = pl.levels as f64
                            * pl.staffing.to_f64_lossy()
                            * plant_tfp(r, m, b).to_f64_lossy()
                            * pl.utilization.to_f64_lossy()
                            * 52.0;
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
                    .take(6)
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
                    "{year} {tag} {key}: made {:.3e} used {:.3e} ({:.1} per $1,000 of GDP), net import {:.0}%; plants {:.0}% [{}], rest {:.0}%",
                    sup,
                    dem,
                    dem / gdp.max(1.0) * 1000.0,
                    m.net_import[g].to_f64_lossy() * 52.0 / dem.max(1.0) * 100.0,
                    plants / dem.max(1.0) * 100.0,
                    tops.join(", "),
                    (dem - plants) / dem.max(1.0) * 100.0
                );
            }
            println!(
                "{year} {tag}: GDP {:.0} bn, pool {:.1}% of GDP, queue {}",
                gdp / 1e9,
                m.pool.to_f64_lossy() / gdp.max(1.0) * 100.0,
                m.queue.len()
            );
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
    // Small countries: plants and pay of each profession, at a few dates.
    if let Ok(list) = std::env::var("BF_DIAG_SMALL") {
        let mis: Vec<usize> = list
            .split(',')
            .filter_map(|t| tags.iter().position(|x| x == t))
            .collect();
        for y in [1925, 1926, 1927, 1930] {
            s.sim.run_until(Date::new(y, 2, 1));
            let e = &s.world().econ;
            let wages = bf_sim::econ::monthly::market_wages(e, false);
            for &mi in &mis {
                let m = &e.markets[mi];
                println!(
                    "{y} {}: gdp {:.0}, wage {:.2}, pool {:.0}",
                    tags[mi],
                    m.gdp_last_year.to_f64_lossy(),
                    wages[mi].to_f64_lossy(),
                    m.pool.to_f64_lossy()
                );
                for r in e.regions.iter().filter(|r| r.market as usize == mi) {
                    for (pi, p) in r.pops.iter().enumerate() {
                        if p.size > 0 {
                            println!(
                                "   pop {pi}: size {} employed {} income {:.1}",
                                p.size,
                                p.employed,
                                p.income.to_f64_lossy()
                            );
                        }
                    }
                    for pl in &r.plants {
                        let b = &cat.buildings[pl.kind as usize];
                        println!(
                            "   plant {}: levels {} method {} staffing {:.2} util {:.2} input {:.2} workers/level {}",
                            b.key,
                            pl.levels,
                            pl.method,
                            pl.staffing.to_f64_lossy(),
                            pl.utilization.to_f64_lossy(),
                            pl.input_ratio.to_f64_lossy(),
                            b.methods[pl.method as usize].workers
                        );
                    }
                }
            }
        }
        return Ok(());
    }
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
                bf_sim::econ::monthly::market_wages(e, false)[mi].to_f64_lossy()
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
