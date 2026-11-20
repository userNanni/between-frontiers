//! The negotiation table (`docs/treaties.pt-BR.md`, section 13), a rare decision of the
//! journal's agendas: the player drafts a proposal to another country clause by clause, each
//! side in its column, with the parameters inside the catalogue's limits; sees the account each
//! side makes, term by term, and the answer the other side will likely give; why the proposal
//! cannot go (a clause outside its limits or forbidden by recognition, an identical proposal
//! refused in the last months) and which clauses sleep. A proposal received is answered here
//! too: accept, refuse, or counter from it on the same table.

use std::collections::BTreeSet;

use bf_i18n::{Arg, I18n};
use bf_sim::treaty::{
    Clause, ClauseCatalogue, ClauseKind, Currency, DebtTerms, LimitUnit, Proposal, Side,
};
use bf_sim::treaty_table::{Outcome, ProposalAnswer};
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Ui};
use laplace_schelling::Fixed;

use super::budget::pct;
use super::journal::Response;
use crate::keys;
use crate::text::{country_name, date_long, money};
use crate::treaty_view::{Account, Likely, OpenLine, TableView, Weighed};
use crate::world::World;

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// Width below which the two sides stack.
const TWO_COLUMNS: f32 = 680.0;

/// The clauses the table's editor can draft: the ones whose parameters it knows how to set.
/// The others (a country released, arms ceilings, a protected community, a debt revised) come
/// with the screens that know their parameters.
const DRAFTABLE: &[&str] = &[
    "payment",
    "debt",
    "assume_debt",
    "cede_states",
    "union",
    "techniques",
    "recognition",
    "non_aggression",
    "defence",
    "consultation",
    "border_guarantee",
    "demilitarized",
    "base",
    "passage",
    "occupation",
    "tariff_cap",
    "tariff_preference",
    "customs_union",
    "currency_peg",
    "monetary_union",
    "capital_protection",
    "financial_control",
    "tribute",
    "foreign_policy",
    "intervention",
    "troops_quota",
    "offence",
    "neutrality",
];

/// Terms the editor steps through: none, then years.
const TERMS: [Option<u16>; 10] = [
    None,
    Some(1),
    Some(5),
    Some(10),
    Some(15),
    Some(20),
    Some(25),
    Some(30),
    Some(50),
    Some(99),
];

/// The table's state in the journal.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableUi {
    /// The draft, with the player as side A, and the other party.
    pub draft: Option<(String, Proposal)>,
    /// The proposal received that the draft counters (its id): the draft goes as the answer.
    pub counter: Option<u32>,
    /// The clause picker is open for the player's column (`true`) or the other's.
    pub adding: Option<bool>,
    /// The clause whose states are being picked.
    pub states_of: Option<usize>,
    /// An answer to a proposal received waiting for its second click: (id, accept).
    pub confirm: Option<(u32, bool)>,
    /// The draft came from another screen (a debt to renegotiate): its other party stays, the
    /// country picked on the map does not replace it.
    pub fixed: bool,
    /// The title of a draft opened from another screen (`table-renegotiate-title`,
    /// `table-peace-title`).
    pub title: Option<&'static str>,
    /// The search for the other party by name (keyboard).
    pub find: super::country_find::FindUi,
}

/// The proposal with the sides swapped: the orientation the other party sent it in, or the
/// draft's from a proposal received.
pub fn swapped(p: &Proposal) -> Proposal {
    let flip = |s: &Side| match s {
        Side::Senior => Side::Junior,
        Side::Junior => Side::Senior,
        other => other.clone(),
    };
    Proposal {
        side_a: p.side_b.clone(),
        side_b: p.side_a.clone(),
        clauses: p
            .clauses
            .iter()
            .map(|c| Clause {
                giver: flip(&c.giver),
                receiver: flip(&c.receiver),
                ..c.clone()
            })
            .collect(),
        ..p.clone()
    }
}

fn empty(me: &str, other: &str) -> Proposal {
    Proposal {
        side_a: vec![me.to_owned()],
        side_b: vec![other.to_owned()],
        clauses: Vec::new(),
        term_years: None,
        notice_years: 0,
    }
}

/// Whether side A (the player, in a draft) gives clause `c`.
fn a_gives(c: &Clause) -> bool {
    matches!(c.giver, Side::Senior)
}

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// A clause of kind `key` as the editor first puts it on the table, its parameters ten steps
/// above their minimum (a debt of twenty years); `giver_gdp` is the base of a share of GDP.
pub fn default_kind(cat: &ClauseCatalogue, key: &str, giver_gdp: f64) -> Option<ClauseKind> {
    let def = cat.get(key)?;
    let start = |param: &str| -> Fixed {
        let Some(l) = def.limits.iter().find(|l| l.param == param) else {
            return Fixed::ZERO;
        };
        let x = (l.min + l.step * Fixed::from_int(10)).min(l.max);
        match l.unit {
            LimitUnit::ShareOfGdp | LimitUnit::ShareOfGdpYearly => {
                Fixed::from_f64_lossy(f(x) * giver_gdp)
            }
            LimitUnit::Fraction | LimitUnit::Years => x,
        }
    };
    use ClauseKind::*;
    Some(match key {
        "payment" => Payment {
            amount: start("amount"),
        },
        "tribute" => Tribute {
            yearly: start("yearly"),
        },
        "debt" => {
            let mut t = DebtTerms {
                principal: start("principal"),
                rate: start("rate"),
                annuity: Fixed::ZERO,
                years: 20,
                currency: Currency::Reference,
                deferrable: Fixed::ZERO,
                transfer_cap: None,
                pledge: None,
                contingent: Fixed::ZERO,
                grace_years: 0,
            };
            t.annuity = annuity(&t);
            Debt { terms: Some(t) }
        }
        "assume_debt" => AssumeDebt {
            share: start("share"),
        },
        "tariff_cap" => TariffCap {
            rate: Some(start("rate")),
        },
        "tariff_preference" => TariffPreference {
            margin: start("margin"),
        },
        "cede_states" => CedeStates { states: Vec::new() },
        "demilitarized" => Demilitarized { states: Vec::new() },
        "occupation" => Occupation { states: Vec::new() },
        "base" => Base { at: Vec::new() },
        "recognition" => Recognition { border: false },
        "offence" => Offence {
            against: Vec::new(),
        },
        "union" => Union,
        "techniques" => Techniques,
        "non_aggression" => NonAggression,
        "defence" => Defence,
        "consultation" => Consultation,
        "border_guarantee" => BorderGuarantee,
        "passage" => Passage,
        "customs_union" => CustomsUnion,
        "currency_peg" => CurrencyPeg,
        "monetary_union" => MonetaryUnion,
        "capital_protection" => CapitalProtection,
        "financial_control" => FinancialControl,
        "foreign_policy" => ForeignPolicy,
        "intervention" => Intervention,
        "troops_quota" => TroopsQuota,
        "neutrality" => Neutrality,
        _ => return None,
    })
}

