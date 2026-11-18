//! The simulation runs off the interface thread, on the Laplace-Schelling engine. The interface
//! reads a published snapshot and sends commands; it never waits for a tick, only for a save.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bf_sim::game::{GameCommand, Session};
use bf_sim::research::Stage;
use laplace_schelling::calendar::Date;

use crate::journal::Journal;
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
    /// The player's laws, interest groups and movements; `None` without the politics data.
    pub laws: Option<Arc<crate::laws_view::Live>>,
    /// The player's journal, read on the first snapshot and then once a month; `None` without
    /// a player, an economy or the rules.
    pub journal: Option<Arc<crate::journal::Journal>>,
    /// The player's population: pyramid, rates, fertility terms, migration; read on the first
    /// snapshot and then once a month.
    pub population: Option<Arc<crate::population::PopulationView>>,
    /// The player's pending independence demands, read once a month.
    pub demands: Option<Arc<crate::demands_view::DemandsView>>,
    /// The player's debts between governments, read once a month.
    pub ledger: Option<Arc<crate::ledger_view::LedgerView>>,
    /// The influence around the player, read once a month; `None` without influence rules.
    pub influence: Option<Arc<crate::influence_view::InfluenceView>>,
    /// Who holds every state, read again when a border operation ran; `None` without the rules.
    pub owners: Option<Arc<bf_sim::borders::op::OwnersView>>,
    /// The ways the player's country can free land (the decision "Libertar território"), read
    /// once a month and when the borders or the player change.
    pub liberation: Option<Arc<LiberationView>>,
    /// The influence around the country the "Influence" lens looks at, read once a month and when
    /// that country changes; `None` without the lens on or without influence.
    pub lens_influence: Option<Arc<crate::influence_view::InfluenceView>>,
    /// The player's crises and wars, read when they change; `None` with the crisis switch off.
    pub crises: Option<Arc<crate::crisis_view::CrisisView>>,
    /// The demand the player is drafting, weighed as its target would weigh it.
    pub crisis_draft: Option<Arc<crate::crisis_view::DraftWeighing>>,
    /// The player's public works and what could be built, read once a month and when the queue
    /// or its order changes (the government's works tab).
    pub works: Option<Arc<crate::works_view::WorksView>>,
    /// The blocs, read once a month, for the "Blocs" lens.
    pub blocs: Option<Arc<crate::blocs_view::BlocsView>>,
    /// The player's armed forces, read once a month; `None` without forces.
    pub forces: Option<Arc<crate::forces_view::ForcesView>>,
    /// The player's agreements, breaches and the reputations, read when they change; `None`
    /// without any agreement on record.
    pub agreements: Option<Arc<crate::agreements_view::AgreementsView>>,
    /// Every country's trade by partner, read once a month; `None` with trade by pair off.
    pub trade: Option<Arc<crate::trade_view::TradeView>>,
    /// The player's wars on land, read when the fighting changes; `None` without the war rule.
    pub war: Option<Arc<crate::war_view::WarView>>,
    /// Who holds what in every war, for the map's war lens; `None` with no war.
    pub control: Option<Arc<crate::war_view::ControlView>>,
    /// The negotiation table of the player's country, read when the table changes and weekly;
    /// `None` with the `negotiation` rule off.
    pub treaty_table: Option<Arc<crate::treaty_view::TableView>>,
    /// The proposal the player is drafting on the table, weighed.
    pub treaty_draft: Option<Arc<crate::treaty_view::Weighed>>,
}

/// What the decision to free land shows.
#[derive(Clone, Debug, PartialEq)]
pub struct LiberationView {
    pub tag: String,
    pub month: (i32, u8),
    pub changes: usize,
    pub options: Vec<bf_sim::borders::op::LiberationOption>,
}

