//! The world snapshot's format: a small header with the format's version, then the world in
//! postcard. Older versions read through the chain of frozen layouts in [`compat`].
//!
//! ```text
//! | "BFWD" | version: u32 LE | n: u32 LE | postcard(Option<CatalogKeys>): n bytes | postcard(GameWorld) |
//! ```
//!
//! The keys of the catalogues the world indexes by position ([`catalog::CatalogKeys`]) let a
//! save outlive a technology added in the middle of the tree ([`catalog::fit`]).
//!
//! Snapshots written before the header (versions 0 to 6 of this list) start straight with the
//! world's postcard, whose first byte is the length of the scenario's name: never `B`.
//!
//! # Changing what the world saves
//!
//! Any change to what [`GameWorld`] serialises (a field added, removed, reordered or retyped,
//! in the world or in any part of it) is a new version:
//!
//! 1. raise [`WORLD_VERSION`];
//! 2. freeze the layout it replaces in [`compat`] as `V<old>` (and freeze the old layout of the
//!    part that changed beside the part, as `EconomyBeforeLedger` is);
//! 3. write `v<old>_to_v<new>` and add the step to [`migrate`];
//! 4. add the old layout to the test of every version (`every_version_loads_with_its_hash`);
//!    a variant added at the end of an enum leaves the old bytes readable as they are: raise
//!    the version and read the old one as the new (as 3 reads as 4), with no frozen struct;
//! 5. bless the new schema: `BF_BLESS=1 cargo test -p bf-sim --test world_schema` writes
//!    `tests/schema/world-v<new>.txt`. The test fails while `GameWorld` differs from the
//!    schema of the current version, so a change without a new version does not pass.

pub mod catalog;
pub mod compat;

pub use catalog::{CatalogError, CatalogKeys};

use laplace_schelling::save::{SaveError, SaveFile, decode_snapshot, encode_snapshot};

use crate::game::{GAME_ID, GameCommand, GameWorld};

/// Marks a snapshot with a header.
pub const WORLD_MAGIC: [u8; 4] = *b"BFWD";

/// Version of the world's format this build writes.
///
/// 0. before the communities in the population;
/// 1. the communities;
/// 2. the border state;
/// 3. the ledger of debts between states;
/// 4. debts agreed by treaty (`DebtTerm::Agreement`) and the dissolution of a country
///    (`Operation::Dissolve`): variants added at the end of their enums, so a version 3 world
///    reads as a 4;
/// 5. research that costs (public works with research, a research priority in the laws);
/// 6. foreign capital (`GameWorld::foreign`), the last field of the world.
pub const WORLD_VERSION: u32 = 6;

/// Why a snapshot does not read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// Written by a newer build than this one.
    Newer { found: u32, supported: u32 },
    /// The bytes do not decode as any known version.
    Malformed(SaveError),
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldError::Newer { found, supported } => write!(
                f,
                "world format {found} is newer than this build reads ({supported})"
            ),
            WorldError::Malformed(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WorldError {}

/// The world with its header, in the current version, with the keys of the catalogues it was
/// played with.
pub fn encode_world(
    world: &GameWorld,
    catalog: Option<&CatalogKeys>,
) -> Result<Vec<u8>, SaveError> {
    let keys = encode_snapshot(&catalog)?;
    let body = encode_snapshot(world)?;
    let mut out = Vec::with_capacity(12 + keys.len() + body.len());
    out.extend_from_slice(&WORLD_MAGIC);
    out.extend_from_slice(&WORLD_VERSION.to_le_bytes());
    let n = u32::try_from(keys.len()).map_err(|e| SaveError::Encode(e.to_string()))?;
    out.extend_from_slice(&n.to_le_bytes());
    out.extend_from_slice(&keys);
    out.extend_from_slice(&body);
    Ok(out)
}

/// A snapshot's header and its world's bytes.
pub struct Split<'a> {
    pub version: u32,
    /// The catalogues' keys; `None` when the header could not hold them (a newer version) or
    /// the writer had no catalogue.
    pub catalog: Option<CatalogKeys>,
    pub body: &'a [u8],
}

/// The header of a snapshot; `None` for a snapshot without one.
pub fn split(bytes: &[u8]) -> Option<Split<'_>> {
    let rest = bytes.strip_prefix(&WORLD_MAGIC)?;
    let (v, rest) = rest.split_at_checked(4)?;
    let version = u32::from_le_bytes(v.try_into().ok()?);
    if version > WORLD_VERSION {
        return Some(Split {
            version,
            catalog: None,
            body: rest,
        });
    }
    let (n, rest) = rest.split_at_checked(4)?;
    let n = usize::try_from(u32::from_le_bytes(n.try_into().ok()?)).ok()?;
    let (keys, body) = rest.split_at_checked(n)?;
    Some(Split {
        version,
        catalog: decode_snapshot::<Option<CatalogKeys>>(keys).ok().flatten(),
        body,
    })
}

