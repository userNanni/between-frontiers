//! Keyboard: every action of the interface is a command in one registry; keys, buttons and key
//! tips all trigger commands.
//!
//! - [`commands`]: the registry and the built-in commands. Other modules add theirs with
//!   [`Keys::register`]; see the example at the top of that module.
//! - [`keymap`]: the player's keys, profiles, conflicts and the file they are saved in.
//! - [`physical`]: keys by position, and the ABNT2 and ANSI boards.
//! - [`hints`]: key tips (leader key, then letters).
//!
//! Screens never read a key themselves. Per frame:
//!
//! - [`fired`]: was the command's key pressed this frame;
//! - [`held`]: is it down (map panning);
//! - [`pressed`]: a button's click, or its key tip typed; registers the tip;
//! - [`target`] / [`item`]: register a tip for an element that is not a plain button;
//! - [`cover`]: a window drawn over earlier elements hides their tips;
//! - [`tip`]: a tooltip text with the command's current key.

pub mod commands;
pub mod hints;
pub mod keymap;
pub mod leader;
pub mod physical;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use bf_i18n::I18n;
use egui::{Context, Id, Rect, Response, Ui};

pub use commands::{Category, Chord, CommandDef, Mods, Registry, id};
pub use keymap::{KeyConfig, Leader};
pub use physical::{Layout, PhysKey};

pub use hints::Side;
use hints::{Spec, Target, Typed};

/// What the keyboard did this frame, for the screens. Lives in the egui context so any screen
/// reaches it without threading it through every call.
#[derive(Clone, Default)]
struct Frame {
    fired: Vec<&'static str>,
    held: Vec<&'static str>,
    /// Target whose key tip was typed last frame.
    activate: Option<String>,
    hint_mode: bool,
    static_info: Arc<StaticInfo>,
}

/// Command letters and key names; rebuilt only when the keys or the language change.
#[derive(Default)]
struct StaticInfo {
    labels: HashMap<&'static str, String>,
    letters: HashMap<&'static str, char>,
}

fn frame_id() -> Id {
    Id::unique("bf-keys-frame")
}

fn targets_id() -> Id {
    Id::unique("bf-keys-targets")
}

fn frame(ctx: &Context) -> Frame {
    ctx.data(|d| d.get_temp::<Frame>(frame_id()))
        .unwrap_or_default()
}

/// The command's key was pressed this frame.
pub fn fired(ctx: &Context, cmd: &str) -> bool {
    ctx.data(|d| {
        d.get_temp::<Frame>(frame_id())
            .is_some_and(|f| f.fired.contains(&cmd))
    })
}

/// The command's key is down.
pub fn held(ctx: &Context, cmd: &str) -> bool {
    ctx.data(|d| {
        d.get_temp::<Frame>(frame_id())
            .is_some_and(|f| f.held.contains(&cmd))
    })
}

/// Key tips are showing.
pub fn hint_mode(ctx: &Context) -> bool {
    frame(ctx).hint_mode
}

/// The command's first key, as printed on the player's board.
pub fn shortcut(ctx: &Context, cmd: &str) -> Option<String> {
    frame(ctx).static_info.labels.get(cmd).cloned()
}

/// `text (key)`, or `text` when the command has no key.
pub fn tip(ctx: &Context, text: &str, cmd: &str) -> String {
    match shortcut(ctx, cmd) {
        Some(k) => format!("{text} ({k})"),
        None => text.to_owned(),
    }
}

/// A button for `cmd` was clicked or its key tip typed. Registers the tip.
pub fn pressed(ui: &Ui, cmd: &'static str, response: &Response) -> bool {
    let tipped = response.enabled() && target(ui, cmd, response.rect);
    response.clicked() || tipped
}

/// Registers a key tip for a command's element; true when it was typed.
pub fn target(ui: &Ui, cmd: &'static str, rect: Rect) -> bool {
    let f = frame(ui.ctx());
    let letter = f
        .static_info
        .letters
        .get(cmd)
        .copied()
        .unwrap_or_else(|| cmd.chars().next().unwrap_or('A'));
    register(ui, &f, cmd.to_owned(), Spec::Command(letter), rect)
}

