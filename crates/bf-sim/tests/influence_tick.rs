//! Influence in the tick (`docs/influence.pt-BR.md`, etapa 2): the monthly map holds only the
//! pairs of the 1925 war debts; with `proposal_weight` zero nobody holds influence and the world
//! keeps the hash it has without the system; saves from before the influence state load.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use bf_sim::data::GameData;
use bf_sim::game::Session;
use laplace_schelling::Date;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn run(weight: Option<u32>) -> Session {
    let d = data();
    let mut rules = d.rules();
    match weight {
        None => rules.influence = None,
        Some(w) => {
            let mut setup = (*rules.influence.clone().unwrap()).clone();
            setup.rules.proposal_weight = w;
            rules.influence = Some(Arc::new(setup));
        }
    }
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(1925, 4, 1));
    s
}

#[test]
fn with_weight_zero_the_hash_is_the_one_without_influence() {
    let without = run(None);
    let zero = run(Some(0));
    assert!(zero.world().influence.is_empty());
    assert_eq!(zero.state_hash(), without.state_hash());
}

#[test]
fn with_the_weight_the_map_holds_only_the_war_debt_pairs() {
    let s = run(Some(1000));
    let w = s.world();
    assert!(!w.influence.map.is_empty());
    for (a, b) in w.influence.map.keys() {
        assert!(
            w.econ
                .ledger
                .debts
                .iter()
                .any(|d| d.creditor == *a && d.debtor == *b),
            "{a} over {b} is not a creditor over a debtor"
        );
    }
    // Save and load in the middle: the same outcome.
    let bytes = s.save("test", "").unwrap();
    let loaded = Session::load(&bytes, 1, Some(data().rules())).unwrap();
    assert_eq!(loaded.state_hash(), s.state_hash());
}

/// A world saved before the influence state (it ends where `influence` begins: two empty lists,
/// one byte each) loads with no influence.
#[test]
fn a_save_from_before_the_influence_state_loads() {
    use laplace_schelling::save::encode_snapshot;
    let world = data().new_world();
    let bytes = encode_snapshot(&world).unwrap();
    let old = &bytes[..bytes.len() - 2];
    let back = bf_sim::save_format::decode_world(old).unwrap();
    assert!(back.influence.is_empty());
    assert_eq!(back, world);
}
