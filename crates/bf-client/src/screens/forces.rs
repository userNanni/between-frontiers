//! Government tab: the armed forces. What is under arms by unit type, with its equipment and
//! training, the reserve and what it costs each week; the orders of new forces, and disbanding
//! some of a type, with what goes back to the depot shown before the confirming click.
//! Mobilizing is a decision of the journal's agendas, with a confirmation step.

use bf_i18n::{Arg, I18n};
use bf_sim::game::GameCommand;
use bf_sim::war::Domain;
use bf_ui_kit::tokens::{color, space};
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
    if let Some(a) = &f.army {
        bf_ui_kit::gap(ui, space::MD);
        army(ui, i18n, a);
    }
    bf_ui_kit::gap(ui, space::MD);
    table(ui, i18n, f, cx.orders);
    disband(ui, cx);
    bf_ui_kit::gap(ui, space::MD);
    orders(ui, cx);
    bf_ui_kit::gap(ui, space::MD);
    composition(ui, cx);
    bf_ui_kit::gap(ui, space::MD);
    programme(ui, cx);
    bf_ui_kit::gap(ui, space::MD);
    bf_ui_kit::small(ui, &i18n.tr("forces-decision-hint"));
}

/// The nuclear programme (N1): its share of GDP a year (a stepper of one per mille, or start and
/// end), what it costs and spent, the chain step by step with the months left at the month's
/// credit, and the stock: warheads, plants and the fissile material (`GameCommand::
/// NuclearProgramme`).
fn programme(ui: &mut Ui, cx: &mut TabContext<'_>) {
    use bf_sim::research::Stage;
    let i18n = cx.i18n;
    let Some(p) = cx.snap.forces.as_deref().and_then(|f| f.programme.as_ref()) else {
        return;
    };
    bf_ui_kit::heading(ui, &i18n.tr("programme-title"));
    bf_ui_kit::small(ui, &i18n.tr("programme-basis"));
    let tag = cx.tag.to_owned();
    let pm = |x: u16| pct(i18n, f64::from(x) / 10.0, 1);
    if p.share == 0 {
        let b = Button::secondary(&i18n.tr("programme-start"))
            .size(ButtonSize::Small)
            .show(ui);
        if b.clicked() || keys::item(ui, 'U', "forces.programme.start".into(), "s", b.rect) {
            cx.commands.push(GameCommand::NuclearProgramme {
                tag,
                share: Some(1),
            });
        }
        return;
    }
    let d = super::gov_ui::control_row(ui, &i18n.tr("programme-share"), |ui| {
        super::gov_ui::stepper_typed(
            ui,
            &pm(p.share),
            super::gov_ui::STEP_VALUE_W,
            &i18n.tr("programme-step-tip"),
            p.share > 1,
            p.share < p.max_share,
            super::gov_ui::StepTips {
                family: 'U',
                id: "forces.programme.share",
                row: 0,
            },
            super::gov_ui::typed_pct(
                i18n,
                f64::from(p.share) / 10.0,
                0.1,
                f64::from(p.max_share) / 10.0,
                1,
            ),
        )
    });
    if let Some(share) =
        super::gov_ui::step_to(d, i32::from(p.share), 1, 10.0, 1, i32::from(p.max_share))
    {
        let share = share as u16;
        cx.commands.push(GameCommand::NuclearProgramme {
            tag: tag.clone(),
            share: Some(share),
        });
    }
    bf_ui_kit::stat_row(ui, &i18n.tr("programme-cost"), &money(i18n, p.money_year));
    bf_ui_kit::stat_row(ui, &i18n.tr("programme-spent"), &money(i18n, p.spent));
    for (key, stage, progress) in &p.steps {
        let st = match stage {
            Stage::Unknown => "programme-stage-unknown",
            Stage::Science => "programme-stage-science",
            Stage::Prototype => "programme-stage-prototype",
            Stage::Industrial => "programme-stage-industrial",
            Stage::Diffused => "programme-stage-diffused",
        };
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr(&format!("tech-{key}")),
            &i18n.tr_args(st, &[("progress", arg(pct(i18n, progress * 100.0, 0)))]),
        );
    }
    let months = match p.months_left {
        Some(mo) => i18n.tr_args("programme-months", &[("months", arg(i18n.decimal(mo, 0)))]),
        None => i18n.tr("programme-months-none"),
    };
    bf_ui_kit::small(ui, &months);
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("programme-warheads"),
        &i18n.tr_args(
            "programme-warheads-value",
            &[
                ("n", arg(i18n.decimal(p.warheads, 0))),
                ("ordered", arg(i18n.decimal(p.ordered, 0))),
            ],
        ),
    );
    // The core plants by method: levels and the cores a year.
    if p.core_plants.is_empty() {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr("programme-core-plants"),
            &i18n.tr("programme-none"),
        );
    }
    for c in &p.core_plants {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr(&format!("method-{}", c.method)),
            &i18n.tr_args(
                "programme-core-plant-value",
                &[
                    ("levels", bf_i18n::Arg::Num(c.levels as f64)),
                    ("cores", arg(i18n.decimal(c.cores_year, 1))),
                ],
            ),
        );
    }
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("programme-fissile"),
        &i18n.tr_args(
            "programme-fissile-value",
            &[
                ("price", arg(money(i18n, p.fissile_price))),
                ("delivered", arg(pct(i18n, p.fissile_delivered * 100.0, 0))),
            ],
        ),
    );
    // Where the ore comes from: the country's mines, the market's stock, the market.
    if let Some(o) = &p.ore {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr("programme-ore"),
            &i18n.tr_args(
                "programme-ore-value",
                &[
                    ("mined", arg(i18n.compact(o.mined_year))),
                    ("used", arg(i18n.compact(o.used_year))),
                ],
            ),
        );
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "programme-ore-market",
                &[
                    ("stock", arg(i18n.compact(o.stock))),
                    ("price", arg(money(i18n, o.price))),
                    ("delivered", arg(pct(i18n, o.delivered * 100.0, 0))),
                ],
            ),
        );
    }
    // N3: the test makes the arsenal known to all; it needs a warhead. The test and the end
    // are rare decisions: a second click that says what changes, and a restore point before
    // (written where the commands are sent, `screens/game.rs`), as in the Agendas.
    let st = &mut *cx.orders;
    if p.tested {
        bf_ui_kit::small(ui, &i18n.tr("programme-tested"));
    } else if p.test_asked {
        bf_ui_kit::small(ui, &i18n.tr("programme-test-asked"));
    } else if p.warheads >= 1.0 && st.programme_confirm != Some(false) {
        if st.programme_confirm == Some(true) {
            match confirm_step(
                ui,
                i18n,
                "journal-programme-test-explain",
                "journal-programme-test-confirm",
                "forces.programme.test",
            ) {
                Some(true) => {
                    cx.commands
                        .push(GameCommand::NuclearTest { tag: tag.clone() });
                    st.programme_confirm = None;
                }
                Some(false) => st.programme_confirm = None,
                None => {}
            }
        } else {
            let t = Button::secondary(&i18n.tr("journal-programme-test"))
                .size(ButtonSize::Small)
                .show(ui);
            if t.clicked() || keys::item(ui, 'U', "forces.programme.test".into(), "t", t.rect) {
                st.programme_confirm = Some(true);
            }
            bf_ui_kit::small(ui, &i18n.tr("programme-test-note"));
        }
    }
    if st.programme_confirm == Some(false) {
        match confirm_step(
            ui,
            i18n,
            "journal-programme-end-explain",
            "journal-programme-end-confirm",
            "forces.programme.end",
        ) {
            Some(true) => {
                cx.commands
                    .push(GameCommand::NuclearProgramme { tag, share: None });
                st.programme_confirm = None;
            }
            Some(false) => st.programme_confirm = None,
            None => {}
        }
    } else if st.programme_confirm.is_none() {
        let end = Button::ghost(&i18n.tr("journal-programme-end"))
            .size(ButtonSize::Small)
            .show(ui);
        if end.clicked() || keys::item(ui, 'U', "forces.programme.end".into(), "e", end.rect) {
            st.programme_confirm = Some(false);
        }
    }
    bf_ui_kit::small(ui, &i18n.tr("programme-note"));
}

