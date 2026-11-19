//! The nuclear balance read from the forces (`docs/war.pt-BR.md`, "Arma nuclear na escada de
//! crise", step 1): read only, on the 1925 world with arsenals put in by hand.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::GameWorld;
use bf_sim::war::balance::balance;
use bf_sim::war::{Formation, Rung};
use laplace_schelling::Fixed;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
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

/// Without a warhead anywhere, there is no balance to show (the 1925 world).
#[test]
fn no_warhead_no_balance() {
    let w = data().new_world();
    let (usa, sov) = (index(&w, "USA"), index(&w, "SOV"));
    assert!(balance(mil(), &w, usa, sov, None).is_none());
}

/// A secure second strike on one side only: it can climb to use, the other stops at crisis.
/// The Soviet Union reads the American arsenal, untested, at half (N3): what is secure for the
/// United States is not secure as Moscow reads it.
#[test]
fn one_sided_deterrence_and_the_opaque_arsenal() {
    let mut w = data().new_world();
    give(&mut w, "USA", "nuclear_warhead", 600);
    give(&mut w, "USA", "strategic_bomber", 300);
    give(&mut w, "SOV", "nuclear_warhead", 5);
    give(&mut w, "SOV", "strategic_bomber", 5);
    let (usa, sov) = (index(&w, "USA"), index(&w, "SOV"));
    let own = balance(mil(), &w, usa, sov, Some(usa)).unwrap();
    eprintln!(
        "as Washington reads it: delivered {:?} against {:?} unacceptable; ceilings {:?}",
        own.deterrence.second_strike[0].delivered, own.sides[1].unacceptable, own.limits.ceiling
    );
    assert!(own.deterrence.secure[0], "the United States is secure");
    assert!(!own.deterrence.secure[1], "the Soviet Union is not");
    assert_eq!(own.limits.ceiling, [Rung::NuclearUse, Rung::Crisis]);
    let theirs = balance(mil(), &w, usa, sov, Some(sov)).unwrap();
    eprintln!(
        "as Moscow reads it: {} warheads, delivered {:?}",
        theirs.sides[0].warheads, theirs.deterrence.second_strike[0].delivered
    );
    assert!(theirs.sides[0].opaque && !own.sides[0].opaque);
    assert_eq!(theirs.sides[0].warheads, 300, "half, untested (N3)");
    assert!(
        theirs.deterrence.second_strike[0].delivered < own.deterrence.second_strike[0].delivered
    );
}

/// Both secure: the balance of terror. War stays open with its risk per week; coercion has no
/// credit for either. Bombers on the ground give no second strike (Wohlstetter): the
/// submarines at sea do.
#[test]
fn mutual_deterrence_carries_the_risk() {
    let mut w = data().new_world();
    for tag in ["USA", "SOV"] {
        give(&mut w, tag, "nuclear_warhead", 3000);
        give(&mut w, tag, "strategic_bomber", 1500);
    }
    let (usa, sov) = (index(&w, "USA"), index(&w, "SOV"));
    let b = balance(mil(), &w, usa, sov, None).unwrap();
    assert_eq!(
        b.deterrence.secure,
        [false, false],
        "bombers on the ground die first"
    );
    for tag in ["USA", "SOV"] {
        give(&mut w, tag, "missile_submarine", 80);
    }
    let (usa, sov) = (index(&w, "USA"), index(&w, "SOV"));
    let b = balance(mil(), &w, usa, sov, None).unwrap();
    assert!(b.deterrence.mutual, "{:?}", b.deterrence.secure);
    assert_eq!(b.limits.ceiling, [Rung::War, Rung::War]);
    assert_eq!(
        b.limits.risk_per_week,
        mil().rules.nuclear.escalation_risk_per_week
    );
}

/// The umbrella (N9): France, bound by its 1925 treaty to defend Czechoslovakia, adds its
/// arsenal to the Czechoslovak side against Hungary. Without the umbrella, Czechoslovakia stands
/// alone and there is no balance.
#[test]
fn the_umbrella_covers_the_ally() {
    let mut w = data().new_world();
    give(&mut w, "FRA", "nuclear_warhead", 100);
    give(&mut w, "FRA", "strategic_bomber", 50);
    let (hun, cze, fra) = (index(&w, "HUN"), index(&w, "CZE"), index(&w, "FRA"));
    let b = balance(mil(), &w, hun, cze, None).unwrap();
    eprintln!(
        "Czechoslovakia under the umbrella: protectors {:?}, {} warheads; ceilings {:?}",
        b.sides[1].protectors, b.sides[1].warheads, b.limits.ceiling
    );
    assert!(b.sides[1].protectors.contains(&fra));
    assert!(b.sides[1].warheads > 0);
    let mut alone = mil().clone();
    alone.rules.nuclear.umbrella = false;
    assert!(balance(&alone, &w, hun, cze, None).is_none());
}
