//! The HUD layout: where each panel of the campaign screen sits, whether it shows, the free area
//! the HUD keeps clear (a streamer's camera), and where the windows were left.
//!
//! A panel's place is an anchor of the screen (a corner, an edge's middle or the centre) and an
//! offset from it, so a layout survives a change of resolution. The rectangles are resolved
//! from the layout, the screen and the panels' sizes, and kept until one of them changes:
//! nothing is worked out per frame.
//!
//! The layout belongs to the player, not to a campaign: it is stored in the config folder
//! (`hud.ron`), versioned, and never enters a save or the simulation's hash.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use egui::{Pos2, Rect, Vec2, pos2, vec2};
use serde::{Deserialize, Serialize};

/// Layout file version. A file of an older version is migrated on load; panels it does not
/// know take their default place.
pub const VERSION: u32 = 1;
/// Space kept between a panel and the screen edge, and between two panels.
pub const GAP: f32 = 12.0;
/// How close an edge must come to another to snap to it.
pub const SNAP: f32 = 10.0;

/// The panels of the campaign screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelId {
    /// The player's banner and figures, top left.
    Country,
    /// Date, speed and the window buttons, top right.
    Time,
    /// Works, research and pinned agendas.
    Outliner,
    /// The map mode picker.
    MapModes,
    /// The panel of the country clicked on the map.
    CountryPanel,
}

impl PanelId {
    /// Placement order: fixed panels first, the ones that shrink to fit last.
    pub const ALL: [PanelId; 5] = [
        PanelId::Country,
        PanelId::Time,
        PanelId::MapModes,
        PanelId::CountryPanel,
        PanelId::Outliner,
    ];

    /// Key in the layout file.
    pub fn key(self) -> &'static str {
        match self {
            PanelId::Country => "country",
            PanelId::Time => "time",
            PanelId::Outliner => "overview",
            PanelId::MapModes => "map-modes",
            PanelId::CountryPanel => "country-panel",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == key)
    }

    /// Message id of its name.
    pub fn label(self) -> &'static str {
        match self {
            PanelId::Country => "hud-panel-country",
            PanelId::Time => "hud-panel-time",
            PanelId::Outliner => "hud-panel-overview",
            PanelId::MapModes => "hud-panel-map-modes",
            PanelId::CountryPanel => "hud-panel-country-panel",
        }
    }

    /// The panel may grow shorter to make room, down to this height.
    pub fn min_height(self) -> Option<f32> {
        match self {
            PanelId::Outliner => Some(72.0),
            PanelId::CountryPanel => Some(240.0),
            _ => None,
        }
    }
}

/// A point of a rectangle: its corners, the middles of its edges and its centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Anchor {
    /// Fractions of the width and height.
    pub fn fraction(self) -> Vec2 {
        let (x, y) = match self {
            Anchor::TopLeft => (0.0, 0.0),
            Anchor::Top => (0.5, 0.0),
            Anchor::TopRight => (1.0, 0.0),
            Anchor::Left => (0.0, 0.5),
            Anchor::Center => (0.5, 0.5),
            Anchor::Right => (1.0, 0.5),
            Anchor::BottomLeft => (0.0, 1.0),
            Anchor::Bottom => (0.5, 1.0),
            Anchor::BottomRight => (1.0, 1.0),
        };
        vec2(x, y)
    }

    pub fn point(self, r: Rect) -> Pos2 {
        r.min + r.size() * self.fraction()
    }

    /// The anchor a rectangle is nearest to: the third of the screen its centre falls in.
    pub fn nearest(r: Rect, screen: Rect) -> Self {
        let c = (r.center() - screen.min) / screen.size();
        let third = |v: f32| {
            if v < 1.0 / 3.0 {
                0
            } else if v < 2.0 / 3.0 {
                1
            } else {
                2
            }
        };
        match (third(c.x), third(c.y)) {
            (0, 0) => Anchor::TopLeft,
            (1, 0) => Anchor::Top,
            (2, 0) => Anchor::TopRight,
            (0, 1) => Anchor::Left,
            (1, 1) => Anchor::Center,
            (2, 1) => Anchor::Right,
            (0, _) => Anchor::BottomLeft,
            (1, _) => Anchor::Bottom,
            _ => Anchor::BottomRight,
        }
    }
}

