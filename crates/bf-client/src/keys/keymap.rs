//! The player's keys: profiles, the file they live in, conflicts and migration.
//!
//! Stored per user in `keys.ron`, next to `settings.ron`; never in a campaign save, never in the
//! simulation. A profile keeps only what the player changed from its base, so a command added in
//! a later version arrives with its default keys; see [`KeyConfig::migrate`].

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::commands::{Chord, CommandDef, Registry, id};
use super::physical::{Layout, PhysKey};

/// Version of the file format this build writes.
pub const VERSION: u32 = 1;

/// The profiles the game ships. Each is a base the player's changes sit on.
pub const PROFILES: [&str; 2] = ["standard", "function_keys"];

/// Fluent message with a profile's name.
pub fn profile_name_key(profile: &str) -> String {
    format!("keys-profile-{}", profile.replace('_', "-"))
}

/// The key that turns key tips on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Leader {
    /// Alt pressed and released alone.
    #[default]
    Alt,
    Key(PhysKey),
}

/// What a profile changed from its base, and which commands it has seen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileFile {
    /// Commands this profile has already been reconciled with. A command missing here is new
    /// to the player: [`KeyConfig::migrate`] gives it its defaults without stealing a key.
    pub known: BTreeSet<String>,
    /// Keys the player set, by command id. An empty list leaves the command without a key.
    pub bindings: BTreeMap<String, Vec<Chord>>,
}

/// The whole key file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyConfig {
    /// Missing in a file means the oldest format.
    #[serde(default)]
    pub version: u32,
    pub layout: Layout,
    pub leader: Leader,
    /// Active profile.
    pub profile: String,
    pub profiles: BTreeMap<String, ProfileFile>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            version: VERSION,
            layout: Layout::Abnt2,
            leader: Leader::Alt,
            profile: PROFILES[0].to_owned(),
            profiles: BTreeMap::new(),
        }
    }
}

/// One profile exported to a file of its own.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Export {
    pub version: u32,
    pub profile: String,
    pub known: BTreeSet<String>,
    pub bindings: BTreeMap<String, Vec<Chord>>,
}

impl Default for Export {
    fn default() -> Self {
        Self {
            version: 0,
            profile: PROFILES[0].to_owned(),
            known: BTreeSet::new(),
            bindings: BTreeMap::new(),
        }
    }
}

/// The keys a profile gives a command before the player changes anything.
pub fn base_keys(profile: &str, def: &CommandDef) -> Vec<Chord> {
    if profile == "function_keys" {
        use PhysKey::*;
        let f = |k| vec![Chord::key(k)];
        match def.id {
            id::COUNTRY => return f(F1),
            // F1 went to the country: the sheet of keys moves to Shift.
            id::KEYS_SHEET => return vec![Chord::key(F1).shift()],
            id::GOVERNMENT => return f(F2),
            id::RESEARCH => return f(F3),
            id::TAB_BUDGET => return f(F5),
            id::TAB_MONEY => return f(F6),
            id::TAB_LAWS => return f(F7),
            id::RESEARCH_FIT => return f(F9),
            // F5 and F9 went to the tabs: the quick save and load move to Shift.
            id::QUICK_SAVE => return vec![Chord::key(F5).shift()],
            id::QUICK_LOAD => return vec![Chord::key(F9).shift()],
            _ => {}
        }
    }
    def.defaults.clone()
}

/// Bindings resolved for the active profile.
#[derive(Clone, Debug, Default)]
pub struct Resolved {
    pub by_command: HashMap<&'static str, Vec<Chord>>,
    /// First command holding each chord, in registry order.
    pub by_chord: HashMap<Chord, &'static str>,
}

impl Resolved {
    pub fn keys(&self, cmd: &str) -> &[Chord] {
        self.by_command.get(cmd).map_or(&[], Vec::as_slice)
    }

    /// Commands bound to `chord`.
    pub fn holder(&self, chord: Chord) -> Option<&'static str> {
        self.by_chord.get(&chord).copied()
    }
}

/// A key asked for by one command and already held by another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub chord: Chord,
    pub wanted_by: &'static str,
    pub held_by: &'static str,
}

