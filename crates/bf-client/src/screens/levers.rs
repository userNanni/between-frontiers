//! The budget levers in the budget tab (`docs/state.pt-BR.md`, "Alavancas do orçamento e da
//! dívida"): the state payroll (D1), public pay (D2), the treasury's reserve and surplus (D4)
//! and the deficit cap (D5). Each one shows what the historical rule gives now and, fixed by the
//! government, a value between the simulation's floor and ceiling. Read from
//! `bf_sim::econ::levers::view` with the government's view, once a month.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::levers::{LeverLine, Levers, LeversView, SurplusUse};
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::space;
use egui::Ui;

use super::gov_ui::{StepTips, control_row, stepper};
use super::government::TabContext;
use crate::keys::{self, Tip};

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// Steps of each stepper, in the lever's own unit.
const SERVANT_STEP: i32 = 1;
const CLERGY_STEP: i32 = 1;
const PAY_STEP: i32 = 25;
const RESERVE_STEP: i32 = 1;
const DEFICIT_STEP: i32 = 5;

/// The levers as fixed now, from the view: what the commands carry.
fn levers_of(v: &LeversView) -> Levers {
    let u = |x: Option<f64>| x.map(|v| v.round() as u16);
    Levers {
        payroll: match (v.servants.fixed, v.clergy.fixed) {
            (Some(s), Some(c)) => Some([s.round() as u16, c.round() as u16]),
            _ => None,
        },
        public_pay: u(v.public_pay.fixed),
        reserve_weeks: u(v.reserve_weeks.fixed),
        surplus: v.surplus_fixed.then_some(v.surplus),
        deficit_cap: u(v.deficit_cap.fixed),
        ..Levers::default()
    }
}

fn clamp(line: &LeverLine, v: i32) -> u16 {
    (v as f64).clamp(line.floor, line.ceiling).round() as u16
}

/// The rule's value, rounded into the lever's limits: where a newly fixed value starts.
fn start(line: &LeverLine) -> u16 {
    clamp(line, line.rule.round() as i32)
}

/// One stepper row of a fixed lever; the new value when stepped.
#[allow(clippy::too_many_arguments)]
fn step_row(
    ui: &mut Ui,
    i18n: &I18n,
    label: &str,
    line: &LeverLine,
    now: u16,
    shown: String,
    step: i32,
    tips: StepTips<'_>,
) -> Option<u16> {
    let tip = i18n.tr_args(
        "lever-limits",
        &[
            ("floor", arg(i18n.decimal(line.floor, 0))),
            ("ceiling", arg(i18n.decimal(line.ceiling, 0))),
        ],
    );
    let d = control_row(ui, label, |ui| {
        stepper(
            ui,
            &shown,
            &tip,
            f64::from(now) > line.floor,
            f64::from(now) < line.ceiling,
            tips,
        )
    });
    (d != 0).then(|| clamp(line, i32::from(now) + d * step))
}

/// Per thousand people, as the payroll is counted.
fn per_mille_people(i18n: &I18n, v: f64) -> String {
    i18n.tr_args("lever-per-mille-people", &[("n", arg(i18n.decimal(v, 0)))])
}

/// Thousandths as a percentage.
fn pct_of_milli(i18n: &I18n, v: f64) -> String {
    format!("{} %", i18n.decimal(v / 10.0, 1))
}

/// What a fixed value does to the treasury in a week against the rule, when the simulation
/// prices it.
fn week_effect(ui: &mut Ui, i18n: &I18n, line: &LeverLine) {
    let Some(e) = line.week_effect.filter(|e| e.abs() >= 0.5) else {
        return;
    };
    let amount = if e > 0.0 {
        format!("+{}", crate::text::money(i18n, e))
    } else {
        super::budget::signed_money(i18n, e)
    };
    bf_ui_kit::small(
        ui,
        &i18n.tr_args("lever-week-effect", &[("amount", arg(amount))]),
    );
}

