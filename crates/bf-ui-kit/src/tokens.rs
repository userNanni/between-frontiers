//! Design tokens: the only place where colours, spacing, radii, type sizes and durations are
//! defined.

use egui::Color32;

/// Colours. Dark ink surfaces with parchment text and a brass accent, so the map stays the most
/// colourful thing on screen.
pub mod color {
    use super::Color32;

    /// Behind everything when no map is drawn.
    pub const BACKDROP: Color32 = Color32::from_rgb(0x0B, 0x0F, 0x14);
    pub const SURFACE: Color32 = Color32::from_rgb(0x15, 0x1B, 0x23);
    /// Panels laid over the map.
    pub const SURFACE_OVERLAY: Color32 = Color32::from_rgba_premultiplied(0x13, 0x18, 0x1F, 0xF2);
    pub const SURFACE_RAISED: Color32 = Color32::from_rgb(0x1D, 0x25, 0x30);
    pub const SURFACE_HOVER: Color32 = Color32::from_rgb(0x24, 0x2D, 0x3A);
    pub const SURFACE_SUNKEN: Color32 = Color32::from_rgb(0x0F, 0x14, 0x1A);
    pub const BORDER: Color32 = Color32::from_rgb(0x2C, 0x35, 0x42);
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x46, 0x51, 0x62);

    pub const TEXT: Color32 = Color32::from_rgb(0xEC, 0xE3, 0xCF);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0xA9, 0xA0, 0x8E);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x72, 0x6C, 0x61);

    pub const ACCENT: Color32 = Color32::from_rgb(0xC9, 0xA4, 0x4C);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xDD, 0xBA, 0x62);
    pub const ACCENT_PRESSED: Color32 = Color32::from_rgb(0xAE, 0x8A, 0x38);
    pub const ON_ACCENT: Color32 = Color32::from_rgb(0x1A, 0x14, 0x08);
    /// Selected rows and active segments.
    pub const ACCENT_SOFT: Color32 = Color32::from_rgba_premultiplied(0x2E, 0x26, 0x12, 0xFF);

    pub const DANGER: Color32 = Color32::from_rgb(0xC8, 0x5A, 0x4C);
    pub const DANGER_SOFT: Color32 = Color32::from_rgb(0x3A, 0x1D, 0x1A);
    pub const POSITIVE: Color32 = Color32::from_rgb(0x7F, 0xB0, 0x72);

    pub const SHADOW: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 0x90);
}

/// Spacing scale in points.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
    pub const HUGE: f32 = 56.0;
}

pub mod radius {
    pub const SM: u8 = 3;
    pub const MD: u8 = 5;
    pub const LG: u8 = 8;
}

/// Type scale in points.
pub mod size {
    pub const DISPLAY: f32 = 64.0;
    pub const TITLE: f32 = 30.0;
    pub const HEADING: f32 = 21.0;
    pub const BODY: f32 = 15.0;
    pub const SMALL: f32 = 13.0;
    pub const CAPTION: f32 = 11.5;
    pub const STAT: f32 = 20.0;
}

pub mod motion {
    /// Hover and press transitions, in seconds.
    pub const FAST: f32 = 0.08;
}

/// Control heights.
pub mod control {
    pub const HEIGHT_LG: f32 = 48.0;
    pub const HEIGHT_MD: f32 = 36.0;
    pub const HEIGHT_SM: f32 = 28.0;
}
