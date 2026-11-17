//! Campaigns: load, rename or delete a saved campaign and its restore points, or create one.
//!
//! The list of campaigns on the left, each with a "…" menu (rename, delete, open the folder;
//! other systems add their entries to [`MenuItem`]); on the right, the restore points of the
//! selected campaign, or the form for a new one.

use bf_i18n::Arg;
use bf_ui_kit::tokens::{color, space};
use bf_ui_kit::{Button, ButtonSize, ListRow};
use egui::{Color32, Rect, Ui};

use super::{backdrop_map, region};
use crate::app::{Shared, Transition};
use crate::keys;
use crate::points::{self, Kind, Point};
use crate::saves::{self, civil_date};
use crate::text::{country_name, date_long};

/// Entries of a campaign's "…" menu, in order. Ids are stable (`saves.menu.<id>`, Fluent
/// `saves-menu-<id>`); "play_other" is reserved for playing the campaign as another country.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    Rename,
    Delete,
    OpenFolder,
}

impl MenuItem {
    pub const ALL: [MenuItem; 3] = [MenuItem::Rename, MenuItem::Delete, MenuItem::OpenFolder];

    pub fn id(self) -> &'static str {
        match self {
            MenuItem::Rename => "saves.menu.rename",
            MenuItem::Delete => "saves.menu.delete",
            MenuItem::OpenFolder => "saves.menu.open_folder",
        }
    }

    fn label(self) -> &'static str {
        match self {
            MenuItem::Rename => "saves-menu-rename",
            MenuItem::Delete => "saves-menu-delete",
            MenuItem::OpenFolder => "saves-menu-open-folder",
        }
    }

    fn letter(self) -> char {
        match self {
            MenuItem::Rename => 'R',
            MenuItem::Delete => 'X',
            MenuItem::OpenFolder => 'F',
        }
    }
}

pub struct State {
    selected: Option<usize>,
    name: String,
    seed: String,
    error: Option<String>,
    confirm_delete: bool,
    /// The right side shows the new campaign form.
    creating: bool,
    /// The campaign whose "…" menu is open.
    menu: Option<usize>,
    /// The campaign's new name, while it is being renamed.
    renaming: Option<String>,
    /// The points of the selected campaign, and which campaign they belong to.
    points: Vec<Point>,
    points_of: Option<usize>,
    point: Option<usize>,
    /// The point's new name, while it is being renamed.
    renaming_point: Option<String>,
    confirm_point_delete: bool,
    /// Bytes each campaign folder takes, in list order.
    sizes: Vec<u64>,
}

impl State {
    pub fn new(shared: &Shared) -> Self {
        let mut n = shared.saves.len() + 1;
        let mut name;
        loop {
            name = shared.i18n.tr_args(
                "saves-new-name-default",
                &[("n", Arg::String(n.to_string()))],
            );
            if !saves::name_taken(&shared.paths.saves(), &name) {
                break;
            }
            n += 1;
        }
        Self {
            selected: if shared.saves.is_empty() {
                None
            } else {
                Some(0)
            },
            name,
            seed: random_seed().to_string(),
            error: None,
            confirm_delete: false,
            creating: shared.saves.is_empty(),
            menu: None,
            renaming: None,
            points: Vec::new(),
            points_of: None,
            point: None,
            renaming_point: None,
            confirm_point_delete: false,
            sizes: sizes(shared),
        }
    }

    /// Selects campaign `i` and drops what belonged to the previous one.
    fn select(&mut self, i: Option<usize>) {
        if self.selected != i {
            self.selected = i;
            self.confirm_delete = false;
            self.renaming = None;
            self.points_of = None;
        }
        self.creating = false;
    }

    fn reload_points(&mut self, shared: &Shared) {
        self.points = self
            .selected
            .and_then(|i| shared.saves.get(i))
            .map(points::list)
            .unwrap_or_default();
        self.points_of = self.selected;
        self.point = (!self.points.is_empty()).then_some(0);
        self.renaming_point = None;
        self.confirm_point_delete = false;
    }
}

fn sizes(shared: &Shared) -> Vec<u64> {
    shared
        .saves
        .iter()
        .map(|s| points::folder_bytes(&s.dir))
        .collect()
}