/// Registers a key tip for an element that is not a command (a close button); true when typed.
pub fn single(ui: &Ui, id: &str, letter: char, rect: Rect) -> bool {
    let f = frame(ui.ctx());
    register(ui, &f, id.to_owned(), Spec::Single(letter), rect)
}

/// Registers a key tip for a list item: `family` is the first letter, the second comes from
/// `key`. `id` must be stable (`laws.group.taxation`). True when typed.
pub fn item(ui: &Ui, family: char, id: String, key: &str, rect: Rect) -> bool {
    item_beside(ui, family, id, key, rect, Side::Auto)
}

/// [`item`] whose label would rather sit on one side (the − and + of a stepper).
pub fn item_beside(ui: &Ui, family: char, id: String, key: &str, rect: Rect, side: Side) -> bool {
    let f = frame(ui.ctx());
    let spec = Spec::Item {
        family,
        key: key.to_owned(),
    };
    register_at(ui, &f, id, spec, rect, side)
}

fn register(ui: &Ui, f: &Frame, id: String, spec: Spec, rect: Rect) -> bool {
    register_at(ui, f, id, spec, rect, Side::Auto)
}

fn register_at(ui: &Ui, f: &Frame, id: String, spec: Spec, rect: Rect, side: Side) -> bool {
    let hit = f.activate.as_deref() == Some(id.as_str());
    if f.hint_mode && ui.is_visible() && ui.clip_rect().intersects(rect) {
        let rect = rect.intersect(ui.clip_rect());
        ui.ctx().data_mut(|d| {
            d.get_temp_mut_or_default::<Vec<Target>>(targets_id())
                .push(Target {
                    id,
                    spec,
                    rect,
                    side,
                });
        });
    }
    hit
}

/// A window drawn over `rect` hides the tips registered before it there.
pub fn cover(ui: &Ui, rect: Rect) {
    if !hint_mode(ui.ctx()) {
        return;
    }
    ui.ctx().data_mut(|d| {
        d.get_temp_mut_or_default::<Vec<Target>>(targets_id())
            .retain(|t| !rect.contains(t.rect.center()));
    });
}

