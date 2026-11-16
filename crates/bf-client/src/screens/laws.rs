//! Government tab: laws. Drawn inside the government window (`government.rs`).
//!
//! Three columns: the law groups by category, each with the law in force; the laws of the chosen
//! group with their effects in words and numbers; and, for the chosen law, how each interest
//! group stands on it with the sum open term by term (the group's weight on an effect times the
//! law's distance from neutral). There is no ideology behind a stance: the terms are all of it.
//! A law not in force is compared with the one in force, effect by effect.

use bf_i18n::{Arg, I18n};
use bf_sim::politics::Scope;
use bf_ui_kit::theme::{sans, sans_medium, sans_semibold};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::text::LayoutJob;
use egui::{Align2, Color32, Rect, Sense, Stroke, Ui, Vec2};

use super::government::TabContext;
use crate::laws_view::{Blocked, Catalogue, LawsView, Reading, Stance, reading};
use bf_sim::game::GameCommand;
use bf_sim::polity::Enactor;

/// Width of the group list and of the stances column; the laws take the rest.
const GROUPS_W: f32 = 236.0;
const STANCES_W: f32 = 384.0;
/// Height of a row in the group list.
const ROW_H: f32 = 40.0;

/// What the player picked, kept per country while the game runs.
#[derive(Clone, Copy, Debug, Default)]
struct Picked {
    group: usize,
    law: Option<usize>,
    /// Law proposed and the tick it was proposed on: pending until the simulation moves on.
    sent: Option<(usize, u64)>,
}

pub fn show(ui: &mut Ui, cx: &mut TabContext<'_>) {
    let i18n = cx.i18n;
    let Some(view) = Catalogue::shared().and_then(|c| c.country(cx.tag, cx.snap.laws.as_deref()))
    else {
        bf_ui_kit::muted(ui, &i18n.tr("laws-missing"));
        return;
    };
    let id = ui.make_persistent_id(("laws-tab", cx.tag));
    let mut picked: Picked = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    if picked.group >= view.group_count() {
        picked = Picked::default();
    }
    // A command applies on the next tick; until then the snapshot does not show it.
    picked.sent = picked.sent.filter(|&(_, tick)| tick == cx.snap.tick);
    let view = view.with_sent(picked.sent.map(|(l, _)| l));
    let law = picked
        .law
        .filter(|&l| view.law_group(l) == picked.group)
        .or_else(|| view.in_force(picked.group))
        .unwrap_or_else(|| view.laws_of(picked.group)[0]);

    // The window scrolls as a whole; the columns scroll each on their own inside the height
    // left in it.
    // A little short of the window, so its own scroll bar never shows and steals width.
    let height = (ui.available_height() - space::SM).max(360.0);
    let width = ui.available_width();
    // Four gaps around the two dividers, and the dividers.
    let laws_w = (width - GROUPS_W - STANCES_W - 4.0 * space::LG - 2.0).max(260.0);
    ui.spacing_mut().item_spacing.x = space::LG;
    ui.horizontal_top(|ui| {
        column(ui, "laws-groups", GROUPS_W, height, |ui| {
            groups(ui, i18n, &view, &mut picked);
        });
        divider_v(ui, height);
        column(ui, "laws-laws", laws_w, height, |ui| {
            laws(ui, i18n, &view, picked.group, law, &mut picked);
        });
        divider_v(ui, height);
        column(ui, "laws-stances", STANCES_W, height, |ui| {
            if let Some(command) = stances(ui, i18n, &view, law) {
                cx.commands.push(command);
                picked.sent = Some((law, cx.snap.tick));
            }
        });
    });
    ui.data_mut(|d| d.insert_temp(id, picked));
}

fn column(ui: &mut Ui, id: &str, width: f32, height: f32, add: impl FnOnce(&mut Ui)) {
    let layout = egui::Layout::top_down(egui::Align::Min);
    ui.allocate_ui_with_layout(Vec2::new(width, height), layout, |ui| {
        ui.set_width(width);
        ui.set_height(height);
        egui::ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(width - space::MD);
                ui.spacing_mut().item_spacing = Vec2::new(space::SM, space::SM);
                add(ui);
            });
    });
}

fn divider_v(ui: &mut Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::BORDER);
}

// ---------------------------------------------------------------------------------------------
// Column 1: categories and law groups

