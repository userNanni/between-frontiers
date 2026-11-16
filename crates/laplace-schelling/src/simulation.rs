//! The simulation driver: world, clock, schedule and command queue in one place.

use crate::calendar::Date;
use crate::command::{Command, CommandLog, Issuer, Stamped};
use crate::schedule::{Clock, Schedule, TickCtx, TickStats, stream_id};

/// Base stream id for commands; each command gets `COMMAND_STREAM ^ seq`, its own stream.
const COMMAND_STREAM: u64 = stream_id("laplace-schelling/command");

/// A running game: world `W`, commands `C`.
///
/// Each [`step`](Simulation::step) applies the commands stamped for the current tick, runs the
/// due systems and advances the clock by one day.
pub struct Simulation<W, C> {
    world: W,
    clock: Clock,
    seed: u64,
    schedule: Schedule<W>,
    pending: Vec<Stamped<C>>,
    log: CommandLog<C>,
    next_seq: u64,
}

impl<W: Send + Sync + 'static, C: Command<W>> Simulation<W, C> {
    /// A new game at tick 0 on `start`.
    pub fn new(world: W, seed: u64, start: Date, schedule: Schedule<W>) -> Self {
        Self::resume(world, seed, Clock::start(start), schedule)
    }

    /// Continues a game from a snapshot taken at `clock`. Feed the saved log back with
    /// [`replay`](Simulation::replay) to reach the same state as the original run.
    pub fn resume(world: W, seed: u64, clock: Clock, schedule: Schedule<W>) -> Self {
        Self {
            world,
            clock,
            seed,
            schedule,
            pending: Vec::new(),
            log: CommandLog::new(),
            next_seq: 0,
        }
    }

    /// The world.
    pub fn world(&self) -> &W {
        &self.world
    }

    /// The world, mutably, for scenario setup and tools. Changes made here are not commands:
    /// they are not logged and a replay will not reproduce them.
    pub fn world_mut(&mut self) -> &mut W {
        &mut self.world
    }

    /// Tick and date of the next tick.
    pub fn clock(&self) -> Clock {
        self.clock
    }

    /// World seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The schedule (for its statistics).
    pub fn schedule(&self) -> &Schedule<W> {
        &self.schedule
    }

    /// Commands applied so far.
    pub fn log(&self) -> &CommandLog<C> {
        &self.log
    }

    /// Commands queued for ticks not yet run, in the order they will apply. A save must keep
    /// them: a command issued while paused applies only at the next tick.
    pub fn pending(&self) -> &[Stamped<C>] {
        &self.pending
    }

    /// Queues a command for the start of the next tick and returns its stamp.
    pub fn submit(&mut self, issuer: Issuer, command: C) -> (u64, u64) {
        let stamped = Stamped {
            tick: self.clock.tick,
            issuer,
            seq: self.next_seq,
            command,
        };
        let key = (stamped.tick, stamped.seq);
        self.enqueue(stamped);
        key
    }

    /// Queues an already stamped command (from the network or a log).
    ///
    /// # Panics
    /// When it is stamped for a tick that already ran.
    pub fn enqueue(&mut self, stamped: Stamped<C>) {
        assert!(
            stamped.tick >= self.clock.tick,
            "command stamped for tick {} but the clock is at {}",
            stamped.tick,
            self.clock.tick
        );
        self.next_seq = self.next_seq.max(stamped.seq + 1);
        let at = self.pending.partition_point(|p| p.key() <= stamped.key());
        self.pending.insert(at, stamped);
    }

    /// Queues every logged command from the current tick on.
    pub fn replay(&mut self, log: &CommandLog<C>) {
        for stamped in log.since(self.clock.tick) {
            self.enqueue(stamped.clone());
        }
    }

    /// Runs one tick: due commands, then due systems, then the clock moves one day.
    pub fn step(&mut self) -> TickStats {
        let due = self.pending.partition_point(|p| p.tick <= self.clock.tick);
        for stamped in self.pending.drain(..due) {
            let ctx = TickCtx {
                tick: self.clock.tick,
                date: self.clock.date,
                seed: self.seed,
                stream: COMMAND_STREAM ^ stamped.seq,
            };
            stamped.command.apply(&ctx, &mut self.world);
            self.log.push(stamped);
        }
        let stats = self.schedule.run(self.clock, self.seed, &mut self.world);
        self.clock = self.clock.advance();
        stats
    }

    /// Steps until the clock reaches `date`.
    pub fn run_until(&mut self, date: Date) {
        while self.clock.date < date {
            self.step();
        }
    }

    /// The world, consuming the simulation.
    pub fn into_world(self) -> W {
        self.world
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::Simulation;
    use crate::calendar::Date;
    use crate::command::{Command, CommandLog};
    use crate::fixed::Fixed;
    use crate::hash::{HashState, StateHasher, hash_of};
    use crate::save::{SaveFile, SaveHeader, decode_snapshot, encode_snapshot};
    use crate::schedule::{Access, Cadence, ResourceId, Schedule, System, TickCtx};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Bank {
        balance: Vec<Fixed>,
    }

    impl HashState for Bank {
        fn hash_state(&self, h: &mut StateHasher) {
            self.balance.hash_state(h);
        }
    }

    struct Interest;

    impl System<Bank> for Interest {
        type Changes = Vec<Fixed>;
        fn name(&self) -> &'static str {
            "interest"
        }
        fn cadence(&self) -> Cadence {
            Cadence::Monthly { day: 1 }
        }
        fn access(&self) -> Access {
            Access::new().read(ResourceId(0)).write(ResourceId(0))
        }
        fn compute(&self, ctx: &TickCtx, bank: &Bank) -> Vec<Fixed> {
            bank.balance
                .iter()
                .enumerate()
                .map(|(i, b)| b.mul_ratio(5 + ctx.rng(i as u64).range(0, 5), 1000))
                .collect()
        }
        fn apply(&self, _ctx: &TickCtx, changes: Vec<Fixed>, bank: &mut Bank) {
            for (b, d) in bank.balance.iter_mut().zip(changes) {
                *b += d;
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    enum Order {
        Deposit { account: usize, amount: Fixed },
        Lottery { account: usize },
    }

    impl Command<Bank> for Order {
        fn apply(&self, ctx: &TickCtx, bank: &mut Bank) {
            match *self {
                Order::Deposit { account, amount } => {
                    if let Some(b) = bank.balance.get_mut(account) {
                        *b += amount;
                    }
                }
                Order::Lottery { account } => {
                    let prize = Fixed::from_int(ctx.rng(0).range(0, 1000));
                    if let Some(b) = bank.balance.get_mut(account) {
                        *b += prize;
                    }
                }
            }
        }
    }

    fn sim(threads: usize) -> Simulation<Bank, Order> {
        let mut schedule = Schedule::new(threads);
        schedule.add(Interest);
        let bank = Bank {
            balance: vec![Fixed::from_int(100); 4],
        };
        Simulation::new(bank, 7, Date::new(1925, 1, 1), schedule)
    }

    #[test]
    fn commands_apply_at_tick_start_in_issuer_order() {
        let mut sim = sim(1);
        sim.step();
        // Submitted out of issuer order; issuer 1 applies before issuer 2.
        sim.submit(
            2,
            Order::Deposit {
                account: 9,
                amount: Fixed::ONE,
            },
        );
        sim.submit(1, Order::Lottery { account: 0 });
        assert_eq!(sim.log().len(), 0, "nothing applies before the tick");
        sim.step();
        let issuers: Vec<_> = sim.log().entries().iter().map(|e| e.issuer).collect();
        assert_eq!(issuers, [1, 2], "invalid commands are logged too");
        assert!(sim.log().entries().iter().all(|e| e.tick == 1));
    }

    #[test]
    fn snapshot_plus_log_replays_the_same_state() {
        let mut original = sim(4);
        original.run_until(Date::new(1925, 3, 1));
        let snapshot_clock = original.clock();
        let snapshot = encode_snapshot(original.world()).unwrap();
        let header = SaveHeader::new(
            "test",
            "0",
            original.seed(),
            snapshot_clock,
            hash_of(original.world()),
        );
        let mut log_since_snapshot = CommandLog::new();
        for day in 0..200u64 {
            if day % 17 == 0 {
                original.submit(
                    (day % 3) as u32,
                    Order::Deposit {
                        account: (day % 4) as usize,
                        amount: Fixed::from_raw(1234),
                    },
                );
            }
            if day % 29 == 0 {
                original.submit(5, Order::Lottery { account: 2 });
            }
            original.step();
        }
        for entry in original.log().entries() {
            log_since_snapshot.push(entry.clone());
        }
        let save = SaveFile {
            header,
            snapshot,
            log: log_since_snapshot,
        };
        let bytes = save.encode().unwrap();

        let loaded = SaveFile::<Order>::decode(&bytes, "test").unwrap();
        let world: Bank = decode_snapshot(&loaded.snapshot).unwrap();
        assert_eq!(hash_of(&world), loaded.header.state_hash);
        let mut schedule = Schedule::new(1);
        schedule.add(Interest);
        let mut replay = Simulation::<Bank, Order>::resume(
            world,
            loaded.header.seed,
            loaded.header.clock,
            schedule,
        );
        replay.replay(&loaded.log);
        while replay.clock() != original.clock() {
            replay.step();
        }
        assert_eq!(hash_of(replay.world()), hash_of(original.world()));
        assert_eq!(replay.log(), original.log());
    }
}
