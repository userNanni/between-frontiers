//! The war at sea, minimal (slice C2; `docs/war.pt-BR.md`, "Mar e ar") on the real scenario:
//! fleets at their stations, battle and control of a zone, the blockade by pair, the air over
//! the fronts, and the sea route of a colonial expedition.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use laplace_schelling::Date;

fn load(tweak: impl FnOnce(&mut GameData)) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    // These tests follow the sea and the air, not the end of the war.
    if let Some(c) = d.war.as_mut().and_then(|w| w.conduct.as_mut()) {
        c.support.overrun = 2000;
        c.support.capital_overrun = 2000;
        c.support.peace = -1;
        c.support.collapse = -1;
    }
    tweak(&mut d);
    d
}

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| load(|_| {}))
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

fn week(s: &mut Session) {
    for _ in 0..7 {
        s.sim.step();
    }
}

/// `by`, played, demands `target` whole, hears the refusal and declares.
fn war(d: &GameData, world: GameWorld, by: &str, target: &str) -> Session {
    let mut s = Session::new(world, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
    s.submit(GameCommand::ChoosePlayer { tag: by.into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: by.into(),
        target: target.into(),
        demand: Proposal {
            side_a: vec![by.into()],
            side_b: vec![target.into()],
            clauses: vec![Clause {
                kind: ClauseKind::Union,
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
        weeks: None,
    });
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: by.into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    s
}

fn zone(d: &GameData, key: &str) -> u16 {
    d.seas.as_ref().unwrap().zone(key).unwrap()
}

fn ships(w: &GameWorld, d: &GameData, c: u16) -> i64 {
    let mil = &d.war.as_ref().unwrap().military;
    w.war.countries[c as usize]
        .formations
        .iter()
        .filter(|f| mil.types[f.unit as usize].domain == bf_sim::war::Domain::Sea)
        .map(|f| f.elements.to_int_floor())
        .sum()
}

/// The blockade of Germany (1914–18): Britain, stronger on the surface, takes station at
/// Germany's way out to sea (the North Sea) and holds it; the pairs of Germany's trade by sea
/// are cut, those by land and by the Baltic stay open. Compared with the hand-closed set of
/// `tests/blockade.rs` (every partner whose cheaper way to Germany is by sea).
#[test]
fn the_blockade_of_germany() {
    let d = data();
    let mut s = war(d, d.new_world(), "GBR", "GER");
    for _ in 0..3 {
        week(&mut s);
    }
    let w = s.world();
    let (gbr, ger, swe) = (index(w, "GBR"), index(w, "GER"), index(w, "SWE"));
    let north = zone(d, "north_sea");
    let sea = &w.war.sea;
    let fleet = |c: u16| sea.fleets.iter().find(|f| f.country == c).cloned();
    let (b, g) = (fleet(gbr).unwrap(), fleet(ger).unwrap());
    eprintln!(
        "GBR {} ships at {} (against {:?}); GER {} at {}",
        b.ships, b.zone, b.against, g.ships, g.zone
    );
    assert_eq!(b.zone, north, "Britain holds Germany's way out");
    assert_eq!(b.against, Some(ger));
    assert_eq!(sea.holder(north), Some(gbr));
    assert!(bf_sim::war::fight::blockaded(w).contains(&(gbr, ger)));
    let cut: BTreeSet<u16> = bf_sim::war::fight::blockaded_pairs(w)
        .into_iter()
        .filter(|&(a, b)| a == ger || b == ger)
        .map(|(a, b)| if a == ger { b } else { a })
        .collect();
    let geo = d.trade_geography();
    let by_hand: BTreeSet<u16> = (0..w.countries.len() as u16)
        .filter(|&x| x != ger && x != gbr && geo.by_sea(x as usize, ger as usize))
        .collect();
    let tag = |c: &u16| w.countries.iter().nth(*c as usize).unwrap().1.tag.clone();
    let open: Vec<String> = by_hand.difference(&cut).map(tag).collect();
    eprintln!(
        "cut {} of the {} sea pairs closed by hand; open by the Baltic or a free route: {open:?}",
        cut.len(),
        by_hand.len()
    );
    assert!(!cut.is_empty());
    assert!(cut.is_subset(&by_hand), "only pairs by sea are cut");
    assert!(
        !cut.contains(&swe),
        "Germany and Sweden trade across the Baltic"
    );
    // A limit, not a target: Norway stays open by the Kattegat. In 1914–18 its trade with
    // Germany did go on, but along its territorial waters (the Leads), which the zones do not
    // see; the British closed them by mining in 1918. The model leaves Norway open for the
    // sea-zone reason, not the territorial-waters one.
    let nor = index(w, "NOR");
    assert!(
        !cut.contains(&nor),
        "Norway open by the Kattegat (the limit above)"
    );
}

/// Jutland: the German fleet does not change who holds the North Sea; the control stays British
/// week after week.
#[test]
fn jutland_the_north_sea_stays_british() {
    let d = data();
    let mut s = war(d, d.new_world(), "GBR", "GER");
    let (gbr, ger) = (index(s.world(), "GBR"), index(s.world(), "GER"));
    let before = (ships(s.world(), d, gbr), ships(s.world(), d, ger));
    for _ in 0..12 {
        week(&mut s);
        assert_eq!(s.world().war.sea.holder(zone(d, "north_sea")), Some(gbr));
    }
    let after = (ships(s.world(), d, gbr), ships(s.world(), d, ger));
    eprintln!(
        "ships GBR {} → {}, GER {} → {}",
        before.0, after.0, before.1, after.1
    );
}

/// The air over the fronts: Germany against France (1925 forces; Germany has no air arm under
/// Versailles), with and without the French air arm. The superiority enters the land step's air
/// term; the week's advance shows it.
#[test]
fn the_air_over_the_front() {
    let d = data();
    let run = |grounded: bool| -> (f64, f64) {
        let mut world = d.new_world();
        if grounded {
            let ger = index(&world, "FRA") as usize;
            let mil = &d.war.as_ref().unwrap().military;
            world.war.countries[ger]
                .formations
                .retain(|f| mil.types[f.unit as usize].domain != bf_sim::war::Domain::Air);
        }
        let mut s = war(d, world, "GER", "FRA");
        s.submit(GameCommand::SetPosture {
            tag: "GER".into(),
            enemy: "FRA".into(),
            attack: true,
        });
        let ger = index(s.world(), "GER");
        let (mut km, mut air, mut n) = (0.0f64, 0.0f64, 0.0f64);
        for _ in 0..8 {
            week(&mut s);
            for g in &s.world().war.fighting.last_week {
                let (sign, i) = if g.a == ger { (1.0, 0) } else { (-1.0, 1) };
                km += sign * g.outcome.advance_km.to_f64_lossy();
                air += g.outcome.terms[i].air.to_f64_lossy();
                n += 1.0;
            }
        }
        (km / n.max(1.0), air / n.max(1.0))
    };
    let (with, air_with) = run(false);
    let (without, air_without) = run(true);
    eprintln!(
        "German advance per segment-week: {with:.2} km against the French air arm (German air \
         term {air_with:.3}), {without:.2} km with it grounded (air term {air_without:.3})"
    );
    assert!(
        air_with < air_without,
        "the French air arm holds the superiority"
    );
    assert!(with <= without, "and the Germans advance less under it");
}

/// A colonial expedition lands only by an open sea route: with the western Mediterranean held
/// by an enemy, Marseille–Algiers is cut and the expedition stops; the screen names the zone.
#[test]
fn the_expedition_needs_an_open_route() {
    let d = data();
    let w = d.new_world();
    let mut s = Session::new(w, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
    s.sim.run_until(Date::new(1925, 2, 3));
    let fra = index(s.world(), "FRA");
    let ita = index(s.world(), "ITA");
    let k = d
        .rules()
        .fight
        .as_ref()
        .and_then(|f| f.colonial.clone())
        .unwrap();
    // A colonial war of France in Algeria, and Italy at war with France holding the western
    // Mediterranean.
    let regions: Vec<u32> = s
        .world()
        .econ
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| k.candidates["DZA"].contains(&r.state) && r.market == fra)
        .map(|(i, _)| i as u32)
        .collect();
    let c = bf_sim::war::colonial::Colonial {
        owner: fra,
        colony: ita,
        unit: "DZA".into(),
        kept: regions.clone(),
        regions,
        ..Default::default()
    };
    let date = "1925-02-03";
    assert_eq!(
        bf_sim::war::colonial::route_cut(&k, s.world(), &c, date),
        None
    );
    {
        let w = s.sim.world_mut();
        w.war.conflicts.wars.push(bf_sim::war::WarOn {
            id: 0,
            side_a: vec![fra],
            side_b: vec![ita],
            crisis: u32::MAX,
            demand: bf_sim::war::colonial::union_demand("FRA", "ITA"),
            since: (1925, 2, 3),
        });
        let west = zone(d, "western_mediterranean");
        w.war.sea.control = vec![(west, ita)];
    }
    let cut = bf_sim::war::colonial::route_cut(&k, s.world(), &c, date).expect("cut");
    eprintln!("cut: {cut:?}");
    assert_eq!(cut.2, zone(d, "western_mediterranean"));
}

/// In peace, nothing at sea: the war's end clears fleets, control and blockades.
#[test]
fn the_sea_is_quiet_in_peace() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1925, 3, 2));
    assert!(s.world().war.sea.is_empty());
    assert!(bf_sim::war::fight::blockaded_pairs(s.world()).is_empty());
}
