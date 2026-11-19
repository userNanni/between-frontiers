//! The economy window: what the player's country has built (the buildings tab). Every building
//! kind with its levels and the levels under construction, workers, a year's output and profit,
//! use, and in how many states; filtered by sector, sorted by any column; a kind opens on its
//! states, one line each with the method the plant runs. Read only, from the monthly snapshot.

use bf_i18n::{Arg, I18n};
use bf_sim::economy::BUILDING_GROUPS;
use bf_ui_kit::tokens::{color, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align, Layout, Rect, RichText, Sense, Ui, vec2};

use super::region;
use crate::buildings_view::{BuildingsView, KindRow, StateRow};
use crate::keys::{self, Tip, id};
use crate::text::money_short;

const WINDOW: egui::Vec2 = egui::vec2(1100.0, 700.0);
const MIN_WINDOW: egui::Vec2 = egui::vec2(760.0, 380.0);
const ROW: f32 = 26.0;
/// Width of each figure column, between these; the name takes what is left.
const COL_MIN: f32 = 84.0;
const COL_MAX: f32 = 110.0;
/// Width the name column keeps before the figure columns narrow.
const NAME_MIN: f32 = 200.0;
const SCROLL_GUTTER: f32 = 14.0;

/// The window's tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Buildings,
}

/// A column the buildings table sorts by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Column {
    Name,
    #[default]
    Levels,
    Workers,
    Output,
    Use,
    Profit,
    States,
}

impl Column {
    const FIGURES: [Column; 6] = [
        Column::Levels,
        Column::Workers,
        Column::Output,
        Column::Use,
        Column::Profit,
        Column::States,
    ];

    fn key(self) -> &'static str {
        match self {
            Column::Name => "economy-col-building",
            Column::Levels => "economy-col-levels",
            Column::Workers => "economy-col-workers",
            Column::Output => "economy-col-output",
            Column::Use => "economy-col-use",
            Column::Profit => "economy-col-profit",
            Column::States => "economy-col-states",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Column::Name => "name",
            Column::Levels => "levels",
            Column::Workers => "workers",
            Column::Output => "output",
            Column::Use => "use",
            Column::Profit => "profit",
            Column::States => "states",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Window {
    pub tab: Tab,
    /// The sector shown (`BUILDING_GROUPS` index); `None` for all.
    pub group: Option<u8>,
    pub sort: Column,
    /// Smallest first.
    pub ascending: bool,
    /// The kind opened on its states.
    pub open: Option<u16>,
    /// A state line was clicked: the map state id to show (the game selects it and closes the
    /// window).
    pub goto: Option<u16>,
}

#[derive(Default)]
pub struct Response {
    pub close: bool,
    pub place_done: bool,
}

fn pct(i18n: &I18n, x: f64) -> String {
    crate::text::pct_text(i18n, i18n.decimal(x * 100.0, 0))
}

fn count(i18n: &I18n, x: f64) -> String {
    i18n.compact_short(x)
}

fn levels_text(i18n: &I18n, levels: i64, building: i64) -> String {
    if building > 0 {
        i18n.tr_args(
            "economy-levels-building",
            &[
                ("levels", Arg::String(levels.to_string())),
                ("building", Arg::String(building.to_string())),
            ],
        )
    } else {
        levels.to_string()
    }
}

fn sort_value(k: &KindRow, c: Column) -> f64 {
    match c {
        Column::Name => 0.0,
        Column::Levels => k.levels as f64 + k.building as f64 * 1e-6,
        Column::Workers => k.workers,
        Column::Output => k.output,
        Column::Use => k.utilization,
        Column::Profit => k.profit,
        Column::States => k.states.len() as f64,
    }
}

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    world: Option<&crate::world::World>,
    view: Option<&BuildingsView>,
    window: &mut Window,
    place: &mut crate::hud::WindowPlace,
) -> Response {
    let mut out = Response::default();
    let top = screen.top() + super::top_bar::LEFT_BOTTOM;
    let size = WINDOW.min(vec2(
        screen.width() - 32.0,
        screen.bottom() - top - space::LG,
    ));
    let default = Rect::from_min_size(egui::pos2(screen.center().x - size.x / 2.0, top), size);
    let (rect, moved) = super::window_frame(ui, "economy", default, screen, place, MIN_WINDOW);
    out.place_done |= moved;
    region(ui, rect.shrink(space::XL), |ui| {
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("economy-title"));
            ui.add_space(space::MD);
            let options = vec![(Tab::Buildings, i18n.tr("economy-tab-buildings"))];
            let tips = [Tip::Item('Z', "economy.tab.buildings".into(), "1")];
            keys::segmented(ui, &mut window.tab, &options, &tips);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = i18n.tr("gov-close");
                let tip = keys::tip(ui.ctx(), &label, id::BACK);
                let close = bf_ui_kit::close_button(ui, &label, &tip);
                if close.clicked() || keys::single(ui, "economy.close", 'X', close.rect) {
                    out.close = true;
                }
            });
        });
        bf_ui_kit::divider(ui);
        let Some(v) = view else {
            bf_ui_kit::muted(ui, &i18n.tr("economy-waiting"));
            return;
        };
        match window.tab {
            Tab::Buildings => buildings(ui, i18n, world, v, window),
        }
    });
    out.place_done |= super::window_grip(ui, "economy", rect, place, MIN_WINDOW);
    out
}

