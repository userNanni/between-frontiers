//! Government tab: the armed forces. What is under arms by unit type, with its equipment and
//! training, the reserve and what it costs each week. Reading only: mobilizing is a decision of
//! the journal's agendas, with a confirmation step.

use bf_i18n::{Arg, I18n};
use bf_sim::war::Domain;
use bf_ui_kit::tokens::space;
use egui::Ui;

use super::budget::{WEEKS, of_gdp, pct};
use super::government::TabContext;
use crate::forces_view::ForcesView;
use crate::text::money;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn domain_key(d: Domain) -> &'static str {
    match d {
        Domain::Land => "forces-domain-land",
        Domain::Sea => "forces-domain-sea",
        Domain::Air => "forces-domain-air",
        Domain::Strategic => "forces-domain-strategic",
    }
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(f) = cx.snap.forces.as_deref() else {
        bf_ui_kit::muted(ui, &i18n.tr("forces-none"));
        return;
    };
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &i18n.tr("forces-basis"));
    bf_ui_kit::gap(ui, space::SM);
    summary(ui, i18n, f, cx.view.map_or(0.0, |v| v.gdp_last_year));
    bf_ui_kit::gap(ui, space::MD);
    table(ui, i18n, f);
    bf_ui_kit::gap(ui, space::MD);
    bf_ui_kit::small(ui, &i18n.tr("forces-decision-hint"));
}

fn summary(ui: &mut Ui, i18n: &I18n, f: &ForcesView, gdp: f64) {
    bf_ui_kit::heading(ui, &i18n.tr("forces-summary"));
    let n = |x: i64| i18n.int(x);
    let lines = [
        ("forces-men", n(f.men)),
        ("forces-reserve", n(f.reserve)),
        (
            "forces-share",
            if f.peace_share > 0.0 {
                pct(i18n, f.peace_share * 100.0, 1)
            } else {
                "—".into()
            },
        ),
        (
            "forces-goods",
            i18n.tr_args(
                "forces-goods-value",
                &[
                    ("week", arg(money(i18n, f.weekly_goods))),
                    ("gdp", arg(of_gdp(i18n, f.weekly_goods * WEEKS, gdp))),
                ],
            ),
        ),
        ("forces-depot", money(i18n, f.depot)),
    ];
    egui::Grid::new("forces-summary")
        .num_columns(2)
        .show(ui, |ui| {
            for (key, value) in lines {
                ui.label(i18n.tr(key));
                ui.label(value);
                ui.end_row();
            }
        });
    let state = if f.mobilized {
        i18n.tr_args(
            "forces-mobilizing",
            &[
                ("reported", arg(n(f.called_up))),
                ("total", arg(n(f.called_up + f.reserve))),
            ],
        )
    } else {
        i18n.tr("forces-peace")
    };
    bf_ui_kit::small(ui, &state);
}

fn table(ui: &mut Ui, i18n: &I18n, f: &ForcesView) {
    bf_ui_kit::heading(ui, &i18n.tr("forces-units"));
    egui::Grid::new("forces-units")
        .num_columns(7)
        .striped(true)
        .show(ui, |ui| {
            for key in [
                "forces-col-unit",
                "forces-col-domain",
                "forces-col-generation",
                "forces-col-size",
                "forces-col-men",
                "forces-col-equipment",
                "forces-col-training",
            ] {
                ui.strong(i18n.tr(key));
            }
            ui.end_row();
            for r in &f.rows {
                let mut name = i18n.tr(&format!("unit-{}", r.unit));
                if r.called_up {
                    name = i18n.tr_args("forces-called-up", &[("unit", arg(name))]);
                }
                ui.label(name);
                ui.label(i18n.tr(domain_key(r.domain)));
                ui.label(i18n.int(i64::from(r.generation)));
                ui.label(i18n.decimal(r.elements, 1));
                ui.label(i18n.int(r.men));
                ui.label(pct(i18n, r.equipment * 100.0, 0));
                ui.label(pct(i18n, r.training * 100.0, 0));
                ui.end_row();
            }
        });
}
