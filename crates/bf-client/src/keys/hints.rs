//! Key tips: with the leader key, a letter shows over every element the player can act on;
//! typing it acts.
//!
//! Codes come from the element, not from where it sits: a command brings its own letter, a
//! list item (a law group, a resource) brings a family letter and its own key. Within one
//! screen the codes never collide and none is the start of another, so typing is unambiguous.

use std::collections::{BTreeMap, HashSet};

use egui::Rect;

/// How an element asks for its code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Spec {
    /// A registered command's own letter; wins a tie with [`Spec::Single`].
    Command(char),
    /// One character, preferably this one.
    Single(char),
    /// Two characters: the family's, then one taken from the item's key.
    Item { family: char, key: String },
}

#[derive(Clone, Debug)]
pub struct Target {
    pub id: String,
    pub spec: Spec,
    pub rect: Rect,
    /// Where the label would rather go.
    pub side: Side,
}

/// Preferred place of a label, tried before the usual corners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Auto,
    Left,
    Right,
}

const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Codes for one screen's targets, by target id. Depends only on the set of targets, never on
/// the order they were drawn in.
pub fn assign(targets: &[Target]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    // (rank, id, letter): commands first.
    let mut singles: Vec<(u8, &str, char)> = Vec::new();
    let mut families: BTreeMap<char, Vec<(&str, &str)>> = BTreeMap::new();
    for t in targets {
        match &t.spec {
            Spec::Command(c) => singles.push((0, &t.id, c.to_ascii_uppercase())),
            Spec::Single(c) => singles.push((1, &t.id, c.to_ascii_uppercase())),
            Spec::Item { family, key } => families
                .entry(family.to_ascii_uppercase())
                .or_default()
                .push((&t.id, key)),
        }
    }
    singles.sort();
    let mut seen_ids = HashSet::new();
    singles.retain(|s| seen_ids.insert(s.1));
    // First characters in use: families first, they hold many items each.
    let mut used: HashSet<char> = families.keys().copied().collect();
    // A single that asks for its own letter wins over one that only falls back to it: first
    // pass gives preferred letters, in id order; second pass places the rest.
    let mut pending = Vec::new();
    for (_, id, c) in &singles {
        if used.insert(*c) {
            out.insert((*id).to_owned(), c.to_string());
        } else {
            pending.push(*id);
        }
    }
    for id in pending {
        let letter = id
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .filter(char::is_ascii_alphanumeric)
            .chain(ALPHABET.chars())
            .find(|c| !used.contains(c));
        if let Some(c) = letter {
            used.insert(c);
            out.insert(id.to_owned(), c.to_string());
        }
        // Out of letters: the element goes without a tip; the mouse still reaches it.
    }
    for (family, mut items) in families {
        items.sort();
        items.dedup_by(|a, b| a.0 == b.0);
        let mut taken = HashSet::new();
        for (id, key) in items {
            let second = key
                .chars()
                .map(|c| c.to_ascii_uppercase())
                .filter(char::is_ascii_alphanumeric)
                .chain(ALPHABET.chars())
                .chain("0123456789".chars())
                .find(|c| !taken.contains(c));
            if let Some(c) = second {
                taken.insert(c);
                out.insert(id.to_owned(), format!("{family}{c}"));
            }
        }
    }
    out
}

/// What the typed characters do against the codes on screen.
#[derive(Debug, PartialEq, Eq)]
pub enum Typed {
    /// A whole code: act on this target.
    Hit(String),
    /// The start of one or more codes: keep typing.
    Partial,
    /// Nothing starts this way.
    Miss,
}

pub fn match_typed(codes: &BTreeMap<String, String>, typed: &str) -> Typed {
    if typed.is_empty() {
        return Typed::Partial;
    }
    if let Some((id, _)) = codes.iter().find(|(_, c)| c.as_str() == typed) {
        return Typed::Hit(id.clone());
    }
    if codes.values().any(|c| c.starts_with(typed)) {
        Typed::Partial
    } else {
        Typed::Miss
    }
}