fn buildings(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&crate::world::World>,
    v: &BuildingsView,
    window: &mut Window,
) {
    ui.spacing_mut().item_spacing.y = space::XS;
    bf_ui_kit::small(ui, &i18n.tr("economy-basis"));
    bf_ui_kit::gap(ui, space::SM);

    // Sectors: all, then each one with its levels.
    let totals = v.groups();
    ui.horizontal_wrapped(|ui| {
        let all = i18n.tr("economy-group-all");
        let mut chip = |ui: &mut Ui, label: String, group: Option<u8>, key: String| {
            let on = window.group == group;
            let b = if on {
                Button::primary(&label)
            } else {
                Button::secondary(&label)
            }
            .size(ButtonSize::Small)
            .show(ui);
            let id = format!(
                "economy.group.{}",
                group.map_or("all", |g| BUILDING_GROUPS[usize::from(g)])
            );
            if b.clicked() || keys::item(ui, 'S', id, &key, b.rect) {
                window.group = group;
                window.open = None;
            }
        };
        chip(ui, all, None, "0".into());
        for (g, key) in BUILDING_GROUPS.iter().enumerate() {
            let levels = totals[g][0];
            if levels <= 0.0 && !v.kinds.iter().any(|k| usize::from(k.group) == g) {
                continue;
            }
            let label = i18n.tr_args(
                "economy-group-chip",
                &[
                    (
                        "group",
                        Arg::String(i18n.tr(&format!("building-group-{key}"))),
                    ),
                    ("levels", Arg::String(count(i18n, levels))),
                ],
            );
            chip(ui, label, Some(g as u8), (g + 1).to_string());
        }
    });
    bf_ui_kit::gap(ui, space::SM);

    let mut rows: Vec<&KindRow> = v
        .kinds
        .iter()
        .filter(|k| window.group.is_none_or(|g| g == k.group))
        .collect();
    let name = |k: &KindRow| i18n.tr(&format!("building-{}", k.key));
    match window.sort {
        Column::Name => rows.sort_by_key(|k| name(k)),
        c => rows.sort_by(|a, b| {
            sort_value(a, c)
                .partial_cmp(&sort_value(b, c))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.kind.cmp(&b.kind))
        }),
    }
    if !window.ascending {
        rows.reverse();
    }

    // Header: each column sorts; again flips the order.
    // Room on the right for the list's scroll bar.
    let width = ui.available_width() - SCROLL_GUTTER;
    let n = Column::FIGURES.len() as f32;
    let col = ((width - NAME_MIN) / n).clamp(COL_MIN, COL_MAX);
    let name_w = width - col * n;
    let header = |ui: &mut Ui, window: &mut Window| {
        let (rect, _) = ui.allocate_exact_size(vec2(name_w + col * n, ROW), Sense::hover());
        let mut x = rect.left();
        for (i, c) in std::iter::once(Column::Name)
            .chain(Column::FIGURES)
            .enumerate()
        {
            let w = if c == Column::Name { name_w } else { col };
            let arrow = if window.sort == c {
                if window.ascending { " ↑" } else { " ↓" }
            } else {
                ""
            };
            let text = format!("{}{arrow}", i18n.tr(c.key()));
            let (left, right) = if c == Column::Name {
                (x + space::XS, false)
            } else {
                (x, true)
            };
            let cell_rect =
                Rect::from_min_size(egui::pos2(left, rect.top()), vec2(w - space::SM, ROW));
            let layout = if right {
                Layout::right_to_left(Align::Center)
            } else {
                Layout::left_to_right(Align::Center)
            };
            let r = ui
                .scope_builder(
                    egui::UiBuilder::new().max_rect(cell_rect).layout(layout),
                    |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(text)
                                    .font(bf_ui_kit::theme::sans_semibold(size::SMALL))
                                    .color(color::TEXT_MUTED),
                            )
                            .sense(Sense::click())
                            .selectable(false)
                            .wrap_mode(egui::TextWrapMode::Extend),
                        )
                    },
                )
                .inner
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            let typed = keys::item(
                ui,
                'O',
                format!("economy.sort.{}", c.id()),
                &(i + 1).to_string(),
                r.rect,
            );
            if r.clicked() || typed {
                if window.sort == c {
                    window.ascending = !window.ascending;
                } else {
                    window.sort = c;
                    window.ascending = c == Column::Name;
                }
            }
            x += w;
        }
    };
    header(ui, window);
    bf_ui_kit::divider(ui);

    if rows.is_empty() {
        bf_ui_kit::muted(ui, &i18n.tr("economy-none"));
        return;
    }
    egui::ScrollArea::vertical()
        .id_salt("economy-buildings")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            super::scroll_keys(ui);
            ui.spacing_mut().item_spacing.y = 0.0;
            for (i, k) in rows.iter().enumerate() {
                let open = window.open == Some(k.kind);
                let figures = [
                    levels_text(i18n, k.levels, k.building),
                    count(i18n, k.workers),
                    money_short(i18n, k.output),
                    if k.levels > 0 {
                        pct(i18n, k.utilization)
                    } else {
                        "—".into()
                    },
                    if k.public {
                        "—".into()
                    } else {
                        money_short(i18n, k.profit)
                    },
                    k.states.len().to_string(),
                ];
                let marker = if open { "− " } else { "+ " };
                let r = line(
                    ui,
                    (name_w, col),
                    &format!("{marker}{}", name(k)),
                    &figures,
                    k.profit < 0.0 && !k.public,
                    color::TEXT,
                    if i % 2 == 1 {
                        Some(color::SURFACE_HOVER)
                    } else {
                        None
                    },
                );
                let typed = keys::item(
                    ui,
                    'E',
                    format!("economy.kind.{}", k.key),
                    &(i + 1).to_string(),
                    r.rect,
                );
                if r.clicked() || typed {
                    window.open = if open { None } else { Some(k.kind) };
                }
                if open {
                    window.goto = states(ui, i18n, world, k, (name_w, col)).or(window.goto);
                }
            }
        });
}

