//! Where the bench reads the game's data, found when it runs, never where it was built:
//! `BF_DATA_ROOT=<dir>` if set (a copy of the repository, or one with a file swapped), else the
//! first directory from the current one up that holds both `data/` and `crates/`; with neither,
//! it stops with a message. The path it was built in is not used: a binary built in one
//! checkout and reused by another (two trees with the same sources in one `CARGO_TARGET_DIR`,
//! as the shared base target did on 61a1918) read the first checkout's `data/`. The first call
//! prints the root and a hash of every file under its `data/` (`data/raw/` left out), so each
//! run says what it read.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The root the data is read from.
pub fn root() -> PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::var_os("BF_DATA_ROOT")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok().and_then(|d| find_up(&d)))
            .unwrap_or_else(|| {
                eprintln!(
                    "bf-bench: no data root: set BF_DATA_ROOT, or run from inside a checkout \
                     (a directory holding data/ and crates/)"
                );
                std::process::exit(2)
            });
        let shown = root.canonicalize().unwrap_or_else(|_| root.clone());
        println!(
            "data root {} (data hash {:016x})",
            shown.display(),
            data_hash(&root)
        );
        root
    })
    .clone()
}

/// The first directory from `start` up that holds both `data/` and `crates/`.
pub fn find_up(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|d| d.join("data").is_dir() && d.join("crates").is_dir())
        .map(Path::to_path_buf)
}

/// FNV-1a over the paths (relative to `data/`) and bytes of every file under `root/data`, in path
/// order, `raw/` left out. Stable across builds and machines.
pub fn data_hash(root: &Path) -> u64 {
    let data = root.join("data");
    let mut files = Vec::new();
    walk(&data, &mut files);
    files.sort();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for f in files {
        let rel = f
            .strip_prefix(&data)
            .unwrap_or(&f)
            .to_string_lossy()
            .into_owned();
        if rel.starts_with("raw") {
            continue;
        }
        eat(rel.as_bytes());
        eat(&std::fs::read(&f).unwrap_or_default());
    }
    h
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_is_found_from_inside_the_checkout() {
        let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = find_up(&here).expect("the checkout holds data/ and crates/");
        assert!(root.join("data").is_dir() && root.join("crates").is_dir());
        assert!(find_up(Path::new("/")).is_none());
    }
}
