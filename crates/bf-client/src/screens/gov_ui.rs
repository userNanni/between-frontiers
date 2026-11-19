//! Pieces the government tabs share: an account row (label, share bar, value, and a tooltip
//! with the sum behind it) and a stepper. They live here until the screens prove them; then
//! they move to `bf_ui_kit`.

use bf_ui_kit::theme::{sans, sans_medium};
use bf_ui_kit::tokens::{color, radius, size, space};
use bf_ui_kit::{Button, ButtonSize};
use egui::{Align2, Color32, Rect, Sense, Ui, Vec2};

/// Height of an account row.
const ROW_H: f32 = 24.0;

/// One line of an account: the label on the left, the value on the right, an optional bar for
/// its share of the total in between, and the sum behind it on hover.
pub struct Account<'a> {
    pub label: &'a str,
    pub value: &'a str,
    /// 0–1 of the section's total; `None` draws no bar.
    pub share: Option<f32>,
    pub tip: &'a str,
    pub strong: bool,
    /// Colour of the value (alerts); the text colour otherwise.
    pub tone: Option<Color32>,
    /// Colour of the share bar; the accent otherwise.
    pub bar: Option<Color32>,
}

impl<'a> Account<'a> {
    pub fn new(label: &'a str, value: &'a str, tip: &'a str) -> Self {
        Self {
            label,
            value,
            share: None,
            tip,
            strong: false,
            tone: None,
            bar: None,
        }
    }

    pub fn share(mut self, share: f64) -> Self {
        self.share = Some(share.clamp(0.0, 1.0) as f32);
        self
    }

    pub fn strong(mut self) -> Self {
        self.strong = true;
        self
    }

    pub fn tone(mut self, tone: Color32) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn bar(mut self, bar: Color32) -> Self {
        self.bar = Some(bar);
        self
    }

    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ROW_H), Sense::hover());
        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(
                rect.expand2(Vec2::new(4.0, 0.0)),
                radius::SM,
                color::SURFACE_HOVER,
            );
        }
        let font = if self.strong {
            sans_medium(size::BODY)
        } else {
            sans(size::BODY)
        };
        let label_color = if self.strong {
            color::TEXT
        } else {
            color::TEXT_MUTED
        };
        let label = painter.text(
            rect.left_center(),
            Align2::LEFT_CENTER,
            self.label,
            font.clone(),
            label_color,
        );
        let value = painter.text(
            rect.right_center(),
            Align2::RIGHT_CENTER,
            self.value,
            sans_medium(size::BODY),
            self.tone.unwrap_or(color::TEXT),
        );
        if let Some(share) = self.share {
            // The bar sits in a fixed column so the rows line up.
            let left = (rect.left() + width * 0.42).max(label.right() + space::SM);
            let right = (rect.right() - 110.0).min(value.left() - space::SM);
            if right > left + 8.0 {
                let track = Rect::from_min_max(
                    egui::pos2(left, rect.center().y - 3.0),
                    egui::pos2(right, rect.center().y + 3.0),
                );
                painter.rect_filled(track, radius::SM, color::SURFACE_SUNKEN);
                let mut fill = track;
                fill.set_width(track.width() * share);
                painter.rect_filled(fill, radius::SM, self.bar.unwrap_or(color::ACCENT));
            }
        }
        let label = self.label.to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &label));
        response.on_hover_text(bf_ui_kit::tooltip_text(self.tip))
    }
}

/// Key tips of a stepper: the family letter, a stable id, and the row's place in its list (the
/// row's buttons read family + 1, 2 for the first row, 3, 4 for the second…).
pub struct StepTips<'a> {
    pub family: char,
    pub id: &'a str,
    pub row: usize,
}

/// Width of a stepper's value: a number with its unit.
pub const STEP_VALUE_W: f32 = 64.0;

/// `−  value  +`: returns −1, 0 or +1. The value sits in a fixed width so stacked steppers
/// line up.
pub fn stepper(
    ui: &mut Ui,
    value: &str,
    tip: &str,
    can_down: bool,
    can_up: bool,
    tips: StepTips<'_>,
) -> i32 {
    stepper_wide(ui, value, STEP_VALUE_W, tip, can_down, can_up, tips)
}

/// The width a stepper's value needs to hold the longest of `values` with room on each side, at
/// least [`STEP_VALUE_W`]: steppers whose value is a word ("No term") stay clear of − and +.
pub fn step_width(ui: &Ui, values: &[String]) -> f32 {
    let font = sans_medium(size::BODY);
    let widest = values
        .iter()
        .map(|v| {
            ui.painter()
                .layout_no_wrap(v.clone(), font.clone(), color::TEXT)
                .size()
                .x
        })
        .fold(0.0f32, f32::max);
    (widest + 2.0 * space::MD).max(STEP_VALUE_W).ceil()
}

