//! Main menu: continue, play, settings, quit.

use bf_i18n::Arg;
use bf_ui_kit::{Button, ButtonSize, tokens::space};
use egui::{Rect, Ui};

use super::{backdrop_map, region};
use crate::app::{Shared, Transition};
use crate::keys::{self, id};
use crate::text::date_long;

const COLUMN: f32 = 380.0;

pub fn show(shared: &mut Shared, ui: &mut Ui, rect: Rect) -> Option<Transition> {
    backdrop_map(shared, ui, rect, 0.0);
    let mut out = None;
    let left = rect.left() + (rect.width() * 0.08).max(48.0);
    let column = Rect::from_min_size(
        // Lower on a tall screen; near the top on a short one, so every entry shows.
        egui::pos2(
            left,
            rect.top() + rect.height() * if rect.height() >= 720.0 { 0.22 } else { 0.06 },
        ),
        egui::vec2(COLUMN + 400.0, rect.height() * 0.7),
    );
    let last = shared.saves.first().cloned();
    region(ui, column, |ui| {
        ui.spacing_mut().item_spacing.y = space::SM;
        bf_ui_kit::caption(ui, "1925 — 2025");
        bf_ui_kit::display(ui, &shared.tr("game-title"));
        bf_ui_kit::muted(ui, &shared.tr("game-subtitle"));
        bf_ui_kit::gap(ui, space::HUGE - space::SM);

        let detail = last.as_ref().map(|s| {
            shared.i18n.tr_args(
                "menu-continue-detail",
                &[
                    ("save", Arg::String(s.meta.name.clone())),
                    ("date", Arg::String(date_long(&shared.i18n, s.meta.date))),
                ],
            )
        });
        let empty = shared.tr("menu-continue-empty");
        let continue_label = shared.tr("menu-continue");
        let resume = Button::primary(&continue_label)
            .detail(Some(detail.as_deref().unwrap_or(&empty)))
            .size(ButtonSize::Large)
            .width(COLUMN)
            .align_left()
            .enabled(last.is_some())
            .show(ui);
        if (resume.clicked() || keys::single(ui, "menu.continue", 'C', resume.rect))
            && let Some(save) = last.clone()
        {
            out = Some(if save.meta.player.is_some() {
                Transition::Game(save)
            } else {
                Transition::Pick(save)
            });
        }
        bf_ui_kit::gap(ui, space::XS);
        let play = shared.tr("menu-play");
        let play = Button::secondary(&play)
            .size(ButtonSize::Large)
            .width(COLUMN)
            .align_left()
            .show(ui);
        if play.clicked() || keys::single(ui, "menu.play", 'P', play.rect) {
            out = Some(Transition::Saves);
        }
        bf_ui_kit::gap(ui, space::XS);
        let settings = shared.tr("menu-settings");
        let settings = Button::ghost(&settings)
            .size(ButtonSize::Large)
            .width(COLUMN)
            .align_left()
            .show(ui);
        if keys::pressed(ui, id::SETTINGS, &settings) {
            out = Some(Transition::Settings);
        }
        let quit = shared.tr("menu-quit");
        let quit = Button::ghost(&quit)
            .size(ButtonSize::Large)
            .width(COLUMN)
            .align_left()
            .show(ui);
        if quit.clicked() || keys::single(ui, "menu.quit", 'Q', quit.rect) {
            out = Some(Transition::Quit);
        }
    });
    let ctx = ui.ctx().clone();
    if keys::fired(&ctx, id::SETTINGS) {
        out = Some(Transition::Settings);
    }
    if keys::fired(&ctx, id::KEYS) {
        shared.key_editor = Some(super::keys_editor::State::default());
        out = Some(Transition::Settings);
    }

    let footer = Rect::from_min_size(
        egui::pos2(left, rect.bottom() - 48.0),
        egui::vec2(COLUMN * 2.0, 24.0),
    );
    region(ui, footer, |ui| {
        let version = shared.i18n.tr_args(
            "game-version",
            &[("version", Arg::Str(env!("CARGO_PKG_VERSION")))],
        );
        bf_ui_kit::small(ui, &version);
    });
    if let Some(e) = &shared.load_error {
        let r = Rect::from_min_size(
            egui::pos2(left, rect.bottom() - 96.0),
            egui::vec2(COLUMN * 2.0, 40.0),
        );
        region(ui, r, |ui| bf_ui_kit::error_text(ui, e));
    }
    out
}