fn groups(ui: &mut Ui, i18n: &I18n, view: &LawsView, picked: &mut Picked) {
    if let Some(live) = view.live() {
        pair_line(
            ui,
            &i18n.tr("laws-legitimacy"),
            &format!("{} %", percent(i18n, i32::from(live.legitimacy))),
            color::TEXT,
        );
        if live.movements.is_empty() {
            bf_ui_kit::small(ui, &i18n.tr("laws-movements-none"));
        }
        for (kind, m) in &live.movements {
            let name = i18n.tr(&format!(
                "movement-{}",
                view.movement_key(usize::from(*kind))
            ));
            pair_line(
                ui,
                &name,
                &i18n.tr_args(
                    "laws-movement-support",
                    &[("value", Arg::String(percent(i18n, i32::from(m.support))))],
                ),
                color::ACCENT,
            );
        }
        bf_ui_kit::gap(ui, space::XS);
    }
    for (category, groups) in view.categories() {
        if groups.is_empty() {
            continue;
        }
        bf_ui_kit::gap(ui, space::XS);
        bf_ui_kit::caption(ui, &i18n.tr(&format!("law-category-{category}")));
        for g in groups {
            let name = i18n.tr(&format!("law-group-{}", view.group_key(g)));
            let mut current = match view.in_force(g) {
                Some(l) => i18n.tr(&format!("law-{}", view.law_key(l))),
                None => i18n.tr("laws-not-held"),
            };
            if view.laws_of(g).iter().any(|&l| view.is_pending(l)) {
                current = i18n.tr_args("laws-row-pending", &[("law", Arg::String(current))]);
            }
            let held = view.in_force(g).is_some();
            if group_row(ui, &name, &current, picked.group == g, held).clicked() {
                picked.group = g;
                picked.law = None;
            }
        }
    }
}

