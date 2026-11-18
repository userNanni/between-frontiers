//! Map mode panel at the bottom left: political, resources, climate, land use, with a legend.

use bf_i18n::{Arg, I18n};
use bf_sim::resources::{CLIMATES, FARMS, RESOURCES};
use bf_ui_kit::theme::sans;
use bf_ui_kit::tokens::{color, radius, space};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::keys::{self, Tip, id};
use crate::map::MapView;
use crate::map::lens::{self, Lens};
use crate::world::World;

// Wide enough for the seven families in English and Portuguese, as the kit draws them
// (`the_lens_families_fit_across_the_panel` measures them).
const WIDTH: f32 = 592.0;
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
    Sea,
    /// Ties between countries: influence and blocs.
    Ties,
}

/// The ties family's list: influence both ways, then the blocs in the order of
/// `blocs_view::KINDS`, then the wars (`lens-ties-*`).
const TIES_ITEMS: [&str; 6] = [
    "influence_over",
    "influence_on",
    "military",
    "monetary",
    "subjects",
    "war",
];
/// Index of the war lens in [`TIES_ITEMS`].
const TIES_WAR: usize = 5;

/// Index of a ties lens in [`TIES_ITEMS`].
fn ties_index(lens: Lens) -> Option<usize> {
    match lens {
        Lens::Influence { on, .. } => Some(usize::from(on)),
        Lens::Blocs(k) => Some(2 + k),
        Lens::War => Some(TIES_WAR),
        _ => None,
    }
}

fn ties_lens(i: usize) -> Lens {
    match i {
        0 | 1 => Lens::Influence {
            issue: 0,
            on: i == 1,
        },
        TIES_WAR => Lens::War,
        k => Lens::Blocs(k - 2),
    }
}

fn family(lens: Lens) -> Family {
    match lens {
        Lens::Political => Family::Political,
        Lens::Resource(_) => Family::Resources,
        Lens::Climate => Family::Climate,
        Lens::Farm(_) => Family::Farms,
        Lens::Local(_) => Family::Infrastructure,
        Lens::Sea => Family::Sea,
        Lens::Influence { .. } | Lens::Blocs(_) | Lens::War => Family::Ties,
    }
}

fn items(fam: Family) -> &'static [&'static str] {
    match fam {
        Family::Resources => RESOURCES,
        Family::Farms => FARMS,
        Family::Infrastructure => &lens::LOCAL_GOODS,
        Family::Ties => &TIES_ITEMS,
        _ => &[],
    }
}

// ---------------------------------------------------------------------------------------------
// Layout. The panel is measured from the same regions it is drawn in: a header, the families,
// the list of the family's items (scrolling inside when the room is short) and the legend,
// fixed at the foot. The heights of the families and the legend are what the last frame drew.

/// Padding inside the frame and the gap between regions.
const PAD: f32 = space::LG;
const GAP: f32 = space::SM;
const HEAD_H: f32 = 20.0;
/// Height of a row of the list (two items a row).
const ROW: f32 = 24.0;
/// The list keeps at least this many rows in view.
const MIN_ROWS: f32 = 2.0;

fn families_id() -> egui::Id {
    egui::Id::NULL.with("bf-lens-families")
}

fn legend_id(lens: Lens) -> egui::Id {
    egui::Id::NULL.with(("bf-lens-legend", family(lens) as u8))
}

/// Height of the families' row as drawn last; a guess before the first frame.
fn families_h(ctx: &egui::Context) -> f32 {
    ctx.data(|d| d.get_temp(families_id())).unwrap_or(36.0)
}

/// Height of the legend as drawn last; a guess before the first frame.
fn legend_h(ctx: &egui::Context, lens: Lens) -> f32 {
    let guess = match family(lens) {
        Family::Political => 0.0,
        Family::Climate => 7.0 * 20.0,
        Family::Sea => 96.0,
        Family::Ties => 80.0,
        _ => 56.0,
    };
    ctx.data(|d| d.get_temp(legend_id(lens))).unwrap_or(guess)
}

