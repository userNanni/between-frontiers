//! Player settings, stored as RON in the config folder.

use bf_i18n::Locale;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Autosave {
    Off,
    Monthly,
    #[default]
    Quarterly,
    Semiannual,
    Yearly,
}

impl Autosave {
    pub const ALL: [Autosave; 5] = [
        Autosave::Off,
        Autosave::Monthly,
        Autosave::Quarterly,
        Autosave::Semiannual,
        Autosave::Yearly,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Autosave::Off => "settings-autosave-off",
            Autosave::Monthly => "settings-autosave-monthly",
            Autosave::Quarterly => "settings-autosave-quarterly",
            Autosave::Semiannual => "settings-autosave-semiannual",
            Autosave::Yearly => "settings-autosave-yearly",
        }
    }
}

/// Choices offered for the automatic points kept.
pub const KEEP_CHOICES: [u32; 4] = [3, 5, 10, 20];
/// Choices offered for the ceiling of points per campaign.
pub const CAP_CHOICES: [u32; 4] = [10, 30, 60, 100];
/// Choices offered for the folder size that warns, MB.
pub const WARN_CHOICES: [u32; 4] = [250, 500, 1000, 2000];

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
    /// Interval of the automatic restore points.
    pub autosave: Autosave,
    /// Automatic points kept per campaign; older ones rotate out.
    pub autosave_keep: u32,
    /// An automatic point before a risky or irreversible decision.
    pub point_before_risk: bool,
    /// Points per campaign at most; past it the oldest automatic go (never a manual one).
    pub points_cap: u32,
    /// A campaign folder past this many MB is flagged.
    pub warn_mb: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: default_locale().code().to_owned(),
            ui_scale: 0,
            ui_stream: None,
            map_labels: true,
            map_relief: true,
            autosave: Autosave::Quarterly,
            autosave_keep: 5,
            point_before_risk: true,
            points_cap: 30,
            warn_mb: 1000,
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
    pub fn limits(&self) -> crate::points::Limits {
        crate::points::Limits {
            keep_auto: self.autosave_keep.max(1) as usize,
            cap: self.points_cap.max(1) as usize,
        }
    }

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
