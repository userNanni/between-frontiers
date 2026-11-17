//! What a save costs: size, time to write and to read back, and an autosave against the time a
//! month of simulation takes, in 1925, 1975 and 2025. Long (a century of simulation): run with
//! `cargo test -p bf-client --release --test save_cost -- --ignored --nocapture`.

use std::path::PathBuf;
use std::time::Instant;

use laplace_schelling::calendar::Date;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
#[ignore = "simulates a century; run with --ignored"]
fn save_cost_by_decade() {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let threads = 2;
    let mut session = bf_sim::game::Session::new(
        data.new_world(),
        1925,
        Date::new(1925, 1, 1),
        threads,
        Some(data.rules()),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "BRA".into() });
    let dir = std::env::temp_dir().join(format!("bf-save-cost-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    println!("| year | MB | write ms | load ms | month ms | autosave / month |");
    println!("|---|---|---|---|---|---|");
    for year in [1925, 1975, 2025] {
        // Up to three months before the point, then time three months.
        let before = Date::new(year, 1, 1);
        let mut d = Date::new(year - 1, 10, 1);
        if year == 1925 {
            d = Date::new(1925, 1, 1);
        }
        while session.clock().date < d {
            session.sim.step();
        }
        let start = Instant::now();
        let target = if year == 1925 {
            Date::new(1925, 4, 1)
        } else {
            before
        };
        while session.clock().date < target {
            session.sim.step();
        }
        let month_ms = start.elapsed().as_secs_f64() * 1000.0 / 3.0;
        // Write as the client does: serialise, write to a temporary file, rename.
        let path = dir.join(format!("world-{year}.lss"));
        let mut write_ms = f64::MAX;
        let mut bytes_len = 0;
        for _ in 0..3 {
            let t = Instant::now();
            let bytes = session.save("bench", "cost").unwrap();
            let tmp = path.with_extension("tmp");
            std::fs::write(&tmp, &bytes).unwrap();
            std::fs::rename(&tmp, &path).unwrap();
            write_ms = write_ms.min(t.elapsed().as_secs_f64() * 1000.0);
            bytes_len = bytes.len();
        }
        let mut load_ms = f64::MAX;
        for _ in 0..3 {
            let t = Instant::now();
            let bytes = std::fs::read(&path).unwrap();
            let loaded = bf_sim::game::Session::load(&bytes, threads, Some(data.rules())).unwrap();
            load_ms = load_ms.min(t.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(loaded.clock().date, session.clock().date);
        }
        println!(
            "| {year} | {:.1} | {:.0} | {:.0} | {:.0} | {:.1} % |",
            bytes_len as f64 / 1e6,
            write_ms,
            load_ms,
            month_ms,
            write_ms / month_ms * 100.0
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