/// Where a panel sits: the same anchor point of the panel and of the screen, `offset` apart.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub anchor: Anchor,
    pub offset: [f32; 2],
    pub visible: bool,
}

impl Placement {
    const fn at(anchor: Anchor, x: f32, y: f32) -> Self {
        Self {
            anchor,
            offset: [x, y],
            visible: true,
        }
    }

    /// The rectangle of a panel of `size` on `screen`.
    pub fn rect(&self, screen: Rect, size: Vec2) -> Rect {
        let f = self.anchor.fraction();
        let point = self.anchor.point(screen) + Vec2::from(self.offset);
        Rect::from_min_size(point - size * f, size)
    }

    /// The placement that puts a panel at `r`, anchored where it is nearest.
    pub fn of_rect(r: Rect, screen: Rect, visible: bool) -> Self {
        let anchor = Anchor::nearest(r, screen);
        let d = anchor.point(r) - anchor.point(screen);
        Self {
            anchor,
            offset: [d.x.round(), d.y.round()],
            visible,
        }
    }
}

/// A rectangle anchored like a panel: the free area.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub place: Placement,
    pub size: [f32; 2],
}

impl Area {
    pub fn rect(&self, screen: Rect) -> Rect {
        self.place.rect(screen, Vec2::from(self.size))
    }

    pub fn of_rect(r: Rect, screen: Rect) -> Self {
        Self {
            place: Placement::of_rect(r, screen, true),
            size: [r.width().round(), r.height().round()],
        }
    }
}

/// Where a window was moved from its default place, and the size it was given.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WindowPlace {
    pub offset: [f32; 2],
    #[serde(default)]
    pub size: Option<[f32; 2]>,
}

impl WindowPlace {
    /// The window's rectangle: its default one moved and resized, kept on `screen`.
    pub fn rect(&self, default: Rect, screen: Rect, min: Vec2) -> Rect {
        let size = self
            .size
            .map_or(default.size(), Vec2::from)
            .max(min)
            .min(screen.size());
        let r = Rect::from_min_size(default.min + Vec2::from(self.offset), size);
        clamp_into(r, screen)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    #[default]
    Standard,
    Compact,
    /// A corner left free for a camera.
    Streaming,
    /// Changed by hand.
    Custom,
}

impl Preset {
    pub const CHOICES: [Preset; 3] = [Preset::Standard, Preset::Compact, Preset::Streaming];

    pub fn label(self) -> &'static str {
        match self {
            Preset::Standard => "hud-preset-standard",
            Preset::Compact => "hud-preset-compact",
            Preset::Streaming => "hud-preset-streaming",
            Preset::Custom => "hud-preset-custom",
        }
    }
}

/// Default places, matching the HUD as it was drawn before it could move.
fn default_place(p: PanelId) -> Placement {
    use crate::screens::top_bar::{LEFT_BOTTOM, RIGHT_BOTTOM};
    match p {
        PanelId::Country => Placement::at(Anchor::TopLeft, GAP, GAP),
        PanelId::Time => Placement::at(Anchor::TopRight, -GAP, GAP),
        PanelId::Outliner => Placement::at(Anchor::TopLeft, GAP, LEFT_BOTTOM),
        PanelId::MapModes => Placement::at(Anchor::BottomLeft, GAP, -GAP),
        PanelId::CountryPanel => Placement::at(Anchor::TopRight, -16.0, RIGHT_BOTTOM),
    }
}

/// Size of the free corner of the streaming preset: as wide as a 1280 px screen leaves beside
/// the country figures.
pub const STREAM_AREA: [f32; 2] = [440.0, 280.0];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub version: u32,
    #[serde(default)]
    pub preset: Preset,
    /// By panel key.
    #[serde(default)]
    pub panels: BTreeMap<String, Placement>,
    /// The rectangle the HUD keeps clear.
    #[serde(default)]
    pub free: Option<Area>,
    /// By window key.
    #[serde(default)]
    pub windows: BTreeMap<String, WindowPlace>,
    /// The blocks of the country figures, in order; `None` is the standard set.
    #[serde(default)]
    pub tiles: Option<Vec<String>>,
    /// The sections of the overview, in order; `None` is all of them.
    #[serde(default)]
    pub overview: Option<Vec<String>>,
    /// The map mode picker kept open as a panel; folded into a button otherwise.
    #[serde(default)]
    pub map_modes_open: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self::preset(Preset::Standard)
    }
}

