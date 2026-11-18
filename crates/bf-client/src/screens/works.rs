//! Government tab: the public works' queue (D8, `docs/state.pt-BR.md`). The order the
//! construction points go in, the government's or arrival's; each work with its progress, the
//! weeks left at last week's pace and what holds it back; moving a work, cancelling one, adding
//! one the rule allows. Only public works: the government has no hand on the private ones
//! (owner's decision O2). "Only my list" is every category's priority at zero: the rule then
//! proposes no work of its own.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::levers::{Bottleneck, WorkKey};
use bf_sim::econ::state::PUBLIC_CATEGORIES;
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::{color, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{RichText, Ui};

use super::government::TabContext;
use crate::keys::{self, Tip};
use crate::text::money;
use crate::works_view::{WorkRow, WorksView};

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// What the works tab keeps between frames.
#[derive(Clone, Debug, Default)]
pub struct WorksUi {
    /// A cancellation waiting for its second click.
    pub cancel: Option<WorkKey>,
    /// The region whose buildings are shown for a new work.
    pub region: Option<u32>,
    /// The priorities before "only my list" set them to zero, to give back.
    pub saved_priority: Option<[u8; PUBLIC_CATEGORIES]>,
}

fn state_name(i18n: &I18n, world: Option<&crate::world::World>, state: u16) -> String {
    world
        .and_then(|w| w.states.get(usize::from(state).saturating_sub(1)))
        .map_or_else(|| "—".into(), |s| crate::text::state_name(i18n, &s.def))
}

fn bottleneck_key(b: Bottleneck) -> Option<&'static str> {
    match b {
        Bottleneck::None => None,
        Bottleneck::Cash => Some("works-held-cash"),
        Bottleneck::Cement => Some("works-held-cement"),
        Bottleneck::Steel => Some("works-held-steel"),
        Bottleneck::Price => Some("works-held-price"),
    }
}

