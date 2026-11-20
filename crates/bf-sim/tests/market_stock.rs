//! Stocks by market (`econ::stock`, `Switches::market_stock`): off, no market keeps anything;
//! on, the uranium the 1925 mines make with no buyer is kept, never below zero, and the world
//! with its stocks saves and loads with the same hash.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{Session, Switches};
use laplace_schelling::hash::hash_of;
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn session(switches: Switches) -> Session {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(switches)),
    );
    s.sim.run_until(Date::new(1926, 3, 1));
    s
}

#[test]
fn off_no_market_keeps_a_stock() {
    let s = session(Switches::shipped());
    assert!(s.world().stocks.is_empty());
}

#[test]
fn on_the_uranium_nobody_buys_is_kept_and_saved() {
    let s = session(Switches::shipped().with("market_stock").unwrap());
    let w = s.world();
    let cat = &data().catalogue;
    let u = usize::from(cat.good("uranium_ore").unwrap());
    let total: Fixed = (0..w.econ.markets.len()).map(|mi| w.stocks.of(mi, u)).sum();
    assert!(total > Fixed::ZERO, "the radium mines' uranium piles up");
    for s in &w.stocks.markets {
        for &(g, q) in s {
            assert!(
                cat.goods[usize::from(g)].stock.is_some(),
                "only storable goods"
            );
            assert!(q > Fixed::ZERO);
        }
    }
    let bytes = bf_sim::save_format::encode_world(w, None).unwrap();
    let back = bf_sim::save_format::decode_world(&bytes).unwrap();
    assert_eq!(hash_of(&back), hash_of(w));
}
