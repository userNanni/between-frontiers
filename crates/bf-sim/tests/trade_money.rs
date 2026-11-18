//! Trade makes no money (`econ::week::margin_transfers`): importers collect their own price and
//! exporters are paid theirs, and the balance left goes back to the markets by their volume. The
//! transfers add up to zero every week, with the pool and with trade by pair. The shares used
//! to saturate `Fixed` and left most of the balance in the pools, about 0.7 % of world GDP a
//! year.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::week::margin_transfers;
use bf_sim::game::{Session, Switches};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn weeks_of_trade(switches: Switches) {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(switches)),
    );
    let mut date = Date::new(1925, 1, 5);
    let mut balance = Fixed::ZERO;
    for _ in 0..8 {
        date = Date::from_days(date.to_days() + 7);
        s.sim.run_until(date);
        let e = &s.world().econ;
        let t = margin_transfers(&d.catalogue, &e.markets);
        let made: Fixed = t.iter().copied().sum();
        assert_eq!(made, Fixed::ZERO, "week of {date}: the margins made {made}");
        balance += t.iter().map(|x| x.abs()).sum::<Fixed>();
    }
    assert!(balance > Fixed::ZERO, "the margins moved some money");
}

#[test]
fn the_margins_of_the_pool_make_no_money() {
    weeks_of_trade(Switches::default());
}

#[test]
fn the_margins_of_trade_by_pair_make_no_money() {
    weeks_of_trade(Switches {
        trade_by_pair: true,
        ..Switches::default()
    });
}
