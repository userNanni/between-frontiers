//! The "Balance of deterrence" panel (`nuclear_view`), on the crisis and the war screens: each
//! side's second strike against what the other holds as unacceptable, who is secure, how far each
//! can climb, and the risk per week of a direct war. The player's side is framed, the other
//! beside it. Read only.

use bf_i18n::{Arg, I18n};
use egui::{RichText, Ui};

use crate::nuclear_view::{Panel, PanelSide, rung_key};
use crate::text::country_name;
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn side(ui: &mut Ui, i18n: &I18n, s: &PanelSide, mine: bool) {
    let name = country_name(i18n, &s.tag);
    ui.strong(if mine {
        i18n.tr_args("war-support-you", &[("country", arg(name))])
    } else {
        name
    });
    let warheads = if s.opaque {
        i18n.tr_args(
            "nuclear-warheads-opaque",
            &[("n", arg(i18n.int(s.warheads as i64)))],
        )
    } else {
        i18n.tr_args(
            "nuclear-warheads",
            &[("n", arg(i18n.int(s.warheads as i64)))],
        )
    };
    bf_ui_kit::small(ui, &warheads);
    if !s.protectors.is_empty() {
        let who: Vec<String> = s.protectors.iter().map(|t| country_name(i18n, t)).collect();
        bf_ui_kit::small(
            ui,
            &i18n.tr_args("nuclear-umbrella", &[("who", arg(who.join(", ")))]),
        );
    }
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "nuclear-second-strike",
            &[
                ("delivered", arg(i18n.int(s.delivered.round() as i64))),
                (
                    "needed",
                    arg(i18n.int(s.unacceptable_to_other.round() as i64)),
                ),
            ],
        ),
    );
    let (key, c) = if s.secure {
        ("nuclear-secure", color::TEXT)
    } else {
        ("nuclear-not-secure", color::TEXT_MUTED)
    };
    ui.label(RichText::new(i18n.tr(key)).font(sans(size::SMALL)).color(c));
    if s.first_strike_incentive {
        ui.label(
            RichText::new(i18n.tr("nuclear-first-strike"))
                .font(sans(size::SMALL))
                .color(color::DANGER),
        );
    }
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "nuclear-ceiling",
            &[("rung", arg(i18n.tr(rung_key(s.ceiling))))],
        ),
    );
}

/// The panel, under a heading.
pub fn show(ui: &mut Ui, i18n: &I18n, p: &Panel) {
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::heading(ui, &i18n.tr("nuclear-balance"));
    // The player's side first, framed as in the war support; the other beside it when there
    // is room, under it when not.
    let mine = |ui: &mut Ui| {
        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, color::ACCENT))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| side(ui, i18n, &p.sides[0], true));
    };
    if ui.available_width() >= 680.0 {
        bf_ui_kit::columns(ui, 2, |cols| {
            mine(&mut cols[0]);
            side(&mut cols[1], i18n, &p.sides[1], false);
        });
    } else {
        mine(ui);
        side(ui, i18n, &p.sides[1], false);
    }
    let summary = if p.mutual {
        i18n.tr_args(
            "nuclear-mutual",
            &[(
                "risk",
                arg(i18n.decimal(f64::from(p.risk_per_week) / 10.0, 1)),
            )],
        )
    } else {
        i18n.tr("nuclear-not-mutual")
    };
    bf_ui_kit::small(ui, &summary);
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "nuclear-opacity-note",
            &[(
                "share",
                arg(i18n.decimal(f64::from(p.opaque_weight) / 10.0, 0)),
            )],
        ),
    );
}
