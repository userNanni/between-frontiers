//! Campaign HUD at the top of the screen, over the map.
//!
//! - Top left: the player's banner with the economic rank under it, and a panel of figures: a
//!   row of headline indicators with bars, then two compact rows whose labels live in the
//!   tooltips.
//! - Top right: date and speed controls.

use bf_i18n::{Arg, I18n};
use bf_sim::indicators::{INCOME_CEILING, INCOME_FLOOR, LIFE_CEILING, LIFE_FLOOR};
use bf_ui_kit::theme::{sans, sans_semibold, serif};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Icon, icon_button, paint_icon};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::flags::{Flags, paint_flag};
use crate::sim::Snapshot;
use crate::text::{
    country_name, date_long, difficulty_key, economy_source, money_exact, money_unit,
    social_source, status_line,
};
use crate::world::{Source, World};

const MARGIN: f32 = 12.0;
const BANNER: Vec2 = Vec2::new(84.0, 60.0);
const COL_W: f32 = 138.0;
const COLS: usize = 5;
const ROW1_H: f32 = 46.0;
const ROW_H: f32 = 26.0;
const PAD: f32 = 10.0;
const TIME_H: f32 = 56.0;

/// Lowest screen y used by the HUD on the right, for panels placed under it.
pub const RIGHT_BOTTOM: f32 = MARGIN + TIME_H + MARGIN;

#[derive(Default)]
pub struct BarResponse {
    pub open_country: bool,
    pub toggle_pause: bool,
    pub speed: Option<u8>,
    pub open_menu: bool,
    pub open_research: bool,
    pub open_government: bool,
}

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    world: Option<&World>,
    flags: &Flags,
    player: Option<usize>,
    snap: &Snapshot,
) -> BarResponse {
    let mut out = BarResponse::default();
    time_panel(ui, screen, i18n, snap, &mut out);
    if let (Some(world), Some(p)) = (world, player) {
        corner_panel(ui, screen, i18n, world, flags, p, &mut out);
    }
    out
}

/// A tooltip in the shape: title and value, a summary, then the parts the value comes from.
struct Tip {
    title: String,
    value: String,
    summary: Vec<String>,
    breakdown: Vec<(String, String)>,
    footer: Vec<String>,
}

struct Item {
    icon: Icon,
    value: String,
    /// 0–1, for the headline row.
    bar: Option<f32>,
    tip: Tip,
}

fn rank(i18n: &I18n, r: usize) -> String {
    i18n.tr_args("bar-rank", &[("rank", Arg::String(r.to_string()))])
}

fn pct(i18n: &I18n, share: f64) -> String {
    let v = share * 100.0;
    let digits = if v < 10.0 { 1 } else { 0 };
    i18n.tr_args(
        "bar-percent",
        &[("value", Arg::String(i18n.decimal(v, digits)))],
    )
}

fn money_short(i18n: &I18n, v: f64) -> String {
    i18n.tr_args(
        "money-intl-dollars",
        &[("value", Arg::String(i18n.compact_short(v)))],
    )
}

fn years(i18n: &I18n, v: f64) -> String {
    i18n.tr_args("bar-years", &[("value", Arg::String(i18n.decimal(v, 1)))])
}

