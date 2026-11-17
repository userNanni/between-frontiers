//! The campaign: map, top bar with date and speed, in-game menu.

use bf_ui_kit::{Button, tokens::space};
use egui::{Color32, Rect, Sense, Ui};

use super::government::Tab;
use super::{country_panel, pick::paint_tip, region, top_bar};
use crate::app::{Shared, Transition};
use crate::hud::PanelId;
use crate::keys::{self, id};
use crate::map::MapMode;
use crate::saves::{self, Save};
use crate::settings::Autosave;
use crate::sim::{Command, SimHandle};
use crate::text::country_name;
use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};

pub struct State {
    save: Save,
    sim: SimHandle,
    selected: Option<usize>,
    menu_open: bool,
    rules: bf_sim::game::Rules,
    research_open: bool,
    /// The government window while it is open.
    government: Option<super::government::Window>,
    /// The journal window while it is open.
    journal: Option<super::journal::Window>,
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
    last_autosave: laplace_schelling::calendar::Date,
    /// An autosave the simulation is writing: its answer, and the date and time it records.
    autosaving: Option<Autosaving>,
}

struct Autosaving {
    answer: std::sync::mpsc::Receiver<Result<(), String>>,
    date: laplace_schelling::calendar::Date,
    played: u64,
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
            Session::load(&bytes, 0, rules.clone()).map_err(|e| e.to_string())?
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
        let sim = SimHandle::start(session, rules.clone(), move || repaint.request_repaint());
        let state = Self {
            last_autosave: save.meta.date,
            autosaving: None,
            save,
            sim,
            selected: None,
            menu_open: false,
            rules: data.rules(),
            research_open: false,
            government: None,
            journal: None,
            fold: super::outliner::Fold::default(),
            edit: super::hud_edit::Edit::default(),
            lens_open: false,
            sea: super::sea_lens::SeaView::default(),
            overview_open: false,
            population: Default::default(),
            research_camera: None,
            live_month: None,
            live_figures: None,
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

    /// The sea lens' route and preview (tests and commands set them).
    pub fn sea_view_mut(&mut self) -> &mut super::sea_lens::SeaView {
        &mut self.sea
    }

    /// What the simulation last published.
    pub fn snapshot(&self) -> crate::sim::Snapshot {
        self.sim.snapshot()
    }

    fn write(&mut self, shared: &mut Shared, now: f64, announce: bool) {
        // A pending autosave was queued first: this save's world lands after it and wins.
        self.autosaving = None;
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

    // Autosave on month or year change.
    let due = match shared.settings.autosave {
        Autosave::Off => false,
        Autosave::Monthly => {
            (snap.date.year, snap.date.month)
                != (state.last_autosave.year, state.last_autosave.month)
        }
        Autosave::Yearly => snap.date.year != state.last_autosave.year,
    };
    if due && state.autosaving.is_none() {
        // The simulation writes the world while the interface keeps drawing; the campaign's
        // metadata follows once the world is on disk.
        state.last_autosave = snap.date;
        let answer = state.sim.save_later(
            state.save.dir.join(saves::WORLD_FILE),
            state.save.meta.name.clone(),
        );
        state.autosaving = Some(Autosaving {
            answer,
            date: snap.date,
            played: saves::now(),
        });
    }
    if let Some(pending) = &state.autosaving {
        let result = match pending.answer.try_recv() {
            Ok(r) => Some(r),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(e) => Some(Err(e.to_string())),
        };
        if let Some(result) = result {
            state.save.meta.date = pending.date;
            state.save.meta.last_played = pending.played;
            state.autosaving = None;
            if let Err(e) =
                result.and_then(|()| saves::write(&state.save).map_err(|e| e.to_string()))
            {
                let msg = shared
                    .i18n
                    .tr_args("saves-error-io", &[("error", bf_i18n::Arg::String(e))]);
                shared.toast(msg, now);
            }
        }
    }

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
        let month = (snap.date.year, snap.date.month);
        if let Some(view) = &mut shared.map_view
            && matches!(view.lens, crate::map::lens::Lens::Local(_))
            && state.live_month != Some(month)
        {
            view.set_palette(crate::map::lens::palette(world, view.lens, snap.date.year));
        }
        state.live_month = Some(month);
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
            state.selected = world.provinces[id as usize - 1].owner;
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
        }
        // Nothing of the map shows through the research screen.
        if !state.research_open
            && let (Some(id), Some(pos)) = (
                response.hovered_province,
                ui.input(|i| i.pointer.hover_pos()),
            )
            && let Some((state_name, owner)) = world.place_name(id)
        {
            let owner = owner.map(names).unwrap_or_else(|| i18n.tr("map-unowned"));
            let extra = world.provinces[id as usize - 1].state.and_then(|s| {
                super::lens_panel::tooltip_line(i18n, world, s, view.lens, snap.date.year)
            });
            paint_tip(ui, pos, state_name, &owner, extra.as_deref());
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
    }
    if bar_out.open_government {
        toggle_government(state);
    }
    if bar_out.open_journal {
        toggle_journal(state);
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

    // Country panel for the clicked country.
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
                .show(ui, |ui| country_panel::show(shared, ui, country));
        });
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
            },
            window,
            &mut gov_place,
        );
        keep_window_place(shared, "government", gov_place, res.place_done);
        for c in res.commands {
            state.sim.send(Command::Issue(c));
        }
        if res.close {
            state.government = None;
        }
    }

    // Journal window.
    if state.journal.is_some() && !state.research_open && state.government.is_none() {
        let pinned = state.save.meta.pinned.clone();
        let mut journal_place = shared.hud.layout.window("journal");
        let res = super::journal::show(
            ui,
            rect,
            &shared.i18n,
            shared.world.as_ref(),
            snap.journal.as_deref(),
            &pinned,
            state.journal.as_mut().expect("journal open"),
            &mut journal_place,
        );
        keep_window_place(shared, "journal", journal_place, res.place_done);
        for id in res.toggle_pin {
            state.toggle_pin(shared, &id, now);
        }
        if let Some(screen) = res.open {
            use super::government::{Tab, Window};
            state.journal = None;
            let tab = match screen {
                crate::journal::Screen::Budget => Some(Tab::Budget),
                crate::journal::Screen::Money => Some(Tab::Money),
                crate::journal::Screen::Laws => Some(Tab::Laws),
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
        }
        if fired(id::GOVERNMENT) {
            toggle_government(state);
        }
        if fired(id::JOURNAL) {
            toggle_journal(state);
        }
        for (cmd, tab) in [
            (id::TAB_BUDGET, Tab::Budget),
            (id::TAB_MONEY, Tab::Money),
            (id::TAB_LAWS, Tab::Laws),
        ] {
            if fired(cmd) {
                state.research_open = false;
                state.journal = None;
                state.government.get_or_insert_with(Default::default).tab = tab;
            }
        }
        if fired(id::COUNTRY) {
            state.selected = player;
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
            } else if state.journal.is_some() {
                state.journal = None;
            } else if state.selected.is_some() {
                state.selected = None;
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
        let panel = Rect::from_center_size(rect.center(), egui::vec2(360.0, 352.0));
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
    out
}

/// Opens the government window on its first tab, or closes it; the research screen gives way.
fn toggle_government(state: &mut State) {
    state.government = match state.government {
        Some(_) => None,
        None => Some(super::government::Window::default()),
    };
    state.research_open = false;
    state.journal = None;
}

/// Opens the journal on its first entry, or closes it; the other windows give way.
fn toggle_journal(state: &mut State) {
    state.journal = match state.journal {
        Some(_) => None,
        None => Some(super::journal::Window::default()),
    };
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
        return;
    }
    if crowded {
        state.selected = None;
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
    let window = state.government.is_some() || state.journal.is_some() || state.research_open;
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
    if state.selected.is_some() || (editing && player.is_some()) {
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
        state.government = Some(super::government::Window::default());
    }
    if res.open_research {
        state.research_open = true;
    }
    if let Some(picked) = res.open_journal {
        state.journal = Some(super::journal::Window { picked });
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
