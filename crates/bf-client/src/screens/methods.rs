//! The economy window's methods tab: for each building the country has, its production
//! methods, side by side. On the left the buildings; on the right the methods of the one picked,
//! each with the technology it needs, what a level uses, makes and employs in a year, its owners'
//! profit per level at this month's prices, how many levels run it, and which one the owners
//! would pick now. Who picks: the market ("historical rule") or the government, which may fix
//! the method of the kind for the whole country (`SetMethod`, owner's decisions M1 and M2),
//! outside laissez-faire.

use bf_i18n::{Arg, I18n};
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize, Tone};
use egui::{RichText, Ui};

use crate::keys;
use crate::methods_view::{MethodRow, MethodsView};
use crate::text::money;

/// Width of the buildings' list.
const LIST: f32 = 240.0;

fn quantities(i18n: &I18n, items: &[(String, f64)], prefix: &str) -> String {
    items
        .iter()
        .map(|(k, q)| {
            i18n.tr_args(
                "methods-quantity",
                &[
                    ("n", Arg::String(i18n.compact_short(*q))),
                    ("what", Arg::String(i18n.tr(&format!("{prefix}-{k}")))),
                ],
            )
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Shows the tab; returns the command of a fixed method or of a hand-back to the market.
pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    v: &MethodsView,
    picked: &mut Option<u16>,
) -> Option<GameCommand> {
    let mut out = None;
    show_in(ui, i18n, v, picked, &mut out);
    out
}

fn show_in(
    ui: &mut Ui,
    i18n: &I18n,
    v: &MethodsView,
    picked: &mut Option<u16>,
    out: &mut Option<GameCommand>,
) {
    bf_ui_kit::small(ui, &i18n.tr("methods-basis"));
    bf_ui_kit::gap(ui, space::SM);
    if v.kinds.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("methods-none"));
        return;
    }
    if picked.is_none_or(|k| !v.kinds.iter().any(|x| x.kind == k)) {
        *picked = v.kinds.first().map(|k| k.kind);
    }
    let height = ui.available_height();
    ui.horizontal_top(|ui| {
        // The buildings.
        ui.allocate_ui_with_layout(
            egui::vec2(LIST, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("methods-kinds")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = space::XS;
                        for (i, k) in v.kinds.iter().enumerate() {
                            let on = *picked == Some(k.kind);
                            let label = i18n.tr_args(
                                "methods-kind",
                                &[
                                    (
                                        "building",
                                        Arg::String(i18n.tr(&format!("building-{}", k.key))),
                                    ),
                                    ("levels", Arg::String(k.levels.to_string())),
                                ],
                            );
                            let b = if on {
                                Button::primary(&label)
                            } else {
                                Button::ghost(&label)
                            }
                            .size(ButtonSize::Small)
                            .show(ui);
                            if b.clicked()
                                || keys::item(
                                    ui,
                                    'B',
                                    format!("methods.kind.{}", k.key),
                                    &(i + 1).to_string(),
                                    b.rect,
                                )
                            {
                                *picked = Some(k.kind);
                            }
                        }
                    });
            },
        );
        ui.add_space(space::LG);
        // Its methods.
        let Some(k) = picked.and_then(|p| v.kinds.iter().find(|k| k.kind == p)) else {
            return;
        };
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("methods-detail")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        super::scroll_keys(ui);
                        bf_ui_kit::heading(ui, &i18n.tr(&format!("building-{}", k.key)));
                        // Who picks the method: the owners by the market, or the government.
                        let can = v.rule_on && v.may_fix;
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(i18n.tr("methods-who"))
                                    .font(bf_ui_kit::theme::sans(size::SMALL))
                                    .color(color::TEXT_MUTED),
                            );
                            ui.add_enabled_ui(can || k.fixed.is_some(), |ui| {
                                let mut fixed = k.fixed.is_some();
                                let options = [
                                    (false, i18n.tr("methods-rule")),
                                    (true, i18n.tr("methods-fixed")),
                                ];
                                let tips = [
                                    keys::Tip::Item('R', "methods.who.rule".into(), "1"),
                                    keys::Tip::Item('R', "methods.who.fixed".into(), "2"),
                                ];
                                if keys::segmented(ui, &mut fixed, &options, &tips) {
                                    // Fixing starts from the method most levels run now.
                                    let method = fixed
                                        .then(|| {
                                            k.methods
                                                .iter()
                                                .filter(|m| m.available)
                                                .max_by_key(|m| {
                                                    (m.levels, std::cmp::Reverse(m.index))
                                                })
                                                .map(|m| m.index)
                                        })
                                        .flatten();
                                    if !fixed || method.is_some() {
                                        *out = Some(GameCommand::SetMethod {
                                            tag: v.tag.clone(),
                                            kind: k.kind,
                                            method,
                                        });
                                    }
                                }
                            });
                        });
                        let note = match k.fixed.and_then(|f| k.methods.get(usize::from(f))) {
                            _ if !v.rule_on => i18n.tr("methods-fixed-off"),
                            Some(m) if v.may_fix => i18n.tr_args(
                                "methods-fixed-now",
                                &[
                                    ("method", Arg::String(i18n.tr(&format!("method-{}", m.key)))),
                                    ("on", Arg::Num(k.states_on_fixed as f64)),
                                    ("states", Arg::Num(k.states as f64)),
                                ],
                            ),
                            Some(_) => i18n.tr("methods-fixed-resting"),
                            None if v.may_fix => i18n.tr("methods-fixed-can"),
                            None => i18n.tr("methods-fixed-laissez"),
                        };
                        bf_ui_kit::small(ui, &note);
                        let best = k
                            .best
                            .and_then(|b| k.methods.get(usize::from(b)))
                            .map_or_else(
                                || i18n.tr("methods-best-none"),
                                |m| {
                                    i18n.tr_args(
                                        "methods-best",
                                        &[(
                                            "method",
                                            Arg::String(i18n.tr(&format!("method-{}", m.key))),
                                        )],
                                    )
                                },
                            );
                        bf_ui_kit::small(ui, &best);
                        bf_ui_kit::gap(ui, space::SM);
                        for (i, m) in k.methods.iter().enumerate() {
                            method(
                                ui,
                                i18n,
                                m,
                                k.best == Some(m.index),
                                k.fixed == Some(m.index),
                            );
                            // With the method fixed, another known one can be fixed instead.
                            if can && k.fixed.is_some_and(|f| f != m.index) && m.available {
                                let b = Button::secondary(&i18n.tr("methods-fix-this"))
                                    .size(ButtonSize::Small)
                                    .show(ui);
                                if b.clicked()
                                    || keys::item(
                                        ui,
                                        'M',
                                        format!("methods.fix.{}", m.key),
                                        &(i + 1).to_string(),
                                        b.rect,
                                    )
                                {
                                    *out = Some(GameCommand::SetMethod {
                                        tag: v.tag.clone(),
                                        kind: k.kind,
                                        method: Some(m.index),
                                    });
                                }
                            }
                            bf_ui_kit::divider(ui);
                        }
                    });
            },
        );
    });
}

