//! The forces tab with orders fits every logical screen, in both languages: Britain in 1925 with
//! a battleship, destroyers and infantry on order, the queue, a large order waiting for its second
//! click and a cancellation asked. Its own binary: the forces ship off and switch on through
//! `BF_RULES=forces`, which only a process alone may write.

use bf_client::app::Transition;
use bf_client::saves;
use bf_sim::game::GameCommand;
use egui_kittest::kittest::{NodeT, Queryable};
use laplace_schelling::calendar::Date;

mod common;
use common::layout::*;

fn session() -> bf_sim::game::Session {
    let data = bf_sim::data::GameData::load(&root()).expect("game data");
    let mut s = bf_sim::game::Session::new(
        data.new_world(),
        7,
        Date::new(1925, 1, 1),
        1,
        Some(data.rules()),
    );
    s.submit(GameCommand::ChoosePlayer { tag: "GBR".into() });
    for (unit, elements) in [("battleship", 1), ("destroyer", 4), ("infantry", 6)] {
        s.submit(GameCommand::OrderForces {
            tag: "GBR".into(),
            unit: unit.into(),
            elements,
            colonial: false,
        });
    }
    while s.clock().date < Date::new(1925, 3, 2) {
        s.sim.step();
    }
    s
}

#[test]
fn the_forces_tab_holds_the_orders() {
    // SAFETY: the only test of this binary; nothing else reads the environment meanwhile.
    unsafe { std::env::set_var("BF_RULES", "forces") };
    let session = session();
    let mut failures = Vec::new();
    for locale in ["pt-BR", "en"] {
        let user = std::env::temp_dir().join(format!("bf-forces-{locale}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&user);
        let mut save =
            saves::create(&user.join("saves"), "Encomendas", 7, Date::new(1925, 1, 1)).unwrap();
        save.meta.player = Some("GBR".into());
        save.meta.date = session.clock().date;
        std::fs::write(
            save.dir.join(saves::WORLD_FILE),
            session.save("test", "orders").unwrap(),
        )
        .unwrap();
        saves::write(&save).unwrap();
        let mut h = app(user.clone(), locale, egui::vec2(1600.0, 900.0));
        h.run_steps(4);
        let ctx = h.ctx.clone();
        h.state_mut().apply(Transition::Game(save), &ctx);
        h.run_steps(4);
        h.key_press(egui::Key::G);
        h.run_steps(4);
        let en = locale == "en";
        // The button of that label, brought into view first (a unit's name is also a line of
        // the forces' table; the tabs are not buttons).
        let click = |h: &mut egui_kittest::Harness<'_, bf_client::App>, label: &str| {
            let pick = |n: &egui_kittest::Node<'_>| {
                !matches!(
                    n.accesskit_node().role(),
                    egui::accesskit::Role::Label | egui::accesskit::Role::Cell
                )
            };
            let Some(n) = h
                .query_all(egui_kittest::kittest::by().label(label))
                .find(|n| pick(n))
            else {
                return false;
            };
            n.scroll_to_me();
            h.run_steps(4);
            if let Some(n) = h
                .query_all(egui_kittest::kittest::by().label(label))
                .find(|n| pick(n))
            {
                n.click();
            }
            h.run_steps(4);
            true
        };
        if !click(&mut h, if en { "Forces" } else { "Forças" }) {
            failures.push(format!("{locale}: no forces tab"));
            continue;
        }
        // The queue holds the three orders.
        let queue = h
            .query_all(egui_kittest::kittest::by().label(if en { "Cancel" } else { "Cancelar" }))
            .count();
        if queue != 3 {
            failures.push(format!("{locale}: {queue} orders in the queue, not 3"));
        }
        // A battleship asks twice; a cancellation asks twice.
        click(&mut h, if en { "Battleships" } else { "Encouraçados" });
        click(&mut h, if en { "Order" } else { "Encomendar" });
        if !click(&mut h, if en { "Cancel" } else { "Cancelar" }) {
            failures.push(format!("{locale}: no cancel"));
        }
        let asks = [
            if en {
                "Place the order"
            } else {
                "Fazer a encomenda"
            },
            if en {
                "Cancel the order"
            } else {
                "Cancelar a encomenda"
            },
        ];
        for a in asks {
            if h.query_all(egui_kittest::kittest::by().label(a))
                .next()
                .is_none()
            {
                failures.push(format!("{locale}: no {a:?}"));
            }
        }
        // Back to the top of the tab: rows scrolled out of view still report where they were.
        h.get_by_label(if en { "Summary" } else { "Resumo" })
            .scroll_to_me();
        h.run_steps(4);
        for (w, hh) in SIZES {
            h.set_size(egui::vec2(w, hh));
            h.run_steps(4);
            let case = format!("{locale} {w}×{hh} forces orders");
            failures.extend(check(&h, &case));
            failures.extend(no_overlapping_text(&h, &case));
        }
        let _ = std::fs::remove_dir_all(&user);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
