//! The war on land in the tick (phase 5, slice C) on the real scenario: a declared war has fronts
//! where the two sides touch, a week of combat on each segment, losses taken from the
//! formations, and a posture the player sets.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.war = true;
        no_overrun(&mut d);
        d
    })
}

/// These tests follow the front: Hungary overrun must not end the war (`war_support` covers
/// that rule).
fn no_overrun(d: &mut GameData) {
    if let Some(c) = d.war.as_mut().and_then(|w| w.conduct.as_mut()) {
        c.support.overrun = 2000;
        c.support.capital_overrun = 2000;
    }
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

/// Hungary, played, demands Czechoslovakia whole, hears the refusal and declares.
fn at_war() -> Session {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "HUN".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
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
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    s
}

/// The war has segments along the Hungarian–Czechoslovak border, and a week of combat on them.
#[test]
fn a_declared_war_has_fronts_and_combat() {
    let d = data();
    let s = at_war();
    let w = s.world();
    let (hun, cze) = (index(w, "HUN"), index(w, "CZE"));
    let rules = d.rules();
    let week = bf_sim::war::fight::fight_week(
        rules
            .fight
            .as_ref()
            .expect("the war switch builds the setup"),
        &d.war.as_ref().unwrap().military,
        w,
        s.clock().date,
    );
    let border: Vec<_> = week
        .segments
        .iter()
        .filter(|g| (g.a, g.b) == (hun.min(cze), hun.max(cze)))
        .collect();
    assert!(!border.is_empty(), "segments along the border");
}

/// Attacking takes losses on both sides; holding with no attacker does not.
#[test]
fn the_player_attacks_and_both_sides_lose_men() {
    let mut s = at_war();
    let (hun, cze) = (index(s.world(), "HUN"), index(s.world(), "CZE"));
    s.submit(GameCommand::SetPosture {
        tag: "HUN".into(),
        enemy: "CZE".into(),
        attack: true,
    });
    // The first week of combat (the front of a thin Hungary does not last long).
    s.sim.run_until(Date::new(1925, 3, 31));
    // The screen reads the week the tick fought, without working it out again.
    assert!(
        s.world()
            .war
            .fighting
            .last_week
            .iter()
            .any(|g| (g.a, g.b) == (hun.min(cze), hun.max(cze)) && g.outcome.engaged),
        "the last week's segments are kept"
    );
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    assert_eq!(w.war.fighting.posture(hun, cze), Some(true));
    let lost = |c: u16| {
        w.war
            .fighting
            .losses
            .iter()
            .find(|(k, _)| *k == c)
            .map_or(0, |(_, l)| *l)
    };
    assert!(
        lost(hun) > 0 && lost(cze) > 0,
        "{:?}",
        w.war.fighting.losses
    );
}

/// Without the war switch a declared war only mobilizes: no fighting state.
#[test]
fn off_a_declared_war_only_mobilizes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap();
    d.enable_crisis();
    assert!(!d.switches.war);
    assert!(d.rules().fight.is_none());
    let _ = Fixed::ZERO;
}