/// The second step of a rare decision: what changes, in red, then the confirming button (Y)
/// and the way back (N) side by side. `Some(true)` confirmed, `Some(false)` kept.
fn confirm_step(ui: &mut Ui, i18n: &I18n, explain: &str, sure: &str, id: &str) -> Option<bool> {
    ui.add(
        egui::Label::new(
            egui::RichText::new(i18n.tr(explain))
                .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                .color(color::DANGER),
        )
        .wrap(),
    );
    ui.horizontal(|ui| {
        let yes = Button::danger(&i18n.tr(sure))
            .size(ButtonSize::Small)
            .show(ui);
        if yes.clicked() || keys::single(ui, &format!("{id}.yes"), 'Y', yes.rect) {
            return Some(true);
        }
        let no = Button::ghost(&i18n.tr("forces-keep"))
            .size(ButtonSize::Small)
            .show(ui);
        (no.clicked() || keys::single(ui, &format!("{id}.no"), 'N', no.rect)).then_some(false)
    })
    .inner
}

/// The composition of the land forces: on wheels and on tracks now, the target (the historical
/// rule's or the government's), the infantry in conversion, and the choice between the rule
/// and a target the government fixes (`GameCommand::SetComposition`).
/// Per-mille steps of the composition's targets: one percentage point.
const COMPOSITION_STEP: i32 = 10;

