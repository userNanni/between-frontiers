//! Government tab: the works (D8, `docs/state.pt-BR.md`), laid out as four questions.
//!
//! 1. **How the queue goes**: works under way, what is left to pay, last week's pace and what
//!    holds the whole queue back (one bottleneck for the country: the builders lack cement or
//!    steel, the treasury pays late, or points are dear), each said once.
//! 2. **The public works**, one line each in the order the points go to them: the building, the
//!    state, progress, when it is ready at this pace, and move up, move down or cancel (two
//!    steps). The order is the rule's (arrival) or the government's; "only my list" stops the
//!    rule from proposing works of its own (every category's priority at zero).
//! 3. **The private works** of the same queue, read only: the government has no hand on them
//!    (owner's decision O2), but they take points from the same builders.
//! 4. **Ordering a work**: a building first (with how many states take it and from what cost),
//!    then the state, with what it has of that building already, the cost and when it would be
//!    ready.

use bf_i18n::{Arg, I18n};
use bf_sim::econ::levers::{Bottleneck, WorkKey};
use bf_sim::econ::state::PUBLIC_CATEGORIES;
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align, Layout, Rect, RichText, Sense, Ui, vec2};
use laplace_schelling::calendar::Date;

use super::government::TabContext;
use crate::keys::{self, Tip};
use crate::text::money;
use crate::works_view::{WorkRow, WorksView};

fn arg(s: String) -> Arg<'static> {
    Arg::String(s)
}

/// Height of a line of the lists.
const ROW: f32 = 34.0;
/// Width of the progress bar.
const BAR: f32 = 90.0;

/// What the works tab keeps between frames.
#[derive(Clone, Debug, Default)]
pub struct WorksUi {
    /// A cancellation waiting for its second click.
    pub cancel: Option<WorkKey>,
    /// The building picked for a new work.
    pub kind: Option<u16>,
    /// The priorities before "only my list" set them to zero, to give back.
    pub saved_priority: Option<[u8; PUBLIC_CATEGORIES]>,
    /// The private works' list is open.
    pub private_open: bool,
    /// Works asked for this month (region, building): they join the queue next month.
    pub asked: Vec<(u32, u16)>,
    /// Month `asked` belongs to.
    pub asked_month: (i32, u8),
}

fn state_name(i18n: &I18n, world: Option<&crate::world::World>, state: u16) -> String {
    world
        .and_then(|w| w.states.get(usize::from(state).saturating_sub(1)))
        .map_or_else(|| "—".into(), |s| crate::text::state_name(i18n, &s.def))
}

fn share(i18n: &I18n, x: f64) -> String {
    crate::text::pct_text(i18n, i18n.decimal(x * 100.0, 0))
}

/// "March 1926": the month a work `weeks` from `today` is ready.
fn ready(i18n: &I18n, today: Date, weeks: i64) -> String {
    let d = Date::from_days(today.to_days() + weeks.max(0) * 7);
    i18n.tr_args(
        "works-ready-month",
        &[
            ("month", arg(i18n.tr(&format!("month-{}", d.month)))),
            ("year", arg(d.year.to_string())),
        ],
    )
}

/// What holds the queue back, in a sentence; `None` when nothing does.
fn bottleneck_line(i18n: &I18n, v: &WorksView) -> Option<String> {
    let key = match v.bottleneck {
        Bottleneck::None => return None,
        Bottleneck::Cement => "works-held-cement",
        Bottleneck::Steel => "works-held-steel",
        Bottleneck::Price => {
            // `met` is the base price over today's: today's is its inverse.
            let over = if v.met > 0.0 { 1.0 / v.met } else { 0.0 };
            return Some(i18n.tr_args("works-held-price", &[("price", arg(share(i18n, over)))]));
        }
        Bottleneck::Cash => {
            return Some(i18n.tr_args(
                "works-held-cash",
                &[
                    ("paid", arg(share(i18n, v.met))),
                    (
                        "short",
                        arg(v.short.map_or_else(|| "—".into(), |m| money(i18n, m))),
                    ),
                ],
            ));
        }
    };
    Some(i18n.tr_args(key, &[("met", arg(share(i18n, v.met)))]))
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

/// A text cell of a line, its left edge at `x` and `w` wide; never shortened.
fn cell(ui: &mut Ui, top: f32, x: f32, w: f32, text: &str, tint: egui::Color32, right: bool) {
    let r = Rect::from_min_size(egui::pos2(x, top), vec2(w, ROW));
    let layout = if right {
        Layout::right_to_left(Align::Center)
    } else {
        Layout::left_to_right(Align::Center)
    };
    ui.scope_builder(egui::UiBuilder::new().max_rect(r).layout(layout), |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(text)
                    .font(bf_ui_kit::theme::sans(size::SMALL))
                    .color(tint),
            )
            .selectable(false)
            .wrap_mode(egui::TextWrapMode::Extend),
        );
    });
}

