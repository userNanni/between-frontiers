//! A decision holds at once, even paused (`Session::submit_as`, `Simulation::apply_due`): with
//! the client's simulation thread paused, a work asked for shows in the works view of the next
//! snapshot, the date and tick unchanged.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bf_client::sim::{Command, SimHandle, Snapshot};
use bf_sim::game::GameCommand;
use laplace_schelling::calendar::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The first snapshot `ok` accepts within ten seconds.
fn wait(sim: &SimHandle, ok: impl Fn(&Snapshot) -> bool) -> Option<Snapshot> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        let s = sim.snapshot();
        if ok(&s) {
            return Some(s);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    None
}

#[test]
fn a_work_asked_for_while_paused_shows_at_once() {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        1,
        Some(rules.clone()),
    );
    session.submit(GameCommand::ChoosePlayer { tag: "FRA".into() });
    while session.clock().date < Date::new(1925, 1, 9) {
        session.sim.step();
    }
    let w = session.world();
    let ci = w
        .countries
        .iter()
        .position(|(_, c)| c.tag == "FRA")
        .unwrap();
    let region = w
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
    let sim = SimHandle::start(session, Some(rules), Default::default(), || {});
    let before = wait(&sim, |s| s.works.is_some()).expect("a works view");
    assert!(before.paused);
    assert!(before.works.as_ref().unwrap().asked.is_empty());
    sim.send(Command::Issue(GameCommand::AddWork {
        tag: "FRA".into(),
        region,
        kind,
    }));
    let after = wait(&sim, |s| {
        s.works.as_ref().is_some_and(|w| !w.asked.is_empty())
    })
    .expect("the work asked for shows while paused");
    assert_eq!(after.tick, before.tick, "paused: no tick ran");
    assert_eq!(after.date, before.date);
    let asked = &after.works.as_ref().unwrap().asked;
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].building, "school");
}

#[path = "common/layout.rs"]
mod layout;

/// The work asked for is saved with the campaign (the save holds the command applied at once)
/// and, loaded paused, Government → Works lists it as asked for.
#[test]
fn the_works_screen_lists_a_work_asked_for_while_paused() {
    use bf_client::app::Transition;
    use bf_client::saves;
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        1,
        Some(data.rules()),
    );
    session.submit(GameCommand::ChoosePlayer { tag: "FRA".into() });
    while session.clock().date < Date::new(1925, 1, 9) {
        session.sim.step();
    }
    let ci = session
        .world()
        .countries
        .iter()
        .position(|(_, c)| c.tag == "FRA")
        .unwrap();
    let region = session
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
    session.submit(GameCommand::AddWork {
        tag: "FRA".into(),
        region,
        kind,
    });
    assert_eq!(session.world().levers.requests.len(), 1, "held at once");
    let user = std::env::temp_dir().join(format!("bf-commands-now-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let mut save = saves::create(&user.join("saves"), "Agora", 7, Date::new(1925, 1, 1)).unwrap();
    save.meta.player = Some("FRA".into());
    std::fs::write(
        save.dir.join(saves::WORLD_FILE),
        session.save("test", "now").unwrap(),
    )
    .unwrap();
    saves::write(&save).unwrap();
    let mut h = layout::app(user.clone(), "pt-BR", egui::vec2(1600.0, 900.0));
    h.run_steps(4);
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(25));
        h.run_steps(1);
    }
    h.key_press(egui::Key::G);
    h.run_steps(4);
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::B);
    h.run_steps(4);
    let asked: Vec<String> = layout::nodes(&h)
        .into_iter()
        .map(|n| n.label)
        .filter(|l| l.starts_with("Pedida:"))
        .collect();
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert!(asked[0].contains("Escola"), "{asked:?}");
    let _ = std::fs::remove_dir_all(&user);
}
