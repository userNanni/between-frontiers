//! The relations' measures (`relations`, switch `relations`, off), read only, against the
//! Correlates of War, measured and not tuned:
//! - the CINC of 1925 of the National Material Capabilities v7.0 (`cinc`; the computation:
//!   NMC Documentation v7.0, "CINC Score", p. 8);
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

/// The CINC of 1925, per mille (NMC v7.0, abridged), the 30 strongest.
const NMC_1925: &[(&str, f64)] = &[
    ("USA", 254.0),
    ("CHN", 138.3),
    ("SOV", 101.7),
    ("GBR", 95.6),
    ("GER", 81.6),
    ("FRA", 60.2),
    ("JAP", 39.7),
    ("ITA", 31.1),
    ("POL", 22.6),
    ("ESP", 18.2),
    ("CZE", 14.0),
    ("BEL", 13.6),
    ("BRA", 11.2),
    ("ROM", 8.9),
    ("CAN", 8.7),
    ("YUG", 8.1),
    ("AUS", 7.8),
    ("ARG", 7.7),
    ("MEX", 7.3),
    ("TUR", 6.9),
    ("NLD", 6.6),
    ("AUT", 6.3),
    ("SWE", 5.6),
    ("HUN", 5.2),
    ("ETH", 4.9),
    ("GRE", 4.6),
    ("LUX", 4.1),
    ("PRS", 3.8),
    ("SAF", 3.6),
    ("POR", 3.6),
];

