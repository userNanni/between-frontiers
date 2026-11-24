//! The strategic AI's crisis and war in shadow (`shadow`, switch `shadow_ai`, off): it records
//! what it would do and leaves the world as it is. The ignored test runs 101 years and writes the
//! record to `target/shadow.csv` (out of the repository), with the counts by decade, by motive and
//! by variant, against the inter-state wars of the Correlates of War read at the time from
//! `BF_COW_DIR` (`Inter-StateWarData_v4.0.csv`; its terms keep it out of the repository).

use std::collections::BTreeMap;
use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::Session;
use bf_sim::shadow::{ShadowEvent, Step};
use laplace_schelling::Date;
use laplace_schelling::hash::hash_of;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(rules: &str) -> GameData {
    GameData::load_with(&root(), rules).unwrap_or_else(|e| panic!("{e}"))
}

/// With the shadow on, the world is the same, and the record grows.
#[test]
fn the_shadow_leaves_the_world_alone() {
    let off = load("");
    let on = load("shadow_ai");
    let rules = on.rules();
    let log = rules
        .shadow
        .as_ref()
        .expect("shadow.ron with the switch on")
        .log
        .clone();
    let mut a = Session::new(
        off.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(off.rules()),
    );
    let mut b = Session::new(on.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    a.sim.run_until(Date::new(1926, 7, 1));
    b.sim.run_until(Date::new(1926, 7, 1));
    assert_eq!(
        hash_of(a.world()),
        hash_of(b.world()),
        "the shadow changed the world"
    );
    let n = log.lock().unwrap().events.len();
    println!("shadow events in 18 months: {n}");
}

fn decade(y: i32) -> i32 {
    y / 10 * 10
}

/// The inter-state wars that started in each decade from 1925 (COW v4.0, each war once).
fn wars_by_decade(dir: &std::path::Path) -> BTreeMap<i32, usize> {
    let text = std::fs::read_to_string(dir.join("Inter-StateWarData_v4.0.csv"))
        .unwrap_or_default()
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap_or("").split(',').collect();
    let col = |k: &str| head.iter().position(|h| *h == k);
    let (Some(num), Some(start)) = (col("WarNum"), col("StartYear1")) else {
        return BTreeMap::new();
    };
    let mut first: BTreeMap<String, i32> = BTreeMap::new();
    for l in lines {
        let f: Vec<&str> = l.split(',').collect();
        if let (Some(n), Some(Ok(y))) = (f.get(num), f.get(start).map(|y| y.parse::<i32>())) {
            first.entry((*n).to_owned()).or_insert(y);
        }
    }
    let mut out = BTreeMap::new();
    for y in first.values().filter(|y| (1925..=2025).contains(*y)) {
        *out.entry(decade(*y)).or_default() += 1;
    }
    out
}

#[test]
#[ignore = "101 years; run in the slot"]
fn a_century_in_shadow() {
    let on = load("shadow_ai");
    let rules = on.rules();
    let setup = rules.shadow.clone().unwrap();
    let mut s = Session::new(on.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(2026, 1, 1));
    let events: Vec<ShadowEvent> = setup.log.lock().unwrap().events.clone();
    // The record, out of the repository.
    let out = root().join("target/shadow.csv");
    let mut csv = String::from(
        "variant,year,month,step,by,target,motive,stake,hostility,ratio,cost,join_by,join_target\n",
    );
    for e in &events {
        csv.push_str(&format!(
            "{},{},{},{:?},{},{},{:?},{},{},{},{},{},{}\n",
            setup.variants[usize::from(e.variant)].0,
            e.year,
            e.month,
            e.step,
            e.by,
            e.target,
            e.motive,
            e.stake,
            e.hostility,
            e.ratio,
            e.cost,
            e.join_by.join(" "),
            e.join_target.join(" ")
        ));
    }
    let _ = std::fs::create_dir_all(root().join("target"));
    std::fs::write(&out, csv).unwrap();
    println!("record: {} ({} events)", out.display(), events.len());
    let history = std::env::var_os("BF_COW_DIR")
        .map(|d| wars_by_decade(&PathBuf::from(d)))
        .unwrap_or_default();
    for (v, (label, _)) in setup.variants.iter().enumerate() {
        let mine: Vec<&ShadowEvent> = events
            .iter()
            .filter(|e| usize::from(e.variant) == v)
            .collect();
        let count = |step: Step| -> BTreeMap<i32, usize> {
            let mut m = BTreeMap::new();
            for e in mine.iter().filter(|e| e.step == step) {
                *m.entry(decade(e.year)).or_default() += 1;
            }
            m
        };
        let (demands, yields, wars) = (
            count(Step::Demand),
            count(Step::Yield),
            count(Step::Declare),
        );
        println!(
            "variant {label}: by decade, demands / yielded / wars (history: inter-state wars)"
        );
        for d in (1920..=2020).step_by(10) {
            println!(
                "  {d}s: {} / {} / {} (history {})",
                demands.get(&d).copied().unwrap_or(0),
                yields.get(&d).copied().unwrap_or(0),
                wars.get(&d).copied().unwrap_or(0),
                history.get(&d).map_or("-".to_owned(), |n| n.to_string())
            );
        }
        let mut motives: BTreeMap<String, usize> = BTreeMap::new();
        for e in mine.iter().filter(|e| e.step == Step::Demand) {
            *motives.entry(format!("{:?}", e.motive)).or_default() += 1;
        }
        println!("  demands by motive: {motives:?}");
        if v == 0 {
            let mut pairs: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
            for e in &mine {
                let k = (e.by.clone(), e.target.clone());
                let x = pairs.entry(k).or_default();
                match e.step {
                    Step::Demand => x.0 += 1,
                    Step::Declare => x.1 += 1,
                    _ => {}
                }
            }
            let mut list: Vec<_> = pairs.into_iter().collect();
            list.sort_by_key(|x| std::cmp::Reverse((x.1.1, x.1.0)));
            println!("  the pairs, demands and wars:");
            for ((a, b), (d, w)) in list.iter().take(25) {
                println!("    {a} -> {b}: {d} demands, {w} wars");
            }
        }
    }
}
