//! The tick scheduler.
//!
//! # Model
//!
//! A tick runs the systems that are due on its date (see [`Cadence`]). Each system is split in
//! two halves:
//!
//! - [`System::compute`] gets the world read-only and returns a private changeset. Computes run
//!   in parallel on a `rayon` pool, and a system may use `rayon` inside its compute too.
//! - [`System::apply`] gets the world mutably and writes the changeset in. Applies run serially.
//!
//! The result is defined as if the due systems ran one at a time, compute then apply, in the
//! order they were added. To get parallelism without changing that result, the due systems are
//! grouped into **waves** from their declared [`Access`]: a system goes one wave after the last
//! earlier system it conflicts with (one writes what the other reads or writes). Systems in a
//! wave compute in parallel against the same world, then apply in declared order. Conflicting
//! systems keep their declared order; non-conflicting ones commute, so the result is the
//! sequential one.
//!
//! # Determinism
//!
//! The wave plan depends only on the declarations and the date, never on the thread count or
//! on timing, and every system draws randomness from its own stream ([`TickCtx::rng`]). The
//! world after a tick is therefore the same with 1 thread and with N. The declarations are
//! trusted, not enforced: a system that reads something it did not declare sees the world as
//! of the start of its wave. That is a logic bug, but a deterministic one.
//!
//! Inside a compute, collect parallel results in index order (`par_iter().map().collect()`)
//! and fold them serially; never reduce with a non-associative operation such as saturating
//! [`Fixed`](crate::fixed::Fixed) addition across threads.

use std::any::Any;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::calendar::{Date, Weekday};
use crate::rng::Rng;

/// A piece of world state that systems read or write (a table, a group of tables). The game
/// assigns the numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub u16);

/// What a system reads and writes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Access {
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
}

fn insert_sorted(set: &mut Vec<ResourceId>, resource: ResourceId) {
    if let Err(at) = set.binary_search(&resource) {
        set.insert(at, resource);
    }
}

fn intersects(a: &[ResourceId], b: &[ResourceId]) -> bool {
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            core::cmp::Ordering::Less => i += 1,
            core::cmp::Ordering::Greater => j += 1,
            core::cmp::Ordering::Equal => return true,
        }
    }
    false
}

impl Access {
    /// Nothing read, nothing written.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a read.
    pub fn read(mut self, resource: ResourceId) -> Self {
        insert_sorted(&mut self.reads, resource);
        self
    }

    /// Adds a write. A system that also uses the old value should declare the read as well.
    pub fn write(mut self, resource: ResourceId) -> Self {
        insert_sorted(&mut self.writes, resource);
        self
    }

    /// Declared reads, sorted.
    pub fn reads(&self) -> &[ResourceId] {
        &self.reads
    }

    /// Declared writes, sorted.
    pub fn writes(&self) -> &[ResourceId] {
        &self.writes
    }

    /// Whether running the two in the same wave could differ from running them in order.
    pub fn conflicts_with(&self, other: &Access) -> bool {
        intersects(&self.writes, &other.reads)
            || intersects(&self.writes, &other.writes)
            || intersects(&self.reads, &other.writes)
    }
}

/// When a system runs, aligned to the calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cadence {
    /// Every tick.
    Daily,
    /// Every week on this weekday.
    Weekly(Weekday),
    /// Every month on this day; past the end of a short month, on its last day.
    Monthly {
        /// Day of the month, 1 to 31.
        day: u8,
    },
    /// Every year on this date; February 29 falls on February 28 in common years.
    Yearly {
        /// Month, 1 to 12.
        month: u8,
        /// Day of the month.
        day: u8,
    },
}

impl Cadence {
    /// Whether a system with this cadence runs on `date`.
    pub fn is_due(self, date: Date) -> bool {
        let clamp = |day: u8, month: u8| day.min(Date::days_in_month(date.year, month));
        match self {
            Cadence::Daily => true,
            Cadence::Weekly(weekday) => date.weekday() == weekday,
            Cadence::Monthly { day } => date.day == clamp(day, date.month),
            Cadence::Yearly { month, day } => date.month == month && date.day == clamp(day, month),
        }
    }
}

