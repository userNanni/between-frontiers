//! Map mode panel at the bottom left: political, resources, climate, land use, with a legend.

use bf_i18n::{Arg, I18n};
use bf_sim::resources::{CLIMATES, FARMS, RESOURCES};
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, radius, space};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::map::MapView;
use crate::map::lens::{self, Lens};
use crate::world::World;

// Wide enough for the five families in English and Portuguese.
const WIDTH: f32 = 440.0;
/// Sets the map's lens and recolours it, open picker or not (a command calls this; the
/// `map.mode_*` commands can pass the family's first lens).
pub fn choose(view: &mut MapView, world: &World, lens: Lens, year: i32) {
    view.lens = lens;
    view.set_palette(lens::palette(world, lens, year));
}

/// The collapsed picker: a button with a magnifier and the lens in use.
pub const BUTTON: Vec2 = Vec2::new(200.0, 40.0);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Political,
    Resources,
    Climate,
    Farms,
    Infrastructure,
}

fn family(lens: Lens) -> Family {
    match lens {
        Lens::Political => Family::Political,
        Lens::Resource(_) => Family::Resources,
        Lens::Climate => Family::Climate,
        Lens::Farm(_) => Family::Farms,
        Lens::Local(_) => Family::Infrastructure,
    }
}

fn items(fam: Family) -> &'static [&'static str] {
    match fam {
        Family::Resources => RESOURCES,
        Family::Farms => FARMS,
        Family::Infrastructure => &lens::LOCAL_GOODS,
        _ => &[],
    }
}

/// Height of the panel for a lens.
fn height(lens: Lens) -> f32 {
    let fam = family(lens);
    let rows = items(fam).len().div_ceil(2) as f32;
    let legend_h = match fam {
        Family::Political => 0.0,
        Family::Climate => 7.0 * 20.0 + 8.0,
        _ => 44.0,
    };
    34.0 + 40.0 + rows * 24.0 + legend_h + space::LG * 2.0
}

/// Size of the panel for a lens.
pub fn size(lens: Lens) -> Vec2 {
    Vec2::new(WIDTH, height(lens))
}

/// The lens in words: its family, and the resource, crop or good when it has one.
pub fn name(i18n: &I18n, lens: Lens) -> String {
    let fam = match family(lens) {
        Family::Political => "lens-political",
        Family::Resources => "lens-resources",
        Family::Climate => "lens-climate",
        Family::Farms => "lens-farms",
        Family::Infrastructure => "lens-infrastructure",
    };
    let fam = i18n.tr(fam);
    let item = match lens {
        Lens::Resource(i) => RESOURCES.get(i).map(|k| i18n.tr(&format!("resource-{k}"))),
        Lens::Farm(i) => FARMS.get(i).map(|k| i18n.tr(&format!("building-{k}"))),
        Lens::Local(i) => lens::LOCAL_GOODS
            .get(i)
            .map(|k| i18n.tr(&format!("good-{k}"))),
        _ => None,
    };
    match item {
        Some(item) => format!("{fam} · {item}"),
        None => fam,
    }
}

/// What the player did in the panel.
#[derive(Clone, Copy, Debug, Default)]
pub struct LensResponse {
    /// Clicked the link that keeps the panel open or folds it.
    pub toggle_pin: bool,
    /// Finished a choice: an item of a list, or a family without one. Picking a family with a
    /// list leaves the list to choose from.
    pub chosen: bool,
}

/// The collapsed picker: a magnifier and the lens in use. True when clicked.
pub fn button(ui: &mut Ui, rect: Rect, i18n: &I18n, lens: Lens, open: bool) -> bool {
    let label = name(i18n, lens);
    let resp = ui.interact(rect, ui.make_persistent_id("lens-button"), Sense::click());
    let a11y = format!("{}: {label}", i18n.tr("lens-title"));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, a11y.clone()));
    bf_ui_kit::paint_overlay(ui, rect);
    let p = ui.painter();
    if resp.hovered() || open {
        p.rect_filled(rect.shrink(2.0), radius::MD, color::SURFACE_HOVER);
    }
    // The magnifier, drawn: a ring and a handle.
    let c = rect.left_center() + Vec2::new(20.0, -2.0);
    let tint = if open {
        color::ACCENT_HOVER
    } else {
        color::ACCENT
    };
    p.circle_stroke(c, 7.0, Stroke::new(2.0, tint));
    p.line_segment(
        [c + Vec2::new(5.0, 5.0), c + Vec2::new(10.5, 10.5)],
        Stroke::new(2.5, tint),
    );
    let (g, cut) =
        super::journal::one_line(p, &label, sans(13.0), color::TEXT, rect.width() - 50.0);
    p.galley(
        Pos2::new(rect.left() + 38.0, rect.center().y - g.size().y / 2.0),
        g,
        color::TEXT,
    );
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let tip = if cut {
        format!("{}\n{label}", i18n.tr("lens-button-tip"))
    } else {
        i18n.tr("lens-button-tip")
    };
    resp.on_hover_text(bf_ui_kit::tooltip_text(&tip)).clicked()
}

