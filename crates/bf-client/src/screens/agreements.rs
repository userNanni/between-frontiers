//! The agreements of the player's country, in the journal (`docs/treaties.pt-BR.md`, sections 9
//! and 13): those in force, each with its parties, year, term, notice and the clauses each side
//! gives; the breaches that touch the country and the causes it holds or faces; those ended;
//! and the two reputations of every country. Denouncing starts here: with notice the agreement
//! ends when the notice runs out, without a breach; without notice it is a breach of every
//! clause the country gives, asked in two steps. A restore point is written before either.

use bf_i18n::{Arg, I18n};
use bf_sim::treaty::BreachCause;
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Ui};

use super::budget::pct;
use super::journal::Response;
use crate::agreements_view::{AgreementLine, AgreementsView};
use crate::keys;
use crate::text::{country_name, date_long, money};

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// The denunciation being weighed: the agreement, and whether its last step (the breach) is
/// asked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AgreementsUi {
    pub denounce: Option<(u32, bool)>,
}

fn names(i18n: &I18n, tags: &[String]) -> String {
    tags.iter()
        .map(|t| country_name(i18n, t))
        .collect::<Vec<_>>()
        .join(", ")
}

/// An agreement's name: the treaty's, or "Agreement of {year}" for one signed in play.
pub fn name(i18n: &I18n, a: &AgreementLine) -> String {
    if a.key.starts_with("agreement-") {
        i18n.tr_args("agreements-in-play", &[("year", arg(a.signed.to_string()))])
    } else {
        super::ledger::treaty_name(i18n, &a.key)
    }
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

pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    view: &AgreementsView,
    st: &mut AgreementsUi,
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::SM;
    bf_ui_kit::title(ui, &i18n.tr("agreements-title"));
    bf_ui_kit::small(ui, &i18n.tr("agreements-basis"));
    // In force.
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            "agreements-in-force",
            &[("n", arg(view.in_force.len().to_string()))],
        ),
    );
    if view.in_force.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("agreements-none"));
    }
    for (k, a) in view.in_force.iter().enumerate() {
        agreement(ui, i18n, view, a, k, st, out);
    }
    // Breaches and causes.
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("agreements-breaches"));
    if view.breaches.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("agreements-breaches-none"));
    }
    for b in &view.breaches {
        let clause = b
            .clause
            .map_or_else(String::new, |c| i18n.tr(&format!("clause-{c}")));
        let line = i18n.tr_args(
            "agreements-breach",
            &[
                ("date", arg(date_long(i18n, b.date))),
                ("by", arg(country_name(i18n, &b.by))),
                ("clause", arg(clause)),
                (
                    "treaty",
                    arg(super::ledger::treaty_name(i18n, &b.agreement)),
                ),
                ("harmed", arg(names(i18n, &b.harmed))),
                ("cause", arg(i18n.tr(cause_key(b.cause)))),
                ("value", arg(value(i18n, b.value))),
            ],
        );
        ui.add(egui::Label::new(RichText::new(line).font(sans(size::SMALL))).wrap());
    }
    for (other, worth, held) in &view.causes {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                if *held {
                    "agreements-cause-held"
                } else {
                    "agreements-cause-faced"
                },
                &[
                    ("country", arg(country_name(i18n, other))),
                    ("value", arg(value(i18n, *worth))),
                ],
            ),
        );
    }
    // Reputation.
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::heading(ui, &i18n.tr("agreements-reputation"));
    if !view.reputation_on {
        bf_ui_kit::small(ui, &i18n.tr("agreements-reputation-off"));
    } else {
        bf_ui_kit::small(ui, &i18n.tr("agreements-reputation-basis"));
        egui::Grid::new("agreements-reputation")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for k in [
                    "agreements-col-country",
                    "agreements-col-general",
                    "agreements-col-security",
                ] {
                    ui.strong(i18n.tr(k));
                }
                ui.end_row();
                let share = |x: f64, gdp: f64| {
                    if gdp > 0.0 {
                        i18n.tr_args(
                            "agreements-reputation-value",
                            &[
                                ("money", arg(money(i18n, x))),
                                ("share", arg(pct(i18n, x / gdp * 100.0, 1))),
                            ],
                        )
                    } else {
                        money(i18n, x)
                    }
                };
                for r in &view.reputation {
                    let mut name = country_name(i18n, &r.tag);
                    if r.tag == view.me {
                        name = i18n.tr_args("agreements-us", &[("country", arg(name))]);
                    }
                    ui.label(name);
                    ui.label(share(r.general, r.gdp));
                    ui.label(share(r.security, r.gdp));
                    ui.end_row();
                }
            });
    }
    // Ended.
    if !view.ended.is_empty() {
        bf_ui_kit::gap(ui, space::SM);
        bf_ui_kit::heading(ui, &i18n.tr("agreements-ended"));
        for a in &view.ended {
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "agreements-ended-line",
                    &[
                        ("name", arg(name(i18n, a))),
                        (
                            "parties",
                            arg(names(i18n, &[a.side_a.clone(), a.side_b.clone()].concat())),
                        ),
                        ("signed", arg(a.signed.to_string())),
                        (
                            "ended",
                            arg(a.ends.map_or_else(String::new, |y| y.to_string())),
                        ),
                    ],
                ),
            );
        }
    }
}

