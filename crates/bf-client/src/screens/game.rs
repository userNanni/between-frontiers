//! The campaign: map, top bar with date and speed, in-game menu.

use bf_ui_kit::{Button, tokens::space};
use egui::{Color32, Rect, Sense, Ui};

use super::government::Tab;
use super::{country_panel, pick::paint_tip, region, top_bar};
use crate::app::{Shared, Transition};
use crate::hud::PanelId;
use crate::keys::{self, id};
use crate::map::MapMode;
use crate::points::{self, Kind, Point, Reason};
use crate::saves::{self, Save};
use crate::sim::{Command, SimHandle};
use crate::text::country_name;
use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};

pub struct State {
    save: Save,
    sim: SimHandle,
    selected: Option<usize>,
    /// The state clicked on the map (index into the map's states); its panel shows in the
    /// country panel's place, with its country above it.
    selected_state: Option<usize>,
    menu_open: bool,
    rules: bf_sim::game::Rules,
    /// The principal each 1925 debt between governments was signed with.
    signed: crate::ledger_view::Signed,
    research_open: bool,
    /// The government window while it is open.
    government: Option<super::government::Window>,
    /// The journal window while it is open.
    journal: Option<super::journal::Window>,
    /// The influence window while it is open.
    influence: Option<super::influence::Window>,
    /// The economy window (buildings), while open.
    economy: Option<super::economy::Window>,
    /// The war window, while open.
    war: Option<super::war::Window>,
    /// What the side panel shows folded.
    fold: super::outliner::Fold,
    /// Arranging the HUD.
    edit: super::hud_edit::Edit,
    /// The folded map mode picker is open.
    lens_open: bool,
    /// The sea lens' route and preview.
    sea: super::sea_lens::SeaView,
    /// The overview, folded into a button for want of room, is open.
    overview_open: bool,
    /// The state whose pyramid the population panel shows.
    population: super::population::Scope,
    research_camera: Option<super::research::TreeCamera>,
    /// Month the live map lenses were last coloured for.
    live_month: Option<(i32, u8)>,
    /// Figures the HUD shows, as last read from the simulation.
    live_figures: Option<std::sync::Arc<crate::live::LiveFigures>>,
    /// The influence and blocs the map was last coloured with.
    lens_seen: (
        Option<std::sync::Arc<crate::influence_view::InfluenceView>>,
        Option<std::sync::Arc<crate::blocs_view::BlocsView>>,
    ),
    /// The control of the wars the map was last coloured with.
    control_seen: Option<std::sync::Arc<crate::war_view::ControlView>>,
    /// The country the influence lens was last asked to look at.
    lens_focus: Option<String>,
    /// The demand draft last sent to be weighed.
    crisis_draft: Option<(String, bf_sim::treaty::Proposal)>,
    /// The proposal on the negotiation table, as last sent to be weighed.
    treaty_draft: Option<(String, bf_sim::treaty::Proposal)>,
    last_autosave: laplace_schelling::calendar::Date,
    /// Saves the simulation is writing while the interface keeps drawing.
    writing: Vec<Writing>,
    /// The name of a point being saved with a name, while its dialog is open.
    naming: Option<String>,
    /// The campaign folder was found past the size that warns, this session.
    warned_size: bool,
}

/// A save the simulation is writing: its answer, and what to record once it is on disk.
struct Writing {
    answer: std::sync::mpsc::Receiver<Result<(), String>>,
    /// The restore point it writes, if any.
    point: Option<Point>,
    /// Also the campaign's current state: the date and time the campaign file then records.
    /// `None` when it is not, or when a later save superseded it.
    world: Option<(laplace_schelling::calendar::Date, u64)>,
    /// Text of the toast once written.
    announce: Option<String>,
}

impl State {
    /// Opens the campaign's engine save, or starts the simulation when the campaign has none
    /// yet (a country was just chosen).
    pub fn new(shared: &Shared, save: Save, ctx: &egui::Context) -> Result<Self, String> {
        let path = save.dir.join(saves::WORLD_FILE);
        let root = shared
            .paths
            .data
            .parent()
            .ok_or("data folder has no parent")?;
        let data = GameData::load(root)?;
        let rules = Some(data.rules());
        let session = if path.is_file() {
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            let mut session = Session::load(&bytes, 0, rules.clone()).map_err(|e| e.to_string())?;
            // Another country chosen on the campaigns screen ("⋯ → Jogar com outro país").
            if let Some(tag) = &save.meta.player
                && session.player() != Some(tag.as_str())
            {
                session.submit(GameCommand::SwitchCountry { tag: tag.clone() });
            }
            session
        } else {
            let mut session = Session::new(
                data.new_world(),
                save.meta.seed,
                save.meta.date,
                0,
                rules.clone(),
            );
            if let Some(tag) = &save.meta.player {
                session.submit(GameCommand::ChoosePlayer { tag: tag.clone() });
            }
            session
        };
        let repaint = ctx.clone();
        let extras = crate::sim::Extras {
            diplomacy: data.diplomacy.clone().map(std::sync::Arc::new),
            focus: None,
            draft: None,
            treaty_draft: None,
        };
        let sim = SimHandle::start(session, rules.clone(), extras, move || {
            repaint.request_repaint()
        });
        let state = Self {
            last_autosave: save.meta.date,
            writing: Vec::new(),
            naming: None,
            warned_size: false,
            save,
            sim,
            selected: None,
            selected_state: None,
            menu_open: false,
            rules: data.rules(),
            signed: crate::ledger_view::signed_1925(&data),
            research_open: false,
            government: None,
            journal: None,
            influence: None,
            economy: None,
            war: None,
            fold: super::outliner::Fold::default(),
            edit: super::hud_edit::Edit::default(),
            lens_open: false,
            sea: super::sea_lens::SeaView::default(),
            overview_open: false,
            population: Default::default(),
            research_camera: None,
            live_month: None,
            live_figures: None,
            lens_seen: (None, None),
            control_seen: None,
            lens_focus: None,
            crisis_draft: None,
            treaty_draft: None,
        };
        if !path.is_file() {
            state.sim.save(path, state.save.meta.name.clone())?;
        }
        Ok(state)
    }

    /// Pins or unpins a journal entry and records it with the campaign.
    fn toggle_pin(&mut self, shared: &mut Shared, id: &str, now: f64) {
        let pinned = &mut self.save.meta.pinned;
        match pinned.iter().position(|p| p == id) {
            Some(i) => {
                pinned.remove(i);
            }
            None => pinned.push(id.to_owned()),
        }
        if let Err(e) = saves::write(&self.save) {
            let msg = shared.i18n.tr_args(
                "saves-error-io",
                &[("error", bf_i18n::Arg::String(e.to_string()))],
            );
            shared.toast(msg, now);
        }
    }

    /// Tab of the government window, while it is open.
    pub fn government_tab(&self) -> Option<super::government::Tab> {
        self.government.as_ref().map(|w| w.tab)
    }

    /// Selects a country (index into the map's countries), as a click on it does (tests).
    pub fn select(&mut self, country: Option<usize>) {
        self.selected = country;
        self.selected_state = None;
    }

    /// Selects a state (index into the map's states) and its owner, as a click on it does
    /// (tests).
    pub fn select_state(&mut self, world: &crate::world::World, state: usize) {
        self.selected = world.states.get(state).and_then(|s| s.owner);
        self.selected_state = self.selected.is_some().then_some(state);
    }

    /// The sea lens' route and preview (tests and commands set them).
    pub fn sea_view_mut(&mut self) -> &mut super::sea_lens::SeaView {
        &mut self.sea
    }

    /// What the simulation last published.
    pub fn snapshot(&self) -> crate::sim::Snapshot {
        self.sim.snapshot()
    }

    /// Writes an automatic restore point before a risky or irreversible decision, when the
    /// player's settings ask for it: the one door for every system that takes such a decision
    /// (a law, freeing territory, accepting an agreement; later war and treaties). Call it
    /// before sending the decision's command, so the point holds the world before it.
    pub fn restore_point_before(&mut self, shared: &Shared, reason: Reason) {
        if shared.settings.point_before_risk {
            self.write_point(Kind::Before(reason), None, None, false);
        }
    }

