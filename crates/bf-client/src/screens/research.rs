//! Research: the technology tree as a graph. Columns are levels (two per era, pushed right by the
//! requirements, each asking for more research), rows are fields; curves link each technology to
//! what it requires. Each card shows the stage the player's country has reached (science,
//! prototype, industrial, diffused). Drag to pan, scroll to zoom, click an open technology to
//! focus research on it.

use std::collections::BTreeSet;

use bf_i18n::{Arg, I18n};
use bf_sim::econ::Catalogue;
use bf_sim::research::{ERAS, FIELDS, LEVELS_PER_ERA, Stage, TechTree};
use bf_ui_kit::theme::{sans, sans_semibold};
use bf_ui_kit::tokens::{color, radius, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::epaint::CubicBezierShape;
use egui::text::{LayoutJob, TextWrapping};
use egui::{Align2, Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

use crate::sim::ResearchView;

/// Card size and gaps, in tree units (pixels at zoom 1).
const NODE: Vec2 = Vec2::new(212.0, 66.0);
const GAP_X: f32 = 84.0;
const GAP_Y: f32 = 12.0;
const LANE_PAD: f32 = 14.0;
/// Room on the left for the field names and on top for the level headers, in screen pixels.
const GUTTER: f32 = 124.0;
const HEADER: f32 = 40.0;
const MINIMAP: Vec2 = Vec2::new(220.0, 132.0);
const ZOOM_MIN: f32 = 0.15;
const ZOOM_MAX: f32 = 1.6;

#[derive(Default)]
pub struct ResearchResponse {
    pub close: bool,
    /// The player picked a focus (`Some(None)` hands the choice back to the country).
    pub focus: Option<Option<u16>>,
}

pub fn stage_color(stage: Stage) -> Color32 {
    match stage {
        Stage::Unknown => color::TEXT_FAINT,
        Stage::Science => Color32::from_rgb(0x7A, 0x9C, 0xC6),
        Stage::Prototype => color::ACCENT,
        Stage::Industrial => color::POSITIVE,
        Stage::Diffused => Color32::from_rgb(0xA8, 0xD8, 0x8E),
    }
}

/// Era shown when the screen opens: the one the calendar is in.
pub fn current_era(year: i32) -> u8 {
    ERAS.iter().rposition(|start| *start <= year).unwrap_or(0) as u8
}

fn open(tree: &TechTree, view: &ResearchView, t: usize) -> bool {
    view.stages[t] == Stage::Unknown
        && tree.techs[t]
            .requires
            .iter()
            .all(|&r| view.stages[r as usize] >= Stage::Science)
}

/// Where each card sits in the tree, in tree units.
pub struct TreeLayout {
    pub pos: Vec<Pos2>,
    /// Field lanes: top and bottom.
    pub lanes: Vec<(f32, f32)>,
    pub levels: usize,
    /// Median cost of each level.
    pub level_cost: Vec<u32>,
    pub size: Vec2,
    /// For each technology, the ones that require it.
    pub dependents: Vec<Vec<u16>>,
}

fn column_x(level: usize) -> f32 {
    level as f32 * (NODE.x + GAP_X)
}

impl TreeLayout {
    pub fn new(tree: &TechTree) -> Self {
        let n = tree.techs.len();
        let levels = usize::from(tree.levels());
        let mut dependents = vec![Vec::new(); n];
        for (t, tech) in tree.techs.iter().enumerate() {
            for &r in &tech.requires {
                dependents[r as usize].push(t as u16);
            }
        }
        // Cells: the technologies of one field in one level, first in historical order.
        let mut cells = vec![vec![Vec::new(); levels]; FIELDS.len()];
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&t| (tree.techs[t].demonstrated, t));
        for t in order {
            let tech = &tree.techs[t];
            cells[usize::from(tech.field)][usize::from(tech.level)].push(t);
        }
        let slot = NODE.y + GAP_Y;
        let mut lanes = Vec::with_capacity(FIELDS.len());
        let mut top = 0.0;
        for field in &cells {
            let most = field.iter().map(Vec::len).max().unwrap_or(0).max(1);
            let height = most as f32 * slot - GAP_Y + slot * 0.5 + 2.0 * LANE_PAD;
            lanes.push((top, top + height));
            top += height;
        }
        let mut pos = vec![Pos2::ZERO; n];
        for (f, field) in cells.iter().enumerate() {
            for (l, cell) in field.iter().enumerate() {
                for (k, &t) in cell.iter().enumerate() {
                    pos[t] = Pos2::new(column_x(l), lanes[f].0 + LANE_PAD + k as f32 * slot);
                }
            }
        }
        // Pull each card toward the cards it links to, so the curves run as flat as they can
        // (the barycentre method of layered graph drawing), keeping the cards of a cell apart
        // and inside their lane.
        for pass in 0..6 {
            let mut desired = vec![0.0f32; n];
            for t in 0..n {
                let links: &[u16] = if pass % 2 == 0 && !tree.techs[t].requires.is_empty() {
                    &tree.techs[t].requires
                } else if !dependents[t].is_empty() {
                    &dependents[t]
                } else {
                    &tree.techs[t].requires
                };
                desired[t] = if links.is_empty() {
                    pos[t].y
                } else {
                    links.iter().map(|&r| pos[r as usize].y).sum::<f32>() / links.len() as f32
                };
            }
            for (f, field) in cells.iter_mut().enumerate() {
                let (lane_top, lane_bottom) = (lanes[f].0 + LANE_PAD, lanes[f].1 - LANE_PAD);
                for cell in field.iter_mut() {
                    cell.sort_by(|&a, &b| desired[a].total_cmp(&desired[b]).then(a.cmp(&b)));
                    let mut next = lane_top;
                    for &t in cell.iter() {
                        let y = desired[t].clamp(lane_top, lane_bottom - NODE.y).max(next);
                        pos[t].y = y;
                        next = y + slot;
                    }
                    let mut limit = lane_bottom - NODE.y;
                    for &t in cell.iter().rev() {
                        pos[t].y = pos[t].y.min(limit);
                        limit = pos[t].y - slot;
                    }
                }
            }
        }
        let mut level_cost = Vec::with_capacity(levels);
        for l in 0..levels {
            let mut costs: Vec<u32> = tree
                .techs
                .iter()
                .filter(|t| usize::from(t.level) == l)
                .map(|t| t.cost)
                .collect();
            costs.sort_unstable();
            level_cost.push(costs.get(costs.len() / 2).copied().unwrap_or(0));
        }
        Self {
            pos,
            lanes,
            levels,
            level_cost,
            size: Vec2::new(column_x(levels) - GAP_X, top),
            dependents,
        }
    }
}

/// Pan and zoom of the tree, kept while the game runs.
pub struct TreeCamera {
    pub pan: Vec2,
    pub zoom: f32,
    pub era: u8,
    layout: Option<TreeLayout>,
    /// Era to bring into view on the next frame.
    jump: Option<u8>,
}

impl TreeCamera {
    pub fn new(era: u8) -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 0.8,
            era,
            layout: None,
            jump: Some(era),
        }
    }

    fn to_screen(&self, canvas: Rect, p: Pos2) -> Pos2 {
        canvas.min + self.pan + p.to_vec2() * self.zoom
    }

    fn to_tree(&self, canvas: Rect, p: Pos2) -> Pos2 {
        ((p - canvas.min - self.pan) / self.zoom).to_pos2()
    }

    fn zoom_at(&mut self, canvas: Rect, pointer: Pos2, factor: f32) {
        let before = self.to_tree(canvas, pointer);
        self.zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
        self.pan = pointer - canvas.min - before.to_vec2() * self.zoom;
    }

    fn clamp(&mut self, canvas: Rect, size: Vec2) {
        let content = size * self.zoom;
        let margin = canvas.size() * 0.5;
        self.pan.x = self
            .pan
            .x
            .clamp((canvas.width() - content.x - margin.x).min(0.0), margin.x);
        self.pan.y = self
            .pan
            .y
            .clamp((canvas.height() - content.y - margin.y).min(0.0), margin.y);
    }
}

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    tree: &TechTree,
    cat: &Catalogue,
    view: &ResearchView,
    camera: &mut TreeCamera,
) -> ResearchResponse {
    let mut out = ResearchResponse::default();
    let layout = camera
        .layout
        .take()
        .unwrap_or_else(|| TreeLayout::new(tree));
    ui.painter()
        .rect_filled(screen, 0, Color32::from_black_alpha(120));
    let panel = Rect::from_min_max(
        screen.min + Vec2::new(space::XL, 72.0),
        screen.max - Vec2::splat(space::XL),
    );
    bf_ui_kit::paint_overlay(ui, panel);
    let block = ui.interact(
        screen,
        ui.make_persistent_id("research-block"),
        Sense::click(),
    );
    if block.clicked()
        && !ui
            .input(|i| i.pointer.interact_pos())
            .is_some_and(|p| panel.contains(p))
    {
        out.close = true;
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(panel.shrink(space::LG)));
    let ui = &mut child;
    ui.spacing_mut().item_spacing.y = space::SM;

    // Header: title, effort, focus, and a jump to each era.
    ui.horizontal(|ui| {
        bf_ui_kit::title(ui, &i18n.tr("research-title"));
        ui.add_space(space::LG);
        let focus = view
            .focus
            .map(|t| i18n.tr(&format!("tech-{}", tree.techs[t as usize].key)))
            .unwrap_or_else(|| i18n.tr("research-focus-auto"));
        bf_ui_kit::muted(
            ui,
            &i18n.tr_args(
                "research-summary",
                &[
                    ("points", Arg::String(format!("{:.0}", view.points))),
                    ("focus", Arg::String(focus)),
                ],
            ),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::ghost(&i18n.tr("research-close"))
                .size(ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                out.close = true;
            }
            if view.focus.is_some()
                && Button::secondary(&i18n.tr("research-focus-clear"))
                    .size(ButtonSize::Small)
                    .show(ui)
                    .clicked()
            {
                out.focus = Some(None);
            }
        });
    });
    let eras: Vec<(u8, String)> = (0..ERAS.len() as u8)
        .map(|e| (e, i18n.tr(&format!("tech-era-{e}"))))
        .collect();
    if bf_ui_kit::segmented(ui, &mut camera.era, &eras) {
        camera.jump = Some(camera.era);
    }

    let canvas = Rect::from_min_max(
        Pos2::new(ui.max_rect().left(), ui.cursor().top() + space::XS),
        ui.max_rect().max,
    );
    if let Some(f) = graph(ui, canvas, i18n, tree, cat, view, camera, &layout) {
        out.focus = Some(f);
    }
    camera.layout = Some(layout);
    out
}

