//! Crisis and declaration (phase 5, slice B; `docs/war.pt-BR.md`, "Crise e declaração"), in the
//! journal: the decision to demand, drafted clause by clause with how the target will weigh it,
//! and each crisis with its deadline, what it takes, the expected answer term by term, who would
//! join (to come) and the player's choice: accept or refuse a demand made on it, declare war or
//! back down after a refusal. Demanding and declaring are rare decisions: two steps, and a
//! restore point before the command goes.

use std::collections::BTreeSet;

use bf_i18n::{Arg, I18n};
use bf_sim::treaty::{Clause, ClauseKind, Proposal, Side};
use bf_sim::war::Stage;
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Sense, Ui, vec2};
use laplace_schelling::Fixed;

use super::journal::Response;
use crate::crisis_view::{CrisisLine, DraftWeighing, Role};
use crate::keys;
use crate::text::country_name;
use crate::world::World;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// The demand being drafted.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DemandUi {
    /// The target the draft was made for: a new target starts a new draft.
    pub target: Option<String>,
    pub states: BTreeSet<String>,
    /// Absorb the whole country.
    pub union: bool,
    /// One payment, in thousandths of the target's GDP.
    pub payment: u16,
    /// A tribute a year, in thousandths of the target's GDP.
    pub tribute: u16,
    pub confirming: bool,
}

/// The choice waiting for its second click in a crisis: accept/declare (`true`) or refuse/back
/// down (`false`).
#[derive(Clone, Debug, Default)]
pub struct CrisisUi {
    pub confirm: Option<(u32, bool)>,
}

const PAYMENT_STEP: i32 = 10;
const TRIBUTE_STEP: i32 = 5;
const MONEY_MAX: i32 = 1000;

fn pct(i18n: &I18n, per_mille: f64) -> String {
    format!("{} %", i18n.decimal(per_mille / 10.0, 1))
}

/// The proposal the draft makes: what the target gives the player.
pub fn proposal(me: &str, target: &str, d: &DemandUi, target_gdp: f64) -> Proposal {
    let give = |kind| Clause {
        kind,
        giver: Side::Junior,
        receiver: Side::Senior,
        until: None,
        note: None,
    };
    let money = |per_mille: u16| Fixed::from_f64_lossy(target_gdp * f64::from(per_mille) / 1000.0);
    let mut clauses = Vec::new();
    if d.union {
        clauses.push(give(ClauseKind::Union));
    } else if !d.states.is_empty() {
        clauses.push(give(ClauseKind::CedeStates {
            states: d.states.iter().cloned().collect(),
        }));
    }
    if d.payment > 0 && target_gdp > 0.0 {
        clauses.push(give(ClauseKind::Payment {
            amount: money(d.payment),
        }));
    }
    if d.tribute > 0 && target_gdp > 0.0 {
        clauses.push(give(ClauseKind::Tribute {
            yearly: money(d.tribute),
        }));
    }
    Proposal {
        side_a: vec![me.to_owned()],
        side_b: vec![target.to_owned()],
        clauses,
        term_years: None,
        notice_years: 0,
    }
}

/// What a demand takes, in words: states, the whole country, money.
fn takes(i18n: &I18n, world: Option<&World>, l: &CrisisLine) -> String {
    let mut parts = Vec::new();
    if l.union {
        parts.push(i18n.tr_args(
            "crisis-takes-union",
            &[("country", arg(country_name(i18n, &l.target)))],
        ));
    }
    if !l.states.is_empty() {
        let names: Vec<String> = l
            .states
            .iter()
            .map(|k| {
                world
                    .and_then(|w| w.states.iter().find(|s| s.def.key == *k))
                    .map_or_else(|| k.clone(), |s| crate::text::state_name(i18n, &s.def))
            })
            .collect();
        parts.push(i18n.tr_args("crisis-takes-states", &[("list", arg(names.join(", ")))]));
    }
    if l.money > 0.0 {
        parts.push(i18n.tr_args(
            "crisis-takes-money",
            &[("money", arg(crate::text::money(i18n, l.money)))],
        ));
    }
    if parts.is_empty() {
        i18n.tr("crisis-takes-nothing")
    } else {
        parts.join(" · ")
    }
}

