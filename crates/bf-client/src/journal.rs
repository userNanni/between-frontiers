//! The journal: what the player's country is going through, read from the simulation once a
//! month, as entries the journal window lists and the side panel pins.
//!
//! An entry has the shape the history rails will need (`docs/history-rails.pt-BR.md`): the
//! conditions that open it, the milestones on its way, the conditions that close it and the
//! forces that push it. Today's entries come only from rules the simulation already runs: the
//! politics state, the treasury and the debt rules, the queue of works and research. Historical
//! entries and country agendas (phase 7) fill the same shape.
//!
//! Every text is a message id with arguments, resolved when drawn, so the snapshot stays the
//! same in every language. Every threshold an entry cites is read from the simulation's own
//! constants or rules, never typed again here.

use bf_sim::econ::Catalogue;
use bf_sim::econ::construction::{MIN_WEEKS, want};
use bf_sim::econ::money::{TARGET, deficit_room};
use bf_sim::econ::public::{RESERVE_WEEKS, austerity};
use bf_sim::econ::state::Market;
use bf_sim::game::GameWorld;
use bf_sim::politics::{Bound, GoalRef, Measure, Politics};
use bf_sim::research::{Stage, TechTree};
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// Works the side panel lists one by one.
pub const WORKS_LISTED: usize = 5;
/// Weeks the estimate looks ahead; beyond it a work shows as "more than" this.
pub const HORIZON_WEEKS: u32 = 520;
/// The journal opens the debt entry once the debt has taken this share of the deficit room
/// the rule allows a country without debt. A choice of the journal, not of the simulation: the
/// rule itself narrows the room gradually (`money::deficit_room`).
pub const DEBT_ROOM_SHOWN: f64 = 0.5;
/// Technologies on their way to industry the research entry names.
const PROTOTYPES_LISTED: usize = 3;

/// A value with its unit, formatted when drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    /// A share, 0–1, shown as a percentage.
    Share(f64),
    /// Already in percent (inflation, rates).
    Percent(f64),
    /// Real dollars at 1925 base prices.
    Money(f64),
    Weeks(f64),
    Count(i64),
    Year(i32),
    /// A plain number (a multiplier, a decaying count).
    Number(f64),
}

/// An argument of a message.
#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    Value(Value),
    /// Another message, translated as it is.
    Key(String),
    /// A map state, by id (1-based), named from the map.
    State(u16),
    Date(Date),
}

/// A message id and its arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub id: String,
    pub args: Vec<(&'static str, Arg)>,
}

/// A label with no arguments.
pub fn label(id: &str) -> Label {
    Label {
        id: id.to_owned(),
        args: Vec::new(),
    }
}

impl Label {
    pub fn arg(mut self, name: &'static str, arg: Arg) -> Self {
        self.args.push((name, arg));
        self
    }

    pub fn value(self, name: &'static str, v: Value) -> Self {
        self.arg(name, Arg::Value(v))
    }

    pub fn key(self, name: &'static str, id: String) -> Self {
        self.arg(name, Arg::Key(id))
    }
}

/// Which way a measured value is compared with its limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp {
    AtLeast,
    AtMost,
}

/// One condition: what is measured, the limit, and whether it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub label: Label,
    /// The value now; `None` when the simulation does not expose it.
    pub measured: Option<Value>,
    pub limit: Option<(Cmp, Value)>,
    /// `None` when it cannot be told from what the simulation exposes.
    pub holds: Option<bool>,
}

impl Condition {
    fn new(label: Label) -> Self {
        Self {
            label,
            measured: None,
            limit: None,
            holds: None,
        }
    }

    fn measured(mut self, v: Value) -> Self {
        self.measured = Some(v);
        self
    }

    fn limit(mut self, cmp: Cmp, v: Value) -> Self {
        self.limit = Some((cmp, v));
        self
    }

    fn holds(mut self, holds: bool) -> Self {
        self.holds = Some(holds);
        self
    }
}

/// A point on the way: reached or not, when, and how far along when it is the next one.
#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    pub label: Label,
    pub reached: bool,
    pub date: Option<Date>,
    /// 0–1 toward it.
    pub progress: Option<f32>,
}

/// A term that pushes the entry on (`helps`) or holds it back.
#[derive(Clone, Debug, PartialEq)]
pub struct Force {
    pub label: Label,
    pub value: Option<Value>,
    pub helps: bool,
}

/// What an entry is about; its id for pinning comes from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The construction queue as a whole.
    Works,
    /// The largest public work in the queue.
    LargeWork,
    Research,
    Fiscal(Fiscal),
    /// An active political movement, by kind.
    Movement(u16),
    /// A law asked for and not yet enacted.
    PendingLaw(u16),
    /// The law changes of the campaign.
    LawChanges,
    /// The armed forces, and the decision to mobilize them.
    Forces {
        mobilized: bool,
    },
    /// A country born in revolt, at war with its metropole (G6), by its tag.
    Revolt(String),
    /// The nuclear programme under way (N1); whether a test may be asked (N3: a warhead, no test
    /// yet, none asked).
    NuclearProgramme {
        can_test: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fiscal {
    /// No money for new public works: the rule of `public::public_works`.
    NoWorksBudget,
    Unfunded,
    PurchasesCut,
    DebtRoom,
    ShutOut,
    Inflation,
}

impl Kind {
    /// Stable id: what the campaign remembers when the entry is pinned.
    pub fn id(&self, politics: Option<&Politics>) -> String {
        match self {
            Kind::Works => "works".into(),
            Kind::LargeWork => "works-large".into(),
            Kind::Research => "research".into(),
            Kind::Fiscal(f) => format!("fiscal-{}", f.key()),
            Kind::Movement(m) => format!(
                "movement-{}",
                politics
                    .and_then(|p| p.movements.get(usize::from(*m)))
                    .map_or_else(|| m.to_string(), |k| k.key.clone())
            ),
            Kind::PendingLaw(l) => format!(
                "law-{}",
                politics
                    .and_then(|p| p.laws.get(usize::from(*l)))
                    .map_or_else(|| l.to_string(), |k| k.key.clone())
            ),
            Kind::LawChanges => "law-changes".into(),
            Kind::Forces { .. } => "forces".into(),
            Kind::Revolt(tag) => format!("revolt-{tag}"),
            Kind::NuclearProgramme { .. } => "nuclear-programme".into(),
        }
    }
}

impl Fiscal {
    fn key(self) -> &'static str {
        match self {
            Fiscal::NoWorksBudget => "works-budget",
            Fiscal::Unfunded => "unfunded",
            Fiscal::PurchasesCut => "purchases",
            Fiscal::DebtRoom => "debt-room",
            Fiscal::ShutOut => "shut-out",
            Fiscal::Inflation => "inflation",
        }
    }
}

