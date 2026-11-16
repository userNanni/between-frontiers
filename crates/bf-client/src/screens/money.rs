//! Government tab: money and debt. Prices, exchange rate and the central bank on the left with
//! its controls; the public debt by holder, the interest it costs and the defaults on the right.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::money::{CentralBank, MAX_RATE, NEUTRAL_REAL, TARGET};
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::{color, space};
use egui::Ui;

use super::budget::{WEEKS, of_gdp, pct, pending_note};
use super::gov_ui::{Account, StepTips, control_row, stepper};
use super::government::TabContext;
use crate::government::GovernmentView;
use crate::keys::{self, Tip};
use crate::text::money;

/// Step of the policy rate, in thousandths a year (half a point).
const RATE_STEP: i32 = 5;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn fx(x: laplace_schelling::Fixed) -> f64 {
    x.to_f64_lossy()
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let Some(v) = cx.view else {
        bf_ui_kit::muted(ui, &cx.i18n.tr(super::government::waiting_key(cx.snap)));
        return;
    };
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &cx.i18n.tr("gov-money-basis"));
    bf_ui_kit::gap(ui, space::SM);
    ui.spacing_mut().item_spacing.x = space::XXL;
    ui.columns(2, |cols| {
        for c in cols.iter_mut() {
            c.spacing_mut().item_spacing = egui::vec2(space::SM, space::XS);
        }
        bank(&mut cols[0], cx, v);
        debt(&mut cols[1], cx.i18n, v);
    });
}

