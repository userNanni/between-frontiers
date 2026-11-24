//! Print of the movement against the war, in the war window.
//! `cargo test -p bf-client --release --test shots_anti_war -- --ignored`; images in
//! `target/screenshots/anti-war/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "anti-war";
const RULES: &str = "forces,crisis,war,anti_war";

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

/// Poland at war with Czechoslovakia (the war's recipe), attacking since March, on 1 December
/// 1925: the movement against the war in Poland, with enough mobilized for the government to
/// fall below the peace threshold. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn anti_war() {
    use bf_sim::game::GameCommand;
    use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let switches = bf_sim::game::Switches::parse(RULES).expect("switches");
    let user = std::env::temp_dir().join(format!("bf-shots-antiwar-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    // The recipe of the war's thread: Poland, a second-generation army, given 20 thousand
    // cavalry (the forces of 1925 have none), demands a payment of Czechoslovakia and declares
    // on 20 March; the front fights for weeks, the cavalry on the weakest Czech segment.
    let mut data = bf_sim::data::GameData::load(&root()).expect("game data");
    data.switches = switches;
    let mil = data.war.as_ref().expect("war data").military.clone();
    let mut world = data.new_world();
    {
        let ty = mil.unit("cavalry").expect("cavalry");
        let spec = mil.spec(ty, 0);
        let pol = world
            .countries
            .iter()
            .position(|(_, c)| c.tag == "POL")
            .unwrap();
        world.war.countries[pol]
            .formations
            .push(bf_sim::war::Formation {
                unit: ty,
                generation: 0,
                elements: Fixed::from_int(20),
                colonial: false,
                called_up: false,
                equipment: spec
                    .equipment
                    .iter()
                    .map(|(g, q)| (*g, *q * Fixed::from_int(20)))
                    .collect(),
                training: Fixed::ONE,
                raised: false,
            });
    }
    let mut s =
        bf_sim::game::Session::new(world, 1925, Date::new(1925, 1, 1), 2, Some(data.rules()));
    s.submit(GameCommand::ChoosePlayer { tag: "POL".into() });
    while s.clock().date < Date::new(1925, 3, 2) {
        s.sim.step();
    }
    s.submit(GameCommand::Demand {
        tag: "POL".into(),
        target: "CZE".into(),
        demand: Proposal {
            side_a: vec!["POL".into()],
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
    while s.clock().date < Date::new(1925, 3, 20) {
        s.sim.step();
    }
    let id = s.world().war.conflicts.crises[0].id;
    s.submit(GameCommand::Declare {
        tag: "POL".into(),
        crisis: id,
        war: true,
    });
    s.submit(GameCommand::SetPosture {
        tag: "POL".into(),
        enemy: "CZE".into(),
        attack: true,
    });
    // Two weeks of combat; the segments are saved, their pace is not.
    while s.clock().date < Date::new(1925, 12, 1) {
        s.sim.step();
    }
    assert_eq!(s.world().war.conflicts.wars.len(), 1, "the war goes on");
    for (locale, size, tag, war, line) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Polônia",
            "Movimento contra a guerra",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Poland",
            "Movement against the war",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Guerra", "POL");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::J);
        settle(&mut h);
        press(&mut h, war);
        settle(&mut h);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(line))
            .next()
            .expect("the movement's line")
            .scroll_to_me();
        settle(&mut h);
        shot(&mut h, THEME, &format!("329-anti-war-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