fn liberation_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<LiberationView>>,
) -> Option<Arc<LiberationView>> {
    let tag = session.player()?;
    let date = session.clock().date;
    let w = session.world();
    let changes = w.borders.changes.len();
    if let Some(l) = last
        && l.tag == tag
        && l.month == (date.year, date.month)
        && l.changes == changes
    {
        return Some(Arc::clone(l));
    }
    let b = rules.as_ref()?.borders.as_ref()?;
    let c = w.countries.iter().position(|(_, c)| c.tag == tag)?;
    Some(Arc::new(LiberationView {
        tag: tag.to_owned(),
        month: (date.year, date.month),
        changes,
        options: bf_sim::borders::op::liberation_options(w, b, c),
    }))
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
    let tag = session.player()?;
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
    /// The country the "Influence" lens looks at (`None`: the lens is off).
    LensFocus(Option<String>),
    /// The demand the player is drafting, to be weighed (`None`: no draft).
    CrisisDraft(Option<(String, bf_sim::treaty::Proposal)>),
    /// The proposal the player is drafting on the negotiation table, to be weighed: the other
    /// party and the proposal (`None`: no draft).
    TreatyDraft(Option<(String, bf_sim::treaty::Proposal)>),
    SetSpeed(u8),
    Issue(GameCommand),
    /// Writes the engine save to `path`; replies with the error text on failure.
    Save {
        path: PathBuf,
        /// More files that get the same bytes (a restore point beside the current state).
        copies: Vec<PathBuf>,
        label: String,
        reply: Sender<Result<(), String>>,
    },
}

/// What the snapshot reads with the world: the rules the session runs.
pub type Rules = Option<bf_sim::game::Rules>;

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

/// The month's journal, read again only when the month changed since `last`.
fn journal_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<Journal>>,
) -> Option<Arc<Journal>> {
    let date = session.clock().date;
    match last {
        Some(j) if j.month == (date.year, date.month) => Some(Arc::clone(j)),
        _ => {
            let r = rules.as_ref()?;
            let rules = crate::journal::Rules {
                catalogue: &r.catalogue,
                tree: r.tree.as_deref(),
                politics: r.politics.as_deref(),
                military: r.war.as_ref().map(|(m, _)| m.as_ref()),
            };
            crate::journal::read(session.world(), session.player(), date, rules).map(Arc::new)
        }
    }
}

/// The month's population view, read again only when the month changed since `last`.
fn population_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::population::PopulationView>>,
) -> Option<Arc<crate::population::PopulationView>> {
    let date = session.clock().date;
    match last {
        Some(p) if p.month == (date.year, date.month) => Some(Arc::clone(p)),
        _ => {
            let table = rules.as_ref().and_then(|r| r.communities.as_deref());
            let cat = rules.as_ref().map(|r| &*r.catalogue);
            crate::population::read(session.world(), session.player(), date, table, cat)
                .map(Arc::new)
        }
    }
}

fn ledger_of(
    session: &Session,
    last: &Option<Arc<crate::ledger_view::LedgerView>>,
) -> Option<Arc<crate::ledger_view::LedgerView>> {
    let date = session.clock().date;
    match last {
        Some(l)
            if l.month == (date.year, date.month) && Some(l.tag.as_str()) == session.player() =>
        {
            Some(Arc::clone(l))
        }
        _ => crate::ledger_view::read(session.world(), session.player(), date).map(Arc::new),
    }
}

fn influence_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::influence_view::InfluenceView>>,
) -> Option<Arc<crate::influence_view::InfluenceView>> {
    let date = session.clock().date;
    match last {
        Some(v)
            if v.month == (date.year, date.month) && Some(v.tag.as_str()) == session.player() =>
        {
            Some(Arc::clone(v))
        }
        _ => crate::influence_view::read(
            session.world(),
            rules.as_ref().and_then(|r| r.influence.as_deref()),
            session.player(),
            date,
        )
        .map(Arc::new),
    }
}

