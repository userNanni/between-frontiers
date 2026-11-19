//! The nuclear programme (`docs/war.pt-BR.md`, "Arma nuclear na escada de crise", step 2; N1,
//! N2): its money spent and credited to the chain, the enrichment plants it asks for, the warheads
//! it orders, and the pace from a start in 1940.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::research::Stage;
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.war = true;
        d
    })
}

fn index(w: &GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
}

fn tech(d: &GameData, key: &str) -> usize {
    d.tree
        .as_ref()
        .unwrap()
        .techs
        .iter()
        .position(|t| t.key == key)
        .unwrap()
}

fn week(s: &mut Session) {
    for _ in 0..7 {
        s.sim.step();
    }
}

/// Progress and stage of the chain, summed as stage + progress.
fn chain(d: &GameData, w: &GameWorld, c: usize) -> f64 {
    ["nuclear_fission", "uranium_enrichment", "atomic_bomb"]
        .iter()
        .map(|k| {
            let t = tech(d, k);
            w.research.stage[c][t] as usize as f64 + w.research.progress[c][t].to_f64_lossy()
        })
        .sum()
}

/// The programme spends its money every week and the research is credited with it: the chain
/// moves faster than without it.
#[test]
fn the_programme_spends_and_moves_the_chain() {
    let d = data();
    let run = |on: bool| {
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        s.sim.run_until(Date::new(1925, 2, 2));
        if on {
            s.submit(GameCommand::NuclearProgramme {
                tag: "USA".into(),
                share: Some(20),
            });
        }
        for _ in 0..26 {
            week(&mut s);
        }
        s
    };
    let (with, without) = (run(true), run(false));
    let (w, wo) = (with.world(), without.world());
    let usa = index(w, "USA");
    let p = w.war.nuclear.of(usa as u16).expect("the programme");
    eprintln!(
        "spent {:.3e}, month {:.3e}; chain {:.3} with, {:.3} without",
        p.spent.to_f64_lossy(),
        p.month.to_f64_lossy(),
        chain(d, w, usa),
        chain(d, wo, usa)
    );
    assert!(p.spent.is_positive() && p.month.is_positive());
    assert!(
        chain(d, w, usa) > chain(d, wo, usa),
        "the chain moves faster"
    );
}

/// With the chain industrial, the programme asks for an enrichment plant, the works queue builds
/// it, and the warhead order is paid and delivered as the fissile material comes.
#[test]
fn plants_and_warheads() {
    let d = data();
    let mut world = d.new_world();
    let usa = index(&world, "USA");
    for k in ["nuclear_fission", "uranium_enrichment", "atomic_bomb"] {
        world.research.stage[usa][tech(d, k)] = Stage::Industrial;
    }
    let mut s = Session::new(world, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
    s.submit(GameCommand::NuclearProgramme {
        tag: "USA".into(),
        share: Some(20),
    });
    let mil = &d.war.as_ref().unwrap().military;
    let warhead = mil.unit("nuclear_warhead").unwrap();
    let plant = d.catalogue.building("nuclear_fuel_plant").unwrap();
    let mut first = None;
    for wk in 0..156 {
        week(&mut s);
        let w = s.world();
        let n: f64 = w.war.countries[usa]
            .formations
            .iter()
            .filter(|f| f.unit == warhead)
            .map(|f| f.elements.to_f64_lossy())
            .sum();
        if n > 0.0 && first.is_none() {
            first = Some(wk);
        }
        if wk % 26 == 25 {
            let levels: u32 = w
                .econ
                .regions
                .iter()
                .filter(|r| usize::from(r.market) == usa)
                .flat_map(|r| r.plants.iter())
                .filter(|p| usize::from(p.kind) == plant)
                .map(|p| u32::try_from(p.levels).unwrap_or(0))
                .sum();
            let fuel = usize::from(d.catalogue.good("nuclear_fuel").unwrap());
            let m = &w.econ.markets[usa];
            eprintln!(
                "week {}: plants {levels}, warheads {n:.0}, fissile price {:.0}, delivered {:.2}",
                wk + 1,
                m.price[fuel].to_f64_lossy(),
                m.buy_ratio[fuel].to_f64_lossy()
            );
        }
    }
    let w = s.world();
    let queued = w.econ.markets[usa]
        .queue
        .iter()
        .any(|q| usize::from(q.kind) == plant);
    let built = w
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == usa)
        .flat_map(|r| r.plants.iter())
        .any(|p| usize::from(p.kind) == plant && p.levels > 0);
    assert!(queued || built, "an enrichment plant asked for");
    eprintln!("first warhead in week {first:?}");
    assert!(first.is_some(), "warheads delivered");
    let _ = Fixed::ZERO;
}

/// The pace (a check, not a target): an industrial country that starts the programme in 1940,
/// at the share of the Manhattan Project, and the stock five and ten years on. Also the price of
/// electricity against the same world without the programme. Run in the slot.
#[test]
#[ignore = "long: 1925 to 1955"]
fn the_pace_from_1940() {
    let d = data();
    let run = |on: bool| {
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        s.sim.run_until(Date::new(1940, 1, 1));
        if on {
            for tag in ["USA", "GER", "GBR"] {
                s.submit(GameCommand::NuclearProgramme {
                    tag: tag.into(),
                    share: Some(4),
                });
            }
        }
        let mut lines = Vec::new();
        for year in 1940..=1955 {
            s.sim.run_until(Date::new(year, 12, 31));
            lines.push(s.world().clone());
        }
        lines
    };
    let with = run(true);
    let without = run(false);
    let mil = &d.war.as_ref().unwrap().military;
    let warhead = mil.unit("nuclear_warhead").unwrap();
    let elec = usize::from(d.catalogue.good("electricity").unwrap());
    let fuel = usize::from(d.catalogue.good("nuclear_fuel").unwrap());
    for tag in ["USA", "GER", "GBR"] {
        eprintln!(
            "== {tag}: year | fission enrichment bomb | warheads | fissile price | electricity with/without"
        );
        for (k, (w, wo)) in with.iter().zip(&without).enumerate() {
            let c = index(w, tag);
            let st = |key: &str| format!("{:?}", w.research.stage[c][tech(d, key)]);
            let n: f64 = w.war.countries[c]
                .formations
                .iter()
                .filter(|f| f.unit == warhead)
                .map(|f| f.elements.to_f64_lossy())
                .sum();
            eprintln!(
                "{} | {} {} {} | {n:.0} | {:.0} | {:.3} / {:.3}",
                1940 + k,
                st("nuclear_fission"),
                st("uranium_enrichment"),
                st("atomic_bomb"),
                w.econ.markets[c].price[fuel].to_f64_lossy(),
                w.econ.markets[c].price[elec].to_f64_lossy(),
                wo.econ.markets[c].price[elec].to_f64_lossy()
            );
        }
    }
}