/// How the entry reads at a glance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Warning,
    Danger,
}

/// Where the player goes to act on the entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Budget,
    Money,
    Laws,
    Research,
    Forces,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    pub kind: Kind,
    pub tone: Tone,
    pub title: Label,
    /// Where it comes from: the rule or the state behind it.
    pub cause: Label,
    /// What opens it; all hold while it is open.
    pub enter: Vec<Condition>,
    pub milestones: Vec<Milestone>,
    /// What closes it.
    pub exit: Vec<Condition>,
    pub forces: Vec<Force>,
    /// 0–1, with the line the side panel shows.
    pub progress: Option<(f32, Label)>,
    /// What happens when it completes.
    pub completes: Option<Label>,
    pub screen: Screen,
}

/// Works of one building, public or private, for the side panel: one line however many
/// levels and states.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkLine {
    /// Index into the catalogue's buildings.
    pub kind: u16,
    /// The state of the first work, and how many states the works are in.
    pub state: u16,
    pub states: usize,
    /// Works in the line.
    pub count: usize,
    pub public: bool,
    /// 0–1 of all their points.
    pub done: f32,
    pub points: f64,
    /// Weeks until the last of them is done at last week's pace; `None` past
    /// [`HORIZON_WEEKS`].
    pub weeks: Option<u32>,
    /// Waits for construction points: earlier works take the week's delivery first.
    pub waiting: bool,
}

/// The queue at a glance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Works {
    pub public: usize,
    pub private: usize,
    /// Public works first, in queue order, then the largest private ones; works of the same
    /// building grouped.
    pub first: Vec<WorkLine>,
    /// What slows the queue down; empty when nothing does.
    pub bottlenecks: Vec<Label>,
}