/// The head of a lever with its key tips; true when the choice changed.
fn head(ui: &mut Ui, i18n: &I18n, label_key: &str, fixed: &mut bool, id: &str) -> bool {
    let (changed, rects) = bf_ui_kit::lever_header(
        ui,
        &i18n.tr(label_key),
        fixed,
        &i18n.tr("lever-rule"),
        &i18n.tr("lever-fixed"),
    );
    let mut typed = false;
    for (r, (key, on)) in rects.into_iter().zip([("rule", false), ("fixed", true)]) {
        if keys::item(ui, 'F', format!("{id}.{key}"), key, r) && *fixed != on {
            *fixed = on;
            typed = true;
        }
    }
    changed || typed
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>, view: &LeversView) {
    let i18n = cx.i18n;
    // What the player set this tick wins over the view until the tick applies it.
    let mut l = cx.drafts.levers.unwrap_or_else(|| levers_of(view));
    let before = l;
    let mut out: Vec<GameCommand> = Vec::new();
    let tag = cx.tag.to_owned();

    bf_ui_kit::heading(ui, &i18n.tr("lever-heading"));
    bf_ui_kit::small(ui, &i18n.tr("lever-basis"));

    // D1, the state payroll: servants and clergy, fixed together.
    bf_ui_kit::gap(ui, space::SM);
    let mut fixed = l.payroll.is_some();
    if head(ui, i18n, "lever-payroll", &mut fixed, "gov.lever.payroll") {
        l.payroll = fixed.then(|| [start(&view.servants), start(&view.clergy)]);
    }
    bf_ui_kit::lever_rule_line(
        ui,
        &i18n.tr_args(
            "lever-payroll-rule",
            &[
                ("servants", arg(per_mille_people(i18n, view.servants.rule))),
                ("clergy", arg(per_mille_people(i18n, view.clergy.rule))),
            ],
        ),
        l.payroll.map(|_| i18n.tr("lever-payroll-note")).as_deref(),
    );
    if let Some([s, c]) = l.payroll {
        let mut p = [s, c];
        for (k, (line, now, key, step)) in [
            (&view.servants, s, "lever-servants", SERVANT_STEP),
            (&view.clergy, c, "lever-clergy", CLERGY_STEP),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(v) = step_row(
                ui,
                i18n,
                &i18n.tr(key),
                line,
                now,
                i18n.decimal(f64::from(now), 0),
                step,
                StepTips {
                    family: 'F',
                    id: &format!("gov.lever.payroll.{k}"),
                    row: k,
                },
            ) {
                p[k] = v;
            }
        }
        l.payroll = Some(p);
        week_effect(ui, i18n, &view.servants);
        week_effect(ui, i18n, &view.clergy);
    }
    // O1: with the fiscal capacity on, servants short of the rule leave taxes uncollected.
    if let Some(u) = view.uncollected.filter(|u| *u > 0.0) {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "lever-uncollected",
                &[("share", arg(format!("{} %", i18n.decimal(u * 100.0, 1))))],
            ),
        );
    }
    if l.payroll != before.payroll {
        out.push(GameCommand::SetPayroll {
            tag: tag.clone(),
            per_mille: l.payroll,
        });
    }

    // D2, public pay.
    bf_ui_kit::gap(ui, space::SM);
    let mut fixed = l.public_pay.is_some();
    if head(ui, i18n, "lever-pay", &mut fixed, "gov.lever.pay") {
        l.public_pay = fixed.then(|| start(&view.public_pay));
    }
    bf_ui_kit::lever_rule_line(
        ui,
        &i18n.tr_args(
            "lever-pay-rule",
            &[("share", arg(pct_of_milli(i18n, view.public_pay.rule)))],
        ),
        l.public_pay.map(|_| i18n.tr("lever-pay-note")).as_deref(),
    );
    if let Some(now) = l.public_pay
        && let Some(v) = step_row(
            ui,
            i18n,
            &i18n.tr("lever-pay-fixed"),
            &view.public_pay,
            now,
            pct_of_milli(i18n, f64::from(now)),
            PAY_STEP,
            StepTips {
                family: 'F',
                id: "gov.lever.pay",
                row: 2,
            },
        )
    {
        l.public_pay = Some(v);
    }
    if l.public_pay.is_some() {
        week_effect(ui, i18n, &view.public_pay);
    }
    if l.public_pay != before.public_pay {
        out.push(GameCommand::SetPublicPay {
            tag: tag.clone(),
            per_mille: l.public_pay,
        });
    }

    // D4, the treasury: the reserve and what the surplus is for, fixed together.
    bf_ui_kit::gap(ui, space::SM);
    let mut fixed = l.reserve_weeks.is_some() || l.surplus.is_some();
    if head(ui, i18n, "lever-treasury", &mut fixed, "gov.lever.treasury") {
        if fixed {
            l.reserve_weeks = Some(start(&view.reserve_weeks));
            l.surplus = Some(view.surplus);
        } else {
            l.reserve_weeks = None;
            l.surplus = None;
        }
    }
    bf_ui_kit::lever_rule_line(
        ui,
        &i18n.tr_args(
            "lever-treasury-rule",
            &[("weeks", arg(i18n.decimal(view.reserve_weeks.rule, 0)))],
        ),
        fixed.then(|| i18n.tr("lever-treasury-note")).as_deref(),
    );
    if fixed {
        let now = l
            .reserve_weeks
            .unwrap_or_else(|| start(&view.reserve_weeks));
        if let Some(v) = step_row(
            ui,
            i18n,
            &i18n.tr("lever-reserve"),
            &view.reserve_weeks,
            now,
            now.to_string(),
            RESERVE_STEP,
            StepTips {
                family: 'F',
                id: "gov.lever.reserve",
                row: 3,
            },
        ) {
            l.reserve_weeks = Some(v);
        }
        let mut use_ = l.surplus.unwrap_or(view.surplus);
        // Added from the right inside the row: listed backwards so it reads spend, repay, keep.
        let options = [
            (SurplusUse::Keep, i18n.tr("lever-surplus-keep")),
            (SurplusUse::Repay, i18n.tr("lever-surplus-repay")),
            (SurplusUse::Spend, i18n.tr("lever-surplus-spend")),
        ];
        let tips = [
            Tip::Item('F', "gov.lever.surplus.keep".into(), "keep"),
            Tip::Item('F', "gov.lever.surplus.repay".into(), "repay"),
            Tip::Item('F', "gov.lever.surplus.spend".into(), "spend"),
        ];
        control_row(ui, &i18n.tr("lever-surplus"), |ui| {
            keys::segmented(ui, &mut use_, &options, &tips)
        });
        l.surplus = Some(use_);
        l.reserve_weeks = Some(l.reserve_weeks.unwrap_or(now));
    }
    if (l.reserve_weeks, l.surplus) != (before.reserve_weeks, before.surplus) {
        out.push(GameCommand::SetTreasury {
            tag: tag.clone(),
            reserve_weeks: l.reserve_weeks,
            surplus: l.surplus,
        });
    }

    // D5, the deficit cap.
    bf_ui_kit::gap(ui, space::SM);
    let mut fixed = l.deficit_cap.is_some();
    if head(ui, i18n, "lever-deficit", &mut fixed, "gov.lever.deficit") {
        l.deficit_cap = fixed.then(|| start(&view.deficit_cap));
    }
    bf_ui_kit::lever_rule_line(
        ui,
        &i18n.tr_args(
            "lever-deficit-rule",
            &[("share", arg(pct_of_milli(i18n, view.deficit_cap.rule)))],
        ),
        l.deficit_cap
            .map(|_| i18n.tr("lever-deficit-note"))
            .as_deref(),
    );
    if let Some(now) = l.deficit_cap
        && let Some(v) = step_row(
            ui,
            i18n,
            &i18n.tr("lever-deficit-fixed"),
            &view.deficit_cap,
            now,
            pct_of_milli(i18n, f64::from(now)),
            DEFICIT_STEP,
            StepTips {
                family: 'F',
                id: "gov.lever.deficit",
                row: 4,
            },
        )
    {
        l.deficit_cap = Some(v);
    }
    if l.deficit_cap.is_some() {
        week_effect(ui, i18n, &view.deficit_cap);
    }
    if l.deficit_cap != before.deficit_cap {
        out.push(GameCommand::SetDeficitCap {
            tag,
            per_mille_gdp: l.deficit_cap,
        });
    }

    if !out.is_empty() {
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.levers = Some(l);
        cx.commands.extend(out);
    }
}
