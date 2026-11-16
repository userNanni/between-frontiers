//! The side panel on the map, under the country figures: the construction queue, what the
//! country researches, and the journal entries the player pinned. Every section folds; the
//! whole panel folds to its title. It reads only the snapshot, which the simulation fills once a
//! month (the journal) or at every publication (research).

use bf_i18n::{Arg, I18n};
use bf_sim::econ::Catalogue;
use bf_sim::research::TechTree;
use bf_ui_kit::theme::{sans, sans_medium, sans_semibold};
use bf_ui_kit::tokens::{color, radius, size, space};
use egui::{Align2, Pos2, Rect, Sense, Ui, Vec2};

use super::journal::{one_line, text, tone_color, value};
use crate::journal::{Journal, Value, WorkLine};
use crate::sim::Snapshot;
use crate::world::World;

pub const WIDTH: f32 = 300.0;
const MARGIN: f32 = 12.0;
const HEADER_H: f32 = 30.0;

/// Which parts are folded; all open at first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fold {
    pub open: bool,
    pub works: bool,
    pub research: bool,
    pub pinned: bool,
}

impl Default for Fold {
    fn default() -> Self {
        Self {
            open: true,
            works: true,
            research: true,
            pinned: true,
        }
    }
}

#[derive(Default)]
pub struct Response {
    pub open_budget: bool,
    pub open_research: bool,
    /// Opens the journal, on this entry when given.
    pub open_journal: Option<Option<String>>,
    pub unpin: Option<String>,
}

pub struct Sources<'a> {
    pub i18n: &'a I18n,
    pub world: Option<&'a World>,
    pub catalogue: &'a Catalogue,
    pub tree: Option<&'a TechTree>,
    pub snap: &'a Snapshot,
    pub pinned: &'a [String],
}

/// Shows the panel between `top` and `bottom`, at the left of `screen`.
pub fn show(
    ui: &mut Ui,
    screen: Rect,
    top: f32,
    bottom: f32,
    src: Sources<'_>,
    fold: &mut Fold,
) -> Response {
    let mut out = Response::default();
    let room = (bottom - top).max(HEADER_H + 8.0);
    let id = ui.make_persistent_id("outliner");
    // The content's height of the last frame sizes the panel; the first frame takes the room.
    let last: f32 = ui.ctx().data(|d| d.get_temp(id)).unwrap_or(room);
    let height = if fold.open {
        (last + space::MD * 2.0).min(room)
    } else {
        HEADER_H + space::SM
    };
    let panel = Rect::from_min_size(
        Pos2::new(screen.left() + MARGIN, top),
        Vec2::new(WIDTH, height),
    );
    bf_ui_kit::paint_overlay(ui, panel);
    ui.interact(panel, id.with("block"), Sense::click_and_drag());
    let inner = panel.shrink2(Vec2::new(space::MD, space::SM));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    let ui = &mut child;
    ui.spacing_mut().item_spacing.y = space::XS;
    let start = ui.cursor().top();

    // Title: folds the panel; the journal opens from its button.
    ui.horizontal(|ui| {
        if fold_header(ui, &src.i18n.tr("outliner-title"), fold.open, true) {
            fold.open = !fold.open;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let r = link(ui, &src.i18n.tr("outliner-journal"));
            if r.clicked() {
                out.open_journal = Some(None);
            }
        });
    });
    if !fold.open {
        return out;
    }
    egui::ScrollArea::vertical()
        .id_salt("outliner-scroll")
        .max_height(room - HEADER_H - space::MD * 2.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.set_max_width(inner.width() - 10.0);
            works(ui, &src, fold, &mut out);
            research(ui, &src, fold, &mut out);
            pinned(ui, &src, fold, &mut out);
        });
    let used = ui.min_rect().bottom() - start + space::XS;
    ui.ctx().data_mut(|d| d.insert_temp(id, used));
    out
}