/// Where each part of the panel goes inside `frame`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LensLayout {
    pub frame: Rect,
    pub header: Rect,
    pub families: Rect,
    /// The part of the list in view; `None` for a family without a list.
    pub list: Option<Rect>,
    /// The whole list's height (more than the view when it scrolls).
    pub list_h: f32,
    pub legend: Option<Rect>,
}

impl LensLayout {
    /// Lays the regions out in `frame`, the list taking what the others leave.
    fn new(frame: Rect, lens: Lens, families_h: f32, legend_h: f32) -> Self {
        let inner = frame.shrink(PAD);
        let header = Rect::from_min_size(inner.min, Vec2::new(inner.width(), HEAD_H));
        let families = Rect::from_min_size(
            Pos2::new(inner.left(), header.bottom() + GAP),
            Vec2::new(inner.width(), families_h),
        );
        let legend = (legend_h > 0.0).then(|| {
            Rect::from_min_max(
                Pos2::new(inner.left(), inner.bottom() - legend_h),
                inner.max,
            )
        });
        let rows = items(family(lens)).len().div_ceil(2) as f32;
        let list = (rows > 0.0).then(|| {
            let bottom = legend.map_or(inner.bottom(), |l| l.top() - GAP);
            Rect::from_min_max(
                Pos2::new(inner.left(), families.bottom() + GAP),
                Pos2::new(inner.right(), bottom.max(families.bottom() + GAP)),
            )
        });
        Self {
            frame,
            header,
            families,
            list,
            list_h: rows * ROW,
            legend,
        }
    }
}

/// Height the panel needs with its whole list in view.
fn natural_h(lens: Lens, families_h: f32, legend_h: f32) -> f32 {
    let rows = items(family(lens)).len().div_ceil(2) as f32;
    let list = if rows > 0.0 { GAP + rows * ROW } else { 0.0 };
    let legend = if legend_h > 0.0 { GAP + legend_h } else { 0.0 };
    PAD * 2.0 + HEAD_H + GAP + families_h + list + legend
}

/// The least height: the list keeps two rows in view.
fn least_h(lens: Lens, families_h: f32, legend_h: f32) -> f32 {
    let rows = items(family(lens)).len().div_ceil(2) as f32;
    natural_h(lens, families_h, legend_h) - (rows - rows.min(MIN_ROWS)) * ROW
}

/// Size of the panel for a lens in at most `max_h` of height: its whole content, or less
/// with the list scrolling (never less than two rows of it).
pub fn size(ctx: &egui::Context, lens: Lens, max_h: f32) -> Vec2 {
    size_with(lens, max_h, families_h(ctx), legend_h(ctx, lens))
}

fn size_with(lens: Lens, max_h: f32, families_h: f32, legend_h: f32) -> Vec2 {
    let h = natural_h(lens, families_h, legend_h)
        .min(max_h)
        .max(least_h(lens, families_h, legend_h));
    Vec2::new(WIDTH, h)
}

/// The regions of the panel in `frame`, as `show` draws them.
pub fn layout(ctx: &egui::Context, lens: Lens, frame: Rect) -> LensLayout {
    LensLayout::new(frame, lens, families_h(ctx), legend_h(ctx, lens))
}

/// Width inside the frame, where the families have to fit.
pub fn inner_width() -> f32 {
    WIDTH - PAD * 2.0
}

/// The families' names, as the panel shows them.
pub fn family_labels(i18n: &I18n) -> Vec<String> {
    [
        "lens-political",
        "lens-resources",
        "lens-climate",
        "lens-farms",
        "lens-infrastructure",
        "lens-sea",
        "lens-ties",
    ]
    .iter()
    .map(|k| i18n.tr(k))
    .collect()
}

/// Every lens, one per family and every item: for checks that go through them all.
pub fn all_lenses() -> Vec<Lens> {
    let mut out = vec![Lens::Political, Lens::Climate];
    out.extend((0..RESOURCES.len()).map(Lens::Resource));
    out.extend((0..FARMS.len()).map(Lens::Farm));
    out.extend((0..lens::LOCAL_GOODS.len()).map(Lens::Local));
    out.push(Lens::Sea);
    out.extend((0..TIES_ITEMS.len()).map(ties_lens));
    out
}

