//! Properties the border operations (part B of `docs/borders.pt-BR.md`) must keep, written before
//! the operations against the design: a change of owner neither creates nor destroys people or
//! money ([`Totals`]), and a world where no operation ever runs keeps the hash of a world without
//! the border system. Until the operations exist, the tests check the measure itself.

use std::path::PathBuf;

use bf_sim::borders::Totals;
use bf_sim::data::GameData;
use laplace_schelling::Fixed;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The 1925 world has people and money everywhere the operations will split them.
#[test]
fn the_totals_of_1925_cover_every_ledger() {
    let data = GameData::load(&root()).unwrap();
    let world = data.new_world();
    let t = Totals::of(&world);
    let people: i64 = world.econ.regions.iter().map(|r| r.population()).sum();
    assert_eq!(t.people, people);
    assert!(t.people > 1_500_000_000, "{}", t.people);
    assert!(t.debt[0] > Fixed::ZERO && t.debt[1] > Fixed::ZERO);
    assert!(t.pool >= Fixed::ZERO);
}

/// Moving a region between markets by hand (what a transfer does first) keeps every total: the
/// ledgers that go with the state go with it, and nothing else moves. The operation will add the
/// splits of the market ledgers; this pins the part that needs no rule.
#[test]
fn moving_a_region_keeps_people_and_money() {
    let data = GameData::load(&root()).unwrap();
    let mut world = data.new_world();
    let before = Totals::of(&world);
    let from = world.econ.regions[0].market;
    let to = world
        .econ
        .regions
        .iter()
        .map(|r| r.market)
        .find(|&m| m != from)
        .unwrap();
    world.econ.regions[0].market = to;
    assert_eq!(Totals::of(&world), before);
}

/// A save written before the border state existed (the world ends where `borders` begins) loads,
/// with no border state.
#[test]
fn a_save_from_before_the_border_state_loads() {
    use laplace_schelling::save::encode_snapshot;
    let data = GameData::load(&root()).unwrap();
    let mut world = data.new_world();
    // A save of then had no treaty state either (the last field; empty, one byte).
    world.treaties = Default::default();
    let bytes = encode_snapshot(&world).unwrap();
    // An empty border state is four empty lists, one byte each, near the end of the snapshot.
    let old = &bytes[..bytes.len() - 5];
    let back = bf_sim::game::decode_world(old).unwrap();
    assert!(back.borders.is_empty());
    assert_eq!(back, world);
}