impl Layout {
    pub fn preset(preset: Preset) -> Self {
        let mut panels: BTreeMap<String, Placement> = PanelId::ALL
            .iter()
            .map(|p| (p.key().to_owned(), default_place(*p)))
            .collect();
        let mut free = None;
        match preset {
            Preset::Standard | Preset::Custom => {}
            Preset::Compact => {
                // The overview takes the map modes' corner; the map modes hide.
                panels.insert(
                    PanelId::Outliner.key().into(),
                    Placement::at(Anchor::BottomLeft, GAP, -GAP),
                );
                if let Some(m) = panels.get_mut(PanelId::MapModes.key()) {
                    m.visible = false;
                }
            }
            Preset::Streaming => {
                // The camera's corner, top right; the date and buttons go to the bottom right
                // and the country panel under the camera.
                free = Some(Area {
                    place: Placement::at(Anchor::TopRight, -GAP, GAP),
                    size: STREAM_AREA,
                });
                panels.insert(
                    PanelId::Time.key().into(),
                    Placement::at(Anchor::BottomRight, -GAP, -GAP),
                );
                panels.insert(
                    PanelId::CountryPanel.key().into(),
                    Placement::at(Anchor::TopRight, -GAP, GAP + STREAM_AREA[1] + GAP),
                );
            }
        }
        // The compact preset keeps the two first rows of figures.
        let tiles = (preset == Preset::Compact).then(|| {
            crate::screens::tiles::DEFAULT[..crate::screens::tiles::COLS * 2]
                .iter()
                .map(|k| (*k).to_owned())
                .collect()
        });
        Self {
            version: VERSION,
            preset,
            panels,
            free,
            windows: BTreeMap::new(),
            tiles,
            overview: None,
            map_modes_open: false,
        }
    }

    /// The blocks of the country figures, in order.
    pub fn tiles(&self) -> Vec<&'static str> {
        use crate::screens::tiles;
        match &self.tiles {
            Some(list) => list
                .iter()
                .filter_map(|k| tiles::def(k).map(|d| d.key))
                .take(tiles::MAX)
                .collect(),
            None => tiles::DEFAULT.to_vec(),
        }
    }

    pub fn set_tiles(&mut self, keys: Vec<String>) {
        self.tiles = (keys
            .iter()
            .map(String::as_str)
            .ne(crate::screens::tiles::DEFAULT))
        .then_some(keys);
        self.preset = Preset::Custom;
    }

    /// The sections of the overview, in order.
    pub fn overview(&self) -> Vec<&'static str> {
        use crate::screens::tiles::SECTIONS;
        match &self.overview {
            Some(list) => list
                .iter()
                .filter_map(|k| SECTIONS.iter().find(|s| s.0 == k).map(|s| s.0))
                .collect(),
            None => SECTIONS.iter().map(|s| s.0).collect(),
        }
    }

    pub fn set_overview(&mut self, keys: Vec<String>) {
        let all: Vec<&str> = crate::screens::tiles::SECTIONS
            .iter()
            .map(|s| s.0)
            .collect();
        self.overview = (keys.iter().map(String::as_str).ne(all)).then_some(keys);
        self.preset = Preset::Custom;
    }

    pub fn place(&self, p: PanelId) -> Placement {
        self.panels
            .get(p.key())
            .copied()
            .unwrap_or_else(|| default_place(p))
    }

    pub fn visible(&self, p: PanelId) -> bool {
        self.place(p).visible
    }

    pub fn set_place(&mut self, p: PanelId, place: Placement) {
        self.panels.insert(p.key().to_owned(), place);
        self.preset = Preset::Custom;
    }

    pub fn window(&self, key: &str) -> WindowPlace {
        self.windows.get(key).copied().unwrap_or_default()
    }

    /// Fills what an older file lacks; drops panels this version does not know.
    pub fn migrate(mut self) -> Self {
        self.panels.retain(|k, _| PanelId::from_key(k).is_some());
        // Blocks this version does not know drop out; an empty list is the standard set.
        if let Some(list) = &mut self.tiles {
            list.retain(|k| crate::screens::tiles::def(k).is_some());
            if list.is_empty() {
                self.tiles = None;
            }
        }
        if let Some(list) = &mut self.overview {
            list.retain(|k| crate::screens::tiles::SECTIONS.iter().any(|s| s.0 == k));
            if list.is_empty() {
                self.overview = None;
            }
        }
        for p in PanelId::ALL {
            self.panels
                .entry(p.key().to_owned())
                .or_insert_with(|| default_place(p));
        }
        self.version = VERSION;
        self
    }

    /// Reads a layout; the default one when the file is missing or unreadable.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| Self::parse(&t).ok())
            .unwrap_or_default()
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let layout: Layout = ron::from_str(text).map_err(|e| e.to_string())?;
        if layout.version > VERSION {
            return Err(format!("layout version {} is newer", layout.version));
        }
        Ok(layout.migrate())
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(std::io::Error::other)?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(tmp, path)
    }
}

