//! The money tab's "Exchange and reserves" (gold standard slice 5): the regime and its parity,
//! the cover against the legal cover and the floor, what the rule does below them, the posture
//! on gold (the historical rule or one the government fixes), the reserves by currency, and
//! capital controls in two steps. Shown only with the gold standard on; changing the regime is
//! a decision of the Agendas.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::gold::{GoldChoice, Posture, Regime};
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::{color, radius, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Rect, Stroke, Ui, Vec2};

use super::budget::pct;
use super::gov_ui::Account;
use super::government::TabContext;
use crate::gold_view::GoldView;
use crate::keys::{self, Tip};
use crate::text::{country_name, money};

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// Points, written out: "1,5 pontos percentuais".
fn points(i18n: &I18n, x: f64) -> String {
    let n = (x * 10.0).round() / 10.0;
    i18n.tr_args(
        "gold-points",
        &[("n", Arg::Num(n)), ("value", arg(i18n.decimal(n, 1)))],
    )
}

pub fn regime_key(r: Regime) -> &'static str {
    match r {
        Regime::Float => "gold-regime-float",
        Regime::Gold => "gold-regime-gold",
        Regime::Control => "gold-regime-control",
    }
}

pub fn choice_key(c: GoldChoice) -> &'static str {
    match c {
        GoldChoice::Prewar => "gold-choice-prewar",
        GoldChoice::Market => "gold-choice-market",
        GoldChoice::Float => "gold-choice-float",
        GoldChoice::Control => "gold-choice-control",
    }
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>, g: &GoldView) {
    let i18n = cx.i18n;
    let p = |x: f64| pct(i18n, x * 100.0, 0);
    bf_ui_kit::heading(ui, &i18n.tr("gold-title"));
    bf_ui_kit::small(ui, &i18n.tr("gold-basis"));

    // The regime, and who chose it.
    let by = i18n.tr(if g.chosen {
        "gold-by-government"
    } else {
        "gold-by-rule"
    });
    let tip = i18n.tr(&format!("{}-tip", regime_key(g.regime)));
    Account::new(
        &i18n.tr("gold-regime"),
        &format!("{} · {by}", i18n.tr(regime_key(g.regime))),
        &tip,
    )
    .show(ui);
    if let Some(c) = g.pending {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args("gold-pending", &[("choice", arg(i18n.tr(choice_key(c))))]),
        );
    }
    if g.regime != Regime::Float {
        let value = i18n.tr_args(
            "gold-parity-value",
            &[
                ("parity", arg(i18n.decimal(g.parity, 1))),
                ("today", arg(i18n.decimal(g.today, 1))),
            ],
        );
        Account::new(&i18n.tr("gold-parity"), &value, &i18n.tr("gold-parity-tip")).show(ui);
    }

    // The cover, against the legal cover and the floor.
    cover_bar(ui, i18n, g);
    if g.regime == Regime::Gold {
        if g.weeks_floor > 0 {
            let line = match g.posture {
                Posture::Rule => i18n.tr_args(
                    "gold-below-floor-rule",
                    &[
                        ("weeks", Arg::Num(f64::from(g.weeks_floor))),
                        (
                            "left",
                            Arg::Num(f64::from(g.exit_weeks.saturating_sub(g.weeks_floor))),
                        ),
                    ],
                ),
                Posture::Defend => i18n.tr("gold-below-floor-defend"),
                Posture::Devalue => i18n.tr_args(
                    "gold-below-floor-devalue",
                    &[("share", arg(p(g.devaluation)))],
                ),
            };
            warn(ui, &line);
        } else if g.weeks_short > 0 {
            warn(
                ui,
                &i18n.tr_args(
                    "gold-below-legal",
                    &[
                        ("weeks", Arg::Num(f64::from(g.weeks_short))),
                        ("points", arg(points(i18n, g.defense))),
                        (
                            "left",
                            Arg::Num(f64::from(g.withdraw_weeks.saturating_sub(g.weeks_short))),
                        ),
                    ],
                ),
            );
        }
        posture(ui, cx, g);
    }
    if g.premium > 0.0 {
        Account::new(
            &i18n.tr("gold-premium"),
            &points(i18n, g.premium),
            &i18n.tr("gold-premium-tip"),
        )
        .show(ui);
    }

    // The reserves, by currency, and our currency in the others' reserves.
    bf_ui_kit::gap(ui, space::SM);
    let total = g.gold + g.fx.iter().map(|x| x.1).sum::<f64>();
    Account::new(
        &i18n.tr("gold-reserves"),
        &money(i18n, total),
        &i18n.tr("gold-reserves-tip"),
    )
    .show(ui);
    let share = |x: f64| (total > 0.0).then(|| (x / total) as f32);
    let gold_label = i18n.tr("gold-reserves-gold");
    let gold_value = money(i18n, g.gold);
    let mut a = Account::new(&gold_label, &gold_value, &gold_label);
    a.share = share(g.gold);
    a.show(ui);
    // The largest currencies one by one; the rest in one line.
    const SHOWN: usize = 5;
    for (tag, x) in g.fx.iter().take(SHOWN) {
        let label = i18n.tr_args(
            "gold-reserves-fx",
            &[("country", arg(country_name(i18n, tag)))],
        );
        let tip = i18n.tr_args(
            "gold-reserves-fx-tip",
            &[
                ("country", arg(country_name(i18n, tag))),
                ("share", arg(p(g.devaluation))),
                ("loss", arg(money(i18n, x * g.devaluation))),
            ],
        );
        let value = money(i18n, *x);
        let mut a = Account::new(&label, &value, &tip);
        a.share = share(*x);
        a.show(ui);
    }
    if g.fx.len() > SHOWN {
        let rest: f64 = g.fx[SHOWN..].iter().map(|x| x.1).sum();
        let label = i18n.tr_args(
            "gold-reserves-rest",
            &[("n", Arg::Num((g.fx.len() - SHOWN) as f64))],
        );
        let value = money(i18n, rest);
        let mut a = Account::new(&label, &value, &label);
        a.share = share(rest);
        a.show(ui);
    }
    Account::new(
        &i18n.tr("gold-owed"),
        &money(i18n, g.owed),
        &i18n.tr("gold-owed-tip"),
    )
    .show(ui);

    if let Some(c) = &g.capital {
        capital(ui, cx, c);
    }
}