/// Where each tip's label goes: on a corner of its element, outside it so the element's text
/// stays readable, and never over another label. `items` are (element, label size), in a fixed
/// order (by target id), so the same screen lays out the same way every frame.
pub fn place_labels(items: &[(Rect, egui::Vec2, Side)], screen: Rect) -> Vec<Rect> {
    let mut placed: Vec<Rect> = Vec::with_capacity(items.len());
    for (i, &(e, s, side)) in items.iter().enumerate() {
        let at = |x: f32, y: f32| Rect::from_min_size(egui::pos2(x, y), s);
        let left = at(e.left() - s.x - 2.0, e.center().y - s.y / 2.0);
        let right = at(e.right() + 2.0, e.center().y - s.y / 2.0);
        // Above the element, overlapping only its top padding, never a neighbour beside it.
        let above_right = at(e.right() - s.x - 2.0, e.top() - s.y + 1.0);
        let above_left = at(e.left() + 2.0, e.top() - s.y + 1.0);
        let below_right = at(e.right() - s.x - 2.0, e.bottom() - 1.0);
        let below_left = at(e.left() + 2.0, e.bottom() - 1.0);
        // Wide rows read from the left: the label sits before the text. Buttons, tabs and
        // other controls side by side take it above their right corner.
        let first = match side {
            Side::Left => Some(left),
            Side::Right => Some(right),
            Side::Auto => None,
        };
        let usual = if e.width() >= 180.0 {
            [left, above_left, below_left, above_right, below_right]
        } else {
            [above_right, above_left, below_right, below_left, left]
        };
        let candidates: Vec<Rect> = first.into_iter().chain(usual).collect();
        let fit = |r: Rect| keep_on(r, screen);
        // Clear of the labels already placed, and of every other element: a label never
        // covers something the player could act on.
        let free = |r: &Rect| {
            placed.iter().all(|p| !p.shrink(0.5).intersects(*r))
                && items
                    .iter()
                    .enumerate()
                    .all(|(j, o)| j == i || !o.0.shrink(1.0).intersects(*r))
        };
        let chosen = candidates
            .iter()
            .map(|r| fit(*r))
            .find(free)
            .unwrap_or_else(|| {
                // Every corner taken: slide from the first one until it is clear.
                let first = fit(candidates[0]);
                (1..=24)
                    .map(|k| {
                        let step = k as f32 * (s.y + 2.0) / 2.0;
                        let d = if k % 2 == 0 { step } else { -step };
                        fit(first.translate(egui::vec2(0.0, d)))
                    })
                    .find(free)
                    .unwrap_or(first)
            });
        placed.push(chosen);
    }
    placed
}