/// What the country researches now.
#[derive(Clone, Debug, PartialEq)]
pub struct ResearchLine {
    pub tech: u16,
    /// The stage it is working toward.
    pub toward: Stage,
    pub progress: f32,
    /// Chosen by the player; `false` for one the country is closest to on its own.
    pub chosen: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Journal {
    /// Month it was read in.
    pub month: (i32, u8),
    pub entries: Vec<Entry>,
    pub works: Works,
    /// The focus, or the technologies closest to their next stage.
    pub research: Vec<ResearchLine>,
}

impl Journal {
    pub fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

/// What the reader needs besides the world.
#[derive(Clone, Copy)]
pub struct Rules<'a> {
    pub catalogue: &'a Catalogue,
    pub tree: Option<&'a TechTree>,
    pub politics: Option<&'a Politics>,
    /// The unit catalogue, when the forces run.
    pub military: Option<&'a bf_sim::war::Military>,
    /// The nuclear programme's rule and chain, with the war switched on.
    pub programme: Option<&'a bf_sim::war::programme::ProgrammeSetup>,
}

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

/// Reads the player's journal; `None` without a player or an economy.
pub fn read(world: &GameWorld, tag: Option<&str>, date: Date, rules: Rules<'_>) -> Option<Journal> {
    let tag = tag?;
    let c = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    read_country(world, c, date, rules)
}

/// Reads the journal of country `c` (index in the simulation's order).
pub fn read_country(world: &GameWorld, c: usize, date: Date, rules: Rules<'_>) -> Option<Journal> {
    let m = world.econ.markets.get(c)?;
    let mut entries = Vec::new();
    let works = works(world, m, rules.catalogue);
    works_entries(world, m, rules.catalogue, &works, &mut entries);
    let research = research(world, c, rules.tree);
    research_entry(&research, rules.tree, &mut entries);
    fiscal_entries(m, WorksBudget::read(m, rules.catalogue), date, &mut entries);
    if let Some(p) = rules.politics {
        politics_entries(world, c, p, &mut entries);
    }
    if let Some(mil) = rules.military {
        forces_entry(world, c, mil, &mut entries);
        revolt_entries(world, mil, &mut entries);
        if let (Some(ps), Some(tree)) = (rules.programme, rules.tree) {
            programme_entry(world, c, mil, ps, tree, rules.catalogue, &mut entries);
        }
    }
    for e in &mut entries {
        e.id = e.kind.id(rules.politics);
    }
    Some(Journal {
        month: (date.year, date.month),
        entries,
        works,
        research,
    })
}

/// A country born in revolt (G6): the colonial war under way, its irregulars and the men the
/// metropole landed, with the way to play the new country (switch countries in the "⋯" of the
/// Campaigns screen, decision C4). Every player sees it.
fn revolt_entries(world: &GameWorld, mil: &bf_sim::war::Military, out: &mut Vec<Entry>) {
    let tag = |m: u16| {
        world
            .countries
            .iter()
            .nth(usize::from(m))
            .map(|(_, c)| c.tag.clone())
    };
    for k in world.war.colonial.iter().filter(|k| !k.peace) {
        let (Some(colony), Some(owner)) = (tag(k.colony), tag(k.owner)) else {
            continue;
        };
        let mut e = entry(
            Kind::Revolt(colony.clone()),
            Tone::Warning,
            label("journal-revolt-title").key("country", format!("country-{colony}")),
            label("journal-revolt-cause")
                .key("country", format!("country-{colony}"))
                .key("owner", format!("country-{owner}")),
            Screen::Forces,
        );
        let (men, armed) = world
            .war
            .countries
            .get(usize::from(k.colony))
            .map_or((0, 0), |cw| {
                bf_sim::war::colonial::irregulars(mil, mil.unit("irregular"), cw)
            });
        e.forces.push(Force {
            label: label("journal-revolt-irregulars"),
            value: Some(Value::Count(men as i64)),
            helps: false,
        });
        e.forces.push(Force {
            label: label("journal-revolt-armed"),
            value: Some(Value::Count(armed as i64)),
            helps: false,
        });
        if !k.by_land {
            e.forces.push(Force {
                label: label("journal-revolt-landed"),
                value: Some(Value::Count(k.landed as i64)),
                helps: true,
            });
        }
        e.completes = Some(label("journal-revolt-switch"));
        out.push(e);
    }
}

/// The nuclear programme under way: the chain's step, the months left at the month's credit,
/// the warheads; its controls are in the forces tab.
fn programme_entry(
    world: &GameWorld,
    c: usize,
    mil: &bf_sim::war::Military,
    ps: &bf_sim::war::programme::ProgrammeSetup,
    tree: &TechTree,
    cat: &Catalogue,
    out: &mut Vec<Entry>,
) {
    let Some(v) = crate::forces_view::programme_view(world, cat, mil, ps, tree, c) else {
        return;
    };
    if v.share == 0 {
        return;
    }
    let mut e = entry(
        Kind::NuclearProgramme {
            can_test: v.warheads >= 1.0 && !v.tested && !v.test_asked,
        },
        Tone::Neutral,
        label("journal-programme-title"),
        label("journal-programme-cause").value("share", Value::Percent(f64::from(v.share) / 10.0)),
        Screen::Forces,
    );
    let done = v
        .steps
        .iter()
        .filter(|(_, s, _)| *s >= bf_sim::research::Stage::Industrial)
        .count();
    let total = v.steps.len().max(1);
    e.progress = Some((
        done as f32 / total as f32,
        label("journal-programme-progress")
            .value("done", Value::Count(done as i64))
            .value("total", Value::Count(total as i64)),
    ));
    e.forces.push(Force {
        label: label("journal-programme-warheads"),
        value: Some(Value::Count(v.warheads.round() as i64)),
        helps: true,
    });
    if let Some(mo) = v.months_left {
        e.forces.push(Force {
            label: label("journal-programme-months"),
            value: Some(Value::Number(mo.round())),
            helps: true,
        });
    }
    out.push(e);
}

fn entry(kind: Kind, tone: Tone, title: Label, cause: Label, screen: Screen) -> Entry {
    Entry {
        id: String::new(),
        kind,
        tone,
        title,
        cause,
        enter: Vec::new(),
        milestones: Vec::new(),
        exit: Vec::new(),
        forces: Vec::new(),
        progress: None,
        completes: None,
        screen,
    }
}

// ---------------------------------------------------------------------------------------------
// Works

/// Points the builders delivered last week, as the queue's ask times the share of the
/// construction demand the market met.
fn weekly_points(m: &Market, cat: &Catalogue) -> f64 {
    let met = cat
        .construction
        .and_then(|g| m.buy_ratio.get(usize::from(g)))
        .map_or(1.0, |r| f(*r));
    let ask: f64 = m.queue.iter().map(|p| f(want(p))).sum();
    ask * met
}

/// Weeks each of the first `n` works needs at a steady weekly delivery, handed out in queue
/// order as `construction::deliver` does.
fn estimate(m: &Market, n: usize, weekly: f64) -> Vec<(Option<u32>, bool)> {
    let n = n.min(m.queue.len());
    let mut left: Vec<f64> = m.queue.iter().map(|p| f(p.left)).collect();
    let caps: Vec<f64> = m
        .queue
        .iter()
        .map(|p| f(p.points.mul_ratio(1, MIN_WEEKS).max(Fixed::EPSILON)))
        .collect();
    let mut out = vec![(None, false); n];
    // The first week tells which works wait for points.
    let mut first = weekly;
    for (i, slot) in out.iter_mut().enumerate() {
        slot.1 = first <= 0.0;
        first -= caps[i].min(left[i]).max(0.0);
    }
    if weekly <= 0.0 {
        return out;
    }
    for week in 1..=HORIZON_WEEKS {
        let mut points = weekly;
        for (i, l) in left.iter_mut().enumerate() {
            if points <= 0.0 {
                break;
            }
            if *l <= 0.0 {
                continue;
            }
            let got = caps[i].min(*l).min(points);
            points -= got;
            *l -= got;
            if *l <= 1e-9 && i < n && out[i].0.is_none() {
                out[i].0 = Some(week);
            }
        }
        if out.iter().all(|o| o.0.is_some()) {
            break;
        }
    }
    out
}

fn works(world: &GameWorld, m: &Market, cat: &Catalogue) -> Works {
    let public = m.queue.iter().filter(|p| p.public).count();
    let weekly = weekly_points(m, cat);
    let estimates = estimate(m, m.queue.len(), weekly);
    // One line per (building, public), in order of first appearance.
    let mut lines: Vec<(WorkLine, f64, Vec<u16>)> = Vec::new();
    for (p, (weeks, waiting)) in m.queue.iter().zip(estimates) {
        let state = state_of(world, p.region);
        let (points, left) = (f(p.points), f(p.left));
        match lines
            .iter_mut()
            .find(|(l, _, _)| l.kind == p.kind && l.public == p.public)
        {
            Some((l, l_left, states)) => {
                l.count += 1;
                l.points += points;
                *l_left += left;
                if !states.contains(&state) {
                    states.push(state);
                }
                l.weeks = match (l.weeks, weeks) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    _ => None,
                };
                l.waiting |= waiting;
            }
            None => lines.push((
                WorkLine {
                    kind: p.kind,
                    state,
                    states: 1,
                    count: 1,
                    public: p.public,
                    done: 0.0,
                    points,
                    weeks,
                    waiting,
                },
                left,
                vec![state],
            )),
        }
    }
    let mut lines: Vec<WorkLine> = lines
        .into_iter()
        .map(|(mut l, left, states)| {
            l.done = done(l.points, left);
            l.states = states.len();
            l
        })
        .collect();
    // Public first in queue order; private by size. The sort is stable.
    lines.sort_by(|a, b| match (a.public, b.public) {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => b.points.total_cmp(&a.points),
    });
    lines.truncate(WORKS_LISTED);
    Works {
        public,
        private: m.queue.len() - public,
        first: lines,
        bottlenecks: if m.queue.is_empty() {
            Vec::new()
        } else {
            bottlenecks(m, cat)
        },
    }
}

fn state_of(world: &GameWorld, region: u32) -> u16 {
    world
        .econ
        .regions
        .get(region as usize)
        .map_or(0, |r| r.state)
}

fn done(points: f64, left: f64) -> f32 {
    if points > 0.0 {
        ((points - left) / points).clamp(0.0, 1.0) as f32
    } else {
        1.0
    }
}

/// Share of the demand met for a good, and its price over the base price.
fn good_state(m: &Market, cat: &Catalogue, g: usize) -> Option<(f64, f64)> {
    let met = f(*m.buy_ratio.get(g)?);
    let base = f(cat.goods.get(g)?.base_price);
    let price = f(*m.price.get(g)?);
    Some((met, if base > 0.0 { price / base } else { 1.0 }))
}

