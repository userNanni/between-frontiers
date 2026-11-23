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
    let plant = d.catalogue.building("weapon_core_plant").unwrap();
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
            let fuel = usize::from(d.catalogue.good("weapon_cores").unwrap());
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
    let fuel = usize::from(d.catalogue.good("weapon_cores").unwrap());
    let ore = usize::from(d.catalogue.good("uranium_ore").unwrap());
    let plant = d.catalogue.building("weapon_core_plant").unwrap();
    let mine = d.catalogue.building("uranium_mine").unwrap();
    let assembly = d.catalogue.building("weapon_assembly_plant").unwrap();
    let assemblies = d.catalogue.good("weapon_assemblies").unwrap();
    let cores = d.catalogue.good("weapon_cores").unwrap();
    for tag in ["USA", "GER", "GBR"] {
        eprintln!(
            "== {tag}: year | fission enrichment bomb | warheads | fissile price | electricity with/without | core plant levels, ore delivered to them | uranium mine levels at home and in colonies | ore price | assembly plant levels | cores waiting for an assembly"
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
            let plants = |kind: usize| {
                w.econ
                    .regions
                    .iter()
                    .filter(|r| usize::from(r.market) == c)
                    .flat_map(|r| r.plants.iter())
                    .filter(|p| usize::from(p.kind) == kind)
                    .map(|p| (p.levels, p.input_ratio.to_f64_lossy()))
                    .fold((0, 1.0f64), |(l, x), (n, y)| (l + n, x.min(y)))
            };
            let (core_levels, ore_in) = plants(plant);
            let (mine_levels, _) = plants(mine);
            let (assembly_levels, _) = plants(assembly);
            // The warhead order's cores held beyond its assemblies.
            let held = |g: u16| -> f64 {
                w.war.countries[c]
                    .queue
                    .iter()
                    .flat_map(|o| o.built.iter())
                    .filter(|(x, _)| *x == g)
                    .map(|(_, v)| v.to_f64_lossy())
                    .sum()
            };
            let waiting = (held(cores) - held(assemblies)).max(0.0);
            eprintln!(
                "{} | {} {} {} | {n:.0} | {:.0} | {:.3} / {:.3} | {core_levels}, {ore_in:.2} | {mine_levels} | {:.0} | {assembly_levels} | {waiting:.1}",
                1940 + k,
                st("nuclear_fission"),
                st("uranium_enrichment"),
                st("atomic_bomb"),
                w.econ.markets[c].price[fuel].to_f64_lossy(),
                w.econ.markets[c].price[elec].to_f64_lossy(),
                wo.econ.markets[c].price[elec].to_f64_lossy(),
                w.econ.markets[c].price[ore].to_f64_lossy()
            );
        }
    }
}

/// U2: the weapon core plant is a state firm, and so is the assembly plant. Their profit, or
/// their loss, goes whole to the treasury: with no other state firm (the ownership law at
/// nothing in 1925), the week's firm revenue of the United States is exactly what its core and
/// assembly plants made. The core costs the country
/// its inputs, wages and works, whatever the price.
#[test]
fn the_core_plants_profit_is_the_treasurys() {
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
    let plant = d.catalogue.building("weapon_core_plant").unwrap();
    let assembly = d.catalogue.building("weapon_assembly_plant").unwrap();
    let mut checked = 0;
    for _ in 0..156 {
        week(&mut s);
        let w = s.world();
        let profit: Fixed = w
            .econ
            .regions
            .iter()
            .filter(|r| usize::from(r.market) == usa)
            .flat_map(|r| r.plants.iter())
            .filter(|p| usize::from(p.kind) == plant || usize::from(p.kind) == assembly)
            .map(|p| p.profit)
            .sum();
        let m = &w.econ.markets[usa];
        let firms = m.budget.revenue_firms;
        let no_other = m
            .laws
            .effects
            .iter()
            .all(|e| e.state_ownership == Fixed::ZERO);
        if profit != Fixed::ZERO && no_other {
            assert_eq!(
                firms, profit,
                "the week's firm revenue is the core and assembly plants' profit"
            );
            checked += 1;
        }
    }
    eprintln!("weeks with a core plant's profit checked: {checked}");
    assert!(checked > 0, "a core plant ran");
}

