//! The influence window: over whom the player's country weighs, and who weighs on it, issue by
//! issue, and where each influence comes from (credit, investment, trade, protection,
//! goodwill). Read only, from the monthly snapshot. Shares of the influenced country's GDP.

use bf_i18n::{Arg, I18n};
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, radius, size, space};
use egui::{Rect, RichText, Sense, Ui, vec2};

use super::region;
use crate::influence_view::{ISSUES, InfluenceView, Pair};
use crate::keys::{self, Tip, id};
use crate::text::country_name;

const WINDOW: egui::Vec2 = egui::vec2(1040.0, 660.0);
const MIN_WINDOW: egui::Vec2 = egui::vec2(560.0, 360.0);
const ROW: f32 = 30.0;

/// Which side the window shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    /// The player over others.
    #[default]
    Over,
    /// Others over the player.
    On,
}

#[derive(Clone, Debug, Default)]
pub struct Window {
    pub side: Side,
    /// Tag of the country selected in the list.
    pub selected: Option<String>,
}

#[derive(Default)]
pub struct Response {
    pub close: bool,
    pub place_done: bool,
}

const ISSUE_KEYS: [&str; ISSUES] = [
    "influence-issue-money",
    "influence-issue-trade",
    "influence-issue-territory",
    "influence-issue-security",
    "influence-issue-sovereignty",
];

const CHANNEL_KEYS: [&str; 4] = [
    "influence-channel-credit",
    "influence-channel-investment",
    "influence-channel-trade",
    "influence-channel-protection",
];

/// A share of GDP as a percentage, two decimals: influence is small.
fn pct(i18n: &I18n, x: f64) -> String {
    format!("{} %", i18n.decimal(x * 100.0, 2))
}

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    view: Option<&InfluenceView>,
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
    let (rect, moved) = super::window_frame(ui, "influence", default, screen, place, MIN_WINDOW);
    out.place_done |= moved;
    region(ui, rect.shrink(space::XL), |ui| {
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("influence-title"));
            ui.add_space(space::MD);
            let options = vec![
                (Side::Over, i18n.tr("influence-side-over")),
                (Side::On, i18n.tr("influence-side-on")),
            ];
            let tips = [
                Tip::Item('Z', "influence.side.over".into(), "1"),
                Tip::Item('Z', "influence.side.on".into(), "2"),
            ];
            if keys::segmented(ui, &mut window.side, &options, &tips) {
                window.selected = None;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = i18n.tr("gov-close");
                let tip = keys::tip(ui.ctx(), &label, id::BACK);
                let close = bf_ui_kit::close_button(ui, &label, &tip);
                if close.clicked() || keys::single(ui, "influence.close", 'X', close.rect) {
                    out.close = true;
                }
            });
        });
        bf_ui_kit::divider(ui);
        let Some(v) = view else {
            bf_ui_kit::muted(ui, &i18n.tr("influence-off"));
            return;
        };
        bf_ui_kit::small(
            ui,
            &i18n.tr(match window.side {
                Side::Over => "influence-basis-over",
                Side::On => "influence-basis-on",
            }),
        );
        bf_ui_kit::gap(ui, space::SM);
        let pairs = match window.side {
            Side::Over => &v.over,
            Side::On => &v.on,
        };
        if pairs.is_empty() {
            bf_ui_kit::muted(ui, &i18n.tr("influence-none"));
            return;
        }
        if window
            .selected
            .as_ref()
            .is_none_or(|s| !pairs.iter().any(|p| &p.other == s))
        {
            window.selected = pairs.first().map(|p| p.other.clone());
        }
        let wide = ui.available_width() > 760.0;
        let list = |ui: &mut Ui, window: &mut Window| {
            egui::ScrollArea::vertical()
                .id_salt("influence-list")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for p in pairs {
                        row(ui, i18n, p, window);
                    }
                });
        };
        if wide {
            ui.spacing_mut().item_spacing.x = space::XL;
            ui.columns(2, |cols| {
                list(&mut cols[0], window);
                let sel = window
                    .selected
                    .as_ref()
                    .and_then(|s| pairs.iter().find(|p| &p.other == s));
                if let Some(p) = sel {
                    egui::ScrollArea::vertical()
                        .id_salt("influence-detail")
                        .auto_shrink([false, false])
                        .show(&mut cols[1], |ui| detail(ui, i18n, v, p, window.side));
                }
            });
        } else {
            egui::ScrollArea::vertical()
                .id_salt("influence-narrow")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for p in pairs {
                        row(ui, i18n, p, window);
                    }
                    bf_ui_kit::divider(ui);
                    let sel = window
                        .selected
                        .as_ref()
                        .and_then(|s| pairs.iter().find(|p| &p.other == s));
                    if let Some(p) = sel {
                        detail(ui, i18n, v, p, window.side);
                    }
                });
        }
    });
    out.place_done |= super::window_grip(ui, "influence", rect, place, MIN_WINDOW);
    out
}

