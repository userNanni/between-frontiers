//! Country choice for a new campaign: the world map, a sidebar with the selected country and the
//! button that starts the campaign.

use bf_ui_kit::{
    Button, ButtonSize,
    tokens::{color, space},
};
use egui::{Rect, Sense, Ui};

use super::{country_panel, region};
use crate::app::{Shared, Transition};
use crate::map::MapMode;
use crate::saves::{self, Save};
use crate::text::country_name;

const SIDEBAR_W: f32 = 400.0;
const FOOTER_H: f32 = 96.0;

pub struct State {
    save: Save,
    selected: Option<usize>,
    /// Choosing another country in a saved campaign: the map shows the save's borders, and the
    /// choice is written to the campaign instead of starting one.
    switching: bool,
}

impl State {
    pub fn new(save: Save) -> Self {
        Self {
            save,
            selected: None,
            switching: false,
        }
    }

    /// Another country for a saved campaign: reads the borders of the save into the world, so
    /// that every country that exists there can be chosen.
    pub fn switching(save: Save, shared: &mut Shared) -> Self {
        let path = save.dir.join(saves::WORLD_FILE);
        if let (Ok(bytes), Some(world)) = (std::fs::read(&path), shared.world.as_mut())
            && let Ok(session) = bf_sim::game::Session::load(&bytes, 1, None)
        {
            world.apply_owners(&owners_of(session.world(), world));
            // The save's own figures, not 1925's: a country born in the campaign has people.
            world.live = crate::live::read(session.world(), session.clock().date)
                .map(|f| crate::world::FigureTable::live(&world.countries, &f));
        }
        Self {
            save,
            selected: None,
            switching: true,
        }
    }

    /// Selects a country as if the player had clicked it (used by tests and screenshots).
    pub fn select(&mut self, country: Option<usize>) {
        self.selected = country;
    }
}

pub fn show(state: &mut State, shared: &mut Shared, ui: &mut Ui, rect: Rect) -> Option<Transition> {
    let mut out = None;
    let i18n = &shared.i18n;
    let Some((map, view, world)) = (match (&shared.map, &mut shared.map_view, &shared.world) {
        (Some(m), Some(v), Some(w)) => Some((m, v, w)),
        _ => None,
    }) else {
        let msg = shared.load_error.clone().unwrap_or_default();
        region(ui, rect.shrink(48.0), |ui| {
            bf_ui_kit::title(ui, &shared.tr("pick-title"));
            bf_ui_kit::error_text(ui, &msg);
            if Button::secondary(&shared.tr("pick-back"))
                .show(ui)
                .clicked()
            {
                out = Some(Transition::Saves);
            }
        });
        return out;
    };

    view.selected_owner = state.selected;
    let names = |c: usize| country_name(i18n, world.countries[c].tag());
    let response = view.show(ui, rect, map, world, &names, MapMode::Interactive);
    if let Some(id) = response.clicked_province {
        state.selected = world.provinces[id as usize - 1].owner;
    }
    if response.clicked_sea {
        state.selected = None;
    }

    // Tooltip with the state and country under the pointer.
    if let (Some(id), Some(pos)) = (
        response.hovered_province,
        ui.input(|i| i.pointer.hover_pos()),
    ) && let Some((state_name, owner)) = world.place_name(&shared.i18n, id)
    {
        let owner = owner
            .map(names)
            .unwrap_or_else(|| shared.i18n.tr("map-unowned"));
        let extra = world.provinces[id as usize - 1].state.and_then(|s| {
            super::lens_panel::tooltip_line(&shared.i18n, world, s, view.lens, world.start.year)
        });
        paint_tip(ui, pos, &state_name, &owner, extra.as_deref());
    }
    let size = super::lens_panel::size(ui.ctx(), view.lens, rect.height() * 0.6);
    let origin = egui::pos2(rect.left() + 12.0, rect.bottom() - 12.0 - size.y);
    super::lens_panel::show(
        ui,
        Rect::from_min_size(origin, size),
        &shared.i18n,
        world,
        view,
        world.start.year,
        None,
    );

    // Title banner
    let banner = Rect::from_center_size(
        egui::pos2(
            rect.center().x
                - if state.selected.is_some() {
                    SIDEBAR_W / 2.0
                } else {
                    0.0
                },
            rect.top() + 58.0,
        ),
        egui::vec2(700.0, 76.0),
    );
    bf_ui_kit::paint_overlay(ui, banner);
    ui.interact(
        banner,
        ui.make_persistent_id("banner"),
        Sense::click_and_drag(),
    );
    region(ui, banner.shrink2(egui::vec2(space::XL, space::MD)), |ui| {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            let (title, hint) = if state.switching {
                ("pick-switch-title", "pick-switch-hint")
            } else {
                ("pick-title", "pick-hint")
            };
            bf_ui_kit::heading(ui, &shared.i18n.tr(title));
            bf_ui_kit::small(ui, &shared.i18n.tr(hint));
        });
    });

    // Back
    let back = Rect::from_min_size(
        rect.min + egui::vec2(space::LG, space::LG + 16.0),
        egui::vec2(240.0, 40.0),
    );
    region(ui, back, |ui| {
        if Button::secondary(&format!("←  {}", shared.i18n.tr("pick-back")))
            .show(ui)
            .clicked()
        {
            out = Some(Transition::Saves);
        }
    });

    // Sidebar
    if let Some(country) = state.selected {
        let side = Rect::from_min_max(
            egui::pos2(rect.right() - SIDEBAR_W - space::LG, rect.top() + space::LG),
            egui::pos2(rect.right() - space::LG, rect.bottom() - space::LG),
        );
        bf_ui_kit::paint_overlay(ui, side);
        ui.interact(
            side,
            ui.make_persistent_id("sidebar"),
            Sense::click_and_drag(),
        );
        let body = Rect::from_min_max(
            side.min + egui::vec2(space::XL, space::XL),
            egui::pos2(side.right() - space::XL, side.bottom() - FOOTER_H),
        );
        region(ui, body, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    country_panel::show(shared, ui, country);
                });
        });
        bf_ui_kit::paint_rule(ui, side, side.bottom() - FOOTER_H + space::SM);
        let footer = Rect::from_min_max(
            egui::pos2(
                side.left() + space::XL,
                side.bottom() - FOOTER_H + space::LG + space::SM,
            ),
            egui::pos2(side.right() - space::XL, side.bottom() - space::LG),
        );
        region(ui, footer, |ui| {
            let label = if state.switching {
                "pick-switch"
            } else {
                "pick-start"
            };
            if Button::primary(&shared.i18n.tr(label))
                .size(ButtonSize::Large)
                .width(f32::INFINITY)
                .show(ui)
                .clicked()
            {
                let tag = shared
                    .world
                    .as_ref()
                    .map(|w| w.countries[country].tag().to_owned());
                state.save.meta.player = tag;
                state.save.meta.last_played = saves::now();
                match saves::write(&state.save) {
                    // A new country for a saved campaign goes back to the campaigns: the game
                    // takes it when the campaign opens.
                    Ok(()) if state.switching => out = Some(Transition::Saves),
                    Ok(()) => out = Some(Transition::Game(state.save.clone())),
                    Err(e) => {
                        let now = ui.input(|i| i.time);
                        let msg = shared.i18n.tr_args(
                            "saves-error-io",
                            &[("error", bf_i18n::Arg::String(e.to_string()))],
                        );
                        shared.toast(msg, now);
                    }
                }
            }
        });
    }

    if crate::keys::fired(ui.ctx(), crate::keys::id::BACK) {
        if state.selected.is_some() {
            state.selected = None;
        } else {
            out = Some(Transition::Saves);
        }
    }
    out
}