/// The yearly payment of a debt's terms (interest and amortization).
fn annuity(t: &DebtTerms) -> Fixed {
    Fixed::from_f64_lossy(bf_sim::treaty::level_annuity(
        f(t.principal),
        f(t.rate),
        t.years,
    ))
}

/// Sets parameter `name` of a clause (the names of `treaty::params`).
pub fn set_param(kind: &mut ClauseKind, name: &str, v: Fixed) {
    use ClauseKind::*;
    match (kind, name) {
        (Payment { amount }, "amount") => *amount = v,
        (Tribute { yearly }, "yearly") => *yearly = v,
        (AssumeDebt { share }, "share") => *share = v,
        (TariffCap { rate: Some(r) }, "rate") => *r = v,
        (TariffPreference { margin }, "margin") => *margin = v,
        (Debt { terms: Some(t) } | ReviseDebt { terms: t, .. }, name) => {
            match name {
                "principal" => t.principal = v,
                "rate" => t.rate = v,
                "years" => t.years = v.to_int_round().clamp(1, i64::from(u16::MAX)) as u16,
                "deferrable" => t.deferrable = v,
                "grace_years" => {
                    t.grace_years = v.to_int_round().clamp(0, i64::from(u16::MAX)) as u16
                }
                _ => return,
            }
            t.annuity = annuity(t);
        }
        _ => {}
    }
}

/// The states a clause names, when it names any.
fn states_mut(kind: &mut ClauseKind) -> Option<&mut Vec<String>> {
    match kind {
        ClauseKind::CedeStates { states }
        | ClauseKind::Demilitarized { states }
        | ClauseKind::Occupation { states } => Some(states),
        ClauseKind::Base { at } => Some(at),
        _ => None,
    }
}

/// What the table sends: a proposal (the other party, the draft) or an answer to one received.
pub enum Sent {
    Propose(String, Proposal),
    Answer(u32, ProposalAnswer),
}

