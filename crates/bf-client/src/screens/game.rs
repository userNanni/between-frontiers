//! The campaign: map, top bar with date and speed, in-game menu.

use bf_ui_kit::{Button, tokens::space};
use egui::{Color32, Rect, Sense, Ui};

use super::{country_panel, pick::paint_tip, region, top_bar};
use crate::app::{Shared, Transition};
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
    research_era: Option<u8>,
    last_autosave: laplace_schelling::calendar::Date,
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
            let mut session =
                Session::new(data.new_world(), save.meta.seed, save.meta.date, 0, rules);
            if let Some(tag) = &save.meta.player {
                session.submit(GameCommand::ChoosePlayer { tag: tag.clone() });
            }
            session
        };
        let repaint = ctx.clone();
        let sim = SimHandle::start(session, move || repaint.request_repaint());
        let state = Self {
            last_autosave: save.meta.date,
            save,
            sim,
            selected: None,
            menu_open: false,
            rules: data.rules(),
            research_open: false,
            research_era: None,
        };
        if !path.is_file() {
            state.sim.save(path, state.save.meta.name.clone())?;
        }
        Ok(state)
    }

    /// What the simulation last published.
    pub fn snapshot(&self) -> crate::sim::Snapshot {
        self.sim.snapshot()
    }

    fn write(&mut self, shared: &mut Shared, now: f64, announce: bool) {
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
    if due {
        state.last_autosave = snap.date;
        state.write(shared, now, false);
    }

    let player = shared.world.as_ref().and_then(|w| {
        state
            .save
            .meta
            .player
            .as_ref()
            .and_then(|t| w.countries.iter().position(|c| c.tag() == t))
    });

    let map_rect = rect;
    let i18n = &shared.i18n;
    if let (Some(map), Some(view), Some(world)) = (&shared.map, &mut shared.map_view, &shared.world)
    {
        view.selected_owner = state.selected;
        let names = |c: usize| country_name(i18n, world.countries[c].tag());
        let response = view.show(ui, map_rect, map, world, &names, MapMode::Interactive);
        if let Some(id) = response.clicked_province {
            state.selected = world.provinces[id as usize - 1].owner;
        }
        if response.clicked_sea {
            state.selected = None;
        }
        if let (Some(id), Some(pos)) = (
            response.hovered_province,
            ui.input(|i| i.pointer.hover_pos()),
        ) && let Some((state_name, owner)) = world.place_name(id)
        {
            let owner = owner.map(names).unwrap_or_else(|| i18n.tr("map-unowned"));
            let extra = world.provinces[id as usize - 1].state.and_then(|s| {
                super::lens_panel::tooltip_line(i18n, world, s, view.lens, snap.date.year)
            });
            paint_tip(ui, pos, state_name, &owner, extra.as_deref());
        }
        super::lens_panel::show(ui, rect, i18n, world, view, snap.date.year);
    }

    // Top bar
    let bar_out = top_bar::show(
        ui,
        rect,
        &shared.i18n,
        shared.world.as_ref(),
        &shared.flags,
        player,
        &snap,
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
    }
    if bar_out.open_menu {
        state.menu_open = true;
        state.sim.send(Command::SetPaused(true));
    }

    // Country panel for the clicked country.
    if let Some(country) = state.selected {
        let side = Rect::from_min_max(
            egui::pos2(
                rect.right() - 380.0 - space::LG,
                rect.top() + top_bar::RIGHT_BOTTOM,
            ),
            egui::pos2(rect.right() - space::LG, rect.bottom() - space::LG),
        );
        bf_ui_kit::paint_overlay(ui, side);
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

    // Research screen.
    if state.research_open
        && let (Some(tree), Some(view)) = (&state.rules.tree, &snap.research)
    {
        let era = state
            .research_era
            .get_or_insert_with(|| super::research::current_era(snap.date.year));
        let res = super::research::show(
            ui,
            rect,
            &shared.i18n,
            tree,
            &state.rules.catalogue,
            view,
            era,
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

    // Keyboard
    let research_key = ui.input(|i| i.key_pressed(egui::Key::R));
    if research_key && !state.menu_open {
        state.research_open = !state.research_open;
    }
    let (space_key, esc, speed_key) = ui.input(|i| {
        let speed = [
            egui::Key::Num1,
            egui::Key::Num2,
            egui::Key::Num3,
            egui::Key::Num4,
            egui::Key::Num5,
        ]
        .iter()
        .position(|k| i.key_pressed(*k));
        (
            i.key_pressed(egui::Key::Space),
            i.key_pressed(egui::Key::Escape),
            speed,
        )
    });
    if !state.menu_open {
        if space_key {
            state.sim.send(Command::SetPaused(!snap.paused));
        }
        if let Some(s) = speed_key {
            state.sim.send(Command::SetSpeed(s as u8 + 1));
        }
        if esc {
            if state.research_open {
                state.research_open = false;
            } else if state.selected.is_some() {
                state.selected = None;
            } else {
                state.menu_open = true;
                state.sim.send(Command::SetPaused(true));
            }
        }
    } else if esc {
        state.menu_open = false;
    }

    // In-game menu
    if state.menu_open {
        ui.painter()
            .rect_filled(rect, 0, Color32::from_black_alpha(140));
        let panel = Rect::from_center_size(rect.center(), egui::vec2(360.0, 300.0));
        bf_ui_kit::paint_overlay(ui, panel);
        ui.interact(
            rect,
            ui.make_persistent_id("menu-block"),
            Sense::click_and_drag(),
        );
        region(ui, panel.shrink(space::XL), |ui| {
            ui.spacing_mut().item_spacing.y = space::SM;
            ui.vertical_centered(|ui| bf_ui_kit::title(ui, &shared.tr("game-menu")));
            bf_ui_kit::gap(ui, space::SM);
            if Button::primary(&shared.tr("game-menu-resume"))
                .width(f32::INFINITY)
                .show(ui)
                .clicked()
            {
                state.menu_open = false;
            }
            if Button::secondary(&shared.tr("game-menu-save"))
                .width(f32::INFINITY)
                .show(ui)
                .clicked()
            {
                state.write(shared, now, true);
            }
            if Button::secondary(&shared.tr("menu-settings"))
                .width(f32::INFINITY)
                .show(ui)
                .clicked()
            {
                state.menu_open = false;
                out = Some(Transition::Settings);
            }
            if Button::ghost(&shared.tr("game-menu-exit"))
                .width(f32::INFINITY)
                .show(ui)
                .clicked()
            {
                state.write(shared, now, false);
                out = Some(Transition::Menu);
            }
        });
    }
    out
}
