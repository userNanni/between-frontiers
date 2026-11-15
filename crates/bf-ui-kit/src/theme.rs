//! Fonts and the base egui style derived from the tokens.

use std::sync::Arc;

use egui::{
    Context, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals,
    style::{Selection, WidgetVisuals, Widgets},
};

use crate::tokens::{color, radius, size, space};

/// Inter for interface text, EB Garamond for display and titles. Both under the SIL Open Font
/// License, embedded so every machine renders the same accents.
pub mod family {
    pub const SANS: &str = "inter";
    pub const SANS_MEDIUM: &str = "inter-medium";
    pub const SANS_SEMIBOLD: &str = "inter-semibold";
    pub const SERIF: &str = "garamond";
    pub const SERIF_MEDIUM: &str = "garamond-medium";
}

const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../../assets/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");
const GARAMOND: &[u8] = include_bytes!("../../../assets/fonts/EBGaramond-SemiBold.ttf");
const GARAMOND_MEDIUM: &[u8] = include_bytes!("../../../assets/fonts/EBGaramond-Medium.ttf");

pub fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn sans_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(family::SANS_MEDIUM.into()))
}

pub fn sans_semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(family::SANS_SEMIBOLD.into()))
}

pub fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(family::SERIF.into()))
}

pub fn serif_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(family::SERIF_MEDIUM.into()))
}

fn fonts() -> FontDefinitions {
    let mut defs = FontDefinitions::default();
    let fallbacks: Vec<String> = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (name, bytes) in [
        (family::SANS, INTER),
        (family::SANS_MEDIUM, INTER_MEDIUM),
        (family::SANS_SEMIBOLD, INTER_SEMIBOLD),
        (family::SERIF, GARAMOND),
        (family::SERIF_MEDIUM, GARAMOND_MEDIUM),
    ] {
        defs.font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    let chain = |first: &str| {
        let mut v = vec![first.to_owned()];
        v.extend(fallbacks.iter().cloned());
        v
    };
    defs.families
        .insert(FontFamily::Proportional, chain(family::SANS));
    for name in [
        family::SANS_MEDIUM,
        family::SANS_SEMIBOLD,
        family::SERIF,
        family::SERIF_MEDIUM,
    ] {
        defs.families
            .insert(FontFamily::Name(name.into()), chain(name));
    }
    defs
}

fn widget(fill: egui::Color32, stroke: egui::Color32, text: egui::Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, stroke),
        corner_radius: radius::MD.into(),
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    }
}

/// Installs fonts and the base style. Call once at start-up.
pub fn install(ctx: &Context) {
    ctx.set_fonts(fonts());
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, serif(size::HEADING)),
            (TextStyle::Body, sans(size::BODY)),
            (TextStyle::Button, sans_medium(size::BODY)),
            (TextStyle::Small, sans(size::SMALL)),
            (TextStyle::Monospace, FontId::monospace(size::SMALL)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(space::SM, space::SM);
        style.spacing.button_padding = egui::vec2(space::MD, space::SM);
        style.spacing.interact_size.y = 28.0;
        style.spacing.scroll.bar_width = 6.0;
        style.interaction.selectable_labels = false;

        let mut v = Visuals::dark();
        v.override_text_color = None;
        v.panel_fill = color::SURFACE;
        v.window_fill = color::SURFACE;
        v.window_stroke = Stroke::new(1.0, color::BORDER);
        v.window_corner_radius = radius::LG.into();
        v.extreme_bg_color = color::SURFACE_SUNKEN;
        v.faint_bg_color = color::SURFACE_RAISED;
        v.hyperlink_color = color::ACCENT;
        v.selection = Selection {
            bg_fill: color::ACCENT.gamma_multiply(0.35),
            stroke: Stroke::new(1.0, color::ACCENT),
        };
        v.text_cursor.stroke = Stroke::new(1.5, color::ACCENT);
        v.widgets = Widgets {
            noninteractive: widget(color::SURFACE, color::BORDER, color::TEXT_MUTED),
            inactive: widget(color::SURFACE_RAISED, color::BORDER, color::TEXT),
            hovered: widget(color::SURFACE_HOVER, color::BORDER_STRONG, color::TEXT),
            active: widget(color::SURFACE_HOVER, color::ACCENT, color::TEXT),
            open: widget(color::SURFACE_RAISED, color::BORDER_STRONG, color::TEXT),
        };
        style.visuals = v;
    });
}
