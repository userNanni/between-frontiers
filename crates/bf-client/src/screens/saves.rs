//! Campaigns: load or delete a saved campaign, or create one.

use bf_i18n::Arg;
use bf_ui_kit::{Button, ButtonSize, ListRow, tokens::space};
use egui::{Color32, Rect, Ui};

use super::{backdrop_map, region};
use crate::app::{Shared, Transition};
use crate::saves::{self, civil_date};
use crate::text::{country_name, date_long};

pub struct State {
    selected: Option<usize>,
    name: String,
    seed: String,
    error: Option<String>,
    confirm_delete: bool,
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
        }
    }
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

pub fn show(state: &mut State, shared: &mut Shared, ui: &mut Ui, rect: Rect) -> Option<Transition> {
    backdrop_map(shared, ui, rect, 0.35);
    let mut out = None;
    let width = 1060.0f32.min(rect.width() - 64.0);
    let outer = Rect::from_center_size(
        rect.center(),
        egui::vec2(width, (rect.height() - 96.0).min(720.0)),
    );
    let left_w = width * 0.58;
    let right_w = width - left_w - space::XL;

    // Header
    let header = Rect::from_min_size(outer.min, egui::vec2(width, 96.0));
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
            });
        });
    });

    let body_top = header.bottom() + space::LG;
    let list_rect = Rect::from_min_max(
        egui::pos2(outer.left(), body_top),
        egui::pos2(outer.left() + left_w, outer.bottom()),
    );
    let form_rect = Rect::from_min_max(
        egui::pos2(list_rect.right() + space::XL, body_top),
        egui::pos2(outer.right(), outer.bottom()),
    );

    // Saved campaigns
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
            let list_h = list_rect.height() - space::XL * 2.0 - 64.0;
            egui::ScrollArea::vertical()
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
                                Arg::String(date_long(
                                    &shared.i18n,
                                    civil_date(save.meta.last_played),
                                )),
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
                            mark: Some(mark.unwrap_or(bf_ui_kit::tokens::color::BORDER_STRONG)),
                            selected: state.selected == Some(i),
                        }
                        .show(ui);
                        if row.clicked() {
                            state.selected = Some(i);
                            state.confirm_delete = false;
                        }
                        if row.double_clicked() {
                            out = Some(open(save.clone()));
                        }
                    }
                });
            ui.add_space(space::MD);
            ui.horizontal(|ui| {
                let selected = state.selected.and_then(|i| shared.saves.get(i)).cloned();
                if Button::primary(&shared.tr("saves-load"))
                    .enabled(selected.is_some())
                    .show(ui)
                    .clicked()
                    && let Some(s) = selected.clone()
                {
                    out = Some(open(s));
                }
                if state.confirm_delete {
                    if let Some(s) = &selected {
                        bf_ui_kit::small(
                            ui,
                            &shared.i18n.tr_args(
                                "saves-delete-confirm",
                                &[("name", Arg::String(s.meta.name.clone()))],
                            ),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Button::danger(&shared.tr("saves-delete-yes"))
                            .show(ui)
                            .clicked()
                            && let Some(s) = &selected
                        {
                            if let Err(e) = saves::delete(s) {
                                state.error = Some(shared.i18n.tr_args(
                                    "saves-error-io",
                                    &[("error", Arg::String(e.to_string()))],
                                ));
                            }
                            shared.refresh_saves();
                            state.selected = if shared.saves.is_empty() {
                                None
                            } else {
                                Some(0)
                            };
                            state.confirm_delete = false;
                        }
                        if Button::ghost(&shared.tr("saves-delete-no"))
                            .show(ui)
                            .clicked()
                        {
                            state.confirm_delete = false;
                        }
                    });
                } else {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Button::ghost(&shared.tr("saves-delete"))
                            .enabled(selected.is_some())
                            .show(ui)
                            .clicked()
                        {
                            state.confirm_delete = true;
                        }
                    });
                }
            });
        });
    });

    // New campaign
    region(ui, form_rect, |ui| {
        bf_ui_kit::overlay_frame().show(ui, |ui| {
            let inner = right_w - space::XL * 2.0;
            ui.set_width(inner);
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
            if Button::primary(&shared.tr("saves-new-create"))
                .size(ButtonSize::Large)
                .width(f32::INFINITY)
                .show(ui)
                .clicked()
            {
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
                        Ok(save) => out = Some(Transition::Pick(save)),
                        Err(e) => {
                            state.error = Some(shared.i18n.tr_args(
                                "saves-error-io",
                                &[("error", Arg::String(e.to_string()))],
                            ))
                        }
                    }
                }
            }
        });
    });

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        out = Some(Transition::Menu);
    }
    out
}

fn open(save: saves::Save) -> Transition {
    if save.meta.player.is_some() {
        Transition::Game(save)
    } else {
        Transition::Pick(save)
    }
}