/// The month's forces of the player, read again only when the month changed since `last`.
fn forces_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::forces_view::ForcesView>>,
) -> Option<Arc<crate::forces_view::ForcesView>> {
    let date = session.clock().date;
    // An order placed or cancelled, or a formation disbanded, shows at once, not at the turn of
    // the month.
    let stock = || {
        let w = session.world();
        let tag = w.player_tag()?;
        let ci = w.countries.iter().position(|(_, c)| c.tag == tag)?;
        w.war.countries.get(ci).map(crate::forces_view::stock)
    };
    match last {
        Some(f) if f.month == (date.year, date.month) && stock() == Some(f.stock) => {
            Some(Arc::clone(f))
        }
        _ => {
            let r = rules.as_ref()?;
            let (mil, peace) = r.war.as_ref()?;
            crate::forces_view::read(
                session.world(),
                date,
                &r.catalogue,
                mil,
                peace,
                r.switches.small_market_wage_floor,
            )
            .map(Arc::new)
        }
    }
}

/// The demands, read again on a new month or when the answers changed (an answer just given
/// takes the demand off the list).
fn demands_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::demands_view::DemandsView>>,
) -> Option<Arc<crate::demands_view::DemandsView>> {
    let date = session.clock().date;
    let w = session.world();
    let fresh = last.as_ref().is_some_and(|d| {
        d.month == (date.year, date.month)
            && Some(d.tag.as_str()) == session.player()
            && w.independence.answers.is_empty()
    });
    if fresh {
        return last.clone();
    }
    crate::demands_view::read(
        w,
        rules.as_ref().and_then(|r| r.independence.as_deref()),
        session.player(),
        date,
    )
    .map(Arc::new)
}

/// What a snapshot carries that is read once a month, or when it changed.
#[derive(Clone, Default)]
struct Monthly {
    figures: Option<Arc<LiveFigures>>,
    journal: Option<Arc<Journal>>,
    population: Option<Arc<crate::population::PopulationView>>,
    demands: Option<Arc<crate::demands_view::DemandsView>>,
    ledger: Option<Arc<crate::ledger_view::LedgerView>>,
    influence: Option<Arc<crate::influence_view::InfluenceView>>,
    owners: Option<Arc<bf_sim::borders::op::OwnersView>>,
    liberation: Option<Arc<LiberationView>>,
    forces: Option<Arc<crate::forces_view::ForcesView>>,
    lens_influence: Option<Arc<crate::influence_view::InfluenceView>>,
    blocs: Option<Arc<crate::blocs_view::BlocsView>>,
    crises: Option<Arc<crate::crisis_view::CrisisView>>,
    crisis_draft: Option<Arc<crate::crisis_view::DraftWeighing>>,
    works: Option<Arc<crate::works_view::WorksView>>,
    agreements: Option<Arc<crate::agreements_view::AgreementsView>>,
    trade: Option<Arc<crate::trade_view::TradeView>>,
    war: Option<Arc<crate::war_view::WarView>>,
    control: Option<Arc<crate::war_view::ControlView>>,
    treaty_table: Option<Arc<crate::treaty_view::TableView>>,
    treaty_draft: Option<Arc<crate::treaty_view::Weighed>>,
}

/// The player's works, read again on a new month or when the queue or its order changed.
fn works_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::works_view::WorksView>>,
) -> Option<Arc<crate::works_view::WorksView>> {
    let date = session.clock().date;
    let w = session.world();
    let ci = w
        .player_tag()
        .and_then(|t| w.countries.iter().position(|(_, c)| c.tag == t))?;
    match last {
        Some(v)
            if v.month == (date.year, date.month)
                && v.signature == crate::works_view::signature(w, ci) =>
        {
            Some(Arc::clone(v))
        }
        _ => {
            let r = rules.as_ref()?;
            crate::works_view::read(w, &r.catalogue, r.tree.as_deref(), date).map(Arc::new)
        }
    }
}