fn method(ui: &mut Ui, i18n: &I18n, m: &MethodRow, best: bool, fixed: bool) {
    ui.spacing_mut().item_spacing.y = space::XS;
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(i18n.tr(&format!("method-{}", m.key)))
                .font(bf_ui_kit::theme::sans_semibold(size::BODY))
                .color(if m.available {
                    color::TEXT
                } else {
                    color::TEXT_MUTED
                }),
        );
        if m.levels > 0 {
            bf_ui_kit::badge(
                ui,
                &i18n.tr_args(
                    "methods-in-use",
                    &[
                        ("levels", Arg::String(m.levels.to_string())),
                        ("states", Arg::Num(m.states as f64)),
                    ],
                ),
                Tone::Accent,
            );
        }
        if fixed {
            bf_ui_kit::badge(ui, &i18n.tr("methods-fixed-badge"), Tone::Accent);
        }
        if best {
            bf_ui_kit::badge(ui, &i18n.tr("methods-owners-pick"), Tone::Neutral);
        }
        if !m.available {
            let needs = m.tech.as_ref().map_or_else(
                || i18n.tr("methods-not-yet"),
                |t| {
                    i18n.tr_args(
                        "methods-needs",
                        &[("tech", Arg::String(i18n.tr(&format!("tech-{t}"))))],
                    )
                },
            );
            bf_ui_kit::badge(ui, &needs, Tone::Neutral);
        }
    });
    let line = |ui: &mut Ui, key: &str, value: String| {
        ui.label(
            RichText::new(i18n.tr_args(key, &[("v", Arg::String(value))]))
                .font(bf_ui_kit::theme::sans(size::SMALL))
                .color(color::TEXT),
        );
    };
    if !m.inputs.is_empty() {
        line(ui, "methods-inputs", quantities(i18n, &m.inputs, "good"));
    }
    line(ui, "methods-outputs", quantities(i18n, &m.outputs, "good"));
    let jobs: Vec<(String, f64)> = m.jobs.iter().map(|(p, n)| (p.clone(), *n as f64)).collect();
    line(ui, "methods-jobs", quantities(i18n, &jobs, "profession"));
    let profit = RichText::new(i18n.tr_args(
        "methods-profit",
        &[("v", Arg::String(money(i18n, m.profit)))],
    ))
    .font(bf_ui_kit::theme::sans(size::SMALL))
    .color(if m.profit < 0.0 {
        color::DANGER
    } else {
        color::TEXT_MUTED
    });
    ui.label(profit);
}