fn random_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1925);
    // SplitMix64 finaliser, so consecutive clicks give unrelated seeds.
    let mut z = nanos.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) % 1_000_000_000
}

fn io_error(shared: &Shared, e: impl ToString) -> String {
    shared
        .i18n
        .tr_args("saves-error-io", &[("error", Arg::String(e.to_string()))])
}

/// When a point was written, on the player's clock.
fn written(shared: &Shared, secs: u64) -> String {
    let (day, h, m) = saves::local_time(secs);
    format!("{} {h:02}:{m:02}", date_long(&shared.i18n, day))
}

pub fn show(state: &mut State, shared: &mut Shared, ui: &mut Ui, rect: Rect) -> Option<Transition> {
    backdrop_map(shared, ui, rect, 0.35);
    let mut out = None;
    let width = 1160.0f32.min(rect.width() - 64.0);
    let outer = Rect::from_center_size(
        rect.center(),
        egui::vec2(width, (rect.height() - 64.0).min(760.0)),
    );
    let left_w = width * 0.5;
    if state.points_of != state.selected {
        state.reload_points(shared);
    }

    // Header
    let header = Rect::from_min_size(outer.min, egui::vec2(width, 80.0));
    region(ui, header, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = space::XS;
                bf_ui_kit::title(ui, &shared.tr("saves-title"));
                bf_ui_kit::muted(ui, &shared.tr("saves-subtitle"));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if Button::secondary(&format!("←  {}", shared.tr("menu-back")))
                    .size(ButtonSize::Medium)
                    .show(ui)
                    .clicked()
                {
                    out = Some(Transition::Menu);
                }
                let new = Button::primary(&shared.tr("saves-new-title"))
                    .size(ButtonSize::Medium)
                    .show(ui);
                if new.clicked() || keys::single(ui, "saves.new", 'N', new.rect) {
                    state.creating = true;
                    state.menu = None;
                }
            });
        });
    });

    let body_top = header.bottom() + space::LG;
    let list_rect = Rect::from_min_max(
        egui::pos2(outer.left(), body_top),
        egui::pos2(outer.left() + left_w, outer.bottom()),
    );
    let side_rect = Rect::from_min_max(
        egui::pos2(list_rect.right() + space::XL, body_top),
        egui::pos2(outer.right(), outer.bottom()),
    );

    super::mark_frame(ui, list_rect, "campaigns");
    region(ui, list_rect, |ui| {
        bf_ui_kit::overlay_frame().show(ui, |ui| {
            ui.set_width(left_w - space::XL * 2.0);
            ui.set_min_height(list_rect.height() - space::XL * 2.0);
            if shared.saves.is_empty() {
                ui.add_space(space::HUGE);
                ui.vertical_centered(|ui| {
                    bf_ui_kit::heading(ui, &shared.tr("saves-empty"));
                    bf_ui_kit::muted(ui, &shared.tr("saves-empty-hint"));
                });
                return;
            }
            campaigns(state, shared, ui, list_rect, &mut out);
        });
    });

    super::mark_frame(ui, side_rect, "campaign-side");
    region(ui, side_rect, |ui| {
        bf_ui_kit::overlay_frame().show(ui, |ui| {
            let inner = side_rect.width() - space::XL * 2.0;
            ui.set_width(inner);
            ui.set_min_height(side_rect.height() - space::XL * 2.0);
            let selected = state.selected.and_then(|i| shared.saves.get(i)).cloned();
            match selected {
                Some(save) if !state.creating => {
                    points_panel(state, shared, ui, side_rect, save, &mut out)
                }
                _ => new_form(state, shared, ui, inner, &mut out),
            }
        });
    });

    // The open "…" menu, over the rest.
    if let Some(i) = state.menu {
        campaign_menu(state, shared, ui, rect, i);
    }

    if keys::fired(ui.ctx(), keys::id::BACK) {
        if state.menu.is_some() || state.renaming.is_some() || state.renaming_point.is_some() {
            state.menu = None;
            state.renaming = None;
            state.renaming_point = None;
        } else {
            out = Some(Transition::Menu);
        }
    }
    out
}

