//! The composition of the forces in the forces tab fits every walk size in both languages and is
//! worked by its key tips: Germany in February 1926 with the war rule on, the target fixed by
//! key and each share stepped by key, sending `SetComposition` each time. Its own binary: the
//! rules switch on through `BF_RULES`, which only a process alone may write.

use bf_client::App;
use bf_client::app::Transition;
use bf_client::saves;
use bf_sim::game::GameCommand;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;

mod common;
use common::layout::*;

const RULES: &str = "forces,war";
const WALK_SIZES: [(f32, f32); 3] = [(960.0, 540.0), (1280.0, 720.0), (1600.0, 900.0)];

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
    s.submit(GameCommand::ChoosePlayer { tag: "GER".into() });
    while s.clock().date < Date::new(1926, 2, 1) {
        s.sim.step();
    }
    s
}

fn steps(h: &mut Harness<'_, App>) {
    for _ in 0..12 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
}

/// Types the key tip of `id` (the leader first); false when no tip for it is on the screen.
fn act(h: &mut Harness<'_, App>, id: &str) -> bool {
    if !h.state().shared.keys.hint_mode() {
        h.key_press(egui::Key::AltLeft);
        h.run_steps(3);
    }
    let Some(code) = h.state().shared.keys.code_of(id).map(str::to_owned) else {
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
        return false;
    };
    for c in code.chars() {
        h.key_press(egui::Key::from_name(&c.to_string()).expect("a key for the code"));
        h.run_steps(4);
    }
    steps(h);
    if h.state().shared.keys.hint_mode() {
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
    }
    true
}

#[test]
fn the_composition_fits_and_answers_its_keys() {
    // SAFETY: the only test of this binary; nothing else reads the environment meanwhile.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let session = session();
    let mut failures = Vec::new();
    for (locale, heading, summary) in [
        ("pt-BR", "Composição das forças", "Resumo"),
        ("en", "Composition of the forces", "Summary"),
    ] {
        let user = std::env::temp_dir().join(format!("bf-comp-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save =
            saves::create(&user.join("saves"), "Composição", 7, Date::new(1925, 1, 1)).unwrap();
        save.meta.player = Some("GER".into());
        save.meta.date = session.clock().date;
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "composition").unwrap(),
        )
        .unwrap();
        saves::write(&save).unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        h.run_steps(4);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        steps(&mut h);
        h.key_press(egui::Key::G);
        steps(&mut h);
        h.key_press(egui::Key::T);
        steps(&mut h);
        let Some(n) = h
            .query_all(egui_kittest::kittest::by().label(heading))
            .next()
        else {
            failures.push(format!("{locale}: no {heading:?} in the forces tab"));
            continue;
        };
        n.scroll_to_me();
        steps(&mut h);
        for id in [
            "forces.composition.fixed",
            "forces.composition.0.up",
            "forces.composition.1.up",
        ] {
            if !act(&mut h, id) {
                failures.push(format!("{locale}: no key tip for {id}"));
            }
        }
        // Back to the top of the tab: rows scrolled out above still report where they were;
        // below the fold the check lets a scrolling area hold them.
        h.query_all(egui_kittest::kittest::by().label(summary))
            .next()
            .expect("the summary")
            .scroll_to_me();
        for (w, hh) in WALK_SIZES {
            h.set_size(egui::vec2(w, hh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh} composition fixed");
            failures.extend(check(&h, &case));
            failures.extend(
                no_overlapping_text(&h, &case)
                    .into_iter()
                    .filter(|l| l.contains(" cut to ")),
            );
        }
        h.set_size(egui::vec2(1600.0, 900.0));
        steps(&mut h);
        h.query_all(egui_kittest::kittest::by().label(heading))
            .next()
            .expect("the heading")
            .scroll_to_me();
        steps(&mut h);
        if !act(&mut h, "forces.composition.rule") {
            failures.push(format!("{locale}: no key tip for the rule"));
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
