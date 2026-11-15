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

    pub fn persist_settings(&self) {
        if let Err(e) = self.settings.save(&self.paths.settings_file()) {
            log(&format!("settings: {e}"));
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
        cc.egui_ctx
            .set_zoom_factor(settings.ui_scale as f32 / 100.0);

        let mut shared = Shared {
            paths,
            settings,
            i18n,
            world: None,
            map: None,
            map_view: None,
            flags: Default::default(),
            load_error: None,
            saves: Vec::new(),
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
    let relief = crate::map::Relief::load(&map_dir.join("relief.png"))
        .unwrap_or_else(crate::map::Relief::flat);
    let map = MapData::build(&world, w, h, ids, relief);
    Ok((world, map))
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let screen = ui.max_rect();
        ui.painter()
            .rect_filled(screen, 0, bf_ui_kit::tokens::color::BACKDROP);
        let transition = screens::show(&mut self.screen, &mut self.shared, ui, screen);
        self.paint_toast(ui, screen);
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