/// A clickable section title with its fold mark; true when clicked.
fn fold_header(ui: &mut Ui, title: &str, open: bool, big: bool) -> bool {
    let font = if big {
        sans_semibold(size::BODY)
    } else {
        sans_semibold(size::CAPTION)
    };
    let label = if big {
        title.to_owned()
    } else {
        title.to_uppercase()
    };
    let tint = if big { color::TEXT } else { color::TEXT_MUTED };
    let (rect, arrow) = ui.allocate_exact_size(Vec2::new(10.0, 16.0), Sense::click());
    let c = rect.center();
    let tri = if open {
        vec![
            c + Vec2::new(-4.0, -2.0),
            c + Vec2::new(4.0, -2.0),
            c + Vec2::new(0.0, 3.0),
        ]
    } else {
        vec![
            c + Vec2::new(-2.0, -4.0),
            c + Vec2::new(3.0, 0.0),
            c + Vec2::new(-2.0, 4.0),
        ]
    };
    ui.painter()
        .add(egui::Shape::convex_polygon(tri, tint, egui::Stroke::NONE));
    let r = ui.add(
        egui::Label::new(egui::RichText::new(label).font(font).color(tint))
            .sense(Sense::click())
            .selectable(false),
    );
    let arrow = arrow.on_hover_cursor(egui::CursorIcon::PointingHand);
    r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() || arrow.clicked()
}

fn link(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .font(sans_medium(size::SMALL))
                .color(color::ACCENT),
        )
        .sense(Sense::click())
        .selectable(false),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A row that opens something: the painter draws, the whole row takes the click.
fn row(ui: &mut Ui, height: f32, label: &str) -> (Rect, egui::Response) {
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let label = label.to_owned();
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label.clone()));
    if resp.hovered() {
        ui.painter().rect_filled(
            rect.expand2(Vec2::new(4.0, 1.0)),
            radius::SM,
            color::SURFACE_HOVER,
        );
    }
    (rect, resp.on_hover_cursor(egui::CursorIcon::PointingHand))
}

/// Left text, right text and a bar under them.
/// Draws the line; true when the left text had to be cut.
fn paint_line(
    ui: &Ui,
    rect: Rect,
    left: &str,
    right: &str,
    share: Option<f32>,
    fill: egui::Color32,
) -> bool {
    let p = ui.painter();
    let right_w = if right.is_empty() {
        0.0
    } else {
        p.layout_no_wrap(right.to_owned(), sans(size::CAPTION), color::TEXT_MUTED)
            .size()
            .x
            + space::SM
    };
    let (galley, cut) = one_line(
        p,
        left,
        sans(size::SMALL),
        color::TEXT,
        rect.width() - right_w,
    );
    p.galley(rect.min + Vec2::new(0.0, 1.0), galley, color::TEXT);
    p.text(
        Pos2::new(rect.right(), rect.top() + 2.0),
        Align2::RIGHT_TOP,
        right,
        sans(size::CAPTION),
        color::TEXT_MUTED,
    );
    if let Some(s) = share {
        let track = Rect::from_min_size(
            Pos2::new(rect.left(), rect.bottom() - 5.0),
            Vec2::new(rect.width(), 3.0),
        );
        p.rect_filled(track, 1.0, color::SURFACE_SUNKEN);
        let mut done = track;
        done.set_width(track.width() * s.clamp(0.0, 1.0));
        p.rect_filled(done, 1.0, fill);
    }
    cut
}

fn work_text(src: &Sources<'_>, w: &WorkLine) -> (String, String) {
    let i18n = src.i18n;
    let left = text(
        i18n,
        src.world,
        &crate::journal::work_label(src.catalogue, w),
    );
    let right = match w.weeks {
        Some(n) => i18n.tr_args(
            "outliner-weeks",
            &[("weeks", Arg::String(i18n.int(i64::from(n))))],
        ),
        None if w.waiting => i18n.tr("outliner-waiting"),
        None => i18n.tr_args(
            "outliner-weeks-beyond",
            &[(
                "weeks",
                Arg::String(i18n.int(i64::from(crate::journal::HORIZON_WEEKS))),
            )],
        ),
    };
    (left, right)
}

