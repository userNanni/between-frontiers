//! Settlement in money and reserves (`econ::payments::settle`, `Switches::payments_settle`):
//! every week the pools and treasuries of the world keep their money to the raw unit, each
//! country's net reserves (reserves less what it owes abroad) change by its balance and its new
//! gold, the world's only by new gold, no reserve goes below zero, and no lender abroad is
//! short. Off, nobody holds reserves and the world hashes as before.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::payments::settle;
use bf_sim::game::{Session, Switches};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn on() -> Switches {
    let mut s = Switches::shipped();
    s.switch_on("payments_settle").unwrap();
    s
}

#[test]
fn settlement_keeps_the_worlds_money_and_moves_reserves_by_the_balance() {
    let d = data();
    let rules = d.rules_with(on());
    let settle_rules = rules.settle.clone().expect("reserves.ron is loaded");
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    // After the first economy week, which seeds the reserves.
    let mut date = Date::new(1925, 1, 12);
    s.sim.run_until(date);
    assert!(!s.world().reserves.is_empty());
    let (mut moved, mut gold, mut unpaid) = (Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
    for _ in 0..20 {
        let next = Date::from_days(date.to_days() + 7);
        // The settlement the next week will run, on a copy of the world as it stands.
        let w = s.world();
        let mut reserves = w.reserves.clone();
        let before = reserves.net();
        let pools = settle(
            &settle_rules,
            &w.econ.markets,
            &w.payments,
            &mut reserves,
            next.year,
        );
        assert_eq!(
            reserves.mismatch,
            Fixed::ZERO,
            "week of {next}: the balance closes"
        );
        let sum: Fixed = pools.iter().copied().sum();
        assert_eq!(
            sum,
            Fixed::ZERO,
            "week of {next}: the pools of the world keep their money"
        );
        let after = reserves.net();
        let mut new_gold = Fixed::ZERO;
        for (mi, r) in reserves.record.iter().enumerate() {
            let b = before.get(mi).copied().unwrap_or(Fixed::ZERO);
            assert_eq!(
                after[mi] - b,
                r.week.balance + r.week.gold,
                "week of {next}: market {mi}'s net reserves move by its balance and new gold"
            );
            assert_eq!(pools[mi], r.week.pool);
            assert_eq!(
                r.week.pool,
                w.payments
                    .markets
                    .get(mi)
                    .map_or(Fixed::ZERO, |p| p.week.trade()),
            );
            new_gold += r.week.gold;
            moved += r.week.balance.abs();
            unpaid += r.week.unpaid;
        }
        gold += new_gold;
        let world_before: Fixed = before.iter().copied().sum();
        let world_after: Fixed = after.iter().copied().sum();
        assert_eq!(
            world_after - world_before,
            new_gold,
            "only new gold adds to the world's"
        );
        for (mi, r) in reserves.markets.iter().enumerate() {
            assert!(r.gold >= Fixed::ZERO, "market {mi}: gold {}", r.gold);
            assert!(
                r.fx.iter()
                    .all(|x| x.1 > Fixed::ZERO && usize::from(x.0) != mi)
            );
        }
        // The week runs the same settlement.
        s.sim.run_until(next);
        assert_eq!(
            s.world().reserves.markets,
            reserves.markets,
            "week of {next}"
        );
        let p = &s.world().payments.unreceived.week;
        assert_eq!(p.interest_received + p.principal_received, Fixed::ZERO);
        date = next;
    }
    assert!(moved > Fixed::ZERO, "some balance was settled");
    assert!(gold > Fixed::ZERO, "the mines added gold");
    eprintln!("20 weeks: balances settled {moved}, new gold {gold}, unpaid {unpaid}");
    // Settlement is state: the reserves are in the hash, and a reload keeps them.
    let bytes = s.save("test", "settle").unwrap();
    let back = Session::load(&bytes, 2, Some(d.rules_with(on()))).unwrap();
    assert_eq!(back.state_hash(), s.state_hash());
}

#[test]
fn off_nobody_holds_reserves() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(Switches::shipped())),
    );
    s.sim.run_until(Date::new(1925, 3, 2));
    assert!(s.world().reserves.is_empty());
}
