//! The budget's levers, first slice (`econ::levers`, docs/state.pt-BR.md, "Alavancas do orçamento
//! e da dívida"): a lever handed back to the rule keeps the world's hash, a fixed one moves its
//! number the way the design says, and the levers survive a save.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::levers::SurplusUse;
use bf_sim::econ::prof;
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::calendar::Date;
use laplace_schelling::hash::hash_of;

fn data() -> GameData {
    GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
}

fn run(d: &GameData, commands: Vec<GameCommand>, until: Date) -> Session {
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    for c in commands {
        s.submit(c);
    }
    while s.clock().date < until {
        s.sim.step();
    }
    s
}

fn market(s: &Session, tag: &str) -> usize {
    s.sim
        .world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .unwrap()
}

fn people(s: &Session, ci: usize, p: usize) -> i64 {
    s.sim
        .world()
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == ci)
        .map(|r| r.pops[p].size)
        .sum()
}

#[test]
fn levers_back_on_the_rule_keep_the_hash() {
    let d = data();
    let until = Date::new(1925, 3, 1);
    let rule = run(&d, vec![], until);
    let back = run(
        &d,
        vec![
            GameCommand::SetPublicPay {
                tag: "BRA".into(),
                per_mille: None,
            },
            GameCommand::SetTreasury {
                tag: "BRA".into(),
                reserve_weeks: None,
                surplus: None,
            },
            GameCommand::SetDeficitCap {
                tag: "BRA".into(),
                per_mille_gdp: None,
            },
            GameCommand::SetIssuance {
                tag: "BRA".into(),
                shares: None,
            },
            GameCommand::SetPayroll {
                tag: "BRA".into(),
                per_mille: None,
            },
            GameCommand::SetStatePurchases {
                tag: "BRA".into(),
                goods: None,
                military: None,
            },
        ],
        until,
    );
    assert!(back.sim.world().levers.is_empty());
    assert_eq!(rule.state_hash(), back.state_hash());
}

#[test]
fn fixed_levers_move_their_numbers() {
    let d = data();
    let until = Date::new(1925, 5, 1);
    let rule = run(&d, vec![], until);
    let fixed = run(
        &d,
        vec![
            // Brazil pays its servants as much as the private sector; Argentina runs no deficit
            // for purchases; Mexico halves its servants.
            GameCommand::SetPublicPay {
                tag: "BRA".into(),
                per_mille: Some(1000),
            },
            GameCommand::SetDeficitCap {
                tag: "ARG".into(),
                per_mille_gdp: Some(0),
            },
            GameCommand::SetPayroll {
                tag: "MEX".into(),
                per_mille: Some([6, 5]),
            },
            GameCommand::SetTreasury {
                tag: "CHL".into(),
                reserve_weeks: Some(500),
                surplus: Some(SurplusUse::Keep),
            },
        ],
        until,
    );
    let (r, f) = (rule.sim.world(), fixed.sim.world());
    let bra = market(&rule, "BRA");
    assert!(
        f.econ.markets[bra].budget.payroll > r.econ.markets[bra].budget.payroll,
        "higher public pay, higher payroll"
    );
    let arg = market(&rule, "ARG");
    assert!(
        f.econ.markets[arg].budget.goods < r.econ.markets[arg].budget.goods,
        "no deficit, fewer purchases"
    );
    // Mexico: servants fall toward 6 per mille, a fortieth of the excess a month, not at once.
    let mex = market(&rule, "MEX");
    let (before, after) = (
        people(&rule, mex, prof::CIVIL_SERVANT),
        people(&fixed, mex, prof::CIVIL_SERVANT),
    );
    assert!(after < before, "{after} servants, {before} on the rule");
    assert!(
        after > before / 2,
        "the cut is gradual: {after} of {before}"
    );
    // Chile's reserve is cut to the ceiling of 52 weeks.
    let chl = market(&rule, "CHL");
    assert_eq!(f.econ.markets[chl].levers.reserve_weeks(), 52);
    assert_ne!(rule.state_hash(), fixed.state_hash());
}

#[test]
fn levers_survive_a_save() {
    let d = data();
    let s = run(
        &d,
        vec![GameCommand::SetPublicPay {
            tag: "BRA".into(),
            per_mille: Some(900),
        }],
        Date::new(1925, 2, 1),
    );
    let bytes = s.save("test", "levers").unwrap();
    let loaded = Session::load(&bytes, 2, Some(d.rules())).unwrap();
    let bra = market(&s, "BRA");
    assert_eq!(
        loaded.sim.world().econ.markets[bra].levers.public_pay,
        Some(900)
    );
    assert_eq!(hash_of(loaded.sim.world()), hash_of(s.sim.world()));
}

/// O1, `fiscal_capacity`: with the switch on, a payroll cut below the rule collects less tax,
/// and the screen reads how much; with it off, the same cut costs no revenue.
#[test]
fn fewer_servants_collect_less_with_the_switch_on() {
    let mut d = data();
    let cut = || {
        vec![GameCommand::SetPayroll {
            tag: "MEX".into(),
            per_mille: Some([2, 5]),
        }]
    };
    let until = Date::new(1925, 7, 1);
    let off = run(&d, cut(), until);
    d.switches.fiscal_capacity = true;
    let on = run(&d, cut(), until);
    let mex = market(&off, "MEX");
    let (m_off, m_on) = (
        &off.sim.world().econ.markets[mex],
        &on.sim.world().econ.markets[mex],
    );
    assert_eq!(m_off.uncollected, None);
    let lost = m_on.uncollected.expect("the switch is on");
    assert!(
        lost > laplace_schelling::Fixed::ZERO,
        "fewer servants, taxes lost"
    );
    let v = bf_sim::econ::levers::view(m_on);
    let share = v.uncollected.unwrap();
    assert!(share > 0.0 && share < 1.0, "share lost {share}");
    assert!(v.servants.week_effect.unwrap() < 0.0);
    assert!(m_on.budget.revenue < m_off.budget.revenue);
}