fn keep_on(r: Rect, screen: Rect) -> Rect {
    let dx =
        (screen.left() + 2.0 - r.left()).max(0.0) - (r.right() - screen.right() + 2.0).max(0.0);
    let dy =
        (screen.top() + 2.0 - r.top()).max(0.0) - (r.bottom() - screen.bottom() + 2.0).max(0.0);
    r.translate(egui::vec2(dx, dy))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single(id: &str, c: char) -> Target {
        Target {
            id: id.into(),
            spec: if id.starts_with("other") {
                Spec::Single(c)
            } else {
                Spec::Command(c)
            },
            rect: Rect::NOTHING,
            side: Side::Auto,
        }
    }

    fn item(id: &str, family: char, key: &str) -> Target {
        Target {
            id: id.into(),
            spec: Spec::Item {
                family,
                key: key.into(),
            },
            rect: Rect::NOTHING,
            side: Side::Auto,
        }
    }

    fn assert_prefix_free(codes: &BTreeMap<String, String>) {
        let all: Vec<&String> = codes.values().collect();
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert!(!b.starts_with(a.as_str()), "{a} is the start of {b}");
                }
            }
        }
    }

    /// The laws screen: the HUD's buttons, the window's tabs, law groups and laws.
    #[test]
    fn codes_never_collide_on_a_screen_and_do_not_depend_on_order() {
        let mut targets = vec![
            single("time.pause", 'P'),
            single("time.speed_down", 'D'),
            single("time.speed_up", 'U'),
            single("window.government", 'G'),
            single("window.research", 'R'),
            single("window.menu", 'N'),
            single("window.country", 'C'),
            single("gov.tab_budget", 'B'),
            single("gov.tab_money", 'M'),
            single("gov.tab_laws", 'L'),
            single("gov.close", 'X'),
            single("gov.propose_law", 'E'),
            // Two elements asking for the same letter.
            single("other.thing", 'G'),
        ];
        for key in [
            "taxation",
            "trade",
            "tariffs",
            "migration",
            "citizenship",
            "colonial_labour",
            "press",
            "church",
            "suffrage",
            "unions",
        ] {
            targets.push(item(&format!("laws.group.{key}"), 'H', key));
        }
        for key in ["flat", "progressive", "none"] {
            targets.push(item(&format!("laws.law.{key}"), 'J', key));
        }
        let codes = assign(&targets);
        assert_eq!(codes.len(), targets.len(), "every target has a code");
        let unique: HashSet<&String> = codes.values().collect();
        assert_eq!(unique.len(), codes.len());
        assert_prefix_free(&codes);
        // Commands keep their letters.
        assert_eq!(codes["window.government"], "G");
        assert_eq!(codes["gov.tab_laws"], "L");
        assert_ne!(codes["other.thing"], "G");
        // The same set drawn in another order gets the same codes.
        let mut shuffled = targets.clone();
        shuffled.reverse();
        shuffled.rotate_left(5);
        assert_eq!(assign(&shuffled), codes);
        // Typing.
        assert_eq!(
            match_typed(&codes, "G"),
            Typed::Hit("window.government".into())
        );
        assert_eq!(match_typed(&codes, "H"), Typed::Partial);
        let taxation = &codes["laws.group.taxation"];
        assert_eq!(
            match_typed(&codes, taxation),
            Typed::Hit("laws.group.taxation".into())
        );
    }

    #[test]
    fn a_family_letter_pushes_a_single_off_it() {
        let codes = assign(&[single("a.b", 'H'), item("x.1", 'H', "one")]);
        assert_ne!(codes["a.b"], "H");
        assert_prefix_free(&codes);
    }

    /// Labels of a dense row of small buttons and a column of rows never overlap, and stay
    /// outside their elements' middle.
    #[test]
    fn labels_never_overlap() {
        let screen = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));
        let size = egui::vec2(22.0, 18.0);
        let mut items = Vec::new();
        // Six 28 px steppers side by side, 4 px apart, and two rows of them.
        for row in 0..2 {
            for i in 0..6 {
                let r = Rect::from_min_size(
                    egui::pos2(600.0 + i as f32 * 32.0, 300.0 + row as f32 * 32.0),
                    egui::vec2(28.0, 28.0),
                );
                items.push((r, size, Side::Auto));
            }
        }
        // Tabs touching each other.
        for i in 0..3 {
            let r = Rect::from_min_size(
                egui::pos2(300.0 + i as f32 * 92.0, 180.0),
                egui::vec2(90.0, 28.0),
            );
            items.push((r, size, Side::Auto));
        }
        // A column of list rows.
        for i in 0..12 {
            let r = Rect::from_min_size(
                egui::pos2(40.0, 200.0 + i as f32 * 44.0),
                egui::vec2(230.0, 40.0),
            );
            items.push((r, size, Side::Auto));
        }
        // An element in the screen's corner.
        items.push((
            Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(60.0, 60.0)),
            size,
            Side::Auto,
        ));
        let placed = place_labels(&items, screen);
        for (i, a) in placed.iter().enumerate() {
            assert!(screen.contains_rect(*a), "{a:?} off screen");
            for b in &placed[i + 1..] {
                assert!(!a.shrink(0.5).intersects(*b), "{a:?} overlaps {b:?}");
            }
            // The label never covers the middle of its element, where the text is.
            let e = items[i].0;
            assert!(!a.contains(e.center()), "{a:?} on the text of {e:?}");
        }
    }
}
