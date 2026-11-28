//! Prints of the methods tab with a method fixed by the government (`SetMethod`, switch
//! `fixed_methods` on), and of a laissez-faire country, where the choice is off.
//! `cargo test -p bf-client --release --test shots_fixed_methods -- --ignored`; images in
//! `target/screenshots/methods/`.

#[path = "common/shots.rs"]
mod shots;

use laplace_schelling::calendar::Date;
use shots::*;

const RULES: &str = "fixed_methods";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// Switzerland (regulated market) fixes the shops in its commerce in 1925; in February 1926 its
/// states run them. Then the United States (laissez-faire), where the government may not fix.
/// Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn fixed_method() {
    let user = std::env::temp_dir().join(format!("bf-shots-fixed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let switches = data.switches.with(RULES).expect("switches");
    let kind = data
        .catalogue
        .buildings
        .iter()
        .position(|b| b.key == "trade_center")
        .expect("trade_center") as u16;
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let mut che = session("CHE", Date::new(1925, 1, 8), switches);
    che.submit(bf_sim::game::GameCommand::SetMethod {
        tag: "CHE".into(),
        kind,
        method: Some(0),
    });
    while che.clock().date < Date::new(1926, 2, 2) {
        che.sim.step();
    }
    let usa = session("USA", Date::new(1925, 3, 1), switches);
    for (locale, size, name, tab, building) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Métodos",
            "Comércio",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Methods",
            "Commerce",
        ),
    ] {
        for (s, tag) in [(&che, "CHE"), (&usa, "USA")] {
            let home = user.join(format!("{name}-{tag}"));
            let save = write_campaign(&home, s, "Métodos", tag);
            let mut h = harness(home.clone(), locale, size);
            open(&mut h, save);
            settle(&mut h);
            h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::E);
            h.run_steps(4);
            assert!(click_containing(&mut h, tab), "no methods tab");
            h.run_steps(4);
            assert!(click_containing(&mut h, building), "no {building} in {tag}");
            h.run_steps(4);
            let label = format!("172-fixed-methods-{}-{name}", tag.to_lowercase());
            shot(&mut h, "methods", &label);
        }
    }
    let _ = std::fs::remove_dir_all(&user);
}
