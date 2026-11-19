//! Embargoes by command (owner's decision CT6, docs/trade.pt-BR.md, section 11): a government's
//! embargo closes trade with the target both ways from the week it is ordered; lifted, the pair
//! trades again. Without one, the world hashes as before.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::{Date, Fixed};

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn session() -> Session {
    let d = data();
    Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    )
}

fn index(s: &Session, tag: &str) -> usize {
    s.world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .unwrap()
}

/// Both ways between `a` and `b`, so far this year, at base prices.
fn between(s: &Session, a: usize, b: usize) -> Fixed {
    let t = &s.world().econ.trade;
    let n = usize::from(t.n);
    t.year[a * n + b] + t.year[b * n + a]
}

#[test]
fn an_embargo_closes_the_pair_both_ways_and_lifting_it_reopens_it() {
    let mut open = session();
    let mut shut = session();
    let (gbr, arg) = (index(&open, "GBR"), index(&open, "ARG"));
    shut.submit(GameCommand::Embargo {
        tag: "GBR".into(),
        target: "ARG".into(),
        on: true,
    });
    let spring = Date::new(1925, 4, 6);
    open.sim.run_until(spring);
    shut.sim.run_until(spring);
    assert!(between(&open, gbr, arg) > Fixed::ZERO, "no trade to close");
    assert_eq!(between(&shut, gbr, arg), Fixed::ZERO);
    assert!(!shut.world().tariffs.embargoes.is_empty());
    assert_ne!(open.state_hash(), shut.state_hash());

    shut.submit(GameCommand::Embargo {
        tag: "GBR".into(),
        target: "ARG".into(),
        on: false,
    });
    shut.sim.run_until(Date::new(1925, 9, 7));
    assert!(shut.world().tariffs.embargoes.is_empty());
    assert!(
        between(&shut, gbr, arg) > Fixed::ZERO,
        "the pair never reopened"
    );
}

#[test]
fn an_embargo_on_oneself_or_an_unknown_country_does_nothing() {
    let (mut a, mut b) = (session(), session());
    for target in ["GBR", "XXX"] {
        a.submit(GameCommand::Embargo {
            tag: "GBR".into(),
            target: target.into(),
            on: true,
        });
    }
    a.sim.run_until(Date::new(1925, 1, 2));
    assert!(a.world().tariffs.embargoes.is_empty());
    b.sim.run_until(Date::new(1925, 1, 2));
    assert_eq!(a.state_hash(), b.state_hash());
}