/// The player's wars, read again when the fighting changed or a week went by.
fn war_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::war_view::WarView>>,
) -> Option<Arc<crate::war_view::WarView>> {
    let setup = rules.as_ref()?.fight.as_deref()?;
    let days = session.clock().date.to_days();
    let w = session.world();
    match last {
        Some(v)
            if v.signature == crate::war_view::signature(w, days) && session.player().is_some() =>
        {
            Some(Arc::clone(v))
        }
        _ => crate::war_view::read(w, setup, days).map(Arc::new),
    }
}

/// The control of every war, read again when it changed.
fn control_of(
    session: &Session,
    last: &Option<Arc<crate::war_view::ControlView>>,
) -> Option<Arc<crate::war_view::ControlView>> {
    let now = crate::war_view::read_control(session.world())?;
    match last {
        Some(v) if **v == now => Some(Arc::clone(v)),
        _ => Some(Arc::new(now)),
    }
}

/// Every country's trade by partner, read again on a new month.
fn trade_of(
    session: &Session,
    last: &Option<Arc<crate::trade_view::TradeView>>,
) -> Option<Arc<crate::trade_view::TradeView>> {
    let date = session.clock().date;
    match last {
        Some(t) if t.month == (date.year, date.month) => Some(Arc::clone(t)),
        _ => crate::trade_view::read(session.world(), date).map(Arc::new),
    }
}

/// The player's agreements, read again when they changed or the year turned.
fn agreements_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::agreements_view::AgreementsView>>,
) -> Option<Arc<crate::agreements_view::AgreementsView>> {
    let year = session.clock().date.year;
    let w = session.world();
    match last {
        Some(v)
            if v.signature == crate::agreements_view::signature(w, year)
                && session.player() == Some(v.me.as_str()) =>
        {
            Some(Arc::clone(v))
        }
        _ => {
            let negotiation = rules.as_ref().is_some_and(|r| r.switches.negotiation);
            crate::agreements_view::read(w, year, negotiation).map(Arc::new)
        }
    }
}

/// What the monthly views read besides the session: the scenario's treaties (for the blocs)
/// and the country the influence lens looks at.
#[derive(Default)]
pub struct Extras {
    pub diplomacy: Option<Arc<bf_sim::diplomacy::DiplomacyFile>>,
    pub focus: Option<String>,
    /// The demand being drafted: target and proposal.
    pub draft: Option<(String, bf_sim::treaty::Proposal)>,
    /// The proposal being drafted on the negotiation table: the other party and the proposal.
    pub treaty_draft: Option<(String, bf_sim::treaty::Proposal)>,
}

/// The clause catalogue of the negotiation table, with the `negotiation` rule on.
fn table_catalogue(rules: &Rules) -> Option<&bf_sim::treaty::ClauseCatalogue> {
    let r = rules.as_ref()?;
    if !r.switches.negotiation {
        return None;
    }
    r.treaties.as_ref()?.clauses.as_deref()
}

/// The player's negotiation table, read again when it changed or a week went by.
fn treaty_table_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::treaty_view::TableView>>,
) -> Option<Arc<crate::treaty_view::TableView>> {
    let cat = table_catalogue(rules)?;
    let date = session.clock().date;
    match last {
        Some(v)
            if v.signature == crate::treaty_view::signature(session.world(), date)
                && session.player().is_some() =>
        {
            Some(Arc::clone(v))
        }
        _ => crate::treaty_view::read(session.world(), cat, date).map(Arc::new),
    }
}

/// The draft on the table, weighed again when the draft or the table changed.
fn treaty_draft_of(
    session: &Session,
    rules: &Rules,
    extras: &Extras,
    table: &Option<Arc<crate::treaty_view::TableView>>,
    last: &Monthly,
) -> Option<Arc<crate::treaty_view::Weighed>> {
    let (other, proposal) = extras.treaty_draft.as_ref()?;
    let cat = table_catalogue(rules)?;
    if let (Some(v), Some(t), Some(lt)) = (&last.treaty_draft, table, &last.treaty_table)
        && v.other == *other
        && v.proposal == *proposal
        && t.signature == lt.signature
    {
        return Some(Arc::clone(v));
    }
    let me = session.player()?;
    Some(Arc::new(crate::treaty_view::weigh(
        session.world(),
        cat,
        me,
        other,
        proposal,
        true,
    )))
}