/// Decodes a snapshot of any version this build knows, up to the current one, with the keys of
/// the catalogues it was written with (`None` before the header).
pub fn decode(bytes: &[u8]) -> Result<(GameWorld, Option<CatalogKeys>), WorldError> {
    if bytes.starts_with(&WORLD_MAGIC) && split(bytes).is_none() {
        return Err(WorldError::Malformed(SaveError::Truncated));
    }
    match split(bytes) {
        Some(s) if s.version > WORLD_VERSION => Err(WorldError::Newer {
            found: s.version,
            supported: WORLD_VERSION,
        }),
        Some(s) => migrate(s.version, s.body)
            .map(|w| (w, s.catalog))
            .map_err(WorldError::Malformed),
        None => decode_legacy(bytes)
            .map(|w| (w, None))
            .map_err(WorldError::Malformed),
    }
}

/// [`decode`] without the keys.
pub fn decode_world(bytes: &[u8]) -> Result<GameWorld, WorldError> {
    decode(bytes).map(|(w, _)| w)
}

/// Reads the world of version `v` and steps it up to the current version.
pub fn migrate(v: u32, body: &[u8]) -> Result<GameWorld, SaveError> {
    use compat::*;
    Ok(match v {
        0 => v5_to_v6(v2_to_v5(v1_to_v2(v0_to_v1(decode_snapshot(body)?)))),
        1 => v5_to_v6(v2_to_v5(v1_to_v2(decode_snapshot(body)?))),
        2 => v5_to_v6(v2_to_v5(decode_snapshot(body)?)),
        // Version 3 lacks only enum variants added at the end since: its bytes read as 4.
        3 | 4 => v5_to_v6(v4_to_v5(decode_snapshot(body)?)),
        5 => v5_to_v6(decode_snapshot(body)?),
        6 => decode_snapshot(body)?,
        _ => return Err(SaveError::Malformed(format!("unknown world version {v}"))),
    })
}

/// A snapshot written before the header: the version is not written, so the layouts are tried
/// newest first. Postcard reads a prefix and ignores the rest, so a layout with fewer fields
/// would read a newer snapshot wrongly; and a newer layout reads an older snapshot only with
/// its missing tail of empty lists padded in (one zero byte each).
fn decode_legacy(bytes: &[u8]) -> Result<GameWorld, SaveError> {
    let first = migrate(WORLD_VERSION, bytes);
    if first.is_ok() {
        return first;
    }
    for v in [5, 4, 2] {
        if let Ok(w) = migrate(v, bytes) {
            return Ok(w);
        }
    }
    // Version 1 (no border state), and the ledger written before the border state on some
    // branches: read with the border state's empty lists padded in.
    let mut padded = bytes.to_vec();
    for k in 1..=8 {
        padded.resize(bytes.len() + k, 0);
        for v in [WORLD_VERSION, 5, 4, 2] {
            if let Ok(w) = migrate(v, &padded) {
                return Ok(w);
            }
        }
    }
    migrate(0, bytes).or(first)
}

/// What the campaign list shows of a save without loading it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveInfo {
    /// Version of the game that wrote it.
    pub game_version: String,
    /// Version of the world's format; `None` before the header.
    pub world_version: Option<u32>,
}

impl SaveInfo {
    /// This build reads it.
    pub fn readable(&self) -> bool {
        self.world_version.is_none_or(|v| v <= WORLD_VERSION)
    }
}

/// Reads a save's container and the version of its world, without decoding the world.
pub fn inspect(bytes: &[u8]) -> Result<SaveInfo, SaveError> {
    let file: SaveFile<GameCommand> = SaveFile::decode(bytes, GAME_ID)?;
    Ok(SaveInfo {
        game_version: file.header.game_version,
        world_version: split(&file.snapshot).map(|s| s.version),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use laplace_schelling::hash::hash_of;

    fn world() -> GameWorld {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        crate::data::GameData::load(&root).unwrap().new_world()
    }

    #[test]
    fn the_header_round_trips_and_a_newer_version_is_refused() {
        let w = world();
        let bytes = encode_world(&w, None).unwrap();
        assert_eq!(split(&bytes).map(|s| s.version), Some(WORLD_VERSION));
        assert_eq!(hash_of(&decode_world(&bytes).unwrap()), hash_of(&w));
        // A snapshot without the header reads as before.
        let legacy = encode_snapshot(&w).unwrap();
        assert_eq!(legacy[0], w.scenario.len() as u8);
        assert_eq!(hash_of(&decode_world(&legacy).unwrap()), hash_of(&w));
        let mut newer = bytes.clone();
        newer[4..8].copy_from_slice(&(WORLD_VERSION + 1).to_le_bytes());
        assert_eq!(
            decode_world(&newer).err(),
            Some(WorldError::Newer {
                found: WORLD_VERSION + 1,
                supported: WORLD_VERSION
            })
        );
    }
}
