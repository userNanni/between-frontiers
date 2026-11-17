//! Restore points of a campaign: copies of the engine save the player can go back to without
//! losing the others. They live in `points/` inside the campaign folder, one `<id>.lss` (the
//! same engine save as `world.lss`) and one `<id>.ron` (what the list shows) each. Going back to
//! a point copies it over `world.lss`; the points stay.
//!
//! Automatic points rotate (the oldest go first); manual points are never removed by the game.
//! The quick save keeps one slot that each quick save replaces.

use std::path::{Path, PathBuf};

use laplace_schelling::calendar::Date;
use serde::{Deserialize, Serialize};

use crate::saves::{Save, WORLD_FILE};

pub const DIR: &str = "points";
const SAVE_EXT: &str = "lss";
const META_EXT: &str = "ron";
/// Id of the quick save's slot.
pub const QUICK: &str = "quick";

/// Why the game wrote a point on its own before the player acted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    /// A law changed (it moves taxes, rights or the economy, and cannot be undone at once).
    Law,
    /// Territory given its freedom.
    Liberate,
    /// An agreement accepted.
    Accept,
    /// War declared.
    War,
    /// A treaty signed.
    Treaty,
    /// The player went back to another point: this is where they were.
    Load,
}

impl Reason {
    pub fn key(self) -> &'static str {
        match self {
            Reason::Law => "points-reason-law",
            Reason::Liberate => "points-reason-liberate",
            Reason::Accept => "points-reason-accept",
            Reason::War => "points-reason-war",
            Reason::Treaty => "points-reason-treaty",
            Reason::Load => "points-reason-load",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// On the autosave's interval.
    Auto,
    /// Before a risky or irreversible decision.
    Before(Reason),
    /// The quick save's slot.
    Quick,
    /// Saved by the player with a name.
    Manual,
}

impl Kind {
    /// Rotated by the game.
    pub fn automatic(self) -> bool {
        matches!(self, Kind::Auto | Kind::Before(_))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub id: String,
    pub kind: Kind,
    /// The player's name for a manual point (or a renamed one).
    #[serde(default)]
    pub name: Option<String>,
    /// Date in the game.
    pub date: Date,
    /// When it was written, seconds since the Unix epoch.
    pub written: u64,
    /// Size of the engine save, bytes.
    #[serde(default)]
    pub bytes: u64,
}

/// The rules of rotation and the disk ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Automatic points kept.
    pub keep_auto: usize,
    /// Points in all; past it the oldest automatic go, never a manual one.
    pub cap: usize,
}

pub fn dir(save: &Save) -> PathBuf {
    save.dir.join(DIR)
}

pub fn save_path(save: &Save, id: &str) -> PathBuf {
    dir(save).join(format!("{id}.{SAVE_EXT}"))
}

fn meta_path(save: &Save, id: &str) -> PathBuf {
    dir(save).join(format!("{id}.{META_EXT}"))
}

/// A new id: when it was written, and a counter in case two land in the same millisecond.
pub fn new_id(save: &Save) -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let mut n = 0;
    loop {
        let id = if n == 0 {
            format!("p{ms}")
        } else {
            format!("p{ms}-{n}")
        };
        if !save_path(save, &id).exists() {
            return id;
        }
        n += 1;
    }
}

/// The campaign's points, newest first.
pub fn list(save: &Save) -> Vec<Point> {
    let Ok(entries) = std::fs::read_dir(dir(save)) else {
        return Vec::new();
    };
    let mut out: Vec<Point> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == META_EXT))
        .filter_map(|e| {
            let p: Point = ron::from_str(&std::fs::read_to_string(e.path()).ok()?).ok()?;
            save_path(save, &p.id).is_file().then_some(p)
        })
        .collect();
    out.sort_by(|a, b| b.written.cmp(&a.written).then_with(|| b.id.cmp(&a.id)));
    out
}

/// Records a point whose engine save is already at [`save_path`].
pub fn record(save: &Save, mut point: Point) -> std::io::Result<Point> {
    point.bytes = std::fs::metadata(save_path(save, &point.id))?.len();
    write_meta(save, &point)?;
    Ok(point)
}

fn write_meta(save: &Save, point: &Point) -> std::io::Result<()> {
    std::fs::create_dir_all(dir(save))?;
    let text = ron::ser::to_string_pretty(point, ron::ser::PrettyConfig::default())
        .map_err(std::io::Error::other)?;
    let path = meta_path(save, &point.id);
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(tmp, path)
}

pub fn rename(save: &Save, id: &str, name: &str) -> std::io::Result<()> {
    let Some(mut p) = list(save).into_iter().find(|p| p.id == id) else {
        return Ok(());
    };
    let name = name.trim();
    p.name = (!name.is_empty()).then(|| name.to_owned());
    write_meta(save, &p)
}

