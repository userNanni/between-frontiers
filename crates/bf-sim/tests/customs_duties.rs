//! Customs duties (`econ::customs`, switch `customs_duties`, docs/trade.pt-BR.md, section 13):
//! the tariff makes the world import less, and its money leaves the importers for the
//! treasuries, the world's money the same.

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

fn imports(s: &Session) -> Fixed {
    let cat = &data().catalogue;
    s.world()
        .econ
        .markets
        .iter()
        .map(|m| {
            (0..cat.goods.len())
                .filter(|&g| m.net_import[g] > Fixed::ZERO)
                .map(|g| m.net_import[g] * cat.goods[g].base_price)
                .sum::<Fixed>()
        })
        .sum()
}

#[test]
fn on_by_default_and_on_the_world_imports_less() {
    assert!(data().rules().switches.customs_duties, "ships on");
    let (off, on) = (run(false), run(true));
    assert_ne!(off.state_hash(), on.state_hash());
    assert!(
        imports(&on) < imports(&off),
        "imports on {} off {}",
        imports(&on),
        imports(&off)
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
