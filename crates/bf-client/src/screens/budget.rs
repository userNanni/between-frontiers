//! Government tab: the budget. Revenue by tax class, spending by item, the balance and the
//! treasury on the left; tax rates, public works and the migration law on the right. Every
//! figure carries the sum behind it in its tooltip.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::migration::MigrationLaw;
use bf_sim::econ::public::{
    DEFAULT_PRIORITY, MAX_PRIORITY, MAX_TAX, PUBLIC_PAY, RESERVE_WEEKS, historical_rates,
};
use bf_sim::econ::state::{PUBLIC_CATEGORIES, TAX_CLASSES};
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::{color, space};
use egui::Ui;

use super::gov_ui::{Account, StepTips, control_row, stepper};
use super::government::TabContext;
use crate::government::{GovernmentView, Work};
use crate::keys::{self, Tip};
use crate::text::money;

/// Weeks in the year the weekly accounts are multiplied by.
pub const WEEKS: f64 = 52.0;
/// Step of a tax rate, in thousandths (one point).
const TAX_STEP: i32 = 10;
/// Public works listed one by one.
const WORKS_LISTED: usize = 8;

pub fn pct(i18n: &I18n, percent: f64, digits: usize) -> String {
    i18n.tr_args(
        "gov-pct",
        &[("value", Arg::String(i18n.decimal(percent, digits)))],
    )
}

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn fx(x: laplace_schelling::Fixed) -> f64 {
    x.to_f64_lossy()
}

/// Money with the minus sign ahead of the currency: "−US$ 1,6 bilhão".
pub fn signed_money(i18n: &I18n, value: f64) -> String {
    if value < 0.0 {
        format!("−{}", money(i18n, -value))
    } else {
        money(i18n, value)
    }
}

/// "US$ 9,3 bilhões · 12,7 % do PIB"; the amount alone without a GDP.
pub fn with_gdp(i18n: &I18n, amount: String, yearly: f64, gdp: f64) -> String {
    if gdp > 0.0 {
        i18n.tr_args(
            "gov-amount-gdp",
            &[
                ("amount", arg(amount)),
                ("gdp", arg(pct(i18n, yearly / gdp * 100.0, 1))),
            ],
        )
    } else {
        amount
    }
}

/// The purchases: what the state buys, and the rule that sets how much, term by term.
fn goods_tip(i18n: &I18n, v: &GovernmentView) -> String {
    let r = &v.goods_rule;
    let y = |x: f64| money(i18n, x * WEEKS);
    let planned = r.revenue + r.deficit_room - r.committed;
    i18n.tr_args(
        "gov-tip-goods",
        &[
            ("revenue", arg(y(r.revenue))),
            ("room", arg(y(r.deficit_room))),
            ("committed", arg(y(r.committed))),
            ("planned", arg(signed_money(i18n, planned * WEEKS))),
            ("floor", arg(y(r.floor))),
            ("kept", arg(pct(i18n, r.austerity * 100.0, 0))),
            ("total", arg(y(r.total))),
        ],
    )
}

/// Share of the GDP of a yearly amount, as text ("—" without a GDP).
pub fn of_gdp(i18n: &I18n, yearly: f64, gdp: f64) -> String {
    if gdp > 0.0 {
        pct(i18n, yearly / gdp * 100.0, 1)
    } else {
        "—".into()
    }
}

/// Public works category of a building, by its key.
pub fn category(key: &str) -> usize {
    match key {
        "school" => 1,
        "hospital" => 2,
        "urban_housing" => 3,
        _ => 0,
    }
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let Some(v) = cx.view else {
        bf_ui_kit::muted(ui, &cx.i18n.tr(super::government::waiting_key(cx.snap)));
        return;
    };
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &cx.i18n.tr("gov-budget-basis"));
    bf_ui_kit::gap(ui, space::SM);
    ui.spacing_mut().item_spacing.x = space::XXL;
    ui.columns(2, |cols| {
        for c in cols.iter_mut() {
            c.spacing_mut().item_spacing = egui::vec2(space::SM, space::XS);
        }
        accounts(&mut cols[0], cx.i18n, v);
        controls(&mut cols[1], cx, v);
    });
}