impl KeyConfig {
    /// Reads the file; a missing file gives the defaults, a broken one too (and says so).
    pub fn load(path: &Path, registry: &Registry) -> (Self, Vec<String>) {
        let mut notes = Vec::new();
        let mut cfg = match std::fs::read_to_string(path) {
            Ok(text) => match Self::parse(&text) {
                Ok(c) => c,
                Err(e) => {
                    notes.push(format!("{}: {e}", path.display()));
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        };
        notes.extend(cfg.migrate(registry));
        (cfg, notes)
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let cfg: Self = ron::from_str(text).map_err(|e| e.to_string())?;
        if cfg.version > VERSION {
            return Err(format!(
                "key file version {} is newer than this game ({VERSION})",
                cfg.version
            ));
        }
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }

    /// Brings a file from an older version, or one written before some commands existed, up to
    /// date. Returns what it changed, in words for the log.
    ///
    /// - Version 0 (no `version` field): profiles had no `known` list; every command in their
    ///   bindings counts as known, the rest as new.
    /// - A command the profile has not seen gets its base keys, except a key the profile already
    ///   gives another command: that one stays with the player's choice and the new command
    ///   goes without it.
    /// - Bindings of commands this build does not know are kept: a module may register them
    ///   later.
    pub fn migrate(&mut self, registry: &Registry) -> Vec<String> {
        let mut notes = Vec::new();
        if self.version == 0 {
            for p in self.profiles.values_mut() {
                let ids: Vec<String> = p.bindings.keys().cloned().collect();
                p.known.extend(ids);
            }
            notes.push("key file: upgraded from version 0".to_owned());
        }
        self.version = VERSION;
        if !PROFILES.contains(&self.profile.as_str()) {
            self.profile = PROFILES[0].to_owned();
        }
        for profile in PROFILES {
            let file = self.profiles.entry(profile.to_owned()).or_default();
            if file.known.is_empty() && file.bindings.is_empty() {
                // A fresh profile knows every command as it is.
                file.known = registry.all().iter().map(|c| c.id.to_owned()).collect();
                continue;
            }
            let new: Vec<&CommandDef> = registry
                .all()
                .iter()
                .filter(|c| !file.known.contains(c.id))
                .collect();
            if new.is_empty() {
                continue;
            }
            // Keys the profile uses for the commands it knows.
            let mut taken: HashMap<Chord, &str> = HashMap::new();
            for def in registry.all().iter().filter(|c| file.known.contains(c.id)) {
                let keys = file
                    .bindings
                    .get(def.id)
                    .cloned()
                    .unwrap_or_else(|| base_keys(profile, def));
                for k in keys {
                    taken.entry(k).or_insert(def.id);
                }
            }
            for def in new {
                let base = base_keys(profile, def);
                let kept: Vec<Chord> = base
                    .iter()
                    .copied()
                    .filter(|k| !taken.contains_key(k))
                    .collect();
                if kept.len() != base.len() {
                    notes.push(format!(
                        "key file: new command {} left without a key the profile {profile} already uses",
                        def.id
                    ));
                    file.bindings.insert(def.id.to_owned(), kept.clone());
                }
                for k in kept {
                    taken.entry(k).or_insert(def.id);
                }
                file.known.insert(def.id.to_owned());
            }
        }
        notes
    }

    fn file(&self) -> Option<&ProfileFile> {
        self.profiles.get(&self.profile)
    }

    fn file_mut(&mut self) -> &mut ProfileFile {
        self.profiles.entry(self.profile.clone()).or_default()
    }

    /// The keys the active profile gives every command.
    pub fn resolve(&self, registry: &Registry) -> Resolved {
        let mut out = Resolved::default();
        for def in registry.all() {
            let keys = self.keys_of(def);
            for k in &keys {
                out.by_chord.entry(*k).or_insert(def.id);
            }
            out.by_command.insert(def.id, keys);
        }
        out
    }

    /// The keys the active profile gives one command.
    pub fn keys_of(&self, def: &CommandDef) -> Vec<Chord> {
        self.file()
            .and_then(|f| f.bindings.get(def.id))
            .cloned()
            .unwrap_or_else(|| base_keys(&self.profile, def))
    }

    /// Sets a command's keys in the active profile; keys equal to the base clear the override.
    pub fn set_keys(&mut self, def: &CommandDef, keys: Vec<Chord>) {
        let base = base_keys(&self.profile, def);
        let file = self.file_mut();
        if keys == base {
            file.bindings.remove(def.id);
        } else {
            file.bindings.insert(def.id.to_owned(), keys);
        }
    }

    /// Who already holds `chord`, other than `cmd`.
    pub fn conflict(
        &self,
        registry: &Registry,
        cmd: &'static str,
        chord: Chord,
    ) -> Option<Conflict> {
        registry
            .all()
            .iter()
            .filter(|d| d.id != cmd)
            .find(|d| self.keys_of(d).contains(&chord))
            .map(|d| Conflict {
                chord,
                wanted_by: cmd,
                held_by: d.id,
            })
    }

    /// Gives `chord` to `cmd` as its first key. When another command holds it, `swap` hands that
    /// command `cmd`'s old first key (or nothing); without `swap` the chord is not assigned and
    /// the conflict comes back.
    pub fn assign(
        &mut self,
        registry: &Registry,
        cmd: &'static str,
        chord: Chord,
        swap: bool,
    ) -> Result<(), Conflict> {
        let Some(def) = registry.get(cmd) else {
            return Ok(());
        };
        let conflict = self.conflict(registry, cmd, chord);
        if let Some(c) = conflict
            && !swap
        {
            return Err(c);
        }
        let mut mine = self.keys_of(def);
        let old = if mine.is_empty() {
            None
        } else {
            Some(mine.remove(0))
        };
        mine.retain(|k| *k != chord);
        mine.insert(0, chord);
        if let Some(c) = conflict
            && let Some(other) = registry.get(c.held_by)
        {
            let mut theirs = self.keys_of(other);
            if let Some(i) = theirs.iter().position(|k| *k == chord) {
                match old {
                    Some(o) if !theirs.contains(&o) => theirs[i] = o,
                    _ => {
                        theirs.remove(i);
                    }
                }
            }
            self.set_keys(other, theirs);
        }
        self.set_keys(def, mine);
        Ok(())
    }

    /// Removes one key from a command.
    pub fn unbind(&mut self, def: &CommandDef, chord: Chord) {
        let mut keys = self.keys_of(def);
        keys.retain(|k| *k != chord);
        self.set_keys(def, keys);
    }

    /// Back to the profile's base for one command.
    pub fn reset(&mut self, cmd: &str) {
        self.file_mut().bindings.remove(cmd);
    }

    /// Back to the profile's base for every command.
    pub fn reset_all(&mut self) {
        self.file_mut().bindings.clear();
    }

    /// Every chord two or more commands share in the active profile.
    pub fn conflicts(&self, registry: &Registry) -> Vec<Conflict> {
        let mut seen: HashMap<Chord, &'static str> = HashMap::new();
        let mut out = Vec::new();
        for def in registry.all() {
            for k in self.keys_of(def) {
                match seen.get(&k) {
                    Some(&held_by) => out.push(Conflict {
                        chord: k,
                        wanted_by: def.id,
                        held_by,
                    }),
                    None => {
                        seen.insert(k, def.id);
                    }
                }
            }
        }
        out
    }

    pub fn export(&self) -> Export {
        Export {
            version: VERSION,
            profile: self.profile.clone(),
            known: self.file().map(|f| f.known.clone()).unwrap_or_default(),
            bindings: self.file().map(|f| f.bindings.clone()).unwrap_or_default(),
        }
    }

    /// Replaces the active profile's changes with an exported file's, then reconciles it with
    /// the commands this build has.
    pub fn import(&mut self, registry: &Registry, export: Export) -> Vec<String> {
        let file = self.file_mut();
        file.bindings = export.bindings;
        file.known = export.known;
        if export.version == 0 {
            let ids: Vec<String> = file.bindings.keys().cloned().collect();
            file.known.extend(ids);
        }
        self.migrate(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::commands::Category;

    fn registry() -> Registry {
        Registry::builtin()
    }

    #[test]
    fn defaults_have_no_conflicts() {
        let r = registry();
        for profile in PROFILES {
            let mut cfg = KeyConfig::default();
            cfg.migrate(&r);
            cfg.profile = profile.to_owned();
            assert_eq!(cfg.conflicts(&r), vec![], "{profile}");
        }
    }

    #[test]
    fn conflict_is_reported_and_swap_trades_keys() {
        let r = registry();
        let mut cfg = KeyConfig::default();
        cfg.migrate(&r);
        let g = Chord::key(PhysKey::KeyG);
        // Research wants G, which the government window holds.
        let err = cfg.assign(&r, id::RESEARCH, g, false).unwrap_err();
        assert_eq!(err.held_by, id::GOVERNMENT);
        assert_eq!(err.wanted_by, id::RESEARCH);
        // Nothing changed.
        assert_eq!(cfg.resolve(&r).holder(g), Some(id::GOVERNMENT));
        // Swapping: research gets G, the government window gets research's R.
        cfg.assign(&r, id::RESEARCH, g, true).unwrap();
        let res = cfg.resolve(&r);
        assert_eq!(res.holder(g), Some(id::RESEARCH));
        assert_eq!(res.holder(Chord::key(PhysKey::KeyR)), Some(id::GOVERNMENT));
        assert!(cfg.conflicts(&r).is_empty());
        // Modifiers make a different chord: Ctrl+G is free.
        assert!(cfg.conflict(&r, id::PAUSE, g.ctrl()).is_none());
        // Resetting one command brings its default back, and with it the conflict.
        cfg.reset(id::RESEARCH);
        assert_eq!(cfg.conflicts(&r).len(), 1);
        cfg.reset_all();
        assert!(cfg.conflicts(&r).is_empty());
    }

    /// A key file written before some commands existed, where the player gave one of the new
    /// commands' default keys to something else.
    #[test]
    fn old_file_migrates_without_stealing_keys() {
        let old = r#"(
            layout: Ansi,
            profiles: {
                "standard": (
                    bindings: {
                        "time.pause": [(key: KeyB)],
                    },
                ),
            },
        )"#;
        let mut cfg = KeyConfig::parse(old).unwrap();
        assert_eq!(cfg.version, 0);
        let r = registry();
        let notes = cfg.migrate(&r);
        assert_eq!(cfg.version, VERSION);
        assert_eq!(cfg.layout, Layout::Ansi);
        let res = cfg.resolve(&r);
        // The player's key stays theirs.
        assert_eq!(res.holder(Chord::key(PhysKey::KeyB)), Some(id::PAUSE));
        // The budget tab is new to this file: it would default to B, which is taken.
        assert!(res.keys(id::TAB_BUDGET).is_empty());
        assert!(
            notes.iter().any(|n| n.contains(id::TAB_BUDGET)),
            "{notes:?}"
        );
        // Other new commands get their defaults.
        assert_eq!(res.keys(id::GOVERNMENT), [Chord::key(PhysKey::KeyG)]);
        assert!(cfg.conflicts(&r).is_empty());
        // Migrating again changes nothing.
        let again = cfg.clone();
        assert!(cfg.migrate(&r).is_empty());
        assert_eq!(cfg, again);

        // A command registered later by another module gets its default when free.
        let mut r2 = r.clone();
        r2.register(
            CommandDef::new("mod.extra_panel", Category::Windows)
                .keys([Chord::key(PhysKey::KeyO).shift()]),
        )
        .unwrap();
        cfg.migrate(&r2);
        assert_eq!(
            cfg.resolve(&r2).keys("mod.extra_panel"),
            [Chord::key(PhysKey::KeyO).shift()]
        );
    }

    #[test]
    fn file_round_trip_and_newer_version_refused() {
        let r = registry();
        let mut cfg = KeyConfig::default();
        cfg.migrate(&r);
        cfg.assign(&r, id::SAVE, Chord::key(PhysKey::F12), false)
            .unwrap();
        let text = ron::ser::to_string_pretty(&cfg, Default::default()).unwrap();
        assert_eq!(KeyConfig::parse(&text).unwrap(), cfg);
        assert!(KeyConfig::parse("(version: 99)").is_err());
        // Export and import move a profile's changes.
        let export = cfg.export();
        let mut other = KeyConfig::default();
        other.migrate(&r);
        other.import(&r, export);
        assert_eq!(other.resolve(&r).keys(id::SAVE), [Chord::key(PhysKey::F12)]);
    }
}
