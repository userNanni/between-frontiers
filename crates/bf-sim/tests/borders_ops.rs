//! The border operations (`borders::op`, part B of `docs/borders.pt-BR.md`) on the 1925 world:
//! each one moves what the design says, conserves people, money, debt, plants and the diaspora,
//! keeps every per-country table the same length, and refuses what it must. A world where no
//! operation runs keeps the hash it has without the border system; save, load and threads do not
//! change the outcome of a campaign with operations.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::borders::Totals;
use bf_sim::borders::op::{self, DebtTerm, Operation, Terms};
use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::research::Stage;
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

/// The world after a month of running, so that pops have incomes and the books are not zero.
fn world() -> GameWorld {
    static WORLD: OnceLock<GameWorld> = OnceLock::new();
    WORLD
        .get_or_init(|| {
            let d = data();
            let mut s = Session::new(
                d.new_world(),
                1925,
                Date::new(1925, 1, 1),
                2,
                Some(d.rules()),
            );
            s.sim.run_until(Date::new(1925, 2, 1));
            s.world().clone()
        })
        .clone()
}

fn ix(w: &GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
}

fn holds(w: &GameWorld, c: usize) -> usize {
    w.econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == c)
        .count()
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// Runs `ops` on the world directly (no tick in between) and checks the conservation.
fn run(w: &mut GameWorld, ops: Vec<Operation>) {
    let before = Totals::of(w);
    let done = w.borders.changes.len();
    w.borders.pending = ops;
    op::run_pending(w, &data().border_rules, Date::new(1925, 2, 1));
    let mut after = Totals::of(w);
    // People abroad whose home and host became one country are home, not lost.
    for c in &w.borders.changes[done..] {
        after.diaspora += c.repatriated.0;
        after.arrived += c.repatriated.1;
    }
    assert_eq!(after, before, "totals changed");
    assert!(op::tables_agree(w), "per-country tables disagree");
}

fn debt(w: &GameWorld, c: usize) -> Fixed {
    let y = &w.econ.markets[c].money;
    y.debt_home + y.debt_bank + y.debt_abroad + y.debt_fx
}