/// Figures of the country `p`: the simulation's during a campaign, read once a month.
fn items(i18n: &I18n, world: &World, p: usize) -> (Vec<Item>, Vec<Item>, Vec<Item>) {
    let c = &world.countries[p];
    let table = world.figures();
    let f = &table.countries[p];
    let live = matches!(table.source, Source::Live { .. });
    let unknown = i18n.tr("bar-unknown");
    let social = social_source(i18n, table.source);
    // Life expectancy is the scenario's even in a campaign: the simulation has none yet.
    let life_source = if live {
        vec![i18n.tr("hud-life-scenario"), i18n.tr("source-social")]
    } else {
        vec![i18n.tr("source-social")]
    };
    let economy = economy_source(i18n, table.source);
    let unit = money_unit(i18n, table.source);
    let due = i18n.tr("hud-due-to");
    let pop = f.population;
    let gdppc = f.gdp_per_capita();
    let tip = |title: &str,
               value: &str,
               summary: Vec<String>,
               breakdown: Vec<(String, String)>,
               footer: Vec<String>| Tip {
        title: i18n.tr(title),
        value: value.to_owned(),
        summary,
        breakdown,
        footer,
    };

    // Row 1: headline indicators with bars.
    let quality_value = f
        .quality
        .map_or(unknown.clone(), |q| i18n.decimal(q.index, 1));
    let quality_breakdown = match f.quality {
        Some(q) => vec![
            (
                format!(
                    "{} ({})",
                    i18n.tr("bar-quality-income"),
                    money_exact(i18n, gdppc)
                ),
                i18n.decimal(q.income, 2),
            ),
            (
                format!(
                    "{} ({})",
                    i18n.tr("bar-quality-health"),
                    f.life_expectancy
                        .map_or(unknown.clone(), |l| years(i18n, l))
                ),
                i18n.decimal(q.health, 2),
            ),
            (
                format!(
                    "{} ({})",
                    i18n.tr("bar-quality-education"),
                    f.literacy.map_or(unknown.clone(), |l| pct(i18n, l))
                ),
                i18n.decimal(q.education, 2),
            ),
        ],
        None => vec![],
    };
    let mut quality_summary = vec![i18n.tr("bar-quality-tip")];
    if f.quality.is_none() {
        quality_summary.push(i18n.tr("bar-quality-missing"));
    } else if live {
        quality_summary.push(i18n.tr("hud-quality-life-scenario"));
    }
    if !quality_breakdown.is_empty() {
        quality_summary.push(due.clone());
    }
    let row1 = vec![
        Item {
            icon: Icon::Heart,
            value: quality_value.clone(),
            bar: f.quality.map(|q| (q.index / 100.0) as f32),
            tip: tip(
                "bar-quality",
                &quality_value,
                quality_summary,
                quality_breakdown,
                vec![
                    f.rank_quality.map(|r| rank(i18n, r)).unwrap_or_default(),
                    social.clone(),
                ],
            ),
        },
        {
            let v = f.literacy.map_or(unknown.clone(), |l| pct(i18n, l));
            Item {
                icon: Icon::Book,
                bar: f.literacy.map(|l| l as f32),
                tip: tip(
                    "bar-literacy",
                    &v,
                    vec![i18n.tr("bar-literacy-tip")],
                    vec![],
                    vec![social.clone()],
                ),
                value: v,
            }
        },
        {
            let v = f
                .life_expectancy
                .map_or(unknown.clone(), |l| years(i18n, l));
            Item {
                icon: Icon::Hourglass,
                bar: f.life_expectancy.map(|l| {
                    ((l - LIFE_FLOOR) / (LIFE_CEILING - LIFE_FLOOR)).clamp(0.0, 1.0) as f32
                }),
                tip: tip(
                    "bar-life",
                    &v,
                    vec![i18n.tr("bar-life-tip"), i18n.tr("hud-life-bar")],
                    vec![],
                    life_source,
                ),
                value: v,
            }
        },
        {
            let v = f.urbanization.map_or(unknown.clone(), |u| pct(i18n, u));
            Item {
                icon: Icon::City,
                bar: f.urbanization.map(|u| u as f32),
                tip: tip(
                    "bar-urban",
                    &v,
                    vec![i18n.tr("bar-urban-tip")],
                    vec![],
                    vec![social.clone()],
                ),
                value: v,
            }
        },
        {
            let v = money_exact(i18n, gdppc);
            let bar = ((gdppc.max(1.0).ln() - INCOME_FLOOR.ln())
                / (INCOME_CEILING.ln() - INCOME_FLOOR.ln()))
            .clamp(0.0, 1.0) as f32;
            Item {
                icon: Icon::Coin,
                bar: Some(bar),
                tip: tip(
                    "hud-per-capita",
                    &v,
                    vec![i18n.tr("hud-per-capita-tip"), unit.clone()],
                    vec![],
                    economy.clone(),
                ),
                value: v,
            }
        },
    ];

    // Row 2: size of the country.
    let gdp = money_short(i18n, f.gdp);
    let population = i18n.compact_short(pop);
    let has_colonies = c.has_colonies();
    let split = |metro: String, colonies: String, total: String| -> Vec<(String, String)> {
        if has_colonies {
            vec![
                (i18n.tr("hud-metropole"), metro),
                (i18n.tr("hud-colonies"), colonies),
                (i18n.tr("hud-total"), total),
            ]
        } else {
            vec![]
        }
    };
    let density = i18n.tr_args(
        "hud-density-value",
        &[(
            "value",
            Arg::String(i18n.decimal(pop / c.area_km2.max(1.0), 1)),
        )],
    );
    let area = i18n.tr_args(
        "stat-area-value",
        &[("value", Arg::String(i18n.compact_short(c.area_km2)))],
    );
    let states = i18n.int(c.states.len() as i64);
    let area_of = |v: f64| {
        i18n.tr_args(
            "stat-area-value",
            &[("value", Arg::String(i18n.compact_short(v)))],
        )
    };
    let row2 = vec![
        Item {
            icon: Icon::Chart,
            bar: None,
            tip: tip(
                "bar-gdp",
                &gdp,
                [
                    vec![i18n.tr("bar-gdp-tip"), unit.clone()],
                    if live && has_colonies {
                        vec![i18n.tr("hud-gdp-split")]
                    } else {
                        vec![]
                    },
                    vec![due.clone()],
                ]
                .concat(),
                [
                    vec![
                        (i18n.tr("bar-population"), i18n.compact(pop)),
                        (
                            i18n.tr("hud-per-capita"),
                            format!("× {}", money_exact(i18n, gdppc)),
                        ),
                    ],
                    split(
                        money_short(i18n, f.gdp),
                        money_short(i18n, f.colonial_gdp),
                        money_short(i18n, f.total_gdp()),
                    ),
                ]
                .concat(),
                [vec![rank(i18n, f.rank_gdp)], economy.clone()].concat(),
            ),
            value: gdp,
        },
        Item {
            icon: Icon::People,
            bar: None,
            tip: tip(
                "bar-population",
                &population,
                vec![i18n.tr("bar-population-tip"), i18n.int(pop as i64)],
                split(
                    i18n.compact(pop),
                    i18n.compact(f.colonial_population),
                    i18n.compact(f.total_population()),
                ),
                vec![rank(i18n, f.rank_population), economy[0].clone()],
            ),
            value: population,
        },
        Item {
            icon: Icon::Density,
            bar: None,
            tip: tip(
                "hud-density",
                &density,
                vec![i18n.tr("hud-density-tip")],
                vec![],
                vec![],
            ),
            value: density,
        },
        Item {
            icon: Icon::Map,
            bar: None,
            tip: tip(
                "hud-area",
                &area,
                vec![
                    i18n.tr("hud-area-tip"),
                    i18n.tr_args(
                        "stat-area-value",
                        &[("value", Arg::String(i18n.int(c.area_km2.round() as i64)))],
                    ),
                ],
                split(
                    area_of(c.area_km2),
                    area_of(c.colonial_area_km2),
                    area_of(c.area_km2 + c.colonial_area_km2),
                ),
                vec![],
            ),
            value: area,
        },
        Item {
            icon: Icon::Grid,
            bar: None,
            tip: tip(
                "hud-states",
                &states,
                vec![i18n.tr("hud-states-tip")],
                split(
                    i18n.int(c.states.len() as i64),
                    i18n.int(c.colonial_states.len() as i64),
                    i18n.int((c.states.len() + c.colonial_states.len()) as i64),
                ),
                vec![],
            ),
            value: states,
        },
    ];

    // Row 3: place in the world.
    let gdp_share = pct(i18n, f.total_gdp() / table.world_gdp.max(1.0));
    let pop_share = pct(i18n, f.total_population() / table.world_population.max(1.0));
    let rank_value =
        |r: usize| i18n.tr_args("hud-rank-value", &[("rank", Arg::String(r.to_string()))]);
    let quality_rank = f.rank_quality.map_or(unknown.clone(), rank_value);
    let population_rank = rank_value(f.rank_population);
    let subjects: Vec<(String, String)> = c
        .subjects
        .iter()
        .map(|&s| {
            let sc = &world.countries[s];
            (country_name(i18n, sc.tag()), status_line(i18n, world, sc))
        })
        .collect();
    let subjects_value = i18n.int(subjects.len() as i64);
    let row3 = vec![
        Item {
            icon: Icon::Pie,
            bar: None,
            tip: tip(
                "hud-gdp-share",
                &gdp_share,
                vec![
                    i18n.tr("hud-gdp-share-tip"),
                    i18n.tr("hud-includes-colonies"),
                ],
                vec![],
                [
                    vec![i18n.tr_args(
                        "hud-world-total",
                        &[("value", Arg::String(money_short(i18n, table.world_gdp)))],
                    )],
                    economy.clone(),
                ]
                .concat(),
            ),
            value: gdp_share,
        },
        Item {
            icon: Icon::Globe,
            bar: None,
            tip: tip(
                "hud-pop-share",
                &pop_share,
                vec![
                    i18n.tr("hud-pop-share-tip"),
                    i18n.tr("hud-includes-colonies"),
                ],
                vec![],
                vec![i18n.tr_args(
                    "hud-world-total",
                    &[("value", Arg::String(i18n.compact(table.world_population)))],
                )],
            ),
            value: pop_share,
        },
        Item {
            icon: Icon::Heart,
            bar: None,
            tip: tip(
                "hud-rank-quality",
                &quality_rank,
                vec![i18n.tr("bar-quality-tip")],
                vec![],
                vec![],
            ),
            value: quality_rank,
        },
        Item {
            icon: Icon::People,
            bar: None,
            tip: tip(
                "hud-rank-population",
                &population_rank,
                vec![i18n.tr("bar-population-tip")],
                vec![],
                vec![],
            ),
            value: population_rank,
        },
        Item {
            icon: Icon::Crown,
            bar: None,
            tip: tip(
                "hud-subjects",
                &subjects_value,
                vec![if subjects.is_empty() {
                    i18n.tr("hud-subjects-none")
                } else {
                    i18n.tr("hud-subjects-tip")
                }],
                subjects,
                vec![],
            ),
            value: subjects_value,
        },
    ];
    (row1, row2, row3)
}