/// Shows the panel with its top left corner at `origin` and recolours the map when the lens
/// changes. With `pinned` given, a link at the top keeps the panel open or folds it back into
/// its button; true when it was clicked.
#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    origin: Pos2,
    i18n: &I18n,
    world: &World,
    view: &mut MapView,
    year: i32,
    pinned: Option<bool>,
) -> LensResponse {
    let mut toggle_pin = false;
    let mut chosen = false;
    let current = view.lens;
    let fam = family(current);
    let list = items(fam);
    let height = height(current);
    let rows = list.len().div_ceil(2) as f32;
    let panel = Rect::from_min_size(origin, Vec2::new(WIDTH, height));
    bf_ui_kit::paint_overlay(ui, panel);
    ui.interact(
        panel,
        ui.make_persistent_id("lens-panel"),
        Sense::click_and_drag(),
    );

    let mut next = current;
    let inner = panel.shrink(space::LG);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    let ui = &mut child;
    ui.spacing_mut().item_spacing.y = space::SM;
    ui.horizontal(|ui| {
        bf_ui_kit::caption(ui, &i18n.tr("lens-title"));
        if let Some(pinned) = pinned {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let text = i18n.tr(if pinned { "lens-fold" } else { "lens-pin" });
                if ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new(text)
                                .font(sans(12.0))
                                .color(color::ACCENT),
                        )
                        .sense(Sense::click())
                        .selectable(false),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    toggle_pin = true;
                }
            });
        }
    });
    let mut f = fam;
    let options = vec![
        (Family::Political, i18n.tr("lens-political")),
        (Family::Resources, i18n.tr("lens-resources")),
        (Family::Climate, i18n.tr("lens-climate")),
        (Family::Farms, i18n.tr("lens-farms")),
        (Family::Infrastructure, i18n.tr("lens-infrastructure")),
    ];
    if bf_ui_kit::segmented(ui, &mut f, &options) {
        chosen = items(f).is_empty();
        next = match f {
            Family::Political => Lens::Political,
            Family::Resources => {
                Lens::Resource(RESOURCES.iter().position(|r| *r == "copper").unwrap_or(0))
            }
            Family::Climate => Lens::Climate,
            Family::Farms => Lens::Farm(0),
            Family::Infrastructure => Lens::Local(lens::LOCAL_DEFAULT),
        };
    }

    if !list.is_empty() {
        let selected = match current {
            Lens::Resource(i) | Lens::Farm(i) | Lens::Local(i) => i,
            _ => usize::MAX,
        };
        let col_w = (inner.width() - space::SM) / 2.0;
        let top = ui.cursor().top();
        for (i, key) in list.iter().enumerate() {
            let r = Rect::from_min_size(
                Pos2::new(
                    inner.left() + (i % 2) as f32 * (col_w + space::SM),
                    top + (i / 2) as f32 * 24.0,
                ),
                Vec2::new(col_w, 22.0),
            );
            let id = ui.make_persistent_id(("lens-item", i));
            let resp = ui.interact(r, id, Sense::click());
            let label = match fam {
                Family::Resources => i18n.tr(&format!("resource-{key}")),
                Family::Infrastructure => i18n.tr(&format!("good-{key}")),
                _ => i18n.tr(&format!("building-{key}")),
            };
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label.clone()));
            let on = i == selected;
            let painter = ui.painter();
            if on {
                painter.rect(
                    r,
                    radius::SM,
                    color::ACCENT_SOFT,
                    Stroke::new(1.0, color::ACCENT.gamma_multiply(0.6)),
                    egui::StrokeKind::Inside,
                );
            } else if resp.hovered() {
                painter.rect_filled(r, radius::SM, color::SURFACE_HOVER);
            }
            painter.text(
                Pos2::new(r.left() + 8.0, r.center().y),
                Align2::LEFT_CENTER,
                &label,
                sans(12.5),
                if on { color::ACCENT } else { color::TEXT },
            );
            if resp
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                chosen = true;
                next = match fam {
                    Family::Resources => Lens::Resource(i),
                    Family::Infrastructure => Lens::Local(i),
                    _ => Lens::Farm(i),
                };
            }
        }
        ui.advance_cursor_after_rect(Rect::from_min_size(
            Pos2::new(inner.left(), top),
            Vec2::new(inner.width(), rows * 24.0),
        ));
    }

    legend(ui, i18n, current, world);

    if next != current {
        choose(view, world, next, year);
    }
    LensResponse { toggle_pin, chosen }
}

fn ramp(ui: &mut Ui, from: [u8; 4], to: [u8; 4], low: &str, high: &str) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::hover());
    let bar = Rect::from_min_size(rect.min, Vec2::new(w, 10.0));
    let mut mesh = egui::Mesh::default();
    let c = |x: [u8; 4]| Color32::from_rgb(x[0], x[1], x[2]);
    mesh.colored_vertex(bar.left_top(), c(from));
    mesh.colored_vertex(bar.right_top(), c(to));
    mesh.colored_vertex(bar.right_bottom(), c(to));
    mesh.colored_vertex(bar.left_bottom(), c(from));
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    let painter = ui.painter();
    painter.add(mesh);
    painter.rect_stroke(
        bar,
        2,
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    painter.text(
        Pos2::new(bar.left(), bar.bottom() + 4.0),
        Align2::LEFT_TOP,
        low,
        sans(11.0),
        color::TEXT_MUTED,
    );
    painter.text(
        Pos2::new(bar.right(), bar.bottom() + 4.0),
        Align2::RIGHT_TOP,
        high,
        sans(11.0),
        color::TEXT_MUTED,
    );
}

