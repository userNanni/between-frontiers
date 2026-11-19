//! Trade makes and destroys no money (`econ::week::margin_transfers`): the clearing makes each
//! market's buyers pay for what came in, which no plant there sold, and pays its plants for what
//! left, which no buyer there bought, so it takes the market's balance out of its money; the
//! transfer gives that balance back to the same pool, every week, with the pool and with trade by
//! pair. Before, only the balance less a share of the world's came back, and the world's balance
//! (the gap between importers' and exporters' prices, 0.35 to 0.5 % of world GDP a year) left
//! the world's money.

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
        for (mi, m) in e.markets.iter().enumerate() {
            // What the clearing took out of this market's money for its trade.
            let mut taken = Fixed::ZERO;
            for gi in 0..d.catalogue.goods.len() {
                let n = m.net_import[gi];
                if n > Fixed::ZERO {
                    taken += n * m.sell_ratio[gi] * m.price[gi];
                } else if n < Fixed::ZERO {
                    taken += n * m.buy_ratio[gi] * m.price[gi];
                }
            }
            assert_eq!(
                t[mi],
                taken,
                "week of {date}: market {mi}'s money changed by {} with its trade",
                t[mi] - taken
            );
        }
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