fn group_row(
    ui: &mut Ui,
    title: &str,
    subtitle: &str,
    selected: bool,
    held: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_H), Sense::click());
    let label = format!("{title}: {subtitle}");
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Button, true, selected, label.clone())
    });
    let fill = if selected {
        color::ACCENT_SOFT
    } else if response.hovered() {
        color::SURFACE_HOVER
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, radius::MD, fill);
    if selected {
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
            radius::SM,
            color::ACCENT,
        );
    }
    let x = rect.left() + space::MD;
    let w = rect.width() - space::MD * 1.5;
    painter.galley(
        egui::pos2(x, rect.top() + 4.0),
        one_line(ui, title, sans_semibold(size::SMALL), color::TEXT, w),
        color::TEXT,
    );
    let sub = if held {
        color::TEXT_MUTED
    } else {
        color::TEXT_FAINT
    };
    painter.galley(
        egui::pos2(x, rect.top() + 21.0),
        one_line(ui, subtitle, sans(size::CAPTION + 0.5), sub, w),
        sub,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// One line of text cut with an ellipsis at `width`.
fn one_line(
    ui: &Ui,
    text: &str,
    font: egui::FontId,
    color: Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(width);
    ui.painter().layout_job(job)
}

// ---------------------------------------------------------------------------------------------
// Column 2: the laws of the group

fn laws(ui: &mut Ui, i18n: &I18n, view: &LawsView, group: usize, law: usize, picked: &mut Picked) {
    bf_ui_kit::heading(
        ui,
        &i18n.tr(&format!("law-group-{}", view.group_key(group))),
    );
    let current = view.in_force(group);
    if current.is_none() {
        bf_ui_kit::small(ui, &i18n.tr("laws-not-held-hint"));
    }
    for l in view.laws_of(group) {
        if law_card(ui, i18n, view, l, l == law, Some(l) == current).clicked() {
            picked.group = group;
            picked.law = Some(l);
        }
    }
    let changes = view.changes_in(group);
    if !changes.is_empty() {
        bf_ui_kit::gap(ui, space::SM);
        bf_ui_kit::caption(ui, &i18n.tr("laws-history"));
        for (date, from, to, by) in changes {
            let by = match by {
                Enactor::Command => i18n.tr("laws-by-command"),
                Enactor::Groups => i18n.tr("laws-by-groups"),
                Enactor::Movement(m) => i18n.tr_args(
                    "laws-by-movement",
                    &[(
                        "movement",
                        Arg::String(
                            i18n.tr(&format!("movement-{}", view.movement_key(usize::from(m)))),
                        ),
                    )],
                ),
            };
            bf_ui_kit::small(
                ui,
                &i18n.tr_args(
                    "laws-history-line",
                    &[
                        ("date", Arg::String(crate::text::date_long(i18n, date))),
                        (
                            "from",
                            Arg::String(i18n.tr(&format!("law-{}", view.law_key(from)))),
                        ),
                        (
                            "to",
                            Arg::String(i18n.tr(&format!("law-{}", view.law_key(to)))),
                        ),
                        ("by", Arg::String(by)),
                    ],
                ),
            );
        }
    }
    if current.is_some()
        && let Some(source) = &view.source
    {
        bf_ui_kit::gap(ui, space::SM);
        bf_ui_kit::caption(ui, &i18n.tr("laws-source"));
        bf_ui_kit::small(ui, source);
    }
}

fn law_card(
    ui: &mut Ui,
    i18n: &I18n,
    view: &LawsView,
    law: usize,
    selected: bool,
    in_force: bool,
) -> egui::Response {
    let key = view.law_key(law);
    let name = i18n.tr(&format!("law-{key}"));
    let (fill, stroke) = if selected {
        (
            color::ACCENT_SOFT,
            Stroke::new(1.0, color::ACCENT.gamma_multiply(0.7)),
        )
    } else {
        (color::SURFACE_RAISED, Stroke::new(1.0, color::BORDER))
    };
    let frame = bf_ui_kit::card_frame()
        .fill(fill)
        .stroke(stroke)
        .inner_margin(egui::Margin::same(space::MD as i8));
    let inner = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(&name)
                    .font(sans_semibold(size::BODY))
                    .color(color::TEXT),
            );
            if in_force {
                bf_ui_kit::badge(ui, &i18n.tr("laws-in-force"), bf_ui_kit::Tone::Accent);
            } else if view.is_pending(law) {
                bf_ui_kit::badge(ui, &i18n.tr("laws-pending"), bf_ui_kit::Tone::Neutral);
            }
            if view.is_targeted(law) {
                bf_ui_kit::badge(ui, &i18n.tr("laws-targeted"), bf_ui_kit::Tone::Danger);
            }
        });
        bf_ui_kit::small(ui, &i18n.tr(&format!("law-{key}-desc")));
        let effects = view.effects(law);
        if effects.is_empty() {
            bf_ui_kit::small(ui, &i18n.tr("laws-no-effects"));
        }
        for e in effects {
            effect_line(
                ui,
                i18n,
                e.effect,
                &value_text(i18n, e.effect, e.value),
                color::TEXT,
            );
        }
        let scopes: Vec<String> = view
            .scopes(law)
            .into_iter()
            .map(|s| {
                i18n.tr(match s {
                    Scope::Metropole => "laws-scope-metropole",
                    Scope::Targeted => "laws-scope-targeted",
                    Scope::Colonies => "laws-scope-colonies",
                })
            })
            .collect();
        ui.label(
            egui::RichText::new(scopes.join(" · "))
                .font(sans(size::CAPTION))
                .color(color::TEXT_FAINT),
        );
    });
    let response = ui.interact(
        inner.response.rect,
        ui.make_persistent_id(("law-card", law)),
        Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Button, true, selected, name.clone())
    });
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// "Effect name ........ value", the name wrapping and the value pinned right.
fn effect_line(ui: &mut Ui, i18n: &I18n, effect: usize, value: &str, value_color: Color32) {
    let name = i18n.tr(&format!(
        "law-effect-{}",
        bf_sim::politics::effect_kind(effect).key
    ));
    pair_line(ui, &name, value, value_color);
}

fn pair_line(ui: &mut Ui, left: &str, right: &str, right_color: Color32) {
    let width = ui.available_width();
    let right_g =
        ui.painter()
            .layout_no_wrap(right.to_owned(), sans_medium(size::SMALL), right_color);
    let left_w = (width - right_g.size().x - space::MD).max(60.0);
    let left_g = ui.painter().layout(
        left.to_owned(),
        sans(size::SMALL),
        color::TEXT_MUTED,
        left_w,
    );
    let h = left_g.size().y.max(right_g.size().y);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, h), Sense::hover());
    let painter = ui.painter();
    painter.galley(rect.min, left_g, color::TEXT_MUTED);
    painter.galley(
        egui::pos2(rect.right() - right_g.size().x, rect.top()),
        right_g,
        right_color,
    );
}

