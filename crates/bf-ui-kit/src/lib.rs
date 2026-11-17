//! Design system of Between Frontiers over egui.
//!
//! Screens never call raw egui widgets: they use the components here, which apply the tokens.
//! That keeps egui's global style from leaking from one screen into another.

pub mod components;
pub mod icons;
pub mod theme;
pub mod tokens;

pub use components::*;
pub use icons::{Icon, close_button, icon, icon_button, paint_icon};
pub use theme::install;