/// The table, drafting a proposal to `other` (or a counter to the proposal received
/// `st.counter`). Returns the draft to weigh.
#[allow(clippy::too_many_arguments)]
pub fn show_table(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    cat: Option<&ClauseCatalogue>,
    me: &str,
    other: Option<&str>,
    table: &TableView,
    weighed: Option<&Weighed>,
    st: &mut TableUi,
    out: &mut Response,
) -> Option<(String, Proposal)> {
    ui.spacing_mut().item_spacing.y = space::SM;
    let countering = st.counter.is_some();
    bf_ui_kit::title(
        ui,
        &i18n.tr(if countering {
            "table-counter-title"
        } else if let Some(t) = st.title {
            t
        } else {
            "table-title"
        }),
    );
    let Some(cat) = cat else {
        bf_ui_kit::muted(ui, &i18n.tr("table-off"));
        return None;
    };
    // The other party found by name (keyboard): a new draft with it, fixed against the map.
    if !countering
        && st.title.is_none()
        && let Some(world) = world
        && let Some(tag) =
            super::country_find::show(ui, i18n, world, me, &mut st.find, "treaty.table", 'C')
    {
        *st = TableUi {
            draft: Some((tag.clone(), empty(me, &tag))),
            fixed: true,
            ..TableUi::default()
        };
    }
    // A new other party starts a new draft; a counter keeps its own.
    if !countering && !st.fixed {
        match other {
            Some(o) if o != me => {
                if st.draft.as_ref().map(|(t, _)| t.as_str()) != Some(o) {
                    *st = TableUi {
                        draft: Some((o.to_owned(), empty(me, o))),
                        ..TableUi::default()
                    };
                }
            }
            _ => {
                bf_ui_kit::muted(ui, &i18n.tr("table-pick-country"));
                return None;
            }
        }
    }
    let (other, mut p) = st.draft.clone()?;
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "table-parties",
            &[
                ("me", arg(country_name(i18n, me))),
                ("other", arg(country_name(i18n, &other))),
            ],
        ),
    );
    let w = weighed.filter(|w| w.other == other && w.proposal == p);
    let gdp = |t: &str| table.gdp.get(t).copied().unwrap_or(0.0);
    // The two sides, each in its column.
    let wide = ui.available_width() >= TWO_COLUMNS;
    let mut edit = Edit::default();
    let mut column = |ui: &mut Ui, mine: bool| {
        let giver = if mine { me } else { other.as_str() };
        side(
            ui,
            i18n,
            world,
            cat,
            &p,
            w,
            giver,
            gdp(giver),
            mine,
            st,
            &mut edit,
        );
    };
    if wide {
        bf_ui_kit::columns(ui, 2, |cols| {
            column(&mut cols[0], true);
            column(&mut cols[1], false);
        });
    } else {
        column(ui, true);
        bf_ui_kit::gap(ui, space::SM);
        column(ui, false);
    }
    edit.apply(&mut p, cat, gdp(me), gdp(&other));
    // Term and notice.
    bf_ui_kit::gap(ui, space::SM);
    term(ui, i18n, &mut p);
    // The accounts and the likely answer.
    if let Some(w) = w {
        bf_ui_kit::gap(ui, space::SM);
        let accounts = |ui: &mut Ui, tag: &str, a: &Account| account(ui, i18n, tag, a);
        if wide {
            bf_ui_kit::columns(ui, 2, |cols| {
                accounts(&mut cols[0], me, &w.me);
                accounts(&mut cols[1], &other, &w.other_account);
            });
        } else {
            accounts(ui, me, &w.me);
            accounts(ui, &other, &w.other_account);
        }
        if let Some(l) = &w.likely {
            likely(ui, i18n, &other, l);
        }
        if let Some((refused, back)) = w.blocked.filter(|_| !countering) {
            warn(
                ui,
                &i18n.tr_args(
                    "table-blocked",
                    &[
                        ("refused", arg(date_long(i18n, refused))),
                        ("back", arg(date_long(i18n, back))),
                    ],
                ),
            );
        }
    }
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "table-deadline",
            &[
                ("weeks", arg(table.deadline_weeks.to_string())),
                ("country", arg(country_name(i18n, &other))),
            ],
        ),
    );
    // Why it cannot go, and the button.
    let refusal = match w {
        Some(w) => w
            .refusal()
            .filter(|r| !(countering && *r == "table-refusal-cooldown")),
        None if p.clauses.is_empty() => Some("table-refusal-empty"),
        None => Some("table-weighing"),
    };
    if let Some(r) = refusal {
        bf_ui_kit::small(ui, &i18n.tr(r));
    }
    ui.horizontal(|ui| {
        let label = i18n.tr(if countering {
            "table-send-counter"
        } else {
            "table-propose"
        });
        let go = Button::primary(&label)
            .size(ButtonSize::Medium)
            .enabled(refusal.is_none())
            .show(ui);
        if refusal.is_none() && (go.clicked() || keys::single(ui, "treaty.propose", 'P', go.rect)) {
            out.treaty = Some(match st.counter {
                Some(id) => Sent::Answer(id, ProposalAnswer::Counter(swapped(&p))),
                None => Sent::Propose(other.clone(), p.clone()),
            });
            *st = TableUi::default();
        }
        if countering {
            let back = Button::ghost(&i18n.tr("table-counter-drop"))
                .size(ButtonSize::Medium)
                .show(ui);
            if back.clicked() || keys::single(ui, "treaty.counter.drop", 'N', back.rect) {
                *st = TableUi::default();
            }
        }
    });
    if let Some((_, d)) = st.draft.as_mut() {
        *d = p.clone();
    }
    st.draft.is_some().then_some((other, p))
}

/// What the columns asked to change, applied after both are drawn.
#[derive(Default)]
struct Edit {
    add: Option<(bool, String)>,
    remove: Option<usize>,
    param: Option<(usize, &'static str, Fixed)>,
    toggle_state: Option<(usize, String)>,
    border: Option<(usize, bool)>,
}

impl Edit {
    fn apply(self, p: &mut Proposal, cat: &ClauseCatalogue, my_gdp: f64, other_gdp: f64) {
        if let Some((mine, key)) = self.add {
            let gdp = if mine { my_gdp } else { other_gdp };
            if let Some(kind) = default_kind(cat, &key, gdp) {
                let (giver, receiver) = if mine {
                    (Side::Senior, Side::Junior)
                } else {
                    (Side::Junior, Side::Senior)
                };
                p.clauses.push(Clause {
                    kind,
                    giver,
                    receiver,
                    until: None,
                    note: None,
                });
            }
        }
        if let Some((ci, name, v)) = self.param
            && let Some(c) = p.clauses.get_mut(ci)
        {
            set_param(&mut c.kind, name, v);
        }
        if let Some((ci, key)) = self.toggle_state
            && let Some(states) = p.clauses.get_mut(ci).and_then(|c| states_mut(&mut c.kind))
        {
            match states.iter().position(|s| *s == key) {
                Some(i) => {
                    states.remove(i);
                }
                None => states.push(key),
            }
        }
        if let Some((ci, on)) = self.border
            && let Some(ClauseKind::Recognition { border }) =
                p.clauses.get_mut(ci).map(|c| &mut c.kind)
        {
            *border = on;
        }
        if let Some(ci) = self.remove
            && ci < p.clauses.len()
        {
            p.clauses.remove(ci);
        }
    }
}

/// One side's column: what `giver` gives, each clause with its parameters, its notes and what it
/// is worth to each party, and the clause picker.
#[allow(clippy::too_many_arguments)]
fn side(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    cat: &ClauseCatalogue,
    p: &Proposal,
    w: Option<&Weighed>,
    giver: &str,
    gdp: f64,
    mine: bool,
    st: &mut TableUi,
    edit: &mut Edit,
) {
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "table-gives",
            &[("country", arg(country_name(i18n, giver)))],
        ),
    );
    let mut any = false;
    for (ci, c) in p.clauses.iter().enumerate() {
        if a_gives(c) != mine {
            continue;
        }
        any = true;
        clause(ui, i18n, world, cat, ci, c, w, giver, gdp, st, edit);
    }
    if !any {
        bf_ui_kit::small(ui, &i18n.tr("table-gives-nothing"));
    }
    let family = if mine { 'A' } else { 'S' };
    let open = st.adding == Some(mine);
    let add = Button::secondary(&i18n.tr("table-add"))
        .size(ButtonSize::Small)
        .show(ui);
    if add.clicked() || keys::single(ui, &format!("treaty.add.{mine}"), family, add.rect) {
        st.adding = (!open).then_some(mine);
    }
    if open {
        picker(ui, i18n, cat, mine, edit, st);
    }
}