/// Position in time: the tick counter and its date. Tick 0 is the start date.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clock {
    /// Ticks run since the start.
    pub tick: u64,
    /// The date of the next tick to run.
    pub date: Date,
}

impl Clock {
    /// Tick 0 on `date`.
    pub const fn start(date: Date) -> Self {
        Self { tick: 0, date }
    }

    /// One day later.
    pub const fn advance(self) -> Self {
        Self {
            tick: self.tick + 1,
            date: self.date.next_day(),
        }
    }
}

/// What a system or command knows about the current tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickCtx {
    /// Tick number.
    pub tick: u64,
    /// The tick's date.
    pub date: Date,
    /// World seed.
    pub seed: u64,
    /// Random stream of the running system, or of the command being applied.
    pub stream: u64,
}

impl TickCtx {
    /// The generator for `entity` in this tick and stream. Draws for different entities are
    /// independent of each other and of the order the entities are visited in.
    pub fn rng(&self, entity: u64) -> Rng {
        Rng::stream(self.seed, self.tick, self.stream, entity)
    }
}

/// A stable 64-bit stream id from a name (FNV-1a), so a system's randomness does not change
/// when other systems are added or reordered.
pub const fn stream_id(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    hash
}

/// One simulation system over the world type `W`.
///
/// A system keeps no state of its own: whatever must survive a tick lives in `W`, so it is
/// hashed and saved.
pub trait System<W>: Send + Sync + 'static {
    /// The private changeset `compute` produces for `apply`.
    type Changes: Send + 'static;

    /// Unique name; also seeds the system's random stream.
    fn name(&self) -> &'static str;

    /// When it runs.
    fn cadence(&self) -> Cadence;

    /// What it reads and writes.
    fn access(&self) -> Access;

    /// Reads the world and computes the changes. Runs in parallel with the rest of its wave.
    fn compute(&self, ctx: &TickCtx, world: &W) -> Self::Changes;

    /// Writes the changes. Runs serially, in declared order.
    fn apply(&self, ctx: &TickCtx, changes: Self::Changes, world: &mut W);
}

type AnyChanges = Box<dyn Any + Send>;

trait ErasedSystem<W>: Send + Sync {
    fn compute(&self, ctx: &TickCtx, world: &W) -> AnyChanges;
    fn apply(&self, ctx: &TickCtx, changes: AnyChanges, world: &mut W);
}

impl<W, S: System<W>> ErasedSystem<W> for S {
    fn compute(&self, ctx: &TickCtx, world: &W) -> AnyChanges {
        Box::new(System::compute(self, ctx, world))
    }

    fn apply(&self, ctx: &TickCtx, changes: AnyChanges, world: &mut W) {
        let changes = changes
            .downcast::<S::Changes>()
            .expect("changeset type matches its system");
        System::apply(self, ctx, *changes, world);
    }
}

/// Accumulated cost of one system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemStats {
    /// System name.
    pub name: &'static str,
    /// Its cadence.
    pub cadence: Cadence,
    /// Ticks it ran on.
    pub runs: u64,
    /// Total compute time.
    pub compute: Duration,
    /// Total apply time.
    pub apply: Duration,
    /// Slowest single run, compute plus apply.
    pub slowest: Duration,
}

/// What one tick of the schedule did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickStats {
    /// Wall time of the tick.
    pub duration: Duration,
    /// Systems that ran.
    pub systems: u32,
    /// Waves they ran in.
    pub waves: u32,
}

struct Entry<W> {
    system: Box<dyn ErasedSystem<W>>,
    access: Access,
    stream: u64,
    stats: SystemStats,
}