fn bank(ui: &mut Ui, cx: &mut TabContext<'_>, v: &GovernmentView) {
    let i18n = cx.i18n;
    let m = &v.money;
    let p = |x: f64, d: usize| pct(i18n, x, d);

    bf_ui_kit::heading(ui, &i18n.tr("gov-prices"));
    let inflation = fx(m.inflation);
    let tip = i18n.tr_args(
        if v.subordinate {
            "gov-tip-inflation-subordinate"
        } else {
            "gov-tip-inflation-autonomous"
        },
        &[
            ("now", arg(p(inflation, 1))),
            ("last", arg(p(fx(m.inflation_last_year), 1))),
            ("expected", arg(p(fx(m.expected), 1))),
            ("target", arg(p(fx(TARGET), 0))),
            ("monetized", arg(p(fx(m.monetized), 1))),
        ],
    );
    let value = p(inflation, 1);
    Account::new(&i18n.tr("gov-inflation"), &value, &tip)
        .tone(if inflation > 10.0 {
            color::DANGER
        } else {
            color::TEXT
        })
        .show(ui);
    let tip = i18n.tr_args(
        "gov-tip-price-level",
        &[
            ("jan", arg(i18n.decimal(fx(m.price_level_jan), 1))),
            ("reforms", arg(m.redenominations.to_string())),
        ],
    );
    let value = i18n.decimal(fx(m.price_level), 1);
    Account::new(&i18n.tr("gov-price-level"), &value, &tip).show(ui);
    let tip = i18n.tr_args(
        "gov-tip-exchange",
        &[
            ("rate", arg(i18n.decimal(fx(m.exchange_rate), 1))),
            ("parity", arg(i18n.decimal(fx(m.parity), 1))),
            ("real", arg(i18n.decimal(fx(m.real_exchange()), 1))),
        ],
    );
    let value = i18n.decimal(fx(m.exchange_rate), 1);
    Account::new(&i18n.tr("gov-exchange"), &value, &tip).show(ui);
    bf_ui_kit::gap(ui, space::MD);

    bf_ui_kit::heading(ui, &i18n.tr("gov-central-bank"));
    let rate = fx(m.policy_rate);
    let tip = match m.rate_command {
        Some(r) => i18n.tr_args("gov-tip-rate-fixed", &[("rate", arg(p(fx(r), 2)))]),
        None => {
            let neutral = fx(NEUTRAL_REAL) + inflation;
            let gap_term = 0.5 * (inflation - fx(TARGET));
            // The rule's third term (half the utilization gap) is what is left.
            let rest = rate - neutral - gap_term;
            i18n.tr_args(
                "gov-tip-rate-taylor",
                &[
                    ("real", arg(p(fx(NEUTRAL_REAL), 0))),
                    ("inflation", arg(p(inflation, 2))),
                    ("neutral", arg(p(neutral, 2))),
                    ("target", arg(p(fx(TARGET), 0))),
                    ("infl_term", arg(p(gap_term, 2))),
                    ("gap_term", arg(p(rest, 2))),
                    ("rate", arg(p(rate, 2))),
                ],
            )
        }
    };
    let value = p(rate, 2);
    Account::new(&i18n.tr("gov-policy-rate"), &value, &tip)
        .strong()
        .show(ui);
    let regime = if v.subordinate {
        "gov-regime-subordinate"
    } else {
        "gov-regime-autonomous"
    };
    let tip = match m.bank {
        CentralBank::Historical if m.autonomous_since > 0 => i18n.tr_args(
            "gov-tip-regime-historical-since",
            &[("year", arg(m.autonomous_since.to_string()))],
        ),
        CentralBank::Historical => i18n.tr("gov-tip-regime-historical"),
        CentralBank::Autonomous => i18n.tr("gov-tip-regime-autonomous"),
        CentralBank::Subordinate => i18n.tr("gov-tip-regime-subordinate"),
    };
    let value = i18n.tr(regime);
    Account::new(&i18n.tr("gov-regime"), &value, &tip).show(ui);
    let gdp = v.gdp_last_year;
    let value = money(i18n, fx(m.credit));
    let tip = i18n.tr_args(
        "gov-tip-credit",
        &[("gdp", arg(of_gdp(i18n, fx(m.credit), gdp)))],
    );
    Account::new(&i18n.tr("gov-credit"), &value, &tip).show(ui);
    let value = i18n.tr_args(
        "gov-pct-gdp-year",
        &[("value", arg(i18n.decimal(fx(m.monetized), 1)))],
    );
    let tip = i18n.tr("gov-tip-monetized");
    Account::new(&i18n.tr("gov-monetized"), &value, &tip)
        .tone(if fx(m.monetized) > 0.5 {
            color::DANGER
        } else {
            color::TEXT
        })
        .show(ui);
    bf_ui_kit::gap(ui, space::MD);

    // Controls: who the bank answers to, and its rate.
    bf_ui_kit::heading(ui, &i18n.tr("gov-bank-controls"));
    let mut bank = cx.drafts.bank.unwrap_or(m.bank);
    let options = [
        (CentralBank::Historical, i18n.tr("gov-bank-historical")),
        (CentralBank::Autonomous, i18n.tr("gov-bank-autonomous")),
        (CentralBank::Subordinate, i18n.tr("gov-bank-subordinate")),
    ];
    let tips = [
        Tip::Item('A', "gov.bank.historical".into(), "historical"),
        Tip::Item('A', "gov.bank.autonomous".into(), "autonomous"),
        Tip::Item('A', "gov.bank.subordinate".into(), "subordinate"),
    ];
    if keys::segmented(ui, &mut bank, &options, &tips) {
        cx.commands.push(GameCommand::SetCentralBank {
            tag: cx.tag.to_owned(),
            bank,
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.bank = Some(bank);
    }
    bf_ui_kit::gap(ui, space::XS);
    let current: Option<u16> = match cx.drafts.rate {
        Some(r) => r,
        None => m.rate_command.map(|r| (fx(r) * 10.0).round() as u16),
    };
    let mut fixed = current.is_some();
    let options = [
        (false, i18n.tr("gov-rate-rule")),
        (true, i18n.tr("gov-rate-fixed")),
    ];
    let shown = current.unwrap_or_else(|| (rate * 10.0).round() as u16);
    let tips = [
        Tip::Item('V', "gov.rate.rule".into(), "rule"),
        Tip::Item('V', "gov.rate.fixed".into(), "fixed"),
    ];
    if keys::segmented(ui, &mut fixed, &options, &tips) {
        let rate = fixed.then_some(shown);
        cx.commands.push(GameCommand::SetPolicyRate {
            tag: cx.tag.to_owned(),
            rate,
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.rate = Some(rate);
    }
    let max = (fx(MAX_RATE) * 10.0).round() as u16;
    let tip = i18n.tr("gov-tip-rate-step");
    let step = control_row(ui, &i18n.tr("gov-policy-rate"), |ui| {
        stepper(
            ui,
            &p(f64::from(shown) / 10.0, 1),
            &tip,
            shown > 0,
            shown < max,
            StepTips {
                family: 'V',
                id: "gov.rate",
                row: 0,
            },
        )
    });
    if step != 0 {
        let new = (i32::from(shown) + step * RATE_STEP).clamp(0, i32::from(max)) as u16;
        cx.commands.push(GameCommand::SetPolicyRate {
            tag: cx.tag.to_owned(),
            rate: Some(new),
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.rate = Some(Some(new));
    }
    pending_note(ui, cx);
}

fn debt(ui: &mut Ui, i18n: &I18n, v: &GovernmentView) {
    let m = &v.money;
    let b = &v.budget;
    let gdp = v.gdp_last_year;
    let p = |x: f64, d: usize| pct(i18n, x, d);

    bf_ui_kit::heading(ui, &i18n.tr("gov-debt"));
    let total = fx(m.total_debt());
    let holders = [
        (fx(m.debt_home), "gov-debt-home", "gov-tip-debt-home"),
        (fx(m.debt_bank), "gov-debt-bank", "gov-tip-debt-bank"),
        (fx(m.debt_abroad), "gov-debt-abroad", "gov-tip-debt-abroad"),
        (fx(m.debt_fx_local()), "gov-debt-fx", "gov-tip-debt-fx"),
    ];
    for (amount, label, tip) in holders {
        let tip = i18n.tr_args(
            tip,
            &[
                ("amount", arg(money(i18n, amount))),
                ("gdp", arg(of_gdp(i18n, amount, gdp))),
                (
                    "share",
                    arg(p(
                        if total > 0.0 {
                            amount / total * 100.0
                        } else {
                            0.0
                        },
                        0,
                    )),
                ),
                ("fx", arg(money(i18n, fx(m.debt_fx)))),
                ("real", arg(i18n.decimal(fx(m.real_exchange()), 1))),
            ],
        );
        let value = money(i18n, amount);
        Account::new(&i18n.tr(label), &value, &tip)
            .share(if total > 0.0 { amount / total } else { 0.0 })
            .show(ui);
    }
    let tip = i18n.tr_args(
        "gov-tip-debt-total",
        &[
            ("gdp", arg(of_gdp(i18n, total, gdp))),
            ("gdp_value", arg(money(i18n, gdp))),
        ],
    );
    let value = i18n.tr_args(
        "gov-debt-total-value",
        &[
            ("amount", arg(money(i18n, total))),
            ("gdp", arg(of_gdp(i18n, total, gdp))),
        ],
    );
    Account::new(&i18n.tr("gov-debt-total"), &value, &tip)
        .strong()
        .show(ui);
    bf_ui_kit::gap(ui, space::MD);

    bf_ui_kit::heading(ui, &i18n.tr("gov-interest"));
    let interest = fx(b.interest) * WEEKS;
    let revenue = fx(b.revenue) * WEEKS;
    let tip = i18n.tr_args(
        "gov-tip-interest-paid",
        &[
            ("week", arg(money(i18n, fx(b.interest)))),
            (
                "revenue",
                arg(if revenue > 0.0 {
                    p(interest / revenue * 100.0, 1)
                } else {
                    "—".into()
                }),
            ),
            ("gdp", arg(of_gdp(i18n, interest, gdp))),
        ],
    );
    let value = money(i18n, interest);
    Account::new(&i18n.tr("gov-interest-paid"), &value, &tip).show(ui);
    let rows = [
        ("gov-bond-rate", fx(m.bond_rate), "gov-tip-bond-rate"),
        ("gov-fx-rate", fx(m.fx_rate), "gov-tip-fx-rate"),
        ("gov-cost-local", fx(m.cost_local), "gov-tip-cost"),
        ("gov-cost-fx", fx(m.cost_fx), "gov-tip-cost"),
    ];
    for (label, rate, tip) in rows {
        let value = p(rate, 2);
        Account::new(&i18n.tr(label), &value, &i18n.tr(tip)).show(ui);
    }
    bf_ui_kit::gap(ui, space::MD);

    bf_ui_kit::heading(ui, &i18n.tr("gov-defaults"));
    let year = v.date.year;
    let tip = i18n.tr_args(
        "gov-tip-defaults",
        &[("history", arg(i18n.decimal(fx(m.default_history), 2)))],
    );
    let value = i18n.int(i64::from(m.defaults));
    Account::new(&i18n.tr("gov-defaults-count"), &value, &tip)
        .tone(if m.defaults > 0 {
            color::DANGER
        } else {
            color::TEXT
        })
        .show(ui);
    let (value, tip) = if m.excluded_until > year {
        (
            i18n.tr_args(
                "gov-excluded-until",
                &[("year", arg(m.excluded_until.to_string()))],
            ),
            i18n.tr("gov-tip-excluded"),
        )
    } else {
        (i18n.tr("gov-credit-open"), i18n.tr("gov-tip-credit-open"))
    };
    Account::new(&i18n.tr("gov-foreign-credit"), &value, &tip).show(ui);
    let value = money(i18n, fx(m.foreign_assets));
    Account::new(
        &i18n.tr("gov-foreign-assets"),
        &value,
        &i18n.tr("gov-tip-foreign-assets"),
    )
    .show(ui);
}
