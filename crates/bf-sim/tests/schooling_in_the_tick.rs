//! Schooling by cohort in the tick (`demography::education`): it is born from 1925's data, every
//! age group of every state adds up to its people by level (also after people are taken out of
//! the pyramid, as war will), literacy is derived from it, and a literacy campaign costs the
//! treasury and teaches adults at the next school year.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::calendar::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn every_group_adds_up(s: &Session) -> usize {
    let w = s.sim.world();
    let mut n = 0;
    for (sc, ages) in w.education.regions.iter().zip(&w.demography.ages) {
        for (g, row) in sc.people(ages).iter().enumerate() {
            assert_eq!(row.iter().sum::<i64>(), ages[g]);
        }
        n += 1;
    }
    n
}

#[test]
fn schooling_is_born_adds_up_and_a_campaign_teaches_and_costs() {
    let mut data = GameData::load(&root()).unwrap();
    data.switches.schooling = true;
    let mut s = Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    while s.clock().date < Date::new(1925, 3, 1) {
        s.sim.step();
    }
    let regions = s.sim.world().econ.regions.len();
    assert_eq!(
        every_group_adds_up(&s),
        regions,
        "schooling for every state"
    );
    // Literacy is the schooling's.
    {
        let w = s.sim.world();
        for ((r, sc), ages) in w
            .econ
            .regions
            .iter()
            .zip(&w.education.regions)
            .zip(&w.demography.ages)
        {
            let lit = i64::from(sc.literacy(ages));
            assert!(
                (r.literacy.raw() * 10 - lit).abs() <= 10,
                "state {}",
                r.state
            );
        }
    }
    let bra = s
        .sim
        .world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == "BRA")
        .unwrap();
    let literacy = |s: &Session| {
        let w = s.sim.world();
        let (mut n, mut l) = (0i64, 0i64);
        for ((r, sc), ages) in w
            .econ
            .regions
            .iter()
            .zip(&w.education.regions)
            .zip(&w.demography.ages)
        {
            if usize::from(r.market) == bra {
                let adults: i64 = ages[3..].iter().sum();
                n += adults;
                l += adults * i64::from(sc.literacy(ages)) / 10_000;
            }
        }
        l as f64 / n as f64
    };
    let before = literacy(&s);
    s.submit(GameCommand::LiteracyCampaign {
        tag: "BRA".into(),
        on: true,
    });
    let treasury = |s: &Session| s.sim.world().econ.markets[bra].treasury;
    while s.clock().date < Date::new(1925, 4, 1) {
        s.sim.step();
    }
    assert_eq!(
        s.sim.world().education.campaigns[bra],
        5,
        "a campaign of five years"
    );
    while s.clock().date < Date::new(1926, 2, 1) {
        s.sim.step();
    }
    let w = s.sim.world();
    assert!(w.education.campaign_cost[bra] > laplace_schelling::Fixed::ZERO);
    let taught: i64 = w
        .econ
        .regions
        .iter()
        .zip(&w.education.regions)
        .filter(|(r, _)| usize::from(r.market) == bra)
        .map(|(_, sc)| sc.campaign)
        .sum();
    assert!(taught > 100_000, "{taught} adults taught in 1926");
    let after = literacy(&s);
    println!(
        "Brazil adult literacy {before:.3} -> {after:.3}, {taught} taught, treasury {}",
        treasury(&s)
    );
    assert!(after > before + 0.02, "{before} -> {after}");
    // People taken out of the pyramid (war, epidemics) leave every group whole.
    let ages = &mut s.sim.world_mut().demography.ages[0];
    ages[5] -= ages[5] / 10;
    every_group_adds_up(&s);
}
