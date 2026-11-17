//! Research that costs: laboratories make the research a country's firms and state pay for, the
//! colonies have none, and the law splits the bill (`docs/research.pt-BR.md`, "Pesquisa que custa").

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::public::{research_spending, research_target};
use bf_sim::game::Session;
use laplace_schelling::{Date, Fixed};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn laboratories_in_the_metropole_paid_by_state_and_firms() {
    let data = GameData::load(&root()).unwrap();
    let cat = &data.catalogue;
    let lab = cat.laboratory.expect("the catalogue has a laboratory") as usize;
    let mut s = Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    s.sim.run_until(Date::new(1925, 2, 1));
    let w = s.world();
    let levels = |c: usize, colony: bool| -> i64 {
        w.econ
            .regions
            .iter()
            .filter(|r| r.market as usize == c && r.colony == colony)
            .flat_map(|r| r.plants.iter())
            .filter(|p| p.kind as usize == lab)
            .map(|p| p.levels)
            .sum()
    };
    let tag = |t: &str| w.countries.iter().position(|(_, x)| x.tag == t).unwrap();
    let (usa, gbr, chn) = (tag("USA"), tag("GBR"), tag("CHN"));
    // 1925 already has its laboratories, sized by the economy outside the farms.
    assert!(levels(usa, false) > levels(chn, false));
    // A colony does not research: its laboratories are the metropole's business.
    assert_eq!(levels(gbr, true), 0);
    // Both buyers pay: the treasury its legal share, the firms' fund the rest.
    let (state, firms) = research_spending(cat, &w.econ.markets[usa]);
    assert!(state > Fixed::ZERO && firms > Fixed::ZERO);
    // The firms near the frontier want more research than those far behind.
    let (near, _) = research_target(cat, &w.econ, usa, Fixed::ONE);
    let (far, _) = research_target(cat, &w.econ, usa, Fixed::from_ratio(1, 2));
    assert!(near > far);
}