/// (pair, dyad-years with a MID) from 1925 to 1939.
const MID_1925_1939: &[(&str, &str, u32)] = &[
    ("AFG", "PRS", 1),
    ("AFG", "SOV", 3),
    ("ALB", "ITA", 1),
    ("ARG", "GER", 1),
    ("AUS", "GER", 1),
    ("AUT", "GER", 1),
    ("AUT", "ITA", 1),
    ("BEL", "GER", 3),
    ("BOL", "PRY", 12),
    ("BRA", "COL", 2),
    ("BRA", "PER", 2),
    ("BRA", "PRY", 1),
    ("BUL", "GRE", 3),
    ("BUL", "ROM", 1),
    ("BUL", "TUR", 1),
    ("BUL", "YUG", 1),
    ("CAN", "GER", 1),
    ("CAN", "SAL", 1),
    ("CHE", "GER", 1),
    ("CHN", "ESP", 1),
    ("CHN", "FRA", 2),
    ("CHN", "GBR", 2),
    ("CHN", "GER", 1),
    ("CHN", "ITA", 2),
    ("CHN", "JAP", 17),
    ("CHN", "POR", 1),
    ("CHN", "SOV", 7),
    ("CHN", "USA", 2),
    ("COL", "PER", 3),
    ("CUB", "USA", 2),
    ("CZE", "GER", 2),
    ("CZE", "HUN", 2),
    ("CZE", "POL", 1),
    ("DEN", "ITA", 1),
    ("ECU", "PER", 7),
    ("ESP", "GER", 4),
    ("ESP", "ITA", 5),
    ("ESP", "POR", 4),
    ("EST", "GER", 1),
    ("EST", "POL", 1),
    ("EST", "SOV", 1),
    ("ETH", "ITA", 4),
    ("FIN", "GER", 1),
    ("FIN", "SOV", 1),
    ("FRA", "GER", 3),
    ("FRA", "ITA", 7),
    ("FRA", "JAP", 2),
    ("FRA", "NEJ", 1),
    ("FRA", "SOV", 1),
    ("FRA", "TUR", 4),
    ("GBR", "GER", 2),
    ("GBR", "ITA", 6),
    ("GBR", "JAP", 4),
    ("GBR", "NEJ", 1),
    ("GBR", "NLD", 1),
    ("GBR", "SOV", 1),
    ("GBR", "TUR", 3),
    ("GBR", "URY", 1),
    ("GBR", "USA", 1),
    ("GER", "ITA", 2),
    ("GER", "LAT", 1),
    ("GER", "LTU", 1),
    ("GER", "LUX", 1),
    ("GER", "NLD", 1),
    ("GER", "NOR", 1),
    ("GER", "NZL", 1),
    ("GER", "POL", 1),
    ("GER", "SAF", 1),
    ("GER", "SOV", 2),
    ("GER", "SWE", 1),
    ("GER", "URY", 1),
    ("GER", "USA", 1),
    ("GRE", "TUR", 1),
    ("GUA", "HON", 2),
    ("GUA", "SAL", 1),
    ("HON", "NIC", 1),
    ("HON", "USA", 1),
    ("HUN", "ROM", 1),
    ("HUN", "YUG", 1),
    ("IRQ", "PRS", 2),
    ("ITA", "NEJ", 1),
    ("ITA", "PAN", 1),
    ("ITA", "SOV", 1),
    ("ITA", "YUG", 2),
    ("JAP", "MGL", 3),
    ("JAP", "SOV", 12),
    ("JAP", "USA", 2),
    ("LAT", "SOV", 1),
    ("LTU", "POL", 1),
    ("NEJ", "YEM", 5),
    ("NIC", "USA", 1),
    ("POL", "SOV", 2),
    ("PRS", "SOV", 1),
];

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
    for (t, nmc) in NMC_1925 {
        let game = index(w, t).and_then(|c| r.countries.iter().find(|x| x.country == c));
        match game {
            Some(g) => println!(
                "  {t}: game {} (rank {}, {:?}), NMC {nmc}; shares {:?}",
                g.cinc, g.rank, g.role, g.shares
            ),
            None => println!("  {t}: not in the game, NMC {nmc}"),
        }
    }
    // The highest hostility of each pair over the years.
    let mut most: BTreeMap<(String, String), i32> = BTreeMap::new();
    for year in 1925..=1939 {
        session.sim.run_until(Date::new(year, 7, 1));
        let w = session.world();
        let r = at(&d, &s, w, year);
        for p in &r.pairs {
            let (a, b) = (tag(w, p.a), tag(w, p.b));
            let k = if a < b { (a, b) } else { (b, a) };
            let e = most.entry(k).or_default();
            *e = (*e).max(p.hostility);
        }
        if year == 1935 {
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
    let mut ranked: Vec<_> = most.iter().collect();
    ranked.sort_by(|x, y| y.1.cmp(x.1).then(x.0.cmp(y.0)));
    let n = MID_1925_1939.len();
    let top: Vec<&(String, String)> = ranked.iter().take(n).map(|x| x.0).collect();
    let key = |a: &str, b: &str| {
        if a < b {
            (a.to_owned(), b.to_owned())
        } else {
            (b.to_owned(), a.to_owned())
        }
    };
    let hits = MID_1925_1939
        .iter()
        .filter(|(a, b, _)| top.contains(&&key(a, b)))
        .count();
    println!(
        "of the {n} pairs with a MID in 1925-1939, {hits} are among the game's {n} most hostile"
    );
    let mut misses: Vec<_> = MID_1925_1939
        .iter()
        .map(|(a, b, y)| (most.get(&key(a, b)).copied().unwrap_or(0), *a, *b, *y))
        .collect();
    misses.sort_by(|x, y| y.3.cmp(&x.3).then(x.0.cmp(&y.0)));
    println!("the pairs with the most MID years and their game hostility:");
    for m in misses.iter().take(20) {
        println!("  {} {}: {} MID years, hostility {}", m.1, m.2, m.3, m.0);
    }
    println!("the game's most hostile pairs without a MID:");
    for (k, h) in ranked.iter().take(40) {
        if !MID_1925_1939.iter().any(|(a, b, _)| key(a, b) == **k) {
            println!("  {} {}: {}", k.0, k.1, h);
        }
    }
}
