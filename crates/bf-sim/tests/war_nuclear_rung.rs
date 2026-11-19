//! The nuclear rung on the crisis ladder (`docs/war.pt-BR.md`, "Arma nuclear na escada de
//! crise", step 3; the owner's N3, N4 and N9): the test makes possession known, a one-sided
//! secure arsenal coerces on peripheral demands, a ceiling below war blocks the declaration, and
//! the umbrella covers the ally.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use bf_sim::war::balance::balance;
use bf_sim::war::{Crisis, Formation, Stage, weigh_crisis};
use laplace_schelling::{Date, Fixed};

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

/// Puts `n` elements of `key` in `tag`'s forces.
fn give(w: &mut GameWorld, tag: &str, key: &str, n: i64) {
    let ty = mil().unit(key).unwrap();
    let c = index(w, tag) as usize;
    w.war.countries[c].formations.push(Formation {
        unit: ty,
        // Thermonuclear warheads (generation 1, U3), as the arsenals that deter; the rest
        // has one generation here.
        generation: u8::from(key == "nuclear_warhead"),
        elements: Fixed::from_int(n),
        colonial: false,
        called_up: false,
        equipment: Vec::new(),
        training: Fixed::ONE,
        raised: true,
    });
}

/// A secure arsenal: warheads on missile submarines, half at sea.
fn arm(w: &mut GameWorld, tag: &str) {
    give(w, tag, "nuclear_warhead", 3000);
    give(w, tag, "missile_submarine", 80);
}

fn demand(by: &str, target: &str, clauses: Vec<ClauseKind>) -> Proposal {
    Proposal {
        side_a: vec![by.into()],
        side_b: vec![target.into()],
        clauses: clauses
            .into_iter()
            .map(|kind| Clause {
                kind,
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            })
            .collect(),
        term_years: None,
        notice_years: 0,
    }
}

/// N3: an arsenal never tested counts half for the others; the test makes it known whole.
#[test]
fn the_test_makes_the_arsenal_known() {
    let d = data();
    let mut world = d.new_world();
    arm(&mut world, "USA");
    let mut s = Session::new(world, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
    let (usa, sov) = (index(s.world(), "USA"), index(s.world(), "SOV"));
    let before = balance(mil(), s.world(), usa, sov, Some(sov)).unwrap();
    assert!(before.sides[0].opaque);
    s.submit(GameCommand::NuclearTest { tag: "USA".into() });
    // The test is checked at the next week of the forces.
    s.sim.run_until(Date::new(1925, 1, 13));
    let w = s.world();
    assert!(w.war.nuclear.tested(usa));
    let after = balance(mil(), w, usa, sov, Some(sov)).unwrap();
    assert!(!after.sides[0].opaque, "known after the test");
    assert_eq!(after.sides[0].warheads, 2 * before.sides[0].warheads);
}

/// A test needs a warhead: without one, nothing happens.
#[test]
fn no_test_without_a_warhead() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::NuclearTest { tag: "USA".into() });
    s.sim.run_until(Date::new(1925, 1, 13));
    assert!(!s.world().war.nuclear.tested(index(s.world(), "USA")));
}

fn crisis(w: &GameWorld, by: &str, target: &str, demand: Proposal) -> Crisis {
    Crisis {
        id: 0,
        by: index(w, by),
        target: index(w, target),
        demand,
        weeks_left: 4,
        stage: Stage::Waiting,
        casus_belli: false,
    }
}

/// N4 with N10: against a one-sided secure arsenal, the demander's nuclear advantage adds to the
/// ratio of force and the target yields a peripheral demand; an existential demand never.
#[test]
fn coercion_weighs_on_the_periphery_only() {
    let d = data();
    let setup = d.rules().crisis.clone().unwrap();
    let mut w = d.new_world();
    let small = demand(
        "GER",
        "POL",
        vec![ClauseKind::Payment {
            amount: Fixed::from_int(1_000_000),
        }],
    );
    let k = crisis(&w, "GER", "POL", small.clone());
    let plain = weigh_crisis(&setup, mil(), &w, &k);
    arm(&mut w, "GER");
    let coerced = weigh_crisis(&setup, mil(), &w, &k);
    eprintln!(
        "ratio {} needed {:?}: plain {} yields {}; coerced term {} yields {}",
        plain.ratio, plain.needed, plain.nuclear, plain.yields, coerced.nuclear, coerced.yields
    );
    assert!(coerced.coerced);
    assert_eq!(plain.nuclear, 0);
    assert!(coerced.nuclear > 0);
    assert!(!plain.yields && coerced.yields);
    let whole = crisis(
        &w,
        "GER",
        "POL",
        demand("GER", "POL", vec![ClauseKind::Union]),
    );
    let all = weigh_crisis(&setup, mil(), &w, &whole);
    assert!(
        all.needed.is_none() && !all.yields,
        "the existential never yields"
    );
}

/// Hungary, played, demands Czechoslovakia whole and hears the refusal.
fn refused(world: GameWorld) -> Session {
    let d = data();
    let mut s = Session::new(world, 1925, Date::new(1925, 1, 1), 2, Some(d.rules()));
    s.submit(GameCommand::ChoosePlayer { tag: "HUN".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: demand("HUN", "CZE", vec![ClauseKind::Union]),
        weeks: None,
    });
    s.sim.run_until(Date::new(1925, 3, 20));
    s
}

fn declare(s: &mut Session) {
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
}

/// A ceiling below war blocks the declaration: Czechoslovakia holds a secure second strike and
/// Hungary none, so Hungary cannot go to war; without the arsenal it can.
#[test]
fn a_secure_target_blocks_the_declaration() {
    let d = data();
    let mut s = refused(d.new_world());
    declare(&mut s);
    assert_eq!(
        s.world().war.conflicts.wars.len(),
        1,
        "war without the arsenal"
    );
    let mut w = d.new_world();
    arm(&mut w, "CZE");
    let mut s = refused(w);
    let (hun, cze) = (index(s.world(), "HUN"), index(s.world(), "CZE"));
    let b = balance(mil(), s.world(), hun, cze, Some(hun)).unwrap();
    assert!(b.limits.ceiling[0] < bf_sim::war::Rung::War);
    declare(&mut s);
    assert!(
        s.world().war.conflicts.wars.is_empty(),
        "the declaration is blocked"
    );
}

/// N9 in the crisis: France, bound to defend Czechoslovakia and holding a secure arsenal, lowers
/// Hungary's ceiling against Czechoslovakia below war.
#[test]
fn the_umbrella_blocks_the_declaration() {
    let d = data();
    let mut w = d.new_world();
    arm(&mut w, "FRA");
    let mut s = refused(w);
    declare(&mut s);
    assert!(
        s.world().war.conflicts.wars.is_empty(),
        "the umbrella holds"
    );
}