fn corner_panel(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    world: &World,
    flags: &Flags,
    p: usize,
    out: &mut BarResponse,
) {
    let (row1, row2, row3) = items(i18n, world, p);
    let banner = Rect::from_min_size(screen.min + Vec2::splat(MARGIN), BANNER);
    let panel = Rect::from_min_size(
        Pos2::new(banner.right() - 6.0, banner.top()),
        Vec2::new(
            COL_W * COLS as f32 + PAD * 2.0 + 6.0,
            ROW1_H + ROW_H * 2.0 + PAD * 2.0,
        ),
    );

    // Panel behind the figures; it also blocks clicks from reaching the map.
    bf_ui_kit::paint_overlay(ui, panel);
    ui.interact(
        panel,
        ui.make_persistent_id("hud-panel"),
        Sense::click_and_drag(),
    );
    let left = panel.left() + 6.0 + PAD;
    let rows_top = panel.top() + PAD;
    let rule = |ui: &Ui, y: f32| {
        ui.painter().hline(
            (left - 2.0)..=(panel.right() - PAD),
            y,
            Stroke::new(1.0, color::BORDER),
        );
    };

    for (i, item) in row1.iter().enumerate() {
        let cell = Rect::from_min_size(
            Pos2::new(left + COL_W * i as f32, rows_top),
            Vec2::new(COL_W - 6.0, ROW1_H - 4.0),
        );
        cell_ui(ui, cell, item, true, ("r1", i));
    }
    rule(ui, rows_top + ROW1_H);
    for (r, row) in [&row2, &row3].into_iter().enumerate() {
        let top = rows_top + ROW1_H + 2.0 + ROW_H * r as f32;
        for (i, item) in row.iter().enumerate() {
            let cell = Rect::from_min_size(
                Pos2::new(left + COL_W * i as f32, top),
                Vec2::new(COL_W - 6.0, ROW_H - 2.0),
            );
            cell_ui(ui, cell, item, false, (if r == 0 { "r2" } else { "r3" }, i));
        }
    }

    banner_ui(ui, banner, i18n, world, flags, p, out);
}

