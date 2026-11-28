//! Print of Government → Works with the game paused right after asking for a work: the school
//! is the last line of the queue, at once (`AddWork` checked at the click, #392).
//! `cargo test -p bf-client --release --test shots_works_now -- --ignored`; images in
//! `target/screenshots/works/`.

#[path = "common/shots.rs"]
mod shots;

use laplace_schelling::calendar::Date;
use shots::*;

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn a_work_asked_for_while_paused() {
    let user = std::env::temp_dir().join(format!("bf-shots-works-now-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut s = session("FRA", Date::new(1925, 3, 2), data.switches);
    let ci = s
        .world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == "FRA")
        .unwrap();
    let region = s
        .world()
        .econ
        .regions
        .iter()
        .position(|r| usize::from(r.market) == ci && !r.colony)
        .unwrap() as u32;
    let kind = data
        .catalogue
        .buildings
        .iter()
        .position(|b| b.key == "school")
        .unwrap() as u16;
    s.submit(bf_sim::game::GameCommand::AddWork {
        tag: "FRA".into(),
        region,
        kind,
    });
    for (locale, size, tag) in [
        ("pt-BR", egui::vec2(1600.0, 900.0), "pt"),
        ("en", egui::vec2(1280.0, 720.0), "en-1280"),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Obras", "FRA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::B);
        settle(&mut h);
        // The queue's end, where the new work is.
        h.input_mut()
            .events
            .push(egui::Event::PointerMoved(egui::pos2(
                size.x / 2.0,
                size.y / 2.0,
            )));
        h.run_steps(1);
        for _ in 0..3 {
            h.input_mut().events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -300.0),
                modifiers: egui::Modifiers::NONE,
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Wheel,
            });
            h.run_steps(1);
        }
        settle(&mut h);
        shot(&mut h, "works", &format!("392-works-asked-paused-{tag}"));
    }
    let _ = std::fs::remove_dir_all(&user);
}
