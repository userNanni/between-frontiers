//! Components. Each one paints itself from the tokens; screens compose these and nothing else.

use egui::{
    Align2, Color32, CornerRadius, Frame, Margin, Rect, Response, RichText, Sense, Shadow, Stroke,
    StrokeKind, TextFormat, Ui, Vec2, WidgetText, text::LayoutJob,
};

use crate::theme::{sans, sans_medium, sans_semibold, serif, serif_medium};
use crate::tokens::{color, control, motion, radius, size, space};

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

// ---------------------------------------------------------------------------------------------
// Text

/// Large serif title of a screen, e.g. the game name on the main menu.
pub fn display(ui: &mut Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .font(serif(size::DISPLAY))
            .color(color::TEXT),
    )
}

pub fn title(ui: &mut Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .font(serif(size::TITLE))
            .color(color::TEXT),
    )
}

pub fn heading(ui: &mut Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .font(serif_medium(size::HEADING))
            .color(color::TEXT),
    )
}

pub fn body(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(sans(size::BODY))
                .color(color::TEXT),
        )
        .wrap(),
    )
}

pub fn muted(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(sans(size::BODY))
                .color(color::TEXT_MUTED),
        )
        .wrap(),
    )
}

pub fn small(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(sans(size::SMALL))
                .color(color::TEXT_MUTED),
        )
        .wrap(),
    )
}

pub fn error_text(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(sans(size::SMALL))
                .color(color::DANGER),
        )
        .wrap(),
    )
}

/// Upper-case, letter-spaced label above a group or a field.
pub fn caption(ui: &mut Ui, text: &str) -> Response {
    ui.label(caption_job(text, color::TEXT_FAINT))
}

pub fn caption_job(text: &str, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        &text.to_uppercase(),
        0.0,
        TextFormat {
            font_id: sans_semibold(size::CAPTION),
            color,
            extra_letter_spacing: 1.2,
            ..Default::default()
        },
    );
    job
}

// ---------------------------------------------------------------------------------------------
// Surfaces

/// A panel laid over the map: sidebars, menus, dialogs.
pub fn overlay_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_OVERLAY)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::LG)
        .inner_margin(Margin::same(space::XL as i8))
        .shadow(Shadow {
            offset: [0, 8],
            blur: 28,
            spread: 0,
            color: color::SHADOW,
        })
}

/// A card inside a panel.
pub fn card_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_RAISED)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::MD)
        .inner_margin(Margin::same(space::LG as i8))
}

/// Edge-to-edge bar (top bar, sidebar body) without rounded corners.
pub fn bar_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_OVERLAY)
        .stroke(Stroke::new(1.0, color::BORDER))
        .inner_margin(Margin::symmetric(space::LG as i8, space::SM as i8))
}

pub fn divider(ui: &mut Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::BORDER);
}

/// A colour square, e.g. a country's map colour.
pub fn swatch(ui: &mut Ui, fill: Color32, side: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
    ui.painter().rect(
        rect,
        radius::SM,
        fill,
        Stroke::new(1.0, Color32::from_black_alpha(140)),
        StrokeKind::Inside,
    );
    response
}

/// Small rounded label, e.g. a country status.
pub fn badge(ui: &mut Ui, text: &str, tone: Tone) -> Response {
    let (fg, bg) = match tone {
        Tone::Neutral => (color::TEXT_MUTED, color::SURFACE_HOVER),
        Tone::Accent => (color::ACCENT, color::ACCENT_SOFT),
        Tone::Danger => (color::DANGER, color::DANGER_SOFT),
    };
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), sans_medium(size::SMALL), fg);
    let size = galley.size() + Vec2::new(space::SM * 2.0, space::XS * 2.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, radius::SM, bg);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, fg);
    response
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Accent,
    Danger,
}

// ---------------------------------------------------------------------------------------------
// Buttons

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// The one action the screen is about.
    Primary,
    Secondary,
    /// Quiet action in a list of menu entries.
    Ghost,
    Danger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    Large,
    Medium,
    Small,
}

