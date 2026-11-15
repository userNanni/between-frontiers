//! The simulation runs off the interface thread. The interface reads a published snapshot and
//! sends commands; it never waits for a tick.
//!
//! Phase 0 only advances the calendar; the systems arrive with the economy phase.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use laplace_schelling::calendar::Date;

/// Days per second at each speed, 1 to 5.
const DAYS_PER_SECOND: [f64; 5] = [1.0, 3.0, 8.0, 20.0, 60.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub date: Date,
    pub paused: bool,
    /// 1 to 5.
    pub speed: u8,
}

pub enum Command {
    SetPaused(bool),
    SetSpeed(u8),
}

pub struct SimHandle {
    commands: Sender<Command>,
    snapshot: Arc<Mutex<Snapshot>>,
}

impl SimHandle {
    pub fn start(date: Date, repaint: impl Fn() + Send + 'static) -> Self {
        let snapshot = Arc::new(Mutex::new(Snapshot {
            date,
            paused: true,
            speed: 2,
        }));
        let (tx, rx) = channel();
        let shared = Arc::clone(&snapshot);
        std::thread::Builder::new()
            .name("simulation".into())
            .spawn(move || run(rx, shared, repaint))
            .expect("spawn simulation thread");
        Self {
            commands: tx,
            snapshot,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        *self.snapshot.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }
}

fn run(rx: Receiver<Command>, shared: Arc<Mutex<Snapshot>>, repaint: impl Fn()) {
    let mut state = *shared.lock().unwrap_or_else(|e| e.into_inner());
    let mut debt = 0.0f64;
    let mut last = Instant::now();
    loop {
        let timeout = if state.paused {
            Duration::from_millis(250)
        } else {
            Duration::from_millis(8)
        };
        match rx.recv_timeout(timeout) {
            Ok(Command::SetPaused(p)) => state.paused = p,
            Ok(Command::SetSpeed(s)) => state.speed = s.clamp(1, 5),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        let mut changed = false;
        if !state.paused {
            debt += dt * DAYS_PER_SECOND[(state.speed - 1) as usize];
            while debt >= 1.0 {
                state.date = state.date.next_day();
                debt -= 1.0;
                changed = true;
            }
        } else {
            debt = 0.0;
        }
        let mut s = shared.lock().unwrap_or_else(|e| e.into_inner());
        if *s != state {
            *s = state;
            changed = true;
        }
        drop(s);
        if changed {
            repaint();
        }
    }
}
