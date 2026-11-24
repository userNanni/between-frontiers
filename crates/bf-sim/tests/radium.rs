//! U4 (the owner, 2026-11-19): radium works the uranium mines before the bomb, and the uranium
//! they make as a by-product goes to the market's stock (`market_stock`). The conference: the
//! world's radium a year against the sources, and the stock of uranium ore by 1940 against what
//! the Manhattan District found (H&A, The New World, p. 26: 1,200 t of ore at 65 % on Staten
//! Island; Jones, Manhattan, p. 79: some 2,200 t at about 20 % above ground in the Congo; about
//! 1,220 t of U3O8 together, my computation).

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameWorld, Session};
use laplace_schelling::Date;

#[test]
#[ignore = "long: 1925 to 1940; run in the slot"]
fn radium_and_the_uranium_stock_to_1940() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load_with(&root, "radium").unwrap_or_else(|e| panic!("{e}"));
    d.switches.market_stock = true;
    let cat = &d.catalogue;
    let radium = usize::from(cat.good("radium").unwrap());
    let ore = usize::from(cat.good("uranium_ore").unwrap());
    let mine = cat.building("uranium_mine").unwrap();
    let hospital = cat.building("hospital").unwrap();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    let tag = |w: &GameWorld, m: usize| {
        w.countries
            .iter()
            .nth(m)
            .map(|(_, c)| c.tag.clone())
            .unwrap_or_default()
    };
    eprintln!(
        "RAD year | radium made (g, at the plants' productivity) | radium the hospitals ask for (g) | hospital levels, and times productivity | mine levels by market | uranium ore stock by market (t)"
    );
    for year in 1926..=1941 {
        s.sim.run_until(Date::new(year, 1, 2));
        let w = s.world();
        let mut made = 0.0;
        let (mut used, mut h_levels, mut h_tfp) = (0.0, 0i64, 0.0);
        let mut levels: Vec<(String, i64)> = Vec::new();
        for r in &w.econ.regions {
            let m = &w.econ.markets[usize::from(r.market)];
            for p in r
                .plants
                .iter()
                .filter(|p| usize::from(p.kind) == hospital && p.levels > 0)
            {
                let b = &cat.buildings[hospital];
                let tfp = bf_sim::econ::plant_tfp(r, m, b).to_f64_lossy();
                let run = p.levels as f64
                    * p.staffing.to_f64_lossy()
                    * tfp
                    * p.utilization.to_f64_lossy();
                if let Some((_, q)) = b.methods[p.method as usize]
                    .inputs
                    .iter()
                    .find(|(g, _)| usize::from(*g) == radium)
                {
                    used += q.to_f64_lossy() * 52.0 * run;
                }
                h_levels += p.levels;
                h_tfp += p.levels as f64 * tfp;
            }
            for p in r
                .plants
                .iter()
                .filter(|p| usize::from(p.kind) == mine && p.levels > 0)
            {
                let m = &cat.buildings[mine].methods[p.method as usize];
                if let Some((_, q)) = m.outputs.iter().find(|(g, _)| usize::from(*g) == radium) {
                    let tfp = bf_sim::econ::plant_tfp(
                        r,
                        &w.econ.markets[usize::from(r.market)],
                        &cat.buildings[mine],
                    )
                    .to_f64_lossy();
                    made += q.to_f64_lossy()
                        * 52.0
                        * p.levels as f64
                        * p.staffing.to_f64_lossy()
                        * tfp
                        * p.utilization.to_f64_lossy()
                        * p.input_ratio.to_f64_lossy();
                }
                let t = tag(w, usize::from(r.market));
                match levels.iter_mut().find(|(x, _)| *x == t) {
                    Some(e) => e.1 += p.levels,
                    None => levels.push((t, p.levels)),
                }
            }
        }
        let stocks: Vec<(String, f64)> = (0..w.econ.markets.len())
            .map(|m| (tag(w, m), w.stocks.of(m, ore).to_f64_lossy() / 1000.0))
            .filter(|(_, t)| *t > 0.5)
            .collect();
        eprintln!(
            "RAD {} | {:.1} | {:.1} | {h_levels}, {h_tfp:.0} | {:?} | {:?} | radium price BEL {:.0}",
            year - 1,
            made / 1000.0,
            used / 1000.0,
            levels,
            stocks,
            w.econ.markets[w
                .countries
                .iter()
                .position(|(_, c)| c.tag == "BEL")
                .unwrap()]
            .price[radium]
                .to_f64_lossy()
        );
    }
}

