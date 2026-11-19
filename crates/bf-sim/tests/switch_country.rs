//! "Play another country" moves the control (owner's decision: switch at any time): after
//! `SwitchCountry` the new country answers to the player and the one left goes back to the AI.
//! A crisis and a proposal on the table, both waiting on the player, are answered by the AI once
//! the player has switched away; a run without a switch keeps its hash; the save loads with the
//! new player.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use bf_sim::war::Stage;
use laplace_schelling::Date;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
        d.enable_crisis();
        d.switches.negotiation = true;
        d
    })
}

fn session(player: &str) -> Session {
    let d = data();
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: player.into() });
    s.sim.run_until(Date::new(1925, 2, 2));
    s
}

fn weeks(s: &mut Session, n: u32) {
    for _ in 0..7 * n {
        s.sim.step();
    }
}

/// Italy demands the Aegean of Greece, played: the demand waits on the player. Switched to
/// Italy, Greece is the AI's again and answers.
#[test]
fn a_crisis_waiting_on_the_player_is_answered_once_the_player_switches_away() {
    let demand = |s: &mut Session| {
        s.submit(GameCommand::Demand {
            tag: "ITA".into(),
            target: "GRE".into(),
            demand: Proposal {
                side_a: vec!["ITA".into()],
                side_b: vec!["GRE".into()],
                clauses: vec![Clause {
                    kind: ClauseKind::CedeStates {
                        states: vec!["GRC-AIGAIO".into()],
                    },
                    giver: Side::Junior,
                    receiver: Side::Senior,
                    until: None,
                    note: None,
                }],
                term_years: None,
                notice_years: 0,
            },
            weeks: None,
        });
    };
    let stage = |s: &Session| s.world().war.conflicts.crises.first().map(|k| k.stage);
    // Greece played: nobody answers for it.
    let mut kept = session("GRE");
    demand(&mut kept);
    weeks(&mut kept, 2);
    assert_eq!(
        stage(&kept),
        Some(Stage::Waiting),
        "the player has not answered"
    );
    // Switched to Italy: Greece answers as the AI.
    let mut switched = session("GRE");
    demand(&mut switched);
    weeks(&mut switched, 1);
    switched.submit(GameCommand::SwitchCountry { tag: "ITA".into() });
    weeks(&mut switched, 1);
    assert_eq!(switched.world().player_tag(), Some("ITA"));
    assert_ne!(
        stage(&switched),
        Some(Stage::Waiting),
        "Greece, the AI's again, answered"
    );
}

/// France proposes non-aggression to Britain, played: the proposal waits on the player. Switched
/// to France, Britain answers on Monday as the AI.
#[test]
fn a_proposal_waiting_on_the_player_is_answered_once_the_player_switches_away() {
    let propose = |s: &mut Session| {
        let clause = |giver: Side, receiver: Side| Clause {
            kind: ClauseKind::NonAggression,
            giver,
            receiver,
            until: None,
            note: None,
        };
        s.submit(GameCommand::ProposeTreaty {
            tag: "FRA".into(),
            to: "GBR".into(),
            proposal: Proposal {
                side_a: vec!["FRA".into()],
                side_b: vec!["GBR".into()],
                clauses: vec![
                    clause(Side::Senior, Side::Junior),
                    clause(Side::Junior, Side::Senior),
                ],
                term_years: Some(10),
                notice_years: 2,
            },
        });
    };
    let open = |s: &Session| {
        s.world()
            .treaties
            .table
            .open
            .iter()
            .any(|p| p.from == "FRA" && p.to == "GBR")
    };
    let mut kept = session("GBR");
    propose(&mut kept);
    weeks(&mut kept, 2);
    assert!(open(&kept), "the proposal waits on the player");
    let mut switched = session("GBR");
    propose(&mut switched);
    weeks(&mut switched, 1);
    assert!(open(&switched), "the proposal waits on the player");
    switched.submit(GameCommand::SwitchCountry { tag: "FRA".into() });
    weeks(&mut switched, 2);
    assert!(!open(&switched), "Britain, the AI's again, answered");
}

/// Without a switch nothing changes: a run that never switches matches one that "switches" to
/// the country it already plays; and a switch is saved: the save loads with the new player.
#[test]
fn a_switch_is_kept_in_the_save_and_a_run_without_one_is_unchanged() {
    let mut a = session("GBR");
    let mut b = session("GBR");
    b.submit(GameCommand::SwitchCountry { tag: "GBR".into() });
    weeks(&mut a, 2);
    weeks(&mut b, 2);
    assert_eq!(a.state_hash(), b.state_hash());

    let mut s = session("GBR");
    s.submit(GameCommand::SwitchCountry { tag: "GER".into() });
    weeks(&mut s, 1);
    let bytes = s.save("test", "switch").unwrap();
    let loaded = Session::load(&bytes, 2, Some(data().rules())).unwrap();
    assert_eq!(loaded.world().player_tag(), Some("GER"));
    assert_eq!(loaded.player(), Some("GER"));
}
