//! Switching the played country at any time (decision 4 of `docs/borders.pt-BR.md`): session
//! state, outside the world and its hash, kept by saves; freeing land only from the played
//! country.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::Date;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn session() -> Session {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "SOV".into() });
    s
}

#[test]
fn switching_changes_the_player_not_the_world() {
    let mut a = session();
    let mut b = session();
    a.sim.run_until(Date::new(1925, 2, 1));
    b.sim.run_until(Date::new(1925, 2, 1));
    a.submit(GameCommand::SwitchCountry { tag: "BRA".into() });
    a.submit(GameCommand::SwitchCountry {
        tag: "NOWHERE".into(),
    });
    a.sim.run_until(Date::new(1925, 3, 1));
    b.sim.run_until(Date::new(1925, 3, 1));
    assert_eq!(a.player(), Some("BRA"));
    assert_eq!(b.player(), Some("SOV"));
    assert_eq!(a.state_hash(), b.state_hash());
    // The save keeps the choice.
    let bytes = a.save("test", "").unwrap();
    let loaded = Session::load(&bytes, 1, Some(data().rules())).unwrap();
    assert_eq!(loaded.player(), Some("BRA"));
    assert_eq!(loaded.state_hash(), a.state_hash());
}

/// Start with the Soviet Union, free Ukraine, play Ukraine; a country not played frees nothing.
#[test]
fn free_a_republic_and_play_it() {
    let mut s = session();
    s.submit(GameCommand::Liberate {
        by: "GBR".into(),
        states: vec!["NGA-WESTERN".into()],
        tag: None,
        subject: false,
    });
    s.submit(GameCommand::Liberate {
        by: "SOV".into(),
        states: vec!["UKR-05".into(), "UKR-09".into()],
        tag: Some("UKR".into()),
        subject: false,
    });
    s.sim.run_until(Date::new(1925, 1, 3));
    let w = s.world();
    assert_eq!(w.borders.changes.len(), 1, "{:?}", w.borders.refused);
    assert!(
        w.borders.refused.is_empty(),
        "the session dropped the other country's command"
    );
    s.submit(GameCommand::SwitchCountry { tag: "UKR".into() });
    assert_eq!(s.player(), Some("UKR"));
    s.sim.run_until(Date::new(1925, 2, 1));
    let bytes = s.save("test", "").unwrap();
    let loaded = Session::load(&bytes, 2, Some(data().rules())).unwrap();
    assert_eq!(loaded.player(), Some("UKR"));
}
