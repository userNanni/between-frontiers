//! Where the bench reads the game's data. By default the checkout it was built from
//! (`CARGO_MANIFEST_DIR/../..`): a binary copied elsewhere still reads that checkout's `data/`,
//! which is a trap when comparing two variants of the data. `BF_DATA_ROOT=<dir>` reads another
//! root (a copy of the repository, or one with a file swapped). The first call prints the root and
//! a hash of every file under its `data/` (`data/raw/` left out), so each run says what it read.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The root the data is read from.
pub fn root() -> PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::var_os("BF_DATA_ROOT")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
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
