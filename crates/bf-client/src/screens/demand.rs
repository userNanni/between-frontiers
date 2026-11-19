//! A colony's demand of independence, answered in the Agendas: the unit, its mobilization against
//! the two marks, what still holds the movement back, the metropole's account, and the four
//! answers with the price of each (computed by the simulation). Rare and hard to undo: a step to
//! confirm, and a restore point before the command goes.

use bf_i18n::{Arg, I18n};
use bf_sim::borders::independence::{Account, Answer};
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Rect, RichText, Sense, Ui, vec2};

use super::journal::Response;
use crate::demands_view::{ANSWERS, Demand, answer_key};
use crate::keys;
use crate::text::{country_name, date_long};
use crate::world::World;

/// The answer picked, and whether the player is at the confirmation step.
#[derive(Clone, Debug, Default)]
pub struct DemandUi {
    pub option: Option<usize>,
    pub confirming: bool,
    /// Bring the answer's price and the next button into view on the next frame.
    pub scroll: bool,
}

fn pct(i18n: &I18n, x: f64) -> String {
    crate::text::pct_text(i18n, i18n.decimal(x * 100.0, 2))
}

/// A state's name: the interface's or the map's, found by key; the key itself as a last resort.
fn state_name(i18n: &I18n, world: Option<&World>, key: &str) -> String {
    world
        .and_then(|w| w.states.iter().find(|s| s.def.key == key))
        .map_or_else(|| key.to_owned(), |s| crate::text::state_name(i18n, &s.def))
}

/// What a refusal brings: the revolt now, past its mark, or once mobilization reaches it.
fn revolt_line(i18n: &I18n, mobilized: u16, mark: u16) -> String {
    let key = if mobilized >= mark {
        "demand-refusal-revolt-now"
    } else {
        "demand-refusal-revolt-later"
    };
    i18n.tr_args(
        key,
        &[
            ("mobilized", Arg::String(mobilized.to_string())),
            ("mark", Arg::String(mark.to_string())),
        ],
    )
}

/// The account, term by term (shares of the metropole's GDP a year).
fn account(ui: &mut Ui, i18n: &I18n, a: &Account, payers: Option<&[String]>) {
    let f = |x: laplace_schelling::Fixed| x.to_f64_lossy();
    for (k, v) in [
        ("independence-account-value", a.value),
        ("independence-account-legitimacy", a.legitimacy),
        ("independence-account-international", a.international),
        ("independence-account-material", a.material),
    ] {
        bf_ui_kit::stat_row(ui, &i18n.tr(k), &pct(i18n, f(v)));
    }
    if let Some(tags) = payers
        && !a.top.is_empty()
    {
        let names: Vec<String> = a
            .top
            .iter()
            .map(|(c, share)| {
                let name = tags
                    .get(usize::from(*c))
                    .map_or_else(|| c.to_string(), |t| country_name(i18n, t));
                format!("{name} {}", pct(i18n, f(*share)))
            })
            .collect();
        ui.add(
            egui::Label::new(
                RichText::new(
                    i18n.tr_args("demand-payers", &[("list", Arg::String(names.join(" · ")))]),
                )
                .font(sans(size::SMALL))
                .color(color::TEXT_MUTED),
            )
            .wrap(),
        );
    }
}

/// The unit's mobilization against the two marks, as a bar from 0 to 1000 ‰.
fn mobilization(ui: &mut Ui, i18n: &I18n, d: &Demand) {
    let v = &d.view;
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 12.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, radius::SM, color::SURFACE_SUNKEN);
    let x = |permille: u16| r.left() + r.width() * f32::from(permille.min(1000)) / 1000.0;
    p.rect_filled(
        Rect::from_min_max(r.min, egui::pos2(x(v.mobilized), r.bottom())),
        radius::SM,
        color::ACCENT,
    );
    for (mark, c) in [
        (v.demand_mark, color::TEXT_MUTED),
        (v.revolt_mark, color::DANGER),
    ] {
        p.vline(x(mark), r.y_range(), egui::Stroke::new(2.0, c));
    }
    // In words first: how near a refusal is to turning into a revolt; the numbers back it.
    let key = if v.mobilized >= v.revolt_mark {
        "demand-closeness-ready"
    } else if u32::from(v.mobilized) * 2 >= u32::from(v.revolt_mark) {
        "demand-closeness-near"
    } else {
        "demand-closeness-far"
    };
    ui.add(egui::Label::new(RichText::new(i18n.tr(key)).font(sans(size::BODY))).wrap());
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "demand-closeness-number",
            &[
                ("mobilized", Arg::String(v.mobilized.to_string())),
                ("revolt", Arg::String(v.revolt_mark.to_string())),
            ],
        ),
    );
}

/// Wide enough for the situation and the answers side by side.
const TWO_COLUMNS: f32 = 680.0;

pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    tags: &[String],
    d: &Demand,
    state: &mut DemandUi,
    out: &mut Response,
) {
    let v = &d.view;
    ui.spacing_mut().item_spacing.y = space::SM;
    let unit = country_name(i18n, &v.unit);
    bf_ui_kit::title(
        ui,
        &i18n.tr_args(
            "independence-demand-title",
            &[("unit", Arg::String(unit.clone()))],
        ),
    );
    ui.add(
        egui::Label::new(
            RichText::new(i18n.tr_args(
                "independence-demand-deadline",
                &[("date", Arg::String(date_long(i18n, v.deadline)))],
            ))
            .font(sans(size::SMALL))
            .color(color::DANGER),
        )
        .wrap(),
    );
    if ui.available_width() >= TWO_COLUMNS {
        bf_ui_kit::columns(ui, 2, |cols| {
            let [left, right] = cols else { return };
            // Plain left-aligned text: the columns' own layout would justify it.
            let plain = egui::Layout::top_down(egui::Align::Min);
            left.with_layout(plain, |ui| {
                ui.spacing_mut().item_spacing.y = space::SM;
                situation(ui, i18n, world, tags, d);
            });
            right.with_layout(plain, |ui| {
                ui.spacing_mut().item_spacing.y = space::SM;
                answers(ui, i18n, d, &unit, state, out);
            });
        });
    } else {
        situation(ui, i18n, world, tags, d);
        answers(ui, i18n, d, &unit, state, out);
    }
}

/// The unit, its mobilization, what holds it back, and what holding it costs.
fn situation(ui: &mut Ui, i18n: &I18n, world: Option<&World>, tags: &[String], d: &Demand) {
    let v = &d.view;
    let states: Vec<String> = v
        .states
        .iter()
        .map(|k| state_name(i18n, world, k))
        .collect();
    ui.add(
        egui::Label::new(
            RichText::new(
                i18n.tr_args("demand-states", &[("list", Arg::String(states.join(", ")))]),
            )
            .font(sans(size::SMALL))
            .color(color::TEXT_MUTED),
        )
        .wrap(),
    );
    mobilization(ui, i18n, d);
    if let Some(w) = &v.weakest {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "independence-demand-weakest",
                &[("condition", Arg::String(i18n.tr(w)))],
            ),
        );
    }
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("demand-account"));
    account(ui, i18n, &v.account, Some(tags));
    bf_ui_kit::gap(ui, space::SM);
}

/// The four answers with the price of the one picked, then the step to confirm.
fn answers(
    ui: &mut Ui,
    i18n: &I18n,
    d: &Demand,
    unit: &str,
    state: &mut DemandUi,
    out: &mut Response,
) {
    let v = &d.view;
    if !state.confirming {
        bf_ui_kit::heading(ui, &i18n.tr("demand-answers"));
        for (k, a) in ANSWERS.iter().enumerate() {
            let picked = state.option == Some(k);
            let label = i18n.tr(answer_key(*a));
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
                    'L',
                    format!("demand.answer.{k}"),
                    &(k + 1).to_string(),
                    b.rect,
                )
            {
                state.option = Some(k);
                state.scroll = true;
            }
            // The price shows in the frame of the click, so the scroll below sees it.
            if state.option == Some(k) {
                price(ui, i18n, d, k);
            }
        }
        let next = Button::secondary(&i18n.tr("demand-review"))
            .size(ButtonSize::Medium)
            .enabled(state.option.is_some())
            .show(ui);
        if std::mem::take(&mut state.scroll) {
            next.scroll_to_me_animation(
                Some(egui::Align::Max),
                egui::style::ScrollAnimation::none(),
            );
        }
        if state.option.is_some()
            && (next.clicked() || keys::single(ui, "demand.review", 'R', next.rect))
        {
            state.confirming = true;
            // The confirmation is laid out next frame; scroll to it then.
            state.scroll = true;
        }
        return;
    }
    let Some(k) = state.option else {
        state.confirming = false;
        return;
    };
    let answer = ANSWERS[k];
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "demand-confirm",
            &[
                ("answer", Arg::String(i18n.tr(answer_key(answer)))),
                ("unit", Arg::String(unit.to_owned())),
            ],
        ),
    );
    price(ui, i18n, d, k);
    bf_ui_kit::small(ui, &i18n.tr("demand-final"));
    ui.horizontal(|ui| {
        let yes = Button::danger(&i18n.tr("demand-yes"))
            .size(ButtonSize::Medium)
            .show(ui);
        if yes.clicked() || keys::single(ui, "demand.yes", 'Y', yes.rect) {
            out.answer = Some((v.unit.clone(), answer));
            *state = DemandUi::default();
        }
        let no = Button::ghost(&i18n.tr("liberate-no"))
            .size(ButtonSize::Medium)
            .show(ui);
        if no.clicked() || keys::single(ui, "demand.no", 'N', no.rect) {
            state.confirming = false;
        }
        if std::mem::take(&mut state.scroll) {
            yes.scroll_to_me_animation(
                Some(egui::Align::Max),
                egui::style::ScrollAnimation::none(),
            );
        }
    });
}

/// The account after answer `k`, and for a refusal how far the revolt is.
fn price(ui: &mut Ui, i18n: &I18n, d: &Demand, k: usize) {
    let Some(Some(e)) = d.effects.get(k) else {
        return;
    };
    ui.indent(("demand-price", k), |ui| {
        bf_ui_kit::caption(ui, &i18n.tr("demand-after"));
        account(ui, i18n, &e.after, None);
        if let Some((mobilized, mark)) = e.revolt {
            ui.add(
                egui::Label::new(
                    RichText::new(revolt_line(i18n, mobilized, mark))
                        .font(sans(size::SMALL))
                        .color(color::DANGER),
                )
                .wrap(),
            );
        }
        if ANSWERS[k] == Answer::SelfRule {
            bf_ui_kit::small(ui, &i18n.tr("demand-self-rule-note"));
        }
    });
}
