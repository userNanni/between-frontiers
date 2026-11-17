//! The decision "Libertar território", in the journal window (J): the player picks a country of
//! the catalogue whose land it holds, or one of its states; independent or as a subject; sees
//! what leaves and what stays; and confirms in a step of its own (`docs/borders.pt-BR.md`, "Onde
//! ficam as ações"). Rare and hard to undo, so never one click away.

use bf_i18n::{Arg, I18n};
use bf_sim::borders::op::LiberationOption;
use bf_ui_kit::tokens::space;
use bf_ui_kit::{Button, ButtonSize};
use egui::Ui;

use super::journal::{Liberate, Response};
use crate::keys;
use crate::sim::LiberationView;
use crate::text::{country_name, money};
use crate::world::World;

/// Highest number of options listed (the largest by people, catalogue countries first).
const LISTED: usize = 60;

fn option_name(i18n: &I18n, world: Option<&World>, o: &LiberationOption) -> String {
    match &o.tag {
        Some(tag) => country_name(i18n, tag),
        None => {
            let key = &o.states[0];
            world
                .and_then(|w| w.states.iter().find(|s| s.def.key == *key))
                .map_or_else(|| key.clone(), |s| crate::text::state_name(i18n, &s.def))
        }
    }
}

fn pct(i18n: &I18n, share: f64) -> String {
    i18n.tr_args(
        "gov-pct",
        &[("value", Arg::String(i18n.decimal(share * 100.0, 1)))],
    )
}

pub fn show(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    view: Option<&LiberationView>,
    state: &mut Liberate,
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::MD;
    bf_ui_kit::title(ui, &i18n.tr("liberate-title"));
    bf_ui_kit::small(ui, &i18n.tr("liberate-intro"));
    let Some(view) = view else {
        bf_ui_kit::muted(ui, &i18n.tr("journal-not-ready"));
        return;
    };
    if view.options.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("liberate-none"));
        return;
    }
    let picked = state.option.and_then(|i| view.options.get(i));

    if !state.confirming {
        // Step 1: what (left), how and what it costs (right).
        ui.columns(2, |cols| {
            let [left, right] = cols else { return };
            left.spacing_mut().item_spacing.y = space::SM;
            bf_ui_kit::caption(left, &i18n.tr("liberate-choose"));
            bf_ui_kit::small(left, &i18n.tr("liberate-choose-hint"));
            egui::ScrollArea::vertical()
                .id_salt("liberate-options")
                .auto_shrink([false, false])
                .show(left, |ui| {
                    for (i, o) in view.options.iter().enumerate().take(LISTED) {
                        let name = option_name(i18n, world, o);
                        let detail = i18n.tr_args(
                            "liberate-option-detail",
                            &[
                                ("states", Arg::Num(o.states.len() as f64)),
                                ("people", Arg::String(i18n.compact(o.preview.people as f64))),
                            ],
                        );
                        let b = if state.option == Some(i) {
                            Button::primary(&name)
                        } else {
                            Button::ghost(&name)
                        }
                        .detail(Some(&detail))
                        .size(ButtonSize::Small)
                        .width(f32::INFINITY)
                        .align_left()
                        .show(ui);
                        let key = o.tag.clone().unwrap_or_else(|| o.states[0].clone());
                        if b.clicked()
                            || keys::item(ui, 'L', format!("liberate.option.{key}"), &key, b.rect)
                        {
                            state.option = Some(i);
                        }
                    }
                });
            egui::ScrollArea::vertical()
                .id_salt("liberate-how")
                .auto_shrink([false, false])
                .show(right, |ui| {
                    ui.spacing_mut().item_spacing.y = space::MD;
                    bf_ui_kit::caption(ui, &i18n.tr("liberate-how"));
                    ui.horizontal(|ui| {
                        for (subject, key, letter) in [
                            (false, "liberate-independent", 'I'),
                            (true, "liberate-subject", 'S'),
                        ] {
                            let label = i18n.tr(key);
                            let b = if state.subject == subject {
                                Button::primary(&label)
                            } else {
                                Button::secondary(&label)
                            }
                            .size(ButtonSize::Small)
                            .show(ui);
                            if b.clicked()
                                || keys::single(ui, &format!("liberate.{key}"), letter, b.rect)
                            {
                                state.subject = subject;
                            }
                        }
                    });
                    let next = Button::secondary(&i18n.tr("liberate-review"))
                        .enabled(picked.is_some())
                        .size(ButtonSize::Medium)
                        .show(ui);
                    if picked.is_some()
                        && (next.clicked() || keys::single(ui, "liberate.review", 'R', next.rect))
                    {
                        state.confirming = true;
                    }
                    match picked {
                        Some(o) => consequences(ui, i18n, world, view, o, state.subject),
                        None => {
                            bf_ui_kit::small(ui, &i18n.tr("liberate-pick-first"));
                        }
                    }
                });
        });
        return;
    }

    // Step 2: the same consequences, and the decision itself.
    let Some(o) = picked else {
        state.confirming = false;
        return;
    };
    let name = option_name(i18n, world, o);
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args(
            if state.subject {
                "liberate-confirm-subject"
            } else {
                "liberate-confirm"
            },
            &[("name", Arg::String(name))],
        ),
    );
    consequences(ui, i18n, world, view, o, state.subject);
    bf_ui_kit::small(ui, &i18n.tr("liberate-final"));
    ui.horizontal(|ui| {
        let yes = Button::danger(&i18n.tr("liberate-yes"))
            .size(ButtonSize::Medium)
            .show(ui);
        if yes.clicked() || keys::single(ui, "liberate.yes", 'Y', yes.rect) {
            out.liberate = Some((o.states.clone(), o.tag.clone(), state.subject));
            *state = Liberate::default();
        }
        let no = Button::ghost(&i18n.tr("liberate-no"))
            .size(ButtonSize::Medium)
            .show(ui);
        if no.clicked() || keys::single(ui, "liberate.no", 'N', no.rect) {
            state.confirming = false;
        }
    });
}