/// A name cell: shortened with an ellipsis when too long, whole on hover.
fn name_cell(ui: &mut Ui, top: f32, x: f32, w: f32, text: &str, tint: egui::Color32) {
    let r = Rect::from_min_size(egui::pos2(x, top), vec2(w, ROW));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(r)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(text)
                        .font(bf_ui_kit::theme::sans(size::SMALL))
                        .color(tint),
                )
                .selectable(false)
                .truncate(),
            );
        },
    );
}

/// Column widths of the public works' list: number, building, state, progress, ready, buttons.
struct Columns {
    num: f32,
    building: f32,
    state: f32,
    progress: f32,
    ready: f32,
}

impl Columns {
    const BUTTONS: f32 = 120.0;

    fn new(width: f32) -> Self {
        let num = 34.0;
        let progress = BAR + 56.0;
        let ready = 150.0;
        let rest = (width - num - progress - ready - Self::BUTTONS).max(200.0);
        Self {
            num,
            building: rest * 0.5,
            state: rest * 0.5,
            progress,
            ready,
        }
    }
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(view) = cx.snap.works.clone() else {
        bf_ui_kit::muted(ui, &i18n.tr(super::government::waiting_key(cx.snap)));
        return;
    };
    let world = cx.world;
    let today = cx.snap.date;
    if cx.works.asked_month != (today.year, today.month) {
        cx.works.asked.clear();
        cx.works.asked_month = (today.year, today.month);
    }
    ui.spacing_mut().item_spacing.y = space::XS;