    /// Asks the simulation for a restore point (and the current state with it when `world`);
    /// the answer is read in [`Self::poll_writes`].
    fn write_point(
        &mut self,
        kind: Kind,
        name: Option<String>,
        announce: Option<String>,
        world: bool,
    ) {
        let id = if kind == Kind::Quick {
            points::QUICK.to_owned()
        } else {
            points::new_id(&self.save)
        };
        let date = self.sim.snapshot().date;
        let point = Point {
            id: id.clone(),
            kind,
            name,
            date,
            written: saves::now(),
            bytes: 0,
        };
        let mut paths = vec![points::save_path(&self.save, &id)];
        if world {
            // The current state first: a later manual save lands after it.
            paths.insert(0, self.save.dir.join(saves::WORLD_FILE));
        }
        let main = paths.remove(0);
        let answer = self.sim.save_to(main, paths, self.save.meta.name.clone());
        self.writing.push(Writing {
            answer,
            point: Some(point),
            world: world.then(|| (date, saves::now())),
            announce,
        });
    }

    /// Records what the simulation finished writing: the points' files, the campaign's date.
    fn poll_writes(&mut self, shared: &mut Shared, now: f64) {
        let mut i = 0;
        while i < self.writing.len() {
            let result = match self.writing[i].answer.try_recv() {
                Ok(r) => r,
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    i += 1;
                    continue;
                }
                Err(e) => Err(e.to_string()),
            };
            let w = self.writing.remove(i);
            self.finish(shared, now, w, result);
        }
    }

    /// Waits for every save in flight (before the campaign goes back to a point).
    fn wait_writes(&mut self, shared: &mut Shared, now: f64) {
        for w in std::mem::take(&mut self.writing) {
            let result = w
                .answer
                .recv_timeout(std::time::Duration::from_secs(60))
                .map_err(|e| e.to_string())
                .and_then(|r| r);
            self.finish(shared, now, w, result);
        }
    }

    fn finish(&mut self, shared: &mut Shared, now: f64, w: Writing, result: Result<(), String>) {
        let result = result.and_then(|()| {
            if let Some(p) = w.point {
                points::record(&self.save, p).map_err(|e| e.to_string())?;
                points::prune(&self.save, shared.settings.limits());
            }
            if let Some((date, played)) = w.world {
                self.save.meta.date = date;
                self.save.meta.last_played = played;
                saves::write(&self.save).map_err(|e| e.to_string())?;
            }
            Ok(())
        });
        match result {
            Ok(()) => {
                if let Some(text) = w.announce {
                    shared.toast(text, now);
                }
                self.check_size(shared, now);
            }
            Err(e) => {
                let msg = shared
                    .i18n
                    .tr_args("saves-error-io", &[("error", bf_i18n::Arg::String(e))]);
                shared.toast(msg, now);
            }
        }
    }

    /// Warns once a session when the campaign folder passes the size the player set.
    fn check_size(&mut self, shared: &mut Shared, now: f64) {
        if self.warned_size {
            return;
        }
        let bytes = points::folder_bytes(&self.save.dir);
        if bytes > u64::from(shared.settings.warn_mb) * 1_000_000 {
            self.warned_size = true;
            let msg = shared.i18n.tr_args(
                "points-folder-large",
                &[
                    ("size", bf_i18n::Arg::String(mb(&shared.i18n, bytes))),
                    (
                        "limit",
                        bf_i18n::Arg::String(shared.settings.warn_mb.to_string()),
                    ),
                ],
            );
            shared.toast(msg, now);
        }
    }

    /// Goes back to the point `id`: writes where the player is now as a point (so the jump can
    /// be undone), makes the point the campaign's state and reopens the campaign on it.
    fn go_back_to(&mut self, shared: &mut Shared, now: f64, id: &str) -> Option<Transition> {
        self.wait_writes(shared, now);
        self.write_point(Kind::Before(Reason::Load), None, None, false);
        self.wait_writes(shared, now);
        let mut save = self.save.clone();
        match points::restore(&mut save, id) {
            Ok(()) => Some(Transition::Game(save)),
            Err(e) => {
                let msg = shared.i18n.tr_args(
                    "saves-error-io",
                    &[("error", bf_i18n::Arg::String(e.to_string()))],
                );
                shared.toast(msg, now);
                None
            }
        }
    }

    fn write(&mut self, shared: &mut Shared, now: f64, announce: bool) {
        // Saves in flight were queued first: this one's world lands after theirs and wins.
        for w in &mut self.writing {
            w.world = None;
        }
        self.save.meta.date = self.sim.snapshot().date;
        self.save.meta.last_played = saves::now();
        let result = self
            .sim
            .save(
                self.save.dir.join(saves::WORLD_FILE),
                self.save.meta.name.clone(),
            )
            .and_then(|()| saves::write(&self.save).map_err(|e| e.to_string()));
        match result {
            Ok(()) if announce => shared.toast(shared.tr("game-menu-saved"), now),
            Ok(()) => {}
            Err(e) => {
                let msg = shared
                    .i18n
                    .tr_args("saves-error-io", &[("error", bf_i18n::Arg::String(e))]);
                shared.toast(msg, now);
            }
        }
    }
}

