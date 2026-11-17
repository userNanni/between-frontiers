//! Application shell: shared resources and the screen state machine.

use bf_i18n::{I18n, Locale};
use egui::{Rect, Ui};

use crate::map::{MapData, MapView, gpu::MapGpu, read_ids};
use crate::paths::Paths;
use crate::saves::{self, Save};
use crate::screens::{self, Screen};
use crate::settings::Settings;
use crate::world::World;

/// Everything the screens share.
pub struct Shared {
    pub paths: Paths,
    pub settings: Settings,
    pub i18n: I18n,
    pub world: Option<World>,
    pub map: Option<MapData>,
    pub map_view: Option<MapView>,
    pub flags: crate::flags::Flags,
    /// Why the map or the scenario could not load; screens that need them show it.
    pub load_error: Option<String>,
    pub saves: Vec<Save>,
    /// The player's HUD layout, from the config folder.
    pub hud: crate::hud::Hud,
    /// Commands, the player's keys and key tips.
    pub keys: crate::keys::Keys,
    /// The keyboard editor, while it is open over the settings.
    pub key_editor: Option<crate::screens::keys_editor::State>,
    /// A scale change waiting for the player to keep it.
    pub scale_trial: Option<crate::ui_scale::Trial>,
    /// The sea lens' layer, read the first time the lens opens (an error says why it could not),
    /// and its overlay on the GPU.
    pub sea: Option<Result<std::sync::Arc<crate::map::sea::SeaLayer>, String>>,
    pub sea_texture: Option<Vec<(egui::TextureHandle, egui::Rect)>>,
    /// The map's owners changed (a border operation): the GPU side is built again next frame.
    pub map_gpu_dirty: bool,
    toast: Option<(String, f64)>,
}

impl Shared {
    pub fn tr(&self, id: &str) -> String {
        self.i18n.tr(id)
    }

    pub fn refresh_saves(&mut self) {
        self.saves = saves::list(&self.paths.saves());
    }

    pub fn set_locale(&mut self, locale: Locale) {
        match load_i18n(&self.paths, locale) {
            Ok(i18n) => {
                self.i18n = i18n;
                self.settings.locale = locale.code().to_owned();
                self.persist_settings();
            }
            Err(e) => log(&format!("language {locale}: {e}")),
        }
    }

    /// The interface scale the player chose.
    pub fn scale_choice(&self) -> crate::ui_scale::Choice {
        crate::ui_scale::Choice::of(self.settings.ui_scale, self.settings.ui_stream)
    }

    /// Applies a scale at once, on trial: it goes back after a few seconds unless kept (a
    /// command calls this).
    pub fn try_scale(&mut self, choice: crate::ui_scale::Choice, now: f64) {
        let previous = match self.scale_trial {
            Some(t) => t.previous,
            None => self.scale_choice(),
        };
        if choice == previous {
            self.scale_trial = None;
        } else {
            self.scale_trial = Some(crate::ui_scale::Trial {
                previous,
                deadline: now + crate::ui_scale::TRIAL_SECONDS,
            });
        }
        (self.settings.ui_scale, self.settings.ui_stream) = choice.store();
        if self.scale_trial.is_none() {
            self.persist_settings();
        }
    }

    /// Keeps the scale on trial and writes it.
    pub fn keep_scale(&mut self) {
        self.scale_trial = None;
        self.persist_settings();
    }

    /// Puts back the scale that was in use before the trial.
    pub fn revert_scale(&mut self) {
        if let Some(t) = self.scale_trial.take() {
            (self.settings.ui_scale, self.settings.ui_stream) = t.previous.store();
        }
    }

    /// One step up (`+1`) or down from the scale in use now (a command calls this).
    pub fn step_scale(&mut self, ppp: f32, dir: i32, now: f64) {
        self.try_scale(crate::ui_scale::step(ppp, dir), now);
    }