/// A list item's name.
fn item_label(i18n: &I18n, fam: Family, key: &str) -> String {
    match fam {
        Family::Resources => i18n.tr(&format!("resource-{key}")),
        Family::Infrastructure => i18n.tr(&format!("good-{key}")),
        Family::Ties => i18n.tr(&format!("lens-ties-{key}")),
        _ => i18n.tr(&format!("building-{key}")),
    }
}

/// The lens in words: its family, and the resource, crop or good when it has one.
pub fn name(i18n: &I18n, lens: Lens) -> String {
    let fam = match family(lens) {
        Family::Political => "lens-political",
        Family::Resources => "lens-resources",
        Family::Climate => "lens-climate",
        Family::Farms => "lens-farms",
        Family::Infrastructure => "lens-infrastructure",
        Family::Sea => "lens-sea",
        Family::Ties => "lens-ties",
    };
    let fam = i18n.tr(fam);
    let item = match lens {
        Lens::Resource(i) => RESOURCES.get(i).map(|k| i18n.tr(&format!("resource-{k}"))),
        Lens::Farm(i) => FARMS.get(i).map(|k| i18n.tr(&format!("building-{k}"))),
        Lens::Local(i) => lens::LOCAL_GOODS
            .get(i)
            .map(|k| i18n.tr(&format!("good-{k}"))),
        Lens::Influence { .. } | Lens::Blocs(_) => ties_index(lens)
            .and_then(|i| TIES_ITEMS.get(i))
            .map(|k| item_label(i18n, Family::Ties, k)),
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
    let tip = keys::tip(ui.ctx(), &tip, id::LENS_PICKER);
    let resp = resp.on_hover_text(bf_ui_kit::tooltip_text(&tip));
    keys::pressed(ui, id::LENS_PICKER, &resp)
}

/// The lens a map mode command picks: the family's first lens, or the one in use when it is
/// already of that family. The influence and blocs commands switch their lens on and off: a
/// second press goes back to the political map.
pub fn mode_lens(mode: usize, current: Lens) -> Lens {
    let lens = match mode {
        0 => Lens::Political,
        1 => Lens::Resource(RESOURCES.iter().position(|r| *r == "copper").unwrap_or(0)),
        2 => Lens::Climate,
        3 => Lens::Farm(0),
        4 => Lens::Local(lens::LOCAL_DEFAULT),
        5 => Lens::Sea,
        6 => Lens::Influence {
            issue: 0,
            on: false,
        },
        _ => Lens::Blocs(0),
    };
    // The ties commands switch: a second press goes back to the political map.
    match (lens, current) {
        (Lens::Influence { .. }, Lens::Influence { .. }) | (Lens::Blocs(_), Lens::Blocs(_)) => {
            Lens::Political
        }
        (Lens::Influence { .. } | Lens::Blocs(_), _) => lens,
        _ if family(lens) == family(current) => current,
        _ => lens,
    }
}

/// Shows the panel in `frame` and recolours the map when the lens changes. Every part is drawn
/// in its region of [`layout`] and clipped to it, so nothing leaves the frame. With `pinned`
/// given, a link at the top keeps the panel open or folds it back into its button.
#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    frame: Rect,
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
    let lay = layout(ui.ctx(), current, frame);
    bf_ui_kit::paint_overlay(ui, frame);
    let block = ui.interact(
        frame,
        ui.make_persistent_id("lens-panel"),
        Sense::click_and_drag(),
    );
    let title = i18n.tr("lens-title");
    block.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Pane, true, title.clone()));
    let region = |ui: &mut Ui, r: Rect| {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
        child.set_clip_rect(r.intersect(ui.clip_rect()));
        child
    };

    // Header: the title and the pin.
    {
        let mut ui = region(ui, lay.header);
        ui.horizontal(|ui| {
            bf_ui_kit::caption(ui, &title);
            if let Some(pinned) = pinned {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let text = i18n.tr(if pinned { "lens-fold" } else { "lens-pin" });
                    let link = ui
                        .add(
                            egui::Label::new(
                                egui::RichText::new(&text)
                                    .font(sans(12.0))
                                    .color(color::ACCENT),
                            )
                            .sense(Sense::click())
                            .selectable(false),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    let link = link.on_hover_text(keys::tip(ui.ctx(), &text, id::LENS_KEEP_OPEN));
                    if keys::pressed(ui, id::LENS_KEEP_OPEN, &link) {
                        toggle_pin = true;
                    }
                });
            }
        });
    }

    // The families; their height is kept for the next measure.
    let mut next = current;
    {
        let mut ui = region(ui, lay.families);
        let mut f = fam;
        let options = vec![
            (Family::Political, i18n.tr("lens-political")),
            (Family::Resources, i18n.tr("lens-resources")),
            (Family::Climate, i18n.tr("lens-climate")),
            (Family::Farms, i18n.tr("lens-farms")),
            (Family::Infrastructure, i18n.tr("lens-infrastructure")),
            (Family::Sea, i18n.tr("lens-sea")),
            (Family::Ties, i18n.tr("lens-ties")),
        ];
        let tips = [
            Tip::Command(id::MODE_POLITICAL),
            Tip::Command(id::MODE_RESOURCES),
            Tip::Command(id::MODE_CLIMATE),
            Tip::Command(id::MODE_FARMS),
            Tip::Command(id::MODE_INFRASTRUCTURE),
            Tip::Command(id::MODE_SEA),
            Tip::Command(id::MODE_INFLUENCE),
        ];
        // The keys of these commands are read by the game screen, folded or not.
        let changed = keys::segmented(&mut ui, &mut f, &options, &tips);
        let used = ui.min_rect().height();
        ui.ctx().data_mut(|d| d.insert_temp(families_id(), used));
        if changed {
            chosen = items(f).is_empty();
            next = match f {
                Family::Political => Lens::Political,
                Family::Resources => {
                    Lens::Resource(RESOURCES.iter().position(|r| *r == "copper").unwrap_or(0))
                }
                Family::Climate => Lens::Climate,
                Family::Farms => Lens::Farm(0),
                Family::Infrastructure => Lens::Local(lens::LOCAL_DEFAULT),
                Family::Sea => Lens::Sea,
                Family::Ties => ties_lens(0),
            };
        }
    }

    // The list, scrolling inside its region when the room is short.
    if let Some(area) = lay.list {
        let selected = match current {
            Lens::Resource(i) | Lens::Farm(i) | Lens::Local(i) => i,
            Lens::Influence { .. } | Lens::Blocs(_) => ties_index(current).unwrap_or(usize::MAX),
            _ => usize::MAX,
        };
        let mut ui = region(ui, area);
        egui::ScrollArea::vertical()
            .id_salt(("lens-list", fam as u8))
            .max_height(area.height())
            .auto_shrink([false, false])
            .show(&mut ui, |ui| {
                let width = ui.available_width()
                    - if lay.list_h > area.height() {
                        12.0
                    } else {
                        0.0
                    };
                let (rows_rect, _) =
                    ui.allocate_exact_size(Vec2::new(width, lay.list_h), Sense::hover());
                let col_w = (width - space::SM) / 2.0;
                for (i, key) in list.iter().enumerate() {
                    let r = Rect::from_min_size(
                        Pos2::new(
                            rows_rect.left() + (i % 2) as f32 * (col_w + space::SM),
                            rows_rect.top() + (i / 2) as f32 * ROW,
                        ),
                        Vec2::new(col_w, ROW - 2.0),
                    );
                    let id = ui.make_persistent_id(("lens-item", i));
                    let resp = ui.interact(r, id, Sense::click());
                    let label = item_label(i18n, fam, key);
                    resp.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Button, true, label.clone())
                    });
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
                    let (g, _) = super::journal::one_line(
                        painter,
                        &label,
                        sans(12.5),
                        if on { color::ACCENT } else { color::TEXT },
                        r.width() - 12.0,
                    );
                    painter.galley(
                        Pos2::new(r.left() + 8.0, r.center().y - g.size().y / 2.0),
                        g,
                        color::TEXT,
                    );
                    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                    if resp.clicked() || keys::item(ui, 'W', format!("lens.item.{key}"), key, r) {
                        chosen = true;
                        next = match fam {
                            Family::Resources => Lens::Resource(i),
                            Family::Infrastructure => Lens::Local(i),
                            Family::Ties => ties_lens(i),
                            _ => Lens::Farm(i),
                        };
                    }
                }
            });
    }

    // The legend, fixed at the foot; its height is kept for the next measure.
    {
        let area = lay.legend.unwrap_or(Rect::from_min_size(
            Pos2::new(lay.header.left(), frame.bottom() - PAD),
            Vec2::new(lay.header.width(), 0.0),
        ));
        // Drawn from the top of its region; a taller legend than measured is clipped once and
        // measured whole for the next frame.
        let mut ui = region(
            ui,
            Rect::from_min_size(area.min, Vec2::new(area.width(), 400.0)),
        );
        ui.set_clip_rect(area.intersect(ui.clip_rect()));
        ui.spacing_mut().item_spacing.y = space::XS;
        let top = ui.cursor().top();
        legend(&mut ui, i18n, current, world);
        let used = ui.min_rect().bottom() - top;
        let used = if current == Lens::Political {
            0.0
        } else {
            used
        };
        ui.ctx()
            .data_mut(|d| d.insert_temp(legend_id(current), used.max(0.0)));
    }

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
    if current == Lens::Sea {
        sea_legend(ui, i18n);
        return;
    }
    if let Lens::Influence { issue, on } = current {
        let _ = issue;
        influence_legend(ui, i18n, world, on);
        return;
    }
    if let Lens::Blocs(kind) = current {
        blocs_legend(ui, i18n, world, kind);
        return;
    }
    if current == Lens::War {
        war_legend(ui, i18n, world);
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
        Lens::Political
        | Lens::Local(_)
        | Lens::Sea
        | Lens::Influence { .. }
        | Lens::Blocs(_)
        | Lens::War => {}
    }
}