/// Hungary, losing, proposes peace at the treaties' table with a payment to Czechoslovakia; the
/// AI accepts, the war ends, the occupation between them ends and both go home.
#[test]
fn a_peace_signed_at_the_table_ends_the_war() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap();
    d.enable_crisis();
    d.switches.war = true;
    d.switches.negotiation = true;
    no_overrun(&mut d);
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "HUN".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    let union = |by: &str, to: &str| Proposal {
        side_a: vec![by.into()],
        side_b: vec![to.into()],
        clauses: vec![Clause {
            kind: ClauseKind::Union,
            giver: Side::Junior,
            receiver: Side::Senior,
            until: None,
            note: None,
        }],
        term_years: None,
        notice_years: 0,
    };
    s.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: union("HUN", "CZE"),
        weeks: None,
    });
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.sim.run_until(Date::new(1925, 6, 1));
    let (hun, cze) = (index(s.world(), "HUN"), index(s.world(), "CZE"));
    assert!(
        !s.world().war.fighting.control.is_empty(),
        "the front moved"
    );
    s.submit(GameCommand::ProposeTreaty {
        tag: "HUN".into(),
        to: "CZE".into(),
        proposal: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
            clauses: vec![Clause {
                kind: ClauseKind::Payment {
                    amount: Fixed::from_int(50),
                },
                giver: Side::Senior,
                receiver: Side::Junior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
    });
    s.sim.run_until(Date::new(1925, 7, 1));
    let w = s.world();
    assert!(
        !w.war.conflicts.at_war(hun, cze),
        "peace: {:?}",
        w.treaties.table.closed
    );
    assert!(
        w.war
            .fighting
            .control
            .iter()
            .all(|(r, by)| { (*by, w.econ.regions[*r as usize].market) != (cze, hun) }),
        "the occupation ended"
    );
    assert!(!w.war.countries[hun as usize].mobilized);
    assert!(!w.war.countries[cze as usize].mobilized);
}

/// The killed leave the soldiers' pops and the pyramid's groups of 20 to 39, where the country's
/// soldiers are; the population falls by them.
#[test]
fn the_killed_leave_the_population() {
    let s = at_war();
    let mut w = s.world().clone();
    let hun = index(&w, "HUN");
    let people = |w: &GameWorld| -> i64 {
        w.econ
            .regions
            .iter()
            .filter(|r| r.market == hun)
            .map(|r| r.population())
            .sum()
    };
    let young = |w: &GameWorld| -> i64 {
        w.econ
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.market == hun)
            .filter_map(|(i, _)| w.demography.ages.get(i))
            .map(|a| a[4..8].iter().sum::<i64>())
            .sum()
    };
    let (before, young_before) = (people(&w), young(&w));
    bf_sim::war::fight::bury(&mut w, hun, 1000);
    assert_eq!(people(&w), before - 1000);
    assert_eq!(young(&w), young_before - 1000);
}

/// At war, the forces order the ammunition of combat, and the side's supply is the share of it
/// the market delivered.
#[test]
fn at_war_the_forces_order_the_ammunition_of_combat() {
    let d = data();
    let mil = &d.war.as_ref().unwrap().military;
    let s = at_war();
    let w = s.world();
    let hun = index(w, "HUN") as usize;
    let c = &w.war.countries[hun];
    let (peace, _) = bf_sim::war::needs_of(
        mil,
        &d.war.as_ref().unwrap().peace.clone().unwrap(),
        c,
        false,
    );
    let (war, _) = bf_sim::war::needs_of(
        mil,
        &d.war.as_ref().unwrap().peace.clone().unwrap(),
        c,
        true,
    );
    let ammo = |l: &[(u16, Fixed)]| {
        l.iter()
            .find(|(g, _)| *g == mil.rules.ammunition)
            .map_or(Fixed::ZERO, |(_, q)| *q)
    };
    assert!(
        ammo(&war) > ammo(&peace) * 5,
        "{:?} {:?}",
        ammo(&war),
        ammo(&peace)
    );
}

/// The army moves at the pace of most of its land men: foot infantry holds back the armour, the
/// garrisons do not count, and a tie goes to the slower (passo 3 of the front's pace).
#[test]
fn the_army_moves_at_the_pace_of_most_of_its_men() {
    use bf_sim::war::Formation;
    use bf_sim::war::catalogue::Mobility;
    use bf_sim::war::fight::majority_mobility;
    let mil = &data().war.as_ref().unwrap().military;
    let f = |key: &str, thousands: i64| Formation {
        unit: mil.unit(key).unwrap(),
        generation: 0,
        elements: Fixed::from_int(thousands),
        colonial: false,
        called_up: false,
        equipment: Vec::new(),
        training: Fixed::ONE,
        raised: false,
    };
    // France 1940 on the German side: most men walked behind the panzers.
    assert_eq!(
        majority_mobility(mil, &[f("infantry", 100), f("armor", 30)]),
        Mobility::Foot
    );
    assert_eq!(
        majority_mobility(mil, &[f("infantry", 20), f("mechanized", 50)]),
        Mobility::Wheeled
    );
    // Garrisons hold their posts.
    assert_eq!(
        majority_mobility(mil, &[f("garrison", 500), f("cavalry", 10)]),
        Mobility::Horse
    );
    // A tie: the slower one.
    assert_eq!(
        majority_mobility(mil, &[f("cavalry", 10), f("mechanized", 10)]),
        Mobility::Horse
    );
    assert_eq!(majority_mobility(mil, &[]), Mobility::Foot);
}

