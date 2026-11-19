//! Customs duties (`econ::customs`, switch `customs_duties`, docs/trade.pt-BR.md, section 13):
//! a higher tariff makes a country buy less from its partners, and its money leaves the importers
//! for the treasuries, the world's money the same.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::econ::customs;
use bf_sim::game::{Session, Switches};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn run(customs_duties: bool) -> Session {
    let d = data();
    let rules = d.rules_with(Switches {
        customs_duties,
        ..Switches::shipped()
    });
    let mut s = Session::new(d.new_world(), 1925, Date::new(1925, 1, 1), 2, Some(rules));
    s.sim.run_until(Date::new(1925, 3, 2));
    s
}

/// What market `mi` bought from every partner so far this year, at base prices (`trade.year`).
fn bought(s: &Session, mi: usize) -> Fixed {
    let t = &s.world().econ.trade;
    let n = s.world().econ.markets.len();
    (0..n)
        .filter(|&a| a != mi)
        .map(|a| t.year.get(a * n + mi).copied().unwrap_or(Fixed::ZERO))
        .sum()
}

/// Britain under `law` from the first day, customs duties on, to the end of June 1925.
fn britain_under(law: &str) -> (Session, usize) {
    let d = data();
    let p = d.politics.as_ref().unwrap();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules_with(Switches {
            customs_duties: true,
            ..Switches::shipped()
        })),
    );
    let gbr = s
        .world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == "GBR")
        .unwrap();
    s.submit(bf_sim::game::GameCommand::SetLaw {
        tag: "GBR".into(),
        law: p.law(law).unwrap() as u16,
    });
    s.sim.run_until(Date::new(1925, 6, 29));
    (s, gbr)
}

#[test]
fn off_by_default_and_a_higher_tariff_buys_less_abroad() {
    assert!(!data().rules().switches.customs_duties, "ships off");
    let (off, on) = (run(false), run(true));
    assert_ne!(off.state_hash(), on.state_hash());
    // The tariff works through the price of arrival (gross trade's split between home and
    // foreign sellers), not a cut in the quantity: the protective law buys less abroad.
    let (free, gbr) = britain_under("free_trade");
    let (protect, _) = britain_under("protective_tariffs");
    assert!(
        bought(&protect, gbr) < bought(&free, gbr),
        "protective {} free {}",
        bought(&protect, gbr),
        bought(&free, gbr)
    );
}

#[test]
fn the_duties_leave_the_importers_for_the_treasuries() {
    let d = data();
    let s = run(true);
    let duties = d.duties();
    let markets = &s.world().econ.markets;
    let rates: Vec<Fixed> = markets
        .iter()
        .enumerate()
        .map(|(mi, m)| duties.rate(mi, m))
        .collect();
    let owed = customs::duties(&d.catalogue, markets, &rates, None);
    let total: Fixed = owed.iter().copied().sum();
    assert!(total > Fixed::ZERO, "no duty collected");
    let mut after = markets.clone();
    for (m, &duty) in after.iter_mut().zip(&owed) {
        customs::pay(m, duty);
    }
    let money =
        |ms: &[bf_sim::econ::Market]| -> Fixed { ms.iter().map(|m| m.pool + m.treasury).sum() };
    assert_eq!(money(&after), money(markets), "the world's money moved");
    let treasuries = |ms: &[bf_sim::econ::Market]| -> Fixed { ms.iter().map(|m| m.treasury).sum() };
    assert_eq!(treasuries(&after) - treasuries(markets), total);
}

#[test]
fn the_week_s_duties_are_kept_for_the_budget() {
    let s = run(true);
    let w = s.world();
    assert_eq!(w.tariffs.revenue.len(), w.econ.markets.len());
    assert!(
        w.tariffs.revenue.iter().any(|r| *r > Fixed::ZERO),
        "no duty recorded"
    );
    assert!(w.tariffs.revenue.iter().all(|r| *r >= Fixed::ZERO));
    // A record only: no table fixed, the world hashes as without one.
    assert!(w.tariffs.is_empty());
}