fn works(ui: &mut Ui, src: &Sources<'_>, fold: &mut Fold, out: &mut Response) {
    let i18n = src.i18n;
    bf_ui_kit::gap(ui, space::XS);
    let Some(j) = src.snap.journal.as_deref() else {
        return;
    };
    let w = &j.works;
    ui.horizontal(|ui| {
        if fold_header(ui, &i18n.tr("outliner-works"), fold.works, false) {
            fold.works = !fold.works;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(i18n.tr_args(
                    "outliner-works-count",
                    &[
                        ("public", Arg::String(i18n.int(w.public as i64))),
                        ("private", Arg::String(i18n.int(w.private as i64))),
                    ],
                ))
                .font(sans(size::CAPTION))
                .color(color::TEXT_MUTED),
            );
        });
    });
    if !fold.works {
        return;
    }
    if w.first.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("outliner-works-none"));
        return;
    }
    for line in &w.first {
        let (left, right) = work_text(src, line);
        let (rect, resp) = row(ui, 24.0, &left);
        let fill = if line.public {
            color::ACCENT
        } else {
            color::TEXT_MUTED
        };
        paint_line(ui, rect, &left, &right, Some(line.done), fill);
        let tip = left.clone()
            + "\n"
            + &i18n.tr_args(
                if line.public {
                    "outliner-work-tip-public"
                } else {
                    "outliner-work-tip-private"
                },
                &[(
                    "done",
                    Arg::String(value(i18n, Value::Share(f64::from(line.done)))),
                )],
            );
        if resp.on_hover_text(bf_ui_kit::tooltip_text(&tip)).clicked() {
            out.open_budget = true;
        }
    }
    for b in &w.bottlenecks {
        ui.add(
            egui::Label::new(
                egui::RichText::new(text(i18n, src.world, b))
                    .font(sans(size::CAPTION))
                    .color(color::DANGER),
            )
            .wrap(),
        );
    }
}

fn research(ui: &mut Ui, src: &Sources<'_>, fold: &mut Fold, out: &mut Response) {
    let i18n = src.i18n;
    let Some(tree) = src.tree else {
        return;
    };
    // The focus from the latest snapshot; without one, what the journal found closest.
    let lines: Vec<(u16, bf_sim::research::Stage, f32, bool)> = match &src.snap.research {
        Some(r) if r.focus.is_some() => {
            let t = r.focus.unwrap_or(0);
            let stage = r.stages.get(usize::from(t)).copied().unwrap_or_default();
            let next = bf_sim::research::Stage::ALL
                .iter()
                .skip_while(|s| **s != stage)
                .nth(1)
                .copied();
            next.map(|n| {
                (
                    t,
                    n,
                    r.progress.get(usize::from(t)).copied().unwrap_or(0.0),
                    true,
                )
            })
            .into_iter()
            .collect()
        }
        _ => src
            .snap
            .journal
            .as_deref()
            .map(|j| {
                j.research
                    .iter()
                    .map(|l| (l.tech, l.toward, l.progress, l.chosen))
                    .collect()
            })
            .unwrap_or_default(),
    };
    bf_ui_kit::gap(ui, space::SM);
    ui.horizontal(|ui| {
        if fold_header(ui, &i18n.tr("outliner-research"), fold.research, false) {
            fold.research = !fold.research;
        }
        if let Some((_, _, _, chosen)) = lines.first() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(i18n.tr(if *chosen {
                        "outliner-research-focus"
                    } else {
                        "outliner-research-auto"
                    }))
                    .font(sans(size::CAPTION))
                    .color(color::TEXT_MUTED),
                )
                .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr(if *chosen {
                    "outliner-research-focus-tip"
                } else {
                    "outliner-research-auto-tip"
                })));
            });
        }
    });
    if !fold.research {
        return;
    }
    if lines.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("outliner-research-none"));
    }
    for (t, toward, progress, _) in lines {
        let name = tree
            .techs
            .get(usize::from(t))
            .map(|t| i18n.tr(&format!("tech-{}", t.key)))
            .unwrap_or_default();
        let stage = i18n.tr(&format!("tech-stage-{}", toward.key()));
        let right = format!(
            "{} · {}",
            stage,
            value(i18n, Value::Share(f64::from(progress)))
        );
        let (rect, resp) = row(ui, 24.0, &name);
        let cut = paint_line(ui, rect, &name, &right, Some(progress), color::ACCENT);
        let resp = if cut {
            resp.on_hover_text(bf_ui_kit::tooltip_text(&name))
        } else {
            resp
        };
        if resp.clicked() {
            out.open_research = true;
        }
    }
}

