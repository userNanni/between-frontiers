//! The "Balance of deterrence" panel's reading (`nuclear_view`): the player's own arsenal whole,
//! the other's untested one at the opacity of N3, and the ceilings of each.

use std::path::PathBuf;

use bf_client::nuclear_view;
use bf_sim::war::{Formation, Rung};
use laplace_schelling::Fixed;

#[test]
fn the_panel_reads_the_balance_as_the_player_does() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = bf_sim::data::GameData::load(&root).expect("game data");
    d.enable_crisis();
    let mil = d.war.as_ref().unwrap().military.clone();
    let mut w = d.new_world();
    let ix = |w: &bf_sim::game::GameWorld, t: &str| {
        w.countries.iter().position(|(_, c)| c.tag == t).unwrap() as u16
    };
    let (usa, sov) = (ix(&w, "USA"), ix(&w, "SOV"));
    assert!(
        nuclear_view::read(&w, &mil, usa, sov).is_none(),
        "nobody armed in 1925"
    );
    for (tag, key, n) in [
        ("USA", "nuclear_warhead", 600),
        ("USA", "strategic_bomber", 300),
        ("SOV", "nuclear_warhead", 40),
        ("SOV", "strategic_bomber", 20),
    ] {
        let c = ix(&w, tag) as usize;
        w.war.countries[c].formations.push(Formation {
            unit: mil.unit(key).unwrap(),
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
    let p = nuclear_view::read(&w, &mil, usa, sov).expect("a balance");
    assert_eq!(p.sides[0].tag, "USA");
    assert!(!p.sides[0].opaque && p.sides[1].opaque);
    assert_eq!(p.sides[0].warheads, 600);
    assert_eq!(
        p.sides[1].warheads, 20,
        "the Soviet arsenal, untested, at half"
    );
    assert!(p.sides[0].secure && !p.sides[1].secure);
    assert_eq!(p.sides[0].ceiling, Rung::NuclearUse);
    assert_eq!(p.sides[1].ceiling, Rung::Crisis);
    assert!(!p.mutual);
}
