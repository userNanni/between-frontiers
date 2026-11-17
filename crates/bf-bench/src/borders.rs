//! `--borders-history`: the changes of sovereignty of `data/history/borders.ron` applied on their
//! dates, only to exercise the border operations over a century (`docs/borders.pt-BR.md`). It is
//! not how history happens in the game (that is phase 7, with conditions, never dates); it is off
//! by default and a run with it is not the reference run of the scorecard.
//!
//! Every operation is first run on a copy of the world and its totals compared
//! ([`bf_sim::borders::Totals`]): a change of owner must create or destroy nothing.

use std::sync::OnceLock;

use bf_sim::borders::op::{self, Operation, Terms};
use bf_sim::borders::{BordersFile, ChangeKind, Totals};
use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use laplace_schelling::Date;
use serde::Serialize;

/// Set once from the command line.
pub static ON: OnceLock<bool> = OnceLock::new();

pub fn on() -> bool {
    ON.get().copied().unwrap_or(false)
}

fn reference() -> &'static BordersFile {
    static FILE: OnceLock<BordersFile> = OnceLock::new();
    FILE.get_or_init(|| {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/history/borders.ron");
        let text = std::fs::read_to_string(&path).expect("data/history/borders.ron");
        ron::from_str(&text).expect("data/history/borders.ron")
    })
}

fn parse(d: &str) -> Date {
    let mut it = d.split('-').map(|x| x.parse::<i32>().unwrap_or(1));
    let y = it.next().unwrap_or(1925);
    let m = it.next().unwrap_or(1) as u8;
    let day = it.next().unwrap_or(1) as u8;
    Date::new(y, m, day)
}

fn holds(w: &GameWorld, tag: &str) -> bool {
    held(w, tag) > 0
}

fn held(w: &GameWorld, tag: &str) -> usize {
    let Some(c) = w.countries.iter().position(|(_, c)| c.tag == tag) else {
        return 0;
    };
    w.econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == c)
        .count()
}

/// The operation for one line of the reference, as the world stands: a secession into a country
/// already born that day (Libya in 1951, from two administrations) is a transfer to it.
/// Map state keys of the states `tag` holds.
fn keys_of(w: &GameWorld, rules: &bf_sim::borders::BorderRules, tag: &str) -> Vec<String> {
    let Some(c) = w.countries.iter().position(|(_, c)| c.tag == tag) else {
        return Vec::new();
    };
    let ids: std::collections::BTreeSet<u16> = w
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == c)
        .map(|r| r.state)
        .collect();
    rules
        .state_ids
        .iter()
        .filter(|(_, id)| ids.contains(id))
        .map(|(k, _)| k.clone())
        .collect()
}

fn operation(
    w: &GameWorld,
    rules: &bf_sim::borders::BorderRules,
    c: &bf_sim::borders::SovereigntyChange,
) -> Option<Operation> {
    let terms = Terms::default();
    // The line takes every state the country still holds: the end of a dissolution (the last
    // heir of Czechoslovakia, Yugoslavia, the Mali Federation) or a whole country handed over.
    let everything = !c.states.is_empty() && c.states.len() == held(w, &c.from);
    Some(match c.kind {
        ChangeKind::Transfer if everything && holds(w, &c.to) => Operation::Annex {
            from: c.from.clone(),
            into: c.to.clone(),
            colony: false,
            terms,
        },
        ChangeKind::Secession | ChangeKind::Transfer if everything && !holds(w, &c.to) => {
            Operation::Dissolve {
                from: c.from.clone(),
                heirs: vec![bf_sim::borders::Heir {
                    tag: c.to.clone(),
                    states: c.states.clone(),
                    overlord: None,
                }],
                terms,
            }
        }
        ChangeKind::Transfer => Operation::Transfer {
            from: c.from.clone(),
            to: c.to.clone(),
            states: c.states.clone(),
            colony: None,
            terms,
        },
        ChangeKind::Secession if holds(w, &c.to) => Operation::Transfer {
            from: c.from.clone(),
            to: c.to.clone(),
            states: c.states.clone(),
            colony: Some(false),
            terms,
        },
        ChangeKind::Secession => Operation::Secede {
            from: c.from.clone(),
            states: c.states.clone(),
            tag: c.to.clone(),
            overlord: None,
            signatories: vec![c.from.clone()],
            terms,
        },
        // Into a country that is not there yet (Slovakia into the Czechoslovakia restored in April
        // 1945, while the Czech lands were still German): the country is born again of it.
        ChangeKind::Annexation if !holds(w, &c.to) => Operation::Dissolve {
            from: c.from.clone(),
            heirs: vec![bf_sim::borders::Heir {
                tag: c.to.clone(),
                states: keys_of(w, rules, &c.from),
                overlord: None,
            }],
            terms,
        },
        ChangeKind::Annexation => Operation::Annex {
            from: c.from.clone(),
            into: c.to.clone(),
            colony: false,
            terms,
        },
        ChangeKind::Status => Operation::Status {
            tag: c.from.clone(),
            overlord: (c.to != c.from).then(|| c.to.clone()),
        },
        ChangeKind::Rename => return None,
    })
}

