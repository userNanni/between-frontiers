//! Capital between countries in the running game (`docs/foreign-capital.pt-BR.md`, sections 3
//! and 4): a level built abroad moves money from the owner's fund to the host's and nowhere
//! else, the foreign share of a plant's profit leaves for the owner's fund, and the stakes stay
//! consistent with the plants and the queue.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::monthly::Investment;
use bf_sim::econ::week::Week;
use bf_sim::foreign_capital::{Handover, REINVESTED};
use bf_sim::foreign_stakes::Stake;
use bf_sim::game::{GameWorld, Session};
use laplace_schelling::schedule::{System, TickCtx};
use laplace_schelling::{Date, Fixed};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world_in(year: i32) -> (GameData, GameWorld) {
    let data = GameData::load(&root()).unwrap();
    let mut s = Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        2,
        Some(flow_on(&data)),
    );
    s.sim.run_until(Date::new(year, 1, 1));
    let w = s.world().clone();
    (data, w)
}

/// The game's rules with capital crossing borders switched on.
fn flow_on(data: &GameData) -> bf_sim::game::Rules {
    let mut r = data.rules();
    r.switches.foreign_flow = true;
    r
}

#[test]
fn switched_off_no_capital_crosses_a_border() {
    let data = GameData::load(&root()).unwrap();
    let mut s = Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    s.sim.run_until(Date::new(1927, 1, 1));
    assert!(s.world().foreign.is_empty());
}

fn ctx(date: Date) -> TickCtx {
    TickCtx {
        tick: 1,
        date,
        seed: 7,
        stream: 0,
    }
}

fn pools(w: &GameWorld) -> Vec<Fixed> {
    w.econ.markets.iter().map(|m| m.pool).collect()
}

/// Money of the world's funds, treasuries and savers.
fn holdings(w: &GameWorld, skip: Option<usize>) -> Fixed {
    let mut sum = Fixed::ZERO;
    for (mi, m) in w.econ.markets.iter().enumerate() {
        if Some(mi) != skip {
            sum += m.pool + m.treasury;
        }
    }
    for r in &w.econ.regions {
        if Some(r.market as usize) != skip {
            sum += r.pops.iter().map(|p| p.savings).sum::<Fixed>();
        }
    }
    sum
}

#[test]
fn capital_abroad_moves_money_between_funds_and_keeps_the_stakes_consistent() {
    let (data, mut w) = world_in(1927);
    let rules = data.rules();
    let inv = Investment {
        cat: rules.catalogue.clone(),
        tree: rules.tree.clone(),
        wage_floor: false,
        foreign_flow: true,
    };
    let before = pools(&w);
    let waiting = w.foreign.building_count();
    let at = ctx(Date::new(1927, 1, 8));
    let changes = inv.compute(&at, &w);
    inv.apply(&at, changes, &mut w);
    let after = pools(&w);
    // The month's works abroad: money left some funds and reached others, and the world's
    // funds hold exactly what they held.
    assert!(
        w.foreign.building_count() > waiting,
        "no level built abroad"
    );
    let sum = |v: &[Fixed]| v.iter().copied().sum::<Fixed>();
    assert_eq!(sum(&before), sum(&after));
    assert!(before.iter().zip(&after).any(|(b, a)| a < b));
    assert!(before.iter().zip(&after).any(|(b, a)| a > b));

    // Every work paid from abroad waits in its host's queue, and no stake holds more levels
    // than its plant.
    for (&(region, kind), owners) in &w.foreign.building {
        let host = w.econ.regions[region as usize].market as usize;
        assert!(owners.iter().all(|o| *o as usize != host));
        let queued = w.econ.markets[host]
            .queue
            .iter()
            .filter(|p| p.region == region && p.kind == kind && !p.public)
            .count();
        let paid = owners.len();
        assert!(paid <= queued, "{paid} paid from abroad, {queued} queued");
    }
    for s in w.foreign.stakes() {
        let r = &w.econ.regions[s.region as usize];
        let levels = r
            .plants
            .iter()
            .find(|p| p.kind == s.kind)
            .map_or(0, |p| p.levels);
        assert!(
            w.foreign.foreign_levels(s.region, s.kind) <= levels,
            "stakes above the plant"
        );
        assert_ne!(r.market, s.owner);
    }
}