pub fn delete(save: &Save, id: &str) -> std::io::Result<()> {
    for path in [save_path(save, id), meta_path(save, id)] {
        match std::fs::remove_file(path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

/// Removes automatic points past the limits, oldest first; returns how many went. Manual and
/// quick points are never removed here.
pub fn prune(save: &Save, limits: Limits) -> usize {
    let all = list(save);
    let autos: Vec<&Point> = all.iter().filter(|p| p.kind.automatic()).collect();
    let mut over_auto = autos.len().saturating_sub(limits.keep_auto);
    let mut over_cap = all.len().saturating_sub(limits.cap);
    let mut gone = 0;
    // `autos` is newest first: walk it from the oldest.
    for p in autos.iter().rev() {
        if over_auto == 0 && over_cap == 0 {
            break;
        }
        if delete(save, &p.id).is_ok() {
            gone += 1;
            over_auto = over_auto.saturating_sub(1);
            over_cap = over_cap.saturating_sub(1);
        }
    }
    gone
}

/// Makes `id` the campaign's state: copies its engine save over `world.lss` and moves the
/// campaign's date to it. The other points stay.
pub fn restore(save: &mut Save, id: &str) -> std::io::Result<()> {
    let p = list(save)
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, id.to_owned()))?;
    let world = save.dir.join(WORLD_FILE);
    let tmp = world.with_extension("tmp");
    std::fs::copy(save_path(save, id), &tmp)?;
    std::fs::rename(tmp, world)?;
    save.meta.date = p.date;
    save.meta.last_played = crate::saves::now();
    crate::saves::write(save)
}

/// Bytes the campaign folder takes on disk.
pub fn folder_bytes(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => folder_bytes(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

/// Size of the newest engine save among the campaigns (the current state or a point), bytes:
/// what one more point will roughly take.
pub fn last_size(saves: &[Save]) -> Option<u64> {
    saves.iter().find_map(|s| {
        std::fs::metadata(s.dir.join(WORLD_FILE))
            .ok()
            .map(|m| m.len())
    })
}

/// Months between automatic points for `interval`; `None` when off.
pub fn interval_months(interval: crate::settings::Autosave) -> Option<i32> {
    use crate::settings::Autosave::*;
    match interval {
        Off => None,
        Monthly => Some(1),
        Quarterly => Some(3),
        Semiannual => Some(6),
        Yearly => Some(12),
    }
}

/// Whether an automatic point is due at `now`, the last one at `last`.
pub fn due(interval: crate::settings::Autosave, last: Date, now: Date) -> bool {
    let Some(n) = interval_months(interval) else {
        return false;
    };
    let month = |d: Date| d.year * 12 + i32::from(d.month) - 1;
    // On the calendar's boundaries (each quarter, each half year), not counted from the start.
    month(now) / n != month(last) / n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Autosave;

    fn campaign(tag: &str) -> Save {
        let dir = std::env::temp_dir().join(format!("bf-points-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::saves::create(&dir, "Pontos", 1, Date::new(1925, 1, 1)).unwrap()
    }

    fn point(save: &Save, kind: Kind, written: u64, date: Date) -> Point {
        let id = format!("p{written}");
        std::fs::create_dir_all(dir(save)).unwrap();
        std::fs::write(save_path(save, &id), format!("world {written}")).unwrap();
        record(
            save,
            Point {
                id,
                kind,
                name: None,
                date,
                written,
                bytes: 0,
            },
        )
        .unwrap()
    }

    #[test]
    fn rotation_keeps_manual_points_and_the_newest_automatic() {
        let save = campaign("rot");
        let d = Date::new(1926, 1, 1);
        for t in 1..=6 {
            point(&save, Kind::Auto, t, d);
        }
        point(&save, Kind::Manual, 7, d);
        point(&save, Kind::Before(Reason::Law), 8, d);
        point(&save, Kind::Quick, 9, d);
        let limits = Limits {
            keep_auto: 3,
            cap: 100,
        };
        assert_eq!(prune(&save, limits), 4);
        let ids: Vec<String> = list(&save).into_iter().map(|p| p.id).collect();
        assert_eq!(ids, ["p9", "p8", "p7", "p6", "p5"]);
        // The ceiling takes automatic points only, even past it.
        let limits = Limits {
            keep_auto: 10,
            cap: 2,
        };
        prune(&save, limits);
        let kinds: Vec<Kind> = list(&save).into_iter().map(|p| p.kind).collect();
        assert_eq!(kinds, [Kind::Quick, Kind::Manual]);
        let _ = std::fs::remove_dir_all(save.dir.parent().unwrap());
    }

    #[test]
    fn restoring_copies_the_point_and_keeps_the_others() {
        let mut save = campaign("restore");
        std::fs::write(save.dir.join(WORLD_FILE), "now").unwrap();
        let a = point(&save, Kind::Auto, 1, Date::new(1930, 6, 1));
        point(&save, Kind::Manual, 2, Date::new(1931, 1, 1));
        restore(&mut save, &a.id).unwrap();
        assert_eq!(
            std::fs::read_to_string(save.dir.join(WORLD_FILE)).unwrap(),
            "world 1"
        );
        assert_eq!(save.meta.date, Date::new(1930, 6, 1));
        assert_eq!(list(&save).len(), 2);
        rename(&save, &a.id, "  Antes da guerra ").unwrap();
        assert_eq!(list(&save)[1].name.as_deref(), Some("Antes da guerra"));
        delete(&save, &a.id).unwrap();
        assert_eq!(list(&save).len(), 1);
        let _ = std::fs::remove_dir_all(save.dir.parent().unwrap());
    }

    #[test]
    fn intervals_fall_on_calendar_boundaries() {
        let d = |y, m| Date::new(y, m, 1);
        assert!(!due(Autosave::Off, d(1925, 1), d(1930, 1)));
        assert!(due(Autosave::Monthly, d(1925, 1), d(1925, 2)));
        assert!(!due(Autosave::Quarterly, d(1925, 1), d(1925, 3)));
        assert!(due(Autosave::Quarterly, d(1925, 3), d(1925, 4)));
        assert!(!due(Autosave::Semiannual, d(1925, 2), d(1925, 6)));
        assert!(due(Autosave::Semiannual, d(1925, 6), d(1925, 7)));
        assert!(due(Autosave::Yearly, d(1925, 12), d(1926, 1)));
        assert!(!due(Autosave::Yearly, d(1925, 1), d(1925, 12)));
    }
}
