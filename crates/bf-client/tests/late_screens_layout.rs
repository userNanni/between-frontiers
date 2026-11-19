//! The screens that came after the layout matrix fit the screen, in both languages, and each
//! is reached by its key tip: Hungary in its first week of war with Czechoslovakia after a demand,
//! with Austria's proposal of an agreement on the table. The walk opens the government's forces,
//! works and debts tabs, the Agendas with the demand, the negotiation table and the agreements,
//! the war window from its line in the Agendas and its peace table, and the country panel with
//! its trade partners; every element lies on the screen and inside its window, no number is cut,
//! and, where one window stands alone, no text runs into another.
//! Its own binary: the rules switch on through `BF_RULES`, which only a process alone may write.

use bf_client::App;
use bf_client::app::Transition;
use bf_client::saves;
use bf_client::screens::Screen;
use bf_sim::game::GameCommand;
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use egui_kittest::Harness;
use laplace_schelling::calendar::Date;

mod common;
use common::layout::*;

const RULES: &str = "forces,crisis,war,negotiation,reputation";

/// The sizes the owner checks by eye, and the floor.
const WALK_SIZES: [(f32, f32); 3] = [(960.0, 540.0), (1280.0, 720.0), (1600.0, 900.0)];

fn clause(kind: ClauseKind, giver: Side, receiver: Side) -> Clause {
    Clause {
        kind,
        giver,
        receiver,
        until: None,
        note: None,
    }
}

fn session() -> bf_sim::game::Session {
    let mut data = bf_sim::data::GameData::load(&root()).expect("game data");
    data.switches = data.switches.with(RULES).expect("switches");
    let mut s = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(data.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "HUN".into() });
    while s.clock().date < Date::new(1925, 3, 2) {
        s.sim.step();
    }
    s.submit(GameCommand::Demand {
        tag: "HUN".into(),
        target: "CZE".into(),
        demand: Proposal {
            side_a: vec!["HUN".into()],
            side_b: vec!["CZE".into()],
            clauses: vec![clause(ClauseKind::Union, Side::Junior, Side::Senior)],
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
        tag: "HUN".into(),
        crisis: id,
        war: true,
    });
    s.submit(GameCommand::SetPosture {
        tag: "HUN".into(),
        enemy: "CZE".into(),
        attack: true,
    });
    s.submit(GameCommand::ProposeTreaty {
        tag: "AUT".into(),
        to: "HUN".into(),
        proposal: Proposal {
            side_a: vec!["AUT".into()],
            side_b: vec!["HUN".into()],
            clauses: vec![
                clause(ClauseKind::NonAggression, Side::Senior, Side::Junior),
                clause(ClauseKind::NonAggression, Side::Junior, Side::Senior),
            ],
            term_years: Some(10),
            notice_years: 2,
        },
    });
    // The first week of combat: the front, its segments, and the war still going.
    while s.world().war.fighting.last_week.is_empty() && s.clock().date < Date::new(1925, 5, 1) {
        s.sim.step();
    }
    assert!(
        !s.world().war.fighting.last_week.is_empty(),
        "the war has a front on {:?}",
        s.clock().date
    );
    s
}

fn steps(h: &mut Harness<'_, App>) {
    for _ in 0..12 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
}

/// Types the key tip of element `id` (the leader key first when the tips are off); false when
/// no tip for it is on the screen.
fn act(h: &mut Harness<'_, App>, id: &str) -> bool {
    if !h.state().shared.keys.hint_mode() {
        h.key_press(egui::Key::AltLeft);
        h.run_steps(3);
    }
    // A button below the fold has no tip until the page scrolls to it (Shift+PageDown).
    let mut scrolled = 0;
    let code = loop {
        if let Some(code) = h.state().shared.keys.code_of(id).map(str::to_owned) {
            break code;
        }
        if scrolled == 8 {
            h.key_press(egui::Key::Escape);
            h.run_steps(2);
            return false;
        }
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::PageDown);
        h.run_steps(4);
        h.key_press(egui::Key::AltLeft);
        h.run_steps(3);
        scrolled += 1;
    };
    for c in code.chars() {
        let key = egui::Key::from_name(&c.to_string()).expect("a key for the code");
        h.key_press(key);
        h.run_steps(4);
    }
    steps(h);
    if h.state().shared.keys.hint_mode() {
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
    }
    true
}