#[test]
fn the_view_prices_a_week_of_the_fixed_value() {
    let d = data();
    let s = run(
        &d,
        vec![
            GameCommand::SetPublicPay {
                tag: "BRA".into(),
                per_mille: Some(500),
            },
            GameCommand::SetDeficitCap {
                tag: "BRA".into(),
                per_mille_gdp: Some(0),
            },
        ],
        Date::new(1925, 3, 1),
    );
    let v = bf_sim::econ::levers::view(&s.sim.world().econ.markets[market(&s, "BRA")]);
    assert!(
        v.public_pay.week_effect.unwrap() > 0.0,
        "lower pay saves cash"
    );
    assert!(
        v.deficit_cap.week_effect.unwrap() >= 0.0,
        "no deficit borrows less"
    );
    assert_eq!(v.reserve_weeks.week_effect, None);
    assert_eq!(v.uncollected, None);
}

/// D3: purchases at half the rule's buy less; a military share fixed at zero leaves no military
/// ceiling and gives the civilian goods the whole budget; one fixed high raises the ceiling the
/// forces read.
#[test]
fn fixed_purchases_and_military_share() {
    let d = data();
    let until = Date::new(1925, 4, 1);
    let rule = run(&d, vec![], until);
    let fixed = run(
        &d,
        vec![
            GameCommand::SetStatePurchases {
                tag: "BRA".into(),
                goods: Some(500),
                military: None,
            },
            GameCommand::SetStatePurchases {
                tag: "ARG".into(),
                goods: None,
                military: Some(0),
            },
            GameCommand::SetStatePurchases {
                tag: "MEX".into(),
                goods: None,
                military: Some(5000),
            },
        ],
        until,
    );
    let cat = &d.catalogue;
    let (r, f) = (rule.sim.world(), fixed.sim.world());
    let bra = market(&rule, "BRA");
    assert!(
        f.econ.markets[bra].budget.goods < r.econ.markets[bra].budget.goods,
        "half the purchases"
    );
    let v = bf_sim::econ::levers::purchases_view(&f.econ.markets[bra], cat);
    assert_eq!(v.goods.fixed, Some(500.0));
    assert!(v.goods.week_effect.unwrap() > 0.0, "buying less keeps cash");
    let arg = market(&rule, "ARG");
    let v = bf_sim::econ::levers::purchases_view(&f.econ.markets[arg], cat);
    assert_eq!(v.military_room, 0.0);
    assert!(v.budget > 0.0);
    let mex = market(&rule, "MEX");
    let v = bf_sim::econ::levers::purchases_view(&f.econ.markets[mex], cat);
    assert_eq!(v.military.fixed, Some(800.0), "cut to the ceiling");
    assert!(v.military_room > v.budget * 0.79);
    assert!(v.military.week_effect.unwrap() > 0.0);
    let rv = bf_sim::econ::levers::purchases_view(&r.econ.markets[mex], cat);
    assert!(rv.military.rule > 0.0 && rv.military.fixed.is_none());
}

/// D9: everything issued at home sells nothing abroad; a poor country asking all abroad in its own
/// currency sells only what lenders accept and leaves the rest unsold, with no switch to dollars
/// and no subordinate bank stepping in; the shares are scaled to add up to 1000.
#[test]
fn where_the_treasury_issues() {
    let d = data();
    let until = Date::new(1925, 4, 1);
    let s = run(
        &d,
        vec![
            GameCommand::SetIssuance {
                tag: "GBR".into(),
                shares: Some([1, 0, 0]),
            },
            GameCommand::SetIssuance {
                tag: "IND".into(),
                shares: Some([0, 1000, 0]),
            },
            GameCommand::SetIssuance {
                tag: "BRA".into(),
                shares: Some([500, 250, 250]),
            },
        ],
        until,
    );
    let w = s.sim.world();
    let gbr = market(&s, "GBR");
    let v = bf_sim::econ::levers::issuance_view(&w.econ.markets[gbr]);
    assert_eq!(v.fixed, Some([1.0, 0.0, 0.0]), "scaled to 1000");
    assert_eq!(v.abroad_local + v.abroad_fx, 0.0, "nothing sold abroad");
    let bra = market(&s, "BRA");
    let v = bf_sim::econ::levers::issuance_view(&w.econ.markets[bra]);
    assert_eq!(v.fixed, Some([0.5, 0.25, 0.25]));
    assert!(v.local_limit > 0.0 && v.local_limit <= 1.0);
    // Somebody poor asks all abroad in its own currency: some of it stays unsold.
    if let Some(ind) = w.countries.iter().position(|(_, c)| c.tag == "IND") {
        let v = bf_sim::econ::levers::issuance_view(&w.econ.markets[ind]);
        assert_eq!(v.abroad_fx, 0.0, "no switch to dollars");
        assert_eq!(v.home, 0.0, "nothing meant for home");
        if v.abroad_local + v.unsold_abroad > 0.0 {
            assert!(v.local_limit < 1.0);
        }
    }
    let rule = run(&d, vec![], until);
    assert_eq!(
        bf_sim::econ::levers::issuance_view(&rule.sim.world().econ.markets[gbr]).fixed,
        None
    );
    assert_ne!(rule.state_hash(), s.state_hash());
}