/// The layout file in the config folder.
pub fn file(config: &Path) -> PathBuf {
    config.join("hud.ron")
}

/// Where the layout is exported to and imported from.
pub fn export_file(config: &Path) -> PathBuf {
    config.join("hud-export.ron")
}

// ---------------------------------------------------------------------------------------------
// Resolving rectangles

/// The rectangles of the visible panels on one screen.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolved {
    pub rects: BTreeMap<PanelId, Rect>,
    pub free: Option<Rect>,
}

impl Resolved {
    pub fn rect(&self, p: PanelId) -> Option<Rect> {
        self.rects.get(&p).copied()
    }
}

/// Moves `r` inside `screen`, keeping its size when it fits.
pub fn clamp_into(r: Rect, screen: Rect) -> Rect {
    let size = r.size().min(screen.size());
    let min = pos2(
        r.min.x.clamp(screen.min.x, screen.max.x - size.x),
        r.min.y.clamp(screen.min.y, screen.max.y - size.y),
    );
    Rect::from_min_size(min, size)
}

/// Ways out of the blocks for `r`: beside each block (left, right, above, below), and for a
/// panel that may shrink, its bottom raised above a block or its top lowered under one.
fn candidates(r: Rect, blocks: &[Rect], min_height: Option<f32>) -> Vec<Rect> {
    let mut out = Vec::new();
    for b in blocks {
        out.push(r.translate(vec2(b.left() - GAP - r.right(), 0.0)));
        out.push(r.translate(vec2(b.right() + GAP - r.left(), 0.0)));
        out.push(r.translate(vec2(0.0, b.top() - GAP - r.bottom())));
        out.push(r.translate(vec2(0.0, b.bottom() + GAP - r.top())));
        if let Some(min) = min_height {
            let mut up = r;
            up.set_bottom(b.top() - GAP);
            let mut down = r;
            down.set_top(b.bottom() + GAP);
            out.extend([up, down].into_iter().filter(|c| c.height() >= min));
        }
    }
    out
}

/// How far a candidate is from where the panel wanted to be.
fn cost(c: Rect, r: Rect) -> f32 {
    (c.min - r.min).length() + (c.max - r.max).length()
}

/// The nearest place for `r` that is inside `screen` and clear of every block; `r` itself when
/// it already is, `None` when nothing within two steps is.
fn clear_place(r: Rect, blocks: &[Rect], screen: Rect, min_height: Option<f32>) -> Option<Rect> {
    let clear = |c: &Rect| screen.contains_rect(*c) && !blocks.iter().any(|b| overlaps(*c, *b));
    if clear(&r) {
        return Some(r);
    }
    let first = candidates(r, blocks, min_height);
    let mut best = first
        .iter()
        .copied()
        .filter(clear)
        .min_by(|a, b| cost(*a, r).total_cmp(&cost(*b, r)));
    if best.is_none() {
        // Two steps: out of one block, then out of the next.
        best = first
            .iter()
            .filter(|c| screen.contains_rect(**c))
            .flat_map(|c| candidates(*c, blocks, min_height))
            .filter(clear)
            .min_by(|a, b| cost(*a, r).total_cmp(&cost(*b, r)));
    }
    best
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.intersects(b) && a.intersect(b).area() > 1.0
}

