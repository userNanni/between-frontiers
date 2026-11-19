//! The blockade's check (docs/trade.pt-BR.md, section 14): Germany in 1914–18 was a net importer
//! of food and fertilizer. With trade by pair, closing every pair whose cheaper way to Germany
//! is by sea should leave it short of what it imported: less food and fertilizer met, dearer.
//! Measured with only the balances trading (as shipped) and with gross trade
//! (`trade_gross`), over two years from 1925. `cargo test --release --test blockade --
//! --nocapture` prints the table.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use bf_sim::data::GameData;
use bf_sim::game::{Session, Switches};
use laplace_schelling::Date;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

const GOODS: [&str; 9] = [
    "grain",
    "meat",
    "dairy",
    "fruit",
    "sugar",
    "processed_food",
    "fertilizer",
    "phosphate_rock",
    "potash",
];

/// By good of `GOODS`: share of Germany's demand met, price over base and net imports at base
/// prices, means over the last 13 weeks of two years.
fn run(gross: bool, blockade: bool) -> Vec<(f64, f64, f64)> {
    let d = data();
    let ger = d
        .countries
        .countries
        .iter()
        .position(|c| c.tag == "GER")
        .unwrap();
    let mut rules = d.rules_with(Switches {
        trade_gross: gross,
        ..Switches::shipped()
    });
    if blockade {
        let mut geo = (**rules.trade.as_ref().unwrap()).clone();
        for s in 0..d.countries.countries.len() {
            if s != ger && geo.by_sea(s, ger) {
                geo.close(s, ger, "1925-01-01", None);
            }
        }
        rules.trade = Some(Arc::new(geo));
    }
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    let cat = &d.catalogue;
    let idx: Vec<usize> = GOODS
        .iter()
        .map(|k| usize::from(cat.good(k).unwrap()))
        .collect();
    let mut out = vec![(0.0, 0.0, 0.0); GOODS.len()];
    let mut date = Date::new(1926, 10, 4);
    s.sim.run_until(date);
    for _ in 0..13 {
        date = Date::from_days(date.to_days() + 7);
        s.sim.run_until(date);
        let m = &s.world().econ.markets[ger];
        for (o, &g) in out.iter_mut().zip(&idx) {
            let base = cat.goods[g].base_price;
            o.0 += m.buy_ratio[g].to_f64_lossy() / 13.0;
            o.1 += (m.price[g] / base).to_f64_lossy() / 13.0;
            o.2 += (m.net_import[g] * base).to_f64_lossy() / 13.0;
        }
    }
    out
}

#[test]
fn a_blockade_by_sea_leaves_germany_short_of_what_it_imported() {
    for gross in [false, true] {
        let open = run(gross, false);
        let shut = run(gross, true);
        println!(
            "{}: good, met open → closed, price/base open → closed, net imports open → closed",
            if gross {
                "gross trade"
            } else {
                "balances only"
            }
        );
        for (k, (o, c)) in GOODS.iter().zip(open.iter().zip(&shut)) {
            println!(
                "  {k:<15} {:.3} → {:.3}   {:.3} → {:.3}   {:.3e} → {:.3e}",
                o.0, c.0, o.1, c.1, o.2, c.2
            );
        }
        // What it imported on net, open, against what it gets of those goods, closed.
        let imported: f64 = open.iter().map(|o| o.2.max(0.0)).sum();
        let left: f64 = open
            .iter()
            .zip(&shut)
            .filter(|(o, _)| o.2 > 0.0)
            .map(|(_, c)| c.2.max(0.0))
            .sum();
        assert!(
            left <= imported,
            "the blockade leaves Germany importing more ({left:.3e} against {imported:.3e})"
        );
    }
}
