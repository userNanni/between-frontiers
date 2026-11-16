//! Interface scale: how many screen pixels a point of the interface takes.
//!
//! The interface is drawn for a reference screen 1080 pixels high and scaled to the real one
//! (the automatic scale); the player can fix the scale instead, or ask for an interface sized
//! for a stream, so that viewers on a small screen can read it. Whatever the choice, the
//! logical screen never goes below [`FLOOR`]: the scale is capped so that it fits. Only the
//! interface scales; the map draws at the screen's own resolution.

use serde::{Deserialize, Serialize};

/// Height of the reference screen the interface is drawn for.
pub const REFERENCE_HEIGHT: f32 = 1080.0;
/// The smallest logical screen the interface supports.
pub const FLOOR: (f32, f32) = (960.0, 540.0);
/// The automatic scale never goes below this.
pub const AUTO_MIN: f32 = 0.75;
/// Scales a fixed choice can take, in percent (without the automatic one).
pub const FIXED: [u32; 8] = [75, 100, 125, 150, 175, 200, 250, 300];

/// A logical screen a stream is made for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamTarget {
    /// 1280 × 720 logical: a full-HD stream, read on a desktop or a tablet.
    Hd,
    /// 960 × 540 logical: viewers on a phone.
    Phone,
}

impl StreamTarget {
    pub fn size(self) -> (f32, f32) {
        match self {
            StreamTarget::Hd => (1280.0, 720.0),
            StreamTarget::Phone => (960.0, 540.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            StreamTarget::Hd => "settings-ui-stream-hd",
            StreamTarget::Phone => "settings-ui-stream-phone",
        }
    }
}

/// What the player chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Auto,
    /// Percent.
    Fixed(u32),
    Stream(StreamTarget),
}

impl Choice {
    /// From the settings: `ui_scale` 0 is automatic; a stream target wins over both.
    pub fn of(ui_scale: u32, stream: Option<StreamTarget>) -> Self {
        match (stream, ui_scale) {
            (Some(t), _) => Choice::Stream(t),
            (None, 0) => Choice::Auto,
            (None, p) => Choice::Fixed(p.clamp(FIXED[0], FIXED[FIXED.len() - 1])),
        }
    }

    /// Back into the settings' two fields.
    pub fn store(self) -> (u32, Option<StreamTarget>) {
        match self {
            Choice::Auto => (0, None),
            Choice::Fixed(p) => (p, None),
            Choice::Stream(t) => (0, Some(t)),
        }
    }
}

/// The highest scale that keeps the logical screen at least [`FLOOR`].
pub fn ceiling(physical: (f32, f32)) -> f32 {
    (physical.0 / FLOOR.0).min(physical.1 / FLOOR.1).max(0.25)
}

/// Pixels per point for a choice on a screen `physical` pixels large, whose system asks for
/// `native` pixels per point.
pub fn pixels_per_point(choice: Choice, physical: (f32, f32), native: f32) -> f32 {
    let wanted = match choice {
        Choice::Auto => {
            // The reference height, a floor, and the system's own factor when it asks more
            // (a high-density laptop, an accessibility setting).
            let by_height = (physical.1 / REFERENCE_HEIGHT).max(AUTO_MIN);
            if native > 1.0 {
                by_height.max(native)
            } else {
                by_height
            }
        }
        Choice::Fixed(p) => p as f32 / 100.0,
        Choice::Stream(t) => {
            let (w, h) = t.size();
            (physical.0 / w).min(physical.1 / h)
        }
    };
    wanted.min(ceiling(physical))
}

/// The scale moved one step up (`+1`) or down (`-1`) from what is applied now; a fixed choice.
pub fn step(current_ppp: f32, dir: i32) -> Choice {
    let now = (current_ppp * 100.0).round() as i32;
    let next = if dir > 0 {
        FIXED.iter().copied().find(|p| *p as i32 > now)
    } else {
        FIXED.iter().rev().copied().find(|p| (*p as i32) < now)
    };
    Choice::Fixed(next.unwrap_or(if dir > 0 {
        FIXED[FIXED.len() - 1]
    } else {
        FIXED[0]
    }))
}

/// A scale change on trial: applied at once, kept only when the player confirms it before the
/// deadline; otherwise the previous choice comes back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trial {
    pub previous: Choice,
    /// Time (egui input time, seconds) the change goes back at.
    pub deadline: f64,
}

/// Seconds a trial waits for the player.
pub const TRIAL_SECONDS: f64 = 10.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_automatic_scale_follows_the_screen_height() {
        let auto = |w, h, n| pixels_per_point(Choice::Auto, (w, h), n);
        assert_eq!(auto(3840.0, 2160.0, 1.0), 2.0);
        assert!((auto(2560.0, 1440.0, 1.0) - 4.0 / 3.0).abs() < 1e-5);
        assert_eq!(auto(1920.0, 1080.0, 1.0), 1.0);
        assert_eq!(auto(1280.0, 720.0, 1.0), 0.75);
        // A system that asks for more is heard.
        assert_eq!(auto(1920.0, 1080.0, 1.25), 1.25);
    }

    #[test]
    fn the_logical_screen_never_goes_below_the_floor() {
        // 300 % on a 1080p screen would leave 640 × 360: capped at 2.0 (960 × 540).
        let ppp = pixels_per_point(Choice::Fixed(300), (1920.0, 1080.0), 1.0);
        assert_eq!(ppp, 2.0);
        for (w, h) in [
            (1280.0, 800.0),
            (1024.0, 768.0),
            (3440.0, 1440.0),
            (3840.0, 2160.0),
        ] {
            for c in FIXED {
                let ppp = pixels_per_point(Choice::Fixed(c), (w, h), 1.0);
                assert!(
                    w / ppp >= FLOOR.0 - 0.5 && h / ppp >= FLOOR.1 - 0.5,
                    "{c}% {w}×{h}"
                );
            }
        }
    }

    #[test]
    fn a_stream_interface_makes_the_logical_screen_the_target() {
        // 4K for a full-HD stream: 3 pixels a point, 1280 × 720 logical.
        let ppp = pixels_per_point(Choice::Stream(StreamTarget::Hd), (3840.0, 2160.0), 1.0);
        assert_eq!(ppp, 3.0);
        let ppp = pixels_per_point(Choice::Stream(StreamTarget::Phone), (3840.0, 2160.0), 1.0);
        assert_eq!(ppp, 4.0);
        // A 16:10 screen keeps the whole target in view.
        let ppp = pixels_per_point(Choice::Stream(StreamTarget::Hd), (1920.0, 1200.0), 1.0);
        assert_eq!(ppp, 1.5);
    }

    #[test]
    fn the_settings_round_trip_and_steps_walk_the_list() {
        for c in [
            Choice::Auto,
            Choice::Fixed(150),
            Choice::Stream(StreamTarget::Phone),
        ] {
            let (s, t) = c.store();
            assert_eq!(Choice::of(s, t), c);
        }
        assert_eq!(step(1.0, 1), Choice::Fixed(125));
        assert_eq!(step(1.33, 1), Choice::Fixed(150));
        assert_eq!(step(1.33, -1), Choice::Fixed(125));
        assert_eq!(step(3.0, 1), Choice::Fixed(300));
        assert_eq!(step(0.75, -1), Choice::Fixed(75));
    }
}
