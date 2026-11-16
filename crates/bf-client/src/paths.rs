//! Where the game reads assets and writes saves and settings.

use std::path::{Path, PathBuf};

/// Game files shipped with the executable.
#[derive(Clone, Debug)]
pub struct Paths {
    /// `assets/`: fonts, i18n, map.
    pub assets: PathBuf,
    /// `data/`: scenario files.
    pub data: PathBuf,
    /// Player files: saves.
    pub user_data: PathBuf,
    /// Player settings.
    pub config: PathBuf,
}

impl Paths {
    /// `BF_ROOT` overrides the folder holding `assets/` and `data/`; otherwise the folder next to
    /// the executable, then the working directory and its parents. `BF_USER_DIR` overrides where
    /// saves and settings go.
    pub fn discover() -> Self {
        let root = std::env::var_os("BF_ROOT")
            .map(PathBuf::from)
            .or_else(|| {
                let exe = std::env::current_exe().ok()?;
                exe.ancestors().find(|p| is_root(p)).map(Path::to_path_buf)
            })
            .or_else(|| {
                let cwd = std::env::current_dir().ok()?;
                cwd.ancestors().find(|p| is_root(p)).map(Path::to_path_buf)
            })
            .unwrap_or_else(|| PathBuf::from("."));
        let (user_data, config) = match std::env::var_os("BF_USER_DIR") {
            Some(dir) => {
                let dir = PathBuf::from(dir);
                (dir.clone(), dir)
            }
            None => match directories::ProjectDirs::from("", "", "between-frontiers") {
                Some(d) => (d.data_dir().to_path_buf(), d.config_dir().to_path_buf()),
                None => (root.join("user"), root.join("user")),
            },
        };
        Self::new(&root, user_data, config)
    }

    pub fn new(root: &Path, user_data: PathBuf, config: PathBuf) -> Self {
        Self {
            assets: root.join("assets"),
            data: root.join("data"),
            user_data,
            config,
        }
    }

    pub fn saves(&self) -> PathBuf {
        self.user_data.join("saves")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config.join("settings.ron")
    }

    /// The player's keyboard shortcuts.
    pub fn keys_file(&self) -> PathBuf {
        self.config.join("keys.ron")
    }
}

fn is_root(p: &Path) -> bool {
    p.join("assets/i18n").is_dir() && p.join("data").is_dir()
}