/// Checks the screen at every walk size, then goes back to 1600 × 900.
fn check_sizes(h: &mut Harness<'_, App>, locale: &str, name: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (w, hh) in WALK_SIZES {
        h.set_size(egui::vec2(w, hh));
        steps(h);
        let case = format!("{locale} {w}×{hh} {name}");
        out.extend(check(h, &case));
        // Over the HUD's panels (the war window), texts of two windows lie one over the other:
        // there only cut numbers count.
        let alone = !name.starts_with("war") && !name.starts_with("peace");
        out.extend(
            no_overlapping_text(h, &case)
                .into_iter()
                .filter(|l| alone || l.contains(" cut to ")),
        );
    }
    h.set_size(egui::vec2(1600.0, 900.0));
    steps(h);
    out
}

#[test]
fn the_late_screens_fit_and_answer_their_keys() {
    // SAFETY: the only test of this binary; nothing else reads the environment meanwhile.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let session = session();
    let war = session.world().war.conflicts.wars[0].id;
    let mut failures = Vec::new();
    for locale in ["pt-BR", "en"] {
        let user = std::env::temp_dir().join(format!("bf-late-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save = saves::create(
            &user.join("saves"),
            "Consistência",
            7,
            Date::new(1925, 1, 1),
        )
        .unwrap();
        save.meta.player = Some("HUN".into());
        save.meta.date = session.clock().date;
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "late").unwrap(),
        )
        .unwrap();
        saves::write(&save).unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        h.run_steps(4);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        let started = std::time::Instant::now();
        while h.state().shared.world.is_none() && started.elapsed().as_secs() < 20 {
            steps(&mut h);
        }
        steps(&mut h);
        let need = |h: &mut Harness<'_, App>, id: &str, failures: &mut Vec<String>| {
            if !act(h, id) {
                failures.push(format!("{locale}: no key tip for {id}"));
            }
        };
        // The government: forces, works, debts between governments.
        h.key_press(egui::Key::G);
        steps(&mut h);
        for (key, shift, name) in [
            (egui::Key::T, false, "forces"),
            (egui::Key::B, true, "works"),
            (egui::Key::M, true, "debts"),
        ] {
            if shift {
                h.key_press_modifiers(egui::Modifiers::SHIFT, key);
            } else {
                h.key_press(key);
            }
            steps(&mut h);
            failures.extend(check_sizes(&mut h, locale, &format!("government {name}")));
        }
        need(&mut h, "gov.close", &mut failures);
        // The Agendas: the demand, the table with Austria selected, the agreements, the crisis.
        h.key_press(egui::Key::J);
        steps(&mut h);
        failures.extend(check_sizes(&mut h, locale, "agendas"));
        need(&mut h, "journal.decision.exigency", &mut failures);
        failures.extend(check_sizes(&mut h, locale, "demand"));
        need(&mut h, "journal.decision.back", &mut failures);
        let austria = h
            .state()
            .shared
            .world
            .as_ref()
            .and_then(|w| w.countries.iter().position(|c| c.tag() == "AUT"));
        if let Screen::Game(g) = &mut h.state_mut().screen {
            g.select(austria);
        }
        steps(&mut h);
        need(&mut h, "journal.decision.treaty", &mut failures);
        failures.extend(check_sizes(&mut h, locale, "treaty table"));
        need(&mut h, "journal.decision.back", &mut failures);
        need(&mut h, "journal.decision.agreements", &mut failures);
        failures.extend(check_sizes(&mut h, locale, "agreements"));
        need(&mut h, "journal.decision.back", &mut failures);
        // The war's line opens the war window over the Agendas; its peace opens the table.
        need(&mut h, &format!("journal.war.{war}"), &mut failures);
        failures.extend(check_sizes(&mut h, locale, "war"));
        need(&mut h, "war.posture.CZE", &mut failures);
        need(&mut h, "war.peace", &mut failures);
        failures.extend(check_sizes(&mut h, locale, "peace table"));
        // The peace closed the war window; Shift+W opens it again, and its X closes it.
        need(&mut h, "journal.close", &mut failures);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::W);
        steps(&mut h);
        need(&mut h, "war.close", &mut failures);
        // The country panel with the trade partners.
        let me = h
            .state()
            .shared
            .world
            .as_ref()
            .and_then(|w| w.countries.iter().position(|c| c.tag() == "HUN"));
        if let Screen::Game(g) = &mut h.state_mut().screen {
            g.select(me);
        }
        steps(&mut h);
        failures.extend(check_sizes(&mut h, locale, "country panel"));
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
