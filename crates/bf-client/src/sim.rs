//! The simulation runs off the interface thread, on the Laplace-Schelling engine. The interface
//! reads a published snapshot and sends commands; it never waits for a tick, only for a save.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bf_sim::game::{GameCommand, Session};
use bf_sim::research::Stage;
use laplace_schelling::calendar::Date;

/// Days per second at each speed, 1 to 5.
const DAYS_PER_SECOND: [f64; 5] = [1.0, 3.0, 8.0, 20.0, 60.0];
/// Most ticks run between two looks at the command queue, so the interface stays responsive.
const MAX_TICKS_PER_BATCH: u32 = 32;

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub date: Date,
    pub tick: u64,
    pub paused: bool,
    /// 1 to 5.
    pub speed: u8,
    /// Short state hash, refreshed when the game pauses and on the first of each month.
    pub hash: String,
    /// Tag the world says the player controls.
    pub player: Option<String>,
    /// Where the player's country stands in research; `None` without research data.
    pub research: Option<ResearchView>,
    /// Share of the demand for each local good (`Catalogue::local` order: electricity,
    /// transport) met in each map state (index = state id − 1), `-1` where no region runs.
    pub local: std::sync::Arc<Vec<[f32; 2]>>,
}

/// The player's research, copied out of the world for the interface.
#[derive(Clone, Debug, PartialEq)]
pub struct ResearchView {
    pub stages: Vec<Stage>,
    /// 0–1 toward the next stage.
    pub progress: Vec<f32>,
    pub focus: Option<u16>,
    /// Research effort per month.
    pub points: f32,
    /// Year each technology first reached each stage anywhere (0 = not yet).
    pub firsts: Vec<[i32; 5]>,
}

fn research_view(session: &Session) -> Option<ResearchView> {
    let w = session.world();
    let tag = w.player_tag()?;
    let c = w.countries.iter().position(|(_, c)| c.tag == tag)?;
    let r = &w.research;
    Some(ResearchView {
        stages: r.stage.get(c)?.clone(),
        progress: r
            .progress
            .get(c)?
            .iter()
            .map(|p| p.to_f64_lossy() as f32)
            .collect(),
        focus: r.focus.get(c).copied().flatten(),
        points: r.points.get(c).map_or(0.0, |p| p.to_f64_lossy() as f32),
        firsts: r.firsts.clone(),
    })
}

pub enum Command {
    SetPaused(bool),
    SetSpeed(u8),
    Issue(GameCommand),
    /// Writes the engine save to `path`; replies with the error text on failure.
    Save {
        path: PathBuf,
        label: String,
        reply: Sender<Result<(), String>>,
    },
}

pub struct SimHandle {
    commands: Sender<Command>,
    snapshot: Arc<Mutex<Snapshot>>,
}

fn short(hash: &laplace_schelling::StateHash) -> String {
    hash.to_string().chars().take(12).collect()
}

fn snapshot_of(session: &Session, paused: bool, speed: u8) -> Snapshot {
    Snapshot {
        date: session.clock().date,
        tick: session.clock().tick,
        paused,
        speed,
        hash: short(&session.state_hash()),
        player: session.world().player_tag().map(str::to_owned),
        research: research_view(session),
        local: Arc::new(local_view(session)),
    }
}

fn local_view(session: &Session) -> Vec<[f32; 2]> {
    let regions = &session.world().econ.regions;
    let n = regions
        .iter()
        .map(|r| usize::from(r.state))
        .max()
        .unwrap_or(0);
    let mut out = vec![[-1.0f32; 2]; n];
    for r in regions {
        let Some(slot) = usize::from(r.state)
            .checked_sub(1)
            .and_then(|i| out.get_mut(i))
        else {
            continue;
        };
        for (k, l) in r.local.iter().take(2).enumerate() {
            slot[k] = if l.demand > laplace_schelling::Fixed::ZERO {
                l.buy.to_f64_lossy() as f32
            } else {
                1.0
            };
        }
    }
    out
}

impl SimHandle {
    pub fn start(session: Session, repaint: impl Fn() + Send + 'static) -> Self {
        let snapshot = Arc::new(Mutex::new(snapshot_of(&session, true, 2)));
        let (tx, rx) = channel();
        let shared = Arc::clone(&snapshot);
        std::thread::Builder::new()
            .name("simulation".into())
            .spawn(move || run(session, rx, shared, repaint))
            .expect("spawn simulation thread");
        Self {
            commands: tx,
            snapshot,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    /// Asks the simulation thread to write a save and waits for it.
    pub fn save(&self, path: PathBuf, label: String) -> Result<(), String> {
        let (reply, answer) = channel();
        self.send(Command::Save { path, label, reply });
        answer
            .recv_timeout(Duration::from_secs(30))
            .map_err(|e| e.to_string())?
    }
}

fn write_save(session: &Session, path: &std::path::Path, label: &str) -> Result<(), String> {
    let bytes = session
        .save(env!("CARGO_PKG_VERSION"), label)
        .map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(tmp, path).map_err(|e| e.to_string())
}

fn run(
    mut session: Session,
    rx: Receiver<Command>,
    shared: Arc<Mutex<Snapshot>>,
    repaint: impl Fn(),
) {
    let (mut paused, mut speed) = (true, 2u8);
    let mut hash = short(&session.state_hash());
    let mut debt = 0.0f64;
    let mut last = Instant::now();
    loop {
        let timeout = if paused {
            Duration::from_millis(250)
        } else {
            Duration::from_millis(8)
        };
        let mut changed = false;
        match rx.recv_timeout(timeout) {
            Ok(Command::SetPaused(p)) => {
                paused = p;
                changed = true;
            }
            Ok(Command::SetSpeed(s)) => {
                speed = s.clamp(1, 5);
                changed = true;
            }
            Ok(Command::Issue(c)) => {
                // Applies at the start of the next tick and enters the history then.
                session.submit(c);
            }
            Ok(Command::Save { path, label, reply }) => {
                let _ = reply.send(write_save(&session, &path, &label));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        if paused {
            debt = 0.0;
        } else {
            debt = (debt + dt * DAYS_PER_SECOND[(speed - 1) as usize])
                .min(f64::from(MAX_TICKS_PER_BATCH));
            while debt >= 1.0 {
                session.sim.step();
                debt -= 1.0;
                changed = true;
                if session.clock().date.day == 1 {
                    hash = short(&session.state_hash());
                }
            }
        }
        if changed {
            if paused {
                hash = short(&session.state_hash());
            }
            let mut snap = snapshot_of(&session, paused, speed);
            snap.hash = hash.clone();
            *shared.lock().unwrap_or_else(|e| e.into_inner()) = snap;
            repaint();
        }
    }
}