/// What slows the queue: builders short of points, their inputs short, construction dearer
/// than its base price, a treasury below its reserve (no new public works).
fn bottlenecks(m: &Market, cat: &Catalogue) -> Vec<Label> {
    let mut out = Vec::new();
    if let Some(g) = cat.construction
        && let Some((met, price)) = good_state(m, cat, usize::from(g))
    {
        if met < 1.0 - 1e-6 {
            out.push(label("journal-bottleneck-builders").value("met", Value::Share(met)));
        }
        if price > 1.0 + 1e-6 {
            out.push(label("journal-bottleneck-price").value("price", Value::Share(price)));
        }
    }
    for key in ["cement", "steel"] {
        if let Some(g) = cat.good(key)
            && let Some((met, _)) = good_state(m, cat, usize::from(g))
            && met < 1.0 - 1e-6
            && f(m.demand[usize::from(g)]) > 0.0
        {
            out.push(
                label("journal-bottleneck-input")
                    .key("good", format!("good-{key}"))
                    .value("met", Value::Share(met)),
            );
        }
    }
    if let Some(w) = WorksBudget::read(m, cat)
        && w.total() <= 0.0
    {
        out.push(label("journal-bottleneck-cash"));
    }
    out
}

fn works_entries(
    world: &GameWorld,
    m: &Market,
    cat: &Catalogue,
    works: &Works,
    out: &mut Vec<Entry>,
) {
    if m.queue.is_empty() {
        return;
    }
    let total = m.queue.len();
    let points: f64 = m.queue.iter().map(|p| f(p.points)).sum();
    let left: f64 = m.queue.iter().map(|p| f(p.left)).sum();
    let weekly = weekly_points(m, cat);
    let mut e = entry(
        Kind::Works,
        if works.bottlenecks.is_empty() {
            Tone::Neutral
        } else {
            Tone::Warning
        },
        label("journal-works-title"),
        label("journal-works-cause"),
        Screen::Budget,
    );
    e.enter.push(
        Condition::new(label("journal-works-enter"))
            .measured(Value::Count(total as i64))
            .holds(true),
    );
    for w in &works.first {
        e.milestones.push(Milestone {
            label: work_label(cat, w),
            reached: false,
            date: None,
            progress: Some(w.done),
        });
    }
    e.exit.push(
        Condition::new(label("journal-works-exit"))
            .measured(Value::Count(total as i64))
            .limit(Cmp::AtMost, Value::Count(0))
            .holds(false),
    );
    e.forces.push(Force {
        label: label("journal-force-points"),
        value: Some(Value::Count(weekly.round() as i64)),
        helps: true,
    });
    for b in &works.bottlenecks {
        e.forces.push(Force {
            label: b.clone(),
            value: None,
            helps: false,
        });
    }
    let share = done(points, left);
    e.progress = Some((
        share,
        label("journal-works-progress")
            .value("public", Value::Count(works.public as i64))
            .value("private", Value::Count(works.private as i64)),
    ));
    e.completes = Some(label("journal-works-completes"));
    out.push(e);

    // The largest public work: the state's biggest bet in the queue.
    let Some((i, p)) = m
        .queue
        .iter()
        .enumerate()
        .filter(|(_, p)| p.public)
        .max_by(|a, b| a.1.points.cmp(&b.1.points).then(b.0.cmp(&a.0)))
    else {
        return;
    };
    let estimate = estimate(m, i + 1, weekly);
    let (weeks, waiting) = estimate.get(i).copied().unwrap_or((None, true));
    let key = building_key(cat, p.kind);
    let state = state_of(world, p.region);
    let mut e = entry(
        Kind::LargeWork,
        Tone::Neutral,
        label("journal-large-title")
            .key("building", format!("building-{key}"))
            .arg("state", Arg::State(state)),
        label("journal-large-cause").value("points", Value::Count(f(p.points).round() as i64)),
        Screen::Budget,
    );
    e.enter.push(
        Condition::new(label("journal-large-enter"))
            .measured(Value::Count(i as i64 + 1))
            .holds(true),
    );
    e.milestones.push(Milestone {
        label: label("journal-large-points"),
        reached: false,
        date: None,
        progress: Some(done(f(p.points), f(p.left))),
    });
    e.exit.push(
        Condition::new(label("journal-large-exit"))
            .measured(Value::Count(f(p.left).round() as i64))
            .limit(Cmp::AtMost, Value::Count(0))
            .holds(false),
    );
    e.forces.push(Force {
        label: label("journal-force-ahead"),
        value: Some(Value::Count(i as i64)),
        helps: false,
    });
    e.forces.push(Force {
        label: label("journal-force-min-weeks"),
        value: Some(Value::Weeks(MIN_WEEKS as f64)),
        helps: false,
    });
    let line = match (weeks, waiting) {
        (Some(w), _) => label("journal-weeks-left").value("weeks", Value::Weeks(f64::from(w))),
        (None, true) => label("journal-waiting"),
        (None, false) => {
            label("journal-weeks-beyond").value("weeks", Value::Weeks(f64::from(HORIZON_WEEKS)))
        }
    };
    e.progress = Some((done(f(p.points), f(p.left)), line));
    e.completes = Some(label("journal-large-completes").key("building", format!("building-{key}")));
    out.push(e);
}

/// "School · São Paulo", "Commerce ×3 · Bahia", "Commerce in 5 states".
pub fn work_label(cat: &Catalogue, w: &WorkLine) -> Label {
    let building = format!("building-{}", building_key(cat, w.kind));
    if w.count == 1 {
        label("journal-work-line")
            .key("building", building)
            .arg("state", Arg::State(w.state))
    } else if w.states == 1 {
        label("journal-work-line-many")
            .key("building", building)
            .value("count", Value::Count(w.count as i64))
            .arg("state", Arg::State(w.state))
    } else {
        label("journal-work-line-states")
            .key("building", building)
            .value("states", Value::Count(w.states as i64))
    }
}

fn building_key(cat: &Catalogue, kind: u16) -> &str {
    cat.buildings
        .get(usize::from(kind))
        .map_or("", |b| b.key.as_str())
}

// ---------------------------------------------------------------------------------------------
// Research

fn next_stage(s: Stage) -> Option<Stage> {
    let i = Stage::ALL.iter().position(|x| *x == s)?;
    Stage::ALL.get(i + 1).copied()
}

