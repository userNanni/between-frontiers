//! Settings dialog. Changes apply at once and are saved immediately.

use bf_i18n::{Arg, Locale};
use bf_ui_kit::{Button, ButtonSize, tokens::space};
use egui::{Rect, Ui};

use super::{centered, region};
use crate::app::Shared;
use crate::settings::Autosave;

/// Returns true when the player closes the dialog.
pub fn show(shared: &mut Shared, ui: &mut Ui, rect: Rect) -> bool {
    let mut done = false;
    let panel = centered(rect, egui::vec2(600.0, 620.0));
    region(ui, panel, |ui| {
        bf_ui_kit::overlay_frame().show(ui, |ui| {
            ui.set_width(panel.width() - space::XL * 2.0);
            ui.spacing_mut().item_spacing.y = space::LG;
            bf_ui_kit::title(ui, &shared.tr("settings-title"));
            bf_ui_kit::divider(ui);

            let mut locale = shared.settings.locale();
            let options: Vec<(Locale, String)> = Locale::ALL
                .iter()
                .map(|l| (*l, l.endonym().to_owned()))
                .collect();
            let hint = shared.tr("settings-language-hint");
            let changed =
                bf_ui_kit::field(ui, &shared.tr("settings-language"), Some(&hint), |ui| {
                    bf_ui_kit::segmented(ui, &mut locale, &options)
                });
            if changed {
                shared.set_locale(locale);
            }

            let mut scale = shared.settings.ui_scale;
            let scales: Vec<(u32, String)> = [90u32, 100, 110, 125, 150]
                .iter()
                .map(|s| {
                    (
                        *s,
                        shared.i18n.tr_args(
                            "settings-ui-scale-value",
                            &[("percent", Arg::Num(*s as f64))],
                        ),
                    )
                })
                .collect();
            let changed = bf_ui_kit::field(ui, &shared.tr("settings-ui-scale"), None, |ui| {
                bf_ui_kit::segmented(ui, &mut scale, &scales)
            });
            if changed {
                shared.settings.ui_scale = scale;
                ui.ctx().set_zoom_factor(scale as f32 / 100.0);
                shared.persist_settings();
            }

            let mut autosave = shared.settings.autosave;
            let options = vec![
                (Autosave::Off, shared.tr("settings-autosave-off")),
                (Autosave::Monthly, shared.tr("settings-autosave-monthly")),
                (Autosave::Yearly, shared.tr("settings-autosave-yearly")),
            ];
            let changed = bf_ui_kit::field(ui, &shared.tr("settings-autosave"), None, |ui| {
                bf_ui_kit::segmented(ui, &mut autosave, &options)
            });
            if changed {
                shared.settings.autosave = autosave;
                shared.persist_settings();
            }

            ui.horizontal(|ui| {
                let mut labels = shared.settings.map_labels;
                if bf_ui_kit::toggle(ui, &mut labels).changed() {
                    shared.settings.map_labels = labels;
                    if let Some(v) = &mut shared.map_view {
                        v.show_labels = labels;
                    }
                    shared.persist_settings();
                }
                bf_ui_kit::body(ui, &shared.tr("settings-map-labels"));
            });

            ui.horizontal(|ui| {
                let mut relief = shared.settings.map_relief;
                if bf_ui_kit::toggle(ui, &mut relief).changed() {
                    shared.settings.map_relief = relief;
                    if let Some(v) = &mut shared.map_view {
                        v.relief_strength = if relief { 1.0 } else { 0.0 };
                    }
                    shared.persist_settings();
                }
                bf_ui_kit::body(ui, &shared.tr("settings-map-relief"));
            });

            let folder = shared.paths.saves().display().to_string();
            bf_ui_kit::field(
                ui,
                &shared.tr("settings-saves-folder"),
                Some(&folder),
                |_| {},
            );

            bf_ui_kit::divider(ui);
            let w = ui.available_width();
            ui.allocate_ui_with_layout(
                egui::vec2(w, 40.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if Button::primary(&shared.tr("settings-done"))
                        .size(ButtonSize::Medium)
                        .show(ui)
                        .clicked()
                    {
                        done = true;
                    }
                },
            );
        });
    });
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        done = true;
    }
    done
}