fn warn(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                .color(color::DANGER),
        )
        .wrap(),
    );
}

/// The cover as a bar, with marks at the floor and the legal cover; the numbers below.
fn cover_bar(ui: &mut Ui, i18n: &I18n, g: &GoldView) {
    let p = |x: f64| pct(i18n, x * 100.0, 0);
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 14.0), egui::Sense::hover());
    let top = (g.legal * 1.6).max(g.cover * 1.1).max(0.01);
    let x = |v: f64| rect.left() + (v / top).clamp(0.0, 1.0) as f32 * rect.width();
    let track = Rect::from_min_max(
        egui::pos2(rect.left(), rect.center().y - 4.0),
        egui::pos2(rect.right(), rect.center().y + 4.0),
    );
    let painter = ui.painter();
    painter.rect_filled(track, radius::SM, color::SURFACE_SUNKEN);
    let tone = if g.cover >= g.legal {
        color::POSITIVE
    } else if g.cover >= g.floor {
        color::ACCENT
    } else {
        color::DANGER
    };
    let mut fill = track;
    fill.set_right(x(g.cover));
    painter.rect_filled(fill, radius::SM, tone);
    for (v, c) in [(g.floor, color::DANGER), (g.legal, color::TEXT)] {
        painter.line_segment(
            [
                egui::pos2(x(v), rect.top()),
                egui::pos2(x(v), rect.bottom()),
            ],
            Stroke::new(2.0, c),
        );
    }
    response.on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr("gold-cover-tip")));
    ui.label(
        egui::RichText::new(i18n.tr_args(
            "gold-cover-line",
            &[
                ("cover", arg(p(g.cover))),
                ("legal", arg(p(g.legal))),
                ("floor", arg(p(g.floor))),
            ],
        ))
        .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
        .color(tone),
    );
}

