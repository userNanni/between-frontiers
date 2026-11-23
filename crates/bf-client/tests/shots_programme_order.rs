//! Prints of the warhead order in the nuclear programme on the forces tab.
//! `cargo test -p bf-client --release --test shots_programme_order -- --ignored`; images in
//! `target/screenshots/nuclear/`.

#[path = "common/shots.rs"]
mod shots;

use egui_kittest::kittest::Queryable;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;
use shots::*;

const THEME: &str = "nuclear";
const RULES: &str = "forces,crisis,war,market_stock";

fn settle(h: &mut egui_kittest::Harness<'_, bf_client::App>) {
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        h.run_steps(1);
    }
}

/// The United States in March 1925 with a programme of 0.5 % of GDP, given a core plant and an
/// assembly plant of one level each and a warhead order holding three cores and one assembly:
/// what holds the warheads back and the open order; then the programme ended a week later, the
/// order stopped with what it holds. Portuguese at 1600 × 900, English at 1280 × 720.
#[test]
#[ignore = "needs a GPU (or software Vulkan) and the generated map; run with --ignored"]
fn programme_order() {
    let user = std::env::temp_dir().join(format!("bf-shots-order-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&user);
    let switches = bf_sim::data::GameData::load(&root())
        .expect("game data")
        .switches
        .with(RULES)
        .expect("switches");
    // SAFETY: the client takes its switches from `BF_RULES`; one thread.
    unsafe { std::env::set_var("BF_RULES", RULES) };
    let mut s = session("USA", Date::new(1925, 1, 8), switches);
    s.submit(bf_sim::game::GameCommand::NuclearProgramme {
        tag: "USA".into(),
        share: Some(5),
    });
    while s.clock().date < Date::new(1925, 3, 2) {
        s.sim.step();
    }
    // The plants and the order the programme would reach in the 1940s, set by hand.
    {
        let data = bf_sim::data::GameData::load(&root()).expect("game data");
        let cat = &data.catalogue;
        let core_plant = cat.building("weapon_core_plant").expect("core plant") as u16;
        let assembly = cat
            .building("weapon_assembly_plant")
            .expect("assembly plant") as u16;
        let cores = cat.good("weapon_cores").expect("cores");
        let assemblies = cat.good("weapon_assemblies").expect("assemblies");
        let w = s.sim.world_mut();
        let usa = w
            .countries
            .iter()
            .position(|(_, c)| c.tag == "USA")
            .unwrap();
        let r = w
            .econ
            .regions
            .iter()
            .position(|r| usize::from(r.market) == usa && !r.colony)
            .unwrap();
        for kind in [core_plant, assembly] {
            w.econ.regions[r].plants.push(bf_sim::econ::Plant {
                kind,
                levels: 1,
                method: 0,
                staffing: Fixed::ONE,
                input_ratio: Fixed::ONE,
                utilization: Fixed::ONE,
                profit: Fixed::ZERO,
                losing_weeks: 0,
                wear: Fixed::ZERO,
            });
        }
        w.war.countries[usa]
            .queue
            .push(bf_sim::war::state::ForceOrder {
                unit: "nuclear_warhead".into(),
                generation: Some(0),
                colonial: false,
                ordered: Fixed::from_int(3),
                delivered: Fixed::ZERO,
                built: vec![(cores, Fixed::from_int(3)), (assemblies, Fixed::ONE)],
                buying: Vec::new(),
                stalled: 0,
            });
    }
    for stage in ["open", "stopped"] {
        if stage == "stopped" {
            // The programme ended; a week on, the order waits with what it holds.
            s.submit(bf_sim::game::GameCommand::NuclearProgramme {
                tag: "USA".into(),
                share: None,
            });
            let until = s.clock().date;
            while s.clock().date <= until {
                s.sim.step();
            }
        }
        for (locale, size, tag, ore) in [
            (
                "pt-BR",
                egui::vec2(1600.0, 900.0),
                "pt",
                "Encomenda de ogivas",
            ),
            ("en", egui::vec2(1280.0, 720.0), "en-1280", "Warhead order"),
        ] {
            let home = user.join(format!("{stage}-{tag}"));
            let save = write_campaign(&home, &s, "Programa", "USA");
            let mut h = harness(home, locale, size);
            open(&mut h, save);
            settle(&mut h);
            h.key_press(egui::Key::G);
            settle(&mut h);
            h.key_press(egui::Key::T);
            settle(&mut h);
            h.query_all(egui_kittest::kittest::by().label_contains(ore))
                .next()
                .expect("the order's line")
                .scroll_to_me();
            settle(&mut h);
            shot(&mut h, THEME, &format!("322-programme-order-{stage}-{tag}"));
        }
    }
    // SAFETY: as above.
    unsafe { std::env::remove_var("BF_RULES") };
    let _ = std::fs::remove_dir_all(&user);
}
