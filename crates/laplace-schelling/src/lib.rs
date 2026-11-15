//! Deterministic simulation engine core: tick scheduler, fixed-point arithmetic, seeded RNG,
//! state hash, save container.
//!
//! The core has no window, no I/O and no third-party engine. The graphical client, the multiplayer
//! server and the benchmark are three consumers of the same library.
//!
//! | Module | What it gives |
//! |---|---|
//! | [`fixed`] | [`Fixed`], decimal fixed point on `i64`; the simulation has no floats |
//! | [`rng`] | [`Rng`], xoshiro256\*\* with streams keyed by seed, tick, system and entity |
//! | [`ids`] | [`Id<K>`] and the dense [`Table<K, V>`] columns keyed by it |
//! | [`schedule`] | [`System`], [`Schedule`]: parallel compute, serial apply, calendar cadences |
//! | [`command`] | [`Command`]s as data, applied at tick start and logged |
//! | [`simulation`] | [`Simulation`], the driver that ties world, clock, schedule and commands |
//! | [`hash`] | [`StateHash`], BLAKE3 over a canonical encoding |
//! | [`save`] | [`SaveFile`], snapshot plus command log with a checksum |
//! | [`calendar`] | [`Date`], one tick per day |
//!
//! Determinism contract: given the same scenario, seed and command log, every run on every
//! platform and thread count reaches the same [`StateHash`] on every tick.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod calendar;
pub mod command;
pub mod fixed;
pub mod hash;
pub mod ids;
pub mod rng;
pub mod save;
pub mod schedule;
pub mod simulation;

pub use calendar::{Date, Weekday};
pub use command::{Command, CommandLog, Stamped};
pub use fixed::Fixed;
pub use hash::{HashState, StateHash, StateHasher, hash_of};
pub use ids::{Id, Table};
pub use rng::Rng;
pub use save::{SaveError, SaveFile, SaveHeader};
pub use schedule::{Access, Cadence, Clock, ResourceId, Schedule, System, TickCtx};
pub use simulation::Simulation;