/// The player's crises, read again when the conflicts changed since `last`.
fn crises_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<crate::crisis_view::CrisisView>>,
) -> Option<Arc<crate::crisis_view::CrisisView>> {
    let r = rules.as_ref()?;
    let setup = r.crisis.as_deref()?;
    let sig = crate::crisis_view::signature(session.world());
    match last {
        Some(v) if v.signature == sig && session.player().is_some() => Some(Arc::clone(v)),
        _ => crate::crisis_view::read(
            session.world(),
            Some(setup),
            r.war.as_ref().map(|(m, _)| m.as_ref()),
        )
        .map(Arc::new),
    }
}

/// The draft's weighing, again when the draft or the conflicts changed.
fn draft_of(
    session: &Session,
    rules: &Rules,
    extras: &Extras,
    last: &Option<Arc<crate::crisis_view::DraftWeighing>>,
) -> Option<Arc<crate::crisis_view::DraftWeighing>> {
    let (target, demand) = extras.draft.as_ref()?;
    if let Some(v) = last
        && v.target == *target
        && v.demand == *demand
    {
        return Some(Arc::clone(v));
    }
    let r = rules.as_ref()?;
    crate::crisis_view::weigh_draft(
        session.world(),
        r.crisis.as_deref(),
        r.war.as_ref().map(|(m, _)| m.as_ref()),
        target,
        demand,
    )
    .map(Arc::new)
}

/// The influence around the lens's country: the player's own view when it is the player.
fn lens_influence_of(
    session: &Session,
    rules: &Rules,
    focus: Option<&str>,
    player: &Option<Arc<crate::influence_view::InfluenceView>>,
    last: &Option<Arc<crate::influence_view::InfluenceView>>,
) -> Option<Arc<crate::influence_view::InfluenceView>> {
    let focus = focus?;
    let date = session.clock().date;
    for v in [player, last].into_iter().flatten() {
        if v.month == (date.year, date.month) && v.tag == focus {
            return Some(Arc::clone(v));
        }
    }
    crate::influence_view::read(
        session.world(),
        rules.as_ref().and_then(|r| r.influence.as_deref()),
        Some(focus),
        date,
    )
    .map(Arc::new)
}

/// The blocs, read again when the month or the borders changed since `last`.
fn blocs_of(
    session: &Session,
    rules: &Rules,
    extras: &Extras,
    last: &Option<Arc<crate::blocs_view::BlocsView>>,
) -> Option<Arc<crate::blocs_view::BlocsView>> {
    let date = session.clock().date;
    let changes = session.world().borders.changes.len();
    match last {
        Some(b) if b.month == (date.year, date.month) && b.changes == changes => {
            Some(Arc::clone(b))
        }
        _ => {
            let borders = rules.as_ref()?.borders.as_deref()?;
            let diplomacy = extras.diplomacy.as_deref()?;
            Some(Arc::new(crate::blocs_view::read(
                session.world(),
                diplomacy,
                borders,
                date,
            )))
        }
    }
}

/// The owners, read again only when a border operation ran since `last`.
fn owners_of(
    session: &Session,
    rules: &Rules,
    last: &Option<Arc<bf_sim::borders::op::OwnersView>>,
) -> Option<Arc<bf_sim::borders::op::OwnersView>> {
    let changes = session.world().borders.changes.len();
    match last {
        Some(o) if o.changes == changes => Some(Arc::clone(o)),
        _ => {
            let b = rules.as_ref()?.borders.as_ref()?;
            Some(Arc::new(bf_sim::borders::op::OwnersView::read(
                session.world(),
                b,
            )))
        }
    }
}