/// The sea lens: the ice of each zone, and how to ask for a route.
fn sea_legend(ui: &mut Ui, i18n: &I18n) {
    use crate::map::sea::zone_tint;
    bf_ui_kit::caption(ui, &i18n.tr("legend-sea-ice"));
    ui.horizontal_wrapped(|ui| {
        for (key, fill, hatch) in [
            ("sea-ice-free", zone_tint(3).gamma_multiply(2.5), None),
            (
                "sea-ice-seasonal",
                zone_tint(3).gamma_multiply(2.5),
                Some(Color32::from_rgb(225, 235, 245)),
            ),
            (
                "sea-ice-permanent",
                Color32::from_rgb(200, 215, 230),
                Some(Color32::from_rgb(245, 248, 252)),
            ),
        ] {
            let (r, _) = ui.allocate_exact_size(Vec2::new(16.0, 12.0), Sense::hover());
            ui.painter().rect_filled(r, 2, fill);
            if let Some(h) = hatch {
                for k in 0..3 {
                    let x = r.left() + 3.0 + k as f32 * 5.0;
                    ui.painter().line_segment(
                        [Pos2::new(x, r.bottom()), Pos2::new(x + 4.0, r.top())],
                        Stroke::new(1.0, h),
                    );
                }
            }
            ui.label(
                egui::RichText::new(i18n.tr(key))
                    .font(sans(11.5))
                    .color(color::TEXT),
            );
            ui.add_space(8.0);
        }
    });
    bf_ui_kit::small(ui, &i18n.tr("legend-sea-route"));
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
        Lens::Political | Lens::Sea => None,
        Lens::Influence { issue, on } => influence_tip(i18n, world, state, issue, on),
        Lens::Blocs(kind) => blocs_tip(i18n, world, state, kind),
        Lens::War => war_tip(i18n, world, state),
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

/// A share of GDP as a percentage: two decimals, influence is small.
fn gdp_pct(i18n: &I18n, x: f64) -> String {
    format!("{} %", i18n.decimal(x * 100.0, 2))
}

/// A colour swatch with its label beside it: every colour of a legend says what it is.
fn swatch(ui: &mut Ui, fill: [u8; 4], hatch: bool, label: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::new(16.0, 12.0), Sense::hover());
        ui.painter()
            .rect_filled(r, 2, Color32::from_rgb(fill[0], fill[1], fill[2]));
        if hatch {
            for k in 0..3 {
                let x = r.left() + 2.0 + k as f32 * 5.0;
                ui.painter().line_segment(
                    [Pos2::new(x, r.top()), Pos2::new(x + 5.0, r.bottom())],
                    Stroke::new(1.2, Color32::from_black_alpha(140)),
                );
            }
        }
        ui.painter().rect_stroke(
            r,
            2,
            Stroke::new(1.0, color::BORDER),
            egui::StrokeKind::Inside,
        );
        ui.add(
            egui::Label::new(
                egui::RichText::new(label)
                    .font(sans(11.5))
                    .color(color::TEXT),
            )
            .wrap(),
        );
    });
}