    /// Switches the stream interface on (full HD) or back to the automatic scale (a command
    /// calls this).
    pub fn toggle_stream_scale(&mut self, now: f64) {
        use crate::ui_scale::{Choice, StreamTarget};
        let next = match self.scale_choice() {
            Choice::Stream(_) => Choice::Auto,
            _ => Choice::Stream(StreamTarget::Hd),
        };
        self.try_scale(next, now);
    }

    pub fn persist_settings(&self) {
        // A scale on trial is not written until kept.
        let mut settings = self.settings.clone();
        if let Some(t) = self.scale_trial {
            (settings.ui_scale, settings.ui_stream) = t.previous.store();
        }
        if let Err(e) = settings.save(&self.paths.settings_file()) {
            log(&format!("settings: {e}"));
        }
    }

    /// Writes the HUD layout to the config folder.
    pub fn persist_hud(&self) {
        if let Err(e) = self.hud.layout.save(&crate::hud::file(&self.paths.config)) {
            log(&format!("hud layout: {e}"));
        }
    }

    pub fn toast(&mut self, text: String, now: f64) {
        self.toast = Some((text, now + 2.5));
    }
}

fn load_i18n(paths: &Paths, locale: Locale) -> Result<I18n, bf_i18n::LoadError> {
    I18n::load(&paths.assets.join("i18n"), locale)
}

fn log(msg: &str) {
    eprintln!("between-frontiers: {msg}");
}

/// Screen changes requested by a screen.
pub enum Transition {
    Menu,
    Settings,
    Saves,
    /// Choose the country of a new campaign.
    Pick(Save),
    Game(Save),
    Quit,
}

pub struct App {
    pub shared: Shared,
    pub screen: Screen,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, paths: Paths) -> Self {
        bf_ui_kit::install(&cc.egui_ctx);
        let settings = Settings::load(&paths.settings_file());
        let i18n = load_i18n(&paths, settings.locale())
            .or_else(|_| load_i18n(&paths, Locale::PtBr))
            .unwrap_or_else(|e| {
                log(&format!("i18n: {e}"));
                I18n::from_sources(Locale::PtBr, []).expect("empty bundle")
            });

        let paths_config = paths.config.clone();
        let (keys, notes) = crate::keys::Keys::load(Some(paths.keys_file()));
        for n in notes {
            log(&n);
        }
        let mut shared = Shared {
            keys,
            key_editor: None,
            paths,
            settings,
            i18n,
            world: None,
            map: None,
            map_view: None,
            flags: Default::default(),
            load_error: None,
            saves: Vec::new(),
            hud: crate::hud::Hud::new(crate::hud::Layout::load(&crate::hud::file(&paths_config))),
            scale_trial: None,
            sea: None,
            sea_texture: None,
            map_gpu_dirty: false,
            toast: None,
        };
        shared.refresh_saves();
        match load_world(&shared.paths) {
            Ok((world, map)) => {
                if let Some(rs) = &cc.wgpu_render_state {
                    let gpu = MapGpu::new(&rs.device, &rs.queue, rs.target_format, &map);
                    rs.renderer.write().callback_resources.insert(gpu);
                }
                let mut view = MapView::new(&map);
                view.show_labels = shared.settings.map_labels;
                view.relief_strength = if shared.settings.map_relief { 1.0 } else { 0.0 };
                shared.map_view = Some(view);
                shared.flags = crate::flags::Flags::load(
                    &cc.egui_ctx,
                    &shared.paths.assets.join("flags"),
                    &world,
                );
                shared.map = Some(map);
                shared.world = Some(world);
            }
            Err(e) => {
                log(&e);
                shared.load_error = Some(e);
            }
        }
        Self {
            shared,
            screen: Screen::Menu,
        }
    }

    pub fn apply(&mut self, transition: Transition, ctx: &egui::Context) {
        let shared = &mut self.shared;
        if let Some(view) = &mut shared.map_view {
            view.camera = None;
            view.selected_owner = None;
        }
        // Outside a campaign the scenario's figures stand; a campaign reads its own again.
        if let Some(world) = &mut shared.world {
            world.clear_live();
        }
        self.screen = match transition {
            Transition::Menu => {
                shared.refresh_saves();
                Screen::Menu
            }
            Transition::Settings => {
                let back = std::mem::replace(&mut self.screen, Screen::Menu);
                Screen::Settings(Box::new(back))
            }
            Transition::Saves => {
                shared.refresh_saves();
                Screen::Saves(screens::saves::State::new(shared))
            }
            Transition::Pick(save) => Screen::Pick(screens::pick::State::new(save)),
            Transition::Game(save) => match screens::game::State::new(shared, save, ctx) {
                Ok(state) => Screen::Game(Box::new(state)),
                Err(e) => {
                    let now = ctx.input(|i| i.time);
                    let msg = shared
                        .i18n
                        .tr_args("saves-error-load", &[("error", bf_i18n::Arg::String(e))]);
                    shared.toast(msg, now);
                    shared.refresh_saves();
                    Screen::Saves(screens::saves::State::new(shared))
                }
            },
            Transition::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        };
    }

    fn paint_toast(&mut self, ui: &Ui, screen: Rect) {
        let now = ui.input(|i| i.time);
        let Some((text, until)) = &self.shared.toast else {
            return;
        };
        if now > *until {
            self.shared.toast = None;
            return;
        }
        ui.ctx().request_repaint();
        let painter = ui.painter();
        let galley = painter.layout_no_wrap(
            text.clone(),
            bf_ui_kit::theme::sans_medium(bf_ui_kit::tokens::size::BODY),
            bf_ui_kit::tokens::color::TEXT,
        );
        let r = Rect::from_center_size(
            egui::pos2(screen.center().x, screen.bottom() - 64.0),
            galley.size() + egui::vec2(36.0, 20.0),
        );
        painter.rect(
            r,
            bf_ui_kit::tokens::radius::MD,
            bf_ui_kit::tokens::color::SURFACE_RAISED,
            egui::Stroke::new(1.0, bf_ui_kit::tokens::color::ACCENT.gamma_multiply(0.6)),
            egui::StrokeKind::Inside,
        );
        painter.galley(
            r.center() - galley.size() / 2.0,
            galley,
            egui::Color32::WHITE,
        );
    }
}

