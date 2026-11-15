//! Campaign top bar: the player's country on the left with its headline figures, the date and
//! the speed controls on the right.

use bf_i18n::{Arg, I18n};
use bf_ui_kit::theme::{sans, sans_semibold, serif};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Icon, icon_button, paint_icon};
use egui::{Align2, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

use crate::sim::Snapshot;
use crate::text::{country_name, date_long, difficulty_key, money_exact};
use crate::world::{Country, World};

pub const HEIGHT: f32 = 64.0;
const PLATE_W: f32 = 290.0;

/// What the player did on the bar.
#[derive(Default)]
pub struct BarResponse {
    pub open_country: bool,
    pub toggle_pause: bool,
    pub speed: Option<u8>,
    pub open_menu: bool,
}

pub fn show(
    ui: &mut Ui,
    rect: Rect,
    i18n: &I18n,
    world: Option<&World>,
    player: Option<usize>,
    snap: &Snapshot,
) -> BarResponse {
    let mut out = BarResponse::default();
    let painter = ui.painter();
    painter.rect_filled(rect, 0, color::SURFACE);
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, color::BORDER_STRONG),
    );

    // Right side first, so the stats know how much room is left.
    let right_w = right_controls(ui, rect, i18n, snap, &mut out);

    if let (Some(world), Some(p)) = (world, player) {
        let country = &world.countries[p];
        let plate = Rect::from_min_size(rect.min, Vec2::new(PLATE_W, rect.height()));
        if plate_ui(ui, plate, i18n, world, country).clicked() {
            out.open_country = true;
        }
        let stats_rect = Rect::from_min_max(
            Pos2::new(plate.right(), rect.top()),
            Pos2::new(rect.right() - right_w - space::LG, rect.bottom()),
        );
        stats(ui, stats_rect, i18n, world, country);
    }
    out
}

fn plate_ui(ui: &mut Ui, rect: Rect, i18n: &I18n, world: &World, c: &Country) -> Response {
    let response = ui.interact(rect, ui.make_persistent_id("top-plate"), Sense::click());
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, 0, color::SURFACE_HOVER);
    }
    painter.vline(
        rect.right(),
        rect.y_range().shrink(10.0),
        Stroke::new(1.0, color::BORDER),
    );

    // Banner in the country's colour, with a notch at the bottom.
    let (r, g, b) = c.def.color;
    let fill = Color32::from_rgb(r, g, b);
    let banner = Rect::from_min_size(
        Pos2::new(rect.left() + space::LG, rect.top() + 9.0),
        Vec2::new(34.0, 46.0),
    );
    let notch = Pos2::new(banner.center().x, banner.bottom() - 8.0);
    let mut mesh = egui::Mesh::default();
    for p in [
        banner.left_top(),
        banner.right_top(),
        banner.right_bottom(),
        notch,
        banner.left_bottom(),
    ] {
        mesh.colored_vertex(p, fill);
    }
    mesh.add_triangle(0, 1, 3);
    mesh.add_triangle(1, 2, 3);
    mesh.add_triangle(0, 3, 4);
    painter.add(mesh);
    painter.add(egui::Shape::closed_line(
        vec![
            banner.left_top(),
            banner.right_top(),
            banner.right_bottom(),
            notch,
            banner.left_bottom(),
        ],
        Stroke::new(1.0, Color32::from_black_alpha(160)),
    ));
    painter.hline(
        banner.x_range(),
        banner.top() + 1.0,
        Stroke::new(2.0, color::ACCENT),
    );

    let x = banner.right() + space::MD;
    let room = rect.right() - x - space::SM;
    let name = country_name(i18n, c.tag());
    // Shrink long names to fit one line; below 13 pt they are cut with an ellipsis.
    let mut font = 21.0;
    let name_galley = loop {
        let g = painter.layout_no_wrap(name.clone(), serif(font), color::TEXT);
        if g.size().x <= room || font <= 13.0 {
            break g;
        }
        font -= 1.0;
    };
    let name_galley = if name_galley.size().x > room {
        let mut job =
            egui::text::LayoutJob::simple_singleline(name.clone(), serif(font), color::TEXT);
        job.wrap = egui::text::TextWrapping::truncate_at_width(room);
        painter.layout_job(job)
    } else {
        name_galley
    };
    painter.galley(Pos2::new(x, rect.top() + 12.0), name_galley, color::TEXT);
    let detail = i18n.tr(difficulty_key(world, c));
    painter.text(
        Pos2::new(x, rect.top() + 42.0),
        Align2::LEFT_CENTER,
        detail,
        sans(size::CAPTION + 0.5),
        color::TEXT_MUTED,
    );
    let tip = format!(
        "{name}\n{}\n{}",
        i18n.tr(&format!("government-{}", c.def.government)),
        i18n.tr("bar-country-tip")
    );

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(bf_ui_kit::tooltip_text(&tip))
}

/// One figure on the bar.
struct Stat {
    icon: Icon,
    label: String,
    value: String,
    tip: Box<dyn Fn(&mut Ui)>,
}