fn legend(ui: &mut Ui, i18n: &I18n, current: Lens, world: &World) {
    if current == Lens::Political {
        return;
    }
    if let Lens::Local(_) = current {
        if world.live_local.is_empty() {
            bf_ui_kit::small(ui, &i18n.tr("lens-no-data"));
            return;
        }
        bf_ui_kit::caption(ui, &i18n.tr("legend-local"));
        ramp(
            ui,
            lens::met_color(0.0),
            lens::met_color(1.0),
            &i18n.tr("legend-unmet"),
            &i18n.tr("legend-met"),
        );
        return;
    }
    if !world.has_resources() {
        bf_ui_kit::small(ui, &i18n.tr("lens-no-data"));
        return;
    }
    match current {
        Lens::Resource(i) => {
            let key = RESOURCES[i];
            bf_ui_kit::caption(ui, &i18n.tr("legend-deposit"));
            ramp(
                ui,
                lens::ramp_color(key, 1),
                lens::ramp_color(key, 5),
                &i18n.tr("legend-small"),
                &i18n.tr("legend-large"),
            );
        }
        Lens::Farm(_) => {
            bf_ui_kit::caption(ui, &i18n.tr("legend-suitability"));
            ramp(
                ui,
                [196, 188, 170, 255],
                [40, 120, 50, 255],
                &i18n.tr("legend-low"),
                &i18n.tr("legend-high"),
            );
        }
        Lens::Climate => {
            let w = ui.available_width();
            let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 7.0 * 20.0), Sense::hover());
            let painter = ui.painter();
            for (i, key) in CLIMATES.iter().enumerate() {
                let x = rect.left() + (i % 2) as f32 * (w / 2.0);
                let y = rect.top() + (i / 2) as f32 * 20.0;
                let c = lens::CLIMATE_COLORS[i];
                painter.rect_filled(
                    Rect::from_min_size(Pos2::new(x, y + 3.0), Vec2::splat(12.0)),
                    2,
                    Color32::from_rgb(c[0], c[1], c[2]),
                );
                painter.text(
                    Pos2::new(x + 18.0, y + 9.0),
                    Align2::LEFT_CENTER,
                    i18n.tr(&format!("climate-{key}")),
                    sans(11.5),
                    color::TEXT,
                );
            }
        }
        Lens::Political | Lens::Local(_) => {}
    }
}

/// Extra tooltip line for the state under the pointer in the current lens.
pub fn tooltip_line(
    i18n: &I18n,
    world: &World,
    state: usize,
    lens: Lens,
    year: i32,
) -> Option<String> {
    match lens {
        Lens::Political => None,
        Lens::Local(i) => {
            let v = world.live_local.get(state)?.get(i).copied()?;
            if v < 0.0 {
                return None;
            }
            Some(i18n.tr_args(
                "tip-local",
                &[
                    (
                        "good",
                        Arg::String(i18n.tr(&format!("good-{}", lens::LOCAL_GOODS[i]))),
                    ),
                    (
                        "share",
                        Arg::String(format!("{}%", i18n.decimal(f64::from(v) * 100.0, 0))),
                    ),
                ],
            ))
        }
        Lens::Resource(i) => {
            let key = RESOURCES[i];
            let d = world.deposits[state]
                .iter()
                .filter(|d| d.resource == key && d.discovered <= year)
                .max_by_key(|d| d.size)?;
            Some(i18n.tr_args(
                "tip-deposit",
                &[
                    ("resource", Arg::String(i18n.tr(&format!("resource-{key}")))),
                    ("size", Arg::Num(d.size as f64)),
                ],
            ))
        }
        Lens::Climate => {
            let land = world.land[state].as_ref()?;
            let (key, share) = land.climate.iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
            Some(i18n.tr_args(
                "tip-climate",
                &[
                    ("climate", Arg::String(i18n.tr(&format!("climate-{key}")))),
                    (
                        "share",
                        Arg::String(format!("{}%", i18n.decimal(*share as f64 * 100.0, 0))),
                    ),
                ],
            ))
        }
        Lens::Farm(i) => {
            let key = FARMS[i];
            let land = world.land[state].as_ref()?;
            let v = land.farms.iter().find(|(k, _)| k == key).map(|(_, v)| *v)?;
            Some(i18n.tr_args(
                "tip-farm",
                &[
                    ("farm", Arg::String(i18n.tr(&format!("building-{key}")))),
                    ("value", Arg::String(i18n.decimal(v as f64, 2))),
                ],
            ))
        }
    }
}