/// Indices of the systems in each wave, in declared order.
type Plan = Vec<Vec<usize>>;

/// The ordered set of systems and the thread pool that runs them.
pub struct Schedule<W> {
    entries: Vec<Entry<W>>,
    pool: rayon::ThreadPool,
    plans: BTreeMap<Vec<usize>, Plan>,
}

impl<W: Send + Sync + 'static> Schedule<W> {
    /// An empty schedule on a pool of `threads` threads (`0` means one per available core).
    ///
    /// # Panics
    /// When the thread pool cannot be created.
    pub fn new(threads: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(|i| format!("laplace-schelling-{i}"))
            .build()
            .expect("failed to build the simulation thread pool");
        Self {
            entries: Vec::new(),
            pool,
            plans: BTreeMap::new(),
        }
    }

    /// Appends a system; the order of `add` calls is the declared order.
    ///
    /// # Panics
    /// When another system already has the same name.
    pub fn add<S: System<W>>(&mut self, system: S) -> &mut Self {
        let name = system.name();
        assert!(
            self.entries.iter().all(|e| e.stats.name != name),
            "duplicate system name {name:?}"
        );
        let stats = SystemStats {
            name,
            cadence: system.cadence(),
            runs: 0,
            compute: Duration::ZERO,
            apply: Duration::ZERO,
            slowest: Duration::ZERO,
        };
        self.entries.push(Entry {
            access: system.access(),
            stream: stream_id(name),
            system: Box::new(system),
            stats,
        });
        self.plans.clear();
        self
    }

    /// Threads in the pool.
    pub fn threads(&self) -> usize {
        self.pool.current_num_threads()
    }

    /// Runs `f` inside the pool, so its `rayon` calls use the simulation's threads.
    pub fn install<R: Send>(&self, f: impl FnOnce() -> R + Send) -> R {
        self.pool.install(f)
    }

    /// Accumulated cost per system, in declared order.
    pub fn stats(&self) -> Vec<SystemStats> {
        self.entries.iter().map(|e| e.stats.clone()).collect()
    }

    /// The waves for `date`, by system name.
    pub fn plan_for(&self, date: Date) -> Vec<Vec<&'static str>> {
        let due = self.due(date);
        build_plan(&self.entries, &due)
            .into_iter()
            .map(|wave| {
                wave.into_iter()
                    .map(|i| self.entries[i].stats.name)
                    .collect()
            })
            .collect()
    }

    fn due(&self, date: Date) -> Vec<usize> {
        (0..self.entries.len())
            .filter(|&i| self.entries[i].stats.cadence.is_due(date))
            .collect()
    }

    /// Runs the systems due at `clock` on `world`.
    pub fn run(&mut self, clock: Clock, seed: u64, world: &mut W) -> TickStats {
        let started = Instant::now();
        let due = self.due(clock.date);
        let plan = self
            .plans
            .entry(due)
            .or_insert_with_key(|due| build_plan(&self.entries, due));
        let entries = &mut self.entries;
        let ctx_for = |entry: &Entry<W>| TickCtx {
            tick: clock.tick,
            date: clock.date,
            seed,
            stream: entry.stream,
        };
        let mut systems = 0;
        self.pool.install(|| {
            for wave in plan.iter() {
                let shared: &W = world;
                let entries_ref: &[Entry<W>] = entries;
                let computed: Vec<(AnyChanges, Duration)> = if wave.len() == 1 {
                    let entry = &entries_ref[wave[0]];
                    let at = Instant::now();
                    vec![(entry.system.compute(&ctx_for(entry), shared), at.elapsed())]
                } else {
                    wave.par_iter()
                        .map(|&i| {
                            let entry = &entries_ref[i];
                            let at = Instant::now();
                            let changes = entry.system.compute(&ctx_for(entry), shared);
                            (changes, at.elapsed())
                        })
                        .collect()
                };
                for (&i, (changes, compute)) in wave.iter().zip(computed) {
                    let entry = &mut entries[i];
                    let at = Instant::now();
                    entry.system.apply(&ctx_for(entry), changes, world);
                    let apply = at.elapsed();
                    let stats = &mut entry.stats;
                    stats.runs += 1;
                    stats.compute += compute;
                    stats.apply += apply;
                    stats.slowest = stats.slowest.max(compute + apply);
                    systems += 1;
                }
            }
        });
        TickStats {
            duration: started.elapsed(),
            systems,
            waves: plan.len() as u32,
        }
    }
}

