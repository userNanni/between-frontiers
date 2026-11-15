//! Player and AI commands as data.
//!
//! Nothing outside the tick mutates the world. A player click or an AI decision becomes a
//! command value, stamped with the tick it takes effect on, applied at the start of that tick
//! before any system runs, and appended to the [`CommandLog`]. A snapshot plus the log replays
//! the game exactly, which is what saves, multiplayer lockstep and bug reports rely on.
//!
//! Commands due on the same tick apply in `(issuer, seq)` order, so the result does not depend
//! on the order in which they arrived.

use serde::{Deserialize, Serialize};

use crate::schedule::TickCtx;

/// A command that can be applied to the world `W`.
///
/// `apply` must be deterministic: it may read the world and draw from `ctx.rng(..)`, nothing
/// else. An invalid command (the target no longer exists, the treasury is empty) is a no-op,
/// not a panic; it still goes into the log, because the rejection is part of the history.
pub trait Command<W>: Clone + Send + 'static {
    /// Applies the command at the start of tick `ctx.tick`.
    fn apply(&self, ctx: &TickCtx, world: &mut W);
}

/// Who issued a command: a player slot, an AI country, a script. Only its order matters to
/// the engine.
pub type Issuer = u32;

/// A command with the tick it applies on and its place in the order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamped<C> {
    /// The tick at whose start it applies.
    pub tick: u64,
    /// The issuer.
    pub issuer: Issuer,
    /// Sequence number, unique within the game; ties between issuers on a tick never happen.
    pub seq: u64,
    /// The command.
    pub command: C,
}

impl<C> Stamped<C> {
    /// The ordering key: `(tick, issuer, seq)`.
    pub fn key(&self) -> (u64, Issuer, u64) {
        (self.tick, self.issuer, self.seq)
    }
}

/// Every applied command, in application order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandLog<C> {
    entries: Vec<Stamped<C>>,
}

impl<C> Default for CommandLog<C> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<C> CommandLog<C> {
    /// An empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an applied command.
    ///
    /// # Panics
    /// When it would come before the last entry: the log is in application order.
    pub fn push(&mut self, entry: Stamped<C>) {
        if let Some(last) = self.entries.last() {
            assert!(
                last.key() < entry.key(),
                "command log entries must be in (tick, issuer, seq) order"
            );
        }
        self.entries.push(entry);
    }

    /// All entries.
    pub fn entries(&self) -> &[Stamped<C>] {
        &self.entries
    }

    /// Entries applied on `tick`.
    pub fn at_tick(&self, tick: u64) -> &[Stamped<C>] {
        let start = self.entries.partition_point(|e| e.tick < tick);
        let end = self.entries.partition_point(|e| e.tick <= tick);
        &self.entries[start..end]
    }

    /// Entries applied on `tick` or later (what a snapshot taken before `tick` needs).
    pub fn since(&self, tick: u64) -> &[Stamped<C>] {
        let start = self.entries.partition_point(|e| e.tick < tick);
        &self.entries[start..]
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandLog, Stamped};

    fn entry(tick: u64, issuer: u32, seq: u64) -> Stamped<()> {
        Stamped {
            tick,
            issuer,
            seq,
            command: (),
        }
    }

    #[test]
    fn log_slices_by_tick() {
        let mut log = CommandLog::new();
        log.push(entry(1, 0, 0));
        log.push(entry(3, 0, 2));
        log.push(entry(3, 1, 1));
        log.push(entry(5, 0, 3));
        assert_eq!(log.at_tick(3).len(), 2);
        assert!(log.at_tick(2).is_empty());
        assert_eq!(log.since(4), &[entry(5, 0, 3)]);
        assert_eq!(log.since(0).len(), 4);
    }

    #[test]
    #[should_panic(expected = "order")]
    fn log_rejects_out_of_order_entries() {
        let mut log = CommandLog::new();
        log.push(entry(3, 0, 0));
        log.push(entry(2, 0, 1));
    }
}
