//! Save container.
//!
//! A save is a snapshot of the world plus the commands applied since, so loading it and
//! replaying the log reproduces the game exactly. The engine owns the container; the game owns
//! what the snapshot means. No I/O happens here: the container is encoded to and decoded from
//! bytes, and the caller decides where they go.
//!
//! # Layout (format version 1)
//!
//! All integers little-endian.
//!
//! | Field | Size |
//! |---|---|
//! | magic `\x89LSV\r\n\x1a\n` | 8 |
//! | format version | `u32` |
//! | header length, header ([`SaveHeader`], postcard) | `u32` + n |
//! | snapshot length, snapshot bytes | `u64` + n |
//! | log length, command log (postcard) | `u64` + n |
//! | BLAKE3 of every preceding byte | 32 |
//!
//! The magic follows PNG's: a high byte catches 7-bit transfers, `\r\n` and `\n` catch newline
//! conversion, `\x1a` stops `type` on Windows. Decoding checks magic, then version, then the
//! checksum, and only then parses, so a foreign, newer or damaged file is rejected with a
//! typed [`SaveError`] before any of its bytes are trusted.

use core::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize as SerializeDerive};

use crate::command::CommandLog;
use crate::hash::StateHash;
use crate::schedule::Clock;

/// File signature.
pub const MAGIC: [u8; 8] = *b"\x89LSV\r\n\x1a\n";

/// Container format version written by this engine.
pub const FORMAT_VERSION: u32 = 1;

const CHECKSUM_LEN: usize = 32;

/// Readable metadata, decoded before the snapshot.
#[derive(Clone, Debug, PartialEq, Eq, SerializeDerive, Deserialize)]
pub struct SaveHeader {
    /// Game identifier; [`SaveFile::decode`] refuses another game's save.
    pub game: String,
    /// Game version that wrote the save.
    pub game_version: String,
    /// Engine version that wrote the save.
    pub engine_version: String,
    /// World seed.
    pub seed: u64,
    /// Clock at the snapshot.
    pub clock: Clock,
    /// State hash of the snapshot, to verify after the game decodes it.
    pub state_hash: StateHash,
    /// Free text for the load screen (player name, country). Not interpreted.
    pub label: String,
}

impl SaveHeader {
    /// A header with this engine's version and an empty label.
    pub fn new(
        game: &str,
        game_version: &str,
        seed: u64,
        clock: Clock,
        state_hash: StateHash,
    ) -> Self {
        Self {
            game: game.to_owned(),
            game_version: game_version.to_owned(),
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
            seed,
            clock,
            state_hash,
            label: String::new(),
        }
    }
}

/// A decoded save.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveFile<C> {
    /// Metadata.
    pub header: SaveHeader,
    /// Opaque world snapshot (see [`encode_snapshot`]).
    pub snapshot: Vec<u8>,
    /// Commands applied from the snapshot's tick on.
    pub log: CommandLog<C>,
}

/// Why bytes are not a usable save.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SaveError {
    /// The magic is missing: not a save of this engine.
    NotASave,
    /// Written by a container format this engine does not read.
    UnsupportedVersion {
        /// Version in the file.
        found: u32,
        /// Version this engine reads.
        supported: u32,
    },
    /// Shorter than its own section lengths say.
    Truncated,
    /// The checksum does not match: the file is damaged.
    ChecksumMismatch,
    /// A save of another game.
    WrongGame {
        /// Game the caller expected.
        expected: String,
        /// Game in the file.
        found: String,
    },
    /// The checksum matched but a section does not decode (a bug in the writer).
    Malformed(String),
    /// A value could not be encoded.
    Encode(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotASave => f.write_str("not a Laplace-Schelling save file"),
            Self::UnsupportedVersion { found, supported } => {
                write!(
                    f,
                    "save format version {found} is not supported (this build reads {supported})"
                )
            }
            Self::Truncated => f.write_str("save file is truncated"),
            Self::ChecksumMismatch => f.write_str("save file is corrupted (checksum mismatch)"),
            Self::WrongGame { expected, found } => {
                write!(f, "save belongs to {found:?}, expected {expected:?}")
            }
            Self::Malformed(why) => write!(f, "save file is malformed: {why}"),
            Self::Encode(why) => write!(f, "could not encode save: {why}"),
        }
    }
}

