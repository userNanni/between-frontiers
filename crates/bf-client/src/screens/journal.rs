//! The journal window (J): every entry of the player's country on the left, the one picked on
//! the right with where it comes from, the conditions that open and close it measured against
//! their limits, its milestones, the forces that push it, and a pin for the side panel.

use crate::keys::{self, id};
use bf_i18n::{Arg as I18nArg, I18n};
use bf_ui_kit::theme::{sans, sans_medium};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align2, Color32, Rect, Sense, Stroke, Ui, Vec2};

use super::region;
use crate::journal::{Arg, Cmp, Condition, Entry, Journal, Kind, Label, Tone, Value};
use crate::text::{date_long, money};
use crate::world::World;

/// Window size: two columns, short enough for 1280 × 720 under the HUD.
const WINDOW: Vec2 = Vec2::new(1000.0, 680.0);
const LIST_W: f32 = 330.0;
/// The smallest the player may make the window.
const MIN_WINDOW: Vec2 = Vec2::new(720.0, 420.0);
const SCROLL_GUTTER: f32 = 16.0;

/// The open window: the entry picked.
#[derive(Clone, Debug, Default)]
pub struct Window {
    pub picked: Option<String>,
}

#[derive(Default)]
pub struct Response {
    pub close: bool,
    /// The player finished moving or resizing the window.
    pub place_done: bool,
    /// Entry ids whose pin was toggled.
    pub toggle_pin: Vec<String>,
    pub open: Option<crate::journal::Screen>,
}

/// Formats a value.
pub fn value(i18n: &I18n, v: Value) -> String {
    let pct = |p: f64| {
        let digits = if p.abs() < 10.0 { 1 } else { 0 };
        i18n.tr_args(
            "gov-pct",
            &[("value", I18nArg::String(i18n.decimal(p, digits)))],
        )
    };
    // A value never breaks across lines.
    let out = match v {
        Value::Share(s) => pct(s * 100.0),
        Value::Percent(p) => pct(p),
        Value::Money(m) if m < 0.0 => format!("−{}", money(i18n, -m)),
        Value::Money(m) => money(i18n, m),
        Value::Weeks(w) => i18n.tr_args(
            "journal-weeks",
            &[("weeks", I18nArg::String(i18n.int(w.round() as i64)))],
        ),
        Value::Count(n) => i18n.int(n),
        Value::Year(y) => y.to_string(),
        Value::Number(x) => i18n.decimal(x, 2),
    };
    out.replace(' ', "\u{a0}")
}

/// Formats a label.
pub fn text(i18n: &I18n, world: Option<&World>, l: &Label) -> String {
    let args: Vec<(&str, I18nArg<'_>)> = l
        .args
        .iter()
        .map(|(name, a)| {
            let s = match a {
                Arg::Value(v) => value(i18n, *v),
                Arg::Key(id) => i18n.tr(id),
                Arg::State(id) => world
                    .and_then(|w| w.states.get(usize::from(*id).checked_sub(1)?))
                    .map(|s| s.def.name.clone())
                    .unwrap_or_else(|| "—".into()),
                Arg::Date(d) => date_long(i18n, *d),
            };
            (*name, I18nArg::String(s))
        })
        .collect();
    i18n.tr_args(&l.id, &args)
}

pub fn tone_color(tone: Tone) -> Color32 {
    match tone {
        Tone::Neutral => color::ACCENT,
        Tone::Warning => color::ACCENT_HOVER,
        Tone::Danger => color::DANGER,
    }
}

/// The group an entry is listed under.
fn group_key(kind: &Kind) -> &'static str {
    match kind {
        Kind::Works | Kind::LargeWork => "journal-group-works",
        Kind::Research => "journal-group-research",
        Kind::Fiscal(_) => "journal-group-fiscal",
        Kind::Movement(_) | Kind::PendingLaw(_) | Kind::LawChanges => "journal-group-politics",
    }
}

/// Agendas first: entries with a condition and a consequence. Then what is only followed.
const AGENDAS: [&str; 2] = ["journal-group-politics", "journal-group-fiscal"];
const FOLLOWED: [&str; 2] = ["journal-group-works", "journal-group-research"];

