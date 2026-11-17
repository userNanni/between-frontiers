//! The command registry: every action of the interface that a key, a button or a key tip can
//! trigger, with a stable id, a category, a name and its default keys.
//!
//! # Registering a command (for other modules)
//!
//! ```ignore
//! shared.keys.register(CommandDef::new("hud.toggle_outliner", Category::Windows)
//!     .keys([Chord::key(PhysKey::KeyO).shift()])
//!     .hint('O'));
//! // then, where the panel is drawn:
//! if keys::fired(ui.ctx(), "hud.toggle_outliner") || keys::pressed(ui, "hud.toggle_outliner", &button) { … }
//! ```
//!
//! - The id is stable: it is what the player's key file stores. Never rename one; add a new id.
//! - The name is the Fluent message `cmd-<id with dots and underscores as dashes>`, in `en` and
//!   `pt-BR` (`cmd-hud-toggle-outliner`).
//! - Registering after the key file loaded is fine: the command gets its default keys, unless a
//!   key the player already uses for something else is among them; that one is left free.

use super::physical::PhysKey;
use serde::{Deserialize, Serialize};

/// A key with the modifiers held with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Chord {
    pub key: PhysKey,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ctrl: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub shift: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub alt: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Chord {
    pub const fn key(key: PhysKey) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    pub const fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub const fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub const fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_mods(key: PhysKey, mods: Mods) -> Self {
        Self {
            key,
            ctrl: mods.ctrl,
            shift: mods.shift,
            alt: mods.alt,
        }
    }

    pub fn mods(self) -> Mods {
        Mods {
            ctrl: self.ctrl,
            shift: self.shift,
            alt: self.alt,
        }
    }
}

/// A modifier layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mods {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Mods {
    pub fn from_egui(m: egui::Modifiers) -> Self {
        Self {
            ctrl: m.ctrl || m.command,
            shift: m.shift,
            alt: m.alt,
        }
    }
}

/// What a command is about; the visual keyboard colours keys by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
    Time,
    Map,
    Government,
    Windows,
    Research,
    System,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Category::Time,
        Category::Map,
        Category::Government,
        Category::Windows,
        Category::Research,
        Category::System,
    ];

    /// Fluent message with the category's name.
    pub fn name_key(self) -> &'static str {
        match self {
            Category::Time => "keys-cat-time",
            Category::Map => "keys-cat-map",
            Category::Government => "keys-cat-government",
            Category::Windows => "keys-cat-windows",
            Category::Research => "keys-cat-research",
            Category::System => "keys-cat-system",
        }
    }
}

/// One action of the interface.
#[derive(Clone, Debug)]
pub struct CommandDef {
    /// Stable id, `area.action`.
    pub id: &'static str,
    pub category: Category,
    /// Keys in the standard profile.
    pub defaults: Vec<Chord>,
    /// Letter of the command's key tip. Derived from the command, so the same element reads the
    /// same letter in every session; another letter is taken only if two elements on one screen
    /// ask for the same one.
    pub hint: Option<char>,
    /// Acts for as long as the key is down (map panning) instead of once per press.
    pub held: bool,
}

impl CommandDef {
    pub fn new(id: &'static str, category: Category) -> Self {
        Self {
            id,
            category,
            defaults: Vec::new(),
            hint: None,
            held: false,
        }
    }

    pub fn keys(mut self, keys: impl IntoIterator<Item = Chord>) -> Self {
        self.defaults = keys.into_iter().collect();
        self
    }

    pub fn hint(mut self, letter: char) -> Self {
        self.hint = Some(letter.to_ascii_uppercase());
        self
    }

    pub fn held(mut self) -> Self {
        self.held = true;
        self
    }

    /// Fluent message with the command's name.
    pub fn name_key(&self) -> String {
        name_key(self.id)
    }
}

pub fn name_key(id: &str) -> String {
    format!("cmd-{}", id.replace(['.', '_'], "-"))
}

