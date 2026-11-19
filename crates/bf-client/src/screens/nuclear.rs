//! The "Balance of deterrence" panel (`nuclear_view`), on the crisis and the war screens: each
//! side's second strike against what the other holds as unacceptable, who is secure, how far each
//! can climb, and the risk per week of a direct war. Read only.

use bf_i18n::{Arg, I18n};
use egui::{RichText, Ui};

use crate::nuclear_view::{Panel, PanelSide, rung_key};
use crate::text::country_name;
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn side(ui: &mut Ui, i18n: &I18n, s: &PanelSide) {
    let name = country_name(i18n, &s.tag);
    bf_ui_kit::caption(ui, &name);
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
    for s in &p.sides {
        side(ui, i18n, s);
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
