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

/// Three months of 1925 with trade by pair on (the trade channel reads it) and the influence at
/// `weight`, or without influence.
fn run(weight: Option<u32>) -> Session {
    let d = data();
    let mut rules = d.rules_with(bf_sim::game::Switches {
        trade_by_pair: true,
        foreign_flow: true,
        ..Default::default()
    });
    match weight {
        None => rules.influence = None,
        Some(w) => {
            let mut setup = (*rules.influence.clone().unwrap()).clone();
            setup.rules.proposal_weight = w;
            setup.rules.live_ties = true;
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
fn with_the_weight_the_map_holds_only_pairs_with_a_live_channel() {
    let s = run(Some(1000));
    let w = s.world();
    assert!(!w.influence.map.is_empty());
    let d = data();
    let ties = bf_sim::influence::Ties::of(w, Some(&d.catalogue), 1925);
    for (a, b) in w.influence.map.keys() {
        let debt = w
            .econ
            .ledger
            .debts
            .iter()
            .any(|d| d.creditor == *a && d.debtor == *b);
        let bought =
            ties.flow(w, usize::from(*b), usize::from(*a)) > laplace_schelling::Fixed::ZERO;
        let owns = ties
            .owned
            .as_ref()
            .is_some_and(|o| o.of(*a, *b) > laplace_schelling::Fixed::ZERO);
        assert!(
            debt || bought || owns,
            "{a} over {b}: no debt, no trade, no plant"
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
    let mut world = data().new_world();
    // A save of then had no treaty state and no forces either (the last fields; empty, one byte
    // and two empty lists).
    world.treaties = Default::default();
    world.war = Default::default();
    let bytes = encode_snapshot(&world).unwrap();
    let old = &bytes[..bytes.len() - 3 - 2];
    let back = bf_sim::save_format::decode_world(old).unwrap();
    assert!(back.influence.is_empty());
    assert_eq!(back, world);
}

/// The window's reading: each issue's bars add up to its influence; credit, trade and investment
/// are live, protection stays dormant with its front, and the worth is the map's.
#[test]
fn the_reading_of_a_pair_adds_up_and_names_the_dormant_fronts() {
    use bf_sim::influence::{Channel, ChannelKind, InfluenceSetup, channels};
    let s = run(Some(1000));
    let world = s.world();
    let (&(a, b), worth) = world
        .influence
        .map
        .iter()
        .next()
        .expect("a pair in the map");
    let mut setup: InfluenceSetup = (*data().rules().influence.unwrap()).clone();
    setup.rules.live_ties = true;
    let r = channels(world, &setup, usize::from(a), usize::from(b), 1925);
    assert_eq!(&r.worth, worth);
    for issue in &r.issues {
        let sum: laplace_schelling::Fixed = issue
            .channels
            .iter()
            .map(|c| c.part)
            .sum::<laplace_schelling::Fixed>()
            + issue.goodwill;
        let gap = (sum - issue.influence).abs();
        assert!(gap <= laplace_schelling::Fixed::from_raw(4), "{issue:?}");
        for c in &issue.channels {
            match c.kind {
                ChannelKind::Protection => assert!(matches!(c.channel, Channel::Dormant(_))),
                _ => assert!(matches!(c.channel, Channel::Live(_)), "{c:?}"),
            }
        }
    }
    assert!(
        r.issues
            .iter()
            .any(|i| i.influence > laplace_schelling::Fixed::ZERO)
    );
}

/// The switch: with `influence_ties` off (`data/rules.ron`) trade and investment stay dormant in the
/// reading, as in the build without them.
#[test]
fn with_the_switch_off_trade_and_investment_stay_dormant() {
    use bf_sim::influence::{Channel, ChannelKind, channels};
    let d = data();
    let rules = d.rules();
    assert!(!rules.switches.influence_ties, "the switch ships off");
    let setup = rules.influence.as_deref().unwrap();
    assert!(!setup.rules.live_ties);
    let on = d.rules_with(bf_sim::game::Switches::parse("influence_ties").unwrap());
    assert!(on.influence.as_deref().unwrap().rules.live_ties);
    let w = d.new_world();
    let debt = w.econ.ledger.debts.first().unwrap();
    let r = channels(
        &w,
        setup,
        usize::from(debt.creditor),
        usize::from(debt.debtor),
        1925,
    );
    for c in &r.issues[0].channels {
        if matches!(c.kind, ChannelKind::Trade | ChannelKind::Investment) {
            assert_eq!(
                c.channel,
                Channel::Dormant(bf_sim::influence::RULES_OFF),
                "switched off, not a front to come: {c:?}"
            );
        }
    }
}