/// The rows in the order shown: the order the player set this tick, then the simulation's.
fn rows_in_order<'a>(view: &'a WorksView, draft: Option<&[WorkKey]>) -> Vec<&'a WorkRow> {
    let mut rows: Vec<&WorkRow> = view.rows.iter().collect();
    if let Some(order) = draft {
        let mut sorted = Vec::with_capacity(rows.len());
        for key in order {
            if let Some(j) = rows.iter().position(|r| r.key == *key) {
                sorted.push(rows.remove(j));
            }
        }
        sorted.extend(rows);
        rows = sorted;
    }
    rows
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(view) = cx.snap.works.clone() else {
        bf_ui_kit::muted(ui, &i18n.tr(super::government::waiting_key(cx.snap)));
        return;
    };
    let world = cx.world;
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &i18n.tr("works-basis"));
    bf_ui_kit::gap(ui, space::SM);

    // The order: arrival (the rule) or the government's.
    let draft = cx.drafts.works_order.clone();
    let ordered = draft.as_ref().map_or(view.ordered, Option::is_some);
    let mut mode = ordered;
    let options = [
        (false, i18n.tr("works-order-arrival")),
        (true, i18n.tr("works-order-mine")),
    ];
    let tips = [
        Tip::Item('W', "works.order.arrival".into(), "arrival"),
        Tip::Item('W', "works.order.mine".into(), "mine"),
    ];
    let rows = rows_in_order(&view, draft.as_ref().and_then(|d| d.as_deref()));
    let keys_now: Vec<WorkKey> = rows.iter().map(|r| r.key).collect();
    let mut new_order: Option<Option<Vec<WorkKey>>> = None;
    if keys::segmented(ui, &mut mode, &options, &tips) {
        new_order = Some(mode.then(|| keys_now.clone()));
    }

    // "Only my list": every priority at zero, the rule proposes nothing.
    let priority = cx
        .drafts
        .priority
        .or_else(|| cx.view.map(|v| v.budget.priority))
        .unwrap_or_default();
    let mut only_mine = priority.iter().all(|w| *w == 0);
    ui.horizontal(|ui| {
        if bf_ui_kit::toggle(ui, &mut only_mine).changed() {
            let weights = if only_mine {
                cx.works.saved_priority = Some(priority);
                [0; PUBLIC_CATEGORIES]
            } else {
                cx.works
                    .saved_priority
                    .take()
                    .unwrap_or(bf_sim::econ::public::DEFAULT_PRIORITY)
            };
            cx.commands.push(GameCommand::SetPublicPriority {
                tag: cx.tag.to_owned(),
                weights,
            });
            cx.drafts.touch(cx.snap.tick);
            cx.drafts.priority = Some(weights);
        }
        ui.label(i18n.tr("works-only-mine"));
    });
    bf_ui_kit::gap(ui, space::SM);

    // The queue.
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args("works-queue", &[("n", arg(rows.len().to_string()))]),
    );
    if rows.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("works-none"));
    }
    let width = ui.available_width();
    let mut cancel: Option<WorkKey> = None;
    for (i, r) in rows.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.set_max_width(width);
            let name = i18n.tr_args(
                "works-row",
                &[
                    ("n", arg((i + 1).to_string())),
                    (
                        "building",
                        arg(i18n.tr(&format!("building-{}", r.building))),
                    ),
                    ("state", arg(state_name(i18n, world, r.state))),
                ],
            );
            ui.label(RichText::new(name).color(color::TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let down = Button::ghost("↓")
                    .size(ButtonSize::Small)
                    .enabled(i + 1 < rows.len())
                    .show(ui);
                let up = Button::ghost("↑")
                    .size(ButtonSize::Small)
                    .enabled(i > 0)
                    .show(ui);
                let id = |d: &str| format!("works.{d}.{i}");
                let moved = if i > 0
                    && (up.clicked()
                        || keys::item(ui, 'U', id("up"), &(i + 1).to_string(), up.rect))
                {
                    Some(i - 1)
                } else if i + 1 < rows.len()
                    && (down.clicked()
                        || keys::item(ui, 'J', id("down"), &(i + 1).to_string(), down.rect))
                {
                    Some(i + 1)
                } else {
                    None
                };
                if let Some(j) = moved {
                    let mut order = keys_now.clone();
                    order.swap(i, j);
                    new_order = Some(Some(order));
                }
            });
        });
        let mut parts = vec![i18n.tr_args(
            "works-progress",
            &[
                (
                    "done",
                    arg(format!("{} %", i18n.decimal(r.progress * 100.0, 0))),
                ),
                ("weeks", arg(r.weeks.to_string())),
            ],
        )];
        if let Some(c) = r.category {
            parts.push(i18n.tr_args(&format!("gov-cat-{c}"), &[("share", arg(String::new()))]));
        }
        if r.fixed {
            parts.push(i18n.tr("works-fixed"));
        }
        super::journal::bar(ui, r.progress as f32, color::ACCENT, width.min(360.0));
        bf_ui_kit::small(ui, &parts.join(" · ").replace(" ()", ""));
        if let Some(k) = bottleneck_key(r.bottleneck) {
            ui.label(
                RichText::new(i18n.tr(k))
                    .font(bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL))
                    .color(color::DANGER),
            );
        }
        ui.horizontal(|ui| {
            if cx.works.cancel == Some(r.key) {
                bf_ui_kit::small(ui, &i18n.tr("works-cancel-explain"));
                let yes = Button::danger(&i18n.tr("works-cancel-yes"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if yes.clicked() || keys::single(ui, "works.cancel.yes", 'Y', yes.rect) {
                    cancel = Some(r.key);
                }
                let no = Button::ghost(&i18n.tr("works-keep"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if no.clicked() || keys::single(ui, "works.cancel.no", 'N', no.rect) {
                    cx.works.cancel = None;
                }
            } else {
                let b = Button::ghost(&i18n.tr("works-cancel"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if b.clicked()
                    || keys::item(
                        ui,
                        'C',
                        format!("works.cancel.{i}"),
                        &(i + 1).to_string(),
                        b.rect,
                    )
                {
                    cx.works.cancel = Some(r.key);
                }
            }
        });
        bf_ui_kit::gap(ui, space::XS);
    }
    if let Some(work) = cancel {
        cx.commands.push(GameCommand::CancelWork {
            tag: cx.tag.to_owned(),
            work,
        });
        cx.works.cancel = None;
    }
    if let Some(order) = new_order {
        cx.commands.push(GameCommand::SetWorksOrder {
            tag: cx.tag.to_owned(),
            order: order.clone(),
        });
        cx.drafts.touch(cx.snap.tick);
        cx.drafts.works_order = Some(order);
    }

    // A new work: the region, then a building the rule allows there, with its cost.
    bf_ui_kit::gap(ui, space::MD);
    bf_ui_kit::heading(ui, &i18n.tr("works-new"));
    bf_ui_kit::small(ui, &i18n.tr("works-new-note"));
    if view.buildable.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("works-nothing-buildable"));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for (k, b) in view.buildable.iter().enumerate() {
            let picked = cx.works.region == Some(b.region);
            let label = state_name(i18n, world, b.state);
            let btn = if picked {
                Button::primary(&label)
            } else {
                Button::secondary(&label)
            }
            .size(ButtonSize::Small)
            .show(ui);
            if btn.clicked()
                || keys::item(
                    ui,
                    'R',
                    format!("works.region.{}", b.region),
                    &(k + 1).to_string(),
                    btn.rect,
                )
            {
                cx.works.region = Some(b.region);
            }
        }
    });
    let Some(b) = cx
        .works
        .region
        .and_then(|r| view.buildable.iter().find(|b| b.region == r))
        .cloned()
    else {
        return;
    };
    ui.horizontal_wrapped(|ui| {
        for (k, (kind, key, cost)) in b.kinds.iter().enumerate() {
            let label = i18n.tr_args(
                "works-add",
                &[
                    ("building", arg(i18n.tr(&format!("building-{key}")))),
                    ("cost", arg(money(i18n, *cost))),
                ],
            );
            let btn = Button::secondary(&label).size(ButtonSize::Small).show(ui);
            if btn.clicked()
                || keys::item(
                    ui,
                    'A',
                    format!("works.add.{kind}"),
                    &(k + 1).to_string(),
                    btn.rect,
                )
            {
                cx.commands.push(GameCommand::AddWork {
                    tag: cx.tag.to_owned(),
                    region: b.region,
                    kind: *kind,
                });
            }
        }
    });
}
