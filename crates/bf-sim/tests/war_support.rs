//! War support (G4), the political surrender, and the occupation (G3) on the real scenario:
//! Hungary, played, declares on Czechoslovakia with `forces`, `crisis` and `war` on.

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, GameWorld, Session};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use bf_sim::war::support::Fall;
use laplace_schelling::{Date, Fixed};

fn data(edit: impl FnOnce(&mut bf_sim::war::support::ConductFile)) -> GameData {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut d = GameData::load(&root).unwrap_or_else(|e| panic!("{e}"));
    d.enable_crisis();
    d.switches.war = true;
    edit(d.war.as_mut().unwrap().conduct.as_mut().unwrap());
    d
}

/// The overrun rule off: the war goes on however much the front takes.
fn no_overrun(c: &mut bf_sim::war::support::ConductFile) {
    c.support.overrun = 2000;
    c.support.capital_overrun = 2000;
}

fn index(w: &GameWorld, tag: &str) -> u16 {
    w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
}

/// Hungary, played, demands Czechoslovakia whole, hears the refusal and declares.
fn at_war(d: &GameData) -> Session {
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "HUN".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
            clauses: vec![Clause {
                kind: ClauseKind::Payment {
                    amount: Fixed::from_int(100),
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
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    s
}

/// At war each side has a support, term by term: the one declared on starts higher, and the
/// casualties and the weeks pull it down.
#[test]
fn each_side_has_a_support_term_by_term() {
    let d = data(no_overrun);
    let mut s = at_war(&d);
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    let (hun, cze) = (index(w, "HUN"), index(w, "CZE"));
    let f = &w.war.fighting;
    let (h, c) = (f.support_of(hun).unwrap(), f.support_of(cze).unwrap());
    assert_eq!(h.terms.start, 400, "Hungary declared without a cause");
    assert_eq!(c.terms.start, 800, "Czechoslovakia was declared on");
    assert!(h.terms.duration < 0 && h.weeks > 4, "{h:?}");
    assert!(h.value < 400 && h.value > 0, "{h:?}");
    assert!(!h.groups.is_empty());
}

/// Below the collapse the countdown runs, then the government falls: Hungary's form of
/// government gives way to a republic (a change of regime the treaties read), and the government
/// that follows gives the war up.
#[test]
fn the_collapse_brings_the_government_down_and_ends_the_war() {
    let d = data(|c| {
        no_overrun(c);
        c.support.collapse = 1000;
    });
    let mut s = at_war(&d);
    let hun = index(s.world(), "HUN");
    let changes_before = s.world().politics.changes.len();
    s.sim.run_until(Date::new(1925, 4, 10));
    let f = &s.world().war.fighting;
    assert!(
        f.support_of(hun).is_some_and(|x| x.countdown.is_some()),
        "the countdown runs: {:?}",
        f.support_of(hun)
    );
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    let fallen: Vec<_> = w.politics.changes[changes_before..]
        .iter()
        .filter(|r| r.country == hun)
        .collect();
    assert!(
        !w.war
            .conflicts
            .wars
            .iter()
            .any(|x| x.side_a.contains(&hun) || x.side_b.contains(&hun)),
        "the war is given up"
    );
    // Hungary was a kingdom (a constitutional monarchy, the regency of Horthy): a war revolution
    // brings in a parliamentary republic, as in 1918.
    let p = d.politics.as_ref().unwrap();
    let r = fallen.first().expect("a change of the form of government");
    assert_eq!(
        usize::from(r.from),
        p.law("constitutional_monarchy").unwrap()
    );
    assert_eq!(usize::from(r.to), p.law("parliamentary_republic").unwrap());
    assert!(matches!(r.by, bf_sim::polity::Enactor::Movement(_)));
    assert_eq!(
        w.war.fighting.support_of(hun).and_then(|x| x.fallen),
        None,
        "out of the war, out of the support"
    );
    let _ = Fall::Cabinet;
}

/// With "the player is never forced" (A3's alternative) the countdown does not run for the
/// player's country.
#[test]
fn the_player_is_not_forced_when_the_data_say_so() {
    let d = data(|c| {
        no_overrun(c);
        c.support.collapse = 1000;
        c.support.player_forced = false;
    });
    let mut s = at_war(&d);
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    let hun = index(w, "HUN");
    let x = w.war.fighting.support_of(hun).unwrap();
    assert!(x.countdown.is_none() && x.fallen.is_none(), "{x:?}");
    assert!(
        w.war.conflicts.wars.iter().any(|x| x.side_a.contains(&hun)),
        "the war goes on"
    );
}

/// The AI declared on, below the peace threshold, asks for peace at the table with part of the
/// demand.
#[test]
fn the_ai_below_the_peace_threshold_asks_for_peace() {
    let d = data(|c| {
        no_overrun(c);
        c.support.peace = 1000;
    });
    let mut s = at_war(&d);
    s.sim.run_until(Date::new(1925, 4, 10));
    let w = s.world();
    assert!(
        w.treaties
            .table
            .open
            .iter()
            .chain(std::iter::empty())
            .any(|p| p.from == "CZE" && p.to == "HUN")
            || w.treaties
                .table
                .closed
                .iter()
                .any(|p| p.from == "CZE" && p.to == "HUN"),
        "{:?}",
        w.treaties.table
    );
    let _ = Fall::Cabinet;
}

/// What the front takes is occupied: the occupier keeps a garrison, takes a share of the GDP,
/// and the combat leaves wrecks.
#[test]
fn the_occupier_garrisons_and_takes_a_share() {
    let d = data(no_overrun);
    let mut s = at_war(&d);
    s.sim.run_until(Date::new(1925, 7, 1));
    let w = s.world();
    let f = &w.war.fighting;
    assert!(!f.control.is_empty(), "the front moved");
    assert_eq!(f.occupied.len(), f.control.len());
    assert!(
        f.occupied.iter().any(|o| o.taken.is_positive()),
        "{:?}",
        f.occupied
    );
    assert!(
        f.occupied.iter().all(|o| o.garrison > 0),
        "{:?}",
        f.occupied
    );
    assert!(!f.wrecks.is_empty(), "the combat leaves wrecks");
}

/// The cost of a great war (design decision 6: a world war under 15 % of the tick): Germany,
/// Italy, Hungary and the Soviet Union against France, Britain, Poland, Czechoslovakia and
/// Yugoslavia, a year of fighting, the forces system's share of the tick by the clock in each
/// system. The AI does not declare yet, so the war is put in by hand. Run alone, in the bench
/// slot: `bf-bench-slot bf-cargo test --release -p bf-sim --test war_support -- --ignored`.
#[test]
#[ignore]
fn a_great_war_costs_under_a_seventh_of_the_tick() {
    // Nobody asks for peace: the war lasts the year, the front moves and the occupation runs.
    let d = data(|c| {
        no_overrun(c);
        c.support.peace = 0;
        c.support.collapse = 0;
    });
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.sim.run_until(Date::new(1925, 3, 2));
    {
        let w = s.sim.world_mut();
        let ix = |w: &GameWorld, t: &str| index(w, t);
        let a: Vec<u16> = ["GER", "ITA", "HUN", "SOV"]
            .iter()
            .map(|t| ix(w, t))
            .collect();
        let b: Vec<u16> = ["FRA", "GBR", "POL", "CZE", "YUG"]
            .iter()
            .map(|t| ix(w, t))
            .collect();
        let id = w.war.conflicts.next_id;
        w.war.conflicts.next_id += 1;
        w.war.conflicts.wars.push(bf_sim::war::WarOn {
            id,
            side_a: a.clone(),
            side_b: b.clone(),
            crisis: u32::MAX,
            demand: Proposal {
                side_a: vec!["GER".into()],
                side_b: vec!["FRA".into()],
                clauses: Vec::new(),
                term_years: None,
                notice_years: 0,
            },
            since: (1925, 3, 2),
        });
        for c in a.iter().chain(&b) {
            w.war.countries[*c as usize].mobilized = true;
        }
    }
    let before = s.sim.schedule().stats();
    s.sim.run_until(Date::new(1926, 3, 1));
    let after = s.sim.schedule().stats();
    let spent = |name: &str| -> f64 {
        let t = |v: &[laplace_schelling::schedule::SystemStats]| {
            v.iter()
                .find(|x| x.name == name)
                .map_or(0.0, |x| (x.compute + x.apply).as_secs_f64())
        };
        t(&after) - t(&before)
    };
    let total: f64 = after.iter().map(|x| spent(x.name)).sum();
    let forces = spent("forces");
    let w = s.world();
    let segments = w.war.fighting.last_week.len();
    let engaged = w
        .war
        .fighting
        .last_week
        .iter()
        .filter(|g| g.outcome.engaged)
        .count();
    println!("engaged segments in the last week: {engaged}");
    for g in w
        .war
        .fighting
        .last_week
        .iter()
        .filter(|g| g.outcome.engaged)
        .take(12)
    {
        let tag = |m: u16| w.countries.iter().nth(m as usize).unwrap().1.tag.clone();
        println!(
            "  {} v {}: power {:?}, men {} v {}, losses {:?}, km {:?}",
            tag(g.a),
            tag(g.b),
            g.outcome.power,
            g.outcome.terms[0].number,
            g.outcome.terms[1].number,
            g.outcome.losses,
            g.outcome.advance_km
        );
    }
    let adv: Vec<_> = w.war.fighting.advance.iter().map(|x| x.2).collect();
    println!("advances held: {adv:?}");
    println!(
        "wars at the end: {}, support: {:?}",
        w.war.conflicts.wars.len(),
        w.war
            .fighting
            .support
            .iter()
            .map(|x| (x.country, x.value, x.weeks))
            .collect::<Vec<_>>()
    );
    let share = forces / total;
    println!(
        "great war, a year: forces {:.1} ms of {:.1} ms in the systems ({:.1} %), {} segments in the last week, {} regions occupied",
        forces * 1000.0,
        total * 1000.0,
        share * 100.0,
        segments,
        w.war.fighting.control.len()
    );
    assert_eq!(w.war.conflicts.wars.len(), 1, "the war lasts the year");
    assert!(segments > 20, "a great war has many segments: {segments}");
    assert!(share < 0.15, "{:.1} %", share * 100.0);
}

/// Czechoslovakia, played, demands Hungary whole (a demand never yielded), declares and attacks. The front takes
/// the whole of Hungary (in the war screen's case, in a month): overrun with no ally, the
/// Hungarian government falls at once (the kingdom gives way to a republic) and the one that
/// follows asks for peace at the table, with the whole demand. No war left standing without a
/// front.
#[test]
fn hungary_overrun_asks_for_peace() {
    let d = data(|_| {});
    let mut s = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "CZE".into() });
    s.sim.run_until(Date::new(1925, 3, 2));
    s.submit(GameCommand::Demand {
        tag: "CZE".into(),
        target: "HUN".into(),
        demand: Proposal {
            side_a: vec!["CZE".into()],
            side_b: vec!["HUN".into()],
            clauses: vec![Clause {
                kind: ClauseKind::Union,
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
    s.sim.run_until(Date::new(1925, 3, 20));
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "CZE".into(),
        crisis: id,
        war: true,
    });
    s.sim.step();
    s.submit(GameCommand::SetPosture {
        tag: "CZE".into(),
        enemy: "HUN".into(),
        attack: true,
    });
    let hun = index(s.world(), "HUN");
    let mut overrun_on = None;
    for _ in 0..30 {
        s.sim
            .run_until(Date::from_days(s.clock().date.to_days() + 7));
        let w = s.world();
        if w.war
            .fighting
            .support_of(hun)
            .is_some_and(|x| x.fallen.is_some())
        {
            overrun_on = Some(s.clock().date);
            break;
        }
    }
    let on = overrun_on.expect("Hungary overrun, its government falls");
    let w = s.world();
    let x = w.war.fighting.support_of(hun).unwrap();
    assert!(matches!(x.fallen, Some(Fall::Regime { .. })), "{x:?}");
    // The government that follows asks for peace with the whole demand, the week it falls or
    // the next.
    s.sim.run_until(Date::from_days(on.to_days() + 14));
    let w = s.world();
    assert!(
        w.treaties
            .table
            .open
            .iter()
            .any(|p| p.from == "HUN" && p.to == "CZE" && p.proposal.clauses.len() == 1),
        "{:?}",
        w.treaties.table.open
    );
    println!("Hungary overrun and fallen on {on:?}");
}

/// The war screen's case: Hungary, played, declares on Czechoslovakia and is overrun within
/// weeks. Its government falls and gives the war up: no war is left open without a front.
#[test]
fn a_war_is_not_left_open_without_a_front() {
    let d = data(|_| {});
    let mut s = at_war(&d);
    let hun = index(s.world(), "HUN");
    s.sim.run_until(Date::new(1925, 6, 1));
    let w = s.world();
    assert!(
        !w.war
            .conflicts
            .wars
            .iter()
            .any(|x| x.side_a.contains(&hun) || x.side_b.contains(&hun)),
        "{:?}",
        w.war.conflicts.wars
    );
    assert!(w.war.fighting.control.is_empty(), "the occupation ended");
}

/// The pairs at war for the trade (CT7): Hungary against Czechoslovakia and the allies that
/// joined it, each pair once with the lower index first; none in peace.
#[test]
fn the_pairs_at_war_for_the_trade() {
    let d = data(|_| {});
    let s0 = Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    );
    assert!(bf_sim::war::fight::war_pairs(s0.world()).is_empty());
    let s = at_war(&d);
    let w = s.world();
    let (hun, cze) = (index(w, "HUN"), index(w, "CZE"));
    let pairs = bf_sim::war::fight::war_pairs(w);
    assert!(pairs.contains(&(hun.min(cze), hun.max(cze))), "{pairs:?}");
    assert!(pairs.windows(2).all(|x| x[0] < x[1]));
    assert!(pairs.iter().all(|(a, b)| a < b));
}

/// No blockade before the war at sea (slice C2): the trade's reader exists and is empty, at war
/// too.
#[test]
fn no_blockade_before_the_war_at_sea() {
    let d = data(|_| {});
    let s = at_war(&d);
    assert!(bf_sim::war::fight::blockaded(s.world()).is_empty());
}
