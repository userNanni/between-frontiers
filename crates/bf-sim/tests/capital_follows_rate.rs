//! Capital that follows interest, and capital controls (`econ::capital`,
//! `Switches::capital_follows_rate` with `payments_settle` and `gold_standard`): every week the
//! world's net reserves still change only by new gold, every line of private capital adds up
//! to zero over the world, no reserve goes below zero, and a reload keeps the hash. The
//! trilemma, in the world: on gold with free capital a rate held below the world's sends capital
//! out and a rate above brings it back with reserves; on gold behind controls the financial
//! account is closed; floating with free capital the exchange rate moves. Off, nobody holds a
//! private claim abroad.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::gold::{GoldChoice, Regime};
use bf_sim::econ::payments::Lines;
use bf_sim::game::{GameCommand, Session, Switches};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn on() -> Switches {
    Switches::shipped()
        .with("payments_settle,gold_standard,capital_follows_rate")
        .unwrap()
}

fn market(tag: &str) -> usize {
    data()
        .countries
        .countries
        .iter()
        .position(|c| c.tag == tag)
        .unwrap()
}

/// The first economy week, which seeds the claims, has run by this date.
const START: Date = Date::new(1925, 1, 12);

fn session() -> Session {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(on())),
    );
    s.sim.run_until(START);
    s
}

fn world_net(s: &Session) -> Fixed {
    s.world().reserves.net().iter().copied().sum()
}

fn new_gold(s: &Session) -> Fixed {
    s.world().reserves.record.iter().map(|r| r.week.gold).sum()
}

fn private(l: &Lines) -> Fixed {
    l.capital_long() + l.capital_short()
}

/// Runs `weeks` economy weeks from `date` and returns market `mi`'s lines summed over them.
fn run(s: &mut Session, date: &mut Date, weeks: usize, mi: usize) -> Lines {
    let mut sum = Lines::default();
    for _ in 0..weeks {
        let next = Date::from_days(date.to_days() + 7);
        s.sim.run_until(next);
        *date = next;
        sum += s.world().payments.markets[mi].week;
    }
    sum
}

#[test]
fn private_capital_keeps_the_worlds_reserves_and_every_line_balances() {
    let mut s = session();
    assert!(!s.world().capital.is_empty());
    let mut moved = Lines::default();
    let mut date = START;
    for _ in 0..30 {
        let next = Date::from_days(date.to_days() + 7);
        date = next;
        let net = world_net(&s);
        s.sim.run_until(next);
        assert_eq!(
            world_net(&s) - net,
            new_gold(&s),
            "week of {next}: the world's net reserves move by new gold only"
        );
        let w = s.world();
        for (mi, r) in w.reserves.markets.iter().enumerate() {
            assert!(r.gold >= Fixed::ZERO, "{next}: market {mi}'s gold");
            assert!(r.fx.iter().all(|x| x.1 > Fixed::ZERO));
        }
        let week: Vec<Lines> = w.payments.markets.iter().map(|p| p.week).collect();
        let sum = |f: &dyn Fn(&Lines) -> Fixed| week.iter().map(f).sum::<Fixed>();
        assert_eq!(sum(&|l| l.capital_in), sum(&|l| l.capital_out), "{next}");
        assert_eq!(sum(&|l| l.hot_in), sum(&|l| l.hot_out), "{next}");
        assert_eq!(
            sum(&|l| l.capital_income_received),
            sum(&|l| l.capital_income_paid),
            "{next}"
        );
        assert_eq!(sum(&|l| l.financial_account()), Fixed::ZERO, "{next}");
        for l in &week {
            moved += *l;
        }
        assert!(w.capital.long.iter().all(|&v| v >= Fixed::ZERO));
        assert!(w.capital.short.iter().all(|&v| v >= Fixed::ZERO));
    }
    assert!(moved.capital_in > Fixed::ZERO && moved.hot_in > Fixed::ZERO);
    assert!(moved.capital_income_paid > Fixed::ZERO);
    let bytes = s.save("test", "capital").unwrap();
    let back = Session::load(&bytes, 2, Some(data().rules_with(on()))).unwrap();
    assert_eq!(back.state_hash(), s.state_hash());
}

