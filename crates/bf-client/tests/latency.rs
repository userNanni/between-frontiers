//! How fast the campaign answers while the simulation runs: the time from pressing pause to a
//! paused snapshot at speed 5, and the longest interface frame across monthly autosaves.
//!
//! Run with `cargo test -p bf-client --release --test latency -- --ignored --nocapture`. Needs a
//! GPU or a software Vulkan driver, like the screenshots.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bf_client::screens::Screen;
use bf_client::{App, app::Transition, paths::Paths, saves};
use egui_kittest::Harness;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn snapshot(h: &Harness<'_, App>) -> bf_client::sim::Snapshot {
    match &h.state().screen {
        Screen::Game(g) => g.snapshot(),
        _ => panic!("not in a campaign"),
    }
}

fn percentile(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

#[test]
#[ignore = "needs a GPU (or software Vulkan) and the game data; run with --ignored"]
fn pause_and_autosave_latency() {
    let user = std::env::temp_dir().join(format!("bf-latency-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        "(locale: \"en\", ui_scale: 100, map_labels: true, autosave: Monthly)",
    )
    .unwrap();
    let paths = Paths::new(&root(), user.clone(), user.clone());
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 900.0))
        .wgpu()
        .build_eframe(move |cc| App::new(cc, paths));
    let mut save = saves::create(
        &user.join("saves"),
        "latency",
        1925,
        laplace_schelling::calendar::Date::new(1925, 1, 1),
    )
    .unwrap();
    save.meta.player = Some("BRA".into());
    // A later campaign runs slower ticks: `BF_LATENCY_YEAR` (say 1940) starts there, stepping the
    // engine once and keeping the world in the temp folder for the next run.
    if let Some(year) = std::env::var("BF_LATENCY_YEAR")
        .ok()
        .and_then(|y| y.parse::<i32>().ok())
    {
        let cached = std::env::temp_dir().join(format!("bf-latency-{year}.bin"));
        if !cached.is_file() {
            let data = bf_sim::data::GameData::load(&root()).expect("game data");
            let mut session = bf_sim::game::Session::new(
                data.new_world(),
                save.meta.seed,
                save.meta.date,
                0,
                Some(data.rules()),
            );
            session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "BRA".into() });
            while session.clock().date < laplace_schelling::calendar::Date::new(year, 1, 1) {
                session.sim.step();
            }
            std::fs::write(&cached, session.save("test", "latency").unwrap()).unwrap();
        }
        std::fs::copy(&cached, save.dir.join(saves::WORLD_FILE)).unwrap();
        save.meta.date = laplace_schelling::calendar::Date::new(year, 1, 1);
    }
    saves::write(&save).unwrap();
    let ctx = h.ctx.clone();
    h.state_mut().apply(Transition::Game(save), &ctx);
    h.run_steps(3);

    h.key_press(egui::Key::Num5);
    h.run_steps(1);

    // Frames across monthly autosaves at speed 5.
    h.key_press(egui::Key::Space);
    let mut frames = Vec::new();
    let started = Instant::now();
    let first = snapshot(&h).date;
    while started.elapsed() < Duration::from_secs(12) {
        let t = Instant::now();
        h.run_steps(1);
        frames.push(t.elapsed().as_secs_f64() * 1e3);
        std::thread::sleep(Duration::from_millis(4));
    }
    let last = snapshot(&h).date;
    let months = (last.year - first.year) * 12 + i32::from(last.month) - i32::from(first.month);
    let mut sorted = frames.clone();
    println!(
        "frames: {} over {months} months ({first:?} -> {last:?}); median {:.1} ms, p99 {:.1} ms, max {:.1} ms",
        frames.len(),
        percentile(&mut sorted, 0.5),
        percentile(&mut sorted, 0.99),
        percentile(&mut sorted, 1.0),
    );
    let slow: Vec<String> = frames
        .iter()
        .filter(|f| **f > 50.0)
        .map(|f| format!("{f:.0}"))
        .collect();
    println!("frames over 50 ms: {}", slow.join(", "));

    // Pause at speed 5, ten times.
    let mut waits = Vec::new();
    for _ in 0..10 {
        if snapshot(&h).paused {
            h.key_press(egui::Key::Space);
        }
        let run = Instant::now();
        while run.elapsed() < Duration::from_millis(1500) {
            h.run_steps(1);
            std::thread::sleep(Duration::from_millis(4));
        }
        h.key_press(egui::Key::Space);
        let asked = Instant::now();
        h.run_steps(1);
        while !snapshot(&h).paused && asked.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(1));
        }
        waits.push(asked.elapsed().as_secs_f64() * 1e3);
        h.run_steps(1);
    }
    let mut sorted = waits.clone();
    println!(
        "pause: median {:.0} ms, max {:.0} ms ({})",
        percentile(&mut sorted, 0.5),
        percentile(&mut sorted, 1.0),
        waits
            .iter()
            .map(|w| format!("{w:.0}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = std::fs::remove_dir_all(&user);
}