/// One country: its name, five small bars (one per issue) and the largest share, whole.
fn row(ui: &mut Ui, i18n: &I18n, p: &Pair, window: &mut Window) {
    let selected = window.selected.as_deref() == Some(p.other.as_str());
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    let name = country_name(i18n, &p.other);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &name));
    if selected || resp.hovered() {
        ui.painter().rect_filled(
            r,
            radius::SM,
            if selected {
                color::ACCENT_SOFT
            } else {
                color::SURFACE_HOVER
            },
        );
    }
    let inner = r.shrink2(vec2(space::SM, 0.0));
    let value = pct(i18n, p.top().max(0.0));
    let value_w = ui
        .painter()
        .layout_no_wrap(value.clone(), sans(size::SMALL), color::TEXT)
        .size()
        .x
        .max(
            ui.painter()
                .layout_no_wrap("99,99 %".into(), sans(size::SMALL), color::TEXT)
                .size()
                .x,
        );
    let bars_w = 5.0 * 8.0;
    let value_rect = Rect::from_min_max(
        egui::pos2(inner.right() - value_w, inner.top()),
        inner.right_bottom(),
    );
    let bars = Rect::from_min_max(
        egui::pos2(value_rect.left() - space::MD - bars_w, inner.top() + 7.0),
        egui::pos2(value_rect.left() - space::MD, inner.bottom() - 7.0),
    );
    let name_rect = Rect::from_min_max(
        inner.min,
        egui::pos2(bars.left() - space::MD, inner.bottom()),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(name_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(name)
                        .font(sans(size::SMALL))
                        .color(color::TEXT),
                )
                .truncate(),
            );
        },
    );
    // Five bars, one per issue, against the largest of the row.
    let top = p.top().max(1e-12);
    for (k, issue) in p.issues.iter().enumerate() {
        let x = bars.left() + k as f32 * 8.0;
        let h = (bars.height() * (issue.influence.max(0.0) / top) as f32).max(1.0);
        ui.painter().rect_filled(
            Rect::from_min_max(
                egui::pos2(x + 1.0, bars.bottom() - h),
                egui::pos2(x + 6.0, bars.bottom()),
            ),
            1.0,
            color::ACCENT,
        );
    }
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(value_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(value)
                        .font(sans(size::SMALL))
                        .color(color::TEXT),
                )
                .extend(),
            );
        },
    );
    let typed = keys::item(
        ui,
        'W',
        format!("influence.country.{}", p.other),
        &p.other,
        r,
    );
    if resp.clicked() || typed {
        window.selected = Some(p.other.clone());
    }
}

fn detail(ui: &mut Ui, i18n: &I18n, v: &InfluenceView, p: &Pair, side: Side) {
    ui.spacing_mut().item_spacing.y = space::XS;
    let (a, b) = match side {
        Side::Over => (v.tag.as_str(), p.other.as_str()),
        Side::On => (p.other.as_str(), v.tag.as_str()),
    };
    ui.label(
        RichText::new(i18n.tr_args(
            "influence-pair",
            &[
                ("a", Arg::String(country_name(i18n, a))),
                ("b", Arg::String(country_name(i18n, b))),
            ],
        ))
        .font(sans_semibold(size::BODY))
        .color(color::TEXT),
    );
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "influence-affinity",
            &[
                ("regime", Arg::String(pct(i18n, p.regime))),
                ("people", Arg::String(pct(i18n, p.people))),
                ("factor", Arg::String(i18n.decimal(p.factor, 2))),
            ],
        ),
    );
    bf_ui_kit::gap(ui, space::SM);
    for (k, issue) in p.issues.iter().enumerate() {
        bf_ui_kit::stat_row(ui, &i18n.tr(ISSUE_KEYS[k]), &pct(i18n, issue.influence));
        for c in &issue.channels {
            let kind = i18n.tr(CHANNEL_KEYS[c.kind.min(3)]);
            match (c.value, c.front) {
                (Some(_), _) => {
                    ui.horizontal(|ui| {
                        ui.add_space(space::LG);
                        bf_ui_kit::small(
                            ui,
                            &i18n.tr_args(
                                "influence-channel-line",
                                &[
                                    ("channel", Arg::String(kind)),
                                    ("part", Arg::String(pct(i18n, c.part))),
                                    (
                                        "weight",
                                        Arg::String(i18n.decimal(f64::from(c.weight) / 10.0, 1)),
                                    ),
                                ],
                            ),
                        );
                    });
                }
                (None, front) => {
                    ui.horizontal(|ui| {
                        ui.add_space(space::LG);
                        ui.label(
                            RichText::new(i18n.tr_args(
                                "influence-channel-dormant",
                                &[
                                    ("channel", Arg::String(kind)),
                                    (
                                        "front",
                                        Arg::String(i18n.tr(&format!(
                                            "influence-front-{}",
                                            front.unwrap_or("unknown")
                                        ))),
                                    ),
                                ],
                            ))
                            .font(sans(size::CAPTION))
                            .color(color::TEXT_FAINT),
                        );
                    });
                }
            }
        }
        if issue.goodwill != 0.0 {
            ui.horizontal(|ui| {
                ui.add_space(space::LG);
                ui.label(
                    RichText::new(i18n.tr_args(
                        "influence-goodwill",
                        &[("part", Arg::String(pct(i18n, issue.goodwill)))],
                    ))
                    .font(sans(size::SMALL))
                    .color(if issue.goodwill < 0.0 {
                        color::DANGER
                    } else {
                        color::POSITIVE
                    }),
                );
            });
        }
        bf_ui_kit::gap(ui, space::XS);
    }
    bf_ui_kit::small(ui, &i18n.tr("influence-note"));
}
