//! The public works' queue (`econ::levers`, D8 of docs/state.pt-BR.md): a fixed order puts the
//! public works in it and leaves the private ones where they were (owner's decision O2), a work
//! asked for is queued at the next month if the state may build it, a cancelled one leaves the
//! queue, and the order and the requests survive a save.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::econ::levers::{self, WorkKey};
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::calendar::Date;
use laplace_schelling::hash::hash_of;

fn data() -> GameData {
    GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
}

fn session(d: &GameData, until: Date) -> Session {
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    run(&mut s, until);
    s
}

fn run(s: &mut Session, until: Date) {
    while s.clock().date < until {
        s.sim.step();
    }
}

fn market(s: &Session, tag: &str) -> usize {
    s.sim
        .world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == tag)
        .unwrap()
}

/// The country with the most public works in its queue.
fn busiest(s: &Session) -> (String, usize) {
    let w = s.sim.world();
    let (ci, _) = w
        .econ
        .markets
        .iter()
        .enumerate()
        .max_by_key(|(i, m)| (m.queue.iter().filter(|p| p.public).count(), usize::MAX - i))
        .unwrap();
    let tag = w.countries.iter().nth(ci).unwrap().1.tag.clone();
    (tag, ci)
}

#[test]
fn a_fixed_order_moves_the_public_works_and_leaves_the_private_ones() {
    let d = data();
    let mut s = session(&d, Date::new(1925, 3, 1));
    let (tag, ci) = busiest(&s);
    let queue = s.sim.world().econ.markets[ci].queue.clone();
    let public: Vec<WorkKey> = queue.iter().filter(|p| p.public).map(WorkKey::of).collect();
    assert!(public.len() >= 2, "{tag} has {} public works", public.len());
    let private_slots: Vec<usize> = (0..queue.len()).filter(|&i| !queue[i].public).collect();
    let reversed: Vec<WorkKey> = public.iter().rev().copied().collect();
    s.submit(GameCommand::SetWorksOrder {
        tag: tag.clone(),
        order: Some(reversed.clone()),
    });
    run(&mut s, Date::new(1925, 3, 10));
    let w = s.sim.world();
    let after = &w.econ.markets[ci].queue;
    // The public works still there follow the fixed order.
    let now: Vec<WorkKey> = after.iter().filter(|p| p.public).map(WorkKey::of).collect();
    let listed: Vec<WorkKey> = reversed
        .iter()
        .filter(|k| now.contains(k))
        .copied()
        .collect();
    assert_eq!(
        &now[..listed.len()],
        &listed[..],
        "public works in the fixed order"
    );
    // Private works kept their relative order.
    let keys = |q: &[bf_sim::econ::state::Project]| -> Vec<WorkKey> {
        q.iter().filter(|p| !p.public).map(WorkKey::of).collect()
    };
    let before_private: Vec<WorkKey> = private_slots
        .iter()
        .map(|&i| WorkKey::of(&queue[i]))
        .collect();
    let after_private = keys(after);
    let kept: Vec<WorkKey> = before_private
        .iter()
        .filter(|k| after_private.contains(k))
        .copied()
        .collect();
    assert_eq!(
        &after_private[..kept.len()],
        &kept[..],
        "private works untouched"
    );
    // The screen reads the order.
    let lines = levers::works(&d.catalogue, &w.econ.markets[ci], w.levers.order(ci));
    assert!(lines.iter().all(|l| l.fixed));
    assert!(
        lines
            .iter()
            .all(|l| l.weeks >= 1 && (0.0..=1.0).contains(&l.progress))
    );
    assert!(lines.iter().all(|l| l.weeks_full >= 1
        && l.weeks_full <= l.weeks
        && (0.0..=1.0).contains(&l.met)
        && (0.0..=1.0).contains(&l.pace)));
}

#[test]
fn a_work_asked_for_is_queued_and_one_cancelled_leaves() {
    let d = data();
    let mut s = session(&d, Date::new(1925, 2, 3));
    let ci = market(&s, "BRA");
    let (region, kind, foreign) = {
        let w = s.sim.world();
        let offer = levers::buildable(&d.catalogue, w, d.tree.as_deref(), 1925, ci);
        let (region, kinds) = offer
            .into_iter()
            .find(|(_, k)| !k.is_empty())
            .expect("Brazil may build something somewhere");
        let foreign = w
            .econ
            .regions
            .iter()
            .position(|r| usize::from(r.market) != ci)
            .unwrap() as u32;
        (region, kinds[0].0, foreign)
    };
    let count = |s: &Session| {
        s.sim.world().econ.markets[ci]
            .queue
            .iter()
            .filter(|p| p.public && p.region == region && p.kind == kind)
            .count()
    };
    let before = count(&s);
    s.submit(GameCommand::AddWork {
        tag: "BRA".into(),
        region,
        kind,
    });
    // Not Brazil's state: dropped.
    s.submit(GameCommand::AddWork {
        tag: "BRA".into(),
        region: foreign,
        kind,
    });
    run(&mut s, Date::new(1925, 3, 10));
    assert_eq!(count(&s), before + 1, "the asked-for work is queued");
    assert!(s.sim.world().levers.requests.is_empty());
    let work = s.sim.world().econ.markets[ci]
        .queue
        .iter()
        .rev()
        .find(|p| p.public && p.region == region && p.kind == kind)
        .map(WorkKey::of)
        .unwrap();
    s.submit(GameCommand::CancelWork {
        tag: "BRA".into(),
        work,
    });
    run(&mut s, Date::new(1925, 3, 11));
    assert_eq!(count(&s), before, "the cancelled work left");
}

#[test]
fn the_order_survives_a_save() {
    let d = data();
    let mut s = session(&d, Date::new(1925, 3, 1));
    let (tag, ci) = busiest(&s);
    let order: Vec<WorkKey> = s.sim.world().econ.markets[ci]
        .queue
        .iter()
        .filter(|p| p.public)
        .map(WorkKey::of)
        .rev()
        .collect();
    s.submit(GameCommand::SetWorksOrder {
        tag,
        order: Some(order.clone()),
    });
    run(&mut s, Date::new(1925, 3, 3));
    let bytes = s.save("test", "works").unwrap();
    let loaded = Session::load(&bytes, 2, Some(d.rules())).unwrap();
    assert_eq!(loaded.sim.world().levers.order(ci), Some(&order[..]));
    assert_eq!(hash_of(loaded.sim.world()), hash_of(s.sim.world()));
}