/// Wave of each due system = 1 + the latest wave of an earlier due system it conflicts with.
fn build_plan<W>(entries: &[Entry<W>], due: &[usize]) -> Plan {
    let mut wave_of: Vec<usize> = Vec::with_capacity(due.len());
    let mut plan: Plan = Vec::new();
    for (k, &i) in due.iter().enumerate() {
        let wave = due[..k]
            .iter()
            .zip(&wave_of)
            .filter(|&(&j, _)| entries[j].access.conflicts_with(&entries[i].access))
            .map(|(_, &w)| w + 1)
            .max()
            .unwrap_or(0);
        wave_of.push(wave);
        if plan.len() <= wave {
            plan.resize_with(wave + 1, Vec::new);
        }
        plan[wave].push(i);
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Fixed;
    use crate::hash::{HashState, StateHasher, hash_of};
    use crate::ids::Table;

    struct Cell;

    const A: ResourceId = ResourceId(0);
    const B: ResourceId = ResourceId(1);
    const C: ResourceId = ResourceId(2);

    #[derive(Clone)]
    struct World {
        a: Table<Cell, Fixed>,
        b: Table<Cell, Fixed>,
        c: Table<Cell, Fixed>,
    }

    impl HashState for World {
        fn hash_state(&self, h: &mut StateHasher) {
            h.section("a");
            self.a.hash_state(h);
            h.section("b");
            self.b.hash_state(h);
            h.section("c");
            self.c.hash_state(h);
        }
    }

    /// Adds a random amount to every cell of `target`, scaled by `source`.
    struct Noise {
        name: &'static str,
        cadence: Cadence,
        source: ResourceId,
        target: ResourceId,
    }

    fn column(world: &World, id: ResourceId) -> &Table<Cell, Fixed> {
        match id.0 {
            0 => &world.a,
            1 => &world.b,
            _ => &world.c,
        }
    }

    impl System<World> for Noise {
        type Changes = Vec<Fixed>;

        fn name(&self) -> &'static str {
            self.name
        }

        fn cadence(&self) -> Cadence {
            self.cadence
        }

        fn access(&self) -> Access {
            Access::new()
                .read(self.source)
                .read(self.target)
                .write(self.target)
        }

        fn compute(&self, ctx: &TickCtx, world: &World) -> Vec<Fixed> {
            let source = column(world, self.source).as_slice();
            source
                .par_iter()
                .enumerate()
                .map(|(i, value)| {
                    let mut rng = ctx.rng(i as u64);
                    rng.unit().mul_ratio(1, 10) + value.mul_ratio(1, 1000)
                })
                .collect()
        }

        fn apply(&self, _ctx: &TickCtx, changes: Vec<Fixed>, world: &mut World) {
            let target = match self.target.0 {
                0 => &mut world.a,
                1 => &mut world.b,
                _ => &mut world.c,
            };
            for (value, delta) in target.as_mut_slice().iter_mut().zip(changes) {
                *value += delta;
            }
        }
    }

    fn schedule(threads: usize) -> Schedule<World> {
        let mut schedule = Schedule::new(threads);
        schedule
            .add(Noise {
                name: "a-from-b",
                cadence: Cadence::Daily,
                source: B,
                target: A,
            })
            .add(Noise {
                name: "c-from-c",
                cadence: Cadence::Daily,
                source: C,
                target: C,
            })
            .add(Noise {
                name: "b-from-a",
                cadence: Cadence::Weekly(Weekday::Monday),
                source: A,
                target: B,
            })
            .add(Noise {
                name: "c-from-a",
                cadence: Cadence::Monthly { day: 31 },
                source: A,
                target: C,
            });
        schedule
    }

    fn run(threads: usize, days: u32) -> (Vec<crate::hash::StateHash>, Schedule<World>) {
        let mut world = World {
            a: Table::from_fn(5_000, |_| Fixed::ONE),
            b: Table::from_fn(5_000, |_| Fixed::ZERO),
            c: Table::from_fn(5_000, |_| Fixed::ONE),
        };
        let mut schedule = schedule(threads);
        let mut clock = Clock::start(Date::new(1925, 1, 1));
        let mut hashes = Vec::new();
        for _ in 0..days {
            schedule.run(clock, 99, &mut world);
            hashes.push(hash_of(&world));
            clock = clock.advance();
        }
        (hashes, schedule)
    }

    #[test]
    fn same_result_with_one_thread_and_many() {
        let (one, _) = run(1, 120);
        let (many, schedule) = run(8, 120);
        assert_eq!(schedule.threads(), 8);
        assert_eq!(one, many);
        let stats = schedule.stats();
        assert_eq!(stats[0].runs, 120);
        assert_eq!(stats[2].runs, 17);
        assert_eq!(stats[3].runs, 4, "Jan 31, Feb 28, Mar 31, Apr 30");
    }

    #[test]
    fn waves_follow_conflicts_in_declared_order() {
        let schedule = schedule(2);
        // 1925-01-05 is a Monday: a-from-b and c-from-c are independent; b-from-a reads A.
        assert_eq!(
            schedule.plan_for(Date::new(1925, 1, 5)),
            vec![vec!["a-from-b", "c-from-c"], vec!["b-from-a"]]
        );
        assert_eq!(
            schedule.plan_for(Date::new(1925, 1, 6)),
            vec![vec!["a-from-b", "c-from-c"]]
        );
        // On Jan 31 c-from-a waits for both a-from-b (A) and c-from-c (C).
        assert_eq!(
            schedule.plan_for(Date::new(1925, 1, 31)),
            vec![vec!["a-from-b", "c-from-c"], vec!["c-from-a"]]
        );
    }

    #[test]
    fn cadences_align_to_the_calendar() {
        assert!(Cadence::Monthly { day: 31 }.is_due(Date::new(1925, 2, 28)));
        assert!(!Cadence::Monthly { day: 31 }.is_due(Date::new(1928, 2, 28)));
        assert!(Cadence::Monthly { day: 31 }.is_due(Date::new(1928, 2, 29)));
        assert!(Cadence::Yearly { month: 2, day: 29 }.is_due(Date::new(1925, 2, 28)));
        assert!(Cadence::Yearly { month: 1, day: 1 }.is_due(Date::new(1926, 1, 1)));
        assert!(Cadence::Weekly(Weekday::Thursday).is_due(Date::new(1925, 1, 1)));
        assert!(!Cadence::Weekly(Weekday::Friday).is_due(Date::new(1925, 1, 1)));
    }

    #[test]
    fn access_conflicts() {
        let reader = Access::new().read(A);
        let writer = Access::new().write(A);
        let other = Access::new().read(B).write(C);
        assert!(reader.conflicts_with(&writer));
        assert!(writer.conflicts_with(&reader));
        assert!(writer.conflicts_with(&writer));
        assert!(!reader.conflicts_with(&reader));
        assert!(!other.conflicts_with(&writer));
    }

    #[test]
    #[should_panic(expected = "duplicate system name")]
    fn names_are_unique() {
        let mut schedule = schedule(1);
        schedule.add(Noise {
            name: "a-from-b",
            cadence: Cadence::Daily,
            source: A,
            target: A,
        });
    }
}
