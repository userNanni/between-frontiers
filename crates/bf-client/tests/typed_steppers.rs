//! A stepper's number can be typed: clicked, the value turns into a field; Enter applies a
//! number within the limits, keeps one outside them in the field, and Escape leaves the value
//! as it was.

mod common;

use bf_client::app::Transition;
use bf_client::saves;
use bf_client::screens::Screen;
use bf_client::screens::gov_ui::parse_typed;
use common::layout::app;
use egui_kittest::kittest::Queryable;

#[test]
fn a_number_is_read_as_typed() {
    assert_eq!(parse_typed("7,5"), Some(7.5));
    assert_eq!(parse_typed("7.5 %"), Some(7.5));
    assert_eq!(parse_typed(" 12 "), Some(12.0));
    assert_eq!(parse_typed("−3"), Some(-3.0));
    assert_eq!(parse_typed("+40"), Some(40.0));
    assert_eq!(parse_typed("sete"), None);
    assert_eq!(parse_typed(""), None);
}

fn steps(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..12 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.run_steps(1);
    }
}

fn rate(h: &egui_kittest::Harness<'_, bf_client::App>) -> Option<Option<u16>> {
    match &h.state().screen {
        Screen::Game(g) => g.government_drafts().and_then(|d| d.rate),
        _ => None,
    }
}

#[test]
fn the_policy_rate_is_typed() {
    let user = std::env::temp_dir().join(format!("bf-typed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut h = app(user.clone(), "en", egui::vec2(1600.0, 900.0));
    h.run_steps(4);
    let mut save = saves::create(
        &user.join("saves"),
        "Typed",
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some("SWE".into());
    saves::write(&save).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    let started = std::time::Instant::now();
    while h.state().shared.world.is_none() && started.elapsed().as_secs() < 20 {
        steps(&mut h);
    }
    steps(&mut h);
    h.key_press(egui::Key::G);
    steps(&mut h);
    h.key_press(egui::Key::M);
    steps(&mut h);
    let field = |h: &mut egui_kittest::Harness<'_, bf_client::App>| {
        h.query_all(egui_kittest::kittest::by().label_contains("Half a point per click"))
            .next()
            .unwrap_or_else(|| {
                let tab = match &h.state().screen {
                    Screen::Game(g) => format!("{:?}", g.government_tab()),
                    _ => "no game".into(),
                };
                let labels: Vec<String> = common::layout::nodes(h)
                    .into_iter()
                    .map(|n| n.label)
                    .filter(|l| l.to_lowercase().contains("rate"))
                    .collect();
                panic!("the policy rate's value; tab {tab}; {labels:?}")
            })
            .click();
        steps(h);
    };
    let type_in = |h: &mut egui_kittest::Harness<'_, bf_client::App>, text: &str| {
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.run_steps(1);
        h.event(egui::Event::Text(text.into()));
        h.run_steps(1);
    };
    // Typed and applied.
    field(&mut h);
    type_in(&mut h, "7,3");
    h.key_press(egui::Key::Enter);
    steps(&mut h);
    assert_eq!(rate(&h), Some(Some(73)), "7.3 % typed");
    // Out of the limits: refused, the field stays open.
    field(&mut h);
    type_in(&mut h, "250");
    h.key_press(egui::Key::Enter);
    steps(&mut h);
    assert_eq!(rate(&h), Some(Some(73)), "250 % refused");
    assert!(h.ctx.egui_wants_keyboard_input(), "the field stays open");
    // Escape: as it was.
    type_in(&mut h, "4");
    h.key_press(egui::Key::Escape);
    steps(&mut h);
    assert_eq!(rate(&h), Some(Some(73)), "Escape leaves it");
    let _ = std::fs::remove_dir_all(&user);
}
