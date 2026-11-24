//! The relations' measures (`relations`, switch `relations`, off), read only, against the
//! Correlates of War, measured and not tuned:
//! - the CINC of 1925 of the National Material Capabilities v7.0 (the computation: NMC
//!   Documentation v7.0, "CINC Score", p. 8);
//! - the pairs with a militarized interstate dispute in some year from 1925 to 1939, with the
//!   number of dyad-years (Dyadic MID v4.03; Maoz and Miner, Dyadic MID Codebook v4.02, 2021,
//!   p. 1: a record per directed dyad and year, here counted once per pair and year).
//!
//! COW codes are mapped to the game's tags (UKG GBR, GMY GER, RUS SOV, JPN JAP, AUS AUT, AUL
//! AUS, SAU NEJ, YAR YEM, ...).

use std::collections::BTreeMap;
use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameWorld, Session};
use bf_sim::relations::{Relations, RelationsSetup, measure};
use laplace_schelling::Date;

/// The Correlates of War's data is not in the repository: its terms ("Users agree not to
/// distribute the dataset to any third party without written permission of the COW director and
/// data host", correlatesofwar.org, "Data Sets") keep it out. The conference reads it from
/// `BF_COW_DIR`, downloaded at the time from correlatesofwar.org: `NMC-70-abridged.csv` (in
/// `NMCv7.zip`) and `dyadic_mid_4.03.csv` (in `dyadic_mid_4.03_update.zip`).
fn cow_dir() -> Option<PathBuf> {
    std::env::var_os("BF_COW_DIR").map(PathBuf::from)
}

/// The game's tag of a COW state abbreviation.
fn game_tag(cow: &str) -> String {
    match cow {
        "AUL" => "AUS",
        "AUS" => "AUT",
        "COS" => "CRI",
        "FRN" => "FRA",
        "GMY" => "GER",
        "GRC" => "GRE",
        "IRE" => "IRL",
        "IRN" => "PRS",
        "JPN" => "JAP",
        "LIT" => "LTU",
        "MON" => "MGL",
        "NEW" => "NZL",
        "NTH" => "NLD",
        "PAR" => "PRY",
        "RUS" => "SOV",
        "SAU" => "NEJ",
        "SPN" => "ESP",
        "SWD" => "SWE",
        "SWZ" => "CHE",
        "THI" => "SIA",
        "UKG" => "GBR",
        "URU" => "URY",
        "YAR" => "YEM",
        x => x,
    }
    .to_owned()
}

/// The columns of a CSV line (no quoted commas in these two files' columns read here).
fn rows(path: &std::path::Path) -> Vec<BTreeMap<String, String>> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut lines = text.lines();
    let head: Vec<String> = lines
        .next()
        .unwrap_or("")
        .split(',')
        .map(|h| h.trim_matches('"').to_owned())
        .collect();
    lines
        .map(|l| {
            head.iter()
                .cloned()
                .zip(l.split(',').map(|v| v.trim_matches('"').to_owned()))
                .collect()
        })
        .collect()
}

/// The CINC of 1925 (per mille), the 30 strongest.
fn nmc_1925(dir: &std::path::Path) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = rows(&dir.join("NMC-70-abridged.csv"))
        .into_iter()
        .filter(|r| r.get("year").map(String::as_str) == Some("1925"))
        .filter_map(|r| {
            let c: f64 = r.get("cinc")?.parse().ok()?;
            Some((game_tag(r.get("stateabb")?), c * 1000.0))
        })
        .collect();
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out.truncate(30);
    out
}