fn pinned(ui: &mut Ui, src: &Sources<'_>, fold: &mut Fold, out: &mut Response) {
    let i18n = src.i18n;
    bf_ui_kit::gap(ui, space::SM);
    let pinned_count = src.pinned.len();
    ui.horizontal(|ui| {
        if fold_header(ui, &i18n.tr("outliner-pinned"), fold.pinned, false) {
            fold.pinned = !fold.pinned;
        }
        if pinned_count > 0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(i18n.int(pinned_count as i64))
                        .font(sans(size::CAPTION))
                        .color(color::TEXT_MUTED),
                );
            });
        }
    });
    if !fold.pinned {
        return;
    }
    if src.pinned.is_empty() {
        bf_ui_kit::small(ui, &i18n.tr("outliner-pinned-none"));
        return;
    }
    let journal: Option<&Journal> = src.snap.journal.as_deref();
    for id in src.pinned {
        match journal.and_then(|j| j.entry(id)) {
            Some(e) => {
                let title = text(i18n, src.world, &e.title);
                let (share, line) = match &e.progress {
                    Some((s, l)) => (Some(*s), text(i18n, src.world, l)),
                    None => (None, String::new()),
                };
                let height = if line.is_empty() { 24.0 } else { 40.0 };
                let (rect, resp) = row(ui, height, &title);
                let p = ui.painter();
                p.rect_filled(
                    Rect::from_min_size(rect.min + Vec2::new(0.0, 3.0), Vec2::new(3.0, 14.0)),
                    1.0,
                    tone_color(e.tone),
                );
                let body = Rect::from_min_max(rect.min + Vec2::new(10.0, 0.0), rect.max);
                let mut cut = paint_line(
                    ui,
                    Rect::from_min_size(body.min, Vec2::new(body.width(), 22.0)),
                    &title,
                    "",
                    None,
                    color::ACCENT,
                );
                if !line.is_empty() {
                    let p = ui.painter();
                    let (g, line_cut) = one_line(
                        p,
                        &line,
                        sans(size::CAPTION),
                        color::TEXT_MUTED,
                        body.width(),
                    );
                    cut |= line_cut;
                    p.galley(body.min + Vec2::new(0.0, 18.0), g, color::TEXT_MUTED);
                    if let Some(s) = share {
                        let lower = Rect::from_min_size(
                            Pos2::new(body.left(), body.bottom() - 5.0),
                            Vec2::new(body.width(), 3.0),
                        );
                        p.rect_filled(lower, 1.0, color::SURFACE_SUNKEN);
                        let mut d = lower;
                        d.set_width(lower.width() * s.clamp(0.0, 1.0));
                        p.rect_filled(d, 1.0, tone_color(e.tone));
                    }
                }
                let resp = if cut {
                    resp.on_hover_text(bf_ui_kit::tooltip_text(&format!("{title}\n{line}")))
                } else {
                    resp
                };
                if resp.clicked() {
                    out.open_journal = Some(Some(id.clone()));
                }
            }
            None => {
                ui.horizontal(|ui| {
                    bf_ui_kit::small(ui, &i18n.tr("outliner-pinned-closed"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if link(ui, &i18n.tr("journal-unpin")).clicked() {
                            out.unpin = Some(id.clone());
                        }
                    });
                });
            }
        }
    }
}