impl std::error::Error for SaveError {}

/// Encodes a world snapshot with postcard.
pub fn encode_snapshot<T: Serialize>(state: &T) -> Result<Vec<u8>, SaveError> {
    postcard::to_allocvec(state).map_err(|e| SaveError::Encode(e.to_string()))
}

/// Decodes a world snapshot written by [`encode_snapshot`].
pub fn decode_snapshot<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SaveError> {
    postcard::from_bytes(bytes).map_err(|e| SaveError::Malformed(format!("snapshot: {e}")))
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], SaveError> {
        if self.bytes.len() < n {
            return Err(SaveError::Truncated);
        }
        let (head, rest) = self.bytes.split_at(n);
        self.bytes = rest;
        Ok(head)
    }

    fn u32(&mut self) -> Result<u32, SaveError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }

    fn u64(&mut self) -> Result<u64, SaveError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }

    fn section_u64(&mut self) -> Result<&'a [u8], SaveError> {
        let len = usize::try_from(self.u64()?).map_err(|_| SaveError::Truncated)?;
        self.take(len)
    }
}

impl<C: Serialize + DeserializeOwned> SaveFile<C> {
    /// Encodes the container.
    pub fn encode(&self) -> Result<Vec<u8>, SaveError> {
        let header =
            postcard::to_allocvec(&self.header).map_err(|e| SaveError::Encode(e.to_string()))?;
        let log = postcard::to_allocvec(&self.log).map_err(|e| SaveError::Encode(e.to_string()))?;
        let header_len = u32::try_from(header.len())
            .map_err(|_| SaveError::Encode("header too large".into()))?;
        let mut out = Vec::with_capacity(
            8 + 4 + 4 + header.len() + 16 + self.snapshot.len() + log.len() + CHECKSUM_LEN,
        );
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&header_len.to_le_bytes());
        out.extend_from_slice(&header);
        out.extend_from_slice(&(self.snapshot.len() as u64).to_le_bytes());
        out.extend_from_slice(&self.snapshot);
        out.extend_from_slice(&(log.len() as u64).to_le_bytes());
        out.extend_from_slice(&log);
        let checksum = blake3::hash(&out);
        out.extend_from_slice(checksum.as_bytes());
        Ok(out)
    }

    /// Decodes and verifies a container written for `expected_game`.
    pub fn decode(bytes: &[u8], expected_game: &str) -> Result<Self, SaveError> {
        if bytes.len() < MAGIC.len() || bytes[..MAGIC.len()] != MAGIC {
            return Err(SaveError::NotASave);
        }
        let mut reader = Reader {
            bytes: &bytes[MAGIC.len()..],
        };
        let version = reader.u32()?;
        if version != FORMAT_VERSION {
            return Err(SaveError::UnsupportedVersion {
                found: version,
                supported: FORMAT_VERSION,
            });
        }
        if bytes.len() < MAGIC.len() + 4 + CHECKSUM_LEN {
            return Err(SaveError::Truncated);
        }
        let (body, checksum) = bytes.split_at(bytes.len() - CHECKSUM_LEN);
        if blake3::hash(body).as_bytes() != checksum {
            return Err(SaveError::ChecksumMismatch);
        }
        let mut reader = Reader {
            bytes: &body[MAGIC.len() + 4..],
        };
        let header_len = reader.u32()? as usize;
        let header: SaveHeader = postcard::from_bytes(reader.take(header_len)?)
            .map_err(|e| SaveError::Malformed(format!("header: {e}")))?;
        if header.game != expected_game {
            return Err(SaveError::WrongGame {
                expected: expected_game.to_owned(),
                found: header.game,
            });
        }
        let snapshot = reader.section_u64()?.to_vec();
        let log = postcard::from_bytes(reader.section_u64()?)
            .map_err(|e| SaveError::Malformed(format!("command log: {e}")))?;
        if !reader.bytes.is_empty() {
            return Err(SaveError::Malformed(
                "trailing bytes after the command log".into(),
            ));
        }
        Ok(Self {
            header,
            snapshot,
            log,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{FORMAT_VERSION, MAGIC, SaveError, SaveFile, SaveHeader};
    use crate::calendar::Date;
    use crate::command::{CommandLog, Stamped};
    use crate::hash::hash_of;
    use crate::schedule::Clock;

    fn sample() -> SaveFile<String> {
        let mut log = CommandLog::new();
        log.push(Stamped {
            tick: 3,
            issuer: 1,
            seq: 0,
            command: "raise taxes".to_owned(),
        });
        let mut header = SaveHeader::new(
            "between-frontiers",
            "0.0.1",
            1925,
            Clock {
                tick: 3,
                date: Date::new(1925, 1, 4),
            },
            hash_of(&7u64),
        );
        header.label = "Brasil".into();
        SaveFile {
            header,
            snapshot: vec![1, 2, 3, 4, 5],
            log,
        }
    }

    fn reseal(mut bytes: Vec<u8>) -> Vec<u8> {
        let body_len = bytes.len() - 32;
        let checksum = blake3::hash(&bytes[..body_len]);
        bytes[body_len..].copy_from_slice(checksum.as_bytes());
        bytes
    }

    #[test]
    fn round_trips() {
        let save = sample();
        let bytes = save.encode().unwrap();
        assert_eq!(&bytes[..8], &MAGIC);
        assert_eq!(SaveFile::decode(&bytes, "between-frontiers"), Ok(save));
    }

    #[test]
    fn rejects_foreign_corrupted_and_newer_files() {
        let bytes = sample().encode().unwrap();
        let decode = |b: &[u8]| SaveFile::<String>::decode(b, "between-frontiers");

        assert_eq!(decode(b""), Err(SaveError::NotASave));
        assert_eq!(decode(b"PK\x03\x04 a zip file"), Err(SaveError::NotASave));
        let mut text_mangled = bytes.clone();
        text_mangled.remove(4); // "\r\n" -> "\n"
        assert_eq!(decode(&text_mangled), Err(SaveError::NotASave));

        let mut newer = bytes.clone();
        newer[8..12].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
        assert_eq!(
            decode(&newer),
            Err(SaveError::UnsupportedVersion {
                found: FORMAT_VERSION + 1,
                supported: FORMAT_VERSION
            })
        );

        for at in [12, 20, bytes.len() / 2, bytes.len() - 40, bytes.len() - 1] {
            let mut flipped = bytes.clone();
            flipped[at] ^= 0x40;
            assert_eq!(
                decode(&flipped),
                Err(SaveError::ChecksumMismatch),
                "byte {at}"
            );
        }
        assert_eq!(
            decode(&bytes[..bytes.len() - 1]),
            Err(SaveError::ChecksumMismatch)
        );
        assert_eq!(decode(&bytes[..13]), Err(SaveError::Truncated));

        assert_eq!(
            SaveFile::<String>::decode(&bytes, "another-game"),
            Err(SaveError::WrongGame {
                expected: "another-game".into(),
                found: "between-frontiers".into()
            })
        );

        // A consistent checksum over inconsistent lengths is still refused.
        let mut lying = bytes.clone();
        lying[12] = lying[12].wrapping_add(200);
        assert!(matches!(
            decode(&reseal(lying)),
            Err(SaveError::Truncated | SaveError::Malformed(_))
        ));
        let mut trailing = bytes[..bytes.len() - 32].to_vec();
        trailing.push(0);
        trailing.extend_from_slice(&[0; 32]);
        assert!(matches!(
            decode(&reseal(trailing)),
            Err(SaveError::Malformed(_))
        ));
    }
}
