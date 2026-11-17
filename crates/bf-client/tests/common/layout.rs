//! What the layout tests share: the logical screens, a headless app at a size, and the checks
//! that every element lies on the screen and inside its window, and that no text runs into
//! another.
#![allow(dead_code)]

use std::path::PathBuf;

use bf_client::App;
use bf_client::paths::Paths;
use egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Logical screens: the floor, 4:3, 16:9, 16:10 (the Steam Deck), 21:9 and 4K at 100 %.
pub const SIZES: [(f32, f32); 10] = [
    (960.0, 540.0),
    (1024.0, 768.0),
    (1280.0, 720.0),
    (1280.0, 800.0),
    (1600.0, 900.0),
    (1920.0, 1080.0),
    (1920.0, 1200.0),
    (2560.0, 1080.0),
    (3440.0, 1440.0),
    (3840.0, 2160.0),
];

pub fn app(user: PathBuf, locale: &str, size: egui::Vec2) -> Harness<'static, App> {
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(
        user.join("settings.ron"),
        format!("(locale: \"{locale}\", ui_scale: 100, map_labels: true, autosave: Off)"),
    )
    .unwrap();
    let paths = Paths::new(&root(), user.clone(), user);
    Harness::builder()
        .with_size(size)
        .build_eframe(move |cc| App::new(cc, paths))
}

pub struct Node {
    pub role: egui::accesskit::Role,
    pub label: String,
    pub rect: Rect,
}

pub fn nodes(h: &Harness<'_, App>) -> Vec<Node> {
    let screen = h.ctx.content_rect();
    h.query_all(egui_kittest::kittest::by())
        .filter_map(|n| {
            let a = n.accesskit_node();
            let b = a.bounding_box()?;
            Some(Node {
                role: a.role(),
                label: a.label().or_else(|| a.value()).unwrap_or_default(),
                rect: Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                ),
            })
        })
        // Veils and catchers the size of the screen hold nothing to read.
        .filter(|n| {
            n.rect.width() > 0.0 && n.rect.height() > 0.0 && n.rect.area() < screen.area() * 0.9
        })
        .collect()
}

/// What failed on one screen, as lines.
pub fn check(h: &Harness<'_, App>, case: &str) -> Vec<String> {
    use egui::accesskit::Role;
    let screen = h.ctx.content_rect();
    let all = nodes(h);
    let is_frame = |n: &Node| matches!(n.role, Role::Window | Role::Pane);
    let scrolls: Vec<Rect> = all
        .iter()
        .filter(|n| n.role == Role::ScrollBar)
        .map(|n| n.rect)
        .collect();
    let tol = 0.5;
    let mut out = Vec::new();
    for (i, n) in all.iter().enumerate() {
        let r = n.rect;
        if is_frame(n) {
            if !screen.expand(tol).contains_rect(r) {
                out.push(format!("{case}: frame {} {r:?} off the screen", n.label));
            }
            continue;
        }
        if n.role == Role::ScrollBar {
            continue;
        }
        // Across: on the screen, and inside the frame drawn last before it under its middle
        // (the window it belongs to; frames drawn later cover it).
        if r.left() < screen.left() - tol || r.right() > screen.right() + tol {
            out.push(format!(
                "{case}: {:?} {:?} {r:?} off the screen across",
                n.role, n.label
            ));
            continue;
        }
        if let Some(f) = all[..i]
            .iter()
            .rev()
            .find(|f| is_frame(f) && f.rect.contains(r.center()))
            && (r.left() < f.rect.left() - tol || r.right() > f.rect.right() + tol)
        {
            out.push(format!(
                "{case}: {:?} {:?} {r:?} out of {} {:?}",
                n.role, n.label, f.label, f.rect
            ));
        }
        // Down: on the screen, unless an area that scrolls holds it (below its top, or
        // scrolled past above it).
        if r.top() < screen.top() - tol || r.bottom() > screen.bottom() + tol {
            let scrolled = scrolls.iter().any(|s| {
                s.right() >= r.right() - tol
                    && (s.top() <= r.top() + tol || r.bottom() <= s.top() + tol)
            });
            if !scrolled {
                out.push(format!(
                    "{case}: {:?} {:?} {r:?} off the screen down",
                    n.role, n.label
                ));
            }
        }
    }
    out
}

/// Texts side by side never run into each other: a number that does not fit would. (A cut text
/// keeps its whole label for screen readers, so the check is on where the texts lie.)
pub fn no_overlapping_text(h: &Harness<'_, App>, case: &str) -> Vec<String> {
    let labels: Vec<Node> = nodes(h)
        .into_iter()
        .filter(|n| n.role == egui::accesskit::Role::Label && !n.label.is_empty())
        .collect();
    let mut out = Vec::new();
    // A sum or a count shown narrower than it is wide was cut.
    let numeric = |t: &str| {
        t.chars().any(|c| c.is_ascii_digit())
            && (t.starts_with("US$") || t.starts_with('$') || t.chars().all(|c| c.is_ascii_digit()))
    };
    for n in labels.iter().filter(|n| numeric(&n.label)) {
        let wide = h.ctx.fonts_mut(|f| {
            f.layout_no_wrap(
                n.label.clone(),
                bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::SMALL),
                egui::Color32::WHITE,
            )
            .size()
            .x
        });
        if n.rect.width() + 1.0 < wide {
            out.push(format!(
                "{case}: {:?} cut to {:.0} of {:.0} px",
                n.label,
                n.rect.width(),
                wide
            ));
        }
    }
    for (i, a) in labels.iter().enumerate() {
        for b in &labels[i + 1..] {
            let both = a.rect.intersect(b.rect);
            let same_row = both.height() > a.rect.height().min(b.rect.height()) * 0.5;
            if same_row && both.width() > 1.0 {
                out.push(format!(
                    "{case}: {:?} {:?} runs into {:?} {:?}",
                    a.label, a.rect, b.label, b.rect
                ));
            }
        }
    }
    out
}