fn accounts(ui: &mut Ui, i18n: &I18n, v: &GovernmentView) {
    let b = &v.budget;
    let gdp = v.gdp_last_year;
    let revenue = fx(b.revenue);
    let tax = v.tax_by_class();

    bf_ui_kit::heading(ui, &i18n.tr("gov-revenue"));
    for c in 0..TAX_CLASSES {
        let rate = fx(b.tax_rates[c]) * 100.0;
        let tip = i18n.tr_args(
            "gov-tip-class-tax",
            &[
                ("people", arg(i18n.compact(v.class_people[c] as f64))),
                ("income", arg(money(i18n, v.class_income[c] * WEEKS))),
                ("rate", arg(pct(i18n, rate, 1))),
                ("tax", arg(money(i18n, tax[c] * WEEKS))),
                ("members", arg(i18n.tr(&format!("gov-class-{c}-members")))),
            ],
        );
        let value = money(i18n, tax[c] * WEEKS);
        let label = i18n.tr(&format!("gov-class-{c}"));
        Account::new(&label, &value, &tip)
            .share(if revenue > 0.0 { tax[c] / revenue } else { 0.0 })
            .show(ui);
    }
    let tip = i18n.tr_args(
        "gov-tip-revenue",
        &[
            ("week", arg(money(i18n, revenue))),
            ("year", arg(money(i18n, revenue * WEEKS))),
            ("gdp", arg(of_gdp(i18n, revenue * WEEKS, gdp))),
            ("avg", arg(money(i18n, fx(b.revenue_avg) * WEEKS))),
        ],
    );
    let value = with_gdp(i18n, money(i18n, revenue * WEEKS), revenue * WEEKS, gdp);
    Account::new(&i18n.tr("gov-revenue-total"), &value, &tip)
        .strong()
        .show(ui);
    bf_ui_kit::gap(ui, space::MD);

    bf_ui_kit::heading(ui, &i18n.tr("gov-spending"));
    let spending = v.spending();
    let total: f64 = spending.iter().sum();
    let wage_pct = pct(i18n, fx(PUBLIC_PAY) * 100.0, 0);
    let interest_cost = if fx(b.debt) > 0.0 {
        pct(i18n, fx(b.interest) * WEEKS / fx(b.debt) * 100.0, 1)
    } else {
        "—".into()
    };
    let tips = [
        i18n.tr_args(
            "gov-tip-payroll",
            &[
                ("workers", arg(i18n.compact(v.gov_workers as f64))),
                ("pay", arg(wage_pct.clone())),
            ],
        ),
        goods_tip(i18n, v),
        i18n.tr("gov-tip-services"),
        i18n.tr("gov-tip-works"),
        i18n.tr_args(
            "gov-tip-interest",
            &[
                ("debt", arg(money(i18n, fx(b.debt)))),
                ("cost", arg(interest_cost)),
            ],
        ),
    ];
    for (k, (amount, tip)) in spending.iter().zip(&tips).enumerate() {
        let value = money(i18n, amount * WEEKS);
        let tip = format!(
            "{tip}\n{}",
            i18n.tr_args(
                "gov-tip-yearly",
                &[
                    ("week", arg(money(i18n, *amount))),
                    ("year", arg(value.clone())),
                    ("gdp", arg(of_gdp(i18n, amount * WEEKS, gdp))),
                ],
            )
        );
        Account::new(&i18n.tr(&format!("gov-spend-{k}")), &value, &tip)
            .share(if total > 0.0 { amount / total } else { 0.0 })
            .bar(if k == 4 { color::DANGER } else { color::ACCENT })
            .show(ui);
    }
    let value = with_gdp(i18n, money(i18n, total * WEEKS), total * WEEKS, gdp);
    let tip = i18n.tr_args(
        "gov-tip-spending",
        &[("gdp", arg(of_gdp(i18n, total * WEEKS, gdp)))],
    );
    Account::new(&i18n.tr("gov-spending-total"), &value, &tip)
        .strong()
        .show(ui);
    bf_ui_kit::gap(ui, space::MD);

    // Balance and how it is covered.
    let balance = v.balance();
    let tip = i18n.tr_args(
        if balance < 0.0 {
            "gov-tip-deficit"
        } else {
            "gov-tip-surplus"
        },
        &[
            ("revenue", arg(money(i18n, revenue * WEEKS))),
            ("spending", arg(money(i18n, total * WEEKS))),
            ("balance", arg(money(i18n, balance.abs() * WEEKS))),
            ("gdp", arg(of_gdp(i18n, balance.abs() * WEEKS, gdp))),
            ("bank", arg(money(i18n, fx(b.issued) * WEEKS))),
        ],
    );
    let value = with_gdp(
        i18n,
        signed_money(i18n, balance * WEEKS),
        balance * WEEKS,
        gdp,
    );
    Account::new(&i18n.tr("gov-balance"), &value, &tip)
        .strong()
        .tone(if balance < 0.0 {
            color::DANGER
        } else {
            color::POSITIVE
        })
        .show(ui);
    let reserve = v.reserve();
    let tip = i18n.tr_args(
        "gov-tip-treasury",
        &[
            ("cash", arg(signed_money(i18n, v.treasury))),
            ("reserve", arg(money(i18n, reserve))),
            ("weeks", arg(RESERVE_WEEKS.to_string())),
            ("avg", arg(money(i18n, fx(b.revenue_avg)))),
        ],
    );
    let value = signed_money(i18n, v.treasury);
    Account::new(&i18n.tr("gov-treasury"), &value, &tip)
        .share(if reserve > 0.0 {
            v.treasury / reserve
        } else {
            0.0
        })
        .tone(if v.treasury < 0.0 {
            color::DANGER
        } else {
            color::TEXT
        })
        .show(ui);
    let value = money(i18n, fx(b.issued) * WEEKS);
    let tip = i18n.tr("gov-tip-issued");
    Account::new(&i18n.tr("gov-issued"), &value, &tip).show(ui);
}

