//! The window: windowed, borderless or full screen, and the window's size in windowed mode,
//! applied by `ViewportCommand`s and kept in the settings. Full screen is the borderless window
//! the size of the monitor (eframe offers no exclusive video mode, and Wayland has none): the
//! resolution chosen is the window's, in windowed mode; to draw the interface smaller or larger,
//! the interface scale is the setting (`ui_scale`).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Windowed,
    /// No title bar nor border, maximized to the monitor.
    Borderless,
    /// Full screen on the monitor the window is on.
    Fullscreen,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Windowed, Mode::Borderless, Mode::Fullscreen];

    pub fn key(self) -> &'static str {
        match self {
            Mode::Windowed => "settings-display-windowed",
            Mode::Borderless => "settings-display-borderless",
            Mode::Fullscreen => "settings-display-fullscreen",
        }
    }
}

/// The window as the player set it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Display {
    pub mode: Mode,
    /// The window's inner size in windowed mode, logical points; `None`: the monitor's size
    /// (maximized).
    pub size: Option<[u32; 2]>,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            mode: Mode::Windowed,
            size: Some(DEFAULT_SIZE),
        }
    }
}

/// The window's size before the player chose one.
pub const DEFAULT_SIZE: [u32; 2] = [1600, 900];

/// The smallest window the interface fits (the layout tests' floor).
pub const MIN_SIZE: [f32; 2] = [960.0, 540.0];

/// Common window sizes, smallest first; the list offers those that fit the monitor.
pub const SIZES: [[u32; 2]; 9] = [
    [960, 540],
    [1280, 720],
    [1366, 768],
    [1440, 900],
    [1600, 900],
    [1920, 1080],
    [2560, 1080],
    [2560, 1440],
    [3840, 2160],
];

/// The sizes that fit a monitor of `monitor` points (all of them when it is unknown).
pub fn fitting(monitor: Option<egui::Vec2>) -> Vec<[u32; 2]> {
    SIZES
        .iter()
        .copied()
        .filter(|[w, h]| monitor.is_none_or(|m| *w as f32 <= m.x && *h as f32 <= m.y))
        .collect()
}

/// Whether the window system is Wayland: no exclusive video mode, no window position.
pub fn wayland() -> bool {
    cfg!(target_os = "linux")
        && std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var("WINIT_UNIX_BACKEND").map_or(true, |b| b != "x11")
}

/// Sends the commands that put the window in `d`.
pub fn apply(ctx: &egui::Context, d: Display) {
    use egui::ViewportCommand as C;
    match d.mode {
        Mode::Fullscreen => {
            ctx.send_viewport_cmd(C::Fullscreen(true));
        }
        Mode::Borderless => {
            ctx.send_viewport_cmd(C::Fullscreen(false));
            ctx.send_viewport_cmd(C::Decorations(false));
            ctx.send_viewport_cmd(C::Maximized(true));
        }
        Mode::Windowed => {
            ctx.send_viewport_cmd(C::Fullscreen(false));
            ctx.send_viewport_cmd(C::Decorations(true));
            match d.size {
                None => ctx.send_viewport_cmd(C::Maximized(true)),
                Some([w, h]) => {
                    ctx.send_viewport_cmd(C::Maximized(false));
                    ctx.send_viewport_cmd(C::InnerSize(egui::vec2(w as f32, h as f32)));
                }
            }
        }
    }
}

/// A window change waiting for the player to keep it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trial {
    pub previous: Display,
    /// Time (egui input time, seconds) it goes back at.
    pub deadline: f64,
}

/// Seconds a trial waits for the player.
pub const TRIAL_SECONDS: f64 = 15.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_sizes_that_fit_are_offered() {
        let fit = fitting(Some(egui::vec2(1920.0, 1080.0)));
        assert_eq!(fit.last(), Some(&[1920, 1080]));
        assert!(fit.contains(&[1366, 768]));
        assert!(!fit.contains(&[2560, 1440]));
        assert_eq!(fitting(None).len(), SIZES.len());
    }

    #[test]
    fn the_window_survives_the_settings_file() {
        let d = Display {
            mode: Mode::Borderless,
            size: Some([1366, 768]),
        };
        let text = ron::to_string(&d).unwrap();
        assert_eq!(ron::from_str::<Display>(&text).unwrap(), d);
        // A settings file from before the window was a setting reads the default.
        assert_eq!(ron::from_str::<Display>("()").unwrap(), Display::default());
    }
}