/// The composition of the forces, as the budget's levers: today's shares, the target by the
/// historical rule or fixed by the government (two steppers, wheels and tracks, each sent as it
/// changes), and the conversions under way with what holds a stalled one, in words.
fn composition(ui: &mut Ui, cx: &mut TabContext<'_>) {
    use crate::forces_view::Stall;
    let i18n = cx.i18n;
    let Some(cv) = cx
        .snap
        .forces
        .as_deref()
        .and_then(|f| f.composition.as_ref())
    else {
        return;
    };
    let p = |x: f64| pct(i18n, x * 100.0, 1);
    let pm = |x: u32| pct(i18n, f64::from(x) / 10.0, 0);
    bf_ui_kit::heading(ui, &i18n.tr("composition-title"));
    bf_ui_kit::small(ui, &i18n.tr("composition-basis"));
    bf_ui_kit::stat_row(ui, &i18n.tr("composition-wheels-now"), &p(cv.wheels));
    bf_ui_kit::stat_row(ui, &i18n.tr("composition-tracks-now"), &p(cv.tracks));
    let Some((rule_w, rule_t)) = cv.rule_target else {
        bf_ui_kit::small(ui, &i18n.tr("composition-target-none"));
        return;
    };
    // What the player set wins over the view until the tick applies it.
    let st = &mut *cx.orders;
    if st.composition.is_some() && st.composition == cv.fixed {
        st.composition = None;
    }
    let shown = st.composition.or(cv.fixed);
    let mut fixed = shown.is_some();
    bf_ui_kit::gap(ui, space::SM);
    let (changed, rects) = bf_ui_kit::lever_header(
        ui,
        &i18n.tr("composition-target"),
        &mut fixed,
        &i18n.tr("lever-rule"),
        &i18n.tr("lever-fixed"),
    );
    let mut typed = false;
    for (r, (key, on)) in rects.into_iter().zip([("rule", false), ("fixed", true)]) {
        if keys::item(ui, 'K', format!("forces.composition.{key}"), key, r) && fixed != on {
            fixed = on;
            typed = true;
        }
    }
    let tag = cx.tag.to_owned();
    let mut send: Option<Option<[u16; 2]>> = None;
    if changed || typed {
        send = Some(fixed.then_some([rule_w as u16, rule_t as u16]));
    }
    bf_ui_kit::lever_rule_line(
        ui,
        &i18n.tr_args(
            "composition-rule-line",
            &[("wheels", arg(pm(rule_w))), ("tracks", arg(pm(rule_t)))],
        ),
        fixed.then(|| i18n.tr("composition-fixed-note")).as_deref(),
    );
    if fixed && let Some([w, t]) = send.flatten().or(shown) {
        let mut now = [w, t];
        for (k, key) in ["composition-wheels", "composition-tracks"]
            .into_iter()
            .enumerate()
        {
            let other = now[1 - k];
            let room = 1000u16.saturating_sub(other);
            let d = super::gov_ui::control_row(ui, &i18n.tr(key), |ui| {
                super::gov_ui::stepper_typed(
                    ui,
                    &pm(u32::from(now[k])),
                    super::gov_ui::STEP_VALUE_W,
                    &i18n.tr("composition-step-tip"),
                    now[k] > 0,
                    now[k] < room,
                    super::gov_ui::StepTips {
                        family: 'K',
                        id: &format!("forces.composition.{k}"),
                        row: k,
                    },
                    super::gov_ui::typed_pct(
                        i18n,
                        f64::from(now[k]) / 10.0,
                        0.0,
                        f64::from(room) / 10.0,
                        0,
                    ),
                )
            });
            if let Some(to) = super::gov_ui::step_to(
                d,
                i32::from(now[k]),
                COMPOSITION_STEP,
                10.0,
                0,
                i32::from(room),
            ) {
                now[k] = to as u16;
                send = Some(Some(now));
            }
        }
    }
    if let Some(c) = send {
        st.composition = c;
        cx.commands.push(GameCommand::SetComposition {
            tag,
            composition: c,
        });
    }
    if !cv.wheels_type {
        bf_ui_kit::small(ui, &i18n.tr("composition-no-wheels"));
    }
    if !cv.tracks_type {
        bf_ui_kit::small(ui, &i18n.tr("composition-no-tracks"));
    }
    for x in &cv.converting {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr_args(
                "composition-converting",
                &[("unit", arg(unit_name(i18n, &x.unit)))],
            ),
            &i18n.tr_args(
                "composition-converting-done",
                &[
                    ("done", arg(i18n.decimal(x.delivered, 1))),
                    ("of", arg(i18n.decimal(x.ordered, 1))),
                ],
            ),
        );
        if let Some(why) = x.why {
            let key = match why {
                Stall::NoMaker => "composition-stall-maker",
                Stall::NoRoom => "composition-stall-room",
                Stall::NoInfantry => "composition-stall-infantry",
                Stall::Prices => "composition-stall-prices",
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(i18n.tr_args(
                        key,
                        &[
                            ("weeks", arg(i18n.int(i64::from(x.stalled)))),
                            ("count", Arg::Num(f64::from(x.stalled))),
                        ],
                    ))
                    .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                    .color(color::ACCENT),
                )
                .wrap(),
            );
        }
    }
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