/// Thousandths a rate shows as (the command's unit).
fn milli(x: laplace_schelling::Fixed) -> u16 {
    (x.to_f64_lossy() * 1000.0).round().clamp(0.0, 1000.0) as u16
}

fn controls(ui: &mut Ui, cx: &mut TabContext<'_>, v: &GovernmentView) {
    let i18n = cx.i18n;
    let b = &v.budget;
    let year = v.date.year;
    let rule: Vec<f64> = historical_rates(year)
        .iter()
        .map(|r| (fx(*r) * fx(b.adjust)).min(fx(MAX_TAX)))
        .collect();

    // Taxes.
    bf_ui_kit::heading(ui, &i18n.tr("gov-taxes"));
    let (auto, rates): (bool, [u16; TAX_CLASSES]) = match cx.drafts.tax {
        Some(Some(r)) => (false, r),
        Some(None) => (
            true,
            std::array::from_fn(|c| (rule[c] * 1000.0).round() as u16),
        ),
        None => (b.tax_auto, std::array::from_fn(|c| milli(b.tax_rates[c]))),
    };
    let mut mode = auto;
    let options = [
        (true, i18n.tr("gov-tax-rule")),
        (false, i18n.tr("gov-tax-fixed")),
    ];
    let tips = [
        Tip::Item('T', "gov.tax.rule".into(), "rule"),
        Tip::Item('T', "gov.tax.fixed".into(), "fixed"),
    ];
    if keys::segmented(ui, &mut mode, &options, &tips) {
        let rates = (!mode).then_some(rates);
        cx.commands.push(GameCommand::SetTaxRates {
            tag: cx.tag.to_owned(),
            rates,
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.tax = Some(rates);
    }
    let adjust = fx(b.adjust);
    for c in 0..TAX_CLASSES {
        let base = fx(historical_rates(year)[c]) * 100.0;
        let tip = i18n.tr_args(
            "gov-tip-tax-rule",
            &[
                ("year", arg(year.to_string())),
                ("base", arg(pct(i18n, base, 1))),
                ("adjust", arg(i18n.decimal(adjust, 2))),
                ("rule", arg(pct(i18n, rule[c] * 100.0, 1))),
                ("max", arg(pct(i18n, fx(MAX_TAX) * 100.0, 0))),
            ],
        );
        let now = rates[c];
        let max = milli(MAX_TAX);
        let step = control_row(ui, &i18n.tr(&format!("gov-class-{c}")), |ui| {
            stepper(
                ui,
                &pct(i18n, f64::from(now) / 10.0, 1),
                &tip,
                now > 0,
                now < max,
                StepTips {
                    family: 'T',
                    id: &format!("gov.tax.{c}"),
                    row: c,
                },
            )
        });
        if step != 0 {
            let mut new = rates;
            new[c] = (i32::from(now) + step * TAX_STEP).clamp(0, i32::from(max)) as u16;
            cx.commands.push(GameCommand::SetTaxRates {
                tag: cx.tag.to_owned(),
                rates: Some(new),
            });
            cx.drafts.touch(cx.snap.tick);
            cx.drafts.tax = Some(Some(new));
        }
    }
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "gov-tax-adjust",
            &[("adjust", arg(i18n.decimal(adjust, 2)))],
        ),
    );
    pending_note(ui, cx);
    bf_ui_kit::gap(ui, space::MD);

    // Public works: weights and the queue.
    bf_ui_kit::heading(ui, &i18n.tr("gov-works"));
    let weights = cx.drafts.priority.unwrap_or(b.priority);
    let sum: u32 = weights.iter().map(|w| u32::from(*w)).sum();
    let mut by_cat = [(0usize, 0.0f64, 0.0f64); PUBLIC_CATEGORIES];
    let public: Vec<&Work> = v.works.iter().filter(|w| w.public).collect();
    for w in &public {
        let key = cx
            .catalogue
            .buildings
            .get(usize::from(w.kind))
            .map_or("", |b| b.key.as_str());
        let e = &mut by_cat[category(key)];
        e.0 += 1;
        e.1 += w.points;
        e.2 += w.points - w.left;
    }
    for k in 0..PUBLIC_CATEGORIES {
        let share = if sum > 0 {
            f64::from(weights[k]) / f64::from(sum) * 100.0
        } else {
            0.0
        };
        let (n, points, done) = by_cat[k];
        let tip = i18n.tr_args(
            "gov-tip-priority",
            &[
                ("weight", arg(weights[k].to_string())),
                ("sum", arg(sum.to_string())),
                ("share", arg(pct(i18n, share, 0))),
                ("default", arg(DEFAULT_PRIORITY[k].to_string())),
                ("works", arg(i18n.int(n as i64))),
                (
                    "done",
                    arg(pct(
                        i18n,
                        if points > 0.0 {
                            done / points * 100.0
                        } else {
                            0.0
                        },
                        0,
                    )),
                ),
            ],
        );
        let label = i18n.tr_args(
            &format!("gov-cat-{k}"),
            &[("share", arg(pct(i18n, share, 0)))],
        );
        let step = control_row(ui, &label, |ui| {
            stepper(
                ui,
                &weights[k].to_string(),
                &tip,
                weights[k] > 0,
                weights[k] < MAX_PRIORITY,
                StepTips {
                    family: 'I',
                    id: &format!("gov.priority.{k}"),
                    row: k,
                },
            )
        });
        if step != 0 {
            let mut new = weights;
            new[k] = (i32::from(weights[k]) + step).clamp(0, i32::from(MAX_PRIORITY)) as u8;
            cx.commands.push(GameCommand::SetPublicPriority {
                tag: cx.tag.to_owned(),
                weights: new,
            });
            cx.drafts.touch(cx.snap.tick);
            cx.drafts.priority = Some(new);
        }
    }
    if sum == 0 {
        bf_ui_kit::small(ui, &i18n.tr("gov-works-stopped"));
    }
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::caption(
        ui,
        &i18n.tr_args(
            "gov-queue",
            &[
                ("public", arg(i18n.int(public.len() as i64))),
                (
                    "private",
                    arg(i18n.int((v.works.len() - public.len()) as i64)),
                ),
            ],
        ),
    );
    for w in public.iter().take(WORKS_LISTED) {
        let key = cx
            .catalogue
            .buildings
            .get(usize::from(w.kind))
            .map_or("", |b| b.key.as_str());
        let building = i18n.tr(&format!("building-{key}"));
        let place = cx
            .world
            .and_then(|world| world.states.get(usize::from(w.state).checked_sub(1)?))
            .map(|s| s.def.name.clone())
            .unwrap_or_default();
        let label = if place.is_empty() {
            building
        } else {
            format!("{building} · {place}")
        };
        let done = if w.points > 0.0 {
            (w.points - w.left) / w.points
        } else {
            1.0
        };
        let value = pct(i18n, done * 100.0, 0);
        let tip = i18n.tr_args(
            "gov-tip-work",
            &[
                ("done", arg(i18n.int((w.points - w.left).round() as i64))),
                ("points", arg(i18n.int(w.points.round() as i64))),
            ],
        );
        Account::new(&label, &value, &tip).share(done).show(ui);
    }
    if public.len() > WORKS_LISTED {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "gov-queue-more",
                &[("count", arg(i18n.int((public.len() - WORKS_LISTED) as i64)))],
            ),
        );
    }
    bf_ui_kit::gap(ui, space::MD);

    // Migration law: who may move to the cities.
    bf_ui_kit::heading(ui, &i18n.tr("gov-migration"));
    let mut law = cx.drafts.migration.unwrap_or(v.laws.migration);
    let options = [
        (MigrationLaw::Open, i18n.tr("gov-migration-open")),
        (
            MigrationLaw::Restricted,
            i18n.tr("gov-migration-restricted"),
        ),
    ];
    let tips = [
        Tip::Item('Y', "gov.migration.open".into(), "open"),
        Tip::Item('Y', "gov.migration.restricted".into(), "restricted"),
    ];
    if keys::segmented(ui, &mut law, &options, &tips) {
        cx.commands.push(GameCommand::SetMigrationLaw {
            tag: cx.tag.to_owned(),
            law,
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.migration = Some(law);
    }
    bf_ui_kit::small(
        ui,
        &i18n.tr(match law {
            MigrationLaw::Open => "gov-migration-open-tip",
            MigrationLaw::Restricted => "gov-migration-restricted-tip",
        }),
    );
}

/// Says that changes made while paused apply when the game runs.
pub fn pending_note(ui: &mut Ui, cx: &TabContext<'_>) {
    let d = &cx.drafts;
    let pending = d.tax.is_some()
        || d.priority.is_some()
        || d.migration.is_some()
        || d.rate.is_some()
        || d.bank.is_some();
    if pending && cx.snap.paused {
        bf_ui_kit::small(ui, &cx.i18n.tr("gov-pending"));
    }
}
