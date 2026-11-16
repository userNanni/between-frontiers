//! Physical keys and keyboard layouts.
//!
//! A binding is stored by the key's position on the board ([`PhysKey`], named after the W3C
//! `KeyboardEvent.code` values), never by the character printed on it: the key right of `L` is
//! the same binding on an ABNT2 board, where it reads `Ç`, and on an ANSI board, where it reads
//! `;`. The [`Layout`] only decides what the key is labelled and how the board is drawn.

use serde::{Deserialize, Serialize};

macro_rules! phys_keys {
    ($($name:ident),* $(,)?) => {
        /// A key by its position on the board.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum PhysKey { $($name),* }

        impl PhysKey {
            pub const ALL: &[PhysKey] = &[$(PhysKey::$name),*];

            /// Stable name, as written in the settings file.
            pub fn code(self) -> &'static str {
                match self { $(PhysKey::$name => stringify!($name)),* }
            }
        }
    };
}

phys_keys!(
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Backquote,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Minus,
    Equal,
    Backspace,
    Tab,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    BracketLeft,
    BracketRight,
    Backslash,
    CapsLock,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    Quote,
    Enter,
    ShiftLeft,
    IntlBackslash,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Period,
    Slash,
    IntlRo,
    ShiftRight,
    ControlLeft,
    SuperLeft,
    AltLeft,
    Space,
    AltRight,
    SuperRight,
    ContextMenu,
    ControlRight,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowLeft,
    ArrowDown,
    ArrowRight,
);

impl PhysKey {
    /// Modifiers are layers, not keys a command can be bound to.
    pub fn is_modifier(self) -> bool {
        use PhysKey::*;
        matches!(
            self,
            ShiftLeft
                | ShiftRight
                | ControlLeft
                | ControlRight
                | AltLeft
                | AltRight
                | SuperLeft
                | SuperRight
        )
    }

    /// The key egui reports. egui names physical keys by the US position, so its
    /// `physical_key` maps one to one; when only the logical key is known (tests, platforms
    /// without physical keys) the same table is the best guess.
    pub fn from_egui(key: egui::Key) -> Option<Self> {
        use PhysKey as P;
        use egui::Key as K;
        Some(match key {
            K::Escape => P::Escape,
            K::F1 => P::F1,
            K::F2 => P::F2,
            K::F3 => P::F3,
            K::F4 => P::F4,
            K::F5 => P::F5,
            K::F6 => P::F6,
            K::F7 => P::F7,
            K::F8 => P::F8,
            K::F9 => P::F9,
            K::F10 => P::F10,
            K::F11 => P::F11,
            K::F12 => P::F12,
            K::Backtick => P::Backquote,
            K::Num1 => P::Digit1,
            K::Num2 => P::Digit2,
            K::Num3 => P::Digit3,
            K::Num4 => P::Digit4,
            K::Num5 => P::Digit5,
            K::Num6 => P::Digit6,
            K::Num7 => P::Digit7,
            K::Num8 => P::Digit8,
            K::Num9 => P::Digit9,
            K::Num0 => P::Digit0,
            K::Minus => P::Minus,
            K::Equals | K::Plus => P::Equal,
            K::Backspace => P::Backspace,
            K::Tab => P::Tab,
            K::Q => P::KeyQ,
            K::W => P::KeyW,
            K::E => P::KeyE,
            K::R => P::KeyR,
            K::T => P::KeyT,
            K::Y => P::KeyY,
            K::U => P::KeyU,
            K::I => P::KeyI,
            K::O => P::KeyO,
            K::P => P::KeyP,
            K::OpenBracket => P::BracketLeft,
            K::CloseBracket => P::BracketRight,
            K::Backslash => P::Backslash,
            K::CapsLock => P::CapsLock,
            K::A => P::KeyA,
            K::S => P::KeyS,
            K::D => P::KeyD,
            K::F => P::KeyF,
            K::G => P::KeyG,
            K::H => P::KeyH,
            K::J => P::KeyJ,
            K::K => P::KeyK,
            K::L => P::KeyL,
            K::Semicolon => P::Semicolon,
            K::Quote => P::Quote,
            K::Enter => P::Enter,
            K::ShiftLeft => P::ShiftLeft,
            K::IntlBackslash => P::IntlBackslash,
            K::Z => P::KeyZ,
            K::X => P::KeyX,
            K::C => P::KeyC,
            K::V => P::KeyV,
            K::B => P::KeyB,
            K::N => P::KeyN,
            K::M => P::KeyM,
            K::Comma => P::Comma,
            K::Period => P::Period,
            K::Slash => P::Slash,
            K::ShiftRight => P::ShiftRight,
            K::ControlLeft => P::ControlLeft,
            K::SuperLeft => P::SuperLeft,
            K::AltLeft => P::AltLeft,
            K::Space => P::Space,
            K::AltRight => P::AltRight,
            K::SuperRight => P::SuperRight,
            K::Menu => P::ContextMenu,
            K::ControlRight => P::ControlRight,
            K::Insert => P::Insert,
            K::Delete => P::Delete,
            K::Home => P::Home,
            K::End => P::End,
            K::PageUp => P::PageUp,
            K::PageDown => P::PageDown,
            K::ArrowUp => P::ArrowUp,
            K::ArrowLeft => P::ArrowLeft,
            K::ArrowDown => P::ArrowDown,
            K::ArrowRight => P::ArrowRight,
            _ => return None,
        })
    }

