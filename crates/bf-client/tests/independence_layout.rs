//! The answer to a demand of independence fits every logical screen, in both languages. Its
//! own binary: the independence rule switches on through `BF_RULES`, which only a process alone
//! may write.

use bf_client::app::Transition;
use bf_client::saves;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

mod common;
use common::layout::*;

fn steps(h: &mut Harness<'_, bf_client::App>) {
    h.run_steps(4);
}

fn click(h: &mut Harness<'_, bf_client::App>, label: &str) {
    h.get_all(egui_kittest::kittest::by().label(label))
        .next()
        .unwrap_or_else(|| panic!("no {label}"))
        .click();
    steps(h);
}

/// A demand of independence on the player: Nigeria's, pending, on Britain in 1925, with an
/// account of every term and five payers. The answer screen fits every size, both languages, at
/// each answer and at the confirmation.
#[test]
fn the_demand_screen_fits() {
    // SAFETY: the only test of this binary; nothing else reads the environment meanwhile.
    unsafe { std::env::set_var("BF_RULES", "independence") };
    use bf_sim::borders::independence::{Account, Answer, Demand, units_of};
    use laplace_schelling::Fixed;
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let rules = data.rules();
    let setup = rules.independence.clone().expect("independence rules");
    let mut world = data.new_world();
    let ix = |w: &bf_sim::game::GameWorld, tag: &str| {
        w.countries.iter().position(|(_, c)| c.tag == tag).unwrap() as u16
    };
    let mut units = units_of(&setup, &world, &[]);
    let top: Vec<(u16, Fixed)> = ["USA", "SOV", "YUG", "TUR", "ITA"]
        .iter()
        .map(|t| (ix(&world, t), Fixed::from_ratio(3, 1000)))
        .collect();
    let u = units.iter_mut().find(|u| u.key == "NGA").expect("NGA");
    u.movement.support = 1000;
    u.mobilized = 640;
    u.demand = Some(Demand {
        since: laplace_schelling::calendar::Date::new(1925, 1, 2),
        answer: Answer::Pending,
        account: Account {
            value: Fixed::from_ratio(12, 1000),
            legitimacy: Fixed::from_ratio(4, 1000),
            international: Fixed::from_ratio(9, 1000),
            material: Fixed::from_ratio(2, 1000),
            top,
        },
    });
    world.independence.units = units;
    let mut session = bf_sim::game::Session::new(
        world,
        7,
        laplace_schelling::calendar::Date::new(1925, 1, 2),
        1,
        Some(rules),
    );
    session.submit(bf_sim::game::GameCommand::ChoosePlayer { tag: "GBR".into() });
    let mut failures = Vec::new();
    for locale in ["pt-BR", "en"] {
        let user =
            std::env::temp_dir().join(format!("bf-matrix-demand-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save = saves::create(
            &user.join("saves"),
            "Exigência",
            7,
            laplace_schelling::calendar::Date::new(1925, 1, 1),
        )
        .unwrap();
        save.meta.player = Some("GBR".into());
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "demand").unwrap(),
        )
        .unwrap();
        saves::write(&save).unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        steps(&mut h);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        steps(&mut h);
        h.key_press(egui::Key::J);
        steps(&mut h);
        let title = if locale == "en" {
            "Nigeria"
        } else {
            "Nigéria"
        };
        match h
            .query_all(egui_kittest::kittest::by().label_contains(title))
            .find(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        {
            Some(b) => b.click(),
            None => {
                failures.push(format!("{locale}: no demand in the Agendas"));
                continue;
            }
        }
        steps(&mut h);
        click(&mut h, if locale == "en" { "Refuse" } else { "Recusar" });
        for (w, hh) in [
            (960.0, 540.0),
            (1280.0, 720.0),
            (1600.0, 900.0),
            (3840.0, 2160.0),
        ] {
            h.set_size(egui::vec2(w, hh));
            steps(&mut h);
            let case = format!("{locale} {w}×{hh} demand");
            failures.extend(check(&h, &case));
            failures.extend(no_overlapping_text(&h, &case));
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