/// A segmented control whose options are commands or list items: the click, or a typed tip,
/// picks the option.
pub fn segmented<T: PartialEq + Clone>(
    ui: &mut Ui,
    value: &mut T,
    options: &[(T, String)],
    tips: &[Tip<'_>],
) -> bool {
    let (mut changed, rects) = bf_ui_kit::segmented_rects(ui, value, options);
    for (i, rect) in rects.into_iter().enumerate() {
        let typed = match tips.get(i) {
            Some(Tip::Command(cmd)) => target(ui, cmd, rect),
            Some(Tip::Item(family, id, key)) => item(ui, *family, id.clone(), key, rect),
            None => false,
        };
        if typed && *value != options[i].0 {
            *value = options[i].0.clone();
            changed = true;
        }
    }
    changed
}

/// Key tip of a segmented option.
pub enum Tip<'a> {
    Command(&'static str),
    Item(char, String, &'a str),
}

/// The keyboard's state across frames: the registry, the player's keys and what the leader key
/// and key tips are doing.
pub struct Keys {
    registry: Registry,
    config: KeyConfig,
    resolved: keymap::Resolved,
    path: Option<PathBuf>,
    /// Bumped when keys change, so names are rebuilt.
    generation: u64,
    info: Arc<StaticInfo>,
    info_for: Option<(u64, String)>,
    /// Physical keys down.
    down: HashSet<PhysKey>,
    tap: leader::LeaderTap,
    hint_mode: bool,
    typed: String,
    codes: BTreeMap<String, String>,
    activate: Option<String>,
    /// The key editor is listening for a key: nothing fires.
    pub capturing: bool,
}

impl Keys {
    /// Built-in commands and the player's key file (`None`: defaults, nothing saved).
    pub fn load(path: Option<PathBuf>) -> (Self, Vec<String>) {
        let registry = Registry::builtin();
        let (config, notes) = match &path {
            Some(p) => KeyConfig::load(p, &registry),
            None => {
                let mut c = KeyConfig::default();
                c.migrate(&registry);
                (c, Vec::new())
            }
        };
        let resolved = config.resolve(&registry);
        (
            Self {
                registry,
                config,
                resolved,
                path,
                generation: 0,
                info: Arc::default(),
                info_for: None,
                down: HashSet::new(),
                tap: leader::LeaderTap::default(),
                hint_mode: false,
                typed: String::new(),
                codes: BTreeMap::new(),
                activate: None,
                capturing: false,
            },
            notes,
        )
    }

    /// Adds a command (see [`commands`]). It gets its default keys unless the player already
    /// uses one of them for something else.
    pub fn register(&mut self, def: CommandDef) -> Result<(), String> {
        self.registry.register(def)?;
        self.config.migrate(&self.registry);
        self.changed();
        Ok(())
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn config(&self) -> &KeyConfig {
        &self.config
    }

    /// Changes the player's keys and saves them.
    pub fn edit<R>(&mut self, f: impl FnOnce(&mut KeyConfig, &Registry) -> R) -> R {
        let out = f(&mut self.config, &self.registry);
        self.changed();
        out
    }

    pub fn keys_of(&self, cmd: &str) -> &[Chord] {
        self.resolved.keys(cmd)
    }

    pub fn holder(&self, chord: Chord) -> Option<&'static str> {
        self.resolved.holder(chord)
    }

    fn changed(&mut self) {
        self.resolved = self.config.resolve(&self.registry);
        self.generation += 1;
        if let Some(p) = &self.path
            && let Err(e) = self.config.save(p)
        {
            eprintln!("between-frontiers: keys: {e}");
        }
    }

    pub fn hint_mode(&self) -> bool {
        self.hint_mode
    }

    pub fn set_hint_mode(&mut self, on: bool) {
        self.hint_mode = on;
        self.typed.clear();
    }

    /// Reads this frame's keys; call before the screens draw.
    pub fn begin_frame(&mut self, ctx: &Context, i18n: &I18n) {
        let key = (self.generation, i18n.locale().code().to_owned());
        if self.info_for.as_ref() != Some(&key) {
            self.info = Arc::new(self.static_info(i18n));
            self.info_for = Some(key);
        }
        let (events, modifiers, focused) =
            ctx.input(|i| (i.events.clone(), i.modifiers, i.focused));
        if !focused {
            self.down.clear();
        }
        let typing = ctx.egui_wants_keyboard_input() || self.capturing;
        let mut fired = Vec::new();
        let mut leader = false;
        for e in &events {
            match e {
                egui::Event::Key {
                    key,
                    physical_key,
                    pressed,
                    repeat,
                    modifiers,
                } => {
                    let phys = physical_key
                        .and_then(PhysKey::from_egui)
                        .or_else(|| PhysKey::from_egui(*key));
                    let Some(phys) = phys else { continue };
                    if *pressed {
                        self.down.insert(phys);
                    } else {
                        self.down.remove(&phys);
                    }
                    // The leader tapped alone toggles key tips; never while typing.
                    let lead = leader::leader_key(self.config.leader);
                    if self.tap.key(self.config.leader, phys, *pressed, *repeat) && !typing {
                        leader = true;
                    }
                    if phys == lead || phys.is_modifier() || !*pressed || typing {
                        continue;
                    }
                    if self.hint_mode {
                        self.hint_key(*key);
                        continue;
                    }
                    if *repeat {
                        continue;
                    }
                    let chord = Chord::with_mods(phys, self.mods(*modifiers));
                    if let Some(cmd) = self.resolved.holder(chord)
                        && self.registry.get(cmd).is_some_and(|d| !d.held)
                    {
                        fired.push(cmd);
                    }
                }
                egui::Event::PointerButton { pressed: true, .. } => {
                    self.tap.pointer();
                    if self.hint_mode {
                        self.set_hint_mode(false);
                    }
                }
                _ => {}
            }
        }
        // Platforms that report the left Alt only as a modifier.
        if self.tap.modifiers(self.config.leader, modifiers.alt) && !typing {
            leader = true;
        }
        if leader {
            let on = !self.hint_mode;
            self.set_hint_mode(on);
        }
        let mut held = Vec::new();
        if !typing && !self.hint_mode {
            let mods = self.mods(modifiers);
            for def in self.registry.all().iter().filter(|d| d.held) {
                let down = self.resolved.keys(def.id).iter().any(|c| {
                    self.down.contains(&c.key) && (!c.ctrl || mods.ctrl) && (!c.alt || mods.alt)
                });
                if down {
                    held.push(def.id);
                }
            }
        }
        let f = Frame {
            fired,
            held,
            activate: self.activate.take(),
            hint_mode: self.hint_mode,
            static_info: Arc::clone(&self.info),
        };
        if f.activate.is_some() || self.hint_mode {
            ctx.request_repaint();
        }
        ctx.data_mut(|d| {
            d.insert_temp(frame_id(), f);
            d.insert_temp::<Vec<Target>>(targets_id(), Vec::new());
        });
    }

    /// The modifiers a shortcut sees: AltGr is never Alt.
    fn mods(&self, m: egui::Modifiers) -> Mods {
        leader::shortcut_mods(
            m,
            self.down.contains(&PhysKey::AltRight),
            self.down.contains(&PhysKey::AltLeft),
        )
    }

    fn hint_key(&mut self, key: egui::Key) {
        match key {
            egui::Key::Escape => self.set_hint_mode(false),
            egui::Key::Backspace => {
                self.typed.pop();
            }
            _ => {
                let name = key.name();
                if name.len() == 1 {
                    self.typed.push_str(&name.to_ascii_uppercase());
                } else if let Some(d) = name.strip_prefix("Num").filter(|d| d.len() == 1) {
                    self.typed.push_str(d);
                }
                self.resolve_typed();
            }
        }
    }

    fn resolve_typed(&mut self) {
        match hints::match_typed(&self.codes, &self.typed) {
            Typed::Hit(id) => {
                self.activate = Some(id);
                self.typed.clear();
            }
            Typed::Partial => {}
            Typed::Miss => self.typed.clear(),
        }
    }

    /// Gives this frame's elements their codes and paints them; call after the screens drew.
    pub fn end_frame(&mut self, ctx: &Context, i18n: &I18n, screen: Rect) {
        let targets = ctx
            .data_mut(|d| d.remove_temp::<Vec<Target>>(targets_id()))
            .unwrap_or_default();
        if !self.hint_mode {
            self.codes.clear();
            return;
        }
        self.codes = hints::assign(&targets);
        paint(ctx, i18n, &targets, &self.codes, &self.typed, screen);
    }

    fn static_info(&self, i18n: &I18n) -> StaticInfo {
        let mut info = StaticInfo::default();
        for def in self.registry.all() {
            if let Some(c) = def.hint {
                info.letters.insert(def.id, c);
            }
            if let Some(chord) = self.resolved.keys(def.id).first() {
                info.labels
                    .insert(def.id, chord_label(*chord, self.config.layout, i18n));
            }
        }
        info
    }
}

/// `Ctrl+Shift+Ç`, in the player's layout and language.
pub fn chord_label(chord: Chord, layout: Layout, i18n: &I18n) -> String {
    let mut out = String::new();
    if chord.ctrl {
        out.push_str("Ctrl+");
    }
    if chord.shift {
        out.push_str("Shift+");
    }
    if chord.alt {
        out.push_str("Alt+");
    }
    out.push_str(&key_label(chord.key, layout, i18n));
    out
}

/// The name of a key as printed on it.
pub fn key_label(key: PhysKey, layout: Layout, i18n: &I18n) -> String {
    if let Some((base, _)) = layout.legend(key) {
        return base.to_owned();
    }
    use PhysKey::*;
    match key {
        Space => i18n.tr("key-space"),
        Enter => "Enter".into(),
        Backspace => "Bksp".into(),
        Tab => "Tab".into(),
        CapsLock => "Caps".into(),
        ShiftLeft | ShiftRight => "Shift".into(),
        ControlLeft | ControlRight => "Ctrl".into(),
        AltLeft => "Alt".into(),
        AltRight => "AltGr".into(),
        SuperLeft | SuperRight => "Super".into(),
        ContextMenu => "Menu".into(),
        Insert => "Ins".into(),
        Delete => "Del".into(),
        Home => "Home".into(),
        End => "End".into(),
        PageUp => "PgUp".into(),
        PageDown => "PgDn".into(),
        ArrowUp => "↑".into(),
        ArrowDown => "↓".into(),
        ArrowLeft => "←".into(),
        ArrowRight => "→".into(),
        other => other.code().to_owned(),
    }
}

fn paint(
    ctx: &Context,
    i18n: &I18n,
    targets: &[Target],
    codes: &BTreeMap<String, String>,
    typed: &str,
    screen: Rect,
) {
    use bf_ui_kit::theme::{sans, sans_semibold};
    use bf_ui_kit::tokens::{color, radius};
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        Id::unique("bf-key-tips"),
    ));
    // One label per code, in id order so the layout is the same every frame.
    let mut seen = HashSet::new();
    let mut shown: Vec<(&Target, &String)> = targets
        .iter()
        .filter_map(|t| codes.get(&t.id).map(|c| (t, c)))
        .filter(|(t, c)| c.starts_with(typed) && seen.insert(&t.id))
        .collect();
    shown.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    let sizes: Vec<(Rect, egui::Vec2, Side)> = shown
        .iter()
        .map(|(t, code)| {
            let g = painter.layout_no_wrap((*code).clone(), sans_semibold(12.0), color::ON_ACCENT);
            (
                t.rect,
                egui::vec2((g.size().x + 8.0).max(18.0), 18.0),
                t.side,
            )
        })
        .collect();
    let places = hints::place_labels(&sizes, screen);
    for ((_, code), r) in shown.iter().zip(places) {
        painter.rect(
            r.translate(egui::vec2(0.0, 1.5)),
            radius::SM,
            color::SHADOW,
            egui::Stroke::NONE,
            egui::StrokeKind::Inside,
        );
        painter.rect(
            r,
            radius::SM,
            color::ACCENT,
            egui::Stroke::new(1.0, color::ON_ACCENT.gamma_multiply(0.5)),
            egui::StrokeKind::Inside,
        );
        // Typed characters fade so the next one to type stands out.
        let mut job = egui::text::LayoutJob::default();
        let done = typed.len().min(code.len());
        job.append(
            &code[..done],
            0.0,
            egui::TextFormat::simple(sans_semibold(12.0), color::ON_ACCENT.gamma_multiply(0.45)),
        );
        job.append(
            &code[done..],
            0.0,
            egui::TextFormat::simple(sans_semibold(12.0), color::ON_ACCENT),
        );
        let g = painter.layout_job(job);
        painter.galley(r.center() - g.size() / 2.0, g, color::ON_ACCENT);
    }
    // What the mode is, at the bottom of the screen.
    let text = if typed.is_empty() {
        i18n.tr("keys-tips-banner")
    } else {
        i18n.tr_args(
            "keys-tips-typed",
            &[("typed", bf_i18n::Arg::String(typed.to_owned()))],
        )
    };
    let galley = painter.layout_no_wrap(text, sans(13.0), color::TEXT);
    let r = Rect::from_center_size(
        egui::pos2(screen.center().x, screen.bottom() - 28.0),
        galley.size() + egui::vec2(28.0, 14.0),
    );
    painter.rect(
        r,
        radius::MD,
        color::SURFACE_RAISED,
        egui::Stroke::new(1.0, color::ACCENT),
        egui::StrokeKind::Inside,
    );
    painter.galley(r.center() - galley.size() / 2.0, galley, color::TEXT);
}