fn is_agenda(kind: &Kind) -> bool {
    AGENDAS.contains(&group_key(kind))
}

/// One line of text no wider than `width`, ended with "…" when it does not fit; the flag
/// says whether it was cut.
pub fn one_line(
    p: &egui::Painter,
    text: &str,
    font: egui::FontId,
    tint: Color32,
    width: f32,
) -> (std::sync::Arc<egui::Galley>, bool) {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, tint);
    job.wrap = egui::text::TextWrapping {
        max_width: width.max(10.0),
        max_rows: 1,
        break_anywhere: false,
        overflow_character: Some('…'),
    };
    let galley = p.layout_job(job);
    let cut = galley.text() != text;
    (galley, cut)
}

/// The mark at the head of a line: drawn, since the fonts lack the geometric glyphs.
#[derive(Clone, Copy)]
pub enum Mark {
    /// A filled dot: the condition holds.
    Holds,
    /// A ring: it does not.
    Fails,
    /// A small dot: the simulation does not tell.
    Unknown,
    /// A filled diamond: a milestone reached.
    Reached,
    /// An empty diamond: one still ahead.
    Ahead,
    Plus,
    Minus,
}

pub fn mark(ui: &mut Ui, m: Mark, tint: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 16.0), Sense::hover());
    let c = rect.center();
    let p = ui.painter();
    let diamond = |r: f32| {
        vec![
            c + Vec2::new(0.0, -r),
            c + Vec2::new(r, 0.0),
            c + Vec2::new(0.0, r),
            c + Vec2::new(-r, 0.0),
        ]
    };
    match m {
        Mark::Holds => {
            p.circle_filled(c, 4.0, tint);
        }
        Mark::Fails => {
            p.circle_stroke(c, 3.5, Stroke::new(1.3, tint));
        }
        Mark::Unknown => {
            p.circle_filled(c, 1.8, tint);
        }
        Mark::Reached => {
            p.add(egui::Shape::convex_polygon(
                diamond(4.5),
                tint,
                Stroke::NONE,
            ));
        }
        Mark::Ahead => {
            p.add(egui::Shape::closed_line(
                diamond(4.0),
                Stroke::new(1.3, tint),
            ));
        }
        Mark::Plus => {
            p.hline(c.x - 4.0..=c.x + 4.0, c.y, Stroke::new(1.6, tint));
            p.vline(c.x, c.y - 4.0..=c.y + 4.0, Stroke::new(1.6, tint));
        }
        Mark::Minus => {
            p.hline(c.x - 4.0..=c.x + 4.0, c.y, Stroke::new(1.6, tint));
        }
    }
}