/// [`stepper`] with the value `width` wide.
pub fn stepper_wide(
    ui: &mut Ui,
    value: &str,
    width: f32,
    tip: &str,
    can_down: bool,
    can_up: bool,
    tips: StepTips<'_>,
) -> i32 {
    match stepper_any(ui, value, width, tip, can_down, can_up, tips, None) {
        StepOut::Step(d) => d,
        _ => 0,
    }
}

/// A stepper whose value can also be typed: what the number accepts.
pub struct Typed {
    /// The number as the value shows it, without its unit.
    pub current: f64,
    pub min: f64,
    pub max: f64,
    /// Decimal places kept from what is typed.
    pub decimals: i32,
    /// The limits, said: shown when a typed number falls outside them.
    pub range: String,
    /// How to type, added to the value's tip.
    pub hint: String,
}

/// What a typed stepper did this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepOut {
    None,
    /// − or +: −1 or +1.
    Step(i32),
    /// A number typed and confirmed with Enter, within the limits.
    Set(f64),
}

/// [`stepper_wide`] whose value, clicked, turns into a field: Enter applies the number typed,
/// Escape or a click elsewhere leaves it as it was, a number out of the limits stays in the
/// field, marked, with the limits said.
#[allow(clippy::too_many_arguments)]
pub fn stepper_typed(
    ui: &mut Ui,
    value: &str,
    width: f32,
    tip: &str,
    can_down: bool,
    can_up: bool,
    tips: StepTips<'_>,
    typed: Typed,
) -> StepOut {
    stepper_any(ui, value, width, tip, can_down, can_up, tips, Some(typed))
}

/// A number as typed: a comma or a point for the decimals, a sign, spaces and a percent sign
/// allowed.
pub fn parse_typed(text: &str) -> Option<f64> {
    let t: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '%' && *c != '+')
        .map(|c| {
            if c == ',' {
                '.'
            } else if c == '−' {
                '-'
            } else {
                c
            }
        })
        .collect();
    let v: f64 = t.parse().ok()?;
    v.is_finite().then_some(v)
}

/// The text a field opens with: the number, in the value's own writing.
fn typed_text(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_digit() || matches!(c, ',' | '.' | '-' | '−'))
        .collect()
}

/// A field being typed into: its text and whether the last Enter was refused.
#[derive(Clone, Debug, Default)]
struct Editing {
    text: String,
    refused: bool,
}

