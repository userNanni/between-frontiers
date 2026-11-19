//! The gold standard's regimes and rule (`econ::gold`, `Switches::gold_standard` with
//! `payments_settle`): the world's net reserves still change only by new gold, no reserve goes
//! below zero, a bank short of its cover creates no money, a country stripped of its reserves
//! leaves after the rule's weeks and its currency falls, with nobody gaining from nowhere; a
//! reload keeps the hash. Off, nobody has a regime.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::gold::Regime;
use bf_sim::game::{Session, Switches};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn on() -> Switches {
    Switches::shipped()
        .with("payments_settle,gold_standard")
        .unwrap()
}

fn market(tag: &str) -> usize {
    data()
        .countries
        .countries
        .iter()
        .position(|c| c.tag == tag)
        .unwrap()
}

fn issued(s: &Session, mi: usize) -> Fixed {
    let m = &s.world().econ.markets[mi].money;
    m.debt_bank + m.credit
}

fn world_net(s: &Session) -> Fixed {
    s.world().reserves.net().iter().copied().sum()
}

fn new_gold(s: &Session) -> Fixed {
    s.world().reserves.record.iter().map(|r| r.week.gold).sum()
}

fn check_reserves(s: &Session, when: Date) {
    for (mi, r) in s.world().reserves.markets.iter().enumerate() {
        assert!(
            r.gold >= Fixed::ZERO,
            "{when}: market {mi}'s gold {}",
            r.gold
        );
        assert!(
            r.fx.iter()
                .all(|x| x.1 > Fixed::ZERO && usize::from(x.0) != mi),
            "{when}: market {mi}'s foreign exchange {:?}",
            r.fx
        );
    }
}

#[test]
fn the_rule_keeps_the_worlds_reserves_and_binds_the_banks_short_of_cover() {
    let d = data();
    let rules = d.rules_with(on());
    let legal = rules
        .gold
        .clone()
        .expect("gold.ron is loaded")
        .legal_cover
        .clone();
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    let mut date = Date::new(1925, 1, 12);
    s.sim.run_until(date);
    let w = s.world();
    assert!(!w.gold.is_empty());
    assert_eq!(w.gold.countries[market("GBR")].regime, Regime::Gold);
    assert_eq!(w.gold.countries[market("FRA")].regime, Regime::Float);
    // Sterling holds its prewar parity from the first week.
    assert_eq!(
        w.econ.markets[market("GBR")].money.exchange_rate,
        w.gold.countries[market("GBR")].parity
    );
    let mut bound = 0;
    for _ in 0..40 {
        let next = Date::from_days(date.to_days() + 7);
        let net = world_net(&s);
        let before: Vec<Fixed> = (0..s.world().econ.markets.len())
            .map(|mi| issued(&s, mi))
            .collect();
        s.sim.run_until(next);
        // Settlement, the rule and the revaluation of foreign exchange: only new gold adds.
        assert_eq!(
            world_net(&s) - net,
            new_gold(&s),
            "week of {next}: the world's net reserves move by new gold only"
        );
        check_reserves(&s, next);
        let w = s.world();
        for (mi, st) in w.gold.countries.iter().enumerate() {
            if st.regime == Regime::Gold {
                assert_eq!(w.econ.markets[mi].money.exchange_rate, st.parity);
                if st.cover < legal[mi] {
                    bound += 1;
                    // Deflation raises the real value of what was issued; nothing else may.
                    let m = &w.econ.markets[mi].money;
                    let erosion =
                        bf_sim::econ::money::weekly(before[mi], (-m.inflation).max(Fixed::ZERO));
                    assert!(
                        issued(&s, mi) <= before[mi] + erosion + Fixed::from_raw(2),
                        "week of {next}: market {mi}, short of its cover, created money"
                    );
                    assert_eq!(w.econ.markets[mi].budget.issued, Fixed::ZERO);
                }
            }
        }
        date = next;
    }
    assert!(bound > 0, "some bank on gold was short of its cover");
    let bytes = s.save("test", "gold").unwrap();
    let back = Session::load(&bytes, 2, Some(d.rules_with(on()))).unwrap();
    assert_eq!(back.state_hash(), s.state_hash());
}

#[test]
fn a_country_without_reserves_leaves_and_its_holders_lose_by_the_fall() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(on())),
    );
    let gbr = market("GBR");
    let usa = market("USA");
    let mut date = Date::new(1925, 1, 12);
    s.sim.run_until(date);
    // Strip the Bank of England of its gold, and give the Federal Reserve a sterling balance
    // in its place: the world's net reserves are the same.
    {
        let w = s.sim.world_mut();
        let gold = std::mem::take(&mut w.reserves.markets[gbr].gold);
        w.reserves.markets[usa].gold -= gold;
        let fx = &mut w.reserves.markets[usa].fx;
        match fx.binary_search_by_key(&(gbr as u16), |x| x.0) {
            Ok(i) => fx[i].1 += gold,
            Err(i) => fx.insert(i, (gbr as u16, gold)),
        }
    }
    let mut left = None;
    for week in 0..40 {
        let next = Date::from_days(date.to_days() + 7);
        let net = world_net(&s);
        s.sim.run_until(next);
        assert_eq!(world_net(&s) - net, new_gold(&s), "week of {next}");
        check_reserves(&s, next);
        let w = s.world();
        let st = &w.gold.countries[gbr];
        if st.regime != Regime::Gold && left.is_none() {
            left = Some(week);
        }
        date = next;
    }
    // Sterling floats down from its parity: the foreign exchange in sterling fell with it, and
    // the world's net reserves above did not move but by new gold, so what the holders lost is
    // what the Bank of England no longer owes (the exact revaluation: `econ::gold`'s tests).
    let week = left.expect("sterling left gold");
    assert!(
        week >= 12,
        "it left after the rule's weeks, not at once: week {week}"
    );
    let w = s.world();
    assert!(w.econ.markets[gbr].money.exchange_rate > w.gold.countries[gbr].parity);
    assert!(
        w.gold
            .events
            .iter()
            .any(|e| usize::from(e.market) == gbr && e.regime != Regime::Gold)
    );
}

#[test]
fn the_player_adheres_at_the_prewar_parity_and_floats_again() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(on())),
    );
    let fra = market("FRA");
    s.sim.run_until(Date::new(1925, 1, 12));
    s.submit(bf_sim::game::GameCommand::SetGoldStandard {
        tag: "FRA".into(),
        choice: Some(bf_sim::econ::gold::GoldChoice::Prewar),
    });
    s.sim.run_until(Date::new(1925, 1, 26));
    let w = s.world();
    let st = &w.gold.countries[fra];
    assert_eq!(st.regime, Regime::Gold);
    assert!(st.chosen);
    assert_eq!(st.parity, st.prewar);
    assert_eq!(w.econ.markets[fra].money.exchange_rate, st.prewar);
    s.submit(bf_sim::game::GameCommand::SetGoldStandard {
        tag: "FRA".into(),
        choice: Some(bf_sim::econ::gold::GoldChoice::Float),
    });
    s.sim.run_until(Date::new(1925, 2, 9));
    assert_eq!(s.world().gold.countries[fra].regime, Regime::Float);
}

#[test]
fn off_or_without_settlement_nobody_has_a_regime() {
    let d = data();
    for switches in [
        Switches::shipped().with("payments_settle").unwrap(),
        Switches::shipped().with("gold_standard").unwrap(),
    ] {
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules_with(switches)),
        );
        s.sim.run_until(Date::new(1925, 3, 2));
        assert!(s.world().gold.is_empty());
    }
}