/// The influence lens: its unit, the ramp with its marks, the country in focus and "nothing
/// measured". The largest issue and its channels are in the tooltip.
fn influence_legend(ui: &mut Ui, i18n: &I18n, world: &World, on: bool) {
    let Some(v) = world.live_influence.as_deref() else {
        bf_ui_kit::small(ui, &i18n.tr("lens-influence-none"));
        return;
    };
    let focus = crate::text::country_name(i18n, &v.tag);
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            if on {
                "legend-influence-unit-on"
            } else {
                "legend-influence-unit-over"
            },
            &[
                ("country", Arg::String(focus.clone())),
                (
                    "article",
                    Arg::String(crate::text::country_article(i18n, &v.tag)),
                ),
            ],
        ),
    );
    // The ramp with a mark at each decade: 0.01 %, 0.1 %, 1 %, 10 %.
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 22.0), Sense::hover());
    let bar = Rect::from_min_size(rect.min, Vec2::new(w, 7.0));
    let steps = 24;
    let painter = ui.painter();
    for k in 0..steps {
        let t0 = k as f32 / steps as f32;
        let t1 = (k + 1) as f32 / steps as f32;
        let share = 10f64.powf(f64::from(t0) * 3.0 - 4.0);
        let c = lens::influence_color(share);
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(bar.left() + t0 * w, bar.top()),
                Pos2::new(bar.left() + t1 * w, bar.bottom()),
            ),
            0,
            Color32::from_rgb(c[0], c[1], c[2]),
        );
    }
    painter.rect_stroke(
        bar,
        2,
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    for (k, share) in lens::INFLUENCE_DECADES.iter().enumerate() {
        let x = bar.left() + lens::influence_t(*share) * w;
        let align = match k {
            0 => Align2::LEFT_TOP,
            3 => Align2::RIGHT_TOP,
            _ => Align2::CENTER_TOP,
        };
        painter.text(
            Pos2::new(x, bar.bottom() + 2.0),
            align,
            decade(i18n, *share),
            sans(11.0),
            color::TEXT_MUTED,
        );
    }
    ui.horizontal_wrapped(|ui| {
        swatch(
            ui,
            [lens::FOCUS[0], lens::FOCUS[1], lens::FOCUS[2], 255],
            false,
            &i18n.tr_args("legend-influence-focus", &[("country", Arg::String(focus))]),
        );
        ui.add_space(space::SM);
        swatch(
            ui,
            [lens::EMPTY[0], lens::EMPTY[1], lens::EMPTY[2], 255],
            false,
            &i18n.tr("legend-influence-nothing"),
        );
        ui.add_space(space::SM);
        swatch(
            ui,
            lens::UNMEASURED,
            true,
            &i18n.tr("legend-influence-unmeasured"),
        );
    });
    // What is measured today, in one line; the whole story on hover.
    ui.add(
        egui::Label::new(
            egui::RichText::new(i18n.tr("legend-influence-scope"))
                .font(sans(11.5))
                .color(color::TEXT_MUTED),
        )
        .sense(Sense::hover()),
    )
    .on_hover_text(bf_ui_kit::tooltip_text(
        &i18n.tr("legend-influence-scope-tip"),
    ));
}