/// The trilemma, first case: on gold with free capital the rate is not free. For the United
/// States (on gold, its reserves all gold, so no contagion takes it off), against the same
/// weeks at the rule's rate, a rate held at zero sends capital out and leaves fewer reserves; a
/// rate of 10 % brings capital in and the reserves with it.
#[test]
fn on_gold_with_free_capital_the_rate_moves_the_reserves() {
    let usa = market("USA");
    let mut outcome = Vec::new();
    for rate in [Some(0), None, Some(100)] {
        let mut s = session();
        let mut date = START;
        assert_eq!(s.world().gold.countries[usa].regime, Regime::Gold);
        if rate.is_some() {
            s.submit(GameCommand::SetPolicyRate {
                tag: "USA".into(),
                rate,
            });
        }
        let l = run(&mut s, &mut date, 26, usa);
        assert_eq!(s.world().gold.countries[usa].regime, Regime::Gold);
        outcome.push((private(&l), s.world().reserves.net()[usa]));
    }
    let [low, rule, high] = [outcome[0], outcome[1], outcome[2]];
    assert!(
        low.0 < rule.0 && rule.0 < high.0,
        "private capital at 0 %, the rule, 10 %: {outcome:?}"
    );
    assert!(
        low.1 < rule.1 && rule.1 < high.1,
        "net reserves at 0 %, the rule, 10 %: {outcome:?}"
    );
}

/// The trilemma, second case: a parity behind controls frees the rate (no cover rule) and
/// closes the financial account.
#[test]
fn on_gold_behind_controls_the_financial_account_is_closed() {
    let mut s = session();
    let mut date = START;
    let nld = market("NLD");
    s.submit(GameCommand::SetGoldStandard {
        tag: "NLD".into(),
        choice: Some(GoldChoice::Control),
    });
    run(&mut s, &mut date, 2, nld);
    assert_eq!(s.world().gold.countries[nld].regime, Regime::Control);
    assert!(s.world().capital.controls[nld], "the rule's controls");
    let before = s.world().capital.position(nld);
    let parity = s.world().gold.countries[nld].parity;
    let l = run(&mut s, &mut date, 13, nld);
    let w = s.world();
    for (name, v) in [
        ("long", l.capital_in + l.capital_out),
        ("short", l.hot_in + l.hot_out),
        ("bonds", l.bonds_sold + l.bonds_bought),
        ("principal", l.principal_repaid),
    ] {
        assert_eq!(v, Fixed::ZERO, "{name} crossed the border");
    }
    assert!(l.capital_income_paid + l.capital_income_received > Fixed::ZERO);
    assert_eq!(w.econ.markets[nld].money.exchange_rate, parity);
    // Claims are only revalued, not moved: the parity holds, so only the real exchange moves.
    let after = w.capital.position(nld);
    assert!(after.0.0 > Fixed::ZERO && before.1.0 > Fixed::ZERO);
    // The player lifts them: capital moves again.
    s.submit(GameCommand::SetCapitalControls {
        tag: "NLD".into(),
        on: false,
    });
    let l = run(&mut s, &mut date, 4, nld);
    assert!(l.capital_in + l.capital_out + l.hot_in + l.hot_out > Fixed::ZERO);
}

/// The trilemma, third case: floating with free capital, capital moves and so does the
/// exchange rate, while the rate follows the Taylor rule (`econ::money`'s tests).
#[test]
fn floating_with_free_capital_the_exchange_rate_moves() {
    let mut s = session();
    let mut date = START;
    let fra = market("FRA");
    assert_eq!(s.world().gold.countries[fra].regime, Regime::Float);
    let ex = s.world().econ.markets[fra].money.exchange_rate;
    let l = run(&mut s, &mut date, 26, fra);
    assert!(l.capital_in + l.capital_out + l.hot_in + l.hot_out > Fixed::ZERO);
    assert_ne!(s.world().econ.markets[fra].money.exchange_rate, ex);
}

#[test]
fn off_or_without_settlement_nobody_holds_a_private_claim() {
    let d = data();
    for switches in [
        Switches::shipped()
            .with("payments_settle,gold_standard")
            .unwrap(),
        Switches::shipped().with("capital_follows_rate").unwrap(),
    ] {
        let mut s = Session::new(
            d.new_world(),
            1925,
            Date::new(1925, 1, 1),
            2,
            Some(d.rules_with(switches)),
        );
        s.sim.run_until(Date::new(1925, 3, 2));
        assert!(s.world().capital.is_empty());
        let moved =
            s.world().payments.markets.iter().any(|p| {
                p.year.capital_in + p.year.hot_in + p.year.capital_income_paid > Fixed::ZERO
            });
        assert!(!moved);
    }
}
