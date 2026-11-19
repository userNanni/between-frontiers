//! The conferences of the nuclear ladder (`docs/war.pt-BR.md`, "Conferências (passo 5)"): the
//! forces of October 1962 and of 1950 put on the 1925 map, and what the rules make of them.
//! Measures, not targets: each case prints what it reads and asserts only what the rules fix.
//!
//! Sources of the forces (Norris, "The Cuban Missile Crisis: A Nuclear Order of Battle", Wilson
//! Center, 2012-10-24, the open version of the Nuclear Notebook, Bulletin 68(6), 2012):
//! - United States: about 3,500 warheads ready (p. 15); 182 ICBMs (p. 15); 7 Polaris submarines
//!   with 112 missiles on patrol (p. 17); 1,479 bombers ready on 4 November (p. 22).
//! - Soviet Union: about 300 warheads ready (p. 15); 42 ICBMs (p. 25, after Karlov); some 160
//!   heavy bombers, Tu-95 and 3M (p. 27); the missile submarines played no part (p. 25).
//! - 1950: 369 United States warheads and 5 Soviet (Norris and Kristensen, 2010, the yearly
//!   table); no missiles; the bombers able to deliver are not sourced and the case gives the
//!   warheads one bomber per two (an estimate).

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::GameWorld;
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use bf_sim::war::balance::balance;
use bf_sim::war::{Crisis, Formation, Stage, weigh_crisis};
use laplace_schelling::Fixed;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.war = true;
        d.switches.treaties_in_force = true;
        d
    })
}

fn mil() -> &'static bf_sim::war::Military {
    &data().war.as_ref().unwrap().military
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

fn give(w: &mut GameWorld, tag: &str, key: &str, n: i64, generation: u8) {
    if n == 0 {
        return;
    }
    let ty = mil().unit(key).unwrap();
    let c = index(w, tag) as usize;
    w.war.countries[c].formations.push(Formation {
        unit: ty,
        generation,
        elements: Fixed::from_int(n),
        colonial: false,
        called_up: false,
        equipment: Vec::new(),
        training: Fixed::ONE,
        raised: true,
    });
}

/// (warheads ready, ICBMs, missile submarines, bombers).
type Forces = (i64, i64, i64, i64);

/// The warheads of 1962 are thermonuclear (generation 1, U3), those of 1950 fission.
fn arm(w: &mut GameWorld, tag: &str, (warheads, icbm, subs, bombers): Forces) {
    let generation = u8::from(icbm > 0 || subs > 0 || warheads > 1000);
    give(w, tag, "nuclear_warhead", warheads, generation);
    give(w, tag, "ballistic_missile", icbm, 0);
    give(w, tag, "missile_submarine", subs, 0);
    give(w, tag, "strategic_bomber", bombers, 0);
}

fn test_both(w: &mut GameWorld, tags: &[&str]) {
    for t in tags {
        let c = index(w, t);
        if let Err(i) = w.war.nuclear.tested.binary_search(&c) {
            w.war.nuclear.tested.insert(i, c);
        }
    }
}

fn report(w: &GameWorld, a: &str, b: &str) -> bf_sim::war::balance::Balance {
    let (x, y) = (index(w, a), index(w, b));
    let bal = balance(mil(), w, x, y, Some(x)).unwrap();
    for (i, s) in bal.sides.iter().enumerate() {
        eprintln!(
            "CONF {}: warheads {} protectors {:?} | delivered after the other's first strike {:.0} of {:.0} unacceptable to the other | secure {} | first-strike incentive {} | ceiling {:?}",
            [a, b][i],
            s.warheads,
            s.protectors,
            bal.deterrence.second_strike[i].delivered.to_f64_lossy(),
            bal.sides[1 - i].unacceptable.to_f64_lossy(),
            bal.deterrence.secure[i],
            bal.deterrence.first_strike_incentive[i],
            bal.limits.ceiling[i],
        );
    }
    eprintln!(
        "CONF mutual {} risk per week {}",
        bal.deterrence.mutual, bal.limits.risk_per_week
    );
    bal
}

