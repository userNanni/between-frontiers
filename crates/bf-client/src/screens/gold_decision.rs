//! The Agendas' decision on the gold standard (slice 5): adhere (at the prewar parity or the
//! day's rate), leave (to float or behind exchange control), or hand the regime back to the
//! historical rule. Each choice says what it would mean this week (`gold::preview`): the
//! parity, the cover it would give against the legal cover and the floor, the points above the
//! Taylor rule that asks for, and how far the parity sits from the market. A choice the rule
//! refuses shows disabled with why. Two steps, with a restore point before.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::gold::{GoldChoice, Refusal, Regime};
use bf_ui_kit::tokens::{color, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::Ui;

use super::budget::pct;
use super::gold::{choice_key, regime_key};
use crate::gold_view::{CHOICES, GoldView, Preview};
use crate::keys;

/// The decision's state: the choice waiting for its confirmation (`None` inside: back to the
/// rule).
#[derive(Clone, Debug, Default)]
pub struct GoldUi {
    pub confirm: Option<Option<GoldChoice>>,
}

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// The label of the Agendas' button.
pub fn open_label(i18n: &I18n, g: &GoldView) -> String {
    i18n.tr(if g.regime == Regime::Gold {
        "gold-decision-open-leave"
    } else {
        "gold-decision-open-join"
    })
}

/// The choices that change something from `regime`.
fn offered(regime: Regime) -> Vec<GoldChoice> {
    CHOICES
        .iter()
        .copied()
        .filter(|c| match regime {
            Regime::Gold => matches!(c, GoldChoice::Float | GoldChoice::Control),
            Regime::Float => !matches!(c, GoldChoice::Float),
            Regime::Control => !matches!(c, GoldChoice::Control),
        })
        .collect()
}

fn preview_lines(i18n: &I18n, c: GoldChoice, p: &Preview) -> Vec<(String, egui::Color32)> {
    let pc = |x: f64| pct(i18n, x * 100.0, 0);
    let mut out = vec![(
        i18n.tr_args(
            "gold-decision-parity",
            &[
                ("parity", arg(i18n.decimal(p.parity, 1))),
                ("gap", arg(pct(i18n, p.gap * 100.0, 1))),
            ],
        ),
        color::TEXT_MUTED,
    )];
    if matches!(c, GoldChoice::Prewar | GoldChoice::Market) {
        let slack = (p.cover - p.legal) * 100.0;
        let tone = if p.cover >= p.legal {
            color::POSITIVE
        } else if p.cover >= p.floor {
            color::ACCENT
        } else {
            color::DANGER
        };
        out.push((
            i18n.tr_args(
                "gold-decision-cover",
                &[
                    ("cover", arg(pc(p.cover))),
                    ("legal", arg(pc(p.legal))),
                    ("floor", arg(pc(p.floor))),
                    (
                        "margin",
                        arg(i18n.tr_args(
                            if slack >= 0.0 {
                                "gold-margin-over"
                            } else {
                                "gold-margin-short"
                            },
                            &[("value", arg(super::gold::points_text(i18n, slack.abs())))],
                        )),
                    ),
                ],
            ),
            tone,
        ));
        if p.defense > 0.0 {
            out.push((
                i18n.tr_args(
                    "gold-decision-defense",
                    &[("points", arg(super::gold::points_text(i18n, p.defense)))],
                ),
                color::DANGER,
            ));
        }
    }
    out.push((
        i18n.tr(&format!("{}-means", choice_key(c))),
        color::TEXT_MUTED,
    ));
    out
}

/// Draws the decision; `Some(choice)` once confirmed (`Some(None)`: back to the rule).
pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    g: &GoldView,
    state: &mut GoldUi,
) -> Option<Option<GoldChoice>> {
    let mut out = None;
    bf_ui_kit::heading(ui, &i18n.tr("gold-decision-title"));
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "gold-decision-now",
            &[
                ("regime", arg(i18n.tr(regime_key(g.regime)))),
                (
                    "by",
                    arg(i18n.tr(if g.chosen {
                        "gold-by-government"
                    } else {
                        "gold-by-rule"
                    })),
                ),
            ],
        ),
    );
    if let Some(c) = g.pending {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args("gold-pending", &[("choice", arg(i18n.tr(choice_key(c))))]),
        );
    }
    bf_ui_kit::gap(ui, space::SM);
    let width = ui.available_width();
    for (k, c) in offered(g.regime).into_iter().enumerate() {
        let ix = CHOICES.iter().position(|x| *x == c).unwrap_or(0);
        let preview = g.choices.get(ix).copied();
        ui.group(|ui| {
            ui.set_width(width - space::LG);
            let title = i18n.tr(&format!("{}-title", choice_key(c)));
            match preview {
                Some(Ok(p)) => {
                    let pick = Button::secondary(&title).size(ButtonSize::Small).show(ui);
                    if pick.clicked()
                        || keys::item(
                            ui,
                            'U',
                            format!("journal.gold.{k}"),
                            &(k + 1).to_string(),
                            pick.rect,
                        )
                    {
                        state.confirm = Some(Some(c));
                    }
                    for (line, tone) in preview_lines(i18n, c, &p) {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(line)
                                    .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                                    .color(tone),
                            )
                            .wrap(),
                        );
                    }
                }
                Some(Err(Refusal::NoPrewarParity)) | None => {
                    Button::secondary(&title)
                        .size(ButtonSize::Small)
                        .enabled(false)
                        .show(ui);
                    bf_ui_kit::small(ui, &i18n.tr("gold-decision-no-prewar"));
                }
            }
        });
    }
    if g.chosen {
        let back = Button::ghost(&i18n.tr("gold-choice-rule-title"))
            .size(ButtonSize::Small)
            .show(ui);
        if back.clicked() || keys::single(ui, "journal.gold.rule", 'R', back.rect) {
            state.confirm = Some(None);
        }
        bf_ui_kit::small(ui, &i18n.tr("gold-choice-rule-means"));
    }
    // The second step: what changes, then confirm or keep.
    if let Some(choice) = state.confirm {
        bf_ui_kit::gap(ui, space::SM);
        let key = match choice {
            Some(c) => format!("{}-confirm", choice_key(c)),
            None => "gold-choice-rule-confirm".to_owned(),
        };
        ui.add(
            egui::Label::new(
                egui::RichText::new(i18n.tr(&key))
                    .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                    .color(color::DANGER),
            )
            .wrap(),
        );
        ui.horizontal(|ui| {
            let yes = Button::danger(&i18n.tr("gold-decision-confirm"))
                .size(ButtonSize::Small)
                .show(ui);
            if yes.clicked() || keys::single(ui, "journal.gold.yes", 'Y', yes.rect) {
                out = Some(choice);
                state.confirm = None;
            }
            let no = Button::ghost(&i18n.tr("forces-keep"))
                .size(ButtonSize::Small)
                .show(ui);
            if no.clicked() || keys::single(ui, "journal.gold.no", 'N', no.rect) {
                state.confirm = None;
            }
        });
    }
    out
}