fn research(world: &GameWorld, c: usize, tree: Option<&TechTree>) -> Vec<ResearchLine> {
    let r = &world.research;
    let (Some(stages), Some(progress)) = (r.stage.get(c), r.progress.get(c)) else {
        return Vec::new();
    };
    let line = |t: usize, chosen: bool| {
        Some(ResearchLine {
            tech: t as u16,
            toward: next_stage(*stages.get(t)?)?,
            progress: progress.get(t).map_or(0.0, |p| f(*p) as f32),
            chosen,
        })
    };
    if let Some(t) = r.focus.get(c).copied().flatten()
        && let Some(l) = line(usize::from(t), true)
    {
        return vec![l];
    }
    // Without a focus the country picks by itself: name the prototypes closest to industry.
    let n = tree.map_or(stages.len(), |t| t.techs.len().min(stages.len()));
    let mut prototypes: Vec<usize> = (0..n).filter(|&t| stages[t] == Stage::Prototype).collect();
    prototypes.sort_by(|&a, &b| progress.get(b).cmp(&progress.get(a)).then(a.cmp(&b)));
    prototypes
        .into_iter()
        .take(PROTOTYPES_LISTED)
        .filter_map(|t| line(t, false))
        .collect()
}

fn research_entry(lines: &[ResearchLine], tree: Option<&TechTree>, out: &mut Vec<Entry>) {
    let (Some(first), Some(tree)) = (lines.first(), tree) else {
        return;
    };
    let tech = |t: u16| {
        format!(
            "tech-{}",
            tree.techs
                .get(usize::from(t))
                .map_or("", |t| t.key.as_str())
        )
    };
    let mut e = entry(
        Kind::Research,
        Tone::Neutral,
        label(if first.chosen {
            "journal-research-title-focus"
        } else {
            "journal-research-title-auto"
        })
        .arg("tech", Arg::Key(tech(first.tech))),
        label(if first.chosen {
            "journal-research-cause-focus"
        } else {
            "journal-research-cause-auto"
        }),
        Screen::Research,
    );
    e.enter.push(
        Condition::new(label(if first.chosen {
            "journal-research-enter-focus"
        } else {
            "journal-research-enter-auto"
        }))
        .holds(true),
    );
    for l in lines {
        e.milestones.push(Milestone {
            label: label("journal-research-milestone")
                .arg("tech", Arg::Key(tech(l.tech)))
                .key("stage", format!("tech-stage-{}", l.toward.key())),
            reached: false,
            date: None,
            progress: Some(l.progress),
        });
    }
    e.exit.push(
        Condition::new(
            label("journal-research-exit")
                .key("stage", format!("tech-stage-{}", first.toward.key())),
        )
        .measured(Value::Share(f64::from(first.progress)))
        .limit(Cmp::AtLeast, Value::Share(1.0))
        .holds(false),
    );
    e.progress = Some((
        first.progress,
        label("journal-research-progress")
            .key("stage", format!("tech-stage-{}", first.toward.key())),
    ));
    e.completes = Some(
        label("journal-research-completes")
            .key("stage", format!("tech-stage-{}", first.toward.key())),
    );
    out.push(e);
}

// ---------------------------------------------------------------------------------------------
// Treasury, debt and money

/// The money for new public works, term by term, as `public::public_works` reckons it: cash
/// above the reserve, less what the public works in the queue still cost, plus what the debt
/// rule lets the state borrow for works.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorksBudget {
    pub over_reserve: f64,
    pub committed: f64,
    pub borrowing: f64,
}

impl WorksBudget {
    pub fn read(m: &Market, cat: &Catalogue) -> Option<Self> {
        let price = f(*m.price.get(usize::from(cat.construction?))?);
        let reserve = f(m.budget.revenue_avg) * RESERVE_WEEKS as f64;
        Some(Self {
            over_reserve: f(m.treasury) - reserve,
            committed: m
                .queue
                .iter()
                .filter(|p| p.public)
                .map(|p| f(p.left) * price)
                .sum(),
            borrowing: f(bf_sim::econ::money::borrowing_room(m)),
        })
    }

    pub fn total(&self) -> f64 {
        self.over_reserve - self.committed + self.borrowing
    }
}

/// Share of the deficit room the debt leaves: the rule's room now over the room the same
/// country would have without debt (`money::deficit_room` read twice, its limits unseen).
fn debt_room_left(m: &Market) -> Option<f64> {
    let mut clean = Market::empty(0);
    clean.gdp_last_year = m.gdp_last_year;
    clean.money.bond_rate = m.money.bond_rate;
    let full = f(deficit_room(&clean));
    (full > 0.0).then(|| (f(deficit_room(m)) / full).clamp(0.0, 1.0))
}

/// The armed forces: an agenda with one decision, to mobilize or to send the reserve home.
/// Mobilizing is gradual: the reserve reports over the weeks of the rule, and the entry shows
/// how much of it has.
fn forces_entry(world: &GameWorld, c: usize, mil: &bf_sim::war::Military, out: &mut Vec<Entry>) {
    let Some(cw) = world.war.countries.get(c) else {
        return;
    };
    let in_service: i64 = cw
        .formations
        .iter()
        .filter(|f| !f.called_up)
        .map(|f| bf_sim::war::men(mil, f))
        .sum();
    let called: i64 = cw
        .formations
        .iter()
        .filter(|f| f.called_up)
        .map(|f| bf_sim::war::men(mil, f))
        .sum();
    if in_service + called == 0 && cw.reserve == 0 {
        return;
    }
    let reserve = cw.reserve as i64;
    let (tone, title) = if cw.mobilized {
        (Tone::Warning, "journal-forces-mobilizing-title")
    } else {
        (Tone::Neutral, "journal-forces-title")
    };
    let mut e = entry(
        Kind::Forces {
            mobilized: cw.mobilized,
        },
        tone,
        label(title),
        label("journal-forces-cause").value(
            "weeks",
            Value::Count(i64::from(mil.rules.reserve.call_up_weeks)),
        ),
        Screen::Forces,
    );
    e.forces.push(Force {
        label: label("journal-force-in-service"),
        value: Some(Value::Count(in_service)),
        helps: true,
    });
    e.forces.push(Force {
        label: label("journal-force-reserve"),
        value: Some(Value::Count(reserve)),
        helps: true,
    });
    if cw.mobilized || called > 0 {
        e.forces.push(Force {
            label: label("journal-force-reported"),
            value: Some(Value::Count(called)),
            helps: true,
        });
        let total = (called + reserve).max(1);
        e.progress = Some((
            called as f32 / total as f32,
            label("journal-forces-progress")
                .value("reported", Value::Count(called))
                .value("total", Value::Count(called + reserve)),
        ));
    }
    out.push(e);
}