fn cell_ui(ui: &mut Ui, cell: Rect, item: &Item, headline: bool, salt: (&str, usize)) {
    let response = ui.interact(
        cell,
        ui.make_persistent_id(("hud-cell", salt)),
        Sense::hover(),
    );
    let a11y = format!("{}: {}", item.tip.title, item.value);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, a11y.clone()));
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(cell.expand(2.0), radius::SM, color::SURFACE_HOVER);
    }
    let icon_size = if headline { 22.0 } else { 16.0 };
    let icon_rect = Rect::from_min_size(
        Pos2::new(
            cell.left() + 2.0,
            cell.top()
                + if headline {
                    4.0
                } else {
                    (cell.height() - 16.0) / 2.0
                },
        ),
        Vec2::splat(icon_size),
    );
    paint_icon(
        ui,
        item.icon,
        icon_rect,
        if headline {
            color::ACCENT
        } else {
            color::TEXT_MUTED
        },
    );
    let tx = icon_rect.right() + if headline { 8.0 } else { 6.0 };
    let font = if headline {
        sans_semibold(16.0)
    } else {
        sans_semibold(13.0)
    };
    let painter = ui.painter();
    painter.text(
        Pos2::new(tx, icon_rect.center().y),
        Align2::LEFT_CENTER,
        &item.value,
        font,
        color::TEXT,
    );
    if headline {
        let track = Rect::from_min_size(
            Pos2::new(tx, cell.bottom() - 9.0),
            Vec2::new(cell.right() - tx - 6.0, 4.0),
        );
        painter.rect_filled(track, 2, color::SURFACE_SUNKEN);
        if let Some(f) = item.bar {
            let mut fill = track;
            fill.set_width(track.width() * f.clamp(0.0, 1.0));
            painter.rect_filled(fill, 2, bar_color(f));
        }
    }
    response.on_hover_ui(|ui| tooltip(ui, item.icon, &item.tip));
}

