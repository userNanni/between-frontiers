//! Trade by pair behind its switch (`game::Switches::trade_by_pair`, docs/trade.pt-BR.md): off,
//! the trade state stays empty and the world keeps the hash of the world pool; on, a short run
//! fills it, so the path switched on keeps compiling and running in the main.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{Session, Switches};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn run(switches: Switches) -> Session {
    let d = data();
    let rules = d.rules_with(switches);
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(1925, 3, 1));
    s
}

#[test]
fn off_by_default_the_trade_state_stays_empty() {
    let rules = data().rules();
    assert_eq!(rules.switches, Switches::default());
    assert!(
        rules.trade.is_none(),
        "switched off, the distances are not built"
    );
    let s = run(Switches::default());
    assert!(s.world().econ.trade.is_empty());
}

#[test]
fn switched_on_a_short_run_trades_by_pair() {
    let on = run(Switches {
        trade_by_pair: true,
        ..Switches::default()
    });
    let t = &on.world().econ.trade;
    assert!(!t.is_empty());
    assert!(!t.pref.is_empty(), "the month's preferences are set");
    let pairs = t.year.iter().filter(|v| **v > Fixed::ZERO).count();
    assert!(pairs > 50, "only {pairs} pairs traded in two months");
    assert_ne!(
        on.state_hash(),
        run(Switches::default()).state_hash(),
        "the switch changes the world"
    );
}

#[test]
fn switches_parse_by_name() {
    assert_eq!(Switches::parse("").unwrap(), Switches::default());
    assert!(Switches::parse(" trade_by_pair ").unwrap().trade_by_pair);
    assert!(Switches::parse("no_such_rule").is_err());
    let all = Switches::parse(&Switches::NAMES.join(",")).unwrap();
    assert!(
        all.trade_by_pair
            && all.jobs_follow_use
            && all.small_market_wage_floor
            && all.foreign_flow
            && all.schooling
            && all.forces
            && all.fiscal_capacity
    );
}

/// `Switches::NAMES` names every field of `Switches`, once: switching every name on leaves no
/// field off, and there are as many names as fields.
#[test]
fn every_switch_field_has_its_name() {
    let all = Switches::parse(&Switches::NAMES.join(",")).unwrap();
    let text = ron::to_string(&all).unwrap();
    assert!(
        !text.contains("false"),
        "a field without a name in NAMES: {text}"
    );
    assert_eq!(
        text.matches("true").count(),
        Switches::NAMES.len(),
        "{text}"
    );
    let mut names = Switches::NAMES.to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), Switches::NAMES.len(), "a name twice in NAMES");
}

/// `data/rules.ron` names every switch, and all of them ship off: the main keeps its hash.
#[test]
fn rules_file_names_every_switch_and_ships_them_off() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/rules.ron");
    let text = std::fs::read_to_string(&path).unwrap();
    let file: Switches = ron::from_str(&text).unwrap();
    assert_eq!(file, Switches::default(), "every rule ships off");
    for name in Switches::NAMES {
        assert!(
            text.contains(&format!("{name}: false")),
            "data/rules.ron lists {name}"
        );
    }
}