/// A breach's or a cause's value; the table prices clauses for the harmed later, until then it
/// is zero and says so.
fn value(i18n: &I18n, x: f64) -> String {
    if x > 0.0 {
        money(i18n, x)
    } else {
        i18n.tr("agreements-value-unmeasured")
    }
}

fn cause_key(c: BreachCause) -> &'static str {
    match c {
        BreachCause::CallUnanswered { .. } => "breach-call-unanswered",
        BreachCause::ServiceUnpaid => "breach-service-unpaid",
        BreachCause::TestFailed => "breach-test-failed",
        BreachCause::Repudiated => "breach-repudiated",
        BreachCause::Attacked { .. } => "breach-attacked",
        BreachCause::Denounced => "breach-denounced",
    }
}

fn agreement(
    ui: &mut Ui,
    i18n: &I18n,
    view: &AgreementsView,
    a: &AgreementLine,
    k: usize,
    st: &mut AgreementsUi,
    out: &mut Response,
) {
    bf_ui_kit::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.strong(name(i18n, a));
        // A multilateral agreement of equals has every party on one side.
        let parties = if a.side_b.is_empty() {
            i18n.tr_args(
                "agreements-parties-all",
                &[
                    ("a", arg(names(i18n, &a.side_a))),
                    ("year", arg(a.signed.to_string())),
                ],
            )
        } else {
            i18n.tr_args(
                "agreements-parties",
                &[
                    ("a", arg(names(i18n, &a.side_a))),
                    ("b", arg(names(i18n, &a.side_b))),
                    ("year", arg(a.signed.to_string())),
                ],
            )
        };
        bf_ui_kit::small(ui, &parties);
        let term = match (a.term_years, a.ends) {
            (Some(t), Some(end)) => i18n.tr_args(
                "agreements-term",
                &[("years", arg(t.to_string())), ("end", arg(end.to_string()))],
            ),
            _ => i18n.tr("agreements-term-none"),
        };
        let notice = if a.notice_years > 0 {
            i18n.tr_args(
                "agreements-notice",
                &[("years", arg(a.notice_years.to_string()))],
            )
        } else {
            i18n.tr("agreements-notice-none")
        };
        bf_ui_kit::small(ui, &format!("{term} · {notice}"));
        // The clauses, by who gives them.
        for c in &a.clauses {
            let clause = arg(i18n.tr(&format!("clause-{}", c.key)));
            // Every party to every other: said once.
            let mut line = if c.givers == c.receivers {
                i18n.tr_args("agreements-clause-mutual", &[("clause", clause)])
            } else {
                i18n.tr_args(
                    "agreements-clause",
                    &[
                        ("clause", clause),
                        ("givers", arg(names(i18n, &c.givers))),
                        ("receivers", arg(names(i18n, &c.receivers))),
                    ],
                )
            };
            if let Some(u) = &c.until {
                line = i18n.tr_args(
                    "agreements-clause-until",
                    &[("line", arg(line)), ("until", arg(u.clone()))],
                );
            }
            ui.add(
                egui::Label::new(
                    RichText::new(line)
                        .font(sans(size::SMALL))
                        .color(if c.mine {
                            color::TEXT
                        } else {
                            color::TEXT_MUTED
                        }),
                )
                .wrap(),
            )
            .on_hover_text(bf_ui_kit::tooltip_text(
                &i18n.tr(&format!("clause-{}-effect", c.key)),
            ));
        }
        denounce(ui, i18n, view, a, k, st, out);
    });
}