#[must_use]
pub struct Button<'a> {
    label: &'a str,
    detail: Option<&'a str>,
    variant: Variant,
    size: ButtonSize,
    enabled: bool,
    width: Option<f32>,
    align_left: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str, variant: Variant) -> Self {
        Self {
            label,
            detail: None,
            variant,
            size: ButtonSize::Medium,
            enabled: true,
            width: None,
            align_left: false,
        }
    }

    pub fn primary(label: &'a str) -> Self {
        Self::new(label, Variant::Primary)
    }

    pub fn secondary(label: &'a str) -> Self {
        Self::new(label, Variant::Secondary)
    }

    pub fn ghost(label: &'a str) -> Self {
        Self::new(label, Variant::Ghost)
    }

    pub fn danger(label: &'a str) -> Self {
        Self::new(label, Variant::Danger)
    }

    /// A second, smaller line under the label.
    pub fn detail(mut self, detail: Option<&'a str>) -> Self {
        self.detail = detail;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Fixed width; `f32::INFINITY` fills the available width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn align_left(mut self) -> Self {
        self.align_left = true;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (height, font) = match self.size {
            ButtonSize::Large => (control::HEIGHT_LG, sans_semibold(size::BODY + 1.0)),
            ButtonSize::Medium => (control::HEIGHT_MD, sans_medium(size::BODY)),
            ButtonSize::Small => (control::HEIGHT_SM, sans_medium(size::SMALL)),
        };
        let height = if self.detail.is_some() {
            height + 16.0
        } else {
            height
        };
        let pad = if self.size == ButtonSize::Small {
            space::MD
        } else {
            space::LG + 4.0
        };

        let label = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), font, Color32::PLACEHOLDER);
        let detail = self.detail.map(|d| {
            ui.painter()
                .layout_no_wrap(d.to_owned(), sans(size::SMALL), Color32::PLACEHOLDER)
        });
        let content_w = label
            .size()
            .x
            .max(detail.as_ref().map_or(0.0, |d| d.size().x));
        let width = match self.width {
            Some(w) if w.is_infinite() => ui.available_width(),
            Some(w) => w,
            None => content_w + pad * 2.0,
        };
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);
        let (enabled, label_text) = (self.enabled, self.label.to_owned());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, enabled, label_text.clone())
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let hover = ui.ctx().animate_bool_with_time(
            response.id,
            self.enabled && response.hovered(),
            motion::FAST,
        );
        let pressed = self.enabled && response.is_pointer_button_down_on();

        let (fill, fill_hover, stroke, text, sub) = match self.variant {
            Variant::Primary => (
                color::ACCENT,
                color::ACCENT_HOVER,
                Color32::TRANSPARENT,
                color::ON_ACCENT,
                color::ON_ACCENT.gamma_multiply(0.75),
            ),
            Variant::Secondary => (
                color::SURFACE_RAISED,
                color::SURFACE_HOVER,
                color::BORDER_STRONG,
                color::TEXT,
                color::TEXT_MUTED,
            ),
            Variant::Ghost => (
                Color32::TRANSPARENT,
                color::SURFACE_HOVER,
                Color32::TRANSPARENT,
                color::TEXT,
                color::TEXT_MUTED,
            ),
            Variant::Danger => (
                color::DANGER_SOFT,
                lerp_color(color::DANGER_SOFT, color::DANGER, 0.25),
                color::DANGER,
                color::TEXT,
                color::TEXT_MUTED,
            ),
        };
        let mut fill = lerp_color(fill, fill_hover, hover);
        if pressed {
            fill = match self.variant {
                Variant::Primary => color::ACCENT_PRESSED,
                _ => color::SURFACE_SUNKEN,
            };
        }
        let (mut text, mut sub) = (text, sub);
        if !self.enabled {
            fill = match self.variant {
                Variant::Ghost => Color32::TRANSPARENT,
                _ => color::SURFACE_RAISED.gamma_multiply(0.6),
            };
            text = color::TEXT_FAINT;
            sub = color::TEXT_FAINT;
        }
        let stroke = if self.enabled && response.has_focus() {
            Stroke::new(1.5, color::ACCENT)
        } else {
            Stroke::new(1.0, stroke)
        };
        ui.painter()
            .rect(rect, radius::MD, fill, stroke, StrokeKind::Inside);

        let total_h = label.size().y + detail.as_ref().map_or(0.0, |d| d.size().y + 2.0);
        let top = rect.center().y - total_h / 2.0;
        let x_of = |w: f32| {
            if self.align_left {
                rect.left() + pad
            } else {
                rect.center().x - w / 2.0
            }
        };
        let label_w = label.size().x;
        let label_h = label.size().y;
        ui.painter()
            .galley(egui::pos2(x_of(label_w), top), label, text);
        if let Some(d) = detail {
            let w = d.size().x;
            ui.painter()
                .galley(egui::pos2(x_of(w), top + label_h + 2.0), d, sub);
        }
        if self.enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Inputs

