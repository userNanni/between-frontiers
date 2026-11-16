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

/// `−  value  +`: returns −1, 0 or +1. The value sits in a fixed width so stacked steppers
/// line up.
pub fn stepper(ui: &mut Ui, value: &str, tip: &str, can_down: bool, can_up: bool) -> i32 {
    let mut out = 0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::XS;
        // Inside a right-aligned row the parts are added from the right.
        let mut parts = [-1, 0, 1];
        if ui.layout().prefer_right_to_left() {
            parts.reverse();
        }
        for part in parts {
            match part {
                0 => {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(64.0, ROW_H), Sense::hover());
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
                    if Button::secondary(label)
                        .size(ButtonSize::Small)
                        .enabled(enabled)
                        .show(ui)
                        .clicked()
                    {
                        out = d;
                    }
                }
            }
        }
    });
    out
}

/// A labelled control row: label on the left, the control on the right.
pub fn control_row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .font(sans(size::BODY))
                .color(color::TEXT_MUTED),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add)
            .inner
    })
    .inner
}
