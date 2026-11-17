//! Government tab: the armed forces. What is under arms by unit type, with its equipment and
//! training, the reserve and what it costs each week. Reading only: mobilizing is a decision of
//! the journal's agendas, with a confirmation step.

use bf_i18n::{Arg, I18n};
use bf_sim::game::GameCommand;
use bf_sim::war::Domain;
use bf_ui_kit::tokens::space;
use bf_ui_kit::{Button, ButtonSize};
use egui::Ui;

use super::budget::{WEEKS, of_gdp, pct};
use super::government::TabContext;
use crate::forces_view::ForcesView;
use crate::keys;
use crate::text::money;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

fn domain_key(d: Domain) -> &'static str {
    match d {
        Domain::Land => "forces-domain-land",
        Domain::Sea => "forces-domain-sea",
        Domain::Air => "forces-domain-air",
        Domain::Strategic => "forces-domain-strategic",
    }
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(f) = cx.snap.forces.as_deref() else {
        bf_ui_kit::muted(ui, &i18n.tr("forces-none"));
        return;
    };
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &i18n.tr("forces-basis"));
    bf_ui_kit::gap(ui, space::SM);
    summary(ui, i18n, f, cx.view.map_or(0.0, |v| v.gdp_last_year));
    bf_ui_kit::gap(ui, space::MD);
    table(ui, i18n, f);
    bf_ui_kit::gap(ui, space::MD);
    orders(ui, cx);
    bf_ui_kit::gap(ui, space::MD);
    bf_ui_kit::small(ui, &i18n.tr("forces-decision-hint"));
}

fn summary(ui: &mut Ui, i18n: &I18n, f: &ForcesView, gdp: f64) {
    bf_ui_kit::heading(ui, &i18n.tr("forces-summary"));
    let n = |x: i64| i18n.int(x);
    let lines = [
        ("forces-men", n(f.men)),
        ("forces-reserve", n(f.reserve)),
        (
            "forces-share",
            if f.peace_share > 0.0 {
                pct(i18n, f.peace_share * 100.0, 1)
            } else {
                "—".into()
            },
        ),
        (
            "forces-goods",
            i18n.tr_args(
                "forces-goods-value",
                &[
                    ("week", arg(money(i18n, f.weekly_goods))),
                    ("gdp", arg(of_gdp(i18n, f.weekly_goods * WEEKS, gdp))),
                ],
            ),
        ),
        ("forces-depot", money(i18n, f.depot)),
    ];
    // The values wrap: on a narrow window the goods' line is longer than its column.
    let half = ui.available_width() / 2.0;
    egui::Grid::new("forces-summary")
        .num_columns(2)
        .max_col_width(half)
        .show(ui, |ui| {
            for (key, value) in lines {
                ui.add(egui::Label::new(i18n.tr(key)).wrap());
                ui.add(egui::Label::new(value).wrap());
                ui.end_row();
            }
        });
    let state = if f.mobilized {
        i18n.tr_args(
            "forces-mobilizing",
            &[
                ("reported", arg(n(f.called_up))),
                ("total", arg(n(f.called_up + f.reserve))),
            ],
        )
    } else {
        i18n.tr("forces-peace")
    };
    bf_ui_kit::small(ui, &state);
}

fn table(ui: &mut Ui, i18n: &I18n, f: &ForcesView) {
    bf_ui_kit::heading(ui, &i18n.tr("forces-units"));
    egui::Grid::new("forces-units")
        .num_columns(7)
        .striped(true)
        .show(ui, |ui| {
            for key in [
                "forces-col-unit",
                "forces-col-domain",
                "forces-col-generation",
                "forces-col-size",
                "forces-col-men",
                "forces-col-equipment",
                "forces-col-training",
            ] {
                ui.strong(i18n.tr(key));
            }
            ui.end_row();
            for r in &f.rows {
                let mut name = i18n.tr(&format!("unit-{}", r.unit));
                if r.called_up {
                    name = i18n.tr_args("forces-called-up", &[("unit", arg(name))]);
                }
                ui.label(name);
                ui.label(i18n.tr(domain_key(r.domain)));
                ui.label(i18n.int(i64::from(r.generation)));
                ui.label(i18n.decimal(r.elements, 1));
                ui.label(i18n.int(r.men));
                ui.label(pct(i18n, r.equipment * 100.0, 0));
                ui.label(pct(i18n, r.training * 100.0, 0));
                ui.end_row();
            }
        });
}