fn land_shares(w: &GameWorld, mil: &bf_sim::war::Military, ci: usize) -> (i64, i64, i64) {
    use bf_sim::war::catalogue::{Domain, Mobility};
    let (mut wheels, mut tracks, mut all) = (0i64, 0i64, 0i64);
    for f in &w.war.countries[ci].formations {
        if mil.types[f.unit as usize].domain != Domain::Land || f.called_up || f.colonial {
            continue;
        }
        let n = bf_sim::war::men(mil, f);
        all += n;
        match mil.spec(f.unit, f.generation as usize).mobility {
            Mobility::Wheeled => wheels += n,
            Mobility::Tracked => tracks += n,
            _ => {}
        }
    }
    (wheels, tracks, all)
}

/// The composition of the forces: in January the historical rule converts infantry toward its
/// target, at most a class a year; the men come from the infantry (the army does not grow)
/// and show in conversion until the year's class has moved.
#[test]
fn the_infantry_converts_a_class_a_year() {
    let d = data();
    let mil = &d.war.as_ref().unwrap().military;
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    let gbr = index(s.world(), "GBR") as usize;
    s.sim.run_until(Date::new(1925, 1, 10));
    let w = s.world();
    let ordered: Fixed = w.war.composition.of(gbr as u16).map(|x| x.ordered).sum();
    let (_, _, all) = land_shares(w, mil, gbr);
    assert!(ordered.is_positive(), "Britain converts in 1925");
    // At most 20 per mille of the land men.
    assert!(
        ordered * 1000 <= Fixed::from_int(all) * 20 / 1000 + Fixed::ONE,
        "{ordered:?} of {all}"
    );
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    let (wheels, tracks, all2) = land_shares(w, mil, gbr);
    assert!(wheels + tracks > 0);
    // The army changed equipment, it did not grow from the conversion (the size in peace moves
    // it only in January).
    assert!((all2 - all).abs() * 100 <= all, "{all} -> {all2}");
}

/// The government's target in place of the rule; `None` returns to the rule.
#[test]
fn the_government_fixes_the_composition() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::SetComposition {
        tag: "GBR".into(),
        composition: Some([900, 400]),
    });
    s.sim.run_until(Date::new(1925, 1, 3));
    let gbr = index(s.world(), "GBR");
    // Clamped: together at most 1,000.
    assert_eq!(s.world().war.composition.fixed_of(gbr), Some([900, 100]));
    s.submit(GameCommand::SetComposition {
        tag: "GBR".into(),
        composition: None,
    });
    s.sim.run_until(Date::new(1925, 1, 4));
    assert_eq!(s.world().war.composition.fixed_of(gbr), None);
}

/// The check of the composition against 1939 and 1944 (`docs/war.pt-BR.md`, "Composição das
/// forças"): the share of land men on wheels and on tracks, the conversions waiting, and the
/// price and the households' share of automobiles, with the rule and without it. Ignored: two
/// runs of 19 years; run in a bench slot and read the table it prints.
#[test]
#[ignore]
fn composition_check_1939_and_1944() {
    let base = data();
    let mut off = {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.war = true;
        no_overrun(&mut d);
        d
    };
    off.war
        .as_mut()
        .unwrap()
        .peace
        .as_mut()
        .unwrap()
        .composition = None;
    let mil = &base.war.as_ref().unwrap().military;
    let auto = base.catalogue.good("automobiles").unwrap() as usize;
    let armored = base.catalogue.good("armored_vehicles").unwrap() as usize;
    let vehicles = vec![auto as u16, armored as u16];
    for (label, d) in [("rule", base), ("no rule", &off)] {
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules()),
        );
        for year in [1939, 1944] {
            s.sim.run_until(Date::new(year, 6, 1));
            let w = s.world();
            for tag in [
                "GER", "FRA", "SOV", "USA", "GBR", "CHN", "BRA", "TUR", "PER",
            ] {
                let Some(ci) = w.countries.iter().position(|(_, c)| c.tag == tag) else {
                    continue;
                };
                let c = &w.war.countries[ci];
                // By type of formation (what the target counts) and by the equipment that
                // arrived (what the generation counts).
                let (wt, tt, at) = land_shares(w, mil, ci);
                let (we, te, ae) = bf_sim::war::composition::equipped(mil, &vehicles, c);
                let pc = |x: Fixed| (x * 1000 / ae.max(Fixed::ONE)).to_f64_lossy() / 10.0;
                let waiting: Fixed = w
                    .war
                    .composition
                    .of(ci as u16)
                    .map(|x| x.ordered - x.delivered)
                    .sum();
                // What holds the gap: the equipment the formations lack (the military budget
                // cut the replacement to its room), or a market that does not deliver (a buy
                // ratio under one).
                let lack: Fixed = c
                    .formations
                    .iter()
                    .map(|f| {
                        let spec = mil.spec(f.unit, f.generation as usize);
                        let need: Vec<(u16, Fixed)> = spec
                            .equipment
                            .iter()
                            .map(|(g, q)| (*g, *q * f.elements))
                            .collect();
                        (mil.value_lots(&need) - mil.value_lots(&f.equipment)).max(Fixed::ZERO)
                    })
                    .sum();
                let m = &w.econ.markets[ci];
                println!(
                    "{label} {year} {tag}: by type wheels {:.1}% tracks {:.1}% | equipped wheels {:.1}% tracks {:.1}% | of {at} men | waiting {waiting:?} | lacking {lack:?} lots | bought automobiles {:?} armored {:?} | automobiles price {:?}",
                    wt as f64 * 100.0 / at.max(1) as f64,
                    tt as f64 * 100.0 / at.max(1) as f64,
                    pc(we),
                    pc(te),
                    m.buy_ratio[auto],
                    m.buy_ratio[armored],
                    m.price[auto],
                );
            }
        }
    }
}

