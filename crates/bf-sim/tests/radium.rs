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