/// What leaves with the land, what stays, and, as a subject, what the new country yields.
fn consequences(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    view: &LiberationView,
    o: &LiberationOption,
    subject: bool,
) {
    let p = &o.preview;
    let share = |a: i64, b: i64| if b > 0 { a as f64 / b as f64 } else { 0.0 };
    bf_ui_kit::caption(ui, &i18n.tr("liberate-leaves"));
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("liberate-people"),
        &format!(
            "{} ({})",
            i18n.compact(p.people as f64),
            pct(i18n, share(p.people, p.people_total))
        ),
    );
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("liberate-plants"),
        &format!(
            "{} ({})",
            i18n.int(p.levels),
            pct(i18n, share(p.levels, p.levels_total))
        ),
    );
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("liberate-fund"),
        &format!(
            "{} ({})",
            money(i18n, p.fund.to_f64_lossy()),
            pct(i18n, p.fund_share.to_f64_lossy())
        ),
    );
    let debt = p.debt.to_f64_lossy();
    bf_ui_kit::stat_row(
        ui,
        &i18n.tr("liberate-debt"),
        &if debt > 0.0 {
            money(i18n, debt)
        } else {
            i18n.tr("liberate-debt-stays")
        },
    );
    // The state list says something only when it is not the country itself under the map's
    // name of 1925 (Belarus is the state BLR, "Belorussiya").
    let whole = o.states.len() == 1 && o.tag.as_deref() == Some(o.states[0].as_str());
    if !whole {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "liberate-states",
                &[(
                    "names",
                    Arg::String(
                        o.states
                            .iter()
                            .map(|k| {
                                world
                                    .and_then(|w| w.states.iter().find(|s| s.def.key == *k))
                                    .map_or_else(
                                        || k.clone(),
                                        |s| crate::text::state_name(i18n, &s.def),
                                    )
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                )],
            ),
        );
    }
    if subject
        && let Some(b) = world
            .and_then(|w| w.clauses.as_ref())
            .and_then(|c| c.bundle_for(&bf_sim::scenario::SubjectKind::Protectorate))
    {
        let names = |give: bool| -> String {
            b.clauses
                .iter()
                .filter(|l| l.overlord_gives != give)
                .map(|l| i18n.tr(&format!("clause-{}", l.clause)).to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        };
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "borders-subject-detail",
                &[
                    ("gives", Arg::String(names(true))),
                    ("receives", Arg::String(names(false))),
                ],
            ),
        );
    }
    let _ = view;
}
