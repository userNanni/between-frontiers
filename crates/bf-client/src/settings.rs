//! Player settings, stored as RON in the config folder.

use bf_i18n::Locale;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Autosave {
    Off,
    Monthly,
    #[default]
    Yearly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub locale: String,
    /// Interface scale in percent; 0 is automatic (`ui_scale`).
    pub ui_scale: u32,
    /// An interface sized for a stream, which wins over `ui_scale`.
    pub ui_stream: Option<crate::ui_scale::StreamTarget>,
    pub map_labels: bool,
    /// Terrain shading on the map.
    pub map_relief: bool,
    pub autosave: Autosave,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: default_locale().code().to_owned(),
            ui_scale: 0,
            ui_stream: None,
            map_labels: true,
            map_relief: true,
            autosave: Autosave::Yearly,
        }
    }
}

/// Portuguese unless the system says English.
fn default_locale() -> Locale {
    let lang = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    if lang.starts_with("en") {
        Locale::En
    } else {
        Locale::PtBr
    }
}

impl Settings {
    pub fn locale(&self) -> Locale {
        Locale::from_code(&self.locale).unwrap_or_default()
    }

    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| ron::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }
}