/// Small tooltip next to the pointer: the state in bold, its country below, and a line from
/// the current map mode when there is one.
pub fn paint_tip(ui: &Ui, pointer: egui::Pos2, title: &str, detail: &str, extra: Option<&str>) {
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::unique("map-tip"),
    ));
    let t = painter.layout_no_wrap(
        title.to_owned(),
        bf_ui_kit::theme::sans_semibold(14.0),
        color::TEXT,
    );
    let d = painter.layout_no_wrap(
        detail.to_owned(),
        bf_ui_kit::theme::sans(12.5),
        color::TEXT_MUTED,
    );
    let e = extra.map(|x| {
        painter.layout_no_wrap(
            x.to_owned(),
            bf_ui_kit::theme::sans_medium(12.5),
            color::ACCENT,
        )
    });
    let e_h = e.as_ref().map_or(0.0, |g| g.size().y + 3.0);
    let e_w = e.as_ref().map_or(0.0, |g| g.size().x);
    let size = egui::vec2(
        t.size().x.max(d.size().x).max(e_w),
        t.size().y + d.size().y + 2.0 + e_h,
    ) + egui::vec2(20.0, 14.0);
    let r = Rect::from_min_size(pointer + egui::vec2(18.0, 20.0), size);
    painter.rect(
        r,
        bf_ui_kit::tokens::radius::MD,
        color::SURFACE_OVERLAY,
        egui::Stroke::new(1.0, color::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );
    let th = t.size().y;
    let dh = d.size().y;
    painter.galley(r.min + egui::vec2(10.0, 7.0), t, color::TEXT);
    painter.galley(r.min + egui::vec2(10.0, 9.0 + th), d, color::TEXT_MUTED);
    if let Some(e) = e {
        painter.galley(r.min + egui::vec2(10.0, 12.0 + th + dh), e, color::ACCENT);
    }
}

/// The owners of a saved world, read without the rules: the subjects of the scenario stand
/// until a border operation ran.
fn owners_of(
    w: &bf_sim::game::GameWorld,
    client: &crate::world::World,
) -> bf_sim::borders::op::OwnersView {
    let n = client.states.len();
    let mut owners = vec![(u16::MAX, false); n];
    let mut people = vec![0i64; n];
    for r in &w.econ.regions {
        if let Some(i) = usize::from(r.state).checked_sub(1).filter(|&i| i < n) {
            owners[i] = (r.market, r.colony);
            people[i] = r.population();
        }
    }
    let tags: Vec<String> = w.countries.iter().map(|(_, c)| c.tag.clone()).collect();
    let overlords = (0..tags.len())
        .map(|c| match w.borders.subjects.get(c) {
            Some(s) if !w.borders.subjects.is_empty() => s.as_ref().map(|s| s.overlord),
            _ => client
                .countries
                .get(c)
                .and_then(|k| k.overlord)
                .map(|o| o as u16),
        })
        .collect();
    bf_sim::borders::op::OwnersView {
        changes: w.borders.changes.len(),
        tags,
        owners,
        overlords,
        people,
    }
}
