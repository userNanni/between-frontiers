//! Campaign saves. One folder per campaign under `saves/`; phase 0 writes only the metadata.
//! The engine's save container (snapshot plus command log) will live in the same folder.

use std::path::{Path, PathBuf};

use laplace_schelling::calendar::Date;
use serde::{Deserialize, Serialize};

pub const FORMAT: u32 = 1;
const META: &str = "campaign.ron";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveMeta {
    pub format: u32,
    pub name: String,
    pub scenario: String,
    pub seed: u64,
    /// Tag of the player's country; `None` until it is chosen on the map.
    pub player: Option<String>,
    pub date: Date,
    /// Seconds since the Unix epoch.
    pub created: u64,
    pub last_played: u64,
}

#[derive(Clone, Debug)]
pub struct Save {
    pub dir: PathBuf,
    pub meta: SaveMeta,
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Folder name from the campaign name: lower-case ASCII letters, digits and dashes.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars().flat_map(fold_accent) {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "campanha".to_owned()
    } else {
        out
    }
}

fn fold_accent(c: char) -> impl Iterator<Item = char> {
    let base = match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
        'ç' | 'Ç' => 'c',
        'ñ' | 'Ñ' => 'n',
        other => other,
    };
    std::iter::once(base)
}

pub fn list(dir: &Path) -> Vec<Save> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut saves: Vec<Save> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let dir = e.path();
            let text = std::fs::read_to_string(dir.join(META)).ok()?;
            let meta: SaveMeta = ron::from_str(&text).ok()?;
            Some(Save { dir, meta })
        })
        .collect();
    saves.sort_by(|a, b| {
        b.meta
            .last_played
            .cmp(&a.meta.last_played)
            .then_with(|| a.meta.name.cmp(&b.meta.name))
    });
    saves
}

pub fn name_taken(dir: &Path, name: &str) -> bool {
    let s = slug(name);
    list(dir).iter().any(|x| {
        x.meta.name.eq_ignore_ascii_case(name.trim())
            || x.dir.file_name().is_some_and(|f| f == s.as_str())
    })
}

pub fn write(save: &Save) -> std::io::Result<()> {
    std::fs::create_dir_all(&save.dir)?;
    let text = ron::ser::to_string_pretty(&save.meta, ron::ser::PrettyConfig::default())
        .map_err(std::io::Error::other)?;
    let tmp = save.dir.join(format!("{META}.tmp"));
    std::fs::write(&tmp, text)?;
    std::fs::rename(tmp, save.dir.join(META))
}

pub fn create(saves_dir: &Path, name: &str, seed: u64, start: Date) -> std::io::Result<Save> {
    let now = now();
    let save = Save {
        dir: saves_dir.join(slug(name)),
        meta: SaveMeta {
            format: FORMAT,
            name: name.trim().to_owned(),
            scenario: "1925".to_owned(),
            seed,
            player: None,
            date: start,
            created: now,
            last_played: now,
        },
    };
    write(&save)?;
    Ok(save)
}

pub fn delete(save: &Save) -> std::io::Result<()> {
    // Only remove folders that hold a campaign file, never anything else.
    if save.dir.join(META).is_file() {
        std::fs::remove_dir_all(&save.dir)
    } else {
        Ok(())
    }
}

/// Civil date (UTC) of a Unix timestamp.
pub fn civil_date(secs: u64) -> Date {
    // Howard Hinnant's days_from_civil inverse.
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    Date::new((y + i64::from(m <= 2)) as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_fold_accents_and_spaces() {
        assert_eq!(slug("Campanha do Brasil"), "campanha-do-brasil");
        assert_eq!(slug("  São Paulo / 1925 "), "sao-paulo-1925");
        assert_eq!(slug("???"), "campanha");
    }

    #[test]
    fn civil_dates() {
        assert_eq!(civil_date(0), Date::new(1970, 1, 1));
        assert_eq!(civil_date(1_763_164_800), Date::new(2025, 11, 15));
        assert_eq!(civil_date(951_782_400), Date::new(2000, 2, 29));
    }

    #[test]
    fn create_list_delete_round_trip() {
        let dir = std::env::temp_dir().join(format!("bf-saves-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let s = create(&dir, "Teste Ação", 7, Date::new(1925, 1, 1)).unwrap();
        assert!(name_taken(&dir, "teste ação"));
        let all = list(&dir);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].meta, s.meta);
        delete(&all[0]).unwrap();
        assert!(list(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