// ---------------------------------------------------------------------------------------------
// Column 3: the chosen law, its comparison with the law in force, and the stances

/// Returns the command when the player proposes the law.
fn stances(ui: &mut Ui, i18n: &I18n, view: &LawsView, law: usize) -> Option<GameCommand> {
    let mut out = None;
    let in_force = view.in_force(view.law_group(law)) == Some(law);
    bf_ui_kit::heading(ui, &i18n.tr(&format!("law-{}", view.law_key(law))));
    let stances = view.stances(law);
    if !in_force && let Some(b) = LawsView::balance(&stances) {
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "laws-balance",
                &[
                    ("for", Arg::String(percent(i18n, b.for_ as i32))),
                    ("against", Arg::String(percent(i18n, b.against as i32))),
                ],
            ),
        );
        // What the balance means for the law, in words.
        let reading = if b.for_ == 0 {
            "laws-balance-none"
        } else if b.against > b.for_ {
            "laws-balance-minority"
        } else {
            "laws-balance-majority"
        };
        ui.label(
            egui::RichText::new(i18n.tr(reading))
                .font(sans(size::SMALL))
                .color(if b.against > b.for_ {
                    color::DANGER
                } else {
                    color::POSITIVE
                }),
        );
    }
    match view.propose(law) {
        Ok(command) => {
            if Button::primary(&i18n.tr("laws-propose"))
                .size(ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                out = Some(command);
            }
        }
        Err(why) => {
            Button::primary(&i18n.tr("laws-propose"))
                .size(ButtonSize::Small)
                .enabled(false)
                .show(ui)
                .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr(why.key())));
            if why == Blocked::Pending {
                bf_ui_kit::small(ui, &i18n.tr(why.key()));
            }
        }
    }

    if !in_force {
        bf_ui_kit::small(ui, &i18n.tr("laws-effects-inert"));
        if view.group_key(view.law_group(law)) == "internal_movement" {
            bf_ui_kit::small(ui, &i18n.tr("laws-migration-in-budget"));
        }
    }

    let changes = view.changes(law);
    if !in_force && let Some(current) = view.in_force(view.law_group(law)) {
        bf_ui_kit::gap(ui, space::XS);
        bf_ui_kit::caption(ui, &i18n.tr("laws-changes"));
        bf_ui_kit::small(
            ui,
            &i18n.tr_args(
                "laws-changes-from",
                &[(
                    "law",
                    Arg::String(i18n.tr(&format!("law-{}", view.law_key(current)))),
                )],
            ),
        );
        if changes.is_empty() {
            bf_ui_kit::small(ui, &i18n.tr("laws-changes-none"));
        }
        for c in &changes {
            let text = format!(
                "{} → {}",
                value_text(i18n, c.effect, c.from),
                value_text(i18n, c.effect, c.to)
            );
            effect_line(ui, i18n, c.effect, &text, color::TEXT);
        }
    }

    bf_ui_kit::gap(ui, space::XS);
    bf_ui_kit::caption(ui, &i18n.tr("laws-stances"));
    bf_ui_kit::small(
        ui,
        &i18n.tr(if in_force {
            "laws-stances-hint"
        } else {
            "laws-stances-hint-change"
        }),
    );
    for s in &stances {
        stance(ui, i18n, view, s);
    }
    out
}