fn pct(i18n: &I18n, share: f32) -> String {
    let v = share as f64 * 100.0;
    let digits = if v < 10.0 { 1 } else { 0 };
    i18n.tr_args(
        "bar-percent",
        &[("value", Arg::String(i18n.decimal(v, digits)))],
    )
}

fn stats(ui: &mut Ui, rect: Rect, i18n: &I18n, world: &World, c: &Country) {
    let unknown = i18n.tr("bar-unknown");
    let source_social = c.def.social_source.clone();
    let rank = |r: usize| i18n.tr_args("bar-rank", &[("rank", Arg::String(r.to_string()))]);

    let gdp_value = i18n.tr_args(
        "money-intl-dollars",
        &[("value", Arg::String(i18n.compact_short(c.gdp)))],
    );
    let gdp_lines = vec![
        i18n.tr("bar-gdp-tip"),
        i18n.tr_args(
            "bar-gdp-per-capita",
            &[(
                "value",
                Arg::String(money_exact(i18n, c.def.gdp_per_capita as f64)),
            )],
        ),
        rank(c.rank_gdp),
    ];
    let pop_lines = vec![
        i18n.tr("bar-population-tip"),
        i18n.int(c.def.population as i64),
        i18n.tr_args(
            "bar-density",
            &[(
                "value",
                Arg::String(i18n.decimal(c.def.population as f64 / c.area_km2.max(1.0), 1)),
            )],
        ),
        rank(c.rank_population),
    ];
    let quality_lines = match c.quality {
        Some(q) => vec![
            i18n.tr("bar-quality-tip"),
            format!(
                "{}  {}",
                i18n.tr("bar-quality-income"),
                i18n.decimal(q.income, 2)
            ),
            format!(
                "{}  {}",
                i18n.tr("bar-quality-health"),
                i18n.decimal(q.health, 2)
            ),
            format!(
                "{}  {}",
                i18n.tr("bar-quality-education"),
                i18n.decimal(q.education, 2)
            ),
            c.rank_quality.map(rank).unwrap_or_default(),
        ],
        None => vec![i18n.tr("bar-quality-tip"), i18n.tr("bar-quality-missing")],
    };

    let source = |s: &str| i18n.tr_args("bar-source", &[("source", Arg::String(s.to_owned()))]);
    let economic_source = source(&c.def.source);
    let social_source = if source_social.is_empty() {
        String::new()
    } else {
        source(&source_social)
    };

    let entries: Vec<(Icon, &str, String, Vec<String>, String)> = vec![
        (
            Icon::Coins,
            "bar-gdp",
            gdp_value,
            gdp_lines,
            economic_source.clone(),
        ),
        (
            Icon::People,
            "bar-population",
            i18n.compact_short(c.def.population as f64),
            pop_lines,
            economic_source,
        ),
        (
            Icon::Heart,
            "bar-quality",
            c.quality
                .map_or(unknown.clone(), |q| i18n.decimal(q.index, 1)),
            quality_lines,
            social_source.clone(),
        ),
        (
            Icon::Book,
            "bar-literacy",
            c.def.literacy.map_or(unknown.clone(), |l| pct(i18n, l)),
            vec![i18n.tr("bar-literacy-tip")],
            social_source.clone(),
        ),
        (
            Icon::Hourglass,
            "bar-life",
            c.def.life_expectancy.map_or(unknown.clone(), |l| {
                i18n.tr_args(
                    "bar-years",
                    &[("value", Arg::String(i18n.decimal(l as f64, 1)))],
                )
            }),
            vec![i18n.tr("bar-life-tip")],
            social_source.clone(),
        ),
        (
            Icon::City,
            "bar-urban",
            c.def.urbanization.map_or(unknown.clone(), |u| pct(i18n, u)),
            vec![i18n.tr("bar-urban-tip")],
            social_source,
        ),
    ];
    let _ = world;
    let stats: Vec<Stat> = entries
        .into_iter()
        .map(|(icon, id, value, lines, source)| {
            let label = i18n.tr(id);
            let title = label.clone();
            let big = value.clone();
            Stat {
                icon,
                label,
                value,
                tip: Box::new(move |ui: &mut Ui| tooltip(ui, icon, &title, &big, &lines, &source)),
            }
        })
        .collect();

    let mut x = rect.left() + space::SM;
    for (i, stat) in stats.iter().enumerate() {
        let painter = ui.painter();
        let value = painter.layout_no_wrap(stat.value.clone(), sans_semibold(16.0), color::TEXT);
        let label = painter.layout_no_wrap(
            stat.label.to_uppercase(),
            sans_semibold(10.0),
            color::TEXT_FAINT,
        );
        let text_w = value.size().x.max(label.size().x);
        let w = 22.0 + space::SM + text_w + space::MD * 2.0;
        if x + w > rect.right() {
            break;
        }
        let cell = Rect::from_min_size(
            Pos2::new(x, rect.top() + 8.0),
            Vec2::new(w, rect.height() - 16.0),
        );
        let response = ui.interact(cell, ui.make_persistent_id(("top-stat", i)), Sense::hover());
        let a11y = format!("{}: {}", stat.label, stat.value);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, a11y.clone()));
        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(cell, radius::MD, color::SURFACE_HOVER);
        }
        let icon_rect = Rect::from_center_size(
            Pos2::new(cell.left() + space::MD + 11.0, cell.center().y),
            Vec2::splat(22.0),
        );
        paint_icon(ui, stat.icon, icon_rect, color::ACCENT);
        let tx = icon_rect.right() + space::SM;
        let painter = ui.painter();
        let label_h = label.size().y;
        painter.galley(
            Pos2::new(tx, cell.center().y - 15.0),
            label,
            color::TEXT_FAINT,
        );
        painter.galley(
            Pos2::new(tx, cell.center().y - 15.0 + label_h + 1.0),
            value,
            color::TEXT,
        );
        response.on_hover_ui(|ui| (stat.tip)(ui));
        x += w;
        if i + 1 < stats.len() {
            ui.painter().vline(
                x,
                rect.y_range().shrink(16.0),
                Stroke::new(1.0, color::BORDER),
            );
        }
    }
}

