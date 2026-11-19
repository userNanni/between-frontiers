//! The contract and the expropriation of foreign plants (`docs/foreign-capital.pt-BR.md`,
//! section 3, "Contrato e estatização", and section 7, C6a and C6c): at independence the plants
//! of a colony stay the old metropole's, less what local owners held; their profit leaves for
//! the metropole's fund the next Monday; on January 1st a contract that ran out passes the plant
//! to the partner, and a host whose laws close the capital or plan the economy takes the plants
//! and is remembered for it.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::borders::op::{self, Operation, Terms};
use bf_sim::data::GameData;
use bf_sim::econ::prof;
use bf_sim::econ::week::Week;
use bf_sim::foreign_capital::{CLOSED_PULL, Handover};
use bf_sim::foreign_stakes::{self, Stake};
use bf_sim::game::{GameWorld, Session};
use laplace_schelling::schedule::{System, TickCtx};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

/// The world a month in, so that plants have profits and pops incomes.
fn world() -> GameWorld {
    static WORLD: OnceLock<GameWorld> = OnceLock::new();
    WORLD
        .get_or_init(|| {
            let d = data();
            let mut s = Session::new(
                d.new_world(),
                1925,
                Date::new(1925, 1, 1),
                2,
                Some(d.rules()),
            );
            s.sim.run_until(Date::new(1925, 2, 1));
            s.world().clone()
        })
        .clone()
}

fn ix(w: &GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
}

const NIGERIA: [&str; 5] = [
    "NGA-BENUE-NIGER",
    "NGA-BORNU-ADAMAWA",
    "NGA-EASTERN",
    "NGA-KANO-SOKOTO",
    "NGA-WESTERN",
];

/// Nigeria leaves Britain, by agreement (Britain signs) or by revolt (nobody signs).
fn independence(w: &mut GameWorld, agreed: bool) -> usize {
    w.borders.pending = vec![Operation::Secede {
        from: "GBR".into(),
        states: NIGERIA.iter().map(|s| s.to_string()).collect(),
        tag: "NGA".into(),
        overlord: None,
        signatories: if agreed {
            vec!["GBR".into()]
        } else {
            Vec::new()
        },
        terms: Terms::default(),
    }];
    let rules = data()
        .border_rules
        .with_colonial_plants(&data().rules().catalogue);
    op::run_pending(w, &rules, Date::new(1925, 2, 1));
    assert!(w.borders.refused.is_empty(), "{:?}", w.borders.refused);
    ix(w, "NGA")
}

/// The levels the rule leaves to the metropole in each state of the colony, read from the
/// world `before` the independence for the states that are `nga`'s `after` it: (region, kind,
/// levels), farms, public and informal plants out.
fn expected(
    before: &GameWorld,
    after: &GameWorld,
    metropole: usize,
    nga: usize,
) -> Vec<(u32, u16, i64)> {
    let cat = &data().rules().catalogue;
    let w = before;
    let mut out = Vec::new();
    for (ri, r) in w.econ.regions.iter().enumerate() {
        if usize::from(after.econ.regions[ri].market) != nga || !r.colony {
            continue;
        }
        assert_eq!(usize::from(r.market), metropole);
        let cap = |r: &bf_sim::econ::Region| {
            (
                r.pops[prof::CAPITALIST].size,
                r.pops.iter().map(|p| p.size).sum::<i64>(),
            )
        };
        let home = w
            .econ
            .regions
            .iter()
            .filter(|h| usize::from(h.market) == metropole && !h.colony)
            .map(cap)
            .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
        let local = foreign_stakes::local_owners(cap(r), home);
        let private = (Fixed::ONE - w.econ.markets[metropole].laws.of(r).state_ownership)
            * (Fixed::ONE - local);
        for p in &r.plants {
            let b = &cat.buildings[p.kind as usize];
            if b.agriculture || b.public || b.informal {
                continue;
            }
            let held = (Fixed::from_int(p.levels) * private).to_int_floor();
            if held > 0 {
                out.push((ri as u32, p.kind, held));
            }
        }
    }
    out
}

fn stakes_of(w: &GameWorld, owner: usize) -> Vec<(u32, u16, i64)> {
    w.foreign
        .stakes()
        .filter(|s| usize::from(s.owner) == owner)
        .map(|s| (s.region, s.kind, s.levels))
        .collect()
}