/// All commands, in the order the editor lists them.
#[derive(Clone, Debug, Default)]
pub struct Registry {
    commands: Vec<CommandDef>,
}

impl Registry {
    /// The game's own commands.
    pub fn builtin() -> Self {
        let mut r = Self::default();
        for def in builtin() {
            r.register(def).expect("built-in ids are unique");
        }
        r
    }

    /// Adds a command. Fails when the id is taken.
    pub fn register(&mut self, def: CommandDef) -> Result<(), String> {
        if self.get(def.id).is_some() {
            return Err(format!("command {} registered twice", def.id));
        }
        self.commands.push(def);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&CommandDef> {
        self.commands.iter().find(|c| c.id == id)
    }

    pub fn all(&self) -> &[CommandDef] {
        &self.commands
    }
}

/// Ids of the built-in commands, so call sites do not repeat string literals.
pub mod id {
    pub const PAUSE: &str = "time.pause";
    pub const SPEED_DOWN: &str = "time.speed_down";
    pub const SPEED_UP: &str = "time.speed_up";
    pub const SPEEDS: [&str; 5] = [
        "time.speed_1",
        "time.speed_2",
        "time.speed_3",
        "time.speed_4",
        "time.speed_5",
    ];
    pub const PAN_LEFT: &str = "map.pan_left";
    pub const PAN_RIGHT: &str = "map.pan_right";
    pub const PAN_UP: &str = "map.pan_up";
    pub const PAN_DOWN: &str = "map.pan_down";
    pub const ZOOM_IN: &str = "map.zoom_in";
    pub const ZOOM_OUT: &str = "map.zoom_out";
    pub const MODE_POLITICAL: &str = "map.mode_political";
    pub const MODE_RESOURCES: &str = "map.mode_resources";
    pub const MODE_CLIMATE: &str = "map.mode_climate";
    pub const MODE_FARMS: &str = "map.mode_farms";
    pub const MODE_INFRASTRUCTURE: &str = "map.mode_infrastructure";
    pub const MODE_SEA: &str = "map.mode_sea";
    pub const GOVERNMENT: &str = "window.government";
    pub const RESEARCH: &str = "window.research";
    pub const COUNTRY: &str = "window.country";
    pub const MENU: &str = "window.menu";
    pub const JOURNAL: &str = "window.journal";
    pub const OUTLINER: &str = "outliner.toggle";
    pub const OUTLINER_WORKS: &str = "outliner.fold_works";
    pub const OUTLINER_RESEARCH: &str = "outliner.fold_research";
    pub const OUTLINER_PINNED: &str = "outliner.fold_pinned";
    pub const JOURNAL_PIN: &str = "journal.pin";
    pub const JOURNAL_GO: &str = "journal.open_screen";
    pub const HUD_EDIT: &str = "hud.edit";
    /// Show or hide each HUD panel, in the order of `hud::PanelId::ALL`.
    pub const HUD_PANELS: [&str; 6] = [
        "hud.toggle_country",
        "hud.toggle_time",
        "hud.toggle_map_modes",
        "hud.toggle_country_panel",
        "hud.toggle_outliner",
        "hud.toggle_population",
    ];
    /// HUD presets, in the order of `hud::Preset::CHOICES`.
    pub const HUD_PRESETS: [&str; 3] = [
        "hud.preset_standard",
        "hud.preset_compact",
        "hud.preset_streaming",
    ];
    pub const HUD_FREE_MARK: &str = "hud.free_mark";
    pub const HUD_FREE_CLEAR: &str = "hud.free_clear";
    pub const HUD_EDIT_BLOCKS: &str = "hud.edit_blocks";
    pub const HUD_EDIT_SECTIONS: &str = "hud.edit_sections";
    pub const HUD_RESET_BLOCKS: &str = "hud.reset_blocks";
    pub const LENS_PICKER: &str = "map.lens_picker";
    pub const LENS_KEEP_OPEN: &str = "map.lens_keep_open";
    /// The map modes, in the order of the picker.
    pub const MODES: [&str; 6] = [
        MODE_POLITICAL,
        MODE_RESOURCES,
        MODE_CLIMATE,
        MODE_FARMS,
        MODE_INFRASTRUCTURE,
        MODE_SEA,
    ];
    pub const TAB_BUDGET: &str = "gov.tab_budget";
    pub const TAB_MONEY: &str = "gov.tab_money";
    pub const TAB_LAWS: &str = "gov.tab_laws";
    pub const PROPOSE_LAW: &str = "gov.propose_law";
    pub const RESEARCH_FIT: &str = "research.fit";
    pub const RESEARCH_PREV_ERA: &str = "research.prev_era";
    pub const RESEARCH_NEXT_ERA: &str = "research.next_era";
    pub const RESEARCH_CLEAR_FOCUS: &str = "research.clear_focus";
    pub const BACK: &str = "system.back";
    pub const SAVE: &str = "system.save";
    pub const SETTINGS: &str = "system.settings";
    pub const KEYS: &str = "system.keys";
    pub const POPULATION: &str = "window.population";
    pub const UI_SCALE_UP: &str = "system.ui_scale_up";
    pub const UI_SCALE_DOWN: &str = "system.ui_scale_down";
    pub const UI_STREAM: &str = "system.ui_stream";
    pub const HUD_CENTER_BAND: &str = "hud.center_band";
}

fn builtin() -> Vec<CommandDef> {
    use Category::*;
    use PhysKey::*;
    use id::*;
    let k = Chord::key;
    let mut out = vec![
        CommandDef::new(PAUSE, Time).keys([k(Space)]).hint('P'),
        CommandDef::new(SPEED_DOWN, Time).keys([k(Minus)]).hint('D'),
        CommandDef::new(SPEED_UP, Time).keys([k(Equal)]).hint('U'),
    ];
    for (i, digit) in [Digit1, Digit2, Digit3, Digit4, Digit5]
        .into_iter()
        .enumerate()
    {
        out.push(CommandDef::new(SPEEDS[i], Time).keys([k(digit)]));
    }
    out.extend([
        CommandDef::new(PAN_LEFT, Map)
            .keys([k(KeyA), k(ArrowLeft)])
            .held(),
        CommandDef::new(PAN_RIGHT, Map)
            .keys([k(KeyD), k(ArrowRight)])
            .held(),
        CommandDef::new(PAN_UP, Map)
            .keys([k(KeyW), k(ArrowUp)])
            .held(),
        CommandDef::new(PAN_DOWN, Map)
            .keys([k(KeyS), k(ArrowDown)])
            .held(),
        CommandDef::new(ZOOM_IN, Map).keys([k(PageUp)]).held(),
        CommandDef::new(ZOOM_OUT, Map).keys([k(PageDown)]).held(),
    ]);
    for (i, (cmd, digit)) in [
        (MODE_POLITICAL, Digit1),
        (MODE_RESOURCES, Digit2),
        (MODE_CLIMATE, Digit3),
        (MODE_FARMS, Digit4),
        (MODE_INFRASTRUCTURE, Digit5),
        (MODE_SEA, Digit6),
    ]
    .into_iter()
    .enumerate()
    {
        // The sea lens takes 9: 6 to 8 are the overview's sections, often on screen with it.
        let hint = if i == 5 {
            '9'
        } else {
            (b'1' + i as u8) as char
        };
        out.push(
            CommandDef::new(cmd, Map)
                .keys([k(digit).shift()])
                .hint(hint),
        );
    }
    out.extend([
        CommandDef::new(GOVERNMENT, Windows)
            .keys([k(KeyG)])
            .hint('G'),
        CommandDef::new(RESEARCH, Windows).keys([k(KeyR)]).hint('R'),
        CommandDef::new(COUNTRY, Windows).keys([k(KeyC)]).hint('C'),
        CommandDef::new(MENU, Windows).keys([k(F10)]).hint('N'),
        CommandDef::new(JOURNAL, Windows).keys([k(KeyJ)]).hint('J'),
        CommandDef::new(POPULATION, Windows).keys([k(KeyY)]),
        CommandDef::new(OUTLINER, Windows).keys([k(KeyO)]).hint('O'),
        CommandDef::new(OUTLINER_WORKS, Windows).hint('6'),
        CommandDef::new(OUTLINER_RESEARCH, Windows).hint('7'),
        CommandDef::new(OUTLINER_PINNED, Windows).hint('8'),
        CommandDef::new(JOURNAL_PIN, Windows)
            .keys([k(KeyP).ctrl()])
            .hint('I'),
        CommandDef::new(JOURNAL_GO, Windows)
            .keys([k(Enter)])
            .hint('A'),
        CommandDef::new(HUD_EDIT, Windows).keys([k(F8)]).hint('H'),
        CommandDef::new(TAB_BUDGET, Government)
            .keys([k(KeyB)])
            .hint('B'),
        CommandDef::new(TAB_MONEY, Government)
            .keys([k(KeyM)])
            .hint('M'),
        CommandDef::new(TAB_LAWS, Government)
            .keys([k(KeyL)])
            .hint('L'),
        CommandDef::new(PROPOSE_LAW, Government)
            .keys([k(Enter).ctrl()])
            .hint('E'),
        CommandDef::new(RESEARCH_FIT, Research)
            .keys([k(KeyF)])
            .hint('F'),
        CommandDef::new(RESEARCH_PREV_ERA, Research).keys([k(KeyQ)]),
        CommandDef::new(RESEARCH_NEXT_ERA, Research).keys([k(KeyE)]),
        CommandDef::new(RESEARCH_CLEAR_FOCUS, Research)
            .keys([k(Delete)])
            .hint('K'),
        CommandDef::new(BACK, System).keys([k(Escape)]),
        CommandDef::new(SAVE, System)
            .keys([k(KeyS).ctrl()])
            .hint('S'),
        CommandDef::new(SETTINGS, System)
            .keys([k(Comma).ctrl()])
            .hint('O'),
        CommandDef::new(KEYS, System)
            .keys([k(KeyK).ctrl()])
            .hint('K'),
    ]);
    for (i, (cmd, hint)) in HUD_PANELS
        .into_iter()
        .zip(['C', 'T', 'M', 'P', 'O', '0'])
        .enumerate()
    {
        let digit = [Digit1, Digit2, Digit3, Digit4, Digit5, Digit6][i];
        out.push(
            CommandDef::new(cmd, Windows)
                .keys([k(digit).ctrl()])
                .hint(hint),
        );
    }
    for (i, cmd) in HUD_PRESETS.into_iter().enumerate() {
        let digit = [Digit1, Digit2, Digit3][i];
        out.push(
            CommandDef::new(cmd, Windows)
                .keys([k(digit).ctrl().shift()])
                .hint((b'1' + i as u8) as char),
        );
    }
    out.push(CommandDef::new(HUD_FREE_MARK, Windows).hint('A'));
    out.push(CommandDef::new(HUD_FREE_CLEAR, Windows).hint('L'));
    out.push(
        CommandDef::new(HUD_EDIT_BLOCKS, Windows)
            .keys([k(F8).shift()])
            .hint('B'),
    );
    out.push(CommandDef::new(HUD_EDIT_SECTIONS, Windows).hint('S'));
    out.push(CommandDef::new(HUD_RESET_BLOCKS, Windows).hint('Z'));
    out.push(CommandDef::new(LENS_PICKER, Map).keys([k(KeyV)]).hint('V'));
    out.push(CommandDef::new(LENS_KEEP_OPEN, Map).hint('K'));
    out.push(CommandDef::new(UI_SCALE_UP, System).keys([k(Equal).ctrl()]));
    out.push(CommandDef::new(UI_SCALE_DOWN, System).keys([k(Minus).ctrl()]));
    out.push(CommandDef::new(UI_STREAM, System).hint('T'));
    out.push(CommandDef::new(HUD_CENTER_BAND, Windows));
    out
}