/// Places every visible panel of `sizes` (a panel without a size is not on screen): at its
/// anchor, kept inside the screen, then at the nearest place off the free area and off the
/// panels placed before it; a panel that may shrink can give up height instead. One with no
/// clear place near stays where it is.
pub fn resolve(layout: &Layout, screen: Rect, sizes: &BTreeMap<PanelId, Vec2>) -> Resolved {
    let free = layout.free.map(|a| clamp_into(a.rect(screen), screen));
    let mut rects: BTreeMap<PanelId, Rect> = BTreeMap::new();
    for p in PanelId::ALL {
        let place = layout.place(p);
        let Some(size) = sizes.get(&p) else { continue };
        if !place.visible {
            continue;
        }
        let mut r = clamp_into(place.rect(screen, *size), screen);
        let blocks: Vec<Rect> = free.into_iter().chain(rects.values().copied()).collect();
        if let Some(c) = clear_place(r, &blocks, screen, p.min_height()) {
            r = c;
        }
        rects.insert(p, r);
    }
    Resolved { rects, free }
}

/// What a resolution depends on; the rectangles are worked out again only when it changes.
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub screen: Rect,
    pub generation: u64,
    pub sizes: Vec<(PanelId, [i32; 2])>,
}

impl Key {
    pub fn new(screen: Rect, generation: u64, sizes: &BTreeMap<PanelId, Vec2>) -> Self {
        Self {
            screen,
            generation,
            sizes: sizes
                .iter()
                .map(|(p, s)| (*p, [s.x.round() as i32, s.y.round() as i32]))
                .collect(),
        }
    }
}

/// The player's layout and the rectangles last resolved from it.
#[derive(Clone, Debug, Default)]
pub struct Hud {
    pub layout: Layout,
    /// Bumped at every change of the layout.
    pub generation: u64,
    cache: Option<(Key, Resolved)>,
}

impl Hud {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            generation: 0,
            cache: None,
        }
    }

    /// The rectangles for this screen and these sizes, from the cache when nothing changed.
    pub fn resolved(&mut self, screen: Rect, sizes: &BTreeMap<PanelId, Vec2>) -> &Resolved {
        let key = Key::new(screen, self.generation, sizes);
        let fresh = self.cache.as_ref().is_none_or(|(k, _)| *k != key);
        if fresh {
            let r = resolve(&self.layout, screen, sizes);
            self.cache = Some((key, r));
        }
        &self.cache.as_ref().expect("resolved").1
    }

    pub fn changed(&mut self) {
        self.generation += 1;
    }
}

// ---------------------------------------------------------------------------------------------
// Actions. Each one is a function, so that a command can call it.

/// Shows or hides a panel.
pub fn toggle_panel(hud: &mut Hud, p: PanelId) {
    let mut place = hud.layout.place(p);
    place.visible = !place.visible;
    hud.layout.set_place(p, place);
    hud.changed();
}

/// Replaces the layout by a preset; the windows keep their places.
pub fn apply_preset(hud: &mut Hud, preset: Preset) {
    let windows = std::mem::take(&mut hud.layout.windows);
    hud.layout = Layout::preset(preset);
    hud.layout.windows = windows;
    hud.changed();
}

/// Back to the standard layout, windows included.
pub fn reset(hud: &mut Hud) {
    hud.layout = Layout::default();
    hud.changed();
}

pub fn set_free_area(hud: &mut Hud, area: Option<Area>) {
    hud.layout.free = area;
    hud.layout.preset = Preset::Custom;
    hud.changed();
}

/// Writes the layout to the export file.
pub fn export(hud: &Hud, config: &Path) -> std::io::Result<PathBuf> {
    let path = export_file(config);
    hud.layout.save(&path)?;
    Ok(path)
}