fn fiscal_entries(m: &Market, works: Option<WorksBudget>, date: Date, out: &mut Vec<Entry>) {
    let b = &m.budget;
    let revenue = f(b.revenue_avg);
    if revenue <= 0.0 {
        return;
    }
    let cash = f(m.treasury);
    let reserve = revenue * RESERVE_WEEKS as f64;
    let interest_share = f(bf_sim::econ::money::interest_share(m));
    let kept = f(austerity(m));

    // No money for new public works: the queue gets none this month.
    if let Some(w) = works
        && w.total() <= 0.0
        && cash >= 0.0
    {
        let mut e = entry(
            Kind::Fiscal(Fiscal::NoWorksBudget),
            Tone::Warning,
            label("journal-noworks-title"),
            label("journal-noworks-cause").value("weeks", Value::Count(RESERVE_WEEKS)),
            Screen::Budget,
        );
        e.enter.push(
            Condition::new(label("journal-cond-works-budget"))
                .measured(Value::Money(w.total()))
                .limit(Cmp::AtMost, Value::Money(0.0))
                .holds(true),
        );
        e.forces.push(Force {
            label: label("journal-force-over-reserve").value("reserve", Value::Money(reserve)),
            value: Some(Value::Money(w.over_reserve)),
            helps: w.over_reserve > 0.0,
        });
        e.forces.push(Force {
            label: label("journal-force-committed"),
            value: Some(Value::Money(-w.committed)),
            helps: false,
        });
        e.forces.push(Force {
            label: label("journal-force-borrowing"),
            value: Some(Value::Money(w.borrowing)),
            helps: true,
        });
        e.forces.push(Force {
            label: label("journal-force-balance"),
            value: Some(Value::Money(
                (f(b.revenue) - f(b.payroll + b.goods + b.services + b.works + b.interest)) * 52.0,
            )),
            helps: f(b.revenue) >= f(b.payroll + b.goods + b.services + b.works + b.interest),
        });
        e.exit.push(
            Condition::new(label("journal-cond-works-budget"))
                .measured(Value::Money(w.total()))
                .limit(Cmp::AtLeast, Value::Money(0.0))
                .holds(false),
        );
        e.completes = Some(label("journal-noworks-completes"));
        out.push(e);
    }

    // Cash below zero: nobody bought the bonds; purchases fall and the tax rule steps up.
    if cash < 0.0 {
        let mut e = entry(
            Kind::Fiscal(Fiscal::Unfunded),
            Tone::Danger,
            label("journal-unfunded-title"),
            label("journal-unfunded-cause"),
            Screen::Money,
        );
        e.enter.push(
            Condition::new(label("journal-cond-cash"))
                .measured(Value::Money(cash))
                .limit(Cmp::AtMost, Value::Money(0.0))
                .holds(true),
        );
        e.exit.push(
            Condition::new(label("journal-cond-cash"))
                .measured(Value::Money(cash))
                .limit(Cmp::AtLeast, Value::Money(0.0))
                .holds(false),
        );
        e.forces.push(Force {
            label: label("journal-force-kept"),
            value: Some(Value::Share(kept)),
            helps: true,
        });
        e.forces.push(Force {
            label: label("journal-force-tax-rule"),
            value: Some(Value::Number(f(b.adjust))),
            helps: true,
        });
        e.completes = Some(label("journal-unfunded-completes"));
        out.push(e);
    }

    // Purchases cut by the budget rule: the treasury in the red or interest above what the
    // rule tolerates. `public::austerity` holds the limits; the entry reads its answer.
    if kept < 1.0 - 1e-6 {
        let mut e = entry(
            Kind::Fiscal(Fiscal::PurchasesCut),
            Tone::Warning,
            label("journal-purchases-title"),
            label("journal-purchases-cause"),
            Screen::Budget,
        );
        e.enter.push(
            Condition::new(label("journal-cond-kept"))
                .measured(Value::Share(kept))
                .limit(Cmp::AtMost, Value::Share(1.0))
                .holds(true),
        );
        e.forces.push(Force {
            label: label("journal-force-interest"),
            value: Some(Value::Share(interest_share)),
            helps: false,
        });
        if cash < 0.0 {
            e.forces.push(Force {
                label: label("journal-cond-cash"),
                value: Some(Value::Money(cash)),
                helps: false,
            });
        }
        e.exit.push(
            Condition::new(label("journal-cond-kept"))
                .measured(Value::Share(kept))
                .limit(Cmp::AtLeast, Value::Share(1.0))
                .holds(false),
        );
        e.progress = Some((
            kept as f32,
            label("journal-purchases-progress").value("kept", Value::Share(kept)),
        ));
        e.completes = Some(label("journal-purchases-completes"));
        out.push(e);
    }

    // Debt eating into the deficit the rule allows; at none left, the state borrows nothing
    // for purchases.
    if let Some(room) = debt_room_left(m)
        && room < DEBT_ROOM_SHOWN
    {
        let gdp = f(m.gdp_last_year);
        let debt = f(m.money.total_debt());
        let mut e = entry(
            Kind::Fiscal(Fiscal::DebtRoom),
            if room <= 0.0 {
                Tone::Danger
            } else {
                Tone::Warning
            },
            label("journal-debt-title"),
            label("journal-debt-cause"),
            Screen::Money,
        );
        e.enter.push(
            Condition::new(label("journal-cond-room"))
                .measured(Value::Share(room))
                .limit(Cmp::AtMost, Value::Share(DEBT_ROOM_SHOWN))
                .holds(true),
        );
        e.milestones.push(Milestone {
            label: label("journal-debt-closed"),
            reached: room <= 0.0,
            date: None,
            progress: Some((1.0 - room) as f32),
        });
        e.forces.push(Force {
            label: label("journal-force-debt-gdp"),
            value: (gdp > 0.0).then(|| Value::Share(debt / gdp)),
            helps: false,
        });
        e.forces.push(Force {
            label: label("journal-force-bond-rate"),
            value: Some(Value::Percent(f(m.money.bond_rate))),
            helps: false,
        });
        e.exit.push(
            Condition::new(label("journal-cond-room"))
                .measured(Value::Share(room))
                .limit(Cmp::AtLeast, Value::Share(DEBT_ROOM_SHOWN))
                .holds(false),
        );
        e.progress = Some((
            (1.0 - room) as f32,
            label("journal-debt-progress").value("room", Value::Share(room)),
        ));
        e.completes = Some(label("journal-debt-completes"));
        out.push(e);
    }

    // After a default: shut out of foreign lending until the year the simulation set.
    let until = m.money.excluded_until;
    if until > date.year && m.money.defaults > 0 {
        let mut e = entry(
            Kind::Fiscal(Fiscal::ShutOut),
            Tone::Danger,
            label("journal-shutout-title"),
            label("journal-shutout-cause")
                .value("defaults", Value::Count(i64::from(m.money.defaults))),
            Screen::Money,
        );
        e.enter.push(
            Condition::new(label("journal-cond-defaults"))
                .measured(Value::Count(i64::from(m.money.defaults)))
                .holds(true),
        );
        e.milestones.push(Milestone {
            label: label("journal-shutout-back").value("year", Value::Year(until)),
            reached: false,
            date: None,
            progress: None,
        });
        e.exit.push(
            Condition::new(label("journal-cond-year"))
                .measured(Value::Year(date.year))
                .limit(Cmp::AtLeast, Value::Year(until))
                .holds(false),
        );
        e.forces.push(Force {
            label: label("journal-force-default-history"),
            value: Some(Value::Number(f(m.money.default_history))),
            helps: false,
        });
        e.completes = Some(label("journal-shutout-completes"));
        out.push(e);
    }

    // Inflation above the central bank's target: the rate rule leans against it.
    let inflation = f(m.money.inflation);
    let target = f(TARGET);
    if inflation > target {
        let mut e = entry(
            Kind::Fiscal(Fiscal::Inflation),
            Tone::Warning,
            label("journal-inflation-title"),
            label(if m.money.subordinate(date.year) {
                "journal-inflation-cause-subordinate"
            } else {
                "journal-inflation-cause-autonomous"
            }),
            Screen::Money,
        );
        e.enter.push(
            Condition::new(label("journal-cond-inflation"))
                .measured(Value::Percent(inflation))
                .limit(Cmp::AtLeast, Value::Percent(target))
                .holds(true),
        );
        e.forces.push(Force {
            label: label("journal-force-policy-rate"),
            value: Some(Value::Percent(f(m.money.policy_rate))),
            helps: true,
        });
        e.forces.push(Force {
            label: label("journal-force-monetized"),
            value: Some(Value::Percent(f(m.money.monetized))),
            helps: false,
        });
        e.forces.push(Force {
            label: label("journal-force-expected"),
            value: Some(Value::Percent(f(m.money.expected))),
            helps: false,
        });
        e.exit.push(
            Condition::new(label("journal-cond-inflation"))
                .measured(Value::Percent(inflation))
                .limit(Cmp::AtMost, Value::Percent(target))
                .holds(false),
        );
        e.completes = Some(label("journal-inflation-completes"));
        out.push(e);
    }
}

