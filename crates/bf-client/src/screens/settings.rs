//! Settings dialog. Changes apply at once and are saved immediately.

use bf_i18n::{Arg, Locale};
use bf_ui_kit::{Button, ButtonSize, tokens::space};
use egui::{Rect, Ui};

use super::{centered, region};
use crate::app::Shared;
use crate::settings::Autosave;

/// Returns true when the player closes the dialog.
pub fn show(shared: &mut Shared, ui: &mut Ui, rect: Rect) -> bool {
    if shared.key_editor.is_some() {
        super::keys_editor::show(shared, ui, rect);
        return false;
    }
    let mut done = false;
    let panel = centered(rect, egui::vec2(600.0, 700.0));
    super::mark_frame(ui, panel, "settings");
    region(ui, panel, |ui| {
        bf_ui_kit::overlay_frame().show(ui, |ui| {
            // Less the frame's own stroke on each side.
            ui.set_width(panel.width() - space::XL * 2.0 - 2.0);
            // On a short logical screen the settings scroll inside their frame.
            egui::ScrollArea::vertical()
                .max_height(panel.height() - space::XL * 2.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(panel.width() - space::XL * 2.0 - 12.0);
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

                    scale_field(shared, ui);

                    let mut autosave = shared.settings.autosave;
                    let options = vec![
                        (Autosave::Off, shared.tr("settings-autosave-off")),
                        (Autosave::Monthly, shared.tr("settings-autosave-monthly")),
                        (Autosave::Yearly, shared.tr("settings-autosave-yearly")),
                    ];
                    let changed =
                        bf_ui_kit::field(ui, &shared.tr("settings-autosave"), None, |ui| {
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

                    let label = shared.tr("settings-keys");
                    let leader =
                        super::keys_editor::leader_name(shared, shared.keys.config().leader);
                    let hint = shared.i18n.tr_args(
                        "settings-keys-hint",
                        &[("key", bf_i18n::Arg::String(leader))],
                    );
                    bf_ui_kit::field(ui, &label, Some(&hint), |ui| {
                        let open = Button::secondary(&shared.tr("settings-keys-open"))
                            .size(ButtonSize::Small)
                            .show(ui);
                        let open = open.on_hover_text(crate::keys::tip(
                            ui.ctx(),
                            &shared.tr("settings-keys-open"),
                            crate::keys::id::KEYS,
                        ));
                        if crate::keys::pressed(ui, crate::keys::id::KEYS, &open) {
                            shared.key_editor = Some(super::keys_editor::State::default());
                        }
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
                            let finish = Button::primary(&shared.tr("settings-done"))
                                .size(ButtonSize::Medium)
                                .show(ui);
                            if finish.clicked()
                                || crate::keys::single(ui, "settings.done", 'D', finish.rect)
                            {
                                done = true;
                            }
                        },
                    );
                });
        });
    });
    if crate::keys::fired(ui.ctx(), crate::keys::id::BACK) {
        done = true;
    }
    if crate::keys::fired(ui.ctx(), crate::keys::id::KEYS) {
        shared.key_editor = Some(super::keys_editor::State::default());
        done = false;
    }
    done
}

/// The interface scale: automatic, fixed, or sized for a stream; what it gives now, in percent
/// and logical screen. A change applies at once and goes back unless kept.
fn scale_field(shared: &mut Shared, ui: &mut Ui) {
    use crate::ui_scale::{Choice, FIXED, StreamTarget};
    let choice = shared.scale_choice();
    let ppp = ui.ctx().pixels_per_point();
    let logical = ui.ctx().content_rect().size();
    let now = ui.input(|i| i.time);
    let hint = shared.i18n.tr_args(
        "settings-ui-scale-now",
        &[
            ("percent", Arg::Num((ppp * 100.0).round() as f64)),
            ("width", Arg::Num(logical.x.round() as f64)),
            ("height", Arg::Num(logical.y.round() as f64)),
        ],
    );
    let label = shared.tr("settings-ui-scale");
    let mut next = None;
    bf_ui_kit::field(ui, &label, Some(&hint), |ui| {
        let mut mode = match choice {
            Choice::Auto => 0u8,
            Choice::Fixed(_) => 1,
            Choice::Stream(_) => 2,
        };
        let modes = vec![
            (0u8, shared.tr("settings-ui-scale-auto")),
            (1, shared.tr("settings-ui-scale-fixed")),
            (2, shared.tr("settings-ui-scale-stream")),
        ];
        if bf_ui_kit::segmented(ui, &mut mode, &modes) {
            next = Some(match mode {
                0 => Choice::Auto,
                // The fixed step nearest to what the screen shows now.
                1 => Choice::Fixed(
                    FIXED
                        .iter()
                        .copied()
                        .min_by_key(|p| (*p as i32 - (ppp * 100.0).round() as i32).abs())
                        .unwrap_or(100),
                ),
                _ => Choice::Stream(StreamTarget::Hd),
            });
        }
        match choice {
            Choice::Fixed(p) => {
                // A stepper over the fixed steps: fits any width.
                let i = FIXED.iter().position(|s| *s == p).unwrap_or(1);
                let value = shared.i18n.tr_args(
                    "settings-ui-scale-value",
                    &[("percent", Arg::Num(p as f64))],
                );
                let tip = shared.tr("settings-ui-scale-step-tip");
                let step = super::gov_ui::stepper(
                    ui,
                    &value,
                    &tip,
                    i > 0,
                    i + 1 < FIXED.len(),
                    super::gov_ui::StepTips {
                        family: 'Q',
                        id: "settings.scale",
                        row: 0,
                    },
                );
                if step != 0 {
                    let j = (i as i32 + step).clamp(0, FIXED.len() as i32 - 1) as usize;
                    next = Some(Choice::Fixed(FIXED[j]));
                }
            }
            Choice::Stream(t) => {
                let mut value = t;
                let options = vec![
                    (StreamTarget::Hd, shared.tr(StreamTarget::Hd.label())),
                    (StreamTarget::Phone, shared.tr(StreamTarget::Phone.label())),
                ];
                if bf_ui_kit::segmented(ui, &mut value, &options) {
                    next = Some(Choice::Stream(value));
                }
                bf_ui_kit::small(ui, &shared.tr("settings-ui-stream-hint"));
            }
            Choice::Auto => {}
        }
    });
    if let Some(c) = next {
        shared.try_scale(c, now);
    }
}