#[test]
fn an_independent_colony_keeps_the_metropoles_plants_less_the_local_owners() {
    for agreed in [true, false] {
        let mut w = world();
        let gbr = ix(&w, "GBR");
        let before = w.clone();
        let plants_before: i64 = w
            .econ
            .regions
            .iter()
            .flat_map(|r| &r.plants)
            .map(|p| p.levels)
            .sum();
        let nga = independence(&mut w, agreed);
        let want = expected(&before, &w, gbr, nga);
        assert!(!want.is_empty(), "the metropole keeps plants in Nigeria");
        assert_eq!(stakes_of(&w, gbr), want, "agreed: {agreed}");
        let held: i64 = want.iter().map(|x| x.2).sum();
        eprintln!(
            "Nigeria: {held} levels stay British, in {} plants",
            want.len()
        );
        for s in w.foreign.stakes() {
            if usize::from(s.owner) == gbr {
                assert_eq!(usize::from(w.econ.regions[s.region as usize].market), nga);
                assert_eq!(s.built, 1925);
                assert_eq!(s.foreign_share(1990), Fixed::ONE, "no contract");
            }
        }
        // Ownership only: no level is built or lost.
        let plants_after: i64 = w
            .econ
            .regions
            .iter()
            .flat_map(|r| &r.plants)
            .map(|p| p.levels)
            .sum();
        assert_eq!(plants_after, plants_before);
    }
}

#[test]
fn switched_off_the_colony_keeps_every_plant() {
    let mut w = world();
    let gbr = ix(&w, "GBR");
    w.borders.pending = vec![Operation::Secede {
        from: "GBR".into(),
        states: NIGERIA.iter().map(|s| s.to_string()).collect(),
        tag: "NGA".into(),
        overlord: None,
        signatories: vec!["GBR".into()],
        terms: Terms::default(),
    }];
    op::run_pending(&mut w, &data().border_rules, Date::new(1925, 2, 1));
    assert!(stakes_of(&w, gbr).is_empty());
}

#[test]
fn the_plants_profit_leaves_for_the_old_metropole_the_next_monday() {
    let mut with = world();
    let gbr = ix(&with, "GBR");
    independence(&mut with, true);
    assert!(!stakes_of(&with, gbr).is_empty());
    let mut without = with.clone();
    without
        .foreign
        .stakes
        .retain(|k, _| usize::from(k.owner) != gbr);
    let rules = data().rules();
    let week = Week {
        cat: rules.catalogue.clone(),
        tree: rules.tree.clone(),
        geo: None,
        by_pair: false,
        treaty_tariffs: false,
        duties: None,
        union_services: false,
        war_closes: false,
        gross: false,
        wage_floor: false,
        // Stakes are read whenever either rule can make them.
        foreign_flow: true,
        fiscal_capacity: false,
    };
    let at = TickCtx {
        tick: 1,
        date: Date::new(1925, 2, 2),
        seed: 1925,
        stream: 0,
    };
    for w in [&mut with, &mut without] {
        let changes = week.compute(&at, w);
        week.apply(&at, changes, w);
    }
    assert!(
        with.econ.markets[gbr].pool > without.econ.markets[gbr].pool,
        "the remittance reached Britain's fund"
    );
}