/// What the run did with the reference.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Log {
    pub submitted: usize,
    pub applied: usize,
    pub refused: Vec<String>,
    /// Operations whose totals changed on the copy of the world.
    pub not_conserved: Vec<String>,
}

/// Submits the lines dated `date` (call before the tick of `date`) and checks each on a copy.
pub fn submit(data: &GameData, s: &mut Session, log: &mut Log) {
    let date = s.clock().date;
    let rules = &data.border_rules;
    let mut copy: Option<GameWorld> = None;
    for c in reference()
        .changes
        .iter()
        .filter(|c| parse(&c.date) == date)
    {
        let w = copy.as_ref().unwrap_or(s.world());
        let Some(op) = operation(w, rules, c) else {
            continue;
        };
        let mut probe = copy.take().unwrap_or_else(|| s.world().clone());
        let before = Totals::of(&probe);
        let done = probe.borders.changes.len();
        probe.borders.pending = vec![op.clone()];
        op::run_pending(&mut probe, rules, date);
        let mut after = Totals::of(&probe);
        for ch in &probe.borders.changes[done..] {
            after.diaspora += ch.repatriated.0;
            after.arrived += ch.repatriated.1;
        }
        if after != before {
            log.not_conserved
                .push(format!("{} {:?} {} -> {}", c.date, c.kind, c.from, c.to));
        }
        copy = Some(probe);
        s.submit(GameCommand::Borders { op });
        log.submitted += 1;
    }
}

/// Counts what the world applied and refused, at the end of the run.
pub fn close(w: &GameWorld, log: &mut Log) {
    log.applied = w.borders.changes.len();
    log.refused = w
        .borders
        .refused
        .iter()
        .map(|r| format!("{:?}: {}", r.date, r.reason))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole reference, applied on the 1925 world in date order with no tick in between:
    /// every line maps to an operation that applies, and none changes the totals.
    #[test]
    fn the_whole_reference_applies_and_conserves() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let data = GameData::load(&root).unwrap();
        let mut w = data.new_world();
        let rules = &data.border_rules;
        for c in &reference().changes {
            let Some(op) = operation(&w, rules, c) else {
                continue;
            };
            let before = Totals::of(&w);
            let done = w.borders.changes.len();
            w.borders.pending = vec![op];
            op::run_pending(&mut w, rules, parse(&c.date));
            let mut after = Totals::of(&w);
            for ch in &w.borders.changes[done..] {
                after.diaspora += ch.repatriated.0;
                after.arrived += ch.repatriated.1;
            }
            assert_eq!(
                after, before,
                "{} {:?} {} -> {}",
                c.date, c.kind, c.from, c.to
            );
        }
        let refused: Vec<String> = w
            .borders
            .refused
            .iter()
            .map(|r| format!("{:?}: {}", r.date, r.reason))
            .collect();
        assert!(refused.is_empty(), "{refused:#?}");
        assert!(op::tables_agree(&w));
    }
}