/// Where each campaign's "…" button sits, for the menu to open under it.
fn menu_button_id(i: usize) -> egui::Id {
    egui::Id::unique(("saves-menu-button", i))
}

fn campaigns(
    state: &mut State,
    shared: &mut Shared,
    ui: &mut Ui,
    list_rect: Rect,
    out: &mut Option<Transition>,
) {
    let list_h = list_rect.height() - space::XL * 2.0 - 64.0;
    let warn = u64::from(shared.settings.warn_mb) * 1_000_000;
    egui::ScrollArea::vertical()
        .id_salt("campaigns")
        .max_height(list_h)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space::SM;
            for (i, save) in shared.saves.iter().enumerate() {
                let (subtitle, mark) = match &save.meta.player {
                    Some(tag) => {
                        let color = shared
                            .world
                            .as_ref()
                            .and_then(|w| w.countries.iter().find(|c| c.tag() == tag))
                            .map(|c| {
                                let (r, g, b) = c.def.color;
                                Color32::from_rgb(r, g, b)
                            });
                        (
                            format!(
                                "{} · {}",
                                country_name(&shared.i18n, tag),
                                date_long(&shared.i18n, save.meta.date)
                            ),
                            color,
                        )
                    }
                    None => (shared.tr("saves-country-pending"), None),
                };
                let trailing = shared.i18n.tr_args(
                    "saves-last-played",
                    &[(
                        "when",
                        Arg::String(date_long(&shared.i18n, civil_date(save.meta.last_played))),
                    )],
                );
                let image = save
                    .meta
                    .player
                    .as_ref()
                    .and_then(|tag| shared.flags.get(tag))
                    .map(|t| t.id());
                let row = ListRow {
                    image,
                    title: &save.meta.name,
                    subtitle: &subtitle,
                    trailing: &trailing,
                    mark: Some(mark.unwrap_or(color::BORDER_STRONG)),
                    selected: state.selected == Some(i),
                }
                .show(ui);
                let typed = keys::item(
                    ui,
                    'Z',
                    format!("saves.campaign.{}", i + 1),
                    &(i + 1).to_string(),
                    row.rect,
                );
                // The "…" menu, at the bottom right of the row; the folder size beside it.
                let dots = Rect::from_min_size(
                    egui::pos2(
                        row.rect.right() - space::LG - 28.0,
                        row.rect.bottom() - 30.0,
                    ),
                    egui::vec2(28.0, 24.0),
                );
                let size = state.sizes.get(i).copied().unwrap_or(0);
                ui.painter().text(
                    egui::pos2(dots.left() - space::SM, dots.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!("{} MB", super::game::mb(&shared.i18n, size)),
                    bf_ui_kit::theme::sans(bf_ui_kit::tokens::size::CAPTION),
                    if size > warn {
                        color::DANGER
                    } else {
                        color::TEXT_FAINT
                    },
                );
                let menu = ui
                    .put(dots, egui::Button::new("…").frame(state.menu == Some(i)))
                    .on_hover_text(shared.tr("saves-menu"));
                ui.ctx()
                    .data_mut(|d| d.insert_temp(menu_button_id(i), menu.rect));
                if menu.clicked() {
                    state.select(Some(i));
                    state.menu = if state.menu == Some(i) { None } else { Some(i) };
                } else if row.clicked() || typed {
                    state.select(Some(i));
                    state.menu = None;
                }
                if row.double_clicked() {
                    *out = Some(open(save.clone()));
                }
            }
        });
    ui.add_space(space::MD);
    ui.horizontal(|ui| {
        let selected = state.selected.and_then(|i| shared.saves.get(i)).cloned();
        if let (Some(name), Some(save)) = (&mut state.renaming, &selected) {
            let field = bf_ui_kit::text_field(ui, name, ui.available_width() - 200.0);
            if !field.has_focus() && !field.lost_focus() {
                field.request_focus();
            }
            let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let ok = Button::primary(&shared.tr("saves-rename-ok")).show(ui);
            if ok.clicked() || enter {
                let name = name.trim().to_owned();
                let taken = !name.eq_ignore_ascii_case(&save.meta.name)
                    && saves::name_taken(&shared.paths.saves(), &name);
                if name.is_empty() || taken {
                    state.error = Some(shared.tr(if taken {
                        "saves-new-name-taken"
                    } else {
                        "saves-new-name-empty"
                    }));
                } else {
                    let mut save = save.clone();
                    if let Err(e) = saves::rename(&mut save, &name) {
                        state.error = Some(io_error(shared, e));
                    }
                    state.renaming = None;
                    shared.refresh_saves();
                    state.selected = shared.saves.iter().position(|s| s.dir == save.dir);
                }
            }
            if Button::ghost(&shared.tr("saves-cancel")).show(ui).clicked() {
                state.renaming = None;
            }
            return;
        }
        let load = Button::primary(&shared.tr("saves-load"))
            .enabled(selected.is_some())
            .show(ui);
        if (load.clicked() || keys::single(ui, "saves.load", 'L', load.rect))
            && let Some(s) = selected.clone()
        {
            *out = Some(open(s));
        }
        if state.confirm_delete
            && let Some(s) = &selected
        {
            bf_ui_kit::small(
                ui,
                &shared.i18n.tr_args(
                    "saves-delete-confirm",
                    &[("name", Arg::String(s.meta.name.clone()))],
                ),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let yes = Button::danger(&shared.tr("saves-delete-yes")).show(ui);
                if yes.clicked() || keys::single(ui, "saves.delete_yes", 'Y', yes.rect) {
                    if let Err(e) = saves::delete(s) {
                        state.error = Some(io_error(shared, e));
                    }
                    shared.refresh_saves();
                    state.sizes = sizes(shared);
                    state.selected = (!shared.saves.is_empty()).then_some(0);
                    state.points_of = None;
                    state.confirm_delete = false;
                    state.creating = shared.saves.is_empty();
                }
                if Button::ghost(&shared.tr("saves-delete-no"))
                    .show(ui)
                    .clicked()
                {
                    state.confirm_delete = false;
                }
            });
        }
    });
    if let Some(e) = &state.error {
        bf_ui_kit::error_text(ui, e);
    }
}