fn table(ui: &mut Ui, i18n: &I18n, f: &ForcesView, st: &mut OrdersUi) {
    bf_ui_kit::heading(ui, &i18n.tr("forces-units"));
    egui::Grid::new("forces-units")
        .num_columns(8)
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
            ui.label("");
            ui.end_row();
            for (i, r) in f.rows.iter().enumerate() {
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
                if f.disbandable(i) > 0 {
                    let open = st.disband.as_ref().is_some_and(|d| d.row == i);
                    let label = i18n.tr("forces-disband");
                    let b = if open {
                        Button::secondary(&label)
                    } else {
                        Button::ghost(&label)
                    }
                    .size(ButtonSize::Small)
                    .show(ui);
                    let id = format!("forces.disband.{}.{}", r.unit, r.generation);
                    if b.clicked() || keys::item(ui, 'D', id, &(i + 1).to_string(), b.rect) {
                        st.disband = (!open).then(|| DisbandUi {
                            row: i,
                            unit: r.unit.clone(),
                            generation: r.generation,
                            count: 1,
                        });
                    }
                } else {
                    ui.label("");
                }
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
    /// The type whose disband is being weighed.
    pub disband: Option<DisbandUi>,
    /// The nuclear programme's rare decision waiting for its second click: `true` the test,
    /// `false` the end.
    pub programme_confirm: Option<bool>,
    /// The forces as they stood when a disband was sent: the step waits for the simulation to
    /// carry it out before taking another (the formations' indices move).
    pub disband_sent: Option<crate::forces_view::Stock>,
    /// The composition being set: wheels and tracks per mille, while "fixed by the government"
    /// is being edited.
    pub composition: Option<[u16; 2]>,
}

/// The disband step of one row of the forces' table.
#[derive(Clone, Debug, PartialEq)]
pub struct DisbandUi {
    pub row: usize,
    /// The row's type and generation, to find it again when the table is read anew.
    pub unit: String,
    pub generation: u8,
    /// Elements to disband.
    pub count: u32,
}

/// The disband step, below the table: how many elements, what goes to the depot and where the
/// men go, then the confirming click. Sends one `DisbandForces` per formation touched.
fn disband(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(f) = cx.snap.forces.as_deref() else {
        return;
    };
    let st = &mut *cx.orders;
    if st.disband_sent.is_some_and(|s| s != f.stock) {
        st.disband_sent = None;
    }
    let Some(mut d) = st.disband.clone() else {
        return;
    };
    // The table read anew may have moved the row, or the type may be gone.
    match f
        .rows
        .iter()
        .position(|r| r.unit == d.unit && r.generation == d.generation && !r.called_up)
    {
        Some(row) => d.row = row,
        None => {
            st.disband = None;
            return;
        }
    }
    let max = f.disbandable(d.row);
    if max == 0 {
        st.disband = None;
        return;
    }
    d.count = d.count.clamp(1, max);
    let Some(plan) = f.disband(d.row, d.count) else {
        st.disband = None;
        return;
    };
    let r = &f.rows[d.row];
    let name = unit_name(i18n, &r.unit);
    let waiting = st.disband_sent.is_some();
    let (mut close, mut sent) = (false, false);
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::card_frame().show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        bf_ui_kit::heading(
            ui,
            &i18n.tr_args("forces-disband-title", &[("unit", arg(name.clone()))]),
        );
        ui.horizontal(|ui| {
            ui.label(i18n.tr("forces-disband-count"));
            let step = super::gov_ui::stepper_typed(
                ui,
                &d.count.to_string(),
                super::gov_ui::STEP_VALUE_W,
                &i18n.tr("forces-disband-count-tip"),
                d.count > 1,
                d.count < max,
                super::gov_ui::StepTips {
                    family: 'Q',
                    id: "forces.disband.count",
                    row: 1,
                },
                super::gov_ui::typed_int(i18n, d.count.into(), 1, max.into()),
            );
            if let Some(to) = super::gov_ui::step_to(step, d.count as i32, 1, 1.0, 1, max as i32) {
                d.count = to as u32;
            }
            ui.label(amount(i18n, r.domain, plan.elements, men_each(f, d.row)));
        });
        // Where the men go.
        if plan.to_reserve > 0 {
            bf_ui_kit::stat_row(
                ui,
                &i18n.tr("forces-disband-reserve"),
                &i18n.int(plan.to_reserve),
            );
        }
        if plan.home > 0 {
            bf_ui_kit::stat_row(ui, &i18n.tr("forces-disband-home"), &i18n.int(plan.home));
        }
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr("forces-disband-depot"),
            &money(i18n, plan.value),
        );
        if plan.goods.is_empty() {
            bf_ui_kit::small(ui, &i18n.tr("forces-disband-no-goods"));
        } else {
            let shown: Vec<String> = plan
                .goods
                .iter()
                .take(GOODS_SHOWN)
                .map(|(g, _, v)| {
                    i18n.tr_args(
                        "forces-disband-good",
                        &[
                            ("good", arg(good_name(i18n, cx.catalogue, *g))),
                            ("value", arg(money(i18n, *v))),
                        ],
                    )
                })
                .collect();
            let mut line = shown.join(" · ");
            if plan.goods.len() > GOODS_SHOWN {
                line = i18n.tr_args(
                    "forces-disband-goods-more",
                    &[
                        ("goods", arg(line)),
                        ("n", arg(i18n.int((plan.goods.len() - GOODS_SHOWN) as i64))),
                        ("count", Arg::Num((plan.goods.len() - GOODS_SHOWN) as f64)),
                    ],
                );
            }
            ui.add(egui::Label::new(line).wrap());
        }
        let note = match r.domain {
            Domain::Land if plan.home > 0 => "forces-disband-note-peace",
            Domain::Land => "forces-disband-note-raised",
            _ => "forces-disband-note-service",
        };
        bf_ui_kit::small(ui, &i18n.tr(note));
        if waiting {
            bf_ui_kit::small(ui, &i18n.tr("forces-disband-waiting"));
        }
        ui.horizontal(|ui| {
            let yes = Button::danger(&i18n.tr("forces-disband-yes"))
                .size(ButtonSize::Medium)
                .enabled(!waiting)
                .show(ui);
            if (yes.clicked() || keys::single(ui, "forces.disband.yes", 'Y', yes.rect)) && !waiting
            {
                for (formation, elements) in &plan.commands {
                    cx.commands.push(GameCommand::DisbandForces {
                        tag: cx.tag.to_owned(),
                        formation: *formation,
                        elements: *elements,
                    });
                }
                sent = true;
            }
            let no = Button::ghost(&i18n.tr("forces-disband-no"))
                .size(ButtonSize::Medium)
                .show(ui);
            if no.clicked() || keys::single(ui, "forces.disband.no", 'N', no.rect) {
                close = true;
            }
        });
    });
    if sent {
        st.disband_sent = Some(f.stock);
    }
    st.disband = (!close && !sent).then_some(d);
}