/// Where the radium's mines go (the orchestrator's question on Canada's GDP with radium on): by
/// year, with and without radium, the uranium mine levels by market, the radium the hospitals
/// ask for, and in Canada the mines' jobs and value added against its GDP.
#[test]
#[ignore = "long: two runs of a century; run in the slot"]
fn radium_mines_by_country_to_2025() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for rules in ["", "radium"] {
        let d = GameData::load_with(&root, rules).unwrap_or_else(|e| panic!("{e}"));
        let cat = &d.catalogue;
        let mine = cat.building("uranium_mine").unwrap();
        let hospital = cat.building("hospital").unwrap();
        let radium = cat.good("radium").map(usize::from);
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        let tele = d.tree.as_ref().and_then(|t| t.tech("cobalt_teletherapy"));
        for year in [
            1935, 1945, 1955, 1965, 1975, 1985, 1995, 2000, 2005, 2015, 2025,
        ] {
            s.sim.run_until(Date::new(year, 1, 2));
            let w = s.world();
            let can = w
                .countries
                .iter()
                .position(|(_, c)| c.tag == "CAN")
                .unwrap();
            let mut by: Vec<(String, i64)> = Vec::new();
            let (mut jobs, mut va, mut asked) = (0.0, 0.0, 0.0);
            // The share of the radium still used, by the stage of cobalt teletherapy.
            let fade = bf_sim::econ::week::fade_shares(
                cat,
                d.tree.as_deref(),
                &w.research,
                w.econ.markets.len(),
            );
            for r in &w.econ.regions {
                let mi = usize::from(r.market);
                let m = &w.econ.markets[mi];
                for p in r.plants.iter().filter(|p| p.levels > 0) {
                    let b = &cat.buildings[usize::from(p.kind)];
                    let method = &b.methods[p.method as usize];
                    let run = p.levels as f64
                        * p.staffing.to_f64_lossy()
                        * bf_sim::econ::plant_tfp(r, m, b).to_f64_lossy()
                        * p.utilization.to_f64_lossy();
                    if usize::from(p.kind) == hospital
                        && let Some(g) = radium
                        && let Some((_, q)) =
                            method.inputs.iter().find(|(x, _)| usize::from(*x) == g)
                    {
                        asked += q.to_f64_lossy()
                            * method.input_share(g as u16, &fade[mi]).to_f64_lossy()
                            * 52.0
                            * run;
                    }
                    if usize::from(p.kind) != mine {
                        continue;
                    }
                    let tag = w
                        .countries
                        .iter()
                        .nth(mi)
                        .map(|(_, c)| c.tag.clone())
                        .unwrap();
                    match by.iter_mut().find(|(x, _)| *x == tag) {
                        Some(e) => e.1 += p.levels,
                        None => by.push((tag, p.levels)),
                    }
                    if mi == can {
                        jobs += p.levels as f64 * p.staffing.to_f64_lossy() * method.workers as f64;
                        let value = |l: &[(u16, laplace_schelling::Fixed)]| -> f64 {
                            l.iter()
                                .map(|(g, q)| {
                                    q.to_f64_lossy() * m.price[usize::from(*g)].to_f64_lossy()
                                })
                                .sum()
                        };
                        va += (value(&method.outputs) - value(&method.inputs)) * 52.0 * run;
                    }
                }
            }
            by.sort_by_key(|x| std::cmp::Reverse(x.1));
            if let Some(t) = tele {
                let stage = |tag: &str| {
                    w.countries
                        .iter()
                        .position(|(_, c)| c.tag == tag)
                        .map(|i| format!("{tag} {:?}", w.research.stage[i][t]))
                        .unwrap_or_default()
                };
                eprintln!(
                    "TELE {year} | {} | {} | {} | {} | {}",
                    stage("USA"),
                    stage("CAN"),
                    stage("BRA"),
                    stage("RAJ"),
                    stage("CHN")
                );
            }
            let gdp = w.econ.markets[can].gdp_last_year.to_f64_lossy();
            eprintln!(
                "MINES radium {} {year} | radium asked {:.0} g | uranium mine levels {:?} | CAN mines: {:.0} jobs, value added {:.2} bn, {:.2} % of GDP {:.0} bn",
                if rules.is_empty() { "off" } else { "on" },
                asked / 1000.0,
                by.iter().take(8).collect::<Vec<_>>(),
                jobs,
                va / 1e9,
                va * 100.0 / gdp.max(1.0),
                gdp / 1e9
            );
        }
    }
}