/// Labelled single-line text field.
pub fn text_field(ui: &mut Ui, value: &mut String, width: f32) -> Response {
    let frame = Frame::new()
        .fill(color::SURFACE_SUNKEN)
        .stroke(Stroke::new(1.0, color::BORDER_STRONG))
        .corner_radius(radius::MD)
        .inner_margin(Margin::symmetric(space::MD as i8, (space::SM + 1.0) as i8));
    frame
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(value)
                    .font(sans(size::BODY))
                    .text_color(color::TEXT)
                    .frame(Frame::NONE)
                    .desired_width(width - space::MD * 2.0),
            )
        })
        .inner
}

/// A row of mutually exclusive options. Returns true when the value changed.
pub fn segmented<T: PartialEq + Clone>(
    ui: &mut Ui,
    value: &mut T,
    options: &[(T, String)],
) -> bool {
    let mut changed = false;
    let frame = Frame::new()
        .fill(color::SURFACE_SUNKEN)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::MD)
        .inner_margin(Margin::same(3));
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for (option, label) in options {
                let selected = *value == *option;
                let galley = ui.painter().layout_no_wrap(
                    label.clone(),
                    sans_medium(size::SMALL + 1.0),
                    Color32::PLACEHOLDER,
                );
                let size = Vec2::new(galley.size().x + space::LG * 2.0, control::HEIGHT_SM);
                let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                let hover = response.hovered();
                let (fill, text) = if selected {
                    (color::ACCENT_SOFT, color::ACCENT)
                } else if hover {
                    (color::SURFACE_HOVER, color::TEXT)
                } else {
                    (Color32::TRANSPARENT, color::TEXT_MUTED)
                };
                ui.painter().rect_filled(rect, radius::SM, fill);
                if selected {
                    ui.painter().rect_stroke(
                        rect,
                        radius::SM,
                        Stroke::new(1.0, color::ACCENT.gamma_multiply(0.6)),
                        StrokeKind::Inside,
                    );
                }
                ui.painter()
                    .galley(rect.center() - galley.size() / 2.0, galley, text);
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    && !selected
                {
                    *value = option.clone();
                    changed = true;
                }
            }
        });
    });
    changed
}

/// On/off switch.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    let size = Vec2::new(40.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let t = ui
        .ctx()
        .animate_bool_with_time(response.id, *on, motion::FAST * 1.5);
    let track = lerp_color(color::SURFACE_SUNKEN, color::ACCENT_SOFT, t);
    let border = lerp_color(color::BORDER_STRONG, color::ACCENT, t);
    ui.painter().rect(
        rect,
        CornerRadius::same(11),
        track,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    let x = egui::lerp(rect.left() + 11.0..=rect.right() - 11.0, t);
    let knob = lerp_color(color::TEXT_MUTED, color::ACCENT, t);
    ui.painter()
        .circle_filled(egui::pos2(x, rect.center().y), 7.5, knob);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Caption above a control, with an optional hint below it.
pub fn field<R>(ui: &mut Ui, label: &str, hint: Option<&str>, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS + 2.0;
        caption(ui, label);
        let r = add(ui);
        if let Some(hint) = hint {
            small(ui, hint);
        }
        r
    })
    .inner
}

// ---------------------------------------------------------------------------------------------
// Data display

/// Label on the left, value on the right.
pub fn stat_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(sans(size::BODY))
                .color(color::TEXT_MUTED),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(value)
                    .font(sans_medium(size::BODY))
                    .color(color::TEXT),
            );
        });
    });
}