/// The value's cell of a typed stepper.
fn typed_cell(
    ui: &mut Ui,
    value: &str,
    width: f32,
    tip: &str,
    id: &str,
    typed: &Typed,
) -> Option<f64> {
    let key = ui.make_persistent_id(("stepper-typed", id));
    let editing: Option<Editing> = ui.data(|d| d.get_temp(key));
    let Some(mut e) = editing else {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ROW_H), Sense::click());
        let hovered = response.hovered();
        if hovered {
            ui.painter().rect_stroke(
                rect.shrink(1.0),
                radius::SM,
                egui::Stroke::new(1.0, color::BORDER_STRONG),
                egui::StrokeKind::Inside,
            );
        }
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            value,
            sans_medium(size::BODY),
            color::TEXT,
        );
        // Read by name as what the stepper sets: the first line of its tip.
        let name = tip.lines().next().unwrap_or_default().to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &name));
        let response = response
            .on_hover_cursor(egui::CursorIcon::Text)
            .on_hover_text(bf_ui_kit::tooltip_text(&format!("{tip}\n{}", typed.hint)));
        if response.clicked() {
            ui.data_mut(|d| {
                d.insert_temp(
                    key,
                    Editing {
                        text: typed_text(value),
                        refused: false,
                    },
                )
            });
            ui.memory_mut(|m| m.request_focus(key.with("field")));
        }
        return None;
    };
    // Escape belongs to the field: it leaves the value as it was, and does not reach Back.
    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        ui.data_mut(|d| d.remove::<Editing>(key));
        ui.memory_mut(|m| m.surrender_focus(key.with("field")));
        return None;
    }
    let response = ui.add_sized(
        [width, ROW_H],
        egui::TextEdit::singleline(&mut e.text)
            .id(key.with("field"))
            .font(sans_medium(size::BODY))
            .text_color(color::TEXT)
            .align(Align2::CENTER_CENTER)
            .char_limit(12)
            // Holds the focus on Escape, so that it reaches the field and not Back.
            .event_filter(egui::EventFilter {
                horizontal_arrows: true,
                escape: true,
                ..Default::default()
            }),
    );
    if e.refused {
        ui.painter().rect_stroke(
            response.rect,
            radius::SM,
            egui::Stroke::new(1.5, color::DANGER),
            egui::StrokeKind::Outside,
        );
        response.show_tooltip_text(typed.range.as_str());
    }
    let mut out = None;
    if response.lost_focus() {
        let (enter, escape) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
            )
        });
        let parsed = parse_typed(&e.text)
            .map(|v| {
                let f = 10f64.powi(typed.decimals);
                (v * f).round() / f
            })
            .filter(|v| (typed.min..=typed.max).contains(v));
        match (enter && !escape, parsed) {
            (true, Some(v)) => {
                out = Some(v);
                ui.data_mut(|d| d.remove::<Editing>(key));
            }
            (true, None) => {
                e.refused = true;
                ui.memory_mut(|m| m.request_focus(key.with("field")));
                ui.data_mut(|d| d.insert_temp(key, e));
            }
            _ => ui.data_mut(|d| d.remove::<Editing>(key)),
        }
    } else {
        if response.changed() {
            e.refused = false;
        }
        ui.data_mut(|d| d.insert_temp(key, e));
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn stepper_any(
    ui: &mut Ui,
    value: &str,
    width: f32,
    tip: &str,
    can_down: bool,
    can_up: bool,
    tips: StepTips<'_>,
    typed: Option<Typed>,
) -> StepOut {
    let mut out = StepOut::None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::XS;
        // Inside a right-aligned row the parts are added from the right.
        let mut parts = [-1, 0, 1];
        if ui.layout().prefer_right_to_left() {
            parts.reverse();
        }
        for part in parts {
            match part {
                0 if typed.is_some() => {
                    if let Some(t) = &typed
                        && let Some(v) = typed_cell(ui, value, width, tip, tips.id, t)
                    {
                        out = StepOut::Set(v);
                    }
                }
                0 => {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(width, ROW_H), Sense::hover());
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        value,
                        sans_medium(size::BODY),
                        color::TEXT,
                    );
                    response.on_hover_text(bf_ui_kit::tooltip_text(tip));
                }
                d => {
                    let (label, enabled) = if d < 0 {
                        ("−", can_down)
                    } else {
                        ("+", can_up)
                    };
                    let response = Button::secondary(label)
                        .size(ButtonSize::Small)
                        .enabled(enabled)
                        .show(ui);
                    let (name, digit) = if d < 0 {
                        ("down", tips.row * 2 + 1)
                    } else {
                        ("up", tips.row * 2 + 2)
                    };
                    let key = if digit < 10 {
                        digit.to_string()
                    } else {
                        name.to_owned()
                    };
                    let id = format!("{}.{name}", tips.id);
                    let key_typed = enabled
                        && crate::keys::item_beside(
                            ui,
                            tips.family,
                            id,
                            &key,
                            response.rect,
                            if d < 0 {
                                crate::keys::Side::Left
                            } else {
                                crate::keys::Side::Right
                            },
                        );
                    if response.clicked() || key_typed {
                        out = StepOut::Step(d);
                    }
                }
            }
        }
    });
    out
}

/// A labelled control row: label on the left, the control on the right.
/// The control is laid out first; a label longer than what is left wraps under itself instead
/// of running under the control.
pub fn control_row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    let (_, out) = egui::containers::Sides::new().shrink_left().wrap().show(
        ui,
        |ui| {
            ui.label(
                egui::RichText::new(label)
                    .font(sans(size::BODY))
                    .color(color::TEXT_MUTED),
            );
        },
        add,
    );
    out
}

/// "Between 0 % and 200 %": the limits of a typed stepper.
pub fn typed_range(i18n: &bf_i18n::I18n, min: &str, max: &str) -> String {
    i18n.tr_args(
        "step-typed-range",
        &[
            ("min", bf_i18n::Arg::String(min.to_owned())),
            ("max", bf_i18n::Arg::String(max.to_owned())),
        ],
    )
}

/// A whole number typed in a stepper, from `min` to `max`.
pub fn typed_int(i18n: &bf_i18n::I18n, current: i64, min: i64, max: i64) -> Typed {
    Typed {
        current: current as f64,
        min: min as f64,
        max: max as f64,
        decimals: 0,
        range: typed_range(i18n, &i18n.int(min), &i18n.int(max)),
        hint: i18n.tr("step-typed-hint"),
    }
}

/// A percentage typed in a stepper, from `min` to `max` %, kept to `decimals` places.
pub fn typed_pct(i18n: &bf_i18n::I18n, current: f64, min: f64, max: f64, decimals: usize) -> Typed {
    let p = |x: f64| super::budget::pct(i18n, x, decimals);
    Typed {
        current,
        min,
        max,
        decimals: decimals as i32,
        range: typed_range(i18n, &p(min), &p(max)),
        hint: i18n.tr("step-typed-hint"),
    }
}

/// The new value of a stepper whose − and + move by `step`, or that was typed (`scale` turns
/// the number typed into the stepper's units), clamped to `min..=max`.
pub fn step_to(out: StepOut, now: i32, step: i32, scale: f64, min: i32, max: i32) -> Option<i32> {
    match out {
        StepOut::None => None,
        StepOut::Step(d) => Some((now + d * step).clamp(min, max)),
        StepOut::Set(v) => Some(((v * scale).round() as i32).clamp(min, max)),
    }
}