/// Reads the layout from the export file.
pub fn import(hud: &mut Hud, config: &Path) -> Result<PathBuf, String> {
    let path = export_file(config);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    hud.layout = Layout::parse(&text)?;
    hud.changed();
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::ZERO, vec2(w, h))
    }

    /// The panels' sizes on a screen `w` wide (the time controls narrow on a small screen).
    fn sizes_on(w: f32) -> BTreeMap<PanelId, Vec2> {
        [
            (PanelId::Country, vec2(794.0, 118.0)),
            (PanelId::Time, vec2((w - 830.0).min(522.0), 56.0)),
            (PanelId::Outliner, vec2(300.0, 420.0)),
            (PanelId::MapModes, vec2(400.0, 122.0)),
            (PanelId::CountryPanel, vec2(380.0, 800.0)),
        ]
        .into_iter()
        .collect()
    }

    fn sizes() -> BTreeMap<PanelId, Vec2> {
        sizes_on(1600.0)
    }

    #[test]
    fn an_anchored_place_follows_its_corner() {
        let p = Placement::at(Anchor::BottomRight, -12.0, -12.0);
        let r = p.rect(screen(1600.0, 900.0), vec2(100.0, 50.0));
        assert_eq!(r.max, pos2(1588.0, 888.0));
        let r = p.rect(screen(1280.0, 720.0), vec2(100.0, 50.0));
        assert_eq!(r.max, pos2(1268.0, 708.0));
        // Back and forth through a rectangle.
        let back = Placement::of_rect(r, screen(1280.0, 720.0), true);
        assert_eq!(back, p);
    }

    #[test]
    fn the_standard_layout_matches_the_old_hud() {
        let r = resolve(&Layout::default(), screen(1600.0, 900.0), &sizes());
        assert_eq!(r.rect(PanelId::Country).unwrap().min, pos2(12.0, 12.0));
        assert_eq!(r.rect(PanelId::Time).unwrap().right(), 1588.0);
        let o = r.rect(PanelId::Outliner).unwrap();
        assert_eq!(o.min, pos2(12.0, crate::screens::top_bar::LEFT_BOTTOM));
        // The overview stops above the map modes.
        let m = r.rect(PanelId::MapModes).unwrap();
        assert!(o.bottom() <= m.top() - GAP + 0.01, "{o:?} {m:?}");
    }

    #[test]
    fn no_panel_leaves_the_screen_or_covers_another_after_a_resize() {
        let mut layout = Layout::default();
        // Dragged to the far right of a large screen.
        layout.set_place(
            PanelId::MapModes,
            Placement::of_rect(
                Rect::from_min_size(pos2(1500.0, 400.0), vec2(400.0, 122.0)),
                screen(1920.0, 1080.0),
                true,
            ),
        );
        for (w, h) in [(1600.0, 900.0), (1280.0, 720.0)] {
            let s = screen(w, h);
            let r = resolve(&layout, s, &sizes_on(w));
            for (p, a) in &r.rects {
                assert!(s.contains_rect(*a), "{p:?} {a:?} off {w}×{h}");
            }
            let rects: Vec<_> = r.rects.iter().collect();
            for (i, (pa, a)) in rects.iter().enumerate() {
                for (pb, b) in &rects[i + 1..] {
                    assert!(
                        !overlaps(**a, **b),
                        "{pa:?} {a:?} over {pb:?} {b:?} at {w}×{h}"
                    );
                }
            }
        }
    }

    #[test]
    fn panels_keep_out_of_the_free_area() {
        let layout = Layout::preset(Preset::Streaming);
        for (w, h) in [(1600.0, 900.0), (1280.0, 720.0)] {
            let r = resolve(&layout, screen(w, h), &sizes_on(w));
            let free = r.free.expect("free area");
            for (p, a) in &r.rects {
                assert!(
                    !overlaps(*a, free),
                    "{p:?} {a:?} in the free area at {w}×{h}"
                );
            }
            // The country panel sits under the camera, shorter, above the time controls.
            let c = r.rect(PanelId::CountryPanel).unwrap();
            let t = r.rect(PanelId::Time).unwrap();
            assert!(c.right() > w - 30.0 && c.top() >= free.bottom(), "{c:?}");
            assert!(
                t.bottom() > h - 30.0 && c.bottom() <= t.top(),
                "{c:?} {t:?}"
            );
        }
    }

    #[test]
    fn an_old_file_gets_the_new_panels() {
        let text = r#"(version: 1, panels: {"time": (anchor: TopLeft, offset: (5.0, 6.0), visible: true), "gone": (anchor: Center, offset: (0.0, 0.0), visible: false)})"#;
        let l = Layout::parse(text).unwrap();
        assert_eq!(l.place(PanelId::Time).offset, [5.0, 6.0]);
        assert!(l.panels.contains_key("overview"));
        assert!(!l.panels.contains_key("gone"));
        let back = Layout::parse(&ron::to_string(&l).unwrap()).unwrap();
        assert_eq!(back, l);
        assert!(Layout::parse("(version: 99)").is_err());
    }

    #[test]
    fn resolution_is_cached_until_something_changes() {
        let mut hud = Hud::default();
        let s = screen(1600.0, 900.0);
        let a = hud.resolved(s, &sizes()).clone();
        assert_eq!(*hud.resolved(s, &sizes()), a);
        toggle_panel(&mut hud, PanelId::MapModes);
        assert!(hud.resolved(s, &sizes()).rect(PanelId::MapModes).is_none());
    }
}