/// The "…" menu of campaign `i`, under its button.
fn campaign_menu(state: &mut State, shared: &mut Shared, ui: &mut Ui, screen: Rect, i: usize) {
    let Some(save) = shared.saves.get(i).cloned() else {
        state.menu = None;
        return;
    };
    let anchor = ui
        .ctx()
        .data(|d| d.get_temp::<Rect>(menu_button_id(i)))
        .unwrap_or(Rect::from_center_size(screen.center(), egui::Vec2::ZERO));
    let w = 240.0;
    let h = MenuItem::ALL.len() as f32 * 40.0 + space::MD * 2.0;
    let mut r = Rect::from_min_size(
        egui::pos2(anchor.right() - w, anchor.bottom() + 4.0),
        egui::vec2(w, h),
    );
    if r.bottom() > screen.bottom() - space::SM {
        r = r.translate(egui::vec2(0.0, anchor.top() - 4.0 - r.bottom()));
    }
    // A click outside closes it.
    let outside = ui.interact(
        screen,
        ui.make_persistent_id("saves-menu-outside"),
        egui::Sense::click(),
    );
    keys::cover(ui, screen);
    super::frame(ui, r, "campaign-menu");
    let mut close = outside.clicked();
    region(ui, r.shrink(space::MD), |ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        for item in MenuItem::ALL {
            let b = Button::ghost(&shared.tr(item.label()))
                .width(f32::INFINITY)
                .show(ui);
            if b.clicked() || keys::single(ui, item.id(), item.letter(), b.rect) {
                close = true;
                match item {
                    MenuItem::Rename => state.renaming = Some(save.meta.name.clone()),
                    MenuItem::Delete => state.confirm_delete = true,
                    MenuItem::OpenFolder => {
                        if let Err(e) = super::keys_editor::open_folder(&save.dir) {
                            state.error = Some(io_error(shared, e));
                        }
                    }
                }
            }
        }
    });
    if close {
        state.menu = None;
    }
}

fn kind_label(shared: &Shared, p: &Point) -> String {
    match p.kind {
        Kind::Auto => shared.tr("points-kind-auto"),
        Kind::Before(r) => shared.tr(r.key()),
        Kind::Quick => shared.tr("points-kind-quick"),
        Kind::Manual => shared.tr("points-kind-manual"),
    }
}