    // 1. How the queue goes.
    let width = ui.available_width();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = space::XL;
        let stat = |ui: &mut Ui, label: &str, value: &str| {
            ui.vertical(|ui| {
                bf_ui_kit::small(ui, label);
                ui.label(
                    RichText::new(value)
                        .font(bf_ui_kit::theme::sans_semibold(size::BODY))
                        .color(color::TEXT),
                );
            });
        };
        stat(
            ui,
            &i18n.tr("works-stat-public"),
            &view.rows.len().to_string(),
        );
        stat(
            ui,
            &i18n.tr("works-stat-private"),
            &view.private.len().to_string(),
        );
        stat(
            ui,
            &i18n.tr("works-stat-left"),
            &money(i18n, view.cost_left()),
        );
        stat(ui, &i18n.tr("works-stat-pace"), &share(i18n, view.pace));
    });
    match bottleneck_line(i18n, &view) {
        Some(line) => {
            ui.label(
                RichText::new(line)
                    .font(bf_ui_kit::theme::sans(size::SMALL))
                    .color(color::DANGER),
            );
        }
        None => {
            bf_ui_kit::small(ui, &i18n.tr("works-held-none"));
        }
    }
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::divider(ui);

    // 2. The public works.
    let draft = cx.drafts.works_order.clone();
    let rows = rows_in_order(&view, draft.as_ref().and_then(|d| d.as_deref()));
    let keys_now: Vec<WorkKey> = rows.iter().map(|r| r.key).collect();
    let mut new_order: Option<Option<Vec<WorkKey>>> = None;
    bf_ui_kit::heading(
        ui,
        &i18n.tr_args("works-queue", &[("n", arg(rows.len().to_string()))]),
    );
    bf_ui_kit::small(ui, &i18n.tr("works-basis"));
    ui.horizontal_wrapped(|ui| {
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
        if keys::segmented(ui, &mut mode, &options, &tips) {
            new_order = Some(mode.then(|| keys_now.clone()));
        }
        ui.add_space(space::LG);
        // "Only my list": every priority at zero, the rule proposes nothing.
        let priority = cx
            .drafts
            .priority
            .or_else(|| cx.view.map(|v| v.budget.priority))
            .unwrap_or_default();
        let mut only_mine = priority.iter().all(|w| *w == 0);
        let t = bf_ui_kit::toggle(ui, &mut only_mine);
        let typed = keys::single(ui, "works.only_mine", 'L', t.rect);
        if typed {
            only_mine = !only_mine;
        }
        if t.changed() || typed {
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
        ui.label(
            RichText::new(i18n.tr("works-only-mine"))
                .font(bf_ui_kit::theme::sans(size::SMALL))
                .color(color::TEXT),
        );
    });
    bf_ui_kit::gap(ui, space::XS);

    let cols = Columns::new(width);
    if rows.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("works-none"));
    } else {
        // Header.
        let (head, _) = ui.allocate_exact_size(vec2(width, ROW), Sense::hover());
        let mut x = head.left() + cols.num;
        let muted = color::TEXT_MUTED;
        name_cell(
            ui,
            head.top(),
            x,
            cols.building,
            &i18n.tr("works-col-building"),
            muted,
        );
        x += cols.building;
        name_cell(
            ui,
            head.top(),
            x,
            cols.state,
            &i18n.tr("works-col-state"),
            muted,
        );
        x += cols.state;
        cell(
            ui,
            head.top(),
            x,
            cols.progress,
            &i18n.tr("works-col-progress"),
            muted,
            false,
        );
        x += cols.progress;
        cell(
            ui,
            head.top(),
            x,
            cols.ready,
            &i18n.tr("works-col-ready"),
            muted,
            false,
        );
    }
    let mut cancel: Option<WorkKey> = None;
    for (i, r) in rows.iter().enumerate() {
        let (line, _) = ui.allocate_exact_size(vec2(width, ROW), Sense::hover());
        if i % 2 == 0 {
            ui.painter()
                .rect_filled(line, 0.0, color::SURFACE_HOVER.gamma_multiply(0.5));
        }
        let top = line.top();
        let mut x = line.left();
        let tint = color::TEXT;
        cell(
            ui,
            top,
            x,
            cols.num - space::SM,
            &(i + 1).to_string(),
            color::TEXT_MUTED,
            true,
        );
        x += cols.num;
        let mut building = i18n.tr(&format!("building-{}", r.building));
        if r.fixed {
            building = i18n.tr_args("works-fixed", &[("building", arg(building))]);
        }
        name_cell(ui, top, x, cols.building - space::SM, &building, tint);
        x += cols.building;
        name_cell(
            ui,
            top,
            x,
            cols.state - space::SM,
            &state_name(i18n, world, r.state),
            tint,
        );
        x += cols.state;
        // Progress: a bar and the share done.
        let bar = Rect::from_min_size(egui::pos2(x, line.center().y - 3.0), vec2(BAR, 6.0));
        ui.painter().rect_filled(bar, 3.0, color::SURFACE_HOVER);
        ui.painter().rect_filled(
            Rect::from_min_size(bar.min, vec2(BAR * r.progress.clamp(0.0, 1.0) as f32, 6.0)),
            3.0,
            color::ACCENT,
        );
        cell(
            ui,
            top,
            x + BAR + space::SM,
            cols.progress - BAR - space::SM,
            &share(i18n, r.progress),
            tint,
            false,
        );
        x += cols.progress;
        let when = if r.stopped {
            i18n.tr("works-stopped-occupied")
        } else {
            ready(i18n, today, r.weeks)
        };
        cell(ui, top, x, cols.ready, &when, tint, false);
        let tip = i18n.tr_args(
            "works-ready-tip",
            &[
                ("weeks", arg(r.weeks.to_string())),
                ("full", arg(r.weeks_full.to_string())),
                ("cost", arg(money(i18n, r.cost_left))),
            ],
        );
        ui.interact(
            Rect::from_min_size(egui::pos2(x, top), vec2(cols.ready, ROW)),
            ui.make_persistent_id(("works-ready", i)),
            Sense::hover(),
        )
        .on_hover_text(tip);
        // Up, down, cancel.
        let buttons = Rect::from_min_max(
            egui::pos2(line.right() - Columns::BUTTONS, top),
            line.right_bottom(),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(buttons)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = space::XS;
                let id = |d: &str| format!("works.{d}.{i}");
                let n = (i + 1).to_string();
                let label = i18n.tr("works-cancel");
                let x_btn = bf_ui_kit::close_button(ui, &label, &label);
                if x_btn.clicked() || keys::item(ui, 'C', id("cancel"), &n, x_btn.rect) {
                    cx.works.cancel = Some(r.key);
                }
                let down = Button::ghost("↓")
                    .size(ButtonSize::Small)
                    .enabled(i + 1 < rows.len())
                    .show(ui);
                let up = Button::ghost("↑")
                    .size(ButtonSize::Small)
                    .enabled(i > 0)
                    .show(ui);
                let moved = if i > 0 && (up.clicked() || keys::item(ui, 'U', id("up"), &n, up.rect))
                {
                    Some(i - 1)
                } else if i + 1 < rows.len()
                    && (down.clicked() || keys::item(ui, 'J', id("down"), &n, down.rect))
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
            },
        );
        if cx.works.cancel == Some(r.key) {
            ui.horizontal_wrapped(|ui| {
                ui.add_space(cols.num);
                ui.label(
                    RichText::new(i18n.tr("works-cancel-explain"))
                        .font(bf_ui_kit::theme::sans(size::SMALL))
                        .color(color::DANGER),
                );
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
            });
        }
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

    // 3. The private works, read only.
    if !view.private.is_empty() {
        bf_ui_kit::gap(ui, space::SM);
        let label = i18n.tr_args(
            if cx.works.private_open {
                "works-private-hide"
            } else {
                "works-private-show"
            },
            &[("n", arg(view.private.len().to_string()))],
        );
        let b = Button::ghost(&label).size(ButtonSize::Small).show(ui);
        if b.clicked() || keys::single(ui, "works.private", 'P', b.rect) {
            cx.works.private_open = !cx.works.private_open;
        }
        if cx.works.private_open {
            bf_ui_kit::small(ui, &i18n.tr("works-private-note"));
            for p in &view.private {
                let (line, _) = ui.allocate_exact_size(vec2(width, ROW), Sense::hover());
                let top = line.top();
                let mut x = line.left() + cols.num;
                let tint = color::TEXT_MUTED;
                name_cell(
                    ui,
                    top,
                    x,
                    cols.building - space::SM,
                    &i18n.tr(&format!("building-{}", p.building)),
                    tint,
                );
                x += cols.building;
                name_cell(
                    ui,
                    top,
                    x,
                    cols.state - space::SM,
                    &state_name(i18n, world, p.state),
                    tint,
                );
                x += cols.state;
                cell(
                    ui,
                    top,
                    x,
                    cols.progress,
                    &share(i18n, p.progress),
                    tint,
                    false,
                );
                x += cols.progress;
                cell(
                    ui,
                    top,
                    x,
                    cols.ready,
                    &if p.stopped {
                        i18n.tr("works-stopped-occupied")
                    } else {
                        ready(i18n, today, p.weeks)
                    },
                    tint,
                    false,
                );
            }
        }
    }

    // 4. Ordering a work: the building, then the state.
    bf_ui_kit::gap(ui, space::MD);
    bf_ui_kit::divider(ui);
    bf_ui_kit::heading(ui, &i18n.tr("works-new"));
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "works-new-note",
            &[
                ("weeks", arg(view.new_weeks.to_string())),
                ("ready", arg(ready(i18n, today, view.new_weeks))),
            ],
        ),
    );
    if view.buildable.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("works-nothing-buildable"));
        return;
    }
    bf_ui_kit::caption(ui, &i18n.tr("works-new-step-building"));
    ui.horizontal_wrapped(|ui| {
        for (k, b) in view.buildable.iter().enumerate() {
            let picked = cx.works.kind == Some(b.kind);
            let cheapest = b.regions.first().map_or(0.0, |o| o.cost);
            let label = i18n.tr_args(
                "works-kind",
                &[
                    (
                        "building",
                        arg(i18n.tr(&format!("building-{}", b.building))),
                    ),
                    ("states", arg(b.regions.len().to_string())),
                    ("cost", arg(money(i18n, cheapest))),
                ],
            );
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
                    'K',
                    format!("works.kind.{}", b.building),
                    &(k + 1).to_string(),
                    btn.rect,
                )
            {
                cx.works.kind = if picked { None } else { Some(b.kind) };
            }
        }
    });
    let Some(b) = cx
        .works
        .kind
        .and_then(|k| view.buildable.iter().find(|b| b.kind == k))
    else {
        return;
    };
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::caption(
        ui,
        &i18n.tr_args(
            "works-new-step-state",
            &[(
                "building",
                arg(i18n.tr(&format!("building-{}", b.building))),
            )],
        ),
    );
    let state_w = (width - 3.0 * 130.0 - 170.0).max(180.0);
    let (head, _) = ui.allocate_exact_size(vec2(width, ROW), Sense::hover());
    let muted = color::TEXT_MUTED;
    let mut x = head.left();
    name_cell(
        ui,
        head.top(),
        x,
        state_w,
        &i18n.tr("works-col-state"),
        muted,
    );
    x += state_w;
    cell(
        ui,
        head.top(),
        x,
        120.0,
        &i18n.tr("works-col-has"),
        muted,
        true,
    );
    x += 130.0;
    cell(
        ui,
        head.top(),
        x,
        120.0,
        &i18n.tr("works-col-cost"),
        muted,
        true,
    );
    x += 130.0;
    cell(
        ui,
        head.top(),
        x + space::MD,
        120.0,
        &i18n.tr("works-col-ready"),
        muted,
        false,
    );
    for (k, o) in b.regions.iter().enumerate() {
        let (line, _) = ui.allocate_exact_size(vec2(width, ROW), Sense::hover());
        if k % 2 == 0 {
            ui.painter()
                .rect_filled(line, 0.0, color::SURFACE_HOVER.gamma_multiply(0.5));
        }
        let top = line.top();
        let mut x = line.left();
        let tint = color::TEXT;
        name_cell(
            ui,
            top,
            x,
            state_w - space::SM,
            &state_name(i18n, world, o.state),
            tint,
        );
        x += state_w;
        let has = if o.queued > 0 {
            i18n.tr_args(
                "works-levels-queued",
                &[
                    ("levels", arg(o.levels.to_string())),
                    ("queued", arg(o.queued.to_string())),
                ],
            )
        } else {
            o.levels.to_string()
        };
        cell(ui, top, x, 120.0, &has, tint, true);
        x += 130.0;
        cell(ui, top, x, 120.0, &money(i18n, o.cost), tint, true);
        x += 130.0;
        cell(
            ui,
            top,
            x + space::MD,
            120.0,
            &ready(i18n, today, view.new_weeks),
            tint,
            false,
        );
        let button = Rect::from_min_max(egui::pos2(line.right() - 160.0, top), line.right_bottom());
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(button)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                if cx.works.asked.contains(&(o.region, b.kind)) {
                    bf_ui_kit::small(ui, &i18n.tr("works-asked"));
                    return;
                }
                let btn = Button::secondary(&i18n.tr("works-order"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if btn.clicked()
                    || keys::item(
                        ui,
                        'A',
                        format!("works.add.{}", o.region),
                        &(k + 1).to_string(),
                        btn.rect,
                    )
                {
                    cx.commands.push(GameCommand::AddWork {
                        tag: cx.tag.to_owned(),
                        region: o.region,
                        kind: b.kind,
                    });
                    cx.works.asked.push((o.region, b.kind));
                }
            },
        );
    }
}