#[test]
fn a_colony_is_born_without_debt_and_with_capped_technique() {
    let mut w = world();
    let gbr = ix(&w, "GBR");
    let gbr_debt = debt(&w, gbr);
    let n = w.countries.len();
    run(
        &mut w,
        vec![Operation::Secede {
            from: "GBR".into(),
            states: s(&[
                "NGA-BENUE-NIGER",
                "NGA-BORNU-ADAMAWA",
                "NGA-EASTERN",
                "NGA-KANO-SOKOTO",
                "NGA-WESTERN",
            ]),
            tag: "NGA".into(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        }],
    );
    assert!(w.borders.refused.is_empty(), "{:?}", w.borders.refused);
    let nga = ix(&w, "NGA");
    assert_eq!(nga, n, "a new country goes to the end of the tables");
    assert_eq!(holds(&w, nga), 5);
    assert!(
        w.econ
            .regions
            .iter()
            .filter(|r| usize::from(r.market) == nga)
            .all(|r| !r.colony)
    );
    assert_eq!(debt(&w, gbr), gbr_debt, "the metropole keeps the debt");
    assert_eq!(debt(&w, nga), Fixed::ZERO);
    assert_eq!(w.econ.markets[nga].treasury, Fixed::ZERO);
    // Currency: the parent's numbers on the day.
    assert_eq!(
        w.econ.markets[nga].money.price_level,
        w.econ.markets[gbr].money.price_level
    );
    assert_eq!(
        w.econ.markets[nga].money.exchange_rate,
        w.econ.markets[gbr].money.exchange_rate
    );
    // Technique: capped at prototype unless a plant of the new country uses it.
    let r = &w.research;
    let above = (0..r.stage[gbr].len())
        .filter(|&t| r.stage[nga][t] > Stage::Prototype)
        .count();
    let gbr_above = (0..r.stage[gbr].len())
        .filter(|&t| r.stage[gbr][t] > Stage::Prototype)
        .count();
    assert!(above < gbr_above, "{above} of {gbr_above}");
    for t in 0..r.stage[gbr].len() {
        assert!(r.stage[nga][t] <= r.stage[gbr][t]);
    }
    // Neighbours stay inside each market.
    for reg in &w.econ.regions {
        for &j in &reg.neighbours {
            assert_eq!(w.econ.regions[j as usize].market, reg.market);
        }
    }
    // Independence won: the movement is gone in the new country.
    let rules = &data().border_rules;
    if let Some(i) = rules.independence {
        assert_eq!(w.politics.movements[nga][i].support, 0);
    }
    assert_eq!(w.borders.changes.len(), 1);
    assert_eq!(w.borders.changes[0].born, Some(nga as u16));
}

#[test]
fn a_metropole_secession_splits_the_debt() {
    let mut w = world();
    let sov = ix(&w, "SOV");
    let before = debt(&w, sov);
    run(
        &mut w,
        vec![Operation::Secede {
            from: "SOV".into(),
            states: s(&["UKR-05", "UKR-09", "UKR-12"]),
            tag: "UKR".into(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        }],
    );
    let ukr = ix(&w, "UKR");
    assert!(debt(&w, ukr) > Fixed::ZERO);
    assert!(debt(&w, sov) < before);
}

#[test]
fn annexation_leaves_a_tombstone_that_can_be_born_again() {
    let mut w = world();
    let aut = ix(&w, "AUT");
    let ger = ix(&w, "GER");
    let states: Vec<String> = data()
        .map
        .states
        .iter()
        .filter(|s| s.owner == "AUT")
        .map(|s| s.key.clone())
        .collect();
    let ger_debt = debt(&w, ger) + debt(&w, aut);
    run(
        &mut w,
        vec![Operation::Annex {
            from: "AUT".into(),
            into: "GER".into(),
            colony: false,
            terms: Terms::default(),
        }],
    );
    assert_eq!(holds(&w, aut), 0);
    assert_eq!(debt(&w, aut), Fixed::ZERO);
    assert_eq!(debt(&w, ger), ger_debt);
    assert_eq!(w.borders.changes[0].ended, Some(aut as u16));
    let n = w.countries.len();
    run(
        &mut w,
        vec![Operation::Secede {
            from: "GER".into(),
            states,
            tag: "AUT".into(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        }],
    );
    assert!(w.borders.refused.is_empty(), "{:?}", w.borders.refused);
    assert_eq!(w.countries.len(), n, "born again in its own place");
    assert!(holds(&w, aut) > 0);
}

#[test]
fn transfers_move_colonies_and_their_laws() {
    let mut w = world();
    let fra = ix(&w, "FRA");
    run(
        &mut w,
        vec![Operation::Transfer {
            from: "GBR".into(),
            to: "FRA".into(),
            states: s(&["GMB"]),
            colony: None,
            terms: Terms::default(),
        }],
    );
    assert!(w.borders.refused.is_empty(), "{:?}", w.borders.refused);
    let r = w
        .econ
        .regions
        .iter()
        .find(|r| {
            data()
                .map
                .states
                .iter()
                .any(|d| d.id == r.state && d.key == "GMB")
        })
        .unwrap();
    assert_eq!(usize::from(r.market), fra);
    assert!(r.colony);
}

#[test]
fn debt_taken_over_by_agreement_moves_and_conserves() {
    let mut w = world();
    let (fra, bel) = (ix(&w, "FRA"), ix(&w, "BEL"));
    let (f0, b0) = (debt(&w, fra), debt(&w, bel));
    let amount = Fixed::from_int(1_000_000_000);
    run(
        &mut w,
        vec![Operation::Terms {
            terms: Terms {
                debt: vec![DebtTerm::Market {
                    from: "FRA".into(),
                    to: "BEL".into(),
                    amount,
                }],
                techniques: Vec::new(),
            },
        }],
    );
    let moved = debt(&w, bel) - b0;
    assert_eq!(f0 - debt(&w, fra), moved);
    assert!((moved - amount).abs() < Fixed::from_int(1_000), "{moved:?}");
}

#[test]
fn the_player_frees_its_own_land_only() {
    let mut w = world();
    let bra = ix(&w, "BRA");
    w.borders.pending = vec![
        // Someone else's state, an unknown state, a state twice, every state: refused.
        Operation::Secede {
            from: "BRA".into(),
            states: s(&["ARG-BUENOS-AIRES"]),
            tag: String::new(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        },
        Operation::Secede {
            from: "BRA".into(),
            states: s(&["NOWHERE"]),
            tag: String::new(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        },
        Operation::Secede {
            from: "BRA".into(),
            states: s(&["BRA-SP", "BRA-SP"]),
            tag: String::new(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        },
        Operation::Secede {
            from: "BRA".into(),
            states: data()
                .map
                .states
                .iter()
                .filter(|s| s.owner == "BRA")
                .map(|s| s.key.clone())
                .collect(),
            tag: String::new(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        },
        Operation::Secede {
            from: "BRA".into(),
            states: s(&["BRA-RJ"]),
            tag: "ARG".into(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        },
        Operation::Status {
            tag: "BRA".into(),
            overlord: Some("BRA".into()),
        },
        // São Paulo as a subject of Brazil, with a generic tag.
        Operation::Secede {
            from: "BRA".into(),
            states: s(&["BRA-SP"]),
            tag: String::new(),
            overlord: Some("BRA".into()),
            signatories: Vec::new(),
            terms: Terms::default(),
        },
    ];
    op::run_pending(&mut w, &data().border_rules, Date::new(1925, 2, 1));
    assert_eq!(w.borders.refused.len(), 6, "{:#?}", w.borders.refused);
    assert_eq!(w.borders.changes.len(), 1);
    let sp = usize::from(w.borders.changes[0].born.unwrap());
    assert_eq!(w.countries.iter().nth(sp).unwrap().1.tag, "X00");
    assert_eq!(data().border_rules.overlord_of(&w, sp), Some(bra));
    // The 1925 subjects are read from the scenario until the first operation, and kept after.
    let raj = ix(&w, "RAJ");
    assert_eq!(
        data().border_rules.overlord_of(&w, raj),
        Some(ix(&w, "GBR"))
    );
}

fn campaign(threads: usize, save: bool) -> (laplace_schelling::hash::StateHash, GameWorld) {
    let d = data();
    let mut ses = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        threads,
        Some(d.rules()),
    );
    ses.submit(GameCommand::Liberate {
        by: "BRA".into(),
        states: s(&["BRA-SP"]),
        tag: None,
        subject: false,
    });
    ses.sim.run_until(Date::new(1925, 4, 1));
    ses.submit(GameCommand::Borders {
        op: Operation::Annex {
            from: "AUT".into(),
            into: "GER".into(),
            colony: false,
            terms: Terms::default(),
        },
    });
    ses.sim.run_until(Date::new(1925, 8, 1));
    if save {
        let bytes = ses.save("test", "mid").unwrap();
        ses = Session::load(&bytes, threads, Some(d.rules())).unwrap();
    }
    ses.submit(GameCommand::Borders {
        op: Operation::Secede {
            from: "GBR".into(),
            states: s(&["NGA-WESTERN", "NGA-EASTERN"]),
            tag: "NGA".into(),
            overlord: None,
            signatories: Vec::new(),
            terms: Terms::default(),
        },
    });
    ses.sim.run_until(Date::new(1926, 3, 1));
    (ses.state_hash(), ses.world().clone())
}

#[test]
fn a_campaign_with_operations_is_deterministic() {
    let (one, w) = campaign(1, false);
    let (two, _) = campaign(2, false);
    let (loaded, _) = campaign(2, true);
    assert_eq!(one, two, "1 and 2 threads");
    assert_eq!(one, loaded, "saved and loaded in the middle");
    assert_eq!(w.borders.changes.len(), 3, "{:?}", w.borders.refused);
    assert!(op::tables_agree(&w));
    assert!(
        w.econ
            .regions
            .iter()
            .flat_map(|r| &r.pops)
            .all(|p| p.size >= 0)
    );
}

#[test]
fn no_operation_keeps_the_hash_of_a_world_without_the_border_system() {
    let d = data();
    let mut rules = d.rules();
    let mut with = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(rules.clone()),
    );
    rules.borders = None;
    let mut without = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    with.sim.run_until(Date::new(1925, 7, 1));
    without.sim.run_until(Date::new(1925, 7, 1));
    assert_eq!(with.state_hash(), without.state_hash());
}
