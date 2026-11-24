//! Print of a work stopped on occupied land, on the works screen, with the switch
//! `occupied_works` on.
//! `cargo test -p bf-client --release --test shots_works_occupied -- --ignored`; images in
//! `target/screenshots/works/`.

#[path = "common/shots.rs"]
mod shots;

use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "works";
const RULES: &str = "occupied_works";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// France in March 1925 with the region of its first public work held by Germany (put in by
/// hand, as an occupation leaves it): that work waits, "stopped: occupied territory", the others
/// go on. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn works_on_occupied_land() {
    let user = std::env::temp_dir().join(format!("bf-shots-occupied-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches
        .with(RULES)
        .expect("switches");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let mut s = session("FRA", Date::new(1925, 3, 2), switches);
    {
        let w = s.sim.world_mut();
        let index = |t: &str| w.countries.iter().position(|(_, c)| c.tag == t).unwrap();
        let (fra, ger) = (index("FRA"), index("GER") as u16);
        let region = w.econ.markets[fra]
            .queue
            .iter()
            .find(|p| p.public)
            .expect("a public work")
            .region;
        w.war.fighting.control.push((region, ger));
        w.war.fighting.control.sort_by_key(|(r, _)| *r);
    }
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
        shot(&mut h, THEME, &format!("331-works-occupied-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
