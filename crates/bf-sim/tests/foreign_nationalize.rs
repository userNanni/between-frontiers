//! Nationalization of foreign plants (`docs/foreign-capital.pt-BR.md`, section 7, (c), (d),
//! (j) and C6e): a country takes the foreign stakes in its states, pays part of their value at
//! once, owes part of it as a debt to each owner and takes the rest; the premium the capital
//! asks of it reads what of the debt is still unpaid.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::milli;
use bf_sim::foreign_capital::Handover;
use bf_sim::foreign_stakes::{self, Nationalization, Stake};
use bf_sim::game::{GameCommand, GameWorld, Session};
use laplace_schelling::schedule::TickCtx;
use laplace_schelling::{Command, Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn ix(w: &GameWorld, tag: &str) -> usize {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap()
}

/// A month into 1925, with the United States owning a third of every plant of Mexico's that
/// is neither public nor informal and Britain a level of its largest one.
fn world() -> (GameWorld, usize, usize, usize) {
    static WORLD: OnceLock<GameWorld> = OnceLock::new();
    let mut w = WORLD
        .get_or_init(|| {
            let d = data();
            let mut s = Session::new(
                d.new_world(),
                1925,
                Date::new(1925, 1, 1),
                2,
                Some(d.rules_with(Default::default())),
            );
            s.sim.run_until(Date::new(1925, 2, 1));
            s.world().clone()
        })
        .clone();
    let (mex, usa, gbr) = (ix(&w, "MEX"), ix(&w, "USA"), ix(&w, "GBR"));
    let cat = &data().rules().catalogue;
    let mut largest: Option<(u32, u16, i64)> = None;
    let mut adds = Vec::new();
    for (ri, r) in w.econ.regions.iter().enumerate() {
        if usize::from(r.market) != mex {
            continue;
        }
        for p in &r.plants {
            let b = &cat.buildings[p.kind as usize];
            if b.public || b.informal || p.levels < 3 {
                continue;
            }
            adds.push((ri as u32, p.kind, p.levels / 3));
            if largest.is_none_or(|l| p.levels > l.2) {
                largest = Some((ri as u32, p.kind, p.levels));
            }
        }
    }
    let stake = |region, kind, owner: usize, levels| Stake {
        region,
        kind,
        owner: owner as u16,
        levels,
        built: 1925,
        partner: Fixed::ZERO,
        term: 0,
        handover: Handover::Never,
    };
    for (r, k, l) in adds {
        w.foreign.add(stake(r, k, usa, l));
    }
    let (r, k, _) = largest.unwrap();
    w.foreign.add(stake(r, k, gbr, 1));
    (w, mex, usa, gbr)
}

fn pools(w: &GameWorld) -> Fixed {
    w.econ.markets.iter().map(|m| m.pool).sum()
}

#[test]
fn a_buyout_pays_the_owners_from_the_fund_and_takes_nothing() {
    let (mut w, mex, usa, gbr) = world();
    w.econ.markets[mex].pool = Fixed::from_int(1_000_000_000);
    let before = pools(&w);
    let usa_before = w.econ.markets[usa].pool;
    let order = Nationalization {
        host: mex as u16,
        cash: Fixed::ONE,
        debt: Fixed::ZERO,
    };
    let done = foreign_stakes::nationalize(&mut w, &data().rules().catalogue, &order, 1926);
    assert_eq!(done.owners.len(), 2);
    assert!(
        w.foreign.stakes.is_empty(),
        "every stake in Mexico is Mexican now"
    );
    assert!(w.foreign.expropriations.is_empty() && w.foreign.indemnities.is_empty());
    let (owner, value, paid, owed) = done.owners[1];
    assert_eq!(usize::from(owner), usa);
    assert_eq!((paid, owed), (value, Fixed::ZERO));
    assert_eq!(w.econ.markets[usa].pool - usa_before, value);
    assert_eq!(pools(&w), before, "money only changed hands");
    assert_eq!(usize::from(done.owners[0].0), gbr);
}

#[test]
fn an_indemnity_is_a_debt_and_the_premium_reads_what_was_paid() {
    let (mut w, mex, usa, _) = world();
    let lines = w.econ.ledger.debts.len();
    let order = Nationalization {
        host: mex as u16,
        cash: Fixed::ZERO,
        debt: milli(600),
    };
    let done = foreign_stakes::nationalize(&mut w, &data().rules().catalogue, &order, 1926);
    let total: Fixed = done.owners.iter().map(|o| o.1).sum();
    let us = done
        .owners
        .iter()
        .find(|o| usize::from(o.0) == usa)
        .unwrap();
    assert_eq!(w.econ.ledger.debts.len(), lines + done.owners.len());
    let line = w
        .econ
        .ledger
        .debts
        .iter()
        .find(|d| usize::from(d.creditor) == usa && usize::from(d.debtor) == mex)
        .unwrap()
        .clone();
    assert_eq!(line.principal, us.1 * milli(600));
    assert_eq!(line.years_left, foreign_stakes::INDEMNITY_YEARS);
    let indemnity = w
        .foreign
        .indemnities
        .iter()
        .find(|i| usize::from(i.owner) == usa)
        .unwrap()
        .clone();
    assert_eq!(indemnity.debt, line.id);
    assert_eq!(indemnity.share, us.1 / total * milli(600));
    // The 40 % not owed is taken without pay.
    let taken: Fixed = w.foreign.expropriations.iter().map(|e| e.share).sum();
    assert!((taken - milli(400)).abs() <= milli(1), "{taken:?}");
    // Nothing paid yet: the whole debt weighs; half paid, half of it; paid off, nothing.
    let ledger = &mut w.econ.ledger;
    assert_eq!(foreign_stakes::unpaid_share(ledger, &indemnity), Fixed::ONE);
    let l = ledger.debts.iter_mut().find(|d| d.id == line.id).unwrap();
    l.principal = line.principal / 2;
    assert_eq!(foreign_stakes::unpaid_share(ledger, &indemnity), milli(500));
    ledger.debts.retain(|d| d.id != line.id);
    assert_eq!(
        foreign_stakes::unpaid_share(ledger, &indemnity),
        Fixed::ZERO
    );
}

#[test]
fn cash_the_fund_does_not_have_is_taken() {
    let (mut w, mex, _, _) = world();
    w.econ.markets[mex].pool = Fixed::ZERO;
    let order = Nationalization {
        host: mex as u16,
        cash: Fixed::ONE,
        debt: Fixed::ZERO,
    };
    foreign_stakes::nationalize(&mut w, &data().rules().catalogue, &order, 1926);
    let taken: Fixed = w.foreign.expropriations.iter().map(|e| e.share).sum();
    assert!((taken - Fixed::ONE).abs() <= milli(1), "{taken:?}");
}

#[test]
fn the_order_waits_in_the_state_and_shares_above_a_thousand_are_cut() {
    let (mut w, mex, _, _) = world();
    let at = TickCtx {
        tick: 1,
        date: Date::new(1925, 2, 2),
        seed: 1925,
        stream: 0,
    };
    GameCommand::Nationalize {
        tag: "MEX".into(),
        cash: 700,
        debt: 700,
    }
    .apply(&at, &mut w);
    assert_eq!(
        w.foreign.requests,
        vec![Nationalization {
            host: mex as u16,
            cash: milli(700),
            debt: milli(300),
        }]
    );
    // An unknown country orders nothing.
    GameCommand::Nationalize {
        tag: "XXX".into(),
        cash: 0,
        debt: 0,
    }
    .apply(&at, &mut w);
    assert_eq!(w.foreign.requests.len(), 1);
}