/// How the target weighs it, term by term (W3).
fn weighing(ui: &mut Ui, i18n: &I18n, l: &CrisisLine) {
    let w = &l.weighing;
    bf_ui_kit::caption(ui, &i18n.tr("crisis-weighing"));
    let ratio = |r: i64| format!("{}×", i18n.decimal(r as f64 / 1000.0, 1));
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("crisis-force"),
        &i18n.tr_args(
            "crisis-force-value",
            &[
                ("ratio", arg(ratio(w.ratio))),
                ("by", arg(i18n.int(l.land_by))),
                ("target", arg(i18n.int(l.land_target))),
            ],
        ),
    );
    // An annexation takes the whole country: the simulation counts it apart, as existential.
    let share = if l.union {
        i18n.tr("crisis-share-all")
    } else {
        pct(i18n, w.share as f64)
    };
    bf_ui_kit::stat_row(ui, &i18n.tr("crisis-share"), &share);
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("crisis-needed"),
        &w.needed
            .map_or_else(|| i18n.tr("crisis-needed-never"), ratio),
    );
    let (key, c) = if w.yields {
        ("crisis-yields", color::POSITIVE)
    } else {
        ("crisis-refuses", color::DANGER)
    };
    ui.label(RichText::new(i18n.tr(key)).font(sans(size::SMALL)).color(c));
    bf_ui_kit::caption(ui, &i18n.tr("crisis-third-parties"));
    bf_ui_kit::small(ui, &i18n.tr("crisis-third-parties-later"));
}

/// The deadline as a bar.
fn deadline(ui: &mut Ui, i18n: &I18n, l: &CrisisLine) {
    let share = if l.deadline_weeks > 0 {
        f32::from(l.weeks_left) / f32::from(l.deadline_weeks)
    } else {
        0.0
    };
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width().min(360.0), 6.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, radius::SM, color::SURFACE_SUNKEN);
    let mut done = r;
    done.set_width(r.width() * share.clamp(0.0, 1.0));
    p.rect_filled(done, radius::SM, color::DANGER);
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "crisis-deadline",
            &[
                ("left", arg(l.weeks_left.to_string())),
                ("count", Arg::Num(f64::from(l.weeks_left))),
            ],
        ),
    );
}

fn stage_key(l: &CrisisLine) -> &'static str {
    match (l.stage, l.role) {
        (Stage::Waiting, Role::Target) => "crisis-stage-answer",
        (Stage::Waiting, Role::Demander) => "crisis-stage-waiting",
        (Stage::Refused, Role::Demander) => "crisis-stage-decide",
        (Stage::Refused, Role::Target) => "crisis-stage-refused",
        (Stage::Accepted, _) => "crisis-stage-accepted",
    }
}

/// A crisis's line in the Agendas: who demands from whom, and where it stands.
pub fn list_label(i18n: &I18n, l: &CrisisLine) -> String {
    i18n.tr_args(
        "crisis-list",
        &[
            ("by", arg(country_name(i18n, &l.by))),
            ("target", arg(country_name(i18n, &l.target))),
            (
                "article",
                arg(crate::text::country_article(i18n, &l.target)),
            ),
            ("stage", arg(i18n.tr(stage_key(l)))),
        ],
    )
}

