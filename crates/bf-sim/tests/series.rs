//! The yearly series for the charts (`econ::series`, `GameWorld::series`): one row per market
//! with people for each closed year, the metropole within the country, mean prices around the
//! base price; and the series are a record only, so the world's hash does not see them.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{Session, Switches};
use laplace_schelling::Date;
use laplace_schelling::hash::hash_of;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

#[test]
fn three_years_of_series_and_their_size() {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(Switches::shipped())),
    );
    s.sim.run_until(Date::new(1928, 1, 3));
    let w = s.world();
    let years: Vec<i32> = {
        let mut y: Vec<i32> = w.series.years.iter().map(|r| r.year).collect();
        y.dedup();
        y
    };
    assert_eq!(years, [1925, 1926, 1927]);
    // One row per market with people, as the yearly record.
    let per_year = w.econ.history.iter().filter(|r| r.year == 1926).count();
    assert_eq!(
        w.series.years.iter().filter(|r| r.year == 1926).count(),
        per_year
    );
    assert_eq!(
        w.series.prices.iter().filter(|r| r.year == 1926).count(),
        per_year
    );
    for r in w.series.years.iter().filter(|r| r.year == 1927) {
        let rec = w
            .econ
            .history
            .iter()
            .find(|h| h.year == 1927 && h.market == r.market)
            .unwrap();
        // The metropole is part of the country.
        assert!(r.population_metro <= rec.population);
        assert!(r.gdp_metro <= rec.gdp);
        assert!(r.life > laplace_schelling::Fixed::ZERO);
    }
    let goods = d.catalogue.goods.len();
    for p in &w.series.prices {
        assert_eq!(p.per_mille.len(), goods);
        // Some good had a market in every country with people.
        assert!(p.per_mille.iter().any(|x| *x > 0));
    }
    // A record only: the hash leaves it out.
    let mut emptied = w.clone();
    emptied.series = Default::default();
    assert_eq!(hash_of(&emptied), hash_of(w));
    // The size, for the PR: bytes in the save per closed year.
    let bytes = bf_sim::save_format::encode_world(&emptied, None)
        .unwrap()
        .len();
    let with = bf_sim::save_format::encode_world(w, None).unwrap().len();
    println!(
        "series: {} bytes in the save for 3 years ({} a year), save {} bytes",
        with - bytes,
        (with - bytes) / 3,
        with
    );
}