    /// The letter on the key, the same on every Latin layout this game draws.
    pub fn letter(self) -> Option<char> {
        let code = self.code();
        code.strip_prefix("Key").and_then(|l| l.chars().next())
    }

    fn digit(self) -> Option<char> {
        self.code()
            .strip_prefix("Digit")
            .and_then(|d| d.chars().next())
    }
}

/// Physical layouts the board can be drawn in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layout {
    /// Brazilian ABNT2 (ISO shape: tall Enter, extra keys left of Z and right of `;`).
    #[default]
    Abnt2,
    /// US ANSI.
    Ansi,
}

impl Layout {
    pub const ALL: [Layout; 2] = [Layout::Abnt2, Layout::Ansi];

    pub fn name(self) -> &'static str {
        match self {
            Layout::Abnt2 => "ABNT2",
            Layout::Ansi => "ANSI",
        }
    }

    /// What is printed on the key: the unshifted character, and the shifted one when it differs
    /// and fits. Named keys (Space, Enter) have i18n names and come back as `None`.
    pub fn legend(self, key: PhysKey) -> Option<(&'static str, Option<&'static str>)> {
        use PhysKey::*;
        if let Some(l) = key.letter() {
            return Some((LETTERS[(l as u8 - b'A') as usize], None));
        }
        if let Some(d) = key.digit() {
            let i = (d as u8 - b'0') as usize;
            let shifted = match self {
                Layout::Abnt2 => ABNT2_DIGITS[i],
                Layout::Ansi => ANSI_DIGITS[i],
            };
            return Some((DIGITS[i], Some(shifted)));
        }
        let pair = match (self, key) {
            (_, Escape) => ("Esc", None),
            (_, F1) => ("F1", None),
            (_, F2) => ("F2", None),
            (_, F3) => ("F3", None),
            (_, F4) => ("F4", None),
            (_, F5) => ("F5", None),
            (_, F6) => ("F6", None),
            (_, F7) => ("F7", None),
            (_, F8) => ("F8", None),
            (_, F9) => ("F9", None),
            (_, F10) => ("F10", None),
            (_, F11) => ("F11", None),
            (_, F12) => ("F12", None),
            (_, Minus) => ("-", Some("_")),
            (_, Equal) => ("=", Some("+")),
            (_, Comma) => (",", Some("<")),
            (_, Period) => (".", Some(">")),
            (Layout::Abnt2, Backquote) => ("'", Some("\"")),
            (Layout::Abnt2, BracketLeft) => ("´", Some("`")),
            (Layout::Abnt2, BracketRight) => ("[", Some("{")),
            (Layout::Abnt2, Backslash) => ("]", Some("}")),
            (Layout::Abnt2, Semicolon) => ("Ç", None),
            (Layout::Abnt2, Quote) => ("~", Some("^")),
            (Layout::Abnt2, IntlBackslash) => ("\\", Some("|")),
            (Layout::Abnt2, Slash) => (";", Some(":")),
            (Layout::Abnt2, IntlRo) => ("/", Some("?")),
            (Layout::Ansi, Backquote) => ("`", Some("~")),
            (Layout::Ansi, BracketLeft) => ("[", Some("{")),
            (Layout::Ansi, BracketRight) => ("]", Some("}")),
            (Layout::Ansi, Backslash) => ("\\", Some("|")),
            (Layout::Ansi, Semicolon) => (";", Some(":")),
            (Layout::Ansi, Quote) => ("'", Some("\"")),
            (Layout::Ansi, Slash) => ("/", Some("?")),
            (Layout::Ansi, IntlBackslash) => ("\\", Some("|")),
            (Layout::Ansi, IntlRo) => ("/", Some("?")),
            _ => return None,
        };
        Some(pair)
    }

    /// The board, row by row: each key with its width in key units. Gaps are `None`.
    pub fn rows(self) -> Vec<Vec<(Option<PhysKey>, f32)>> {
        use PhysKey::*;
        let k = |key: PhysKey| (Some(key), 1.0);
        let w = |key: PhysKey, width: f32| (Some(key), width);
        let gap = |width: f32| (None, width);
        let mut rows = vec![vec![
            k(Escape),
            gap(1.0),
            k(F1),
            k(F2),
            k(F3),
            k(F4),
            gap(0.5),
            k(F5),
            k(F6),
            k(F7),
            k(F8),
            gap(0.5),
            k(F9),
            k(F10),
            k(F11),
            k(F12),
        ]];
        let digits = vec![
            k(Backquote),
            k(Digit1),
            k(Digit2),
            k(Digit3),
            k(Digit4),
            k(Digit5),
            k(Digit6),
            k(Digit7),
            k(Digit8),
            k(Digit9),
            k(Digit0),
            k(Minus),
            k(Equal),
            w(Backspace, 2.0),
        ];
        let qwerty = [KeyQ, KeyW, KeyE, KeyR, KeyT, KeyY, KeyU, KeyI, KeyO, KeyP]
            .map(k)
            .to_vec();
        let home = [KeyA, KeyS, KeyD, KeyF, KeyG, KeyH, KeyJ, KeyK, KeyL]
            .map(k)
            .to_vec();
        let bottom = [
            KeyZ, KeyX, KeyC, KeyV, KeyB, KeyN, KeyM, Comma, Period, Slash,
        ]
        .map(k)
        .to_vec();
        let cat = |a: Vec<_>, b: Vec<_>, c: Vec<_>| [a, b, c].concat();
        match self {
            Layout::Abnt2 => {
                rows.push(digits);
                rows.push(cat(
                    vec![w(Tab, 1.5)],
                    qwerty,
                    vec![k(BracketLeft), k(BracketRight), w(Enter, 1.5)],
                ));
                rows.push(cat(
                    vec![w(CapsLock, 1.75)],
                    home,
                    vec![k(Semicolon), k(Quote), k(Backslash), w(Enter, 1.25)],
                ));
                rows.push(cat(
                    vec![w(ShiftLeft, 1.25), k(IntlBackslash)],
                    bottom,
                    vec![k(IntlRo), w(ShiftRight, 1.75)],
                ));
            }
            Layout::Ansi => {
                rows.push(digits);
                rows.push(cat(
                    vec![w(Tab, 1.5)],
                    qwerty,
                    vec![k(BracketLeft), k(BracketRight), w(Backslash, 1.5)],
                ));
                rows.push(cat(
                    vec![w(CapsLock, 1.75)],
                    home,
                    vec![k(Semicolon), k(Quote), w(Enter, 2.25)],
                ));
                rows.push(cat(
                    vec![w(ShiftLeft, 2.25)],
                    bottom,
                    vec![w(ShiftRight, 2.75)],
                ));
            }
        }
        rows.push(vec![
            w(ControlLeft, 1.25),
            w(SuperLeft, 1.25),
            w(AltLeft, 1.25),
            w(Space, 6.25),
            w(AltRight, 1.25),
            w(SuperRight, 1.25),
            w(ContextMenu, 1.25),
            w(ControlRight, 1.25),
        ]);
        rows
    }

    /// The navigation block right of the board: rows aligned with the board's rows 1–5.
    pub fn nav_rows() -> [[Option<PhysKey>; 3]; 5] {
        use PhysKey::*;
        [
            [Some(Insert), Some(Home), Some(PageUp)],
            [Some(Delete), Some(End), Some(PageDown)],
            [None, None, None],
            [None, Some(ArrowUp), None],
            [Some(ArrowLeft), Some(ArrowDown), Some(ArrowRight)],
        ]
    }

    /// Every key the layout draws.
    pub fn keys(self) -> Vec<PhysKey> {
        let mut out: Vec<PhysKey> = self
            .rows()
            .into_iter()
            .flatten()
            .filter_map(|k| k.0)
            .collect();
        out.extend(Self::nav_rows().into_iter().flatten().flatten());
        let mut seen = std::collections::HashSet::new();
        out.retain(|k| seen.insert(*k));
        out
    }
}

const LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];
const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
const ANSI_DIGITS: [&str; 10] = [")", "!", "@", "#", "$", "%", "^", "&", "*", "("];
const ABNT2_DIGITS: [&str; 10] = [")", "!", "@", "#", "$", "%", "¨", "&", "*", "("];

#[cfg(test)]
mod tests {
    use super::*;

    /// The same physical key is the same binding in both layouts, whatever it reads.
    #[test]
    fn physical_key_is_the_same_in_abnt2_and_ansi() {
        let abnt2 = Layout::Abnt2.keys();
        let ansi = Layout::Ansi.keys();
        // Every key both boards have resolves from the same egui key to the same PhysKey.
        for key in [
            egui::Key::Semicolon,
            egui::Key::G,
            egui::Key::Slash,
            egui::Key::Num1,
        ] {
            let p = PhysKey::from_egui(key).unwrap();
            assert!(abnt2.contains(&p) && ansi.contains(&p), "{p:?}");
        }
        // The key right of L: same position, different legend.
        assert_eq!(Layout::Abnt2.legend(PhysKey::Semicolon).unwrap().0, "Ç");
        assert_eq!(Layout::Ansi.legend(PhysKey::Semicolon).unwrap().0, ";");
        // Letters read the same.
        assert_eq!(
            Layout::Abnt2.legend(PhysKey::KeyG),
            Layout::Ansi.legend(PhysKey::KeyG)
        );
        // ABNT2 has the two ISO extras; ANSI does not.
        assert!(abnt2.contains(&PhysKey::IntlBackslash) && abnt2.contains(&PhysKey::IntlRo));
        assert!(!ansi.contains(&PhysKey::IntlRo));
        // A chord stored on one layout survives a round trip through the file unchanged.
        let text = ron::to_string(&PhysKey::Semicolon).unwrap();
        assert_eq!(text, "Semicolon");
    }

    #[test]
    fn rows_are_fifteen_units_wide() {
        for layout in Layout::ALL {
            for row in layout.rows().iter().skip(1) {
                let w: f32 = row.iter().map(|k| k.1).sum();
                assert!((w - 15.0).abs() < 1e-3, "{layout:?}: {w}");
            }
        }
    }
}
