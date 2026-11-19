//! Prints of the nuclear finish: the programme's test in two steps, the crisis's coercion and
//! the blocked declaration.
//! `cargo test -p bf-client --release --test shots_nuclear -- --ignored`; images in
//! `target/screenshots/nuclear/`.

#[path = "common/shots.rs"]
mod shots;

use bf_client::App;
use bf_client::app::Transition;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use shots::*;

const THEME: &str = "nuclear";

fn leader(h: &mut Harness<'_, App>) {
    h.key_press(egui::Key::AltLeft);
    h.run_steps(3);
}

/// Types a key tip and lets the element act.
fn tip(h: &mut Harness<'_, App>, key: egui::Key) {
    h.key_press(key);
    h.run_steps(4);
}

/// The nuclear finish: the United States (600 warheads, 300 bombers) with its programme running,
/// the test asked from the forces tab and waiting for its second step; its demand on the Soviet
/// Union (40 untested warheads), where its secure second strike coerces; and the Soviet Union's
/// demand on the United States, refused, where the nuclear ceiling blocks the declaration (the
/// American arsenal tested, so the Soviet Union reads it whole).
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn nuclear_finish() {
    use bf_sim::game::GameCommand;
    use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
    use laplace_schelling::Fixed;
    use laplace_schelling::calendar::Date;
    // SAFETY: the screenshot tests run one at a time (`RUST_TEST_THREADS=1`).
    unsafe { std::env::set_var("BF_RULES", "forces,crisis,war") };
    let user = std::env::temp_dir().join(format!("bf-shots-nuclear2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut data = bf_sim::data::GameData::load(&root()).expect("game data");
    data.switches = data.switches.with("forces,crisis,war").expect("switches");
    let mil = data.war.as_ref().expect("war data").military.clone();
    let world = |tested: bool| {
        let mut w = data.new_world();
        for (tag, key, n) in [
            ("USA", "nuclear_warhead", 600),
            ("USA", "strategic_bomber", 300),
            ("SOV", "nuclear_warhead", 40),
            ("SOV", "strategic_bomber", 20),
        ] {
            let c = w.countries.iter().position(|(_, c)| c.tag == tag).unwrap();
            w.war.countries[c].formations.push(bf_sim::war::Formation {
                unit: mil.unit(key).unwrap(),
                generation: 0,
                elements: Fixed::from_int(n),
                colonial: false,
                called_up: false,
                equipment: Vec::new(),
                training: Fixed::ONE,
                raised: true,
            });
        }
        // The American arsenal tested: known to all, as the Soviet Union reads it whole.
        if tested {
            let usa = w
                .countries
                .iter()
                .position(|(_, c)| c.tag == "USA")
                .unwrap() as u16;
            w.war.nuclear.tested.push(usa);
        }
        w
    };
    // A state of the Soviet Union's, for a demand on the periphery (where coercion counts).
    let soviet_state = {
        let w = data.new_world();
        let sov = w
            .countries
            .iter()
            .position(|(_, c)| c.tag == "SOV")
            .unwrap() as u16;
        let id = w
            .econ
            .regions
            .iter()
            .find(|r| r.market == sov && !r.colony)
            .map(|r| r.state)
            .unwrap();
        data.border_rules
            .state_ids
            .iter()
            .find(|(_, v)| **v == id)
            .map(|(k, _)| k.clone())
            .unwrap()
    };
    let demand = |by: &str, target: &str, kind: ClauseKind| GameCommand::Demand {
        tag: by.into(),
        target: target.into(),
        demand: Proposal {
            side_a: vec![by.into()],
            side_b: vec![target.into()],
            clauses: vec![Clause {
                kind,
                giver: Side::Junior,
                receiver: Side::Senior,
                until: None,
                note: None,
            }],
            term_years: None,
            notice_years: 0,
        },
        weeks: None,
    };
    // The United States: the programme, then its demand.
    let mut usa = bf_sim::game::Session::new(
        world(false),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    usa.submit(GameCommand::ChoosePlayer { tag: "USA".into() });
    usa.submit(GameCommand::NuclearProgramme {
        tag: "USA".into(),
        share: Some(5),
    });
    while usa.clock().date < Date::new(1925, 3, 2) {
        usa.sim.step();
    }
    let save_prog = write_campaign(&user.join("prog"), &usa, "Programa", "USA");
    // The demand, with the American arsenal tested: the Soviet Union reads it whole.
    let mut usa = bf_sim::game::Session::new(
        world(true),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    usa.submit(GameCommand::ChoosePlayer { tag: "USA".into() });
    while usa.clock().date < Date::new(1925, 3, 2) {
        usa.sim.step();
    }
    usa.submit(demand(
        "USA",
        "SOV",
        ClauseKind::CedeStates {
            states: vec![soviet_state.clone()],
        },
    ));
    while usa.clock().date < Date::new(1925, 3, 4) {
        usa.sim.step();
    }
    let usa_crisis = usa.world().war.conflicts.crises[0].id;
    // The Soviet Union: its demand, refused.
    let mut sov = bf_sim::game::Session::new(
        world(true),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    sov.submit(GameCommand::ChoosePlayer { tag: "SOV".into() });
    while sov.clock().date < Date::new(1925, 2, 2) {
        sov.sim.step();
    }
    sov.submit(demand("SOV", "USA", ClauseKind::Union));
    let refused = |s: &bf_sim::game::Session| {
        s.world()
            .war
            .conflicts
            .crises
            .iter()
            .any(|k| k.stage == bf_sim::war::Stage::Refused)
    };
    while !refused(&sov) && sov.clock().date < Date::new(1925, 5, 1) {
        sov.sim.step();
    }
    assert!(refused(&sov), "the United States refused");
    // The nuclear rung marks the blocked crises once a week.
    for _ in 0..7 {
        sov.sim.step();
    }
    let sov_crisis = sov.world().war.conflicts.crises[0].id;
    let save_usa = write_campaign(&user.join("usa"), &usa, "Dissuasão", "USA");
    let save_sov = write_campaign(&user.join("sov"), &sov, "Teto", "SOV");
    let keep = |h: &mut Harness<'_, App>, name: &str| shot(h, THEME, name);
    let act = |h: &mut Harness<'_, App>, id: &str| {
        for _ in 0..30 {
            leader(h);
            if let Some(code) = h.state().shared.keys.code_of(id).map(str::to_owned) {
                for c in code.chars() {
                    tip(h, egui::Key::from_name(&c.to_string()).expect("key"));
                }
                h.key_press(egui::Key::Escape);
                h.run_steps(3);
                return;
            }
            h.key_press(egui::Key::Escape);
            h.run_steps(2);
            h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
            h.run_steps(4);
        }
        panic!("no key tip for {id}");
    };
    let wait = |h: &mut Harness<'_, App>| {
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            h.run_steps(1);
        }
    };
    for (locale, size, tag, test_sure, coerced, blocked) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Confirmar o teste",
            "pode coagir",
            "declaração está bloqueada",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Confirm the test",
            "can coerce",
            "declaration is blocked",
        ),
    ] {
        let mut h = harness(user.join(format!("prog-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_prog.clone()), &ctx);
        wait(&mut h);
        // The forces tab: the programme, the test asked, its second step.
        h.key_press(egui::Key::G);
        h.run_steps(4);
        h.key_press(egui::Key::T);
        wait(&mut h);
        act(&mut h, "forces.programme.test");
        wait(&mut h);
        h.get_by_label(test_sure).scroll_to_me();
        wait(&mut h);
        keep(&mut h, &format!("261-programme-test-{tag}"));
        // The demand on the Soviet Union: coercion.
        let mut h = harness(user.join(format!("usa-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_usa.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::J);
        wait(&mut h);
        act(&mut h, &format!("journal.crisis.{usa_crisis}"));
        wait(&mut h);
        if let Some(n) = h
            .query_all(egui_kittest::kittest::by().label_contains(coerced))
            .next()
        {
            n.scroll_to_me();
        }
        wait(&mut h);
        keep(&mut h, &format!("262-crisis-coerced-{tag}"));
        // The Soviet Union, refused: the ceiling blocks the declaration.
        let mut h = harness(user.join(format!("sov-{tag}")), locale, size);
        let ctx = h.ctx.clone();
        h.state_mut()
            .apply(Transition::Game(save_sov.clone()), &ctx);
        wait(&mut h);
        h.key_press(egui::Key::J);
        wait(&mut h);
        act(&mut h, &format!("journal.crisis.{sov_crisis}"));
        wait(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(blocked))
            .next()
            .expect("the blocked declaration")
            .scroll_to_me();
        wait(&mut h);
        keep(&mut h, &format!("263-crisis-blocked-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