fn stance(ui: &mut Ui, i18n: &I18n, view: &LawsView, s: &Stance) {
    let key = view.interest_group_key(s.group);
    let name = i18n.tr(&format!("ig-{key}"));
    let total = s.total;
    let (word, tone) = match total.signum() {
        1 => ("laws-stance-for", color::POSITIVE),
        -1 => ("laws-stance-against", color::DANGER),
        _ => ("laws-stance-indifferent", color::TEXT_FAINT),
    };
    let frame = egui::Frame::new()
        .fill(color::SURFACE_SUNKEN)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::MD)
        .inner_margin(egui::Margin::symmetric(space::MD as i8, space::SM as i8));
    let inner = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 2.0;
        // Header: name, then the total and what it means.
        let width = ui.available_width();
        let right = format!("{} {}", points(i18n, total), i18n.tr(word));
        let name = match s.clout {
            Some(c) => i18n.tr_args(
                "laws-group-clout",
                &[
                    ("group", Arg::String(name.clone())),
                    ("clout", Arg::String(percent(i18n, i32::from(c)))),
                ],
            ),
            None => name.clone(),
        };
        let right_g = ui
            .painter()
            .layout_no_wrap(right, sans_semibold(size::SMALL), tone);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 18.0), Sense::hover());
        let name_g = one_line(
            ui,
            &name,
            sans_semibold(size::SMALL),
            color::TEXT,
            width - right_g.size().x - space::MD,
        );
        ui.painter().galley(rect.min, name_g, color::TEXT);
        ui.painter().galley(
            egui::pos2(rect.right() - right_g.size().x, rect.top()),
            right_g,
            tone,
        );
        if s.terms.is_empty() {
            ui.label(
                egui::RichText::new(i18n.tr("laws-terms-none"))
                    .font(sans(size::CAPTION))
                    .color(color::TEXT_FAINT),
            );
        }
        for t in &s.terms {
            let effect = i18n.tr(&format!(
                "law-effect-{}",
                bf_sim::politics::effect_kind(t.effect).key
            ));
            // weight × how far the effect moves = term.
            let sum = format!(
                "{} × {} = {}",
                signed_int(t.weight.into()),
                points(i18n, t.delta.into()),
                points(i18n, t.value)
            );
            term_line(ui, &effect, &sum);
        }
    });
    inner
        .response
        .on_hover_text(bf_ui_kit::tooltip_text(&i18n.tr(&format!("ig-{key}-desc"))));
}

fn term_line(ui: &mut Ui, left: &str, right: &str) {
    let width = ui.available_width();
    let right_g = ui.painter().layout(
        right.to_owned(),
        sans(size::CAPTION + 0.5),
        color::TEXT,
        width * 0.6,
    );
    let left_w = (width - right_g.size().x - space::SM).max(40.0);
    let left_g = ui.painter().layout(
        left.to_owned(),
        sans(size::CAPTION + 0.5),
        color::TEXT_MUTED,
        left_w,
    );
    let h = left_g.size().y.max(right_g.size().y);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, h), Sense::hover());
    let painter = ui.painter();
    painter.galley(rect.min, left_g, color::TEXT_MUTED);
    let rg = right_g.size();
    painter.galley(
        Align2::RIGHT_TOP
            .anchor_size(egui::pos2(rect.right(), rect.top()), rg)
            .min,
        right_g,
        color::TEXT,
    );
}

// ---------------------------------------------------------------------------------------------
// Numbers

/// An effect's value in words: a ceiling, a share or a multiplier.
fn value_text(i18n: &I18n, effect: usize, value: i32) -> String {
    match reading(effect, value) {
        Reading::NoCap => i18n.tr("laws-value-no-cap"),
        Reading::Cap(v) => i18n.tr_args(
            "laws-value-cap",
            &[("value", Arg::String(percent(i18n, v)))],
        ),
        Reading::Share(v) => i18n.tr_args(
            "laws-value-share",
            &[("value", Arg::String(percent(i18n, v)))],
        ),
        Reading::Multiplier(v) => i18n.tr_args(
            "laws-value-multiplier",
            &[(
                "value",
                Arg::String(i18n.decimal(f64::from(v) / 1000.0, if v % 100 == 0 { 1 } else { 2 })),
            )],
        ),
    }
}

/// Per mille as a percentage, with a decimal only when it has one.
fn percent(i18n: &I18n, per_mille: i32) -> String {
    let digits = usize::from(per_mille % 10 != 0);
    let s = i18n.decimal(f64::from(per_mille.abs()) / 10.0, digits);
    if per_mille < 0 { format!("−{s}") } else { s }
}

/// A stance or a term in points: weight × percentage points, signed.
fn points(i18n: &I18n, per_mille: i64) -> String {
    let digits = usize::from(per_mille % 10 != 0);
    let s = i18n.decimal(per_mille.unsigned_abs() as f64 / 10.0, digits);
    match per_mille.signum() {
        1 => format!("+{s}"),
        -1 => format!("−{s}"),
        _ => s,
    }
}

fn signed_int(v: i64) -> String {
    match v.signum() {
        1 => format!("+{v}"),
        -1 => format!("−{}", v.unsigned_abs()),
        _ => "0".into(),
    }
}