/// One crisis, with the player's choice when there is one.
pub fn show_crisis(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    l: &CrisisLine,
    st: &mut CrisisUi,
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::SM;
    bf_ui_kit::title(
        ui,
        &i18n.tr_args(
            "crisis-title",
            &[
                ("by", arg(country_name(i18n, &l.by))),
                ("target", arg(country_name(i18n, &l.target))),
                (
                    "article",
                    arg(crate::text::country_article(i18n, &l.target)),
                ),
            ],
        ),
    );
    bf_ui_kit::small(ui, &i18n.tr(stage_key(l)));
    if l.stage == Stage::Waiting {
        deadline(ui, i18n, l);
    }
    bf_ui_kit::body(ui, &takes(i18n, world, l));
    if !l.casus_belli {
        bf_ui_kit::small(ui, &i18n.tr("crisis-no-cause"));
    }
    bf_ui_kit::gap(ui, space::SM);
    weighing(ui, i18n, l);
    bf_ui_kit::gap(ui, space::SM);
    // The choice: the target answers while the deadline runs; the side that demanded decides
    // after a refusal.
    let choice = match (l.role, l.stage) {
        (Role::Target, Stage::Waiting) => Some((
            "crisis-accept",
            "crisis-refuse",
            "crisis-accept-confirm",
            "crisis-refuse-confirm",
        )),
        (Role::Demander, Stage::Refused) => Some((
            "crisis-declare",
            "crisis-back-down",
            "crisis-declare-confirm",
            "crisis-back-down-confirm",
        )),
        _ => None,
    };
    let Some((yes, no, yes_sure, no_sure)) = choice else {
        return;
    };
    match st.confirm {
        Some((id, pick)) if id == l.id => {
            bf_ui_kit::small(ui, &i18n.tr(if pick { yes_sure } else { no_sure }));
            ui.horizontal(|ui| {
                let go = Button::danger(&i18n.tr("crisis-confirm"))
                    .size(ButtonSize::Medium)
                    .show(ui);
                if go.clicked() || keys::single(ui, "crisis.confirm", 'Y', go.rect) {
                    match l.role {
                        Role::Target => out.crisis_answer = Some((l.id, pick)),
                        Role::Demander => out.declare = Some((l.id, pick)),
                    }
                    st.confirm = None;
                }
                let back = Button::ghost(&i18n.tr("crisis-cancel"))
                    .size(ButtonSize::Medium)
                    .show(ui);
                if back.clicked() || keys::single(ui, "crisis.cancel", 'N', back.rect) {
                    st.confirm = None;
                }
            });
        }
        _ => {
            ui.horizontal(|ui| {
                let a = Button::secondary(&i18n.tr(yes))
                    .size(ButtonSize::Medium)
                    .show(ui);
                if a.clicked() || keys::single(ui, "crisis.yes", 'A', a.rect) {
                    st.confirm = Some((l.id, true));
                }
                let b = Button::secondary(&i18n.tr(no))
                    .size(ButtonSize::Medium)
                    .show(ui);
                if b.clicked() || keys::single(ui, "crisis.no", 'R', b.rect) {
                    st.confirm = Some((l.id, false));
                }
            });
        }
    }
}