#[test]
fn the_foreign_share_of_profit_leaves_for_the_owners_fund() {
    let (data, w) = world_in(1926);
    let rules = data.rules();
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
        foreign_flow: true,
        fiscal_capacity: false,
        settle: None,
        gold: None,
        capital: None,
        stock: false,
        biomass: None,
    };
    // The plant with the largest profit last week, of a kind with owners.
    let cat = &rules.catalogue;
    let (ri, kind, profit) = w
        .econ
        .regions
        .iter()
        .enumerate()
        .flat_map(|(ri, r)| {
            r.plants
                .iter()
                .filter(|p| {
                    let b = &cat.buildings[p.kind as usize];
                    p.levels > 0 && !b.public && !b.informal && b.farm.is_none()
                })
                .map(move |p| (ri, p.kind, p.profit))
        })
        .max_by(|a, b| a.2.cmp(&b.2).then(b.0.cmp(&a.0)))
        .unwrap();
    assert!(profit > Fixed::ZERO);
    let host = w.econ.regions[ri].market as usize;
    let owner = if host == 0 { 1 } else { 0 };
    let levels = w.econ.regions[ri]
        .plants
        .iter()
        .find(|p| p.kind == kind)
        .unwrap()
        .levels;
    let mut with = w.clone();
    with.foreign.stakes.retain(|k, _| k.region != ri as u32);
    let mut without = with.clone();
    with.foreign.add(Stake {
        region: ri as u32,
        kind,
        owner: owner as u16,
        levels,
        built: 1925,
        partner: Fixed::ZERO,
        term: 0,
        handover: Handover::Never,
    });
    let at = ctx(Date::new(1926, 1, 4));
    for w in [&mut with, &mut without] {
        let changes = week.compute(&at, w);
        week.apply(&at, changes, w);
    }
    // The owner's fund got the plant's profit of the week less what it reinvests here, which
    // stays in the host's fund as the owner's claim.
    let gain = with.econ.markets[owner].pool - without.econ.markets[owner].pool;
    let paid = with.econ.regions[ri]
        .plants
        .iter()
        .find(|p| p.kind == kind)
        .unwrap()
        .profit;
    assert!(paid > Fixed::ZERO);
    let out = paid * (Fixed::ONE - REINVESTED);
    // To the unit, but for the weekly interest the money module pays on the owner's larger
    // fund before the week closes.
    assert!(
        (gain - out).abs() <= paid / 1_000_000,
        "owner gained {gain:?} of {paid:?}"
    );
    let claim = with.foreign.retained_in(owner as u16, host as u16)
        - without.foreign.retained_in(owner as u16, host as u16);
    assert_eq!(claim, paid - out);
    // What the owner got, the rest of the world lacks: the same money changed hands. Rounding
    // of the shares that split what stayed is the only difference allowed.
    let lost = holdings(&without, Some(owner)) - holdings(&with, Some(owner));
    let gap = (lost - gain).abs();
    assert!(
        gap <= paid / 1000,
        "owner gained {gain:?}, the others lost {lost:?}"
    );
}

#[test]
fn with_every_door_closed_no_capital_crosses_a_border() {
    let data = GameData::load(&root()).unwrap();
    let p = data.politics.as_ref().unwrap();
    let closed = p.law("closed_capital").unwrap() as u16;
    let mut s = Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        2,
        Some(flow_on(&data)),
    );
    let tags: Vec<_> = s
        .world()
        .countries
        .iter()
        .map(|c| c.1.tag.clone())
        .collect();
    assert!(s.world().econ.markets.len() <= tags.len());
    for tag in tags {
        s.submit(bf_sim::game::GameCommand::SetLaw { tag, law: closed });
    }
    s.sim.run_until(Date::new(1928, 1, 1));
    // Nothing built abroad, nothing owned abroad: the foreign state stays out of the hash and
    // every other rule runs as before it existed.
    assert!(s.world().foreign.is_empty());
}

#[test]
fn owned_shares_add_up_by_host() {
    let (data, w) = world_in(1927);
    let rules = data.rules();
    let o = bf_sim::foreign_stakes::owned_shares(&w, &rules.catalogue, 1927);
    assert!(!o.pair.is_empty(), "no foreign plant by 1927");
    for host in 0..w.econ.markets.len() as u16 {
        let sum: Fixed = o
            .pair
            .iter()
            .filter(|(k, _)| k.1 == host)
            .map(|(_, v)| *v)
            .sum();
        assert_eq!(sum, o.foreign(host));
        assert!(o.foreign(host) >= Fixed::ZERO);
    }
}