/// The order being made in the forces tab, and the step to confirm a large one or a cancellation.
#[derive(Clone, Debug, Default)]
pub struct OrdersUi {
    /// Unit key picked.
    pub unit: Option<String>,
    /// Elements to order (0 reads as 1).
    pub count: u32,
    pub colonial: bool,
    /// A large order waits for its second click.
    pub confirming: bool,
    /// The queue index whose cancellation waits for its second click.
    pub cancel: Option<u32>,
}

/// Most elements one order takes.
const MAX_COUNT: u32 = 200;

/// The queue of orders and the order to make, below the table.
pub fn orders(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(view) = cx.snap.forces.as_deref().and_then(|f| f.orders.as_ref()) else {
        return;
    };
    bf_ui_kit::heading(ui, &i18n.tr("orders-title"));
    bf_ui_kit::small(ui, &i18n.tr("orders-rule"));
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "orders-room",
            &[("room", arg(money(i18n, view.spare_week.max(0.0))))],
        ),
    );
    queue(ui, i18n, view, cx.orders, cx.tag, cx.commands);
    bf_ui_kit::gap(ui, space::MD);
    new_order(ui, i18n, view, cx.orders, cx.tag, cx.commands);
}

fn unit_name(i18n: &I18n, unit: &str) -> String {
    i18n.tr(&format!("unit-{unit}"))
}

/// What `elements` of a type are, in men, ships or aircraft.
fn amount(i18n: &I18n, domain: Domain, elements: f64, men_each: u32) -> String {
    let n = |x: f64| i18n.int(x.round() as i64);
    match domain {
        Domain::Land => i18n.tr_args(
            "orders-amount-men",
            &[("men", arg(n(elements * f64::from(men_each))))],
        ),
        Domain::Sea => i18n.tr_args(
            "orders-amount-ships",
            &[("n", arg(n(elements))), ("count", Arg::Num(elements))],
        ),
        Domain::Air => i18n.tr_args(
            "orders-amount-aircraft",
            &[("n", arg(n(elements))), ("count", Arg::Num(elements))],
        ),
        Domain::Strategic => i18n.tr_args(
            "orders-amount-weapons",
            &[("n", arg(n(elements))), ("count", Arg::Num(elements))],
        ),
    }
}