/// A data file that may not exist yet; a file that exists but does not parse is logged.
fn read_optional<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Option<T> {
    let text = std::fs::read_to_string(path).ok()?;
    match ron::from_str(&text) {
        Ok(v) => Some(v),
        Err(e) => {
            log(&format!("{}: {e}", path.display()));
            None
        }
    }
}

fn load_world(paths: &Paths) -> Result<(World, MapData), String> {
    let map_dir = paths.assets.join("map");
    let mut world = World::load(
        &map_dir.join("provinces.ron"),
        &paths.data.join("scenario/1925/countries.ron"),
    )
    .map_err(|e| e.to_string())?;
    let (w, h, ids) = read_ids(&map_dir.join("provinces.png")).map_err(|e| e.to_string())?;
    if (w, h) != (world.map.meta.width, world.map.meta.height) {
        return Err(format!(
            "provinces.png is {w}×{h}, provinces.ron says {}×{}",
            world.map.meta.width, world.map.meta.height
        ));
    }
    world.locate_capitals(&ids);
    let scenario = paths.data.join("scenario/1925");
    let deposits = read_optional(&scenario.join("deposits.ron"));
    let land = read_optional(&scenario.join("land.ron"));
    world.attach_resources(deposits, land);
    world.candidates = read_optional::<bf_sim::borders::CandidatesFile>(
        &paths.data.join("borders/candidates.ron"),
    )
    .map(|f| f.countries)
    .unwrap_or_default();
    let relief = crate::map::Relief::load(&map_dir.join("relief.png"))
        .unwrap_or_else(crate::map::Relief::flat);
    let map = MapData::build(&world, w, h, ids, relief);
    Ok((world, map))
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        apply_scale(ui.ctx(), self.shared.scale_choice());
        if std::mem::take(&mut self.shared.map_gpu_dirty)
            && let (Some(rs), Some(map)) = (frame.wgpu_render_state(), &self.shared.map)
        {
            let gpu = MapGpu::new(&rs.device, &rs.queue, rs.target_format, map);
            rs.renderer.write().callback_resources.insert(gpu);
        }
        let screen = ui.max_rect();
        ui.painter()
            .rect_filled(screen, 0, bf_ui_kit::tokens::color::BACKDROP);
        self.shared.keys.begin_frame(ui.ctx(), &self.shared.i18n);
        let transition = screens::show(&mut self.screen, &mut self.shared, ui, screen);
        self.shared
            .keys
            .end_frame(ui.ctx(), &self.shared.i18n, screen);
        self.paint_toast(ui, screen);
        self.scale_controls(ui, screen);
        if let Some(t) = transition {
            let ctx = ui.ctx().clone();
            self.apply(t, &ctx);
            ctx.request_repaint();
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        bf_ui_kit::tokens::color::BACKDROP.to_normalized_gamma_f32()
    }
}

