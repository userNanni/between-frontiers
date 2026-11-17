//! Monochrome SVG icons from `assets/icons`, drawn in white and tinted at use.

use egui::{Color32, ImageSource, Response, Ui, Vec2, include_image};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Coins,
    People,
    Heart,
    Book,
    Hourglass,
    City,
    Pause,
    Play,
    Minus,
    Plus,
    Menu,
    /// Three dots: more options.
    More,
    Shield,
    Chart,
    Globe,
    Star,
    Map,
    Grid,
    Density,
    Crown,
    Coin,
    Pie,
    Flask,
}

impl Icon {
    pub fn source(self) -> ImageSource<'static> {
        match self {
            Icon::Coins => include_image!("../../../assets/icons/coins.svg"),
            Icon::Flask => include_image!("../../../assets/icons/flask.svg"),
            Icon::People => include_image!("../../../assets/icons/people.svg"),
            Icon::Heart => include_image!("../../../assets/icons/heart.svg"),
            Icon::Book => include_image!("../../../assets/icons/book.svg"),
            Icon::Hourglass => include_image!("../../../assets/icons/hourglass.svg"),
            Icon::City => include_image!("../../../assets/icons/city.svg"),
            Icon::Pause => include_image!("../../../assets/icons/pause.svg"),
            Icon::Play => include_image!("../../../assets/icons/play.svg"),
            Icon::Minus => include_image!("../../../assets/icons/minus.svg"),
            Icon::Plus => include_image!("../../../assets/icons/plus.svg"),
            Icon::Menu => include_image!("../../../assets/icons/menu.svg"),
            Icon::More => include_image!("../../../assets/icons/more.svg"),
            Icon::Shield => include_image!("../../../assets/icons/shield.svg"),
            Icon::Chart => include_image!("../../../assets/icons/chart.svg"),
            Icon::Globe => include_image!("../../../assets/icons/globe.svg"),
            Icon::Star => include_image!("../../../assets/icons/star.svg"),
            Icon::Map => include_image!("../../../assets/icons/map.svg"),
            Icon::Grid => include_image!("../../../assets/icons/grid.svg"),
            Icon::Density => include_image!("../../../assets/icons/density.svg"),
            Icon::Crown => include_image!("../../../assets/icons/crown.svg"),
            Icon::Coin => include_image!("../../../assets/icons/coin.svg"),
            Icon::Pie => include_image!("../../../assets/icons/pie.svg"),
        }
    }
}

/// An icon of `size` points in `color`.
pub fn icon(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Response {
    ui.add(
        egui::Image::new(icon.source())
            .fit_to_exact_size(Vec2::splat(size))
            .tint(color),
    )
}

/// Paints an icon into a rectangle without allocating layout space.
pub fn paint_icon(ui: &Ui, icon: Icon, rect: egui::Rect, color: Color32) {
    egui::Image::new(icon.source())
        .tint(color)
        .paint_at(ui, rect);
}

/// Square button showing only an icon; `tip` explains it on hover.
pub fn icon_button(ui: &mut Ui, icon: Icon, side: f32, tip: &str, active: bool) -> Response {
    use crate::tokens::{color, radius};
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), egui::Sense::click());
    let tip_text = tip.to_owned();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tip_text.clone()));
    let hovered = response.hovered();
    let fill = if active {
        color::ACCENT
    } else if hovered {
        color::SURFACE_HOVER
    } else {
        color::SURFACE_RAISED
    };
    // Focused from the keyboard: the accent around it.
    let stroke = if active || response.has_focus() {
        color::ACCENT
    } else if hovered {
        color::BORDER_STRONG
    } else {
        color::BORDER
    };
    ui.painter().rect(
        rect,
        radius::MD,
        fill,
        egui::Stroke::new(1.0, stroke),
        egui::StrokeKind::Inside,
    );
    let tint = if active {
        color::ON_ACCENT
    } else {
        color::TEXT
    };
    paint_icon(ui, icon, rect.shrink(side * 0.24), tint);
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(crate::components::tooltip_text(tip))
}

/// The close button of a window's header: an X drawn 16 px wide in a 32 px square, the size of
/// the header's other buttons, with a fill on hover and on focus (the accent around it when
/// focused). `label` ("Close") is what screen readers read; `tip` ("Close (Esc)") shows on hover.
pub fn close_button(ui: &mut Ui, label: &str, tip: &str) -> Response {
    use crate::tokens::{color, radius};
    const SIDE: f32 = 32.0;
    const MARK: f32 = 16.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(SIDE), egui::Sense::click());
    let name = label.to_owned();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name.clone()));
    let hovered = response.hovered();
    let focused = response.has_focus();
    if hovered || focused {
        ui.painter().rect(
            rect,
            radius::MD,
            color::SURFACE_HOVER,
            egui::Stroke::new(
                1.0,
                if focused {
                    color::ACCENT
                } else {
                    color::BORDER
                },
            ),
            egui::StrokeKind::Inside,
        );
    }
    // Two strokes, not a glyph: the same weight at every scale.
    let c = rect.center();
    let h = MARK / 2.0;
    let stroke = egui::Stroke::new(
        2.0,
        if hovered || focused {
            color::TEXT
        } else {
            color::TEXT_MUTED
        },
    );
    ui.painter()
        .line_segment([c + Vec2::new(-h, -h), c + Vec2::new(h, h)], stroke);
    ui.painter()
        .line_segment([c + Vec2::new(h, -h), c + Vec2::new(-h, h)], stroke);
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(crate::components::tooltip_text(tip))
}