fn queue(
    ui: &mut Ui,
    i18n: &I18n,
    view: &crate::orders_view::OrdersView,
    st: &mut OrdersUi,
    tag: &str,
    commands: &mut Vec<GameCommand>,
) {
    use crate::orders_view::Progress;
    if view.rows.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("orders-none"));
        return;
    }
    let width = ui.available_width();
    for r in &view.rows {
        ui.horizontal_wrapped(|ui| {
            ui.set_max_width(width);
            let mut name = unit_name(i18n, &r.unit);
            if let Some(g) = r.generation {
                name = i18n.tr_args(
                    "orders-generation",
                    &[("unit", arg(name)), ("generation", arg(g.to_string()))],
                );
            }
            if r.colonial {
                name = i18n.tr_args("orders-colonial-unit", &[("unit", arg(name))]);
            }
            ui.strong(name);
            ui.label(i18n.tr_args(
                "orders-delivered",
                &[
                    ("done", arg(i18n.decimal(r.delivered, 0))),
                    ("ordered", arg(i18n.decimal(r.ordered, 0))),
                ],
            ));
        });
        let (share, line) = match r.progress {
            Progress::Raising { reported } => (
                reported,
                i18n.tr_args(
                    "orders-raising",
                    &[("share", arg(pct(i18n, reported * 100.0, 0)))],
                ),
            ),
            Progress::Building { built } => (
                built,
                i18n.tr_args(
                    "orders-building",
                    &[("share", arg(pct(i18n, built * 100.0, 0)))],
                ),
            ),
        };
        super::journal::bar(
            ui,
            share as f32,
            bf_ui_kit::tokens::color::ACCENT,
            width.min(360.0),
        );
        let mut parts = vec![line];
        if let Some(w) = r.weeks {
            parts.push(i18n.tr_args("orders-weeks", &[("weeks", arg(w.to_string()))]));
        }
        bf_ui_kit::small(ui, &parts.join(" · "));
        if r.waiting_weeks > 0 {
            warn(
                ui,
                &i18n.tr_args(
                    "orders-waiting",
                    &[
                        ("weeks", arg(r.waiting_weeks.to_string())),
                        ("count", Arg::Num(f64::from(r.waiting_weeks))),
                    ],
                ),
            );
        }
        ui.horizontal(|ui| {
            let id = format!("orders.cancel.{}", r.index);
            if st.cancel == Some(r.index) {
                bf_ui_kit::small(ui, &i18n.tr("orders-cancel-explain"));
                let yes = Button::danger(&i18n.tr("orders-cancel-yes"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if yes.clicked() || keys::single(ui, "orders.cancel.yes", 'Y', yes.rect) {
                    commands.push(GameCommand::CancelOrder {
                        tag: tag.to_owned(),
                        index: r.index,
                    });
                    st.cancel = None;
                }
                let no = Button::ghost(&i18n.tr("orders-keep"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if no.clicked() || keys::single(ui, "orders.cancel.no", 'N', no.rect) {
                    st.cancel = None;
                }
            } else {
                let b = Button::ghost(&i18n.tr("orders-cancel"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if b.clicked() || keys::item(ui, 'C', id, &(r.index + 1).to_string(), b.rect) {
                    st.cancel = Some(r.index);
                }
            }
        });
        bf_ui_kit::gap(ui, space::XS);
    }
}

fn warn(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                .color(bf_ui_kit::tokens::color::DANGER),
        )
        .wrap(),
    );
}

fn new_order(
    ui: &mut Ui,
    i18n: &I18n,
    view: &crate::orders_view::OrdersView,
    st: &mut OrdersUi,
    tag: &str,
    commands: &mut Vec<GameCommand>,
) {
    bf_ui_kit::heading(ui, &i18n.tr("orders-new"));
    if view.orderable.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("orders-nothing"));
        return;
    }
    // The type, grouped by domain.
    let mut k = 0usize;
    for domain in [Domain::Land, Domain::Sea, Domain::Air, Domain::Strategic] {
        let of: Vec<&crate::orders_view::Orderable> = view
            .orderable
            .iter()
            .filter(|o| o.domain == domain)
            .collect();
        if of.is_empty() {
            continue;
        }
        bf_ui_kit::caption(ui, &i18n.tr(domain_key(domain)));
        ui.horizontal_wrapped(|ui| {
            for o in of {
                k += 1;
                let picked = st.unit.as_deref() == Some(o.unit.as_str());
                let label = unit_name(i18n, &o.unit);
                let b = if picked {
                    Button::primary(&label)
                } else {
                    Button::secondary(&label)
                }
                .size(ButtonSize::Small)
                .show(ui);
                if b.clicked()
                    || keys::item(
                        ui,
                        'U',
                        format!("orders.unit.{}", o.unit),
                        &k.to_string(),
                        b.rect,
                    )
                {
                    st.unit = Some(o.unit.clone());
                    st.confirming = false;
                }
            }
        });
    }
    let Some(o) = st
        .unit
        .as_deref()
        .and_then(|u| view.orderable.iter().find(|o| o.unit == u))
    else {
        bf_ui_kit::small(ui, &i18n.tr("orders-pick"));
        return;
    };
    let count = st.count.max(1);
    bf_ui_kit::gap(ui, space::SM);
    ui.horizontal(|ui| {
        ui.label(i18n.tr("orders-count"));
        let d = super::gov_ui::stepper(
            ui,
            &count.to_string(),
            &i18n.tr("orders-count-tip"),
            count > 1,
            count < MAX_COUNT,
            super::gov_ui::StepTips {
                family: 'Q',
                id: "orders.count",
                row: 0,
            },
        );
        if d != 0 {
            st.count = (count as i32 + d).clamp(1, MAX_COUNT as i32) as u32;
            st.confirming = false;
        }
        ui.label(amount(i18n, o.domain, f64::from(count), o.men));
    });
    if view.colonies {
        ui.horizontal(|ui| {
            if bf_ui_kit::toggle(ui, &mut st.colonial).changed() {
                st.confirming = false;
            }
            ui.label(i18n.tr("orders-colonial"));
        });
    }
    // What it costs: to build, and every year after.
    let build = o.build_cost * f64::from(count);
    let yearly = o.yearly_cost() * f64::from(count);
    bf_ui_kit::stat_row(ui, &i18n.tr("orders-cost-build"), &money(i18n, build));
    let share = if view.military_year > 0.0 {
        i18n.tr_args(
            "orders-cost-yearly-share",
            &[
                ("money", arg(money(i18n, yearly))),
                (
                    "share",
                    arg(pct(i18n, yearly / view.military_year * 100.0, 1)),
                ),
            ],
        )
    } else {
        money(i18n, yearly)
    };
    bf_ui_kit::stat_row(ui, &i18n.tr("orders-cost-yearly"), &share);
    let lead = match o.training_weeks {
        Some(t) => i18n.tr_args(
            "orders-lead-land",
            &[
                ("weeks", arg(o.weeks.to_string())),
                ("training", arg(t.to_string())),
            ],
        ),
        None => i18n.tr_args("orders-lead-build", &[("weeks", arg(o.weeks.to_string()))]),
    };
    bf_ui_kit::stat_row(ui, &i18n.tr("orders-lead"), &lead);
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "orders-yearly-parts",
            &[
                ("pay", arg(money(i18n, o.pay_year * f64::from(count)))),
                ("upkeep", arg(money(i18n, o.upkeep_year * f64::from(count)))),
            ],
        ),
    );
    let large = crate::orders_view::is_large(o, count, view.military_year);
    let order = GameCommand::OrderForces {
        tag: tag.to_owned(),
        unit: o.unit.clone(),
        elements: count,
        colonial: view.colonies && st.colonial,
    };
    if st.confirming && large {
        warn(
            ui,
            &i18n.tr_args(
                "orders-confirm",
                &[
                    (
                        "amount",
                        arg(amount(i18n, o.domain, f64::from(count), o.men)),
                    ),
                    ("unit", arg(unit_name(i18n, &o.unit))),
                    ("build", arg(money(i18n, build))),
                    ("yearly", arg(share.clone())),
                ],
            ),
        );
        ui.horizontal(|ui| {
            let yes = Button::primary(&i18n.tr("orders-confirm-yes"))
                .size(ButtonSize::Medium)
                .show(ui);
            if yes.clicked() || keys::single(ui, "orders.yes", 'Y', yes.rect) {
                commands.push(order.clone());
                st.confirming = false;
            }
            let no = Button::ghost(&i18n.tr("orders-confirm-no"))
                .size(ButtonSize::Medium)
                .show(ui);
            if no.clicked() || keys::single(ui, "orders.no", 'N', no.rect) {
                st.confirming = false;
            }
        });
        return;
    }
    let b = Button::primary(&i18n.tr("orders-place"))
        .size(ButtonSize::Medium)
        .show(ui);
    if b.clicked() || keys::single(ui, "orders.place", 'O', b.rect) {
        if large {
            st.confirming = true;
        } else {
            commands.push(order);
        }
    }
}