/// Sets the zoom so that a point takes the pixels the scale asks for, on the screen as it is
/// now: a new resolution or a moved window take a new scale the next frame.
pub fn apply_scale(ctx: &egui::Context, choice: crate::ui_scale::Choice) {
    let ppp_now = ctx.pixels_per_point();
    let size = ctx.content_rect().size() * ppp_now;
    let native = ctx.native_pixels_per_point().unwrap_or(1.0);
    let ppp = crate::ui_scale::pixels_per_point(choice, (size.x, size.y), native);
    let zoom = ppp / native;
    if (ctx.zoom_factor() - zoom).abs() > 1e-3 {
        ctx.set_zoom_factor(zoom);
    }
}

impl App {
    /// The scale commands, on any screen, and the bar that asks to keep a new scale.
    fn scale_controls(&mut self, ui: &mut Ui, screen: Rect) {
        use crate::keys::{self, id};
        use bf_ui_kit::tokens::space;
        let ctx = ui.ctx().clone();
        let now = ctx.input(|i| i.time);
        let ppp = ctx.pixels_per_point();
        if keys::fired(&ctx, id::UI_SCALE_UP) {
            self.shared.step_scale(ppp, 1, now);
        }
        if keys::fired(&ctx, id::UI_SCALE_DOWN) {
            self.shared.step_scale(ppp, -1, now);
        }
        if keys::fired(&ctx, id::UI_STREAM) {
            self.shared.toggle_stream_scale(now);
        }
        let Some(trial) = self.shared.scale_trial else {
            return;
        };
        let left = (trial.deadline - now).max(0.0);
        if left <= 0.0 {
            self.shared.revert_scale();
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        let card = Rect::from_center_size(
            egui::pos2(screen.center().x, screen.top() + 140.0),
            egui::vec2(460.0_f32.min(screen.width() - 32.0), 96.0),
        );
        bf_ui_kit::paint_overlay(ui, card);
        ui.interact(
            card,
            ui.make_persistent_id("scale-trial"),
            egui::Sense::click_and_drag(),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(card.shrink(space::LG)));
        let ui = &mut child;
        ui.spacing_mut().item_spacing.y = space::SM;
        bf_ui_kit::body(
            ui,
            &self.shared.i18n.tr_args(
                "settings-ui-scale-keep-ask",
                &[("seconds", bf_i18n::Arg::Num(left.ceil()))],
            ),
        );
        ui.horizontal(|ui| {
            if bf_ui_kit::Button::primary(&self.shared.tr("settings-ui-scale-keep"))
                .size(bf_ui_kit::ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                self.shared.keep_scale();
            }
            if bf_ui_kit::Button::ghost(&self.shared.tr("settings-ui-scale-revert"))
                .size(bf_ui_kit::ButtonSize::Small)
                .show(ui)
                .clicked()
            {
                self.shared.revert_scale();
            }
        });
    }
}
