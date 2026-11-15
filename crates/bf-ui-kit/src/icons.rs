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
}

impl Icon {
    pub fn source(self) -> ImageSource<'static> {
        match self {
            Icon::Coins => include_image!("../../../assets/icons/coins.svg"),
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
    let stroke = if active { color::ACCENT } else { color::BORDER };
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
