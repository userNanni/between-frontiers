//! Print of the Fronts tab of the war window, with the switch `fronts` on.
//! `cargo test -p bf-client --release --test shots_fronts -- --ignored`; images in
//! `target/screenshots/fronts/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "fronts";
const RULES: &str = "forces,crisis,war,fronts";

/// Clicks the first button whose label contains `text`, scrolled into sight.
fn press(h: &mut egui_kittest::Harness<'_, bf_client::App>, text: &str) {
    let query = || {
        egui_kittest::kittest::by()
            .label_contains(text)
            .role(egui::accesskit::Role::Button)
    };
    h.query_all(query())
        .next()
        .unwrap_or_else(|| panic!("button {text}"))
        .scroll_to_me();
    h.run_steps(4);
    h.query_all(query()).next().unwrap().click();
}

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The Soviet Union at war with Poland over a payment (the war's recipe), attacking from 20
/// March, with an objective on the front (no share of its own: it takes what the reserve leaves)
/// and a tenth of the army in reserve: the Fronts tab a week after the orders. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn fronts_tab() {
    use bf_sim::game::GameCommand;
    use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let switches = bf_sim::game::Switches::parse(RULES).expect("switches");
    let user = std::env::temp_dir().join(format!("bf-shots-fronts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut data = data;
    data.switches = switches;
    let mut s = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "SOV".into() });
    while s.clock().date < Date::new(1925, 3, 2) {
        s.sim.step();
    }
    s.submit(GameCommand::Demand {
        tag: "SOV".into(),
        target: "POL".into(),
        demand: Proposal {
            side_a: vec!["SOV".into()],
            side_b: vec!["POL".into()],
            clauses: vec![Clause {
                kind: ClauseKind::Payment {
                    amount: laplace_schelling::Fixed::from_int(100),
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
    while s.clock().date < Date::new(1925, 3, 20) {
        s.sim.step();
    }
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "SOV".into(),
        crisis: id,
        war: true,
    });
    s.submit(GameCommand::SetPosture {
        tag: "SOV".into(),
        enemy: "POL".into(),
        attack: true,
    });
    // A week of combat, then the objective (the Polish state of the first segment) and a tenth
    // in reserve, and a week more.
    while s.world().war.fighting.last_week.is_empty() && s.clock().date < Date::new(1925, 5, 1) {
        s.sim.step();
    }
    let objective = {
        let w = s.world();
        let pol = w
            .countries
            .iter()
            .position(|(_, c)| c.tag == "POL")
            .unwrap() as u16;
        let g = w
            .war
            .fighting
            .last_week
            .iter()
            .find(|g| g.a == pol || g.b == pol)
            .expect("a segment with Poland");
        let region = if g.a == pol { g.from } else { g.into };
        w.econ.regions[region as usize].state
    };
    s.submit(GameCommand::SetFrontObjective {
        tag: "SOV".into(),
        enemy: "POL".into(),
        state: Some(objective),
    });
    s.submit(GameCommand::SetReserve {
        tag: "SOV".into(),
        share: 100,
    });
    let until = Date::from_days(s.clock().date.to_days() + 7);
    while s.clock().date < until {
        s.sim.step();
    }
    for (locale, size, tag, war, tab) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Polônia",
            "Frentes",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Poland",
            "Fronts",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Frentes", "SOV");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::J);
        settle(&mut h);
        press(&mut h, war);
        settle(&mut h);
        settle(&mut h);
        assert!(click_containing(&mut h, tab), "the Fronts tab");
        settle(&mut h);
        shot(&mut h, THEME, &format!("333-fronts-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