pub fn show(state: &mut State, shared: &mut Shared, ui: &mut Ui, rect: Rect) -> Option<Transition> {
    let mut out = None;
    let now = ui.input(|i| i.time);
    let snap = state.sim.snapshot();

    // Automatic restore point on the interval's calendar boundary: the simulation writes it
    // (and the current state with it) while the interface keeps drawing.
    if points::due(shared.settings.autosave, state.last_autosave, snap.date)
        && !state.writing.iter().any(|w| w.world.is_some())
    {
        state.last_autosave = snap.date;
        state.write_point(Kind::Auto, None, None, true);
    }
    state.poll_writes(shared, now);

    let player = shared.world.as_ref().and_then(|w| {
        state
            .save
            .meta
            .player
            .as_ref()
            .and_then(|t| w.countries.iter().position(|c| c.tag() == t))
    });

    // Live map lenses: the world takes the simulation's figures; the map recolours once a
    // month, or at once when the lens opens. The HUD's country figures change with the month too.
    if let Some(world) = &mut shared.world {
        world.live_local = std::sync::Arc::clone(&snap.local);
        // New month, or the world lost its figures on a trip through another screen.
        let fresh = world.live.is_some() != snap.figures.is_some()
            || match (&snap.figures, &state.live_figures) {
                (Some(new), Some(old)) => !std::sync::Arc::ptr_eq(new, old),
                (new, old) => new.is_some() != old.is_some(),
            };
        if fresh {
            world.live = snap
                .figures
                .as_deref()
                .map(|f| crate::world::FigureTable::live(&world.countries, f));
            state.live_figures = snap.figures.clone();
        }
        world.live_influence = snap.lens_influence.clone();
        world.live_blocs = snap.blocs.clone();
        world.live_control = snap.control.clone();
        let month = (snap.date.year, snap.date.month);
        let lens_data_new = !same(&snap.lens_influence, &state.lens_seen.0)
            || !same(&snap.blocs, &state.lens_seen.1)
            || !same(&snap.control, &state.control_seen);
        if let Some(view) = &mut shared.map_view {
            use crate::map::lens::Lens;
            let live = match view.lens {
                Lens::Local(_) => state.live_month != Some(month),
                Lens::Influence { .. } | Lens::Blocs(_) | Lens::War => lens_data_new,
                _ => false,
            };
            if live {
                view.set_palette(crate::map::lens::palette(world, view.lens, snap.date.year));
            }
            // The influence lens looks at the selected country, or the player's.
            let focus = matches!(view.lens, Lens::Influence { .. })
                .then(|| state.selected.or(player))
                .flatten()
                .and_then(|c| world.countries.get(c))
                .map(|c| c.tag().to_owned());
            if focus != state.lens_focus {
                state.lens_focus.clone_from(&focus);
                state.sim.send(crate::sim::Command::LensFocus(focus));
            }
        }
        state.lens_seen = (snap.lens_influence.clone(), snap.blocs.clone());
        state.control_seen = snap.control.clone();
        state.live_month = Some(month);
    }
    // Border changes: owners, new countries, the map's colours and borders, the flags.
    if let (Some(owners), Some(world)) = (&snap.owners, &mut shared.world)
        && world.apply_owners(owners)
    {
        world.live = state
            .live_figures
            .as_deref()
            .map(|f| crate::world::FigureTable::live(&world.countries, f))
            .or(world.live.take());
        shared.rebuild_map(ui.ctx());
        // Annexed, or beaten by a revolt: the choice of another country is needed now, not
        // offered (decision 4 of `docs/borders.pt-BR.md`).
        let gone = shared.world.as_ref().is_some_and(|w| {
            state.save.meta.player.as_ref().is_some_and(|tag| {
                w.countries
                    .iter()
                    .find(|c| c.tag() == tag)
                    .is_some_and(|c| c.states.is_empty() && c.colonial_states.is_empty())
            })
        });
        if gone {
            state.write(shared, now, false);
            shared.toast(shared.tr("game-country-gone"), now);
            return Some(Transition::SwitchCountry(state.save.clone()));
        }
    }
    // Where the HUD's panels go: the player's layout resolved for this screen and the sizes the
    // panels ask for; worked out again only when one of them changes.
    let sizes = hud_sizes(state, shared, ui.ctx(), rect, player);
    let places = shared.hud.resolved(rect, &sizes).clone();
    let place = |p: PanelId| places.rect(p);

    let map_rect = rect;
    let mut fold_lens = false;
    // The sea lens reads its layer the first time it opens.
    let sea_lens = shared
        .map_view
        .as_ref()
        .is_some_and(|v| v.lens == crate::map::lens::Lens::Sea);
    if sea_lens && shared.sea.is_none() {
        shared.sea = Some(load_sea(shared, ui.ctx()));
    }
    let sea_layer = shared
        .sea
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .cloned()
        .filter(|_| sea_lens);
    let sea_texture = shared.sea_texture.clone();
    let i18n = &shared.i18n;
    if let (Some(map), Some(view), Some(world)) = (&shared.map, &mut shared.map_view, &shared.world)
    {
        view.selected_owner = state.selected;
        let names = |c: usize| country_name(i18n, world.countries[c].tag());
        let response = view.show(ui, map_rect, map, world, &names, MapMode::Interactive);
        if let (Some(id), Some(layer)) = (response.clicked_province, &sea_layer) {
            // On the sea lens a click picks the ends of a route, not a country.
            if let Some(s) = world.provinces[id as usize - 1].state {
                state.sea.pick(layer, world.states[s].def.id);
            }
        } else if let Some(id) = response.clicked_province {
            // A click picks the state; its panel leads up to the country.
            let p = &world.provinces[id as usize - 1];
            state.selected = p.owner;
            state.selected_state = p.state.filter(|_| p.owner.is_some());
        }
        if let (Some(layer), Some(texture), Some(camera)) = (&sea_layer, &sea_texture, view.camera)
            && !state.research_open
        {
            let date = format!(
                "{:04}-{:02}-{:02}",
                snap.date.year, snap.date.month, snap.date.day
            );
            let res = super::sea_lens::show(
                ui,
                map_rect,
                rect,
                &super::sea_lens::Sources {
                    i18n,
                    world,
                    layer,
                    texture,
                    camera,
                    date: &date,
                    month: snap.date.month,
                },
                &mut state.sea,
            );
            if let Some(s) = res.toggle
                && !state.sea.closed.remove(&s)
            {
                state.sea.closed.insert(s);
            }
            if res.clear {
                state.sea.clear();
            }
        }
        if response.clicked_sea {
            state.selected = None;
            state.selected_state = None;
        }
        // Nothing of the map shows through the research screen.
        if !state.research_open
            && let (Some(id), Some(pos)) = (
                response.hovered_province,
                ui.input(|i| i.pointer.hover_pos()),
            )
            && let Some((state_name, owner)) = world.place_name(i18n, id)
        {
            let owner = owner.map(names).unwrap_or_else(|| i18n.tr("map-unowned"));
            let extra = world.provinces[id as usize - 1].state.and_then(|s| {
                super::lens_panel::tooltip_line(i18n, world, s, view.lens, snap.date.year)
            });
            paint_tip(ui, pos, &state_name, &owner, extra.as_deref());
        }
        // The government and journal windows cover the map; the mode picker would show
        // through.
        if let Some(r) = place(PanelId::MapModes) {
            if shared.hud.layout.map_modes_open {
                // Kept open: the panel as it always was, with a link to fold it.
                fold_lens |=
                    super::lens_panel::show(ui, r, i18n, world, view, snap.date.year, Some(true))
                        .toggle_pin;
            } else {
                // Folded into a button; open, it shows the panel over the rest of the HUD,
                // further down.
                if super::lens_panel::button(ui, r, i18n, view.lens, state.lens_open)
                    && !state.edit.on
                {
                    toggle_lens_picker(state);
                }
            }
        }
    }
    if fold_lens {
        shared.hud.layout.map_modes_open = false;
        shared.hud.changed();
        shared.persist_hud();
    }

    // Side panel under the country figures, above the map mode picker, while no window covers
    // the map.
    if keys::fired(ui.ctx(), id::OUTLINER)
        && state.government.is_none()
        && state.journal.is_none()
        && !state.research_open
    {
        state.fold.open = !state.fold.open;
    }
    if let Some(r) = place(PanelId::Outliner) {
        if places.folded.contains(&PanelId::Outliner) {
            // Folded for want of room: a button; open, the panel shows over the HUD, further
            // down.
            if super::outliner::folded_button(ui, r, &shared.i18n, state.overview_open)
                && !state.edit.on
            {
                state.overview_open = !state.overview_open;
            }
        } else {
            overview(state, shared, ui, r, &snap, now);
        }
    }

    // Top bar
    let tiles = shared.hud.layout.tiles();
    let bar_out = top_bar::show(
        ui,
        rect,
        top_bar::Places {
            corner: place(PanelId::Country),
            banner_only: places.folded.contains(&PanelId::Country),
            time: place(PanelId::Time),
        },
        &shared.i18n,
        shared.world.as_ref(),
        &shared.flags,
        player,
        top_bar::Extra {
            snap: &snap,
            tree: state.rules.tree.as_deref(),
            tiles: &tiles,
        },
    );
    if bar_out.open_country {
        state.selected = player;
        state.selected_state = None;
    }
    if bar_out.toggle_pause {
        state.sim.send(Command::SetPaused(!snap.paused));
    }
    if let Some(speed) = bar_out.speed {
        state.sim.send(Command::SetSpeed(speed));
    }
    if bar_out.open_research {
        state.research_open = !state.research_open;
        state.government = None;
        state.journal = None;
        state.influence = None;
        state.economy = None;
        state.war = None;
    }
    if bar_out.open_government {
        toggle_government(state);
    }
    if bar_out.open_journal {
        toggle_journal(state);
    }
    if bar_out.open_economy {
        toggle_economy(state);
    }
    if bar_out.open_population && !state.edit.on {
        toggle_population(state, shared, rect);
        shared.persist_hud();
    }
    if let Some(screen) = bar_out.open {
        open_from_tile(state, screen);
    }
    if bar_out.open_menu {
        state.menu_open = true;
        state.sim.send(Command::SetPaused(true));
    }

    // Country panel for the clicked country, or the clicked state's panel.
    let mut go_up = false;
    if let (Some(country), Some(side)) = (
        state.selected.or(player.filter(|_| state.edit.on)),
        place(PanelId::CountryPanel),
    ) {
        super::frame(ui, side, "country-panel");
        ui.interact(
            side,
            ui.make_persistent_id("game-side"),
            Sense::click_and_drag(),
        );
        region(ui, side.shrink(space::XL), |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let state_shown = match (state.selected_state, &shared.world) {
                        (Some(s), Some(world)) if !state.edit.on => {
                            let figures = snap.states.as_ref().and_then(|v| v.states.get(s));
                            let res = super::state_panel::show(
                                ui,
                                &shared.i18n,
                                world,
                                &shared.flags,
                                s,
                                figures,
                                snap.date.year,
                            );
                            if res.to_country {
                                go_up = true;
                            }
                            true
                        }
                        _ => false,
                    };
                    if !state_shown {
                        country_panel::show(shared, ui, country);
                        // Its trade by partner, with trade by pair on.
                        if let (Some(view), Some(world)) = (&snap.trade, &shared.world) {
                            super::trade::show(ui, &shared.i18n, world, view, country);
                        }
                    }
                });
        });
    }

    if go_up {
        state.selected_state = None;
    }

    // The player's population, read once a month.
    if let Some(r) = place(PanelId::Population) {
        match (&snap.population, &shared.world) {
            (Some(view), Some(world)) => {
                super::population::show(ui, r, &shared.i18n, world, view, &mut state.population)
            }
            _ => super::frame(ui, r, "population"),
        }
    }

    // The folded overview, open over the HUD; a click elsewhere folds it.
    if state.overview_open
        && places.folded.contains(&PanelId::Outliner)
        && let Some(r) = place(PanelId::Outliner)
    {
        let catcher = ui.interact(
            rect,
            ui.make_persistent_id("overview-outside"),
            Sense::click(),
        );
        let ctx = ui.ctx().clone();
        let fold = state.fold;
        let popup = lens_popup(
            r,
            |max_h| {
                let s = super::outliner::size(&ctx, &fold);
                egui::vec2(s.x, s.y.min(max_h))
            },
            rect,
            places.free,
        );
        keys::cover(ui, popup);
        overview(state, shared, ui, popup, &snap, now);
        if catcher.clicked() {
            state.overview_open = false;
        }
    } else if !places.folded.contains(&PanelId::Outliner) {
        state.overview_open = false;
    }

    // The open map mode picker, over every panel of the HUD: a click anywhere else folds it.
    if state.lens_open
        && !shared.hud.layout.map_modes_open
        && let Some(r) = place(PanelId::MapModes)
        && let (Some(view), Some(world)) = (&mut shared.map_view, &shared.world)
    {
        let catcher = ui.interact(rect, ui.make_persistent_id("lens-outside"), Sense::click());
        let ctx = ui.ctx().clone();
        let lens = view.lens;
        let free = shared.hud.layout.free.map(|a| {
            let area = crate::hud::hud_area(&shared.hud.layout, rect);
            crate::hud::clamp_into(a.rect(area), area)
        });
        let popup = lens_popup(
            r,
            |max_h| super::lens_panel::size(&ctx, lens, max_h),
            rect,
            free,
        );
        // Open over the HUD: the tips of what it covers go.
        keys::cover(ui, popup);
        let res = super::lens_panel::show(
            ui,
            popup,
            &shared.i18n,
            world,
            view,
            snap.date.year,
            Some(false),
        );
        if res.chosen || catcher.clicked() {
            state.lens_open = false;
        }
        if res.toggle_pin {
            state.lens_open = false;
            shared.hud.layout.map_modes_open = true;
            shared.hud.changed();
            shared.persist_hud();
        }
    }

    // Research screen.
    if state.research_open
        && let (Some(tree), Some(view)) = (&state.rules.tree, &snap.research)
    {
        let camera = state.research_camera.get_or_insert_with(|| {
            super::research::TreeCamera::new(super::research::current_era(snap.date.year))
        });
        let res = super::research::show(
            ui,
            rect,
            &shared.i18n,
            tree,
            &state.rules.catalogue,
            view,
            camera,
        );
        if res.close {
            state.research_open = false;
        }
        if let (Some(focus), Some(tag)) = (res.focus, &state.save.meta.player) {
            state.sim.send(Command::Issue(GameCommand::SetResearch {
                tag: tag.clone(),
                tech: focus,
            }));
        }
    }

    // Government window.
    let mut gov_place = shared.hud.layout.window("government");
    if let Some(window) = &mut state.government
        && !state.research_open
    {
        let res = super::government::show(
            ui,
            rect,
            super::government::Sources {
                i18n: &shared.i18n,
                catalogue: &state.rules.catalogue,
                world: shared.world.as_ref(),
                snap: &snap,
                tag: state.save.meta.player.as_deref(),
                signed: &state.signed,
            },
            window,
            &mut gov_place,
        );
        keep_window_place(shared, "government", gov_place, res.place_done);
        // "Renegotiate" in the debts tab: the negotiation table, in the Agendas, with the
        // revision on it.
        if let Some(draft) = res.renegotiate.clone() {
            state.government = None;
            state.journal = Some(super::journal::Window {
                picked: Some(super::journal::TABLE.to_owned()),
                table: super::treaty_table::TableUi {
                    draft: Some(draft),
                    fixed: true,
                    title: Some("table-renegotiate-title"),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
        // Ships and aircraft disbanded leave service for good: a point to come back to first.
        if res
            .commands
            .iter()
            .any(|c| matches!(c, GameCommand::DisbandForces { .. }))
        {
            state.restore_point_before(shared, Reason::Disband);
        }
        for c in res.commands {
            if matches!(c, GameCommand::SetLaw { .. }) {
                state.restore_point_before(shared, Reason::Law);
            }
            // Breaking an agreement's tariff clause on purpose (CT2): a point to go back to.
            if matches!(c, GameCommand::SetPartnerTariff { breach: true, .. }) {
                state.restore_point_before(shared, Reason::Treaty);
            }
            state.sim.send(Command::Issue(c));
        }
        if res.close {
            state.government = None;
        }
    }

    // Economy window.
    if state.economy.is_some()
        && !state.research_open
        && state.government.is_none()
        && state.journal.is_none()
    {
        let mut place = shared.hud.layout.window("economy");
        let res = super::economy::show(
            ui,
            rect,
            &shared.i18n,
            shared.world.as_ref(),
            snap.buildings.as_deref(),
            state.economy.as_mut().expect("economy open"),
            &mut place,
        );
        keep_window_place(shared, "economy", place, res.place_done);
        if res.close {
            state.economy = None;
        }
        // A state line: its panel, the window closed so the map shows.
        if let Some(id) = state.economy.as_mut().and_then(|w| w.goto.take())
            && let Some(world) = &shared.world
        {
            let s = usize::from(id).saturating_sub(1);
            state.selected = world.states.get(s).and_then(|st| st.owner);
            state.selected_state = state.selected.is_some().then_some(s);
            state.economy = None;
        }
    }

    // Influence window.
    if state.influence.is_some()
        && !state.research_open
        && state.government.is_none()
        && state.journal.is_none()
    {
        let mut place = shared.hud.layout.window("influence");
        let res = super::influence::show(
            ui,
            rect,
            &shared.i18n,
            snap.influence.as_deref(),
            state.influence.as_mut().expect("influence open"),
            &mut place,
        );
        keep_window_place(shared, "influence", place, res.place_done);
        if res.close {
            state.influence = None;
            state.economy = None;
            state.war = None;
        }
    }

    // War window.
    if state.war.is_some()
        && !state.research_open
        && state.government.is_none()
        && state.journal.is_none()
    {
        let mut place = shared.hud.layout.window("war");
        let res = super::war::show(
            ui,
            rect,
            &shared.i18n,
            shared.world.as_ref(),
            snap.war.as_deref(),
            snap.forces.as_deref().and_then(|f| f.army.as_ref()),
            state.war.as_mut().expect("war open"),
            &mut place,
        );
        keep_window_place(shared, "war", place, res.place_done);
        for c in res.commands {
            state.sim.send(Command::Issue(c));
        }
        if let Some(c) = res.nuclear_use {
            // A use cannot be undone: a point to come back to first.
            state.restore_point_before(shared, Reason::War);
            state.sim.send(Command::Issue(c));
        }
        // Peace goes through the negotiation table, in the Agendas, with the enemy fixed.
        if let (Some(enemy), Some(me)) = (res.peace, snap.player.clone()) {
            state.war = None;
            state.journal = Some(super::journal::Window {
                picked: Some(super::journal::TABLE.to_owned()),
                table: super::treaty_table::TableUi {
                    draft: Some((
                        enemy.clone(),
                        bf_sim::treaty::Proposal {
                            side_a: vec![me],
                            side_b: vec![enemy],
                            clauses: Vec::new(),
                            term_years: None,
                            notice_years: 0,
                        },
                    )),
                    fixed: true,
                    title: Some("table-peace-title"),
                    ..Default::default()
                },
                ..Default::default()
            });
        } else if res.close {
            state.war = None;
        }
    }

    // Journal window.
    if state.journal.is_some() && !state.research_open && state.government.is_none() {
        let pinned = state.save.meta.pinned.clone();
        let mut journal_place = shared.hud.layout.window("journal");
        // A demand goes to the country selected on the map, when it is not the player's.
        let target = shared.world.as_ref().and_then(|w| {
            let c = w.countries.get(state.selected?)?;
            (Some(c.tag()) != snap.player.as_deref()).then(|| c.tag().to_owned())
        });
        let crises = super::journal::CrisisInput {
            view: snap.crises.as_deref(),
            draft: snap.crisis_draft.as_deref(),
            me: snap.player.as_deref(),
            target: target.as_deref(),
        };
        let table = super::journal::TableInput {
            view: snap.treaty_table.as_deref(),
            draft: snap.treaty_draft.as_deref(),
            catalogue: state
                .rules
                .treaties
                .as_ref()
                .and_then(|t| t.clauses.as_deref()),
            today: Some(snap.date),
            agreements: snap.agreements.as_deref(),
            trade: snap.tariffs.as_deref(),
        };
        let res = super::journal::show(
            ui,
            rect,
            &shared.i18n,
            shared.world.as_ref(),
            snap.journal.as_deref(),
            snap.liberation.as_deref(),
            snap.demands.as_deref(),
            crises,
            table,
            &pinned,
            state.journal.as_mut().expect("journal open"),
            &mut journal_place,
        );
        if let (Some((states, tag, subject)), Some(by)) = (res.liberate, snap.player.clone()) {
            // Freeing land cannot be undone in the game: a point to go back to first.
            state.restore_point_before(shared, Reason::Liberate);
            state.sim.send(Command::Issue(GameCommand::Liberate {
                by,
                states,
                tag,
                subject,
            }));
        }
        if let Some((unit, answer)) = res.answer {
            // Some answers cannot be undone in the game: a point to go back to first.
            state.restore_point_before(shared, Reason::Demand);
            state
                .sim
                .send(Command::Issue(GameCommand::AnswerDemand { unit, answer }));
        }
        keep_window_place(shared, "journal", journal_place, res.place_done);
        for id in res.toggle_pin {
            state.toggle_pin(shared, &id, now);
        }
        // Denouncing ends an agreement, or breaks it: a point to go back to first.
        if let (Some(agreement), Some(tag)) = (res.denounce, snap.player.clone()) {
            state.restore_point_before(shared, Reason::Denounce);
            state
                .sim
                .send(Command::Issue(GameCommand::Denounce { tag, agreement }));
        }
        // An embargo closes trade both ways from next week: a point to go back to first.
        if let (Some((target, on)), Some(tag)) = (res.embargo, snap.player.clone()) {
            let c = GameCommand::Embargo { tag, target, on };
            if matches!(c, GameCommand::Embargo { on: true, .. }) {
                state.restore_point_before(shared, Reason::Embargo);
            }
            state.sim.send(Command::Issue(c));
        }
        // The proposal on the negotiation table, too.
        if res.treaty_draft != state.treaty_draft {
            state.treaty_draft.clone_from(&res.treaty_draft);
            state
                .sim
                .send(Command::TreatyDraft(res.treaty_draft.clone()));
        }
        if let (Some(sent), Some(tag)) = (res.treaty, snap.player.clone()) {
            use super::treaty_table::Sent;
            match sent {
                Sent::Propose(to, proposal) => {
                    // The other side may sign it on Monday: a point to go back to first.
                    state.restore_point_before(shared, Reason::Treaty);
                    state.sim.send(Command::Issue(GameCommand::ProposeTreaty {
                        tag,
                        to,
                        proposal,
                    }));
                }
                Sent::Answer(id, answer) => {
                    // Accepting signs it; a counter may be accepted on Monday.
                    if !matches!(answer, bf_sim::treaty_table::ProposalAnswer::Refuse) {
                        state.restore_point_before(shared, Reason::Treaty);
                    }
                    state.sim.send(Command::Issue(GameCommand::AnswerProposal {
                        tag,
                        id,
                        answer,
                    }));
                }
            }
        }
        // The demand being drafted is weighed by the simulation thread.
        if res.draft != state.crisis_draft {
            state.crisis_draft.clone_from(&res.draft);
            state.sim.send(Command::CrisisDraft(res.draft.clone()));
        }
        if let Some(tag) = snap.player.clone() {
            // Demanding and declaring start what cannot be undone: a point to go back to first.
            if let Some((target, demand)) = res.crisis_demand {
                state.restore_point_before(shared, Reason::War);
                state.sim.send(Command::Issue(GameCommand::Demand {
                    tag: tag.clone(),
                    target,
                    demand,
                    weeks: None,
                }));
            }
            if let Some((crisis, accept)) = res.crisis_answer {
                state.restore_point_before(shared, Reason::War);
                state.sim.send(Command::Issue(GameCommand::AnswerCrisis {
                    tag: tag.clone(),
                    crisis,
                    accept,
                }));
            }
            if let Some((war, honour)) = res.call_answer {
                // Honoured, the country enters a war; refused, the breach is on the record.
                state.restore_point_before(shared, Reason::War);
                state.sim.send(Command::Issue(GameCommand::AnswerCall {
                    tag: tag.clone(),
                    war,
                    honour,
                }));
            }
            if let Some((crisis, war)) = res.declare {
                state.restore_point_before(shared, Reason::War);
                state
                    .sim
                    .send(Command::Issue(GameCommand::Declare { tag, crisis, war }));
            }
        }
        if let (Some(on), Some(tag)) = (res.mobilize, state.save.meta.player.clone()) {
            // Mobilizing moves men out of work for months: a point to come back to first.
            state.restore_point_before(shared, Reason::War);
            state
                .sim
                .send(Command::Issue(GameCommand::Mobilize { tag, on }));
        }
        if let (true, Some(tag)) = (res.programme_end, state.save.meta.player.clone()) {
            // Ending the programme stops the money and the warhead order at once: a point first.
            state.restore_point_before(shared, Reason::War);
            state
                .sim
                .send(Command::Issue(GameCommand::NuclearProgramme {
                    tag,
                    share: None,
                }));
        }
        if let (true, Some(tag)) = (res.nuclear_test, state.save.meta.player.clone()) {
            // The test cannot be undone: the arsenal is known to all from then on.
            state.restore_point_before(shared, Reason::War);
            state
                .sim
                .send(Command::Issue(GameCommand::NuclearTest { tag }));
        }
        if res.open_war {
            toggle_war(state);
        }
        if let Some(screen) = res.open {
            use super::government::{Tab, Window};
            state.journal = None;
            state.influence = None;
            state.economy = None;
            state.war = None;
            let tab = match screen {
                crate::journal::Screen::Budget => Some(Tab::Budget),
                crate::journal::Screen::Money => Some(Tab::Money),
                crate::journal::Screen::Laws => Some(Tab::Laws),
                crate::journal::Screen::Forces => Some(Tab::Forces),
                crate::journal::Screen::Research => None,
            };
            match tab {
                Some(tab) => {
                    state.government = Some(Window {
                        tab,
                        ..Window::default()
                    })
                }
                None => state.research_open = true,
            }
        }
        if res.close {
            state.journal = None;
            state.influence = None;
            state.economy = None;
            state.war = None;
        }
    }

    // HUD panels and presets, in and out of the edit mode; the map lenses, folded or not.
    if !state.menu_open {
        let ctx = ui.ctx().clone();
        let mut changed = false;
        if keys::fired(&ctx, id::HUD_EDIT_BLOCKS) {
            super::hud_edit::edit_blocks(&mut state.edit, super::hud_edit::Blocks::Country);
        }
        if keys::fired(&ctx, id::HUD_EDIT_SECTIONS) {
            super::hud_edit::edit_blocks(&mut state.edit, super::hud_edit::Blocks::Overview);
        }
        if keys::fired(&ctx, id::LENS_PICKER) && !shared.hud.layout.map_modes_open {
            toggle_lens_picker(state);
        }
        if keys::fired(&ctx, id::LENS_KEEP_OPEN) {
            shared.hud.layout.map_modes_open = !shared.hud.layout.map_modes_open;
            state.lens_open = false;
            changed = true;
        }
        if let (Some(view), Some(world)) = (&mut shared.map_view, &shared.world)
            && let Some(mode) = id::MODES.iter().position(|c| keys::fired(&ctx, c))
        {
            let lens = super::lens_panel::mode_lens(mode, view.lens);
            super::lens_panel::choose(view, world, lens, snap.date.year);
        }
        if keys::fired(&ctx, id::POPULATION) {
            toggle_population(state, shared, ctx.content_rect());
            changed = true;
        }
        for (p, cmd) in PanelId::ALL.into_iter().zip(id::HUD_PANELS) {
            if keys::fired(&ctx, cmd) {
                crate::hud::toggle_panel(&mut shared.hud, p);
                changed = true;
            }
        }
        for (p, cmd) in crate::hud::Preset::CHOICES.into_iter().zip(id::HUD_PRESETS) {
            if keys::fired(&ctx, cmd) {
                crate::hud::apply_preset(&mut shared.hud, p);
                changed = true;
            }
        }
        if changed {
            shared.persist_hud();
        }
    }

    // Arranging the HUD: over everything but the in-game menu; the other keys wait.
    if keys::fired(ui.ctx(), id::HUD_EDIT) && !state.menu_open {
        toggle_hud_edit(state);
    }
    if state.edit.on && !state.menu_open {
        let res = super::hud_edit::show(
            ui,
            rect,
            &shared.i18n,
            &shared.paths.config.clone(),
            &mut shared.hud,
            &sizes,
            &mut state.edit,
        );
        if res.save {
            shared.persist_hud();
        }
        if let Some(m) = res.message {
            shared.toast(m, now);
        }
        if keys::fired(ui.ctx(), id::BACK) {
            super::hud_edit::toggle(&mut state.edit);
        }
        return out;
    }

    // Keyboard: every key arrives as a command.
    let ctx = ui.ctx().clone();
    let fired = |cmd: &str| keys::fired(&ctx, cmd);
    if !state.menu_open {
        if fired(id::RESEARCH) {
            state.research_open = !state.research_open;
            state.government = None;
            state.journal = None;
            state.influence = None;
            state.economy = None;
            state.war = None;
        }
        if fired(id::GOVERNMENT) {
            toggle_government(state);
        }
        if fired(id::JOURNAL) {
            toggle_journal(state);
        }
        if fired(id::INFLUENCE) {
            toggle_influence(state);
        }
        if fired(id::ECONOMY) {
            toggle_economy(state);
        }
        if fired(id::WAR) {
            toggle_war(state);
        }
        for (cmd, tab) in [
            (id::TAB_BUDGET, Tab::Budget),
            (id::TAB_MONEY, Tab::Money),
            (id::TAB_LAWS, Tab::Laws),
            (id::TAB_LEDGER, Tab::Ledger),
            (id::TAB_FORCES, Tab::Forces),
            (id::TAB_WORKS, Tab::Works),
            (id::TAB_TRADE, Tab::Trade),
        ] {
            if fired(cmd) {
                state.research_open = false;
                state.journal = None;
                state.influence = None;
                state.economy = None;
                state.war = None;
                state.government.get_or_insert_with(Default::default).tab = tab;
            }
        }
        if fired(id::COUNTRY) {
            state.selected = player;
            state.selected_state = None;
        }
        if fired(id::PAUSE) {
            state.sim.send(Command::SetPaused(!snap.paused));
        }
        if fired(id::SPEED_DOWN) {
            state
                .sim
                .send(Command::SetSpeed(snap.speed.saturating_sub(1).max(1)));
        }
        if fired(id::SPEED_UP) {
            state.sim.send(Command::SetSpeed((snap.speed + 1).min(5)));
        }
        if let Some(s) = id::SPEEDS.iter().position(|c| fired(c)) {
            state.sim.send(Command::SetSpeed(s as u8 + 1));
        }
        if fired(id::MENU) {
            state.menu_open = true;
            state.sim.send(Command::SetPaused(true));
        } else if fired(id::BACK) {
            if state.lens_open {
                state.lens_open = false;
            } else if state.research_open {
                state.research_open = false;
            } else if state.government.is_some() {
                state.government = None;
            } else if state.journal.is_some()
                || state.influence.is_some()
                || state.economy.is_some()
            {
                state.journal = None;
                state.influence = None;
                state.economy = None;
                state.war = None;
            } else if state.selected_state.is_some() {
                // Up from the state to its country.
                state.selected_state = None;
            } else if state.selected.is_some() {
                state.selected = None;
                state.selected_state = None;
            } else {
                state.menu_open = true;
                state.sim.send(Command::SetPaused(true));
            }
        }
    } else if fired(id::BACK) || fired(id::MENU) {
        state.menu_open = false;
    }
    if fired(id::SAVE) {
        state.write(shared, now, true);
    }
    if state.naming.is_none() {
        if fired(id::QUICK_SAVE) {
            quick_save(state, shared);
        }
        if fired(id::QUICK_LOAD) {
            out = out.or_else(|| quick_load(state, shared, now));
        }
        if fired(id::SAVE_NAMED) {
            state.naming = Some(default_point_name(shared, &state.save, &snap));
            state.menu_open = false;
        }
    }
    if fired(id::SETTINGS) || fired(id::KEYS) {
        state.menu_open = false;
        if fired(id::KEYS) {
            shared.key_editor = Some(super::keys_editor::State::default());
        }
        out = Some(Transition::Settings);
    }

    // In-game menu
    if state.menu_open {
        ui.painter()
            .rect_filled(rect, 0, Color32::from_black_alpha(140));
        let panel = Rect::from_center_size(rect.center(), egui::vec2(360.0, 400.0));
        keys::cover(ui, rect);
        super::frame(ui, panel, "game-menu");
        ui.interact(
            rect,
            ui.make_persistent_id("menu-block"),
            Sense::click_and_drag(),
        );
        region(ui, panel.shrink(space::XL), |ui| {
            ui.spacing_mut().item_spacing.y = space::SM;
            ui.vertical_centered(|ui| bf_ui_kit::title(ui, &shared.tr("game-menu")));
            bf_ui_kit::gap(ui, space::SM);
            let resume = Button::primary(&shared.tr("game-menu-resume"))
                .width(f32::INFINITY)
                .show(ui);
            if resume.clicked() || keys::single(ui, "menu.resume", 'R', resume.rect) {
                state.menu_open = false;
            }
            let save = Button::secondary(&shared.tr("game-menu-save"))
                .width(f32::INFINITY)
                .show(ui);
            if keys::pressed(ui, id::SAVE, &save) {
                state.write(shared, now, true);
            }
            bf_ui_kit::columns(ui, 2, |cols| {
                let quick = Button::secondary(&shared.tr("game-menu-quick-save"))
                    .width(f32::INFINITY)
                    .show(&mut cols[0]);
                let quick = quick.on_hover_text(keys::tip(
                    cols[0].ctx(),
                    &shared.tr("game-menu-quick-save"),
                    id::QUICK_SAVE,
                ));
                if keys::pressed(&cols[0], id::QUICK_SAVE, &quick) {
                    quick_save(state, shared);
                }
                let has_quick = points::save_path(&state.save, points::QUICK).is_file();
                let load = Button::secondary(&shared.tr("game-menu-quick-load"))
                    .width(f32::INFINITY)
                    .enabled(has_quick)
                    .show(&mut cols[1]);
                let load = load.on_hover_text(keys::tip(
                    cols[1].ctx(),
                    &shared.tr("game-menu-quick-load"),
                    id::QUICK_LOAD,
                ));
                if has_quick && keys::pressed(&cols[1], id::QUICK_LOAD, &load) {
                    out = quick_load(state, shared, now);
                }
            });
            let named = Button::secondary(&shared.tr("game-menu-save-named"))
                .width(f32::INFINITY)
                .show(ui);
            if keys::pressed(ui, id::SAVE_NAMED, &named) {
                state.naming = Some(default_point_name(shared, &state.save, &snap));
                state.menu_open = false;
            }
            let hud = Button::secondary(&shared.tr("game-menu-hud"))
                .width(f32::INFINITY)
                .show(ui);
            if keys::pressed(ui, id::HUD_EDIT, &hud) {
                state.menu_open = false;
                toggle_hud_edit(state);
            }
            let settings = Button::secondary(&shared.tr("menu-settings"))
                .width(f32::INFINITY)
                .show(ui);
            if keys::pressed(ui, id::SETTINGS, &settings) {
                state.menu_open = false;
                out = Some(Transition::Settings);
            }
            let exit = Button::ghost(&shared.tr("game-menu-exit"))
                .width(f32::INFINITY)
                .show(ui);
            if exit.clicked() || keys::single(ui, "menu.exit", 'Q', exit.rect) {
                state.write(shared, now, false);
                out = Some(Transition::Menu);
            }
        });
    }
    // Saving with a name: a small dialog over the map.
    if let Some(mut name) = state.naming.take() {
        ui.painter()
            .rect_filled(rect, 0, Color32::from_black_alpha(140));
        let panel = Rect::from_center_size(rect.center(), egui::vec2(420.0, 200.0));
        keys::cover(ui, rect);
        super::frame(ui, panel, "save-named");
        ui.interact(
            rect,
            ui.make_persistent_id("naming-block"),
            Sense::click_and_drag(),
        );
        let mut keep = true;
        region(ui, panel.shrink(space::XL), |ui| {
            ui.spacing_mut().item_spacing.y = space::MD;
            bf_ui_kit::heading(ui, &shared.tr("points-name-title"));
            let field = bf_ui_kit::text_field(ui, &mut name, ui.available_width());
            if !field.has_focus() && !field.lost_focus() {
                field.request_focus();
            }
            let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.horizontal(|ui| {
                let ok = Button::primary(&shared.tr("points-name-save")).show(ui);
                let cancel = Button::ghost(&shared.tr("points-name-cancel")).show(ui);
                if (ok.clicked() || enter) && !name.trim().is_empty() {
                    let text = shared.i18n.tr_args(
                        "points-saved-named",
                        &[("name", bf_i18n::Arg::String(name.trim().to_owned()))],
                    );
                    state.write_point(Kind::Manual, Some(name.trim().to_owned()), Some(text), true);
                    keep = false;
                }
                if cancel.clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    keep = false;
                }
            });
        });
        if keep {
            state.naming = Some(name);
        }
    }
    out
}

/// The quick save: the campaign's state and the quick slot, replaced each time.
fn quick_save(state: &mut State, shared: &Shared) {
    let text = shared.tr("points-saved-quick");
    state.write_point(Kind::Quick, None, Some(text), true);
}

/// Goes back to the quick save, if there is one.
fn quick_load(state: &mut State, shared: &mut Shared, now: f64) -> Option<Transition> {
    if !points::save_path(&state.save, points::QUICK).is_file() {
        shared.toast(shared.tr("points-no-quick"), now);
        return None;
    }
    state.go_back_to(shared, now, points::QUICK)
}

/// What the name field starts with: the country and the date.
fn default_point_name(shared: &Shared, save: &Save, snap: &crate::sim::Snapshot) -> String {
    let date = crate::text::date_long(&shared.i18n, snap.date);
    match &save.meta.player {
        Some(tag) => format!("{} · {date}", country_name(&shared.i18n, tag)),
        None => date,
    }
}

/// Megabytes, one decimal, in the player's language.
pub fn mb(i18n: &bf_i18n::I18n, bytes: u64) -> String {
    i18n.decimal(bytes as f64 / 1e6, 1)
}

/// Opens the government window on its first tab, or closes it; the research screen gives way.
fn toggle_government(state: &mut State) {
    state.government = match state.government {
        Some(_) => None,
        None => Some(super::government::Window::default()),
    };
    state.research_open = false;
    state.journal = None;
    state.influence = None;
    state.economy = None;
    state.war = None;
}

/// Opens the war window, or closes it; the other windows give way.
fn toggle_war(state: &mut State) {
    let open = state.war.is_none();
    state.research_open = false;
    state.government = None;
    state.journal = None;
    state.influence = None;
    state.economy = None;
    state.war = open.then(super::war::Window::default);
}

/// Opens the economy window, or closes it; the other windows give way.
fn toggle_economy(state: &mut State) {
    let open = state.economy.is_none();
    state.research_open = false;
    state.government = None;
    state.journal = None;
    state.influence = None;
    state.war = None;
    state.economy = open.then(super::economy::Window::default);
}

/// Opens the influence window, or closes it; the other windows give way.
fn toggle_influence(state: &mut State) {
    state.influence = match state.influence {
        Some(_) => None,
        None => Some(super::influence::Window::default()),
    };
    state.economy = None;
    state.research_open = false;
    state.government = None;
    state.journal = None;
    state.war = None;
}

/// Opens the journal on its first entry, or closes it; the other windows give way.
fn toggle_journal(state: &mut State) {
    state.journal = match state.journal {
        Some(_) => None,
        None => Some(super::journal::Window::default()),
    };
    state.influence = None;
    state.economy = None;
    state.war = None;
    state.research_open = false;
    state.government = None;
}

/// Turns the HUD edit mode on or off; the windows close so every panel shows (a command calls
/// this).
pub fn toggle_hud_edit(state: &mut State) {
    super::hud_edit::toggle(&mut state.edit);
    state.lens_open = false;
    if state.edit.on {
        state.government = None;
        state.journal = None;
        state.influence = None;
        state.economy = None;
        state.war = None;
        state.research_open = false;
    }
}

/// Opens or closes the population panel; opened on a screen too narrow for it beside the
/// country panel, it takes the country panel's place.
fn toggle_population(state: &mut State, shared: &mut Shared, screen: Rect) {
    let open = shared.hud.layout.place(PanelId::Population).visible;
    let crowded = state.selected.is_some() && screen.width() < super::population::SIDE_BY_SIDE;
    if crowded && open {
        // Hidden behind the country panel: show it instead of closing it.
        state.selected = None;
        state.selected_state = None;
        return;
    }
    if crowded {
        state.selected = None;
        state.selected_state = None;
    }
    crate::hud::toggle_panel(&mut shared.hud, PanelId::Population);
}

/// The size each panel of the HUD asks for now; a panel not on screen asks for none.
fn hud_sizes(
    state: &State,
    shared: &Shared,
    ctx: &egui::Context,
    screen: Rect,
    player: Option<usize>,
) -> std::collections::BTreeMap<PanelId, egui::Vec2> {
    let mut out = std::collections::BTreeMap::new();
    let editing = state.edit.on;
    // The windows and the research screen cover the map.
    let window = state.government.is_some()
        || state.journal.is_some()
        || state.influence.is_some()
        || state.economy.is_some()
        || state.research_open;
    if player.is_some() {
        out.insert(
            PanelId::Country,
            top_bar::corner_size(shared.hud.layout.tiles().len()),
        );
    }
    out.insert(PanelId::Time, top_bar::time_size(screen));
    // The windows cover the map; the mode picker and the overview would show through.
    if let Some(view) = &shared.map_view
        && (editing || !window)
    {
        out.insert(
            PanelId::MapModes,
            if shared.hud.layout.map_modes_open {
                // Kept open, it may take most of the screen's height; the list scrolls past it.
                super::lens_panel::size(
                    ctx,
                    view.lens,
                    screen.height() - top_bar::LEFT_BOTTOM - crate::hud::GAP * 2.0,
                )
            } else {
                super::lens_panel::BUTTON
            },
        );
    }
    if editing || !(window || state.research_open) {
        out.insert(PanelId::Outliner, super::outliner::size(ctx, &state.fold));
    }
    // On a narrow screen the country panel, while a country is picked, takes the place.
    let crowded =
        screen.width() < super::population::SIDE_BY_SIDE && state.selected.is_some() && !editing;
    if player.is_some() && (editing || !window) && !crowded {
        out.insert(
            PanelId::Population,
            egui::vec2(
                super::population::WIDTH,
                (screen.height() - top_bar::LEFT_BOTTOM - crate::hud::GAP * 2.0).max(240.0),
            ),
        );
    }
    // A window covers the country panel too: it would show through the window's edge. The
    // selection stays (a demand goes to the country picked), only the panel waits.
    if (state.selected.is_some() && !window) || (editing && player.is_some()) {
        out.insert(
            PanelId::CountryPanel,
            egui::vec2(
                380.0,
                (screen.height() - top_bar::RIGHT_BOTTOM - space::LG).max(240.0),
            ),
        );
    }
    out
}

/// Keeps where the player left a window, and writes the layout once the move ends.
fn keep_window_place(shared: &mut Shared, key: &str, place: crate::hud::WindowPlace, done: bool) {
    if shared.hud.layout.window(key) != place {
        shared.hud.layout.windows.insert(key.to_owned(), place);
    }
    if done {
        shared.persist_hud();
    }
}

/// Opens the screen a block of the country figures reads from.
fn open_from_tile(state: &mut State, open: super::tiles::Open) {
    use super::government::{Tab, Window};
    use super::tiles::Open;
    let tab = match open {
        Open::Budget => Tab::Budget,
        Open::Money => Tab::Money,
        Open::Laws => Tab::Laws,
        Open::Research => {
            state.research_open = true;
            state.government = None;
            state.journal = None;
            state.influence = None;
            state.economy = None;
            state.war = None;
            return;
        }
        Open::Journal => {
            state.journal = Some(super::journal::Window::default());
            state.government = None;
            state.research_open = false;
            return;
        }
    };
    state.government = Some(Window {
        tab,
        ..Window::default()
    });
    state.journal = None;
    state.influence = None;
    state.economy = None;
    state.war = None;
    state.research_open = false;
}

/// Opens or closes the folded map mode picker (a command calls this).
pub fn toggle_lens_picker(state: &mut State) {
    state.lens_open = !state.lens_open;
}

/// Where the open picker goes, beside its button: above or below it and aligned on its left or
/// right edge, whichever way leaves room for the most of the panel (its size for a height is
/// `size_for`), then toward the middle of the screen. Never off the screen and never in the
/// free area when a way out of it exists.
pub fn lens_popup(
    button: Rect,
    size_for: impl Fn(f32) -> egui::Vec2,
    screen: Rect,
    free: Option<Rect>,
) -> Rect {
    let margin = crate::hud::GAP;
    let gap = space::SM;
    let up_room = button.top() - gap - (screen.top() + margin);
    let down_room = screen.bottom() - margin - (button.bottom() + gap);
    // Toward the middle first, for ties.
    let up_first = button.center().y > screen.center().y;
    let left_first = button.center().x <= screen.center().x;
    let width = size_for(screen.height()).x;
    let fit_x = |x: f32| x.clamp(screen.left(), (screen.right() - width).max(screen.left()));
    // Columns: aligned on the button's left or right edge; when the free area is in the way,
    // also beside the free area, as a last resort.
    let mut columns: Vec<(f32, bool, usize)> = Vec::new();
    for (j, left) in [left_first, !left_first].into_iter().enumerate() {
        let x = if left {
            button.left()
        } else {
            button.right() - width
        };
        columns.push((fit_x(x), true, j));
    }
    if let Some(f) = free {
        for x in [f.left() - margin - width, f.right() + margin] {
            if x >= screen.left() && x + width <= screen.right() {
                columns.push((x, false, 2));
            }
        }
    }
    let mut ways = Vec::new();
    for (k, up) in [up_first, !up_first].into_iter().enumerate() {
        for &(x, aligned, j) in &columns {
            // A free area in the column cuts the room on that side.
            let column = egui::Rangef::new(x, x + width);
            let mut room = if up { up_room } else { down_room };
            if let Some(f) = free
                && f.x_range().intersects(column)
            {
                if up && f.bottom() <= button.top() {
                    room = room.min(button.top() - gap - f.bottom() - margin);
                } else if !up && f.top() >= button.bottom() {
                    room = room.min(f.top() - margin - button.bottom() - gap);
                }
            }
            if room <= 0.0 {
                continue;
            }
            let size = size_for(room);
            let y = if up {
                button.top() - gap - size.y
            } else {
                button.bottom() + gap
            };
            let r = crate::hud::clamp_into(Rect::from_min_size(egui::pos2(x, y), size), screen);
            let clear = free.is_none_or(|f| !f.intersects(r));
            // Clear of the free area first, then beside the button, then the most height, then
            // toward the middle.
            let pref = 10 - (k * 3 + j) as i32;
            ways.push((r, (clear, aligned, r.height().round() as i32, pref)));
        }
    }
    // Last resort, when the free area takes the button's column and its side: just under the
    // free area, wherever the panel fits across.
    if let Some(f) = free {
        let y = f.bottom() + margin;
        let room = screen.bottom() - margin - y;
        if room > 0.0 {
            let size = size_for(room);
            let x = fit_x(if left_first {
                button.left()
            } else {
                button.right() - size.x
            });
            let r = crate::hud::clamp_into(Rect::from_min_size(egui::pos2(x, y), size), screen);
            let clear = !f.intersects(r);
            ways.push((r, (clear, false, r.height().round() as i32, -1)));
        }
    }
    let best = ways.into_iter().max_by_key(|(_, key)| *key);
    best.map(|(r, _)| r).unwrap_or_else(|| {
        crate::hud::clamp_into(
            Rect::from_min_size(button.min, size_for(screen.height())),
            screen,
        )
    })
}

/// The overview panel in `r`, and what its clicks open.
fn overview(
    state: &mut State,
    shared: &mut Shared,
    ui: &mut Ui,
    r: Rect,
    snap: &crate::sim::Snapshot,
    now: f64,
) {
    let sections = shared.hud.layout.overview();
    let res = super::outliner::show(
        ui,
        r,
        super::outliner::Sources {
            i18n: &shared.i18n,
            world: shared.world.as_ref(),
            catalogue: &state.rules.catalogue,
            tree: state.rules.tree.as_deref(),
            snap,
            pinned: &state.save.meta.pinned,
            sections: &sections,
        },
        &mut state.fold,
    );
    if res.open_budget {
        // The side panel's works open the works tab; the full queue is there.
        state.government = Some(super::government::Window {
            tab: super::government::Tab::Works,
            ..Default::default()
        });
    }
    if res.open_research {
        state.research_open = true;
    }
    if let Some(picked) = res.open_journal {
        state.journal = Some(super::journal::Window {
            picked,
            ..Default::default()
        });
    }
    if let Some(id) = res.unpin {
        state.toggle_pin(shared, &id, now);
    }
}

/// Reads the sea lens' layer and uploads its overlay; why not, when it cannot.
fn load_sea(
    shared: &mut Shared,
    ctx: &egui::Context,
) -> Result<std::sync::Arc<crate::map::sea::SeaLayer>, String> {
    let mut layer = crate::map::sea::load(&shared.paths.assets)?;
    if let Some(world) = &shared.world {
        layer.place_straits(|id| {
            let s = world.states.get(usize::from(id).checked_sub(1)?)?;
            Some(egui::pos2(s.def.center.0, s.def.center.1))
        });
    }
    shared.sea_texture = Some(layer.textures(ctx));
    Ok(std::sync::Arc::new(layer))
}

/// The same shared value, or both absent.
fn same<T>(a: &Option<std::sync::Arc<T>>, b: &Option<std::sync::Arc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
        (a, b) => a.is_none() && b.is_none(),
    }
}