#[allow(clippy::too_many_arguments)]
fn graph(
    ui: &mut Ui,
    canvas: Rect,
    i18n: &I18n,
    tree: &TechTree,
    cat: &Catalogue,
    view: &ResearchView,
    camera: &mut TreeCamera,
    layout: &TreeLayout,
) -> Option<Option<u16>> {
    let mut picked = None;
    // The tree itself, right of the field names and below the level headers.
    let area = Rect::from_min_max(canvas.min + Vec2::new(GUTTER, HEADER), canvas.max);
    let bg = ui.interact(
        area,
        ui.make_persistent_id("research-canvas"),
        Sense::click_and_drag(),
    );
    if let Some(era) = camera.jump.take() {
        let level = usize::from(era * LEVELS_PER_ERA).min(layout.levels.saturating_sub(1));
        camera.pan.x = space::LG - column_x(level) * camera.zoom;
        camera.pan.y = space::LG;
    }
    if bg.dragged() {
        camera.pan += bg.drag_delta();
    }
    if let Some(pointer) = bg.hover_pos() {
        let (scroll, pinch) = ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta()));
        let factor = pinch * (scroll.y * 0.0015).exp();
        if factor != 1.0 {
            camera.zoom_at(area, pointer, factor);
        }
        camera.pan.x += scroll.x;
    }
    camera.clamp(area, layout.size);
    // The era shown in the tabs follows the column at the left edge.
    let left = camera.to_tree(area, area.min).x;
    let level_at_left = ((left + GAP_X * 0.5) / (NODE.x + GAP_X)).max(0.0) as u8;
    camera.era = (level_at_left / LEVELS_PER_ERA).min(ERAS.len() as u8 - 1);

    let frame_painter = ui.painter_at(canvas);
    frame_painter.rect_filled(canvas, radius::MD, color::SURFACE_SUNKEN);
    let painter = ui.painter_at(area);
    let canvas = area;
    let z = camera.zoom;
    // Dotted background, as in node editors.
    let step = 24.0 * z;
    if step > 6.0 {
        let origin = canvas.min + camera.pan;
        let mut x = canvas.left() + (origin.x - canvas.left()).rem_euclid(step);
        while x < canvas.right() {
            let mut y = canvas.top() + (origin.y - canvas.top()).rem_euclid(step);
            while y < canvas.bottom() {
                painter.circle_filled(Pos2::new(x, y), 0.9, color::BORDER);
                y += step;
            }
            x += step;
        }
    }
    // Eras, as alternating bands of two levels each.
    for era in 0..ERAS.len() {
        let first = era * usize::from(LEVELS_PER_ERA);
        if first >= layout.levels {
            break;
        }
        let last = if era + 1 == ERAS.len() {
            layout.levels
        } else {
            (first + usize::from(LEVELS_PER_ERA)).min(layout.levels)
        };
        let a = camera
            .to_screen(canvas, Pos2::new(column_x(first) - GAP_X * 0.5, 0.0))
            .x;
        let b = camera
            .to_screen(canvas, Pos2::new(column_x(last) - GAP_X * 0.5, 0.0))
            .x;
        if era % 2 == 1 {
            painter.rect_filled(
                Rect::from_x_y_ranges(a..=b, canvas.y_range()),
                0,
                Color32::from_white_alpha(5),
            );
        }
    }
    // Field lanes.
    for (f, &(top, bottom)) in layout.lanes.iter().enumerate() {
        let a = camera.to_screen(canvas, Pos2::new(0.0, top));
        let b = camera.to_screen(canvas, Pos2::new(layout.size.x, bottom));
        if f % 2 == 1 {
            painter.rect_filled(Rect::from_min_max(a, b), 0, Color32::from_white_alpha(4));
        }
        painter.line_segment(
            [
                Pos2::new(canvas.left(), a.y),
                Pos2::new(canvas.right(), a.y),
            ],
            Stroke::new(1.0, color::BORDER),
        );
    }

    // What the pointer is on: that technology, what it needs and what it leads to.
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let over_canvas = pointer.filter(|p| canvas.contains(*p));
    let hovered = over_canvas.and_then(|p| {
        (0..tree.techs.len()).find(|&t| {
            Rect::from_min_size(camera.to_screen(canvas, layout.pos[t]), NODE * z).contains(p)
        })
    });
    let chain = hovered.or(view.focus.map(usize::from)).map(|t| {
        let mut set = BTreeSet::new();
        let mut stack = vec![t];
        while let Some(x) = stack.pop() {
            if set.insert(x) {
                stack.extend(tree.techs[x].requires.iter().map(|&r| r as usize));
            }
        }
        let mut stack = vec![t];
        let mut down = BTreeSet::new();
        while let Some(x) = stack.pop() {
            if down.insert(x) {
                stack.extend(layout.dependents[x].iter().map(|&d| d as usize));
            }
        }
        set.extend(down);
        set
    });
    let lit = |t: usize| chain.as_ref().is_none_or(|c| c.contains(&t));
    let dim = |c: Color32, t: usize| {
        if hovered.is_some() && !lit(t) {
            c.gamma_multiply(0.3)
        } else {
            c
        }
    };

    // Edges, behind the cards.
    let time = ui.input(|i| i.time) as f32;
    let mut animated = false;
    for (t, tech) in tree.techs.iter().enumerate() {
        let to = camera.to_screen(canvas, layout.pos[t]) + Vec2::new(0.0, NODE.y * z * 0.5);
        for &r in &tech.requires {
            let r = r as usize;
            let from =
                camera.to_screen(canvas, layout.pos[r]) + Vec2::new(NODE.x * z, NODE.y * z * 0.5);
            let span = Rect::from_two_pos(from, to);
            if !span.intersects(canvas) {
                continue;
            }
            let met = view.stages[r] >= Stage::Science;
            let on_chain = chain
                .as_ref()
                .is_some_and(|c| c.contains(&t) && c.contains(&r));
            let (col, width) = if on_chain {
                (color::ACCENT, 2.0)
            } else if met {
                (stage_color(view.stages[r]).gamma_multiply(0.55), 1.4)
            } else {
                (color::BORDER_STRONG, 1.2)
            };
            let col = if hovered.is_some() && !on_chain {
                col.gamma_multiply(0.25)
            } else {
                col
            };
            let dx = ((to.x - from.x) * 0.5).max(40.0 * z);
            let curve = CubicBezierShape::from_points_stroke(
                [from, from + Vec2::new(dx, 0.0), to - Vec2::new(dx, 0.0), to],
                false,
                Color32::TRANSPARENT,
                Stroke::new(width * z.max(0.6), col),
            );
            // The path to the technology in focus moves, like an animated edge.
            if on_chain && view.focus.is_some_and(|f| leads_to(tree, t, f as usize)) {
                animated = true;
                let points = curve.flatten(Some(0.5));
                painter.extend(Shape::dashed_line_with_offset(
                    &points,
                    Stroke::new(width * z.max(0.6), col),
                    &[7.0 * z],
                    &[5.0 * z],
                    (time * 24.0 * z).rem_euclid(12.0 * z),
                ));
            } else {
                painter.add(curve);
            }
        }
    }
    if animated {
        ui.ctx().request_repaint();
    }

    // Cards.
    for t in 0..tree.techs.len() {
        let r = Rect::from_min_size(camera.to_screen(canvas, layout.pos[t]), NODE * z);
        if !r.intersects(canvas) {
            continue;
        }
        let response = ui.interact(
            r.intersect(canvas),
            ui.make_persistent_id(("tech", t)),
            Sense::click(),
        );
        if let Some(f) = card(
            ui,
            &painter,
            r,
            z,
            i18n,
            tree,
            view,
            t,
            &response,
            !layout.dependents[t].is_empty(),
            &dim,
        ) {
            picked = Some(f);
        }
        response.on_hover_ui(|ui| tooltip(ui, i18n, tree, cat, view, t));
    }

    // Level headers, pinned to the top.
    let painter = frame_painter;
    let header = Rect::from_min_max(
        Pos2::new(area.left(), area.top() - HEADER),
        Pos2::new(area.right(), area.top()),
    );
    painter.rect_filled(header, 0, color::SURFACE);
    painter.line_segment(
        [header.left_bottom(), header.right_bottom()],
        Stroke::new(1.0, color::BORDER),
    );
    for (l, cost) in layout.level_cost.iter().enumerate() {
        let x = camera.to_screen(canvas, Pos2::new(column_x(l), 0.0)).x;
        if x > canvas.right() || x + NODE.x * z < canvas.left() {
            continue;
        }
        let era = l / usize::from(LEVELS_PER_ERA);
        if l % usize::from(LEVELS_PER_ERA) == 0 && era < ERAS.len() {
            painter.text(
                Pos2::new(x, header.top() + 5.0),
                Align2::LEFT_TOP,
                i18n.tr(&format!("tech-era-{era}")).to_uppercase(),
                sans_semibold(11.0),
                color::ACCENT,
            );
        }
        // As much of the level as fits over its column.
        let width = (NODE.x + GAP_X) * z;
        let n = Arg::String((l + 1).to_string());
        let label = if width > 230.0 {
            i18n.tr_args(
                "research-level",
                &[("n", n), ("cost", Arg::String(cost.to_string()))],
            )
        } else if width > 70.0 {
            i18n.tr_args("research-level-short", &[("n", n)])
        } else {
            (l + 1).to_string()
        };
        painter.text(
            Pos2::new(x, header.top() + 21.0),
            Align2::LEFT_TOP,
            label,
            sans(12.0),
            color::TEXT_MUTED,
        );
    }
    // Field names, pinned to the left.
    let gutter = Rect::from_min_max(
        Pos2::new(area.left() - GUTTER, area.top()),
        Pos2::new(area.left(), area.bottom()),
    );
    painter.rect_filled(gutter, 0, color::SURFACE);
    painter.line_segment(
        [gutter.right_top(), gutter.right_bottom()],
        Stroke::new(1.0, color::BORDER),
    );
    for (f, &(top, bottom)) in layout.lanes.iter().enumerate() {
        let a = camera.to_screen(canvas, Pos2::new(0.0, top)).y;
        let b = camera.to_screen(canvas, Pos2::new(0.0, bottom)).y;
        let y = (a.max(gutter.top()) + space::SM).min(b - 16.0);
        if b < gutter.top() || a > gutter.bottom() {
            continue;
        }
        if a > gutter.top() {
            painter.line_segment(
                [Pos2::new(gutter.left(), a), Pos2::new(gutter.right(), a)],
                Stroke::new(1.0, color::BORDER),
            );
        }
        painter.text(
            Pos2::new(gutter.left() + space::MD, y),
            Align2::LEFT_TOP,
            i18n.tr(&format!("tech-field-{}", FIELDS[f])),
            sans_semibold(12.5),
            color::TEXT_MUTED,
        );
    }

    minimap(ui, area, view, tree, camera, layout);
    controls(ui, area, i18n, camera, layout);
    picked
}