fn tooltip(ui: &mut Ui, icon: Icon, title: &str, value: &str, lines: &[String], source: &str) {
    ui.set_max_width(320.0);
    ui.spacing_mut().item_spacing.y = space::XS + 2.0;
    ui.horizontal(|ui| {
        bf_ui_kit::icon(ui, icon, 18.0, color::ACCENT);
        ui.label(
            egui::RichText::new(title)
                .font(sans_semibold(size::BODY))
                .color(color::TEXT),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .font(sans_semibold(size::BODY))
                    .color(color::ACCENT),
            );
        });
    });
    bf_ui_kit::divider(ui);
    for line in lines.iter().filter(|l| !l.is_empty()) {
        ui.label(
            egui::RichText::new(line)
                .font(sans(size::SMALL))
                .color(color::TEXT),
        );
    }
    if !source.is_empty() {
        ui.add_space(space::XS);
        ui.label(
            egui::RichText::new(source)
                .font(sans(size::CAPTION))
                .color(color::TEXT_FAINT),
        );
    }
}

/// Menu, speed and date, laid out from the right edge. Returns the width used.
fn right_controls(
    ui: &mut Ui,
    rect: Rect,
    i18n: &I18n,
    snap: &Snapshot,
    out: &mut BarResponse,
) -> f32 {
    let button = 36.0;
    let y = rect.center().y - button / 2.0;
    let mut x = rect.right() - space::LG;

    let mut place = |w: f32| {
        x -= w;
        let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, button));
        x -= space::SM;
        r
    };

    let menu_rect = place(button);
    let pause_rect = place(button);
    let plus_rect = place(button - 8.0);
    let date_rect = place(210.0);
    let minus_rect = place(button - 8.0);

    let mut at = |r: Rect, f: &mut dyn FnMut(&mut Ui)| {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
        f(&mut child);
    };

    at(menu_rect, &mut |ui| {
        if icon_button(ui, Icon::Menu, button, &i18n.tr("bar-menu"), false).clicked() {
            out.open_menu = true;
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
    at(plus_rect, &mut |ui| {
        if icon_button(
            ui,
            Icon::Plus,
            button - 8.0,
            &i18n.tr("bar-speed-up"),
            false,
        )
        .clicked()
        {
            out.speed = Some((snap.speed + 1).min(5));
        }
    });
    at(minus_rect, &mut |ui| {
        if icon_button(
            ui,
            Icon::Minus,
            button - 8.0,
            &i18n.tr("bar-speed-down"),
            false,
        )
        .clicked()
        {
            out.speed = Some(snap.speed.saturating_sub(1).max(1));
        }
    });

    // Date with five speed pips under it; clicking it pauses or resumes.
    let block = Rect::from_center_size(
        date_rect.center(),
        Vec2::new(date_rect.width(), rect.height() - 14.0),
    );
    let response = ui.interact(block, ui.make_persistent_id("top-date"), Sense::click());
    let painter = ui.painter();
    painter.rect(
        block,
        radius::MD,
        if response.hovered() {
            color::SURFACE_HOVER
        } else {
            color::SURFACE_SUNKEN
        },
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    let date_color = if snap.paused {
        color::TEXT_MUTED
    } else {
        color::TEXT
    };
    painter.text(
        Pos2::new(block.center().x, block.top() + 17.0),
        Align2::CENTER_CENTER,
        date_long(i18n, snap.date),
        serif(19.0),
        date_color,
    );
    if snap.paused {
        painter.text(
            Pos2::new(block.center().x, block.bottom() - 10.0),
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
                block.center().x - total / 2.0 + i as f32 * (pip.x + 4.0),
                block.bottom() - 12.0,
            );
            let fill = if i < snap.speed {
                color::ACCENT
            } else {
                color::BORDER_STRONG
            };
            painter.rect_filled(Rect::from_min_size(min, pip), 2, fill);
        }
    }
    let tip = i18n.tr_args("bar-date-tip", &[("speed", Arg::Num(snap.speed as f64))]);
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(bf_ui_kit::tooltip_text(&tip))
        .clicked()
    {
        out.toggle_pause = true;
    }
    rect.right() - x
}