/// One line of the table: the name, then the figures right-aligned in their columns; a click
/// anywhere on it is the response's.
fn line(
    ui: &mut Ui,
    (name_w, col): (f32, f32),
    name: &str,
    figures: &[String],
    loss: bool,
    text: egui::Color32,
    stripe: Option<egui::Color32>,
) -> egui::Response {
    let width = name_w + col * figures.len() as f32;
    let (rect, response) = ui.allocate_exact_size(vec2(width, ROW), Sense::click());
    if let Some(fill) = stripe {
        ui.painter()
            .rect_filled(rect, 0.0, fill.gamma_multiply(0.5));
    }
    if response.hovered() {
        ui.painter().rect_filled(rect, 0.0, color::SURFACE_HOVER);
    }
    let mut x = rect.left();
    let name_cell = cell(
        ui,
        Rect::from_min_size(
            egui::pos2(x + space::XS, rect.top()),
            vec2(name_w - space::SM, ROW),
        ),
        name,
        text,
        false,
    );
    x += name_w;
    for (j, s) in figures.iter().enumerate() {
        // The profit column (fifth) shows a loss in red.
        let c = if j == 4 && loss { color::DANGER } else { text };
        cell(
            ui,
            Rect::from_min_size(egui::pos2(x, rect.top()), vec2(col - space::SM, ROW)),
            s,
            c,
            true,
        );
        x += col;
    }
    let name_cell = name_cell.on_hover_cursor(egui::CursorIcon::PointingHand);
    response | name_cell
}