fn picker(
    ui: &mut Ui,
    i18n: &I18n,
    cat: &ClauseCatalogue,
    mine: bool,
    edit: &mut Edit,
    st: &mut TableUi,
) {
    let mut k = 0usize;
    for (once, caption) in [(true, "table-once"), (false, "table-in-force")] {
        bf_ui_kit::caption(ui, &i18n.tr(caption));
        ui.horizontal_wrapped(|ui| {
            for def in cat
                .clauses
                .iter()
                .filter(|d| d.once == once && DRAFTABLE.contains(&d.key.as_str()))
            {
                k += 1;
                let b = Button::ghost(&i18n.tr(&format!("clause-{}", def.key)))
                    .size(ButtonSize::Small)
                    .show(ui)
                    .on_hover_text(bf_ui_kit::tooltip_text(
                        &i18n.tr(&format!("clause-{}-effect", def.key)),
                    ));
                if b.clicked()
                    || keys::item(
                        ui,
                        'K',
                        format!("treaty.pick.{}", def.key),
                        &k.to_string(),
                        b.rect,
                    )
                {
                    edit.add = Some((mine, def.key.clone()));
                    st.adding = None;
                }
            }
        });
    }
}

/// One clause on the table.
#[allow(clippy::too_many_arguments)]
fn clause(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    cat: &ClauseCatalogue,
    ci: usize,
    c: &Clause,
    w: Option<&Weighed>,
    giver: &str,
    gdp: f64,
    st: &mut TableUi,
    edit: &mut Edit,
) {
    let key = c.kind.key();
    bf_ui_kit::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.horizontal(|ui| {
            ui.strong(i18n.tr(&format!("clause-{key}")))
                .on_hover_text(bf_ui_kit::tooltip_text(
                    &i18n.tr(&format!("clause-{key}-effect")),
                ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let b = Button::ghost(&i18n.tr("table-remove"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if b.clicked()
                    || keys::item(
                        ui,
                        'R',
                        format!("treaty.remove.{ci}"),
                        &(ci + 1).to_string(),
                        b.rect,
                    )
                {
                    edit.remove = Some(ci);
                }
            });
        });
        // The numeric parameters, inside the catalogue's limits.
        let def = cat.get(key);
        for (row, (name, value)) in bf_sim::treaty::params(&c.kind).into_iter().enumerate() {
            let Some(l) = def.and_then(|d| d.limits.iter().find(|l| l.param == name)) else {
                continue;
            };
            let scale = match l.unit {
                LimitUnit::ShareOfGdp | LimitUnit::ShareOfGdpYearly => gdp,
                LimitUnit::Fraction | LimitUnit::Years => 1.0,
            };
            let (lo, hi, step) = (f(l.min) * scale, f(l.max) * scale, f(l.step) * scale);
            let v = f(value);
            let shown = match l.unit {
                LimitUnit::ShareOfGdp | LimitUnit::ShareOfGdpYearly => {
                    if gdp > 0.0 {
                        pct(i18n, v / gdp * 100.0, 1)
                    } else {
                        "—".into()
                    }
                }
                LimitUnit::Fraction => pct(i18n, v * 100.0, if f(l.step) < 0.01 { 1 } else { 0 }),
                LimitUnit::Years => i18n.int(v.round() as i64),
            };
            let d =
                super::gov_ui::control_row(ui, &i18n.tr(&format!("table-param-{name}")), |ui| {
                    super::gov_ui::stepper_typed(
                        ui,
                        &shown,
                        super::gov_ui::STEP_VALUE_W,
                        &i18n.tr(&format!("table-param-{name}")),
                        v > lo + step / 2.0,
                        v < hi - step / 2.0 && step > 0.0,
                        super::gov_ui::StepTips {
                            family: 'V',
                            id: &format!("treaty.param.{ci}.{name}"),
                            row,
                        },
                        param_typed(i18n, l.unit, gdp, v, lo, hi),
                    )
                });
            // A number typed is in the unit shown (a share of GDP in %, a fraction in %, years),
            // and lands on the clause's steps.
            let x = match d {
                super::gov_ui::StepOut::Step(d) => Some(v + f64::from(d) * step),
                super::gov_ui::StepOut::Set(t) => {
                    let x = match l.unit {
                        LimitUnit::ShareOfGdp | LimitUnit::ShareOfGdpYearly => t / 100.0 * gdp,
                        LimitUnit::Fraction => t / 100.0,
                        LimitUnit::Years => t,
                    };
                    Some(if step > 0.0 {
                        lo + ((x - lo) / step).round() * step
                    } else {
                        x
                    })
                }
                super::gov_ui::StepOut::None => None,
            };
            if let Some(x) = x {
                let x = x.clamp(lo, hi);
                edit.param = Some((ci, name, Fixed::from_f64_lossy(x)));
            }
            match l.unit {
                LimitUnit::ShareOfGdp => {
                    bf_ui_kit::small(ui, &money(i18n, v));
                }
                LimitUnit::ShareOfGdpYearly => {
                    bf_ui_kit::small(
                        ui,
                        &i18n.tr_args("table-per-year", &[("money", arg(money(i18n, v)))]),
                    );
                }
                LimitUnit::Fraction | LimitUnit::Years => {}
            }
        }
        revision_lines(ui, i18n, &c.kind);
        if let ClauseKind::Debt { terms: Some(t) } = &c.kind {
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "table-annuity",
                    &[("money", arg(money(i18n, f(t.annuity))))],
                ),
            );
        }
        if let ClauseKind::Recognition { border } = &c.kind {
            let mut on = *border;
            let b = ui.checkbox(&mut on, i18n.tr("table-recognition-border"));
            if keys::item(
                ui,
                'B',
                format!("treaty.border.{ci}"),
                &(ci + 1).to_string(),
                b.rect,
            ) {
                edit.border = Some((ci, !*border));
            } else if b.changed() {
                edit.border = Some((ci, on));
            }
        }
        states(ui, i18n, world, ci, c, giver, st, edit);
        // What the table says of it.
        if let Some(n) = w.and_then(|w| w.notes.get(ci)) {
            if let Some(front) = n.dormant {
                bf_ui_kit::small(ui, &i18n.tr(&format!("front-{}", front.key())));
            }
            for param in &n.limits {
                warn(
                    ui,
                    &i18n.tr_args(
                        "table-out-of-limits",
                        &[("param", arg(i18n.tr(&format!("table-param-{param}"))))],
                    ),
                );
            }
            if n.unrecognised {
                warn(ui, &i18n.tr("table-unrecognised"));
            }
        }
        if let Some(w) = w {
            let mine = w.me.clauses.get(ci).copied().unwrap_or(0.0);
            let theirs = w.other_account.clauses.get(ci).copied().unwrap_or(0.0);
            if mine != 0.0 || theirs != 0.0 {
                bf_ui_kit::small(
                    ui,
                    &i18n.tr_args(
                        "table-clause-value",
                        &[
                            ("me", arg(signed(i18n, mine))),
                            ("other", arg(signed(i18n, theirs))),
                        ],
                    ),
                );
            }
        }
    });
}