/// Low values red, middle brass, high green: readable at a glance without reading the number.
fn bar_color(f: f32) -> Color32 {
    if f < 0.33 {
        color::DANGER
    } else if f < 0.66 {
        color::ACCENT
    } else {
        color::POSITIVE
    }
}

fn tooltip(ui: &mut Ui, icon: Icon, tip: &Tip) {
    ui.set_max_width(340.0);
    ui.spacing_mut().item_spacing.y = space::XS + 1.0;
    ui.spacing_mut().interact_size.y = 16.0;
    ui.horizontal(|ui| {
        bf_ui_kit::icon(ui, icon, 18.0, color::ACCENT);
        ui.label(
            egui::RichText::new(&tip.title)
                .font(sans_semibold(size::BODY))
                .color(color::TEXT),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(&tip.value)
                    .font(sans_semibold(size::BODY))
                    .color(color::ACCENT),
            );
        });
    });
    bf_ui_kit::divider(ui);
    for line in tip.summary.iter().filter(|l| !l.is_empty()) {
        ui.label(
            egui::RichText::new(line)
                .font(sans(size::SMALL))
                .color(color::TEXT),
        );
    }
    for (label, value) in &tip.breakdown {
        ui.horizontal(|ui| {
            ui.add_space(space::MD);
            ui.label(
                egui::RichText::new(label)
                    .font(sans(size::SMALL))
                    .color(color::TEXT_MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(value)
                        .font(sans_semibold(size::SMALL))
                        .color(color::ACCENT),
                );
            });
        });
    }
    let footer: Vec<&String> = tip.footer.iter().filter(|l| !l.is_empty()).collect();
    if !footer.is_empty() {
        ui.add_space(space::XS);
        for line in footer {
            ui.label(
                egui::RichText::new(line)
                    .font(sans(size::CAPTION))
                    .color(color::TEXT_FAINT),
            );
        }
    }
}