/// "Denunciar…": with notice, one confirmation; without, the breach said and asked again.
fn denounce(
    ui: &mut Ui,
    i18n: &I18n,
    view: &AgreementsView,
    a: &AgreementLine,
    k: usize,
    st: &mut AgreementsUi,
    out: &mut Response,
) {
    if view.pending.contains(&a.id) {
        bf_ui_kit::small(ui, &i18n.tr("agreements-denounce-sent"));
        return;
    }
    if !view.can_denounce {
        Button::ghost(&i18n.tr("agreements-denounce"))
            .size(ButtonSize::Small)
            .enabled(false)
            .show(ui)
            .on_disabled_hover_text(i18n.tr("agreements-denounce-off"));
        return;
    }
    let asked = st.denounce.filter(|(id, _)| *id == a.id);
    let Some((_, last)) = asked else {
        let b = Button::ghost(&i18n.tr("agreements-denounce"))
            .size(ButtonSize::Small)
            .show(ui);
        if b.clicked()
            || keys::item(
                ui,
                'U',
                format!("agreements.denounce.{}", a.id),
                &(k + 1).to_string(),
                b.rect,
            )
        {
            st.denounce = Some((a.id, false));
        }
        return;
    };
    let given: Vec<String> = a
        .given()
        .map(|c| i18n.tr(&format!("clause-{}", c.key)))
        .collect();
    if a.notice_years > 0 {
        // With notice: it ends when the notice runs out, without a breach.
        let year = view_year(view) + i32::from(a.notice_years);
        bf_ui_kit::body(
            ui,
            &i18n.tr_args(
                "agreements-denounce-notice",
                &[
                    ("years", arg(a.notice_years.to_string())),
                    ("year", arg(year.to_string())),
                ],
            ),
        );
    } else if !last {
        warn(
            ui,
            &i18n.tr_args(
                "agreements-denounce-breach",
                &[(
                    "clauses",
                    arg(if given.is_empty() {
                        i18n.tr("agreements-denounce-nothing-given")
                    } else {
                        given.join(", ")
                    }),
                )],
            ),
        );
    } else {
        warn(ui, &i18n.tr("agreements-denounce-breach-last"));
    }
    ui.horizontal(|ui| {
        let (label, final_step) = if a.notice_years > 0 {
            (i18n.tr("agreements-denounce-yes"), true)
        } else if !last {
            (i18n.tr("agreements-denounce-breach-next"), false)
        } else {
            (i18n.tr("agreements-denounce-breach-yes"), true)
        };
        let yes = Button::danger(&label).size(ButtonSize::Medium).show(ui);
        if yes.clicked() || keys::single(ui, "agreements.denounce.yes", 'Y', yes.rect) {
            if final_step {
                out.denounce = Some(a.id);
                st.denounce = None;
            } else {
                st.denounce = Some((a.id, true));
            }
        }
        let no = Button::ghost(&i18n.tr("agreements-keep"))
            .size(ButtonSize::Medium)
            .show(ui);
        if no.clicked() || keys::single(ui, "agreements.denounce.no", 'N', no.rect) {
            st.denounce = None;
        }
    });
}

/// The year the view was read in.
fn view_year(view: &AgreementsView) -> i32 {
    view.signature.5
}
