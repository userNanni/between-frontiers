//! The leader key: tapped alone, it turns key tips on and off.
//!
//! By default it is the left Alt only. AltGr (the right Alt) types characters on ABNT2 and
//! other ISO layouts, so it never counts as Alt: not as the leader unless the player picks it,
//! and not as the Alt of a shortcut.

use super::commands::Mods;
use super::keymap::Leader;
use super::physical::PhysKey;

/// The physical key a leader setting stands for.
pub fn leader_key(leader: Leader) -> PhysKey {
    match leader {
        Leader::Alt => PhysKey::AltLeft,
        Leader::Key(k) => k,
    }
}

/// Tells a tap of the leader from its use as a modifier, across frames.
#[derive(Clone, Debug, Default)]
pub struct LeaderTap {
    /// The leader is down and nothing else was pressed since.
    armed: bool,
    /// The platform sends Alt as a key; the modifier-only fallback is off from then on.
    alt_keys: bool,
    alt_was: bool,
}

impl LeaderTap {
    /// A key went down or up. True when this completes a tap of the leader.
    ///
    /// A modifier leader (left Alt, AltGr, a Ctrl) acts when released with nothing pressed in
    /// between, so Alt+X still works as a shortcut; any other leader acts when pressed.
    pub fn key(&mut self, leader: Leader, key: PhysKey, pressed: bool, repeat: bool) -> bool {
        if matches!(key, PhysKey::AltLeft | PhysKey::AltRight) {
            self.alt_keys = true;
        }
        let lead = leader_key(leader);
        if key == lead {
            if !lead.is_modifier() {
                self.armed = false;
                return pressed && !repeat;
            }
            if pressed {
                if !repeat {
                    self.armed = true;
                }
                return false;
            }
            let tap = self.armed;
            self.armed = false;
            return tap;
        }
        if pressed {
            self.armed = false;
        }
        false
    }

    /// A click cancels a tap in progress.
    pub fn pointer(&mut self) {
        self.armed = false;
    }

    /// Once a frame, with the Alt modifier as the platform reports it. Some platforms never
    /// send Alt as a key, only as a modifier; there, the left Alt leader is read from the
    /// modifier. Where Alt keys do arrive (AltGr reported as `ISO_Level3_Shift` comes with the
    /// Alt modifier, but also as the key `AltRight`), the modifier is never read, so AltGr can
    /// not arm the leader this way.
    pub fn modifiers(&mut self, leader: Leader, alt: bool) -> bool {
        let mut tap = false;
        if leader == Leader::Alt && !self.alt_keys {
            if alt && !self.alt_was {
                self.armed = true;
            }
            if !alt && self.alt_was && self.armed {
                self.armed = false;
                tap = true;
            }
        }
        self.alt_was = alt;
        tap
    }
}

/// The modifiers a shortcut sees: AltGr down without the left Alt is not Alt.
pub fn shortcut_mods(m: egui::Modifiers, altgr_down: bool, alt_left_down: bool) -> Mods {
    let mut mods = Mods::from_egui(m);
    if altgr_down && !alt_left_down {
        mods.alt = false;
    }
    mods
}

#[cfg(test)]
mod tests {
    use super::*;
    use PhysKey::*;

    fn tap(t: &mut LeaderTap, leader: Leader, key: PhysKey, alt_modifier: bool) -> bool {
        let mut on = t.key(leader, key, true, false);
        on |= t.modifiers(leader, alt_modifier);
        on |= t.key(leader, key, false, false);
        on |= t.modifiers(leader, false);
        on
    }

    #[test]
    fn left_alt_is_the_leader_and_altgr_is_not() {
        let mut t = LeaderTap::default();
        // AltGr reported as ISO_Level3_Shift: the key AltRight, with the Alt modifier on.
        assert!(!tap(&mut t, Leader::Alt, AltRight, true));
        assert!(tap(&mut t, Leader::Alt, AltLeft, true));
        // Alt used as a modifier (Alt+G) is not a tap.
        t.key(Leader::Alt, AltLeft, true, false);
        t.key(Leader::Alt, KeyG, true, false);
        t.key(Leader::Alt, KeyG, false, false);
        assert!(!t.key(Leader::Alt, AltLeft, false, false));
    }

    #[test]
    fn altgr_as_leader_inverts() {
        let mut t = LeaderTap::default();
        let altgr = Leader::Key(AltRight);
        assert!(tap(&mut t, altgr, AltRight, true));
        assert!(!tap(&mut t, altgr, AltLeft, true));
    }

    #[test]
    fn modifier_only_platform_and_other_leaders() {
        // No Alt key events at all: the left Alt leader comes from the modifier.
        let mut t = LeaderTap::default();
        assert!(!t.modifiers(Leader::Alt, true));
        assert!(t.modifiers(Leader::Alt, false));
        // Once Alt keys are seen, the modifier alone no longer arms it (AltGr held).
        t.key(Leader::Alt, AltRight, true, false);
        assert!(!t.modifiers(Leader::Alt, true));
        t.key(Leader::Alt, AltRight, false, false);
        assert!(!t.modifiers(Leader::Alt, false));
        // Any other key as leader acts on press.
        let f12 = Leader::Key(F12);
        assert!(t.key(f12, F12, true, false));
        assert!(!t.key(f12, F12, true, true));
    }

    #[test]
    fn altgr_never_counts_as_alt_in_a_shortcut() {
        let m = egui::Modifiers {
            alt: true,
            ..Default::default()
        };
        assert!(!shortcut_mods(m, true, false).alt);
        assert!(shortcut_mods(m, true, true).alt);
        assert!(shortcut_mods(m, false, false).alt);
    }
}
