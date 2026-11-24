//! Scorched earth against an enemy (switch `scorched_earth`): the Agendas' decision, a rare
//! order in two steps with a restore point first. It lists the player's regions fought over
//! against the enemy, with their plant levels, their share of the country's output, the weeks
//! of fighting and the share that would burn if each fell this week; then gives or withdraws
//! the order (`GameCommand::SetScorchedEarth`, sent in `screens/game.rs`).

use bf_i18n::{Arg, I18n};
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Ui};

use super::journal::Response;
use crate::keys;
use crate::text::{country_article, country_name};
use crate::war_view::{Scorch, WarView};
use crate::world::World;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// The confirmation waiting, if any.
#[derive(Clone, Debug, Default)]
pub struct ScorchUi {
    pub confirming: bool,
}

/// The enemies the player fights with the rule on, and their scorched earth.
pub fn fronts(war: &WarView) -> Vec<(&str, &Scorch)> {
    let mut out: Vec<(&str, &Scorch)> = Vec::new();
    for w in &war.wars {
        for f in &w.fronts {
            if let Some(s) = &f.scorch
                && !out.iter().any(|(e, _)| *e == f.enemy)
            {
                out.push((f.enemy.as_str(), s));
            }
        }
    }
    out
}

/// The button's label in the Agendas.
pub fn open_label(i18n: &I18n, enemy: &str, s: &Scorch) -> String {
    i18n.tr_args(
        if s.ordered {
            "scorch-open-ordered"
        } else {
            "scorch-open"
        },
        &[
            ("country", arg(country_name(i18n, enemy))),
            ("article", arg(country_article(i18n, enemy))),
        ],
    )
}

pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    enemy: &str,
    s: &Scorch,
    st: &mut ScorchUi,
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::SM;
    let name = |k: &str| {
        i18n.tr_args(
            k,
            &[
                ("country", arg(country_name(i18n, enemy))),
                ("article", arg(country_article(i18n, enemy))),
            ],
        )
    };
    bf_ui_kit::title(ui, &name("scorch-title"));
    bf_ui_kit::small(ui, &i18n.tr("scorch-basis"));
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("scorch-front"),
        &i18n.tr(if s.ordered {
            "scorch-front-ordered"
        } else {
            "scorch-front-none"
        }),
    );
    bf_ui_kit::heading(ui, &i18n.tr("scorch-regions"));
    if s.regions.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("scorch-regions-none"));
    } else {
        egui::Grid::new("scorch-regions")
            .num_columns(5)
            .striped(true)
            .show(ui, |ui| {
                for key in [
                    "scorch-col-state",
                    "scorch-col-levels",
                    "scorch-col-output",
                    "scorch-col-weeks",
                    "scorch-col-burn",
                ] {
                    ui.label(
                        RichText::new(i18n.tr(key))
                            .font(sans(size::SMALL))
                            .color(color::TEXT_MUTED),
                    );
                }
                ui.end_row();
                for r in &s.regions {
                    ui.label(super::war::state_name(i18n, world, r.state));
                    ui.label(i18n.int(r.levels));
                    ui.label(super::budget::pct(i18n, r.output * 100.0, 1));
                    ui.label(i18n.int(i64::from(r.weeks)));
                    ui.label(super::budget::pct(i18n, f64::from(r.would_burn) / 10.0, 0));
                    ui.end_row();
                }
            });
    }
    bf_ui_kit::small(ui, &i18n.tr("scorch-burn-note"));
    if !s.burned.is_empty() {
        bf_ui_kit::heading(ui, &i18n.tr("scorch-burned"));
        for b in &s.burned {
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "scorch-burned-line",
                    &[
                        ("state", arg(super::war::state_name(i18n, world, b.state))),
                        ("to", arg(country_name(i18n, &b.to))),
                        ("levels", arg(i18n.int(b.levels))),
                        (
                            "share",
                            arg(super::budget::pct(i18n, f64::from(b.share) / 10.0, 0)),
                        ),
                        (
                            "date",
                            arg(crate::text::date_long(
                                i18n,
                                laplace_schelling::calendar::Date::from_days(b.day),
                            )),
                        ),
                    ],
                ),
            );
        }
    }
    bf_ui_kit::gap(ui, space::SM);
    let (ask, sure, explain) = if s.ordered {
        (
            "scorch-withdraw",
            "scorch-withdraw-confirm",
            "scorch-withdraw-explain",
        )
    } else {
        ("scorch-give", "scorch-give-confirm", "scorch-give-explain")
    };
    bf_ui_kit::small(ui, &i18n.tr(explain));
    if st.confirming {
        ui.horizontal(|ui| {
            let yes = Button::primary(&i18n.tr(sure))
                .size(ButtonSize::Small)
                .show(ui);
            if yes.clicked() || keys::single(ui, "journal.scorch.yes", 'Y', yes.rect) {
                out.scorch = Some((enemy.to_owned(), !s.ordered));
                st.confirming = false;
                return;
            }
            let no = Button::ghost(&i18n.tr("journal-decision-cancel"))
                .size(ButtonSize::Small)
                .show(ui);
            if no.clicked() || keys::single(ui, "journal.scorch.no", 'N', no.rect) {
                st.confirming = false;
            }
        });
    } else {
        let first = Button::secondary(&i18n.tr(ask))
            .size(ButtonSize::Small)
            .show(ui);
        if first.clicked() || keys::single(ui, "journal.scorch", 'D', first.rect) {
            st.confirming = true;
        }
    }
}