impl Monthly {
    fn read(session: &Session, rules: &Rules, extras: &Extras, last: &Monthly) -> Self {
        let influence = influence_of(session, rules, &last.influence);
        let treaty_table = treaty_table_of(session, rules, &last.treaty_table);
        Self {
            treaty_draft: treaty_draft_of(session, rules, extras, &treaty_table, last),
            treaty_table,
            lens_influence: lens_influence_of(
                session,
                rules,
                extras.focus.as_deref(),
                &influence,
                &last.lens_influence,
            ),
            blocs: blocs_of(session, rules, extras, &last.blocs),
            crises: crises_of(session, rules, &last.crises),
            crisis_draft: draft_of(session, rules, extras, &last.crisis_draft),
            works: works_of(session, rules, &last.works),
            agreements: agreements_of(session, rules, &last.agreements),
            trade: trade_of(session, &last.trade),
            war: war_of(session, rules, &last.war),
            control: control_of(session, &last.control),
            figures: figures_of(session, &last.figures),
            journal: journal_of(session, rules, &last.journal),
            population: population_of(session, rules, &last.population),
            demands: demands_of(session, rules, &last.demands),
            ledger: ledger_of(session, &last.ledger),
            influence,
            owners: owners_of(session, rules, &last.owners),
            liberation: liberation_of(session, rules, &last.liberation),
            forces: forces_of(session, rules, &last.forces),
        }
    }
}

fn snapshot_of(
    session: &Session,
    paused: bool,
    speed: u8,
    hash: String,
    monthly: Monthly,
    rules: &Rules,
) -> Snapshot {
    Snapshot {
        date: session.clock().date,
        tick: session.clock().tick,
        paused,
        speed,
        hash,
        player: session.player().map(str::to_owned),
        research: research_view(session),
        local: Arc::new(local_view(session)),
        figures: monthly.figures,
        journal: monthly.journal,
        population: monthly.population,
        demands: monthly.demands,
        ledger: monthly.ledger,
        influence: monthly.influence,
        owners: monthly.owners,
        liberation: monthly.liberation,
        forces: monthly.forces,
        lens_influence: monthly.lens_influence,
        blocs: monthly.blocs,
        crises: monthly.crises,
        crisis_draft: monthly.crisis_draft,
        works: monthly.works,
        agreements: monthly.agreements,
        trade: monthly.trade,
        war: monthly.war,
        control: monthly.control,
        treaty_table: monthly.treaty_table,
        treaty_draft: monthly.treaty_draft,
        government: crate::government::read(
            session.world(),
            session.player(),
            session.clock().date,
            rules.as_ref().and_then(|r| r.catalogue.research),
            rules.as_ref().and_then(|r| {
                let price = r
                    .catalogue
                    .goods
                    .iter()
                    .find(|g| g.key == "schooling")
                    .map_or(0.0, |g| g.base_price.to_f64_lossy());
                r.education.as_deref().map(|e| (e, price))
            }),
        )
        .map(|mut g| {
            // The purchases' view needs the catalogue, which the government's reading has not.
            g.purchases = purchases_of(session, rules);
            g
        })
        .map(Arc::new),
        laws: crate::laws_view::Live::read(session.world(), session.player()).map(Arc::new),
    }
}

/// The player's purchases lever (D3), read with the government's view.
fn purchases_of(session: &Session, rules: &Rules) -> Option<bf_sim::econ::levers::PurchasesView> {
    let w = session.world();
    let ci = w
        .player_tag()
        .and_then(|t| w.countries.iter().position(|(_, c)| c.tag == t))?;
    let m = w.econ.markets.get(ci)?;
    Some(bf_sim::econ::levers::purchases_view(
        m,
        &rules.as_ref()?.catalogue,
    ))
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
    pub fn start(
        session: Session,
        rules: Rules,
        extras: Extras,
        repaint: impl Fn() + Send + 'static,
    ) -> Self {
        let monthly = Monthly::read(&session, &rules, &extras, &Monthly::default());
        let hash = short(&session.state_hash());
        let snapshot = Arc::new(Mutex::new(snapshot_of(
            &session,
            true,
            2,
            hash,
            monthly.clone(),
            &rules,
        )));
        let (tx, rx) = channel();
        let shared = Arc::clone(&snapshot);
        std::thread::Builder::new()
            .name("simulation".into())
            .spawn(move || run(session, rules, extras, monthly, rx, shared, repaint))
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
        self.save_to(path, Vec::new(), label)
    }

    /// [`Self::save_later`] writing the same save to `path` and to each of `copies`: the world is
    /// serialised once.
    pub fn save_to(
        &self,
        path: PathBuf,
        copies: Vec<PathBuf>,
        label: String,
    ) -> Receiver<Result<(), String>> {
        let (reply, answer) = channel();
        self.send(Command::Save {
            path,
            copies,
            label,
            reply,
        });
        answer
    }
}

