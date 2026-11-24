//! Prints of the reserve by cohort (Forces tab) and of the decision to lengthen the service
//! (Agendas), with the switch `reserve_cohorts` on.
//! `cargo test -p bf-client --release --test shots_reserve -- --ignored`; images in
//! `target/screenshots/reserve/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "reserve";
const RULES: &str = "forces,crisis,war,reserve_cohorts";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

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

/// France in February 1926, a January after the start: the reserve by cohort on the Forces tab
/// (18 months of service, 28 years in the reserve), and the Agendas' decision to lengthen the
/// service at its second step, 24 months asked. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn reserve_and_service() {
    let user = std::env::temp_dir().join(format!("bf-shots-reserve-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches
        .with(RULES)
        .expect("switches");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let s = session("FRA", Date::new(1926, 2, 2), switches);
    assert!(
        s.world()
            .war
            .reserve
            .of(s
                .world()
                .countries
                .iter()
                .position(|(_, c)| c.tag == "FRA")
                .unwrap() as u16)
            .is_some(),
        "France has a reserve by cohort"
    );
    for (locale, size, tag, reserve, entry, ask, more) in [
        (
            "pt-BR",
            egui::vec2(1600.0, 900.0),
            "pt",
            "Saída em",
            "Forças armadas em paz",
            "Prolongar o serviço",
            "+",
        ),
        (
            "en",
            egui::vec2(1280.0, 720.0),
            "en-1280",
            "Left in",
            "Armed forces in peace",
            "Lengthen the service",
            "+",
        ),
    ] {
        let home = user.join(tag);
        let save = write_campaign(&home, &s, "Reserva", "FRA");
        let mut h = harness(home, locale, size);
        open(&mut h, save);
        settle(&mut h);
        h.key_press(egui::Key::G);
        settle(&mut h);
        h.key_press(egui::Key::T);
        settle(&mut h);
        h.query_all(egui_kittest::kittest::by().label_contains(reserve))
            .next()
            .expect("the cohorts' table")
            .scroll_to_me();
        settle(&mut h);
        shot(&mut h, THEME, &format!("325-reserve-cohorts-{tag}"));
        h.key_press(egui::Key::Escape);
        settle(&mut h);
        h.key_press(egui::Key::J);
        settle(&mut h);
        let node = || egui_kittest::kittest::by().label_contains(entry);
        h.query_all(node())
            .last()
            .expect("the forces' entry")
            .scroll_to_me();
        h.run_steps(4);
        h.query_all(node()).last().unwrap().click();
        settle(&mut h);
        press(&mut h, ask);
        settle(&mut h);
        press(&mut h, more);
        settle(&mut h);
        shot(&mut h, THEME, &format!("326-service-decision-{tag}"));
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