/// The restore points of the selected campaign.
fn points_panel(
    state: &mut State,
    shared: &mut Shared,
    ui: &mut Ui,
    side: Rect,
    mut save: saves::Save,
    out: &mut Option<Transition>,
) {
    ui.spacing_mut().item_spacing.y = space::SM;
    bf_ui_kit::heading(ui, &shared.tr("points-title"));
    let bytes = state
        .selected
        .and_then(|i| state.sizes.get(i))
        .copied()
        .unwrap_or(0);
    let manual = state.points.iter().filter(|p| !p.kind.automatic()).count();
    let summary = shared.i18n.tr_args(
        "points-summary",
        &[
            ("name", Arg::String(save.meta.name.clone())),
            ("count", Arg::Num(state.points.len() as f64)),
            ("manual", Arg::Num(manual as f64)),
            ("size", Arg::String(super::game::mb(&shared.i18n, bytes))),
        ],
    );
    bf_ui_kit::muted(ui, &summary);
    if bytes > u64::from(shared.settings.warn_mb) * 1_000_000 {
        bf_ui_kit::error_text(
            ui,
            &shared.i18n.tr_args(
                "points-folder-large",
                &[
                    ("size", Arg::String(super::game::mb(&shared.i18n, bytes))),
                    ("limit", Arg::String(shared.settings.warn_mb.to_string())),
                ],
            ),
        );
    }
    if state.points.is_empty() {
        ui.add_space(space::LG);
        bf_ui_kit::muted(ui, &shared.tr("points-empty"));
        return;
    }
    let list_h = (side.bottom() - ui.cursor().top() - space::XL - 64.0).max(80.0);
    let mut open_now = false;
    egui::ScrollArea::vertical()
        .id_salt("points")
        .max_height(list_h)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space::SM;
            for (k, p) in state.points.iter().enumerate() {
                let kind = kind_label(shared, p);
                let title = p.name.clone().unwrap_or_else(|| kind.clone());
                let subtitle = if p.name.is_some() {
                    format!("{} · {kind}", date_long(&shared.i18n, p.date))
                } else {
                    date_long(&shared.i18n, p.date)
                };
                let trailing = written(shared, p.written);
                let mark = match p.kind {
                    Kind::Manual => color::ACCENT,
                    Kind::Quick => color::POSITIVE,
                    Kind::Before(_) => color::DANGER,
                    Kind::Auto => color::BORDER_STRONG,
                };
                let row = ListRow {
                    image: None,
                    title: &title,
                    subtitle: &subtitle,
                    trailing: &trailing,
                    mark: Some(mark),
                    selected: state.point == Some(k),
                }
                .show(ui);
                let typed = keys::item(
                    ui,
                    'W',
                    format!("saves.point.{}", p.id),
                    &(k + 1).to_string(),
                    row.rect,
                );
                if row.clicked() || typed {
                    state.point = Some(k);
                    state.confirm_point_delete = false;
                    state.renaming_point = None;
                }
                if row.double_clicked() {
                    state.point = Some(k);
                    open_now = true;
                }
            }
        });
    if open_now {
        *out = go_back(state, shared, &mut save);
        return;
    }
    ui.add_space(space::MD);
    let Some(p) = state.point.and_then(|k| state.points.get(k)).cloned() else {
        return;
    };
    ui.horizontal(|ui| {
        if let Some(name) = &mut state.renaming_point {
            let field = bf_ui_kit::text_field(ui, name, ui.available_width() - 200.0);
            if !field.has_focus() && !field.lost_focus() {
                field.request_focus();
            }
            let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if Button::primary(&shared.tr("saves-rename-ok"))
                .show(ui)
                .clicked()
                || enter
            {
                if let Err(e) = points::rename(&save, &p.id, name) {
                    state.error = Some(io_error(shared, e));
                }
                state.points_of = None;
            }
            if Button::ghost(&shared.tr("saves-cancel")).show(ui).clicked() {
                state.renaming_point = None;
            }
            return;
        }
        if state.confirm_point_delete {
            bf_ui_kit::small(ui, &shared.tr("points-delete-confirm"));
            let yes = Button::danger(&shared.tr("points-delete-yes")).show(ui);
            if yes.clicked() || keys::single(ui, "saves.point_delete_yes", 'Y', yes.rect) {
                if let Err(e) = points::delete(&save, &p.id) {
                    state.error = Some(io_error(shared, e));
                }
                state.points_of = None;
                state.sizes = sizes(shared);
            }
            if Button::ghost(&shared.tr("saves-delete-no"))
                .show(ui)
                .clicked()
            {
                state.confirm_point_delete = false;
            }
            return;
        }
        let back = Button::primary(&shared.tr("points-load")).show(ui);
        if back.clicked() || keys::single(ui, "saves.point_load", 'V', back.rect) {
            *out = go_back(state, shared, &mut save);
        }
        let rename = Button::secondary(&shared.tr("points-rename")).show(ui);
        if rename.clicked() || keys::single(ui, "saves.point_rename", 'E', rename.rect) {
            state.renaming_point = Some(p.name.clone().unwrap_or_default());
        }
        let delete = Button::ghost(&shared.tr("points-delete")).show(ui);
        if delete.clicked() || keys::single(ui, "saves.point_delete", 'D', delete.rect) {
            state.confirm_point_delete = true;
        }
    });
}