fn banner_ui(
    ui: &mut Ui,
    banner: Rect,
    i18n: &I18n,
    world: &World,
    flags: &Flags,
    p: usize,
    out: &mut BarResponse,
) {
    let c = &world.countries[p];
    let rank_gdp = world.figures().countries[p].rank_gdp;
    let response = ui.interact(banner, ui.make_persistent_id("hud-banner"), Sense::click());
    ui.painter().add(
        egui::Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: color::SHADOW,
        }
        .as_shape(banner, radius::MD),
    );
    paint_flag(ui, banner, flags, c, color::ACCENT);
    if response.hovered() {
        ui.painter()
            .rect_filled(banner, radius::SM, Color32::from_white_alpha(16));
    }
    ui.painter().rect_stroke(
        banner,
        radius::SM,
        Stroke::new(1.5, color::ACCENT),
        egui::StrokeKind::Inside,
    );

    // Economic rank medallion under the banner.
    let center = Pos2::new(banner.center().x, banner.bottom() + 12.0);
    let medal = Rect::from_center_size(center, Vec2::splat(34.0));
    let medal_response = ui.interact(medal, ui.make_persistent_id("hud-medal"), Sense::hover());
    let painter = ui.painter();
    painter.circle(
        center,
        17.0,
        color::SURFACE_RAISED,
        Stroke::new(2.0, color::ACCENT),
    );
    painter.circle_stroke(
        center,
        13.5,
        Stroke::new(1.0, color::ACCENT.gamma_multiply(0.5)),
    );
    painter.text(
        center,
        Align2::CENTER_CENTER,
        rank_gdp.to_string(),
        serif(17.0),
        color::TEXT,
    );
    let medal_tip = format!(
        "{}\n{}",
        i18n.tr_args(
            "hud-medal-tip",
            &[("rank", Arg::String(rank_gdp.to_string()))]
        ),
        i18n.tr(difficulty_key(world, c))
    );
    medal_response.on_hover_text(bf_ui_kit::tooltip_text(&medal_tip));

    let tip = format!(
        "{}\n{}\n{}\n\n{}",
        country_name(i18n, c.tag()),
        status_line(i18n, world, c),
        i18n.tr(&format!("government-{}", c.def.government)),
        i18n.tr("bar-country-tip")
    );
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(bf_ui_kit::tooltip_text(&tip))
        .clicked()
    {
        out.open_country = true;
    }
}