/// Whether `t` is `focus` or one of the technologies it needs.
fn leads_to(tree: &TechTree, t: usize, focus: usize) -> bool {
    let mut stack = vec![focus];
    let mut seen = BTreeSet::new();
    while let Some(x) = stack.pop() {
        if x == t {
            return true;
        }
        if seen.insert(x) {
            stack.extend(tree.techs[x].requires.iter().map(|&r| r as usize));
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn card(
    ui: &Ui,
    painter: &egui::Painter,
    r: Rect,
    z: f32,
    i18n: &I18n,
    tree: &TechTree,
    view: &ResearchView,
    t: usize,
    response: &egui::Response,
    has_dependents: bool,
    dim: &dyn Fn(Color32, usize) -> Color32,
) -> Option<Option<u16>> {
    let tech = &tree.techs[t];
    let stage = view.stages[t];
    let focused = view.focus == Some(t as u16);
    let can_focus = open(tree, view, t);
    let fill = if response.hovered() {
        color::SURFACE_HOVER
    } else {
        color::SURFACE_RAISED
    };
    let (border, width) = if focused {
        (color::ACCENT, 2.0)
    } else if can_focus {
        (color::ACCENT.gamma_multiply(0.55), 1.0)
    } else if stage >= Stage::Industrial {
        (stage_color(stage).gamma_multiply(0.45), 1.0)
    } else {
        (color::BORDER, 1.0)
    };
    painter.rect(
        r,
        radius::MD,
        dim(fill, t),
        Stroke::new(width, dim(border, t)),
        egui::StrokeKind::Inside,
    );
    // Handles where the curves meet the card.
    for (side, show) in [
        (r.left_center(), !tech.requires.is_empty()),
        (r.right_center(), has_dependents),
    ] {
        if show {
            painter.circle(
                side,
                3.5 * z.max(0.6),
                dim(color::SURFACE, t),
                Stroke::new(1.2, dim(color::BORDER_STRONG, t)),
            );
        }
    }
    let mut picked = None;
    if focused {
        // already the focus
    } else if can_focus && response.clicked() {
        picked = Some(Some(t as u16));
    }
    if response.hovered() && can_focus {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if z < 0.45 {
        // Far away: only the stage, as a coloured strip.
        let strip = Rect::from_min_size(
            r.left_bottom() - Vec2::new(0.0, 5.0),
            Vec2::new(r.width(), 5.0),
        );
        painter.rect_filled(strip, 2.0, dim(stage_color(stage), t));
        return picked;
    }

    let inner = r.shrink(space::SM * z);
    let name = i18n.tr(&format!("tech-{}", tech.key));
    let name_color = if stage == Stage::Unknown && !can_focus {
        color::TEXT_MUTED
    } else {
        color::TEXT
    };
    let mut job = LayoutJob::simple_singleline(name, sans_semibold(13.5 * z), dim(name_color, t));
    job.wrap = TextWrapping {
        max_width: inner.width(),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    painter.galley(
        inner.left_top(),
        ui.fonts_mut(|f| f.layout_job(job)),
        name_color,
    );

    let line = inner.top() + 21.0 * z;
    let (status, status_color) = if focused {
        (i18n.tr("research-focused"), color::ACCENT)
    } else if can_focus && response.hovered() {
        (i18n.tr("research-set-focus"), color::ACCENT_HOVER)
    } else if stage == Stage::Unknown && !can_focus {
        (i18n.tr("research-locked-short"), color::TEXT_FAINT)
    } else {
        (
            i18n.tr(&format!("tech-stage-{}", stage.key())),
            stage_color(stage),
        )
    };
    painter.text(
        Pos2::new(inner.left(), line),
        Align2::LEFT_TOP,
        status,
        sans(12.0 * z),
        dim(status_color, t),
    );
    painter.text(
        Pos2::new(inner.right(), line),
        Align2::RIGHT_TOP,
        tech.demonstrated.to_string(),
        sans(11.0 * z),
        dim(color::TEXT_FAINT, t),
    );
    // The four-step track.
    let track = Rect::from_min_size(
        Pos2::new(inner.left(), inner.bottom() - 6.0 * z),
        Vec2::new(inner.width(), 6.0 * z),
    );
    let gap = 3.0 * z;
    let step_w = (track.width() - 3.0 * gap) / 4.0;
    for (i, s) in Stage::ALL.iter().skip(1).enumerate() {
        let seg = Rect::from_min_size(
            Pos2::new(track.left() + i as f32 * (step_w + gap), track.top()),
            Vec2::new(step_w, track.height()),
        );
        painter.rect_filled(seg, 2.0, dim(color::SURFACE_SUNKEN, t));
        let filled = if stage >= *s {
            1.0
        } else if stage.next() == Some(*s) {
            view.progress[t].clamp(0.0, 1.0)
        } else {
            0.0
        };
        if filled > 0.0 {
            let mut done = seg;
            done.set_width(seg.width() * filled);
            painter.rect_filled(done, 2.0, dim(stage_color(*s), t));
        }
    }
    picked
}

fn minimap(
    ui: &mut Ui,
    canvas: Rect,
    view: &ResearchView,
    tree: &TechTree,
    camera: &mut TreeCamera,
    layout: &TreeLayout,
) {
    let frame = Rect::from_min_size(
        canvas.right_bottom() - MINIMAP - Vec2::splat(space::MD),
        MINIMAP,
    );
    let painter = ui.painter_at(canvas);
    painter.rect(
        frame,
        radius::MD,
        color::SURFACE.gamma_multiply(0.95),
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    let inner = frame.shrink(space::SM);
    let scale = (inner.width() / layout.size.x).min(inner.height() / layout.size.y);
    let origin = inner.center() - layout.size * scale * 0.5;
    let at = |p: Pos2| origin + p.to_vec2() * scale;
    for t in 0..tree.techs.len() {
        let r = Rect::from_min_size(at(layout.pos[t]), NODE * scale);
        let stage = view.stages[t];
        let c = if stage == Stage::Unknown {
            color::BORDER_STRONG
        } else {
            stage_color(stage)
        };
        painter.rect_filled(r, 0.5, c);
    }
    let seen = Rect::from_min_max(
        at(camera.to_tree(canvas, canvas.min)),
        at(camera.to_tree(canvas, canvas.max)),
    )
    .intersect(inner);
    painter.rect_stroke(
        seen,
        2.0,
        Stroke::new(1.5, color::ACCENT),
        egui::StrokeKind::Inside,
    );
    let response = ui.interact(
        frame,
        ui.make_persistent_id("research-minimap"),
        Sense::click_and_drag(),
    );
    if let Some(p) = response.interact_pointer_pos()
        && (response.dragged() || response.clicked())
    {
        let target = ((p - origin) / scale).to_pos2();
        camera.pan = canvas.size() * 0.5 - target.to_vec2() * camera.zoom;
    }
}

fn controls(ui: &mut Ui, canvas: Rect, i18n: &I18n, camera: &mut TreeCamera, layout: &TreeLayout) {
    let h = 28.0;
    let base = canvas.left_bottom() + Vec2::new(space::MD, -space::MD - h);
    let center = canvas.center();
    let buttons = [
        ("+".to_owned(), i18n.tr("research-zoom-in"), 30.0),
        ("−".to_owned(), i18n.tr("research-zoom-out"), 30.0),
        (i18n.tr("research-fit"), i18n.tr("research-fit-hint"), 76.0),
    ];
    let total: f32 = buttons.iter().map(|b| b.2 + 4.0).sum::<f32>() + 48.0;
    ui.painter_at(canvas).rect(
        Rect::from_min_size(base - Vec2::splat(4.0), Vec2::new(total, h + 8.0)),
        radius::MD,
        color::SURFACE,
        Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    let mut x = base.x;
    for (k, (label, hint, w)) in buttons.iter().enumerate() {
        let r = Rect::from_min_size(Pos2::new(x, base.y), Vec2::new(*w, h));
        x += w + 4.0;
        let response = ui
            .put(
                r,
                egui::Button::new(egui::RichText::new(label).font(sans_semibold(13.0)))
                    .fill(color::SURFACE_RAISED)
                    .stroke(Stroke::new(1.0, color::BORDER)),
            )
            .on_hover_text(hint);
        if response.clicked() {
            match k {
                0 => camera.zoom_at(canvas, center, 1.2),
                1 => camera.zoom_at(canvas, center, 1.0 / 1.2),
                _ => {
                    camera.zoom = (canvas.width() / layout.size.x)
                        .min(canvas.height() / layout.size.y)
                        .clamp(ZOOM_MIN, ZOOM_MAX);
                    camera.pan = (canvas.size() - layout.size * camera.zoom) * 0.5;
                }
            }
        }
    }
    ui.painter_at(canvas).text(
        Pos2::new(x + space::XS, base.y + h * 0.5),
        Align2::LEFT_CENTER,
        format!("{:.0} %", camera.zoom * 100.0),
        sans(12.0),
        color::TEXT_FAINT,
    );
}

fn tooltip(
    ui: &mut Ui,
    i18n: &I18n,
    tree: &TechTree,
    cat: &Catalogue,
    view: &ResearchView,
    t: usize,
) {
    let tech = &tree.techs[t];
    ui.set_max_width(320.0);
    ui.label(
        egui::RichText::new(i18n.tr(&format!("tech-{}", tech.key)))
            .font(sans_semibold(14.0))
            .color(color::TEXT),
    );
    ui.label(
        egui::RichText::new(i18n.tr(&format!("tech-{}-desc", tech.key)))
            .font(sans(13.0))
            .color(color::TEXT_MUTED),
    );
    ui.add_space(space::XS);
    ui.label(
        egui::RichText::new(i18n.tr_args(
            "research-cost",
            &[
                ("n", Arg::String((tech.level + 1).to_string())),
                ("cost", Arg::String(tech.cost.to_string())),
            ],
        ))
        .font(sans(12.5)),
    );
    let names = |list: Vec<String>| list.join(", ");
    if !tech.requires.is_empty() {
        let req = names(
            tech.requires
                .iter()
                .map(|&r| i18n.tr(&format!("tech-{}", tree.techs[r as usize].key)))
                .collect(),
        );
        ui.label(
            egui::RichText::new(i18n.tr_args("research-requires", &[("list", Arg::String(req))]))
                .font(sans(12.5)),
        );
    }
    let mut unlocks: Vec<String> = tech
        .methods
        .iter()
        .map(|&(b, m)| {
            let b = &cat.buildings[b as usize];
            format!(
                "{} ({})",
                i18n.tr(&format!("method-{}", b.methods[m as usize].key)),
                i18n.tr(&format!("building-{}", b.key))
            )
        })
        .collect();
    unlocks.extend(
        tech.goods
            .iter()
            .map(|&g| i18n.tr(&format!("good-{}", cat.goods[g as usize].key))),
    );
    if !unlocks.is_empty() {
        ui.label(
            egui::RichText::new(
                i18n.tr_args("research-unlocks", &[("list", Arg::String(names(unlocks)))]),
            )
            .font(sans(12.5)),
        );
    }
    ui.label(
        egui::RichText::new(i18n.tr_args(
            "research-history",
            &[
                ("demonstrated", Arg::String(tech.demonstrated.to_string())),
                (
                    "industrialized",
                    Arg::String(tech.industrialized.to_string()),
                ),
            ],
        ))
        .font(sans(12.5))
        .color(color::TEXT_FAINT),
    );
    let first = view.firsts[t][Stage::Industrial as usize];
    if first > 0 {
        ui.label(
            egui::RichText::new(i18n.tr_args(
                "research-first-industrial",
                &[("year", Arg::String(first.to_string()))],
            ))
            .font(sans(12.5))
            .color(color::TEXT_FAINT),
        );
    }
    if tech.state_buyer {
        ui.label(
            egui::RichText::new(i18n.tr("research-state-buyer"))
                .font(sans(12.5))
                .color(color::TEXT_FAINT),
        );
    }
}