/// One cell's text; the name cell (left-aligned) also takes clicks, so the line can be
/// clicked by its name.
fn cell(ui: &mut Ui, r: Rect, s: &str, c: egui::Color32, right: bool) -> egui::Response {
    let layout = if right {
        Layout::right_to_left(Align::Center)
    } else {
        Layout::left_to_right(Align::Center)
    };
    ui.scope_builder(egui::UiBuilder::new().max_rect(r).layout(layout), |ui| {
        let label = egui::Label::new(
            RichText::new(s)
                .font(bf_ui_kit::theme::sans(size::SMALL))
                .color(c),
        )
        .selectable(false);
        // Figures never shorten; a name too long for its column ends in an ellipsis and shows
        // whole on hover.
        if right {
            ui.add(label.wrap_mode(egui::TextWrapMode::Extend))
        } else {
            ui.add(
                label
                    .wrap_mode(egui::TextWrapMode::Truncate)
                    .sense(Sense::click()),
            )
        }
    })
    .inner
}

/// The kind's states, under its line: the state and the method its plant runs, then the same
/// figures.
fn states(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&crate::world::World>,
    k: &KindRow,
    widths: (f32, f32),
) -> Option<u16> {
    let mut goto = None;
    for (j, s) in k.states.iter().enumerate() {
        let state = world
            .and_then(|w| w.states.get(usize::from(s.state).saturating_sub(1)))
            .map_or_else(|| "—".into(), |st| crate::text::state_name(i18n, &st.def));
        let label = if s.method.is_empty() {
            format!("    {state}")
        } else {
            format!("    {state} · {}", i18n.tr(&format!("method-{}", s.method)))
        };
        let r = line(
            ui,
            widths,
            &label,
            &state_figures(i18n, k, s),
            s.profit < 0.0 && !k.public,
            color::TEXT_MUTED,
            None,
        )
        .on_hover_text(i18n.tr("economy-goto-state"));
        let typed = keys::item(
            ui,
            'T',
            format!("economy.state.{}", s.state),
            &(j + 1).to_string(),
            r.rect,
        );
        if r.clicked() || typed {
            goto = Some(s.state);
        }
    }
    bf_ui_kit::gap(ui, space::XS);
    goto
}

fn state_figures(i18n: &I18n, k: &KindRow, s: &StateRow) -> [String; 6] {
    [
        levels_text(i18n, s.levels, s.building),
        count(i18n, s.workers),
        money_short(i18n, s.output),
        if s.levels > 0 {
            pct(i18n, s.utilization)
        } else {
            "—".into()
        },
        if k.public {
            "—".into()
        } else {
            money_short(i18n, s.profit)
        },
        String::new(),
    ]
}
