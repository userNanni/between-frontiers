//! The simulation runs off the interface thread, on the Laplace-Schelling engine. The interface
//! reads a published snapshot and sends commands; it never waits for a tick, only for a save.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bf_sim::game::{GameCommand, Session};
use bf_sim::research::Stage;
use laplace_schelling::calendar::Date;

use crate::live::LiveFigures;

/// Days per second at each speed, 1 to 5.
const DAYS_PER_SECOND: [f64; 5] = [1.0, 3.0, 8.0, 20.0, 60.0];
/// Most days the simulation owes the clock: past this it stops catching up.
const MAX_DEBT_DAYS: f64 = 32.0;
/// Longest run of ticks between two published snapshots. The command queue is read between
/// every two ticks, so a pause waits at most for the tick under way.
const BATCH_BUDGET: Duration = Duration::from_millis(50);

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
    /// Country figures for the HUD, read on the first snapshot and then once a month; `None`
    /// without an economy.
    pub figures: Option<Arc<LiveFigures>>,
    /// The player's treasury, currency and debt, read at every publication; `None` without a
    /// player or an economy.
    pub government: Option<Arc<crate::government::GovernmentView>>,
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

/// The month's figures, read again only when the month changed since `last`.
fn figures_of(session: &Session, last: &Option<Arc<LiveFigures>>) -> Option<Arc<LiveFigures>> {
    let date = session.clock().date;
    match last {
        Some(f) if f.month == (date.year, date.month) => Some(Arc::clone(f)),
        _ => crate::live::read(session.world(), date).map(Arc::new),
    }
}

fn snapshot_of(
    session: &Session,
    paused: bool,
    speed: u8,
    hash: String,
    figures: Option<Arc<LiveFigures>>,
) -> Snapshot {
    Snapshot {
        date: session.clock().date,
        tick: session.clock().tick,
        paused,
        speed,
        hash,
        player: session.world().player_tag().map(str::to_owned),
        research: research_view(session),
        local: Arc::new(local_view(session)),
        figures,
        government: crate::government::read(session.world(), session.clock().date).map(Arc::new),
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
        let figures = figures_of(&session, &None);
        let hash = short(&session.state_hash());
        let snapshot = Arc::new(Mutex::new(snapshot_of(&session, true, 2, hash, figures)));
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
        self.save_later(path, label)
            .recv_timeout(Duration::from_secs(30))
            .map_err(|e| e.to_string())?
    }

    /// Asks the simulation thread to write a save; the answer comes on the returned channel, so
    /// the interface keeps drawing meanwhile.
    pub fn save_later(&self, path: PathBuf, label: String) -> Receiver<Result<(), String>> {
        let (reply, answer) = channel();
        self.send(Command::Save { path, label, reply });
        answer
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

/// The loop's state that commands change.
struct Control {
    paused: bool,
    speed: u8,
    /// A command changed what the interface shows.
    changed: bool,
}

/// Applies one command.
fn apply(session: &mut Session, ctl: &mut Control, command: Command) {
    match command {
        Command::SetPaused(p) => {
            ctl.paused = p;
            ctl.changed = true;
        }
        Command::SetSpeed(s) => {
            ctl.speed = s.clamp(1, 5);
            ctl.changed = true;
        }
        // Applies at the start of the next tick and enters the history then.
        Command::Issue(c) => session.submit(c),
        Command::Save { path, label, reply } => {
            let _ = reply.send(write_save(session, &path, &label));
        }
    }
}

/// Applies every command waiting; `false` when the interface is gone.
fn drain(session: &mut Session, ctl: &mut Control, rx: &Receiver<Command>) -> bool {
    loop {
        match rx.try_recv() {
            Ok(c) => apply(session, ctl, c),
            Err(std::sync::mpsc::TryRecvError::Empty) => return true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => return false,
        }
    }
}

fn run(
    mut session: Session,
    rx: Receiver<Command>,
    shared: Arc<Mutex<Snapshot>>,
    repaint: impl Fn(),
) {
    let mut ctl = Control {
        paused: true,
        speed: 2,
        changed: false,
    };
    // The hash shown, and the tick it belongs to: computed only when a snapshot shows a new
    // one (on pause, and on the first of the month).
    let (mut hash, mut figures) = {
        let s = shared.lock().unwrap_or_else(|e| e.into_inner());
        (s.hash.clone(), s.figures.clone())
    };
    let mut hash_tick = session.clock().tick;
    let mut debt = 0.0f64;
    let mut last = Instant::now();
    loop {
        let timeout = if ctl.paused {
            Duration::from_millis(250)
        } else {
            Duration::from_millis(8)
        };
        ctl.changed = false;
        match rx.recv_timeout(timeout) {
            Ok(c) => apply(&mut session, &mut ctl, c),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
        }
        if !drain(&mut session, &mut ctl, &rx) {
            return;
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        let mut month_turned = false;
        if ctl.paused {
            debt = 0.0;
        } else {
            debt = (debt + dt * DAYS_PER_SECOND[(ctl.speed - 1) as usize]).min(MAX_DEBT_DAYS);
            let batch = Instant::now();
            while debt >= 1.0 && !ctl.paused && batch.elapsed() < BATCH_BUDGET {
                session.sim.step();
                debt -= 1.0;
                ctl.changed = true;
                month_turned |= session.clock().date.day == 1;
                if !drain(&mut session, &mut ctl, &rx) {
                    return;
                }
            }
        }
        if ctl.changed {
            let tick = session.clock().tick;
            if (ctl.paused || month_turned) && tick != hash_tick {
                hash = short(&session.state_hash());
                hash_tick = tick;
            }
            figures = figures_of(&session, &figures);
            let snap = snapshot_of(
                &session,
                ctl.paused,
                ctl.speed,
                hash.clone(),
                figures.clone(),
            );
            *shared.lock().unwrap_or_else(|e| e.into_inner()) = snap;
            repaint();
        }
    }
}
