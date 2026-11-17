//! The wage of a market too small to have one (`econ::monthly::market_wages`, switch
//! `small_market_wage_floor`, off by default): with it on, San Marino,
//! Andorra and Liechtenstein (fewer than 5,000 plant workers) read at least the lowest wage of
//! the markets with a reading, instead of the near-zero pay of two plants at the floor of use.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::monthly::market_wages;
use bf_sim::game::{Session, Switches};
use laplace_schelling::{Date, Fixed};

#[test]
fn microstates_read_at_least_the_lowest_wage_of_the_others() {
    let d = GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
    assert!(
        !d.rules().switches.small_market_wage_floor,
        "off by default"
    );
    let mut rules = d.rules();
    rules.switches = Switches {
        small_market_wage_floor: true,
        ..Switches::default()
    };
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(1927, 1, 2));
    let tags: Vec<&str> = d
        .countries
        .countries
        .iter()
        .map(|c| c.tag.as_str())
        .collect();
    let wages = market_wages(&s.world().econ, true);
    let lowest_large = s
        .world()
        .econ
        .regions
        .iter()
        .map(|r| r.market as usize)
        .filter(|&m| !["SMR", "AND", "LIE", "MCO"].contains(&tags[m]))
        .map(|m| wages[m])
        .filter(|w| *w > Fixed::ZERO)
        .min()
        .unwrap();
    for tag in ["SMR", "AND", "LIE"] {
        let mi = tags.iter().position(|t| *t == tag).unwrap();
        assert!(
            wages[mi] >= lowest_large,
            "{tag}: {} below {}",
            wages[mi].to_f64_lossy(),
            lowest_large.to_f64_lossy()
        );
    }
}