fn write_save(session: &Session, paths: &[&std::path::Path], label: &str) -> Result<(), String> {
    let bytes = session
        .save(crate::saves::GAME_VERSION, label)
        .map_err(|e| e.to_string())?;
    for path in paths {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
        std::fs::rename(tmp, path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The loop's state that commands change.
struct Control {
    paused: bool,
    speed: u8,
    /// A command changed what the interface shows.
    changed: bool,
    /// The played country changed: the monthly views are read again now.
    reread: bool,
    /// The influence lens's country, as the last `LensFocus` set it.
    focus: Option<String>,
    /// The demand being drafted, as the last `CrisisDraft` set it.
    draft: Option<(String, bf_sim::treaty::Proposal)>,
    /// The proposal on the negotiation table, as the last `TreatyDraft` set it.
    treaty_draft: Option<(String, bf_sim::treaty::Proposal)>,
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
        Command::CrisisDraft(d) => {
            if ctl.draft != d {
                ctl.draft = d;
                ctl.changed = true;
            }
        }
        Command::TreatyDraft(d) => {
            if ctl.treaty_draft != d {
                ctl.treaty_draft = d;
                ctl.changed = true;
            }
        }
        Command::LensFocus(tag) => {
            if ctl.focus != tag {
                ctl.focus = tag;
                ctl.changed = true;
            }
        }
        // Applies at the start of the next tick and enters the history then.
        Command::Issue(c) => {
            if matches!(
                c,
                GameCommand::SwitchCountry { .. } | GameCommand::ChoosePlayer { .. }
            ) {
                ctl.reread = true;
                ctl.changed = true;
            }
            session.submit(c)
        }
        Command::Save {
            path,
            copies,
            label,
            reply,
        } => {
            let paths: Vec<&std::path::Path> = std::iter::once(path.as_path())
                .chain(copies.iter().map(PathBuf::as_path))
                .collect();
            let _ = reply.send(write_save(session, &paths, &label));
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
    rules: Rules,
    mut extras: Extras,
    mut monthly: Monthly,
    rx: Receiver<Command>,
    shared: Arc<Mutex<Snapshot>>,
    repaint: impl Fn(),
) {
    let mut ctl = Control {
        paused: true,
        speed: 2,
        changed: false,
        reread: false,
        focus: extras.focus.clone(),
        draft: extras.draft.clone(),
        treaty_draft: extras.treaty_draft.clone(),
    };
    // The hash shown, and the tick it belongs to: computed only when a snapshot shows a new
    // one (on pause, and on the first of the month).
    let mut hash = shared
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .hash
        .clone();
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
            if std::mem::take(&mut ctl.reread) {
                monthly = Monthly::default();
            }
            extras.focus.clone_from(&ctl.focus);
            extras.draft.clone_from(&ctl.draft);
            extras.treaty_draft.clone_from(&ctl.treaty_draft);
            monthly = Monthly::read(&session, &rules, &extras, &monthly);
            let snap = snapshot_of(
                &session,
                ctl.paused,
                ctl.speed,
                hash.clone(),
                monthly.clone(),
                &rules,
            );
            *shared.lock().unwrap_or_else(|e| e.into_inner()) = snap;
            repaint();
        }
    }
}