/// A thin progress bar across `width`.
pub fn bar(ui: &mut Ui, share: f32, fill: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 5.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, radius::SM, color::SURFACE_SUNKEN);
    let mut done = rect;
    done.set_width(rect.width() * share.clamp(0.0, 1.0));
    if done.width() > 0.5 {
        p.rect_filled(done, radius::SM, fill);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    world: Option<&World>,
    journal: Option<&Journal>,
    pinned: &[String],
    window: &mut Window,
    place: &mut crate::hud::WindowPlace,
) -> Response {
    let mut out = Response::default();
    let top = screen.top() + super::top_bar::LEFT_BOTTOM;
    let size = WINDOW.min(Vec2::new(
        screen.width() - 32.0,
        screen.bottom() - top - space::LG,
    ));
    let default = Rect::from_min_size(egui::pos2(screen.center().x - size.x / 2.0, top), size);
    let (rect, moved) = super::window_frame(ui, "journal", default, screen, place, MIN_WINDOW);
    out.place_done |= moved;
    region(ui, rect.shrink(space::XL), |ui| {
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("journal-title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close = Button::ghost(&i18n.tr("gov-close"))
                    .size(ButtonSize::Small)
                    .show(ui);
                if close.clicked() || keys::single(ui, "journal.close", 'X', close.rect) {
                    out.close = true;
                }
            });
        });
        bf_ui_kit::divider(ui);
        let Some(journal) = journal else {
            bf_ui_kit::muted(ui, &i18n.tr("journal-not-ready"));
            return;
        };
        if journal.entries.is_empty() {
            bf_ui_kit::muted(ui, &i18n.tr("journal-empty"));
            return;
        }
        let picked = window
            .picked
            .as_deref()
            .and_then(|id| journal.entry(id))
            .or_else(|| journal.entries.iter().find(|e| is_agenda(&e.kind)))
            .unwrap_or(&journal.entries[0]);
        let picked_id = picked.id.clone();
        let body = ui.available_rect_before_wrap();
        let list = Rect::from_min_size(body.min, Vec2::new(LIST_W, body.height()));
        let detail = Rect::from_min_max(egui::pos2(list.right() + space::XL, body.top()), body.max);
        ui.painter().vline(
            list.right() + space::XL / 2.0,
            body.y_range(),
            Stroke::new(1.0, color::BORDER),
        );
        region(ui, list, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("journal-list")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(LIST_W - SCROLL_GUTTER);
                    bf_ui_kit::heading(ui, &i18n.tr("journal-agendas"));
                    if !journal.entries.iter().any(|e| is_agenda(&e.kind)) {
                        bf_ui_kit::small(ui, &i18n.tr("journal-agendas-none"));
                        bf_ui_kit::gap(ui, space::SM);
                    }
                    for (i, g) in AGENDAS.iter().chain(&FOLLOWED).enumerate() {
                        if i == AGENDAS.len() {
                            bf_ui_kit::gap(ui, space::SM);
                            bf_ui_kit::heading(ui, &i18n.tr("journal-followed"));
                            bf_ui_kit::small(ui, &i18n.tr("journal-followed-hint"));
                            bf_ui_kit::gap(ui, space::XS);
                        }
                        let g = *g;
                        let entries: Vec<&Entry> = journal
                            .entries
                            .iter()
                            .filter(|e| group_key(&e.kind) == g)
                            .collect();
                        if entries.is_empty() {
                            continue;
                        }
                        bf_ui_kit::caption(ui, &i18n.tr(g));
                        for e in entries {
                            let pin = pinned.contains(&e.id);
                            let row = list_row(ui, i18n, world, e, e.id == picked_id, pin);
                            let id = format!("journal.entry.{}", e.id);
                            if row.clicked() || keys::item(ui, 'H', id, &e.id, row.rect) {
                                window.picked = Some(e.id.clone());
                            }
                        }
                        bf_ui_kit::gap(ui, space::SM);
                    }
                });
        });
        region(ui, detail, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("journal-detail")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(detail.width() - SCROLL_GUTTER);
                    entry_detail(ui, i18n, world, picked, pinned, &mut out);
                });
        });
    });
    out.place_done |= super::window_grip(ui, "journal", rect, place, MIN_WINDOW);
    out
}

