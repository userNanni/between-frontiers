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

fn give(w: &mut GameWorld, tag: &str, key: &str, n: i64) {
    if n == 0 {
        return;
    }
    let ty = mil().unit(key).unwrap();
    let c = index(w, tag) as usize;
    w.war.countries[c].formations.push(Formation {
        unit: ty,
        generation: 0,
        elements: Fixed::from_int(n),
        colonial: false,
        called_up: false,
        equipment: Vec::new(),
        training: Fixed::ONE,
        raised: true,
    });
}

/// (warheads ready, ICBMs, missile submarines, bombers).
fn arm(w: &mut GameWorld, tag: &str, (warheads, icbm, subs, bombers): (i64, i64, i64, i64)) {
    give(w, tag, "nuclear_warhead", warheads);
    give(w, tag, "ballistic_missile", icbm);
    give(w, tag, "missile_submarine", subs);
    give(w, tag, "strategic_bomber", bombers);
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