/// October 1962: the balance, the Soviet answer to a peripheral demand, and whether the United
/// States can declare.
#[test]
fn cuba_1962() {
    let d = data();
    let mut w = d.new_world();
    arm(&mut w, "USA", (3500, 182, 7, 1479));
    arm(&mut w, "SOV", (300, 42, 0, 160));
    test_both(&mut w, &["USA", "SOV"]);
    eprintln!("CONF ==== October 1962");
    let b = report(&w, "USA", "SOV");
    // The demand: a peripheral one (the missiles out of Cuba), as a small payment.
    let setup = d.rules().crisis.clone().unwrap();
    let k = Crisis {
        id: 0,
        by: index(&w, "USA"),
        target: index(&w, "SOV"),
        demand: Proposal {
            side_a: vec!["USA".into()],
            side_b: vec!["SOV".into()],
            clauses: vec![Clause {
                kind: ClauseKind::Payment {
                    amount: Fixed::from_int(1_000_000),
                },
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
        weeks_left: 4,
        stage: Stage::Waiting,
        casus_belli: false,
    };
    let wg = weigh_crisis(&setup, mil(), &w, &k);
    eprintln!(
        "CONF the Soviet weighing: ratio {} needed {:?} yields {} coerced {}",
        wg.ratio, wg.needed, wg.yields, wg.coerced
    );
    eprintln!(
        "CONF the United States may declare: {}",
        b.limits.ceiling[0] >= bf_sim::war::Rung::War
    );
    // With the bombers on airborne alert, as in the crisis (an eighth; doctrine to come).
    let mut alert = mil().clone();
    alert.rules.nuclear.bomber_alert = 125;
    let ba = balance(&alert, &w, index(&w, "USA"), index(&w, "SOV"), None).unwrap();
    eprintln!(
        "CONF with an eighth of the bombers on alert: US delivered {:.0}, secure {} | SOV secure {} | US ceiling {:?}",
        ba.deterrence.second_strike[0].delivered.to_f64_lossy(),
        ba.deterrence.secure[0],
        ba.deterrence.secure[1],
        ba.limits.ceiling[0]
    );
}

/// Korea, 1950–53: the United States holds the weapon, China none, the Soviet Union a handful;
/// at war with China, the rules of use do not make the United States strike first (N7: it is
/// not occupied).
#[test]
fn korea_1950() {
    let d = data();
    let mut w = d.new_world();
    arm(&mut w, "USA", (369, 0, 0, 185));
    arm(&mut w, "SOV", (5, 0, 0, 3));
    test_both(&mut w, &["USA", "SOV"]);
    eprintln!("CONF ==== 1950");
    report(&w, "USA", "CHN");
    report(&w, "USA", "SOV");
    let (usa, chi) = (index(&w, "USA"), index(&w, "CHN"));
    w.war.conflicts.wars.push(bf_sim::war::crisis::War {
        id: 0,
        side_a: vec![chi],
        side_b: vec![usa],
        crisis: 0,
        demand: Proposal {
            side_a: vec!["CHN".into()],
            side_b: vec!["USA".into()],
            clauses: Vec::new(),
            term_years: None,
            notice_years: 0,
        },
        since: (1950, 11, 25),
    });
    for taboo in [0, 200, 1000] {
        w.war.nuclear.taboo = taboo;
        let uses = bf_sim::war::nuclear_use::first_uses(mil(), &w, None);
        eprintln!("CONF taboo {taboo}: first uses {uses:?}");
        assert!(uses.is_empty(), "the United States is not occupied");
    }
}

/// A country's surface power at sea (gun and torpedo of its ships afloat), as the war at sea
/// reads it (`war::sea`): who holds the sea of an overseas theatre.
fn surface(w: &GameWorld, tag: &str) -> f64 {
    let c = index(w, tag) as usize;
    w.war.countries[c]
        .formations
        .iter()
        .filter(|f| mil().types[f.unit as usize].domain == bf_sim::war::Domain::Sea)
        .map(|f| {
            let n = f.elements.to_int_floor().max(0) as u64;
            let g = mil().ship_group(f.unit, f.generation as usize, n);
            if g.submerged {
                0.0
            } else {
                ((g.gun + g.torpedo) * g.count as i64).to_f64_lossy()
            }
        })
        .sum()
}

/// A peripheral demand of `by` on `target` (a small payment).
fn peripheral(w: &GameWorld, by: &str, target: &str) -> Crisis {
    Crisis {
        id: 0,
        by: index(w, by),
        target: index(w, target),
        demand: Proposal {
            side_a: vec![by.into()],
            side_b: vec![target.into()],
            clauses: vec![Clause {
                kind: ClauseKind::Payment {
                    amount: Fixed::from_int(1_000_000),
                },
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
        weeks_left: 4,
        stage: Stage::Waiting,
        casus_belli: false,
    }
}

/// N10 (the owner, 2026-11-19: (a), symmetric): on a peripheral demand, the demander's one-sided
/// nuclear advantage adds to the ratio of land force and the target's subtracts. Cuba 1962: the
/// Soviet Union yields; Berlin 1961: the United States does not, and the Soviet Union cannot
/// declare; Korea 1950: China does not yield.
///
/// The inputs: the land armies and navies are the game's of 1925, not those of the dates; Berlin
/// 1961 takes the 1962 arsenals (an estimate: 1961 has no source here); the bombers of 1950 are
/// an estimate. The theatre variant (b), measured and left for a later slice, is printed: the
/// side that holds the sea of an overseas theatre brings its land forces, the other none.
#[test]
fn n10_coercion_and_theatre() {
    let d = data();
    let setup = d.rules().crisis.clone().unwrap();
    struct Case {
        name: &'static str,
        by: &'static str,
        target: &'static str,
        forces: [(&'static str, Forces); 2],
        yields: bool,
        may_declare: bool,
    }
    let cases = [
        Case {
            name: "Cuba 1962",
            by: "USA",
            target: "SOV",
            forces: [("USA", (3500, 182, 7, 1479)), ("SOV", (300, 42, 0, 160))],
            yields: true,
            may_declare: true,
        },
        Case {
            name: "Berlin 1961 (1962 arsenals, estimate)",
            by: "SOV",
            target: "USA",
            forces: [("USA", (3500, 182, 7, 1479)), ("SOV", (300, 42, 0, 160))],
            yields: false,
            may_declare: false,
        },
        Case {
            name: "Korea 1950",
            by: "USA",
            target: "CHN",
            forces: [("USA", (369, 0, 0, 185)), ("SOV", (5, 0, 0, 3))],
            yields: false,
            may_declare: true,
        },
    ];
    eprintln!(
        "N10 case | land ratio | nuclear term | yields | demander may declare | sea by / target"
    );
    for c in &cases {
        let mut w = d.new_world();
        for (t, f) in c.forces {
            arm(&mut w, t, f);
        }
        test_both(&mut w, &["USA", "SOV"]);
        let k = peripheral(&w, c.by, c.target);
        let wg = weigh_crisis(&setup, mil(), &w, &k);
        let b = balance(mil(), &w, k.by, k.target, Some(k.by));
        let declare = b
            .as_ref()
            .is_none_or(|b| b.limits.ceiling[0] >= bf_sim::war::Rung::War);
        eprintln!(
            "N10 {} ({} on {}) | {} | {} | {} | {} | {:.0} / {:.0}",
            c.name,
            c.by,
            c.target,
            wg.ratio,
            wg.nuclear,
            wg.yields,
            declare,
            surface(&w, c.by),
            surface(&w, c.target)
        );
        assert_eq!(wg.yields, c.yields, "{}: {wg:?}", c.name);
        assert_eq!(declare, c.may_declare, "{}", c.name);
    }
}

/// U3, the deterrence check with real arsenals (a measure for the PR): what each side delivers
/// after the other's first strike, in equivalent megatons, the dead by the rule (370,000 an
/// equivalent megaton), against a quarter of the other's people. With the forces of each date as
/// in the sources above; 1955 is left out, its launchers have no source here. Also the 3,000
/// fission warheads of the earlier tests, to show why they no longer deter.
#[test]
fn deterrence_with_real_arsenals() {
    let d = data();
    let n = &mil().rules.nuclear;
    let ty = mil().unit("nuclear_warhead").unwrap();
    struct Case {
        name: &'static str,
        by: &'static str,
        on: &'static str,
        forces: [(&'static str, Forces); 2],
        generation: Option<u8>,
    }
    let cases = [
        Case {
            name: "US 1950 (bombers an estimate)",
            by: "USA",
            on: "SOV",
            forces: [("USA", (369, 0, 0, 185)), ("SOV", (5, 0, 0, 3))],
            generation: None,
        },
        Case {
            name: "US 1962",
            by: "USA",
            on: "SOV",
            forces: [("USA", (3500, 182, 7, 1479)), ("SOV", (300, 42, 0, 160))],
            generation: None,
        },
        Case {
            name: "USSR 1962",
            by: "SOV",
            on: "USA",
            forces: [("USA", (3500, 182, 7, 1479)), ("SOV", (300, 42, 0, 160))],
            generation: None,
        },
        Case {
            name: "3,000 fission warheads on 1,500 bombers (the earlier tests)",
            by: "USA",
            on: "SOV",
            forces: [("USA", (3000, 0, 0, 1500)), ("SOV", (300, 42, 0, 160))],
            generation: Some(0),
        },
    ];
    eprintln!(
        "U3 case | warheads | delivered after the other's first strike | EMT each, delivered | dead by the rule (capped at the people) | a quarter of the target | deters"
    );
    for c in &cases {
        let mut w = d.new_world();
        for (t, f) in c.forces {
            arm(&mut w, t, f);
        }
        if let Some(g) = c.generation {
            let by = index(&w, c.by) as usize;
            for f in w.war.countries[by]
                .formations
                .iter_mut()
                .filter(|f| f.unit == ty)
            {
                f.generation = g;
            }
        }
        test_both(&mut w, &["USA", "SOV"]);
        let (a, b) = (index(&w, c.by), index(&w, c.on));
        let bal = balance(mil(), &w, a, b, Some(a)).unwrap();
        let delivered = bal.deterrence.second_strike[0].delivered.to_f64_lossy();
        let (sum, count) = bf_sim::war::balance::warhead_emt(mil(), &w.war.countries[a as usize]);
        let emt = if count > 0 {
            sum as f64 / count as f64 / 1000.0
        } else {
            0.0
        };
        let dead_raw = delivered * emt * f64::from(n.killed_per_warhead);
        let people: i64 = w
            .econ
            .regions
            .iter()
            .filter(|r| r.market == b)
            .map(|r| r.population())
            .sum();
        // No attack kills more than the target's people.
        let dead = dead_raw.min(people as f64);
        let quarter = people as f64 * f64::from(n.unacceptable_share) / 1000.0;
        assert_eq!(bal.deterrence.secure[0], dead >= quarter, "{}", c.name);
        eprintln!(
            "U3 {} | {} | {:.0} | {:.3}, {:.1} | {:.1} M (uncapped {:.1} M) | {:.1} M | {}",
            c.name,
            bal.sides[0].warheads,
            delivered,
            emt,
            delivered * emt,
            dead / 1e6,
            dead_raw / 1e6,
            quarter / 1e6,
            bal.deterrence.secure[0]
        );
    }
}