/// On gold, how the country meets a cover below the floor: the historical rule, or a posture
/// the government fixes (defend or devalue), each with what it does.
fn posture(ui: &mut Ui, cx: &mut TabContext<'_>, g: &GoldView) {
    let i18n = cx.i18n;
    let now = cx.drafts.posture.unwrap_or(g.posture);
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::caption(ui, &i18n.tr("gold-posture"));
    let mut fixed = now != Posture::Rule;
    let options = [
        (false, i18n.tr("gold-posture-rule")),
        (true, i18n.tr("gold-posture-fixed")),
    ];
    let tips = [
        Tip::Item('O', "gov.gold.posture.rule".into(), "rule"),
        Tip::Item('O', "gov.gold.posture.fixed".into(), "fixed"),
    ];
    let mut send = None;
    if keys::segmented(ui, &mut fixed, &options, &tips) {
        send = Some(if fixed {
            Posture::Defend
        } else {
            Posture::Rule
        });
    }
    if fixed {
        let mut which = if now == Posture::Rule {
            Posture::Defend
        } else {
            now
        };
        let options = [
            (Posture::Defend, i18n.tr("gold-posture-defend")),
            (Posture::Devalue, i18n.tr("gold-posture-devalue")),
        ];
        let tips = [
            Tip::Item('O', "gov.gold.posture.defend".into(), "defend"),
            Tip::Item('O', "gov.gold.posture.devalue".into(), "devalue"),
        ];
        if keys::segmented(ui, &mut which, &options, &tips) {
            send = Some(which);
        }
    }
    let shown = send.unwrap_or(now);
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            match shown {
                Posture::Rule => "gold-posture-rule-means",
                Posture::Defend => "gold-posture-defend-means",
                Posture::Devalue => "gold-posture-devalue-means",
            },
            &[("share", arg(pct(i18n, g.devaluation * 100.0, 0)))],
        ),
    );
    if let Some(posture) = send {
        cx.commands.push(GameCommand::SetGoldPosture {
            tag: cx.tag.to_owned(),
            posture,
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.posture = Some(posture);
    }
}

/// Private capital across the border, and capital controls: on and off in two steps, each
/// saying what changes.
fn capital(ui: &mut Ui, cx: &mut TabContext<'_>, c: &crate::gold_view::CapitalView) {
    let i18n = cx.i18n;
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::caption(ui, &i18n.tr("gold-capital"));
    let tip = i18n.tr("gold-capital-tip");
    for (key, x) in [
        ("gold-capital-abroad-long", c.abroad.0),
        ("gold-capital-abroad-short", c.abroad.1),
        ("gold-capital-here-long", c.from_abroad.0),
        ("gold-capital-here-short", c.from_abroad.1),
    ] {
        let value = money(i18n, x);
        Account::new(&i18n.tr(key), &value, &tip).show(ui);
    }
    let on = cx.drafts.controls.unwrap_or(c.controls);
    let by = i18n.tr(if c.chosen.is_some() {
        "gold-by-government"
    } else {
        "gold-by-rule"
    });
    Account::new(
        &i18n.tr("gold-controls"),
        &format!(
            "{} · {by}",
            i18n.tr(if on {
                "gold-controls-on"
            } else {
                "gold-controls-off"
            })
        ),
        &i18n.tr("gold-controls-tip"),
    )
    .show(ui);
    if cx.drafts.controls_confirm {
        let explain = i18n.tr_args(
            if on {
                "gold-controls-off-means"
            } else {
                "gold-controls-on-means"
            },
            &[("points", arg(points(i18n, c.control_premium)))],
        );
        warn(ui, &explain);
        ui.horizontal(|ui| {
            let yes = Button::danger(&i18n.tr(if on {
                "gold-controls-turn-off"
            } else {
                "gold-controls-turn-on"
            }))
            .size(ButtonSize::Small)
            .show(ui);
            if yes.clicked() || keys::single(ui, "gov.gold.controls.yes", 'Y', yes.rect) {
                cx.commands.push(GameCommand::SetCapitalControls {
                    tag: cx.tag.to_owned(),
                    on: !on,
                });
                cx.drafts.touch(cx.snap.tick);
                cx.drafts.controls = Some(!on);
                cx.drafts.controls_confirm = false;
            }
            let no = Button::ghost(&i18n.tr("forces-keep"))
                .size(ButtonSize::Small)
                .show(ui);
            if no.clicked() || keys::single(ui, "gov.gold.controls.no", 'N', no.rect) {
                cx.drafts.controls_confirm = false;
            }
        });
    } else {
        let ask = Button::secondary(&i18n.tr(if on {
            "gold-controls-ask-off"
        } else {
            "gold-controls-ask-on"
        }))
        .size(ButtonSize::Small)
        .show(ui);
        if ask.clicked() || keys::single(ui, "gov.gold.controls", 'K', ask.rect) {
            cx.drafts.controls_confirm = true;
        }
    }
}