/// With `war` off, no country builds a weapon core plant: a state firm is built only when the
/// state asks for it (no programme without the war). Also who reaches the production reactor, and
/// when. A measure for the PR: run in the slot.
#[test]
#[ignore = "long: a century with war off"]
fn no_core_plant_without_a_programme() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    assert!(!d.switches.war);
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    let plant = d.catalogue.building("weapon_core_plant").unwrap();
    let reactor = tech(&d, "production_reactor");
    let method_levels = |w: &GameWorld, building: &str, method: &str| -> i64 {
        let k = d.catalogue.building(building).unwrap();
        let m = d.catalogue.buildings[k]
            .methods
            .iter()
            .position(|x| x.key == method)
            .unwrap();
        w.econ
            .regions
            .iter()
            .flat_map(|r| r.plants.iter())
            .filter(|p| usize::from(p.kind) == k && usize::from(p.method) == m)
            .map(|p| p.levels)
            .sum()
    };
    for year in [
        1935, 1945, 1950, 1955, 1965, 1975, 1985, 1995, 2005, 2015, 2025,
    ] {
        s.sim.run_until(Date::new(year, 1, 1));
        let w = s.world();
        let levels: i64 = w
            .econ
            .regions
            .iter()
            .flat_map(|r| r.plants.iter())
            .filter(|p| usize::from(p.kind) == plant)
            .map(|p| p.levels)
            .sum();
        let reached: Vec<String> = w
            .countries
            .iter()
            .enumerate()
            .filter(|(i, _)| w.research.stage[*i][reactor] > Stage::Unknown)
            .map(|(_, (_, c))| c.tag.clone())
            .collect();
        let count = |k: &str| {
            let t = tech(&d, k);
            (0..w.countries.len())
                .filter(|&i| w.research.stage[i][t] >= Stage::Science)
                .count()
        };
        let with_bomb = count("atomic_bomb");
        let np = tech(&d, "nuclear_power");
        eprintln!(
            "{year}: jet aircraft plant levels {}, supermarket levels {}; nuclear power first industrial {}",
            method_levels(w, "aircraft_plant", "jet"),
            method_levels(w, "trade_center", "supermarkets"),
            w.research.firsts[np][Stage::Industrial as usize]
        );
        eprintln!(
            "{year}: the hydrogen bomb in {}, nuclear power in {}, enrichment in {}",
            count("thermonuclear_weapon"),
            count("nuclear_power"),
            count("uranium_enrichment")
        );
        eprintln!(
            "{year}: weapon core plant levels in the world {levels}; production reactor known in {} countries; the atomic bomb understood or beyond in {with_bomb}",
            reached.len(),
        );
        assert_eq!(levels, 0, "no core plant without a programme");
        let assembly = d.catalogue.building("weapon_assembly_plant").unwrap();
        let assembly_levels: i64 = w
            .econ
            .regions
            .iter()
            .flat_map(|r| r.plants.iter())
            .filter(|p| usize::from(p.kind) == assembly)
            .map(|p| p.levels)
            .sum();
        assert_eq!(assembly_levels, 0, "no assembly plant without a programme");
        assert!(
            reached.is_empty(),
            "no production reactor without a programme"
        );
    }
}