/// A headline number: caption, value, detail.
pub fn stat_block(ui: &mut Ui, label: &str, value: &str, detail: Option<&str>) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        caption(ui, label);
        ui.label(
            RichText::new(value)
                .font(sans_semibold(size::STAT))
                .color(color::TEXT),
        );
        if let Some(d) = detail {
            ui.label(
                RichText::new(d)
                    .font(sans(size::SMALL))
                    .color(color::TEXT_MUTED),
            );
        }
    });
}

/// Selectable list row with a colour mark, title, subtitle and trailing text.
pub struct ListRow<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub trailing: &'a str,
    pub mark: Option<Color32>,
    /// Image shown instead of the colour mark, e.g. a flag (3:2).
    pub image: Option<egui::TextureId>,
    pub selected: bool,
}

impl ListRow<'_> {
    pub fn show(self, ui: &mut Ui) -> Response {
        let height = 62.0;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
        let title = self.title.to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title.clone()));
        let hover = ui
            .ctx()
            .animate_bool_with_time(response.id, response.hovered(), motion::FAST);
        let fill = if self.selected {
            color::ACCENT_SOFT
        } else {
            lerp_color(color::SURFACE_RAISED, color::SURFACE_HOVER, hover)
        };
        let stroke = if self.selected {
            Stroke::new(1.0, color::ACCENT.gamma_multiply(0.7))
        } else {
            Stroke::new(1.0, color::BORDER)
        };
        let painter = ui.painter();
        painter.rect(rect, radius::MD, fill, stroke, StrokeKind::Inside);
        let mut x = rect.left() + space::LG;
        if let Some(tex) = self.image {
            let r = Rect::from_center_size(
                egui::pos2(x + 15.0, rect.center().y),
                Vec2::new(30.0, 20.0),
            );
            painter.image(
                tex,
                r,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            painter.rect_stroke(
                r,
                radius::SM,
                Stroke::new(1.0, Color32::from_black_alpha(140)),
                StrokeKind::Inside,
            );
            x += 30.0 + space::MD;
        } else if let Some(mark) = self.mark {
            let r = Rect::from_center_size(egui::pos2(x + 9.0, rect.center().y), Vec2::splat(18.0));
            painter.rect(
                r,
                radius::SM,
                mark,
                Stroke::new(1.0, Color32::from_black_alpha(140)),
                StrokeKind::Inside,
            );
            x += 18.0 + space::MD;
        }
        painter.text(
            egui::pos2(x, rect.top() + 12.0),
            Align2::LEFT_TOP,
            self.title,
            sans_semibold(size::BODY),
            color::TEXT,
        );
        painter.text(
            egui::pos2(x, rect.top() + 34.0),
            Align2::LEFT_TOP,
            self.subtitle,
            sans(size::SMALL),
            color::TEXT_MUTED,
        );
        painter.text(
            egui::pos2(rect.right() - space::LG, rect.top() + 14.0),
            Align2::RIGHT_TOP,
            self.trailing,
            sans(size::SMALL),
            color::TEXT_FAINT,
        );
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

/// Tooltip text in the kit's style.
pub fn tooltip_text(text: &str) -> WidgetText {
    RichText::new(text)
        .font(sans(size::SMALL))
        .color(color::TEXT)
        .into()
}

/// Vertical gap from the spacing scale.
pub fn gap(ui: &mut Ui, size: f32) {
    ui.add_space(size);
}

/// Paints an overlay panel's background into a fixed rectangle, for layouts that place the
/// panel's sections themselves (sidebars with a pinned footer).
pub fn paint_overlay(ui: &Ui, rect: Rect) {
    let painter = ui.painter();
    painter.add(
        Shadow {
            offset: [0, 8],
            blur: 28,
            spread: 0,
            color: color::SHADOW,
        }
        .as_shape(rect, radius::LG),
    );
    painter.rect(
        rect,
        radius::LG,
        color::SURFACE_OVERLAY,
        Stroke::new(1.0, color::BORDER),
        StrokeKind::Inside,
    );
}

/// Horizontal rule across a panel, at `y`.
pub fn paint_rule(ui: &Ui, rect: Rect, y: f32) {
    ui.painter()
        .hline(rect.x_range(), y, Stroke::new(1.0, color::BORDER));
}