/// Date and speed, in a panel at the top right.
fn time_panel(ui: &mut Ui, screen: Rect, i18n: &I18n, snap: &Snapshot, out: &mut BarResponse) {
    let button = 36.0;
    let small = 28.0;
    let date_w = 210.0;
    let inner_w = small + date_w + small + button * 4.0 + space::SM * 6.0;
    let panel = Rect::from_min_size(
        Pos2::new(
            screen.right() - MARGIN - inner_w - PAD * 2.0,
            screen.top() + MARGIN,
        ),
        Vec2::new(inner_w + PAD * 2.0, TIME_H),
    );
    bf_ui_kit::paint_overlay(ui, panel);
    ui.interact(
        panel,
        ui.make_persistent_id("hud-time"),
        Sense::click_and_drag(),
    );

    let y = panel.center().y;
    let mut x = panel.left() + PAD;
    let mut next = |w: f32, h: f32| {
        let r = Rect::from_min_size(Pos2::new(x, y - h / 2.0), Vec2::new(w, h));
        x += w + space::SM;
        r
    };
    let minus_rect = next(small, small);
    let date_rect = next(date_w, TIME_H - 14.0);
    let plus_rect = next(small, small);
    let pause_rect = next(button, button);
    let government_rect = next(button, button);
    let research_rect = next(button, button);
    let menu_rect = next(button, button);

    let mut at = |r: Rect, f: &mut dyn FnMut(&mut Ui)| {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
        f(&mut child);
    };
    at(minus_rect, &mut |ui| {
        if icon_button(ui, Icon::Minus, small, &i18n.tr("bar-speed-down"), false).clicked() {
            out.speed = Some(snap.speed.saturating_sub(1).max(1));
        }
    });
    at(plus_rect, &mut |ui| {
        if icon_button(ui, Icon::Plus, small, &i18n.tr("bar-speed-up"), false).clicked() {
            out.speed = Some((snap.speed + 1).min(5));
        }
    });
    at(pause_rect, &mut |ui| {
        let (icon, tip) = if snap.paused {
            (Icon::Play, i18n.tr("bar-play"))
        } else {
            (Icon::Pause, i18n.tr("bar-pause"))
        };
        if icon_button(ui, icon, button, &tip, snap.paused).clicked() {
            out.toggle_pause = true;
        }
    });
    at(government_rect, &mut |ui| {
        if icon_button(ui, Icon::Shield, button, &i18n.tr("bar-government"), false).clicked() {
            out.open_government = true;
        }
    });
    at(research_rect, &mut |ui| {
        if icon_button(ui, Icon::Flask, button, &i18n.tr("bar-research"), false).clicked() {
            out.open_research = true;
        }
    });
    at(menu_rect, &mut |ui| {
        if icon_button(ui, Icon::Menu, button, &i18n.tr("bar-menu"), false).clicked() {
            out.open_menu = true;
        }
    });

    let response = ui.interact(date_rect, ui.make_persistent_id("hud-date"), Sense::click());
    let painter = ui.painter();
    painter.rect(
        date_rect,
        radius::MD,
        if response.hovered() {
            color::SURFACE_HOVER
        } else {
            color::SURFACE_SUNKEN
        },
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    painter.text(
        Pos2::new(date_rect.center().x, date_rect.top() + 15.0),
        Align2::CENTER_CENTER,
        date_long(i18n, snap.date),
        serif(19.0),
        if snap.paused {
            color::TEXT_MUTED
        } else {
            color::TEXT
        },
    );
    if snap.paused {
        painter.text(
            Pos2::new(date_rect.center().x, date_rect.bottom() - 9.0),
            Align2::CENTER_CENTER,
            i18n.tr("game-paused").to_uppercase(),
            sans_semibold(9.5),
            color::ACCENT,
        );
    } else {
        let pip = Vec2::new(16.0, 4.0);
        let total = pip.x * 5.0 + 4.0 * 4.0;
        for i in 0..5u8 {
            let min = Pos2::new(
                date_rect.center().x - total / 2.0 + i as f32 * (pip.x + 4.0),
                date_rect.bottom() - 11.0,
            );
            let fill = if i < snap.speed {
                color::ACCENT
            } else {
                color::BORDER_STRONG
            };
            painter.rect_filled(Rect::from_min_size(min, pip), 2, fill);
        }
    }
    let tip = format!(
        "{}\n{}",
        i18n.tr_args("bar-date-tip", &[("speed", Arg::Num(snap.speed as f64))]),
        i18n.tr_args(
            "hud-engine",
            &[
                ("tick", Arg::String(i18n.int(snap.tick as i64))),
                ("hash", Arg::String(snap.hash.clone())),
            ],
        )
    );
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(bf_ui_kit::tooltip_text(&tip))
        .clicked()
    {
        out.toggle_pause = true;
    }
}
