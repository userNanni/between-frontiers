//! Deterministic simulation engine core: tick scheduler, fixed-point arithmetic, seeded RNG,
//! state hash, save container.
//!
//! The core has no window, no I/O and no third-party engine. The graphical client, the multiplayer
//! server and the benchmark are three consumers of the same library.

pub mod calendar;