/// A mark of the influence ramp: 0,01 %, 0,1 %, 1 %, 10 %.
fn decade(i18n: &I18n, share: f64) -> String {
    let pct = share * 100.0;
    let digits = if pct >= 1.0 {
        0
    } else {
        (-pct.log10()).ceil() as usize
    };
    format!("{} %", i18n.decimal(pct, digits))
}

/// Rows of blocs the legend shows before it scrolls.
const BLOC_ROWS: f32 = 3.0;

/// The blocs lens: each bloc of the kind with its colour, name, leader and members; the
/// leader's tone and the customs hatching.
fn blocs_legend(ui: &mut Ui, i18n: &I18n, world: &World, kind: usize) {
    let Some(v) = world.live_blocs.as_deref() else {
        bf_ui_kit::small(ui, &i18n.tr("lens-blocs-none"));
        return;
    };
    let order = lens::blocs_of_kind(v, kind);
    if order.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("lens-blocs-empty"));
        return;
    }
    // Every bloc with its colour and name, in a short list that scrolls: no colour without
    // its label.
    egui::ScrollArea::vertical()
        .id_salt(("lens-blocs", kind))
        .max_height(BLOC_ROWS * 18.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, b) in order.iter().enumerate() {
                swatch(
                    ui,
                    lens::bloc_color(i, true),
                    false,
                    &bloc_line(i18n, &v.blocs[*b]),
                );
            }
        });
    ui.horizontal_wrapped(|ui| {
        // The tones only differ where a bloc has a leader.
        if order.iter().any(|b| v.blocs[*b].leader.is_some()) {
            swatch(
                ui,
                lens::bloc_color(0, false),
                false,
                &i18n.tr("legend-blocs-member"),
            );
        }
        if kind == 0 {
            ui.add_space(space::SM);
            swatch(
                ui,
                [lens::EMPTY[0], lens::EMPTY[1], lens::EMPTY[2], 255],
                true,
                &i18n.tr("legend-blocs-customs"),
            );
        }
    });
}

