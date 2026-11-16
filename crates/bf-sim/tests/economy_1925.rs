//! The 1925 economy starts at the scenario's size: the plants the setup lays add the value the
//! scenario's GDP asks for, and the money people, the pool and the treasury start with spends
//! about what the plants make (docs/economy.pt-BR.md, "1925").

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::plant_tfp;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn plants_and_incomes_match_the_scenario() {
    let data = GameData::load(&root()).unwrap();
    let cat = &data.catalogue;
    let world = data.new_world();
    let e = &world.econ;
    let target: f64 = e
        .markets
        .iter()
        .map(|m| m.gdp_last_year.to_f64_lossy())
        .sum();
    let mut value_added = 0.0;
    for r in &e.regions {
        let m = &e.markets[r.market as usize];
        for p in &r.plants {
            let b = &cat.buildings[p.kind as usize];
            let meth = &b.methods[p.method as usize];
            let va = meth.value_added(|g| cat.goods[g].base_price).to_f64_lossy();
            value_added += va * 52.0 * p.levels as f64 * plant_tfp(r, m, b).to_f64_lossy();
        }
    }
    // Plants make the GDP less the state's payroll and the peasants' own food (a few percent).
    let share = value_added / target;
    assert!(
        (0.88..=1.02).contains(&share),
        "plants' value added is {:.0}% of the scenario's GDP",
        share * 100.0
    );
    // People's after-tax income is most of it: wages and the owners' payout.
    let income: f64 = e
        .regions
        .iter()
        .flat_map(|r| r.pops.iter())
        .map(|p| p.income.to_f64_lossy() * 52.0)
        .sum();
    let share = income / target;
    assert!(
        (0.65..=0.90).contains(&share),
        "households start with {:.0}% of the scenario's GDP",
        share * 100.0
    );
}