/// The terms a debt revision sets: of which treaty, and what the debt becomes.
fn revision_lines(ui: &mut Ui, i18n: &I18n, kind: &ClauseKind) {
    let ClauseKind::ReviseDebt { treaty, terms: t } = kind else {
        return;
    };
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "table-revise-of",
            &[("treaty", arg(super::ledger::treaty_name(i18n, treaty)))],
        ),
    );
    for (key, value) in [
        ("table-param-principal", money(i18n, f(t.principal))),
        ("table-param-rate", pct(i18n, f(t.rate) * 100.0, 1)),
        ("table-annuity-label", money(i18n, f(t.annuity))),
        (
            "table-param-years",
            i18n.tr_args("table-years", &[("n", arg(t.years.to_string()))]),
        ),
    ] {
        bf_ui_kit::stat_row(ui, &i18n.tr(key), &value);
    }
    if t.grace_years > 0 {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr("table-param-grace_years"),
            &i18n.tr_args("table-years", &[("n", arg(t.grace_years.to_string()))]),
        );
    }
}

/// The states a clause names: how many, and the giver's states to pick from.
#[allow(clippy::too_many_arguments)]
fn states(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    ci: usize,
    c: &Clause,
    giver: &str,
    st: &mut TableUi,
    edit: &mut Edit,
) {
    let picked: BTreeSet<&str> = match &c.kind {
        ClauseKind::CedeStates { states }
        | ClauseKind::Demilitarized { states }
        | ClauseKind::Occupation { states } => states.iter().map(String::as_str).collect(),
        ClauseKind::Base { at } => at.iter().map(String::as_str).collect(),
        _ => return,
    };
    let n = picked.len();
    ui.horizontal(|ui| {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "table-states",
                &[
                    ("n", arg(i18n.int(n as i64))),
                    ("count", Arg::Num(n as f64)),
                ],
            ),
        );
        let open = st.states_of == Some(ci);
        let b = Button::ghost(&i18n.tr("table-states-pick"))
            .size(ButtonSize::Small)
            .show(ui);
        if b.clicked()
            || keys::item(
                ui,
                'E',
                format!("treaty.states.{ci}"),
                &(ci + 1).to_string(),
                b.rect,
            )
        {
            st.states_of = (!open).then_some(ci);
        }
    });
    if st.states_of != Some(ci) {
        return;
    }
    let Some(world) = world else {
        return;
    };
    let Some(country) = world.countries.iter().find(|x| x.tag() == giver) else {
        return;
    };
    egui::ScrollArea::vertical()
        .id_salt(("treaty-states", ci))
        .max_height(140.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (k, s) in country
                    .states
                    .iter()
                    .chain(&country.colonial_states)
                    .filter_map(|s| world.states.get(*s))
                    .enumerate()
                {
                    let mut on = picked.contains(s.def.key.as_str());
                    let b = ui.checkbox(&mut on, crate::text::state_name(i18n, &s.def));
                    let typed = keys::item(
                        ui,
                        'G',
                        format!("treaty.state.{ci}.{}", s.def.key),
                        &(k + 1).to_string(),
                        b.rect,
                    );
                    if b.changed() || typed {
                        edit.toggle_state = Some((ci, s.def.key.clone()));
                    }
                }
            });
        });
}