/// Goods named in the disband step; the rest are counted.
const GOODS_SHOWN: usize = 4;

fn good_name(i18n: &I18n, cat: &bf_sim::econ::Catalogue, g: u16) -> String {
    cat.goods
        .get(g as usize)
        .map_or_else(String::new, |x| i18n.tr(&format!("good-{}", x.key)))
}

/// Men in one element of the row's formations.
fn men_each(f: &ForcesView, row: usize) -> u32 {
    f.formations
        .iter()
        .find(|x| x.row == row)
        .map_or(0, |x| x.men_each)
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
        if let (Some(w), Some((year, month))) = (r.weeks, r.expected) {
            parts.push(i18n.tr_args(
                "orders-weeks",
                &[
                    ("weeks", arg(w.to_string())),
                    ("month", arg(i18n.tr(&format!("month-{month}")))),
                    ("year", arg(year.to_string())),
                ],
            ));
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
        let d = super::gov_ui::stepper_typed(
            ui,
            &count.to_string(),
            super::gov_ui::STEP_VALUE_W,
            &i18n.tr("orders-count-tip"),
            count > 1,
            count < MAX_COUNT,
            super::gov_ui::StepTips {
                family: 'Q',
                id: "orders.count",
                row: 0,
            },
            super::gov_ui::typed_int(i18n, count.into(), 1, MAX_COUNT.into()),
        );
        if let Some(to) = super::gov_ui::step_to(d, count as i32, 1, 1.0, 1, MAX_COUNT as i32) {
            st.count = to as u32;
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

/// The generation and how far the readiness is from the next step, in one line.
pub fn generation_line(i18n: &I18n, a: &crate::army_view::ArmyView) -> String {
    let n = |x: u32| i18n.int(i64::from(x));
    if a.generation >= 3 {
        i18n.tr_args(
            "army-third",
            &[("total", arg(n(a.total))), ("down", arg(n(a.down)))],
        )
    } else {
        i18n.tr_args(
            "army-second",
            &[
                ("total", arg(n(a.total))),
                ("up", arg(n(a.up))),
                ("short", arg(n(a.short_of_up()))),
            ],
        )
    }
}

/// The army's generation (`docs/war.pt-BR.md`, "Geração do exército"): the readiness on a bar
/// with the two thresholds, then each term with its points, what holds it back, and what the
/// country does in peace to move it.
fn army(ui: &mut Ui, i18n: &I18n, a: &crate::army_view::ArmyView) {
    use crate::army_view::HoldView;
    bf_ui_kit::heading(ui, &i18n.tr("army-heading"));
    bf_ui_kit::small(ui, &i18n.tr("army-basis"));
    ui.add(
        egui::Label::new(
            egui::RichText::new(generation_line(i18n, a))
                .font(bf_ui_kit::theme::sans_medium(bf_ui_kit::tokens::size::BODY)),
        )
        .wrap(),
    );
    // The readiness out of 1,000, a light tick where it falls back, a gold one where it moves up.
    let width = ui.available_width().min(420.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 3.0, color::SURFACE_SUNKEN);
    let mut fill = rect;
    fill.set_width(rect.width() * (a.total.min(1000) as f32 / 1000.0));
    p.rect_filled(fill, 3.0, color::ACCENT);
    for (t, c) in [(a.down, color::TEXT_MUTED), (a.up, color::ACCENT_HOVER)] {
        let x = rect.left() + rect.width() * (t.min(1000) as f32 / 1000.0);
        p.vline(x, rect.expand(3.0).y_range(), egui::Stroke::new(1.5, c));
    }
    bf_ui_kit::gap(ui, space::XS);
    for t in &a.terms {
        let taken = t.key == "central_control";
        if taken && t.points == 0 && t.hold.is_none() {
            continue;
        }
        let value = i18n.tr_args(
            if taken {
                "army-points-taken"
            } else {
                "army-points"
            },
            &[
                ("points", arg(i18n.int(i64::from(t.points)))),
                ("most", arg(i18n.int(i64::from(t.most)))),
            ],
        );
        bf_ui_kit::stat_row(ui, &i18n.tr(&format!("army-term-{}", t.key)), &value);
        let (text, tint) = match &t.hold {
            Some(h) => {
                let args: Vec<(&str, Arg<'_>)> = match h {
                    HoldView::SchoolWithoutDoctrine(stage) => {
                        vec![("stage", arg(i18n.tr(&format!("tech-stage-{stage}"))))]
                    }
                    HoldView::RegimeChange { years_left } => vec![
                        ("years", arg(years_left.to_string())),
                        ("count", Arg::Num(f64::from(*years_left))),
                    ],
                    _ => Vec::new(),
                };
                (i18n.tr_args(h.key(), &args), color::ACCENT)
            }
            None => (i18n.tr(&format!("army-do-{}", t.key)), color::TEXT_MUTED),
        };
        ui.add(
            egui::Label::new(
                egui::RichText::new(text)
                    .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                    .color(tint),
            )
            .wrap(),
        );
    }
}