fn list_row(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    e: &Entry,
    selected: bool,
    pinned: bool,
) -> egui::Response {
    let title = text(i18n, world, &e.title);
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 46.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title.clone()));
    let p = ui.painter();
    let fill = if selected {
        color::ACCENT_SOFT
    } else if response.hovered() {
        color::SURFACE_HOVER
    } else {
        color::SURFACE_RAISED
    };
    p.rect(
        rect,
        radius::MD,
        fill,
        Stroke::new(
            1.0,
            if selected {
                color::ACCENT.gamma_multiply(0.7)
            } else {
                color::BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );
    let mark = Rect::from_min_size(rect.min + Vec2::new(8.0, 10.0), Vec2::new(3.0, 26.0));
    p.rect_filled(mark, 1.0, tone_color(e.tone));
    let x = rect.left() + 20.0;
    let title_w = width - 20.0 - if pinned { 54.0 } else { 10.0 };
    let (galley, mut cut) = one_line(p, &title, sans_medium(size::SMALL), color::TEXT, title_w);
    p.galley(egui::pos2(x, rect.top() + 6.0), galley, color::TEXT);
    let mut full = title.clone();
    if pinned {
        p.text(
            egui::pos2(rect.right() - 10.0, rect.top() + 8.0),
            Align2::RIGHT_TOP,
            i18n.tr("journal-pinned"),
            sans(size::CAPTION),
            color::ACCENT,
        );
    }
    if let Some((share, line)) = &e.progress {
        let line = text(i18n, world, line);
        let (g, line_cut) = one_line(
            p,
            &line,
            sans(size::CAPTION),
            color::TEXT_MUTED,
            width - 30.0,
        );
        p.galley(egui::pos2(x, rect.top() + 25.0), g, color::TEXT_MUTED);
        cut |= line_cut;
        full = format!("{full}\n{line}");
        let track = Rect::from_min_size(
            egui::pos2(x, rect.bottom() - 6.0),
            Vec2::new(width - 30.0, 3.0),
        );
        p.rect_filled(track, 1.0, color::SURFACE_SUNKEN);
        let mut done = track;
        done.set_width(track.width() * share.clamp(0.0, 1.0));
        p.rect_filled(done, 1.0, tone_color(e.tone));
    }
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if cut {
        response.on_hover_text(bf_ui_kit::tooltip_text(&full))
    } else {
        response
    }
}

fn condition_line(ui: &mut Ui, i18n: &I18n, world: Option<&World>, c: &Condition) {
    let (glyph, tint) = match c.holds {
        Some(true) => (Mark::Holds, color::POSITIVE),
        Some(false) => (Mark::Fails, color::TEXT_MUTED),
        None => (Mark::Unknown, color::TEXT_FAINT),
    };
    let mut parts = vec![text(i18n, world, &c.label)];
    if let Some(m) = c.measured {
        parts.push(i18n.tr_args(
            "journal-measured",
            &[("value", I18nArg::String(value(i18n, m)))],
        ));
    }
    if let Some((cmp, limit)) = c.limit {
        parts.push(i18n.tr_args(
            match cmp {
                Cmp::AtLeast => "journal-limit-at-least",
                Cmp::AtMost => "journal-limit-at-most",
            },
            &[("value", I18nArg::String(value(i18n, limit)))],
        ));
        // Both sides round alike: say how far apart they are.
        if let Some(m) = c.measured
            && value(i18n, m) == value(i18n, limit)
            && let Some(diff) = difference(m, limit)
        {
            parts.push(i18n.tr_args(
                "journal-difference",
                &[("value", I18nArg::String(value(i18n, diff)))],
            ));
        }
    }
    if c.holds.is_none() {
        parts.push(i18n.tr("journal-unknown"));
    }
    mark_line(ui, glyph, tint, parts.join(" · "), None);
}

/// The measured value minus the limit, in the same unit; `None` when they are equal.
fn difference(m: Value, limit: Value) -> Option<Value> {
    let d = match (m, limit) {
        (Value::Money(a), Value::Money(b)) => Value::Money(a - b),
        (Value::Share(a), Value::Share(b)) => Value::Share(a - b),
        (Value::Percent(a), Value::Percent(b)) => Value::Percent(a - b),
        (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
        _ => return None,
    };
    let zero = match d {
        Value::Money(x) | Value::Share(x) | Value::Percent(x) | Value::Number(x) => x == 0.0,
        _ => true,
    };
    (!zero).then_some(d)
}

/// A line with its mark on the left, wrapped text, and a value on the right.
fn mark_line(ui: &mut Ui, glyph: Mark, tint: Color32, line: String, right: Option<String>) {
    let width = ui.available_width();
    let p = ui.painter().clone();
    let right = right.map(|r| p.layout_no_wrap(r, sans(size::SMALL), color::TEXT_MUTED));
    let right_w = right.as_ref().map_or(0.0, |g| g.size().x + space::MD);
    let text_w = (width - 12.0 - space::SM - right_w).max(40.0);
    let galley = p.layout(line, sans(size::SMALL), color::TEXT, text_w);
    let height = galley.size().y.max(16.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let mark_rect = Rect::from_min_size(rect.min, Vec2::new(12.0, 16.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(mark_rect));
    mark(&mut child, glyph, tint);
    p.galley(
        rect.min + Vec2::new(12.0 + space::SM, 0.0),
        galley,
        color::TEXT,
    );
    if let Some(g) = right {
        p.galley(
            egui::pos2(rect.right() - g.size().x, rect.top()),
            g,
            color::TEXT_MUTED,
        );
    }
}

fn section(ui: &mut Ui, i18n: &I18n, key: &str) {
    bf_ui_kit::gap(ui, space::SM);
    bf_ui_kit::caption(ui, &i18n.tr(key));
}

fn entry_detail(
    ui: &mut Ui,
    i18n: &I18n,
    world: Option<&World>,
    e: &Entry,
    pinned: &[String],
    out: &mut Response,
) {
    ui.spacing_mut().item_spacing.y = space::XS;
    let is_pinned = pinned.contains(&e.id);
    ui.horizontal(|ui| {
        bf_ui_kit::heading(ui, &text(i18n, world, &e.title));
    });
    ui.horizontal(|ui| {
        let unpin = i18n.tr("journal-unpin");
        let pin_label = i18n.tr("journal-pin");
        let pin = if is_pinned {
            Button::secondary(&unpin)
        } else {
            Button::primary(&pin_label)
        };
        let pin = pin.size(ButtonSize::Small).show(ui);
        let label = if is_pinned { &unpin } else { &pin_label };
        let pin = pin.on_hover_text(keys::tip(ui.ctx(), label, id::JOURNAL_PIN));
        if keys::pressed(ui, id::JOURNAL_PIN, &pin) || keys::fired(ui.ctx(), id::JOURNAL_PIN) {
            out.toggle_pin.push(e.id.clone());
        }
        let go = match e.screen {
            crate::journal::Screen::Budget => "journal-open-budget",
            crate::journal::Screen::Money => "journal-open-money",
            crate::journal::Screen::Laws => "journal-open-laws",
            crate::journal::Screen::Research => "journal-open-research",
        };
        let open = Button::ghost(&i18n.tr(go)).size(ButtonSize::Small).show(ui);
        let open = open.on_hover_text(keys::tip(ui.ctx(), &i18n.tr(go), id::JOURNAL_GO));
        if keys::pressed(ui, id::JOURNAL_GO, &open) || keys::fired(ui.ctx(), id::JOURNAL_GO) {
            out.open = Some(e.screen);
        }
    });
    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::small(ui, &text(i18n, world, &e.cause));
    if let Some((share, line)) = &e.progress {
        bf_ui_kit::gap(ui, space::SM);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(text(i18n, world, line))
                    .font(sans_medium(size::SMALL))
                    .color(color::TEXT),
            );
        });
        let w = ui.available_width();
        bar(ui, *share, tone_color(e.tone), w);
    }
    if !e.enter.is_empty() {
        section(ui, i18n, "journal-enter");
        for c in &e.enter {
            condition_line(ui, i18n, world, c);
        }
    }
    if !e.milestones.is_empty() {
        section(ui, i18n, "journal-milestones");
        for m in &e.milestones {
            let mut line = text(i18n, world, &m.label);
            if let Some(d) = m.date {
                line = format!("{} · {line}", date_long(i18n, d));
            }
            let (glyph, tint) = if m.reached {
                (Mark::Reached, color::POSITIVE)
            } else {
                (Mark::Ahead, color::TEXT_MUTED)
            };
            let right = m.progress.map(|p| value(i18n, Value::Share(f64::from(p))));
            mark_line(ui, glyph, tint, line, right);
        }
    }
    if !e.forces.is_empty() {
        section(ui, i18n, "journal-forces");
        for f in &e.forces {
            let (glyph, tint) = if f.helps {
                (Mark::Plus, color::POSITIVE)
            } else {
                (Mark::Minus, color::DANGER)
            };
            let right = f.value.map(|v| value(i18n, v));
            mark_line(ui, glyph, tint, text(i18n, world, &f.label), right);
        }
    }
    if !e.exit.is_empty() {
        section(ui, i18n, "journal-exit");
        for c in &e.exit {
            condition_line(ui, i18n, world, c);
        }
    }
    if let Some(c) = &e.completes {
        section(ui, i18n, "journal-completes");
        bf_ui_kit::small(ui, &text(i18n, world, c));
    }
}