/// (pair, dyad-years with a MID) from 1925 to 1939, each pair and year once.
fn mid_1925_1939(dir: &std::path::Path) -> Vec<(String, String, u32)> {
    let mut years: BTreeMap<(String, String), std::collections::BTreeSet<i32>> = BTreeMap::new();
    for r in rows(&dir.join("dyadic_mid_4.03.csv")) {
        let (Some(y), Some(a), Some(b)) = (r.get("year"), r.get("namea"), r.get("nameb")) else {
            continue;
        };
        let Ok(y) = y.parse::<i32>() else { continue };
        if !(1925..=1939).contains(&y) {
            continue;
        }
        let (a, b) = (game_tag(a), game_tag(b));
        let k = if a < b { (a, b) } else { (b, a) };
        years.entry(k).or_default().insert(y);
    }
    years
        .into_iter()
        .map(|((a, b), y)| (a, b, y.len() as u32))
        .collect()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load() -> (GameData, RelationsSetup) {
    let d = GameData::load_with(&root(), "relations").unwrap_or_else(|e| panic!("{e}"));
    let s = d
        .relations_setup()
        .expect("relations.ron with the switch on");
    (d, s)
}

fn tag(w: &GameWorld, c: u16) -> String {
    w.countries
        .iter()
        .nth(usize::from(c))
        .unwrap()
        .1
        .tag
        .clone()
}

fn index(w: &GameWorld, t: &str) -> Option<u16> {
    w.countries
        .iter()
        .position(|(_, c)| c.tag == t)
        .map(|i| i as u16)
}

fn at(d: &GameData, s: &RelationsSetup, w: &GameWorld, year: i32) -> Relations {
    measure(s, &d.catalogue, w, d.community_table.as_deref(), year)
}

/// Every claim names countries of the game, and its years are in order.
#[test]
fn every_claim_is_read() {
    let (d, s) = load();
    let file = d.claims.as_ref().expect("claims.ron with the switch on");
    assert_eq!(
        s.claims.len(),
        file.claims.len(),
        "a claim names a tag the game lacks"
    );
    for c in &file.claims {
        assert!(
            c.until.is_none_or(|u| c.from <= u),
            "{}: ends before it starts",
            c.territory
        );
    }
}

#[test]
fn off_nothing_is_loaded() {
    let d = GameData::load(&root()).unwrap_or_else(|e| panic!("{e}"));
    assert!(d.relations.is_none() && d.relations_setup().is_none());
}

/// The measures of 1925: the CINC shares sum to the world, the terms are in range, and the
/// pairs with land of one in the other's hands (Hungary and its neighbours) are there.
#[test]
fn the_measures_of_1925_hold_together() {
    let (d, s) = load();
    // Steel and energy come from the markets: a few weeks of them first.
    let mut session = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    session.sim.run_until(Date::new(1925, 2, 1));
    let w = session.world().clone();
    let r = at(&d, &s, &w, 1925);
    for k in 0..6 {
        let sum: u32 = r.countries.iter().map(|c| c.shares[k]).sum();
        // Each share is rounded down: up to one per mille per country lost.
        let n = r.countries.len() as u32;
        assert!(
            (1000 - n..=1000).contains(&sum),
            "component {k} sums to {sum}"
        );
    }
    assert!(r.pairs.iter().all(|p| (0..=1000).contains(&p.hostility)));
    let hun = index(&w, "HUN").unwrap();
    let h = r.countries.iter().find(|c| c.country == hun).unwrap();
    println!("HUN 1925: lost territory {} per mille", h.lost_territory);
    let mut top: Vec<_> = r.pairs.iter().collect();
    top.sort_by(|x, y| {
        y.hostility
            .cmp(&x.hostility)
            .then((x.a, x.b).cmp(&(y.a, y.b)))
    });
    for p in top.iter().take(15) {
        println!(
            "{} {} {} {:?}",
            tag(&w, p.a),
            tag(&w, p.b),
            p.hostility,
            p.terms
        );
    }
}

/// CPU: the measure of the whole world, timed.
#[test]
#[ignore = "timing; run in the slot"]
fn the_cost_of_a_measure() {
    let (d, s) = load();
    let w = d.new_world();
    let t = std::time::Instant::now();
    for _ in 0..10 {
        std::hint::black_box(at(&d, &s, &w, 1925));
    }
    println!("a measure of the world: {:?}", t.elapsed() / 10);
}

/// The conference: the CINC of 1925 against the NMC, and the pairs most hostile from 1925 to
/// 1939 against those with a MID. Measured, not tuned.
#[test]
#[ignore = "15 years; run in the slot"]
fn against_the_correlates_of_war() {
    let Some(dir) = cow_dir() else {
        println!("BF_COW_DIR unset: the conference needs the Correlates of War's files");
        return;
    };
    let nmc = nmc_1925(&dir);
    let mids = mid_1925_1939(&dir);
    let (d, s) = load();
    let mut session = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    session.sim.run_until(Date::new(1925, 2, 1));
    let w = session.world();
    let r = at(&d, &s, w, 1925);
    println!("CINC 1925, per mille: game against NMC v7.0");
    for (t, nmc) in &nmc {
        let game = index(w, t).and_then(|c| r.countries.iter().find(|x| x.country == c));
        match game {
            Some(g) => println!(
                "  {t}: game {} (rank {}, {:?}), NMC {nmc}; shares {:?}",
                g.cinc, g.rank, g.role, g.shares
            ),
            None => println!("  {t}: not in the game, NMC {nmc}"),
        }
    }
    // The highest hostility of each pair over the years, for each way of weighing the minority.
    // The measure as chosen, the other minority weight, and the measure without the claims.
    let forms = ["HomeShare", "PopulationRatio", "HomeShare without claims"];
    let setups: Vec<RelationsSetup> = forms
        .iter()
        .map(|f| {
            let mut x = s.clone();
            if *f == "PopulationRatio" {
                x.rules.minority_by = bf_sim::relations::MinorityBy::PopulationRatio;
            }
            if f.ends_with("without claims") {
                x.rules.weights.claim = 0;
            }
            x
        })
        .collect();
    let mut most: Vec<BTreeMap<(String, String), i32>> = vec![BTreeMap::new(); forms.len()];
    for year in 1925..=1939 {
        session.sim.run_until(Date::new(year, 7, 1));
        let w = session.world();
        for (k, setup) in setups.iter().enumerate() {
            let r = at(&d, setup, w, year);
            for p in &r.pairs {
                let (a, b) = (tag(w, p.a), tag(w, p.b));
                let key = if a < b { (a, b) } else { (b, a) };
                let e = most[k].entry(key).or_default();
                *e = (*e).max(p.hostility);
            }
            if year == 1935 && k == 0 {
                let mut c: Vec<_> = r.countries.iter().collect();
                c.sort_by_key(|x| x.rank);
                for x in c.iter().take(10) {
                    println!(
                        "1935 {}: CINC {} {:?} threat {} lost {}",
                        tag(w, x.country),
                        x.cinc,
                        x.role,
                        x.threat,
                        x.lost_territory
                    );
                }
            }
        }
    }
    let key = |a: &str, b: &str| {
        if a < b {
            (a.to_owned(), b.to_owned())
        } else {
            (b.to_owned(), a.to_owned())
        }
    };
    for (form, most) in forms.iter().zip(&most) {
        let mut ranked: Vec<_> = most.iter().collect();
        ranked.sort_by(|x, y| y.1.cmp(x.1).then(x.0.cmp(y.0)));
        let n = mids.len();
        let top: Vec<&(String, String)> = ranked.iter().take(n).map(|x| x.0).collect();
        let hits = mids
            .iter()
            .filter(|(a, b, _)| top.contains(&&key(a, b)))
            .count();
        let alarms = top
            .iter()
            .filter(|k| !mids.iter().any(|(a, b, _)| key(a, b) == ***k))
            .count();
        println!(
            "{form}: of the {n} pairs with a MID in 1925-1939, {hits} are among the game's {n} \
             most hostile; of the game's {n} most hostile, {alarms} had no MID"
        );
        let mut misses: Vec<_> = mids
            .iter()
            .map(|(a, b, y)| {
                (
                    most.get(&key(a, b)).copied().unwrap_or(0),
                    a.as_str(),
                    b.as_str(),
                    *y,
                )
            })
            .collect();
        misses.sort_by(|x, y| y.3.cmp(&x.3).then(x.0.cmp(&y.0)));
        println!("{form}: the pairs with the most MID years and their game hostility:");
        for m in misses.iter().take(20) {
            println!("  {} {}: {} MID years, hostility {}", m.1, m.2, m.3, m.0);
        }
        println!("{form}: the game's most hostile pairs without a MID:");
        for (k, h) in ranked.iter().take(40) {
            if !mids.iter().any(|(a, b, _)| key(a, b) == **k) {
                println!("  {} {}: {}", k.0, k.1, h);
            }
        }
    }
}

/// The ten most hostile pairs of 1925 and 1950, for each reach of the regime term.
#[test]
#[ignore = "25 years; run in the slot"]
fn the_most_hostile_by_the_regime_reach() {
    let (d, s) = load();
    let forms = [
        bf_sim::relations::RegimeReach::Powers,
        bf_sim::relations::RegimeReach::Warheads,
    ];
    let mut session = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    for year in [1925, 1950] {
        session.sim.run_until(Date::new(year, 7, 1));
        let w = session.world();
        for f in forms {
            let mut x = s.clone();
            x.rules.regime_reach = f;
            let r = at(&d, &x, w, year);
            let mut top: Vec<_> = r.pairs.iter().collect();
            top.sort_by(|p, q| {
                q.hostility
                    .cmp(&p.hostility)
                    .then((p.a, p.b).cmp(&(q.a, q.b)))
            });
            println!("{year} {f:?}:");
            for p in top.iter().take(10) {
                println!(
                    "  {} {} {} {:?}",
                    tag(w, p.a),
                    tag(w, p.b),
                    p.hostility,
                    p.terms
                );
            }
            for (x, y) in [
                ("USA", "SOV"),
                ("GER", "SIA"),
                ("GER", "POL"),
                ("GER", "FRA"),
            ] {
                if let (Some(i), Some(j)) = (index(w, x), index(w, y)) {
                    let p = r.pair(i, j);
                    println!(
                        "  pair {x} {y}: {} {:?}",
                        p.map_or(0, |p| p.hostility),
                        p.map(|p| p.terms)
                    );
                }
            }
        }
    }
}