/// A clause's parameter typed in the unit it shows: a share of GDP or a fraction in percent,
/// years as a whole number; the limits in the same unit.
fn param_typed(
    i18n: &I18n,
    unit: LimitUnit,
    gdp: f64,
    v: f64,
    lo: f64,
    hi: f64,
) -> super::gov_ui::Typed {
    match unit {
        LimitUnit::ShareOfGdp | LimitUnit::ShareOfGdpYearly => {
            let of = |x: f64| if gdp > 0.0 { x / gdp * 100.0 } else { 0.0 };
            super::gov_ui::typed_pct(i18n, of(v), of(lo), of(hi), 1)
        }
        LimitUnit::Fraction => super::gov_ui::typed_pct(i18n, v * 100.0, lo * 100.0, hi * 100.0, 1),
        LimitUnit::Years => {
            super::gov_ui::typed_int(i18n, v.round() as i64, lo.round() as i64, hi.round() as i64)
        }
    }
}

/// The term and the notice of the draft. Both values sit in one width, the longest either can
/// show in this language ("No term", "99 years"), so the words stay clear of − and +.
fn term(ui: &mut Ui, i18n: &I18n, p: &mut Proposal) {
    let years = |n: u16| i18n.tr_args("table-years", &[("n", arg(n.to_string()))]);
    let term_text = |t: Option<u16>| t.map_or_else(|| i18n.tr("table-term-none"), years);
    let all: Vec<String> = TERMS.iter().map(|t| term_text(*t)).collect();
    let width = super::gov_ui::step_width(ui, &all);
    let at = TERMS.iter().position(|t| *t == p.term_years).unwrap_or(0);
    let d = super::gov_ui::control_row(ui, &i18n.tr("table-term"), |ui| {
        super::gov_ui::stepper_wide(
            ui,
            &term_text(p.term_years),
            width,
            &i18n.tr("table-term-tip"),
            at > 0,
            at + 1 < TERMS.len(),
            super::gov_ui::StepTips {
                family: 'W',
                id: "treaty.term",
                row: 0,
            },
        )
    });
    if d != 0 {
        p.term_years = TERMS[(at as i32 + d).clamp(0, TERMS.len() as i32 - 1) as usize];
    }
    let most = p.term_years.unwrap_or(10);
    p.notice_years = p.notice_years.min(most);
    let n = p.notice_years;
    let d = super::gov_ui::control_row(ui, &i18n.tr("table-notice"), |ui| {
        super::gov_ui::stepper_typed(
            ui,
            &years(n),
            width,
            &i18n.tr("table-notice-tip"),
            n > 0,
            n < most,
            super::gov_ui::StepTips {
                family: 'W',
                id: "treaty.notice",
                row: 1,
            },
            super::gov_ui::typed_int(i18n, i64::from(n), 0, i64::from(most)),
        )
    });
    if let Some(to) = super::gov_ui::step_to(d, i32::from(n), 1, 1.0, 0, i32::from(most)) {
        p.notice_years = to as u16;
    }
    bf_ui_kit::small(
        ui,
        &i18n.tr(if p.notice_years > 0 {
            "table-notice-some"
        } else {
            "table-notice-none"
        }),
    );
}

fn signed(i18n: &I18n, x: f64) -> String {
    if x > 0.0 {
        format!("+{}", money(i18n, x))
    } else if x < 0.0 {
        format!("−{}", money(i18n, -x))
    } else {
        money(i18n, 0.0)
    }
}

/// One party's account: the total and each term.
fn account(ui: &mut Ui, i18n: &I18n, tag: &str, a: &Account) {
    let total = signed(i18n, a.total);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(i18n.tr_args(
                "table-account",
                &[("country", arg(country_name(i18n, tag)))],
            ))
            .font(sans(size::BODY))
            .strong(),
        );
        ui.label(
            RichText::new(total)
                .font(sans(size::BODY))
                .color(if a.total > 0.0 {
                    color::POSITIVE
                } else {
                    color::DANGER
                }),
        );
    });
    for (t, v) in &a.terms {
        bf_ui_kit::stat_row(
            ui,
            &i18n.tr(&format!("term-{}", t.key())),
            &signed(i18n, *v),
        );
    }
    if a.terms.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("table-account-empty"));
    }
}

fn likely(ui: &mut Ui, i18n: &I18n, other: &str, l: &Likely) {
    let country = arg(country_name(i18n, other));
    let (text, good) = match l {
        Likely::Accept => (
            i18n.tr_args("table-likely-accept", &[("country", country)]),
            true,
        ),
        Likely::Counter(q) => (
            i18n.tr_args(
                "table-likely-counter",
                &[
                    ("country", country),
                    ("n", arg(q.clauses.len().to_string())),
                    ("count", Arg::Num(q.clauses.len() as f64)),
                ],
            ),
            false,
        ),
        Likely::Refuse(term) => (
            i18n.tr_args(
                "table-likely-refuse",
                &[
                    ("country", country),
                    (
                        "term",
                        arg(term.map_or_else(
                            || i18n.tr("table-term-unknown"),
                            |t| i18n.tr(&format!("term-{}", t.key())),
                        )),
                    ),
                ],
            ),
            false,
        ),
    };
    ui.add(
        egui::Label::new(RichText::new(text).font(sans(size::BODY)).color(if good {
            color::POSITIVE
        } else {
            color::ACCENT
        }))
        .wrap(),
    );
}

fn warn(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(sans(size::SMALL))
                .color(color::DANGER),
        )
        .wrap(),
    );
}