/// The decision to demand, from the country selected on the map. Returns the draft to weigh.
#[allow(clippy::too_many_arguments)]
pub fn show_demand(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    me: &str,
    target: Option<&str>,
    weighed: Option<&DraftWeighing>,
    d: &mut DemandUi,
    out: &mut Response,
) -> Option<(String, Proposal)> {
    ui.spacing_mut().item_spacing.y = space::SM;
    bf_ui_kit::title(ui, &i18n.tr("crisis-demand-title"));
    let (Some(target), Some(world)) = (target, world) else {
        bf_ui_kit::muted(ui, &i18n.tr("crisis-demand-pick-target"));
        return None;
    };
    if d.target.as_deref() != Some(target) {
        *d = DemandUi {
            target: Some(target.to_owned()),
            ..DemandUi::default()
        };
    }
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "crisis-demand-of",
            &[
                ("country", arg(country_name(i18n, target))),
                ("article", arg(crate::text::country_article(i18n, target))),
            ],
        ),
    );
    let gdp = weighed
        .filter(|w| w.target == target)
        .map_or(0.0, |w| w.target_gdp);
    if !d.confirming {
        // What to ask: the whole country, or states of it; money.
        bf_ui_kit::caption(ui, &i18n.tr("crisis-demand-clauses"));
        let union = ui.checkbox(&mut d.union, i18n.tr("crisis-demand-union"));
        if union.changed() {
            d.states.clear();
        }
        if !d.union {
            let ci = world.countries.iter().position(|c| c.tag() == target);
            let states: Vec<(String, String)> = ci
                .map(|c| {
                    let country = &world.countries[c];
                    country
                        .states
                        .iter()
                        .chain(&country.colonial_states)
                        .filter_map(|s| world.states.get(*s))
                        .map(|s| (s.def.key.clone(), crate::text::state_name(i18n, &s.def)))
                        .collect()
                })
                .unwrap_or_default();
            bf_ui_kit::small(ui, &i18n.tr("crisis-demand-states"));
            egui::ScrollArea::vertical()
                .id_salt("crisis-demand-states")
                .max_height(140.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (key, name) in &states {
                            let mut on = d.states.contains(key);
                            if ui.checkbox(&mut on, name).changed() {
                                if on {
                                    d.states.insert(key.clone());
                                } else {
                                    d.states.remove(key);
                                }
                            }
                        }
                    });
                });
        }
        for (label, value, step, id) in [
            (
                "crisis-demand-payment",
                &mut d.payment,
                PAYMENT_STEP,
                "crisis.demand.payment",
            ),
            (
                "crisis-demand-tribute",
                &mut d.tribute,
                TRIBUTE_STEP,
                "crisis.demand.tribute",
            ),
        ] {
            let now = *value;
            let s = super::gov_ui::control_row(ui, &i18n.tr(label), |ui| {
                super::gov_ui::stepper(
                    ui,
                    &pct(i18n, f64::from(now)),
                    &i18n.tr("crisis-demand-of-gdp"),
                    now > 0 && gdp > 0.0,
                    i32::from(now) < MONEY_MAX && gdp > 0.0,
                    super::gov_ui::StepTips {
                        family: 'M',
                        id,
                        row: usize::from(label == "crisis-demand-tribute"),
                    },
                )
            });
            if s != 0 {
                *value = (i32::from(now) + s * step).clamp(0, MONEY_MAX) as u16;
            }
        }
    }
    let p = proposal(me, target, d, gdp);
    let empty = p.clauses.is_empty();
    if let Some(w) = weighed.filter(|w| w.target == target && w.demand == p) {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "crisis-demand-deadline",
                &[("weeks", arg(w.line.deadline_weeks.to_string()))],
            ),
        );
        bf_ui_kit::small(ui, &i18n.tr("crisis-no-cause"));
        if !empty {
            weighing(ui, i18n, &w.line);
        }
        if let Some(b) = w.blocked {
            ui.label(
                RichText::new(i18n.tr(b))
                    .font(sans(size::SMALL))
                    .color(color::DANGER),
            );
            return Some((target.to_owned(), p));
        }
    }
    bf_ui_kit::gap(ui, space::SM);
    if !d.confirming {
        let next = Button::secondary(&i18n.tr("crisis-demand-review"))
            .size(ButtonSize::Medium)
            .enabled(!empty)
            .show(ui);
        if !empty && (next.clicked() || keys::single(ui, "crisis.demand.review", 'R', next.rect)) {
            d.confirming = true;
        }
    } else {
        bf_ui_kit::body(
            ui,
            &i18n.tr_args(
                "crisis-demand-confirm",
                &[("country", arg(country_name(i18n, target)))],
            ),
        );
        bf_ui_kit::small(ui, &i18n.tr("crisis-demand-final"));
        ui.horizontal(|ui| {
            let go = Button::danger(&i18n.tr("crisis-demand-send"))
                .size(ButtonSize::Medium)
                .show(ui);
            if go.clicked() || keys::single(ui, "crisis.demand.send", 'Y', go.rect) {
                out.crisis_demand = Some((target.to_owned(), p.clone()));
                d.confirming = false;
            }
            let back = Button::ghost(&i18n.tr("crisis-cancel"))
                .size(ButtonSize::Medium)
                .show(ui);
            if back.clicked() || keys::single(ui, "crisis.demand.back", 'N', back.rect) {
                d.confirming = false;
            }
        });
    }
    Some((target.to_owned(), p))
}