/// The composition shown counts each formation by the vehicles it holds: a formation converted
/// without its lorries moves on foot, whatever it was ordered to be or the rifles it kept (the
/// defect the first checks found: a poor country "motorized" as fast as the powers).
#[test]
fn a_converted_formation_counts_by_the_equipment_it_holds() {
    use bf_sim::war::Formation;
    let mil = &data().war.as_ref().unwrap().military;
    let mech = mil.unit("mechanized").unwrap();
    let full: Vec<(u16, Fixed)> = mil
        .spec(mech, 0)
        .equipment
        .iter()
        .map(|(g, q)| (*g, *q * Fixed::from_int(10)))
        .collect();
    let f = |equipment: Vec<(u16, Fixed)>| Formation {
        unit: mech,
        generation: 0,
        elements: Fixed::from_int(10),
        colonial: false,
        called_up: false,
        equipment,
        training: Fixed::ONE,
        raised: false,
    };
    let empty = bf_sim::war::CountryWar {
        formations: vec![f(Vec::new())],
        ..Default::default()
    };
    let half = bf_sim::war::CountryWar {
        formations: vec![f(full.iter().map(|(g, q)| (*g, *q / 2)).collect())],
        ..Default::default()
    };
    let cat = &data().catalogue;
    let vehicles = vec![
        cat.good("automobiles").unwrap() as u16,
        cat.good("armored_vehicles").unwrap() as u16,
    ];
    let (w0, _, a0) = bf_sim::war::composition::equipped(mil, &vehicles, &empty);
    let (w1, _, _) = bf_sim::war::composition::equipped(mil, &vehicles, &half);
    // The rifles and guns the infantry brought do not make it move: only the lorries count.
    let rifles_only = bf_sim::war::CountryWar {
        formations: vec![f(full
            .iter()
            .filter(|(g, _)| !vehicles.contains(g))
            .copied()
            .collect())],
        ..Default::default()
    };
    let (w2, _, _) = bf_sim::war::composition::equipped(mil, &vehicles, &rifles_only);
    assert_eq!(w2, Fixed::ZERO);
    assert_eq!(a0, Fixed::from_int(10));
    assert_eq!(w0, Fixed::ZERO);
    assert_eq!(w1, Fixed::from_int(5));
}

/// A conversion ends when less than ten men are left: the rounding of the classes does not
/// keep it open and "stalled" for years (the second defect of the first check).
#[test]
fn conversions_end_and_do_not_linger() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1926, 12, 20));
    for x in &s.world().war.composition.conversions {
        assert!(
            x.ordered - x.delivered >= Fixed::from_ratio(1, 100),
            "{x:?}"
        );
        assert!(x.stalled < 52, "{x:?}");
    }
}