/// U1: with its core plants short of ore, the programme opens a uranium mine with the state's
/// capital in a deposit already found, at home first: the United States in its own carnotite.
#[test]
fn the_programme_opens_a_mine() {
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
    let mine = d.catalogue.building("uranium_mine").unwrap();
    let res = d.catalogue.buildings[mine].resource.unwrap();
    let levels = |s: &Session| -> (i64, bool) {
        let w = s.world();
        let built = w
            .econ
            .regions
            .iter()
            .filter(|r| usize::from(r.market) == usa)
            .flat_map(|r| r.plants.iter())
            .filter(|p| usize::from(p.kind) == mine)
            .map(|p| p.levels)
            .sum();
        let queued = w.econ.markets[usa]
            .queue
            .iter()
            .any(|q| usize::from(q.kind) == mine);
        (built, queued)
    };
    let (before, _) = levels(&s);
    let mut asked = None;
    let mut most = before;
    for wk in 0..156 {
        week(&mut s);
        let (built, queued) = levels(&s);
        most = most.max(built);
        if asked.is_none() && (queued || built > before) {
            asked = Some(wk);
        }
        if wk % 13 == 0 {
            let w = s.world();
            let ore = usize::from(d.catalogue.good("uranium_ore").unwrap());
            let core = d.catalogue.building("weapon_core_plant").unwrap();
            let plants: i64 = w
                .econ
                .regions
                .iter()
                .filter(|r| usize::from(r.market) == usa)
                .flat_map(|r| r.plants.iter())
                .filter(|p| usize::from(p.kind) == core)
                .map(|p| p.levels)
                .sum();
            eprintln!(
                "week {wk}: mines {built} queued {queued}, core plants {plants}, ore delivered {:.2} price {:.0}",
                w.econ.markets[usa].buy_ratio[ore].to_f64_lossy(),
                w.econ.markets[usa].price[ore].to_f64_lossy()
            );
        }
    }
    let w = s.world();
    let (after, _) = levels(&s);
    let homes: Vec<String> = w
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == usa && !r.colony)
        .filter(|r| r.deposits.iter().any(|(x, y, _)| *x == res && *y <= 1925))
        .map(|r| r.state.to_string())
        .collect();
    eprintln!(
        "uranium mine levels {before} → {after}, first asked in week {asked:?}; home states with a deposit found: {homes:?}"
    );
    assert!(!homes.is_empty(), "the United States has carnotite");
    assert!(asked.is_some(), "a mine asked for");
    assert!(most > before, "a mine built");
}

/// The warhead order follows the money (no batch): it grows past one warhead, and the warheads
/// join one by one. When the programme ends, the order waits with the cores and assemblies it
/// holds: the budget does not buy it, nothing is lost, and no warhead is added.
#[test]
fn the_order_follows_the_money_and_waits_when_the_programme_ends() {
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
    let count = |w: &GameWorld| -> Fixed {
        w.war.countries[usa]
            .formations
            .iter()
            .filter(|f| f.unit == warhead)
            .map(|f| f.elements)
            .sum()
    };
    let order = |w: &GameWorld| {
        w.war.countries[usa]
            .queue
            .iter()
            .find(|o| o.unit == "nuclear_warhead")
            .cloned()
    };
    let mut largest = Fixed::ZERO;
    for _ in 0..156 {
        week(&mut s);
        if let Some(o) = order(s.world()) {
            largest = largest.max(o.ordered - o.delivered);
        }
    }
    let w = s.world();
    eprintln!(
        "after three years: warheads {:.1}, the largest open order {:.1}",
        count(w).to_f64_lossy(),
        largest.to_f64_lossy()
    );
    assert!(largest > Fixed::ONE, "the order grows with the money");
    s.submit(GameCommand::NuclearProgramme {
        tag: "USA".into(),
        share: None,
    });
    week(&mut s);
    week(&mut s);
    let (n0, o0) = (count(s.world()), order(s.world()));
    for _ in 0..26 {
        week(&mut s);
    }
    let (n1, o1) = (count(s.world()), order(s.world()));
    eprintln!(
        "programme ended: warheads {:.1} → {:.1}; held {:?} → {:?}",
        n0.to_f64_lossy(),
        n1.to_f64_lossy(),
        o0.as_ref().map(|o| o.built.clone()),
        o1.as_ref().map(|o| o.built.clone())
    );
    assert_eq!(n0, n1, "no warhead without the programme");
    assert_eq!(
        o0.map(|o| o.built),
        o1.map(|o| o.built),
        "what the order holds waits, and nothing is lost"
    );
}
