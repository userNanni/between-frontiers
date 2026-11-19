//! The trade of the country picked on the map, below its summary: what it sold and bought in the
//! year, its own tariff, and its largest partners, each with what goes each way and what the
//! agreements in force do to the tariff between them (customs union, preference, cap). Reading
//! only; trade by pair must be on.

use bf_i18n::{Arg, I18n};
use bf_sim::treaty::TariffTerm;
use bf_ui_kit::tokens::space;
use bf_ui_kit::{Tone, badge};
use egui::Ui;

use super::budget::pct;
use crate::text::{country_name, money};
use crate::trade_view::{Partner, TradeView};
use crate::world::World;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// What one tariff term says, short.
pub fn term(i18n: &I18n, t: TariffTerm) -> String {
    match t {
        TariffTerm::Union => i18n.tr("trade-union"),
        TariffTerm::Margin(m) => i18n.tr_args(
            "trade-preference",
            &[("points", arg(i18n.decimal(m.to_f64_lossy() * 100.0, 1)))],
        ),
        TariffTerm::Cap(c) => i18n.tr_args(
            "trade-cap",
            &[("rate", arg(pct(i18n, c.to_f64_lossy() * 100.0, 1)))],
        ),
    }
}

pub fn show(ui: &mut Ui, i18n: &I18n, world: &World, view: &TradeView, country: usize) {
    let Some(c) = view.countries.get(country) else {
        return;
    };
    bf_ui_kit::divider(ui);
    bf_ui_kit::heading(ui, &i18n.tr("trade-title"));
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            if view.so_far {
                "trade-year-so-far"
            } else {
                "trade-year"
            },
            &[("year", arg(view.year.to_string()))],
        ),
    );
    bf_ui_kit::stat_row(ui, &i18n.tr("trade-exports"), &money(i18n, c.exports));
    bf_ui_kit::stat_row(ui, &i18n.tr("trade-imports"), &money(i18n, c.imports));
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("trade-tariff"),
        &pct(i18n, c.tariff * 100.0, 1),
    );
    if c.partners.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("trade-none"));
        return;
    }
    bf_ui_kit::caption(ui, &i18n.tr("trade-partners"));
    let total = (c.exports + c.imports).max(f64::MIN_POSITIVE);
    for p in &c.partners {
        partner(ui, i18n, world, p, total);
    }
    bf_ui_kit::small(ui, &i18n.tr("trade-basis"));
}

fn partner(ui: &mut Ui, i18n: &I18n, world: &World, p: &Partner, total: f64) {
    let Some(other) = world.countries.get(p.country) else {
        return;
    };
    ui.spacing_mut().item_spacing.y = space::XS;
    ui.horizontal_wrapped(|ui| {
        ui.strong(country_name(i18n, other.tag()));
        ui.label(pct(i18n, (p.exports + p.imports) / total * 100.0, 0));
        if p.union() {
            badge(ui, &i18n.tr("trade-union"), Tone::Accent);
        }
    });
    ui.add(
        egui::Label::new(
            egui::RichText::new(i18n.tr_args(
                "trade-flows",
                &[
                    ("sold", arg(money(i18n, p.exports))),
                    ("bought", arg(money(i18n, p.imports))),
                ],
            ))
            .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL)),
        )
        .wrap(),
    );
    // The agreements' terms each way, but a customs union is said once.
    if !p.union() {
        for (terms, key) in [
            (&p.charges, "trade-we-charge"),
            (&p.charged, "trade-they-charge"),
        ] {
            if terms.is_empty() {
                continue;
            }
            let said: Vec<String> = terms.iter().map(|t| term(i18n, *t)).collect();
            bf_ui_kit::small(ui, &i18n.tr_args(key, &[("terms", arg(said.join(" · ")))]));
        }
    }
    bf_ui_kit::gap(ui, space::XS);
}