/// A bloc's name: the organization's when it has one, else by kind and leader, else by members.
pub fn bloc_name(i18n: &I18n, b: &crate::blocs_view::BlocLine) -> String {
    let org = format!("org-{}", b.key);
    if i18n.has(&org) {
        return i18n.tr(&org);
    }
    let kind = match b.kind {
        bf_sim::influence::BlocKind::Military => "military",
        bf_sim::influence::BlocKind::Monetary => "monetary",
        bf_sim::influence::BlocKind::Subjects => "subjects",
        bf_sim::influence::BlocKind::Customs => "customs",
    };
    match &b.leader {
        Some(l) => i18n.tr_args(
            &format!("bloc-name-{kind}"),
            &[
                ("leader", Arg::String(crate::text::country_name(i18n, l))),
                (
                    "article",
                    Arg::String(crate::text::country_article(i18n, l)),
                ),
            ],
        ),
        None => b
            .members
            .iter()
            .map(|t| crate::text::country_name(i18n, t))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn bloc_line(i18n: &I18n, b: &crate::blocs_view::BlocLine) -> String {
    i18n.tr_args(
        if b.leader.is_some() {
            "legend-bloc-line"
        } else {
            "legend-bloc-leaderless"
        },
        &[
            ("name", Arg::String(bloc_name(i18n, b))),
            ("members", Arg::String(b.members.len().to_string())),
            ("count", Arg::Num(b.members.len() as f64)),
        ],
    )
}

fn owner_tag(world: &World, state: usize) -> Option<&str> {
    let c = world.states.get(state)?.owner?;
    Some(world.countries.get(c)?.tag())
}

/// The influence under the pointer: the share, the largest issue and its channels.
fn influence_tip(
    i18n: &I18n,
    world: &World,
    state: usize,
    issue: usize,
    on: bool,
) -> Option<String> {
    let v = world.live_influence.as_deref()?;
    let tag = owner_tag(world, state)?;
    if tag == v.tag {
        return Some(i18n.tr("tip-influence-focus"));
    }
    let pairs = if on { &v.on } else { &v.over };
    let Some(p) = pairs.iter().find(|p| p.other == tag) else {
        return Some(i18n.tr("tip-influence-nothing"));
    };
    if p.top() <= 0.0 {
        return Some(i18n.tr("tip-influence-unmeasured"));
    }
    let k = if issue == 0 {
        p.issues
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.influence.total_cmp(&b.1.influence))
            .map_or(0, |(k, _)| k)
    } else {
        issue - 1
    };
    let row = p.issues.get(k)?;
    let mut parts: Vec<String> = row
        .channels
        .iter()
        .filter(|c| c.part > 0.0)
        .map(|c| {
            format!(
                "{} {}",
                i18n.tr(super::influence::CHANNEL_KEYS[c.kind.min(3)]),
                gdp_pct(i18n, c.part)
            )
        })
        .collect();
    if row.goodwill > 0.0 {
        parts.push(i18n.tr_args(
            "influence-goodwill",
            &[("part", Arg::String(gdp_pct(i18n, row.goodwill)))],
        ));
    }
    Some(i18n.tr_args(
        "tip-influence",
        &[
            ("share", Arg::String(gdp_pct(i18n, row.influence))),
            (
                "issue",
                Arg::String(i18n.tr(super::influence::ISSUE_KEYS[k.min(4)])),
            ),
            ("channels", Arg::String(parts.join(" · "))),
        ],
    ))
}

/// The bloc under the pointer, and the others of the kind it is in.
/// The war lens: what the colours say.
fn war_legend(ui: &mut Ui, i18n: &I18n, world: &World) {
    let Some(v) = world.live_control.as_deref() else {
        bf_ui_kit::small(ui, &i18n.tr("legend-war-none"));
        return;
    };
    // The samples take the colour of a country at war on the map, so they read as it does;
    // outside the wars, a neutral grey faded the same way.
    let c = v
        .at_war
        .first()
        .and_then(|t| world.countries.iter().find(|c| c.tag() == t))
        .map_or((120, 112, 98), |c| c.def.color);
    let sample = [c.0, c.1, c.2, 255];
    swatch(ui, sample, false, &i18n.tr("legend-war-at-war"));
    swatch(ui, sample, true, &i18n.tr("legend-war-occupied"));
    swatch(
        ui,
        lens::faded((150, 150, 150)),
        false,
        &i18n.tr("legend-war-outside"),
    );
    bf_ui_kit::small(
        ui,
        &i18n.tr_args(
            "legend-war-count",
            &[
                ("wars", Arg::String(i18n.int(v.at_war.len() as i64))),
                ("states", Arg::String(i18n.int(v.occupied.len() as i64))),
            ],
        ),
    );
}

/// Who holds the state under the pointer, in the wars.
fn war_tip(i18n: &I18n, world: &World, state: usize) -> Option<String> {
    let v = world.live_control.as_deref()?;
    let owner = owner_tag(world, state)?;
    let id = world.states.get(state)?.def.id;
    let name = |t: &str| crate::text::country_name(i18n, t);
    Some(match v.occupier(id) {
        Some(by) => i18n.tr_args(
            "tip-war-occupied",
            &[
                ("by", Arg::String(name(by))),
                ("owner", Arg::String(name(owner))),
            ],
        ),
        None if v.at_war.iter().any(|t| t == owner) => i18n.tr("tip-war-at-war"),
        None => i18n.tr("tip-war-outside"),
    })
}

fn blocs_tip(i18n: &I18n, world: &World, state: usize, kind: usize) -> Option<String> {
    let v = world.live_blocs.as_deref()?;
    let tag = owner_tag(world, state)?;
    let k = *crate::blocs_view::KINDS.get(kind)?;
    let mut lines: Vec<String> = v
        .blocs
        .iter()
        .filter(|b| b.kind == k && b.members.iter().any(|m| m == tag))
        .map(|b| {
            if b.leader.as_deref() == Some(tag) {
                i18n.tr_args(
                    "tip-bloc-leader",
                    &[("name", Arg::String(bloc_name(i18n, b)))],
                )
            } else {
                bloc_name(i18n, b)
            }
        })
        .collect();
    if kind == 0
        && v.blocs.iter().any(|b| {
            b.kind == bf_sim::influence::BlocKind::Customs && b.members.iter().any(|m| m == tag)
        })
    {
        lines.push(i18n.tr("legend-blocs-customs"));
    }
    if lines.is_empty() {
        return Some(i18n.tr("tip-bloc-none"));
    }
    Some(lines.join(" · "))
}