// ---------------------------------------------------------------------------------------------
// Politics

/// The measure a movement's condition reads, as a message id.
fn measure_key(m: Measure) -> String {
    match m {
        Measure::Indicator(i) => format!(
            "indicator-{}",
            bf_sim::politics::INDICATORS[usize::from(i)].key
        ),
        Measure::Effect(e) => format!(
            "law-effect-{}",
            bf_sim::politics::effect_kind(usize::from(e)).key
        ),
    }
}

fn politics_entries(world: &GameWorld, c: usize, p: &Politics, out: &mut Vec<Entry>) {
    let s = &world.politics;
    if c >= s.laws.len() {
        return;
    }
    let law_key = |l: u16| p.laws.get(usize::from(l)).map_or("", |l| l.key.as_str());

    // Movements: what they want, the cause each condition measures, support and pressure.
    for (kind, state) in s.active_movements(c) {
        let Some(k) = p.movements.get(usize::from(kind)) else {
            continue;
        };
        let goal = match k.goal {
            GoalRef::Law(l) => label("journal-goal-law").key("law", format!("law-{}", law_key(l))),
            GoalRef::Independence => label("journal-goal-independence"),
            GoalRef::Revision => label("journal-goal-revision"),
        };
        let mut e = entry(
            Kind::Movement(kind),
            if state.pressure > 0 {
                Tone::Warning
            } else {
                Tone::Neutral
            },
            label("journal-movement-title").key("movement", format!("movement-{}", k.key)),
            label("journal-movement-cause").key("desc", format!("movement-{}-desc", k.key)),
            Screen::Laws,
        );
        // The simulation keeps the pressure and the weakest condition, not each measured
        // value: every condition holds while the pressure is above zero, and the weakest one
        // is the one that fails when it is zero.
        for (i, (measure, bound)) in k.cause.iter().enumerate() {
            let (cmp, limit) = match *bound {
                Bound::AtLeast(t) => (Cmp::AtLeast, t),
                Bound::AtMost(t) => (Cmp::AtMost, t),
            };
            let holds = if state.pressure > 0 {
                Some(true)
            } else if usize::from(state.weakest) == i {
                Some(false)
            } else {
                None
            };
            let mut cond =
                Condition::new(label("journal-cond-measure").key("measure", measure_key(*measure)))
                    .limit(cmp, Value::Share(f64::from(limit) / 1000.0));
            cond.holds = holds;
            e.enter.push(cond);
        }
        e.milestones.push(Milestone {
            label: goal.clone(),
            reached: false,
            date: None,
            progress: Some(f32::from(state.support) / 1000.0),
        });
        e.forces.push(Force {
            label: label("journal-force-pressure"),
            value: Some(Value::Share(f64::from(state.pressure) / 1000.0)),
            helps: state.pressure > 0,
        });
        e.forces.push(Force {
            label: label("journal-force-growth"),
            value: Some(Value::Share(f64::from(k.growth) / 1000.0)),
            helps: true,
        });
        if let Some(cond) = k.cause.get(usize::from(state.weakest)) {
            e.forces.push(Force {
                label: label("journal-force-weakest").key("measure", measure_key(cond.0)),
                value: None,
                helps: false,
            });
        }
        e.exit.push(
            Condition::new(
                label("journal-movement-exit")
                    .value("years", Value::Count(i64::from(k.persistence))),
            )
            .measured(Value::Share(f64::from(state.support) / 1000.0))
            .holds(false),
        );
        e.progress = Some((
            f32::from(state.support) / 1000.0,
            label("journal-movement-progress")
                .value("support", Value::Share(f64::from(state.support) / 1000.0)),
        ));
        e.completes = Some(goal);
        out.push(e);
    }

    // Laws asked for and not yet enacted.
    for law in s.pending_laws(c) {
        let mut e = entry(
            Kind::PendingLaw(law),
            Tone::Neutral,
            label("journal-pending-title").key("law", format!("law-{}", law_key(law))),
            label("journal-pending-cause"),
            Screen::Laws,
        );
        e.enter
            .push(Condition::new(label("journal-pending-enter")).holds(true));
        e.exit
            .push(Condition::new(label("journal-pending-exit")).holds(false));
        e.completes =
            Some(label("journal-pending-completes").key("law", format!("law-{}", law_key(law))));
        out.push(e);
    }

    // The campaign's law changes, newest first, as milestones already reached.
    let changes: Vec<_> = s.law_changes(c).collect();
    if !changes.is_empty() {
        let mut e = entry(
            Kind::LawChanges,
            Tone::Neutral,
            label("journal-changes-title"),
            label("journal-changes-cause").value(
                "legitimacy",
                Value::Share(f64::from(s.legitimacy(c)) / 1000.0),
            ),
            Screen::Laws,
        );
        e.enter.push(
            Condition::new(label("journal-changes-enter"))
                .measured(Value::Count(changes.len() as i64))
                .holds(true),
        );
        for r in changes.iter().rev().take(6) {
            e.milestones.push(Milestone {
                label: label("journal-change-line")
                    .key("from", format!("law-{}", law_key(r.from)))
                    .key("to", format!("law-{}", law_key(r.to))),
                reached: true,
                date: Some(r.date),
                progress: None,
            });
        }
        e.forces.push(Force {
            label: label("journal-force-legitimacy"),
            value: Some(Value::Share(f64::from(s.legitimacy(c)) / 1000.0)),
            helps: true,
        });
        out.push(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_sim::econ::state::Project;

    fn project(points: i64, left: i64, public: bool) -> Project {
        Project {
            region: 0,
            kind: 0,
            method: 0,
            public,
            points: Fixed::from_int(points),
            left: Fixed::from_int(left),
        }
    }

    fn market(cash: i64, revenue: i64) -> Market {
        let mut m = Market::empty(0);
        m.treasury = Fixed::from_int(cash);
        m.budget.revenue_avg = Fixed::from_int(revenue);
        m.budget.revenue = Fixed::from_int(revenue);
        m.gdp_last_year = Fixed::from_int(revenue * 52 * 5);
        m
    }

    #[test]
    fn works_take_points_in_queue_order() {
        let mut m = Market::empty(0);
        // 2,600 points: at most 100 a week each.
        m.queue = vec![project(2600, 2600, true), project(2600, 2600, false)];
        // 150 a week: the first takes 100, the second 50 until the first is done.
        let out = estimate(&m, 2, 150.0);
        assert_eq!(out[0], (Some(26), false));
        // 26 weeks at 50 (1,300), then 100 a week for 13 more.
        assert_eq!(out[1], (Some(39), false));
        // Nothing delivered: both wait, no estimate.
        assert_eq!(estimate(&m, 2, 0.0), vec![(None, true), (None, true)]);
        // 100 a week: the second gets nothing in the first week.
        assert!(estimate(&m, 2, 100.0)[1].1);
    }

    #[test]
    fn no_money_for_public_works_opens_its_entry() {
        let m = market(130, 10);
        // Cash at the reserve, works still to pay and no borrowing: nothing for new works.
        let w = WorksBudget {
            over_reserve: 0.0,
            committed: 40.0,
            borrowing: 10.0,
        };
        let mut out = Vec::new();
        fiscal_entries(&m, Some(w), Date::new(1940, 1, 1), &mut out);
        let e = out
            .iter()
            .find(|e| e.kind == Kind::Fiscal(Fiscal::NoWorksBudget))
            .expect("works budget entry");
        assert_eq!(e.enter[0].measured, Some(Value::Money(-30.0)));
        // Borrowing room enough to pay for the queue: nothing to say.
        let w = WorksBudget {
            borrowing: 50.0,
            ..w
        };
        let mut out = Vec::new();
        fiscal_entries(&m, Some(w), Date::new(1940, 1, 1), &mut out);
        assert!(out.is_empty(), "{out:?}");
    }

    #[test]
    fn a_treasury_in_the_red_cuts_purchases() {
        let m = market(-260, 10);
        let mut out = Vec::new();
        fiscal_entries(&m, None, Date::new(1940, 1, 1), &mut out);
        let kinds: Vec<_> = out.iter().map(|e| e.kind.clone()).collect();
        assert!(kinds.contains(&Kind::Fiscal(Fiscal::Unfunded)), "{kinds:?}");
        assert!(
            kinds.contains(&Kind::Fiscal(Fiscal::PurchasesCut)),
            "{kinds:?}"
        );
        assert!(!kinds.contains(&Kind::Fiscal(Fiscal::NoWorksBudget)));
    }

    #[test]
    fn the_debt_entry_reads_the_rules_room() {
        let mut m = market(500, 10);
        assert_eq!(debt_room_left(&m), Some(1.0));
        // Debt as large as the room's own limit leaves nothing; half of it leaves half.
        m.money.debt_home = m.gdp_last_year * 100;
        assert_eq!(debt_room_left(&m), Some(0.0));
        let mut out = Vec::new();
        fiscal_entries(&m, None, Date::new(1940, 1, 1), &mut out);
        let e = out
            .iter()
            .find(|e| e.kind == Kind::Fiscal(Fiscal::DebtRoom))
            .expect("debt entry");
        assert!(e.milestones[0].reached);
    }

    #[test]
    fn a_default_shuts_out_until_the_year_set() {
        let mut m = market(500, 10);
        m.money.defaults = 1;
        m.money.excluded_until = 1943;
        let mut out = Vec::new();
        fiscal_entries(&m, None, Date::new(1941, 6, 1), &mut out);
        assert!(out.iter().any(|e| e.kind == Kind::Fiscal(Fiscal::ShutOut)));
        let mut out = Vec::new();
        fiscal_entries(&m, None, Date::new(1943, 6, 1), &mut out);
        assert!(!out.iter().any(|e| e.kind == Kind::Fiscal(Fiscal::ShutOut)));
    }

    #[test]
    fn ids_are_stable_keys() {
        assert_eq!(Kind::Fiscal(Fiscal::DebtRoom).id(None), "fiscal-debt-room");
        assert_eq!(Kind::Works.id(None), "works");
        assert_eq!(Kind::Movement(3).id(None), "movement-3");
    }
}