/// A proposal received: what each side gives, the player's account, the deadline, and the three
/// answers: accept and refuse in two steps, counter on the table.
pub fn show_received(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    line: &OpenLine,
    st: &mut TableUi,
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::SM;
    let me = line.to.as_str();
    let other = line.from.as_str();
    bf_ui_kit::title(
        ui,
        &i18n.tr_args(
            "table-received-title",
            &[("country", arg(country_name(i18n, other)))],
        ),
    );
    if line.round > 1 {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args("table-round", &[("round", arg(line.round.to_string()))]),
        );
    }
    // The draft's orientation: the player as side A.
    let p = swapped(&line.weighed.proposal);
    let w = &line.weighed;
    let wide = ui.available_width() >= TWO_COLUMNS;
    let column = |ui: &mut Ui, mine: bool| {
        let giver = if mine { me } else { other };
        bf_ui_kit::heading(
            ui,
            &i18n.tr_args(
                "table-gives",
                &[("country", arg(country_name(i18n, giver)))],
            ),
        );
        let mut any = false;
        for (ci, c) in p.clauses.iter().enumerate() {
            if a_gives(c) != mine {
                continue;
            }
            any = true;
            read_only(ui, i18n, world, ci, c, w);
        }
        if !any {
            bf_ui_kit::small(ui, &i18n.tr("table-gives-nothing"));
        }
    };
    if wide {
        bf_ui_kit::columns(ui, 2, |cols| {
            column(&mut cols[0], true);
            column(&mut cols[1], false);
        });
    } else {
        column(ui, true);
        column(ui, false);
    }
    let term = match p.term_years {
        None => i18n.tr("table-term-none"),
        Some(y) => i18n.tr_args("table-years", &[("n", arg(y.to_string()))]),
    };
    bf_ui_kit::stat_row(ui, &i18n.tr("table-term"), &term);
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("table-notice"),
        &i18n.tr_args("table-years", &[("n", arg(p.notice_years.to_string()))]),
    );
    account(ui, i18n, me, &w.me);
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "table-answer-by",
            &[("date", arg(date_long(i18n, line.deadline)))],
        ),
    );
    if let Some((id, accept)) = st.confirm.filter(|(id, _)| *id == line.id) {
        bf_ui_kit::body(
            ui,
            &i18n.tr(if accept {
                "table-accept-confirm"
            } else {
                "table-refuse-confirm"
            }),
        );
        ui.horizontal(|ui| {
            let yes_label = i18n.tr(if accept {
                "table-accept-yes"
            } else {
                "table-refuse-yes"
            });
            let yes = if accept {
                Button::primary(&yes_label)
            } else {
                Button::danger(&yes_label)
            }
            .size(ButtonSize::Medium)
            .show(ui);
            if yes.clicked() || keys::single(ui, "treaty.answer.yes", 'Y', yes.rect) {
                out.treaty = Some(Sent::Answer(
                    id,
                    if accept {
                        ProposalAnswer::Accept
                    } else {
                        ProposalAnswer::Refuse
                    },
                ));
                st.confirm = None;
            }
            let no = Button::ghost(&i18n.tr("table-answer-back"))
                .size(ButtonSize::Medium)
                .show(ui);
            if no.clicked() || keys::single(ui, "treaty.answer.no", 'N', no.rect) {
                st.confirm = None;
            }
        });
        return;
    }
    ui.horizontal(|ui| {
        let accept = Button::primary(&i18n.tr("table-accept"))
            .size(ButtonSize::Medium)
            .show(ui);
        if accept.clicked() || keys::single(ui, "treaty.answer.accept", 'A', accept.rect) {
            st.confirm = Some((line.id, true));
        }
        let refuse = Button::secondary(&i18n.tr("table-refuse"))
            .size(ButtonSize::Medium)
            .show(ui);
        if refuse.clicked() || keys::single(ui, "treaty.answer.refuse", 'R', refuse.rect) {
            st.confirm = Some((line.id, false));
        }
        let may_counter = line.round < bf_sim::treaty_table::MAX_ROUNDS;
        let counter = Button::secondary(&i18n.tr("table-counter"))
            .size(ButtonSize::Medium)
            .enabled(may_counter)
            .show(ui);
        if may_counter
            && (counter.clicked() || keys::single(ui, "treaty.answer.counter", 'C', counter.rect))
        {
            *st = TableUi {
                draft: Some((other.to_owned(), p.clone())),
                counter: Some(line.id),
                ..TableUi::default()
            };
        }
    });
    if line.round >= bf_sim::treaty_table::MAX_ROUNDS {
        bf_ui_kit::small(ui, &i18n.tr("table-last-round"));
    }
}

/// A clause of a proposal received, without editing.
fn read_only(ui: &mut Ui, i18n: &I18n, world: Option<&World>, ci: usize, c: &Clause, w: &Weighed) {
    let key = c.kind.key();
    bf_ui_kit::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.strong(i18n.tr(&format!("clause-{key}")))
            .on_hover_text(bf_ui_kit::tooltip_text(
                &i18n.tr(&format!("clause-{key}-effect")),
            ));
        for (name, value) in bf_sim::treaty::params(&c.kind) {
            let shown = match name {
                "amount" | "principal" => money(i18n, f(value)),
                "yearly" => {
                    i18n.tr_args("table-per-year", &[("money", arg(money(i18n, f(value))))])
                }
                "years" => i18n.tr_args("table-years", &[("n", arg(i18n.int(f(value) as i64)))]),
                _ => pct(i18n, f(value) * 100.0, 1),
            };
            bf_ui_kit::stat_row(ui, &i18n.tr(&format!("table-param-{name}")), &shown);
        }
        revision_lines(ui, i18n, &c.kind);
        let names: Vec<String> = match &c.kind {
            ClauseKind::CedeStates { states }
            | ClauseKind::Demilitarized { states }
            | ClauseKind::Occupation { states } => states.clone(),
            ClauseKind::Base { at } => at.clone(),
            _ => Vec::new(),
        };
        if !names.is_empty() {
            let shown: Vec<String> = names
                .iter()
                .map(|k| {
                    world
                        .and_then(|w| w.states.iter().find(|s| s.def.key == *k))
                        .map_or_else(|| k.clone(), |s| crate::text::state_name(i18n, &s.def))
                })
                .collect();
            ui.add(egui::Label::new(shown.join(", ")).wrap());
        }
        if let Some(n) = w.notes.get(ci)
            && let Some(front) = n.dormant
        {
            bf_ui_kit::small(ui, &i18n.tr(&format!("front-{}", front.key())));
        }
        let mine = w.me.clauses.get(ci).copied().unwrap_or(0.0);
        if mine != 0.0 {
            bf_ui_kit::small(
                ui,
                &i18n.tr_args("table-clause-mine", &[("value", arg(signed(i18n, mine)))]),
            );
        }
    });
}