#[test]
fn a_host_that_closes_its_capital_takes_the_plants_and_is_remembered() {
    let cat = &data().rules().catalogue;
    for planned in [false, true] {
        let mut w = world();
        let gbr = ix(&w, "GBR");
        let nga = independence(&mut w, true);
        let stakes = stakes_of(&w, gbr);
        let levels: Vec<(u32, u16, i64)> = stakes
            .iter()
            .map(|&(r, k, _)| {
                let p = w.econ.regions[r as usize]
                    .plants
                    .iter()
                    .find(|p| p.kind == k)
                    .unwrap();
                (r, k, p.levels)
            })
            .collect();
        // The open law of the day takes nothing.
        let none = foreign_stakes::january(&mut w, cat, 1926);
        assert!(none.taken.is_empty());
        assert_eq!(stakes_of(&w, gbr), stakes);
        // The law changes in the year: the next January takes the plants.
        let laws = &mut w.econ.markets[nga].laws.effects[0];
        if planned {
            laws.state_ownership = foreign_stakes::PLANNED_OWNERSHIP;
        } else {
            laws.foreign_investment = CLOSED_PULL;
        }
        w.foreign.retain(gbr as u16, nga as u16, Fixed::from_int(5));
        w.foreign.paid(stakes[0].0, stakes[0].1, gbr as u16);
        let done = foreign_stakes::january(&mut w, cat, 1927);
        assert!(stakes_of(&w, gbr).is_empty(), "planned: {planned}");
        assert_eq!(w.foreign.retained_in(gbr as u16, nga as u16), Fixed::ZERO);
        assert!(
            w.foreign
                .building
                .keys()
                .all(|k| usize::from(w.econ.regions[k.0 as usize].market) != nga),
            "the works paid from abroad are the host's"
        );
        assert_eq!(done.taken.len(), 1);
        assert_eq!(usize::from(done.taken[0].1), gbr);
        let memory: Vec<_> = w
            .foreign
            .expropriations
            .iter()
            .filter(|e| usize::from(e.host) == nga)
            .collect();
        assert_eq!(memory.len(), 1);
        assert_eq!(memory[0].year, 1927);
        assert_eq!(memory[0].share, Fixed::ONE, "all of it was Britain's");
        // The plants stay where they were, now the host's own.
        for (r, k, l) in levels {
            let p = w.econ.regions[r as usize]
                .plants
                .iter()
                .find(|p| p.kind == k)
                .unwrap();
            assert_eq!(p.levels, l);
        }
    }
}

#[test]
fn a_share_of_the_owners_is_taken_when_only_some_states_close() {
    let cat = &data().rules().catalogue;
    let mut w = world();
    let gbr = ix(&w, "GBR");
    let nga = independence(&mut w, true);
    // Another owner in the host, in a state that stays open.
    let regions: Vec<u32> = w.foreign.regions();
    assert!(!regions.is_empty());
    let r0 = regions[0];
    let kind = w.foreign.stakes().find(|s| s.region == r0).unwrap().kind;
    let other = (0..w.econ.markets.len())
        .find(|&m| m != gbr && m != nga)
        .unwrap();
    // Colonies of the host do not exist; take the whole host and compare to the two owners.
    w.foreign.add(Stake {
        region: r0,
        kind,
        owner: other as u16,
        levels: 1,
        built: 1925,
        partner: Fixed::ZERO,
        term: 0,
        handover: Handover::Never,
    });
    w.econ.markets[nga].laws.effects[0].foreign_investment = CLOSED_PULL;
    let done = foreign_stakes::january(&mut w, cat, 1926);
    assert_eq!(done.taken.len(), 2);
    let shares: Fixed = w
        .foreign
        .expropriations
        .iter()
        .filter(|e| usize::from(e.host) == nga)
        .map(|e| e.share)
        .sum();
    // The shares of one January add up to the whole taken, to the rounding of the division.
    assert!(
        (shares - Fixed::ONE).abs() <= Fixed::from_raw(4),
        "{shares:?}"
    );
    let gbr_share = w
        .foreign
        .expropriations
        .iter()
        .find(|e| usize::from(e.host) == nga)
        .unwrap()
        .share;
    assert!(gbr_share > Fixed::ZERO && gbr_share < Fixed::ONE);
}

#[test]
fn a_contract_that_ran_out_passes_the_plant_to_the_partner() {
    let cat = &data().rules().catalogue;
    let w = world();
    let (ri, kind) = w
        .econ
        .regions
        .iter()
        .enumerate()
        .find_map(|(ri, r)| r.plants.iter().find(|p| p.levels > 0).map(|p| (ri, p.kind)))
        .unwrap();
    let host = usize::from(w.econ.regions[ri].market);
    let owner = if host == 0 { 1 } else { 0 };
    for (handover, gone) in [
        (Handover::AtTerm, true),
        (Handover::Gradual, true),
        (Handover::Never, false),
    ] {
        let mut v = w.clone();
        v.foreign.stakes.clear();
        v.foreign.add(Stake {
            region: ri as u32,
            kind,
            owner: owner as u16,
            levels: 1,
            built: 1915,
            partner: bf_sim::econ::milli(400),
            term: 10,
            handover,
        });
        let done = foreign_stakes::january(&mut v, cat, 1925);
        assert_eq!(v.foreign.stakes.is_empty(), gone, "{handover:?}");
        assert_eq!(done.handed_over, usize::from(gone));
        assert!(
            v.foreign.expropriations.is_empty(),
            "a handover is not a taking"
        );
    }
}
