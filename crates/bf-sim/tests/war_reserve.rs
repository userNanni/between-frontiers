//! The trained reserve by cohort (item 9, switch `reserve_cohorts`, off): the 1925 reserve split
//! into cohorts sums to the same men, each country's reserve is the sum of its cohorts every
//! week, and a January moves a class in and the old ones out. The ignored test prints France's
//! reserve by cohort in 1925, 1935 and 1939 and what it could mobilize, against the 1939
//! conference figures (measured, not a target: docs/war.pt-BR.md, "Reserva por coorte").

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use bf_sim::war::catalogue::Domain;
use laplace_schelling::Date;

fn load() -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.switches.forces = true;
    d.switches.reserve_cohorts = true;
    d
}

fn session(d: &GameData) -> Session {
    Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    )
}

#[test]
fn the_reserve_is_the_sum_of_its_cohorts() {
    // The same week with the switch off: the reserve of 1925, unsplit.
    let mut off = load();
    off.switches.reserve_cohorts = false;
    let mut o = session(&off);
    o.sim.run_until(Date::new(1925, 1, 9));
    let before: Vec<u64> = o.world().war.countries.iter().map(|c| c.reserve).collect();
    assert!(o.world().war.reserve.is_empty());
    let d = load();
    let mut s = session(&d);
    s.sim.run_until(Date::new(1925, 1, 9));
    let w = s.world();
    assert!(!w.war.reserve.is_empty(), "the switch is on and no reserve");
    for r in &w.war.reserve.countries {
        let i = usize::from(r.country);
        assert_eq!(
            r.men(),
            before[i],
            "country {i}: the 1925 split changed the men"
        );
        assert_eq!(w.war.countries[i].reserve, r.men());
    }
    // GBR has a reserve in the 1925 data (Territorial Army and Army Reserve) and no
    // conscription: 15 cohorts (the design value) and no class comes in.
    let gbr = w
        .countries
        .iter()
        .position(|(_, c)| c.tag == "GBR")
        .unwrap();
    let r = w.war.reserve.of(gbr as u16).unwrap();
    assert_eq!(r.service_months, 0);
    assert_eq!(r.cohorts.len(), 15);
    let gbr_1925 = r.cohorts.clone();
    // France has no reserve row in the 1925 data (a dependency, docs/war.pt-BR.md): it starts
    // empty and fills class by class.
    let fra = w
        .countries
        .iter()
        .position(|(_, c)| c.tag == "FRA")
        .unwrap();
    let r = w.war.reserve.of(fra as u16).unwrap();
    assert_eq!(r.service_months, 18);
    assert_eq!(r.men(), 0);
    s.sim.run_until(Date::new(1926, 1, 15));
    let w = s.world();
    for r in &w.war.reserve.countries {
        assert_eq!(w.war.countries[usize::from(r.country)].reserve, r.men());
    }
    // January 1926: France's class of 1925 came in; GBR's oldest cohort left and the rest
    // lost training.
    let r = w.war.reserve.of(fra as u16).unwrap();
    assert_eq!(r.cohorts.len(), 1);
    assert_eq!(r.cohorts[0].year, 1925);
    assert!(r.cohorts[0].men > 0);
    let r = w.war.reserve.of(gbr as u16).unwrap();
    assert_eq!(r.cohorts.len(), 14);
    assert!(
        r.cohorts
            .iter()
            .all(|c| c.year > gbr_1925.last().unwrap().year)
    );
    assert!(r.cohorts[0].training < gbr_1925[0].training);
}

/// A longer service keeps more men under arms in peace, a year at a time (the classes it keeps
/// stay on); the command is kept within its bounds.
#[test]
fn a_longer_service_grows_the_peace_army_a_year_at_a_time() {
    let d = load();
    let mil = d.war.as_ref().unwrap().military.clone();
    let land = |s: &Session, tag: &str| -> (i64, u16, u16) {
        let w = s.world();
        let ci = w.countries.iter().position(|(_, c)| c.tag == tag).unwrap();
        let men = w.war.countries[ci]
            .formations
            .iter()
            .filter(|f| {
                mil.types[f.unit as usize].domain == Domain::Land && !f.colonial && !f.called_up
            })
            .map(|f| bf_sim::war::state::men(&mil, f))
            .sum();
        let r = w.war.reserve.of(ci as u16).unwrap();
        (men, r.service_months, r.serving_months)
    };
    let mut a = session(&d);
    let mut b = session(&d);
    a.sim.run_until(Date::new(1926, 3, 1));
    b.sim.run_until(Date::new(1926, 3, 1));
    b.submit(GameCommand::SetServiceMonths {
        tag: "FRA".into(),
        months: 99,
    });
    let mut ratios = Vec::new();
    for year in [1927, 1928] {
        a.sim.run_until(Date::new(year, 2, 1));
        b.sim.run_until(Date::new(year, 2, 1));
        let (men_a, _, _) = land(&a, "FRA");
        let (men_b, months, serving) = land(&b, "FRA");
        assert_eq!(months, 36, "the command is kept within 6 and 36");
        println!(
            "FRA land under arms, February {year}: 18 months {men_a}, 36 by law ({serving} in effect) {men_b}"
        );
        ratios.push(men_b as f64 / men_a as f64);
    }
    // 30 months in effect after a year, 36 after two.
    assert!((ratios[0] - 30.0 / 18.0).abs() < 0.1, "{ratios:?}");
    assert!((ratios[1] - 2.0).abs() < 0.1, "{ratios:?}");
}

#[test]
#[ignore = "15 years; run in the slot"]
fn france_by_cohort_1925_to_1939() {
    let d = load();
    let mil = d.war.as_ref().unwrap().military.clone();
    let mut s = session(&d);
    for year in [1925, 1935, 1939] {
        s.sim.run_until(Date::new(year, 2, 1));
        let w = s.world();
        for tag in ["FRA", "GER", "ITA", "POL", "SOV"] {
            let Some(ci) = w.countries.iter().position(|(_, c)| c.tag == tag) else {
                continue;
            };
            let c = &w.war.countries[ci];
            let under_arms: f64 = c
                .formations
                .iter()
                .filter(|f| mil.types[f.unit as usize].domain == Domain::Land && !f.colonial)
                .map(|f| bf_sim::war::state::men(&mil, f) as f64)
                .sum();
            let Some(r) = w.war.reserve.of(ci as u16) else {
                continue;
            };
            println!(
                "{year} {tag}: service {} months | reserve {} in {} cohorts | land under arms {:.0} | mobilizable {:.0}",
                r.service_months,
                r.men(),
                r.cohorts.len(),
                under_arms,
                under_arms + r.men() as f64
            );
            if tag == "FRA" {
                for k in &r.cohorts {
                    println!(
                        "  FRA cohort {}: {} men, training {:.3}",
                        k.year,
                        k.men,
                        k.training.to_f64_lossy()
                    );
                }
            }
        }
    }
}