/// The line of a proposal of the player's in the journal's list: who, and where it stands.
pub fn closed_label(i18n: &I18n, me: &str, c: &crate::treaty_view::ClosedLine) -> String {
    let other = if c.from == me { &c.to } else { &c.from };
    let country = arg(country_name(i18n, other));
    let date = arg(date_long(i18n, c.date));
    match &c.outcome {
        Outcome::Signed(_) => i18n.tr_args(
            "table-closed-signed",
            &[("country", country), ("date", date)],
        ),
        Outcome::Refused(term) => {
            let who = if c.from == me {
                "table-closed-refused-them"
            } else {
                "table-closed-refused-me"
            };
            i18n.tr_args(
                who,
                &[
                    ("country", country),
                    ("date", date),
                    (
                        "term",
                        arg(term.map_or_else(
                            || i18n.tr("table-term-unknown"),
                            |t| i18n.tr(&format!("term-{}", t.key())),
                        )),
                    ),
                ],
            )
        }
        Outcome::Countered(_) => i18n.tr_args(
            "table-closed-countered",
            &[("country", country), ("date", date)],
        ),
        Outcome::Lapsed => i18n.tr_args(
            "table-closed-lapsed",
            &[("country", country), ("date", date)],
        ),
        Outcome::Failed(_) => i18n.tr_args(
            "table-closed-failed",
            &[("country", country), ("date", date)],
        ),
    }
}

/// The line of a proposal waiting for the other side.
pub fn outgoing_label(i18n: &I18n, l: &OpenLine) -> String {
    i18n.tr_args(
        "table-outgoing",
        &[
            ("country", arg(country_name(i18n, &l.to))),
            ("date", arg(date_long(i18n, l.sent))),
        ],
    )
}

/// The label of the button of a proposal received.
pub fn received_label(
    i18n: &I18n,
    l: &OpenLine,
    today: laplace_schelling::calendar::Date,
) -> String {
    let weeks = ((l.deadline.to_days() - today.to_days()).max(0) + 6) / 7;
    i18n.tr_args(
        "table-received",
        &[
            ("country", arg(country_name(i18n, &l.from))),
            ("weeks", arg(weeks.to_string())),
            ("count", Arg::Num(weeks as f64)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue() -> ClauseCatalogue {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        bf_sim::data::GameData::load(&root)
            .expect("game data")
            .clauses
            .expect("the clause catalogue")
    }

    #[test]
    fn every_draftable_clause_has_a_first_form_and_its_parameters_inside_the_limits() {
        let cat = catalogue();
        let gdp = 1.0e11;
        for key in DRAFTABLE {
            let kind = default_kind(&cat, key, gdp).unwrap_or_else(|| panic!("{key}"));
            assert_eq!(kind.key(), *key);
            let p = Proposal {
                side_a: vec!["GBR".into()],
                side_b: vec!["FRA".into()],
                clauses: vec![Clause {
                    kind,
                    giver: Side::Senior,
                    receiver: Side::Junior,
                    until: None,
                    note: None,
                }],
                term_years: None,
                notice_years: 0,
            };
            let gdps = [("GBR".to_owned(), Fixed::from_f64_lossy(gdp))]
                .into_iter()
                .collect();
            assert!(
                bf_sim::treaty::limit_violations(&cat, &p, &gdps).is_empty(),
                "{key}"
            );
        }
    }

    #[test]
    fn a_debt_keeps_its_annuity_with_its_terms() {
        let cat = catalogue();
        let mut kind = default_kind(&cat, "debt", 1.0e11).unwrap();
        let annuity = |k: &ClauseKind| match k {
            ClauseKind::Debt { terms: Some(t) } => f(t.annuity),
            _ => 0.0,
        };
        let before = annuity(&kind);
        assert!(before > 0.0);
        set_param(&mut kind, "years", Fixed::from_int(40));
        assert!(annuity(&kind) < before, "a longer debt pays less a year");
    }

    #[test]
    fn swapping_twice_gives_the_proposal_back() {
        let p = Proposal {
            side_a: vec!["FRA".into()],
            side_b: vec!["GBR".into()],
            clauses: vec![Clause {
                kind: ClauseKind::NonAggression,
                giver: Side::Senior,
                receiver: Side::Junior,
                until: None,
                note: None,
            }],
            term_years: Some(20),
            notice_years: 2,
        };
        let q = swapped(&p);
        assert_eq!(q.side_a, vec!["GBR".to_owned()]);
        assert_eq!(q.clauses[0].giver, Side::Junior);
        assert_eq!(swapped(&q), p);
    }
}