/// Makes the selected point the campaign's state and opens it.
fn go_back(state: &mut State, shared: &mut Shared, save: &mut saves::Save) -> Option<Transition> {
    let p = state.point.and_then(|k| state.points.get(k))?;
    match points::restore(save, &p.id) {
        Ok(()) => Some(open(save.clone())),
        Err(e) => {
            state.error = Some(io_error(shared, e));
            None
        }
    }
}

fn new_form(
    state: &mut State,
    shared: &mut Shared,
    ui: &mut Ui,
    inner: f32,
    out: &mut Option<Transition>,
) {
    ui.spacing_mut().item_spacing.y = space::LG;
    bf_ui_kit::heading(ui, &shared.tr("saves-new-title"));
    bf_ui_kit::field(ui, &shared.tr("saves-new-name"), None, |ui| {
        bf_ui_kit::text_field(ui, &mut state.name, inner)
    });
    bf_ui_kit::field(ui, &shared.tr("saves-new-start"), None, |ui| {
        bf_ui_kit::body(ui, &shared.tr("saves-new-start-value"))
    });
    let hint = shared.tr("saves-new-seed-hint");
    bf_ui_kit::field(ui, &shared.tr("saves-new-seed"), Some(&hint), |ui| {
        let r = bf_ui_kit::text_field(ui, &mut state.seed, inner);
        state.seed.retain(|c| c.is_ascii_digit());
        state.seed.truncate(19);
        r
    });
    if let Some(e) = &state.error {
        bf_ui_kit::error_text(ui, e);
    }
    bf_ui_kit::gap(ui, space::SM);
    let create = Button::primary(&shared.tr("saves-new-create"))
        .size(ButtonSize::Large)
        .width(f32::INFINITY)
        .show(ui);
    if create.clicked() || keys::single(ui, "saves.create", 'C', create.rect) {
        let name = state.name.trim().to_owned();
        let dir = shared.paths.saves();
        if name.is_empty() {
            state.error = Some(shared.tr("saves-new-name-empty"));
        } else if saves::name_taken(&dir, &name) {
            state.error = Some(shared.tr("saves-new-name-taken"));
        } else {
            let seed = state.seed.parse().unwrap_or_else(|_| random_seed());
            let start = shared
                .world
                .as_ref()
                .map(|w| w.start)
                .unwrap_or(laplace_schelling::calendar::Date::new(1925, 1, 1));
            match saves::create(&dir, &name, seed, start) {
                Ok(save) => *out = Some(Transition::Pick(save)),
                Err(e) => state.error = Some(io_error(shared, e)),
            }
        }
    }
    if !shared.saves.is_empty()
        && Button::ghost(&shared.tr("saves-new-cancel"))
            .width(f32::INFINITY)
            .show(ui)
            .clicked()
    {
        state.creating = false;
        state.error = None;
    }
}

fn open(save: saves::Save) -> Transition {
    if save.meta.player.is_some() {
        Transition::Game(save)
    } else {
        Transition::Pick(save)
    }
}
