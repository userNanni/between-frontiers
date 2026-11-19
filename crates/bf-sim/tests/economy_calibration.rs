//! Diagnostic of the catalogue at base prices: value added per worker of every method. A method
//! that destroys value at base prices would close everywhere; one that makes absurd value would
//! swallow the economy. Run with `--nocapture` to see the table.

use bf_sim::economy::{BuildingsFile, GoodsFile};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn load<T: serde::de::DeserializeOwned>(p: &str) -> T {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    ron::from_str(&std::fs::read_to_string(root.join(p)).unwrap()).unwrap()
}

#[test]
fn value_added_per_worker_is_plausible() {
    let goods: GoodsFile = load("data/economy/goods.ron");
    let buildings: BuildingsFile = load("data/economy/buildings.ron");
    let price: BTreeMap<&str, f64> = goods
        .goods
        .iter()
        .map(|g| (g.key.as_str(), f64::from(g.base_price)))
        .collect();
    let mut bad = Vec::new();
    for b in &buildings.buildings {
        for m in &b.methods {
            let out: f64 = m
                .outputs
                .iter()
                .map(|(g, q)| price[g.as_str()] * f64::from(*q))
                .sum();
            let inp: f64 = m
                .inputs
                .iter()
                .map(|(g, q)| price[g.as_str()] * f64::from(*q))
                .sum();
            let workers: u32 = m.jobs.iter().map(|(_, n)| n).sum();
            let va = (out - inp) / f64::from(workers.max(1));
            println!(
                "{:<26} {:<28} VA/worker {:>9.0}  margin {:>5.2}",
                b.key,
                m.key,
                va,
                (out - inp) / out.max(1.0)
            );
            // Utilities and offshore fields are capital-heavy: few workers, much value.
            let ceiling = if matches!(b.group.as_str(), "energy" | "extraction") {
                250_000.0
            } else {
                120_000.0
            };
            // The weapon core plant (U2) sells one good by two methods of very different cost: the
            // core's price is set above the dearer one (the high enrichment of 1945), and the
            // production reactor earns a rent no wage explains (plutonium was the cheaper route).
            // It is the state's, bought by the programme, not a market plant.
            if b.key == "weapon_core_plant" {
                continue;
            }
            if !(1_000.0..=ceiling).contains(&va) {
                bad.push(format!("{}/{}: {va:.0}", b.key, m.key));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "value added per worker outside 1k–120k (250k for energy and extraction; 2011 $):\n{}",
        bad.join("\n")
    );
}
