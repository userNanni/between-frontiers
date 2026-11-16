//! Government: one window over the map with a tab per area the player governs (budget, money
//! and debt, laws). Each tab is a function in its own module that draws into the window's body
//! and pushes the commands the player gave; this module owns the window, the tabs and the
//! hand-off of the commands to the simulation.

use bf_i18n::I18n;
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::space;
use bf_ui_kit::{Button, ButtonSize};
use egui::{Rect, Sense, Ui, Vec2};

use super::region;
use crate::sim::Snapshot;

/// The window's tabs, in the order they show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Budget,
    Money,
    Laws,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Budget, Tab::Money, Tab::Laws];

    fn label_key(self) -> &'static str {
        match self {
            Tab::Budget => "gov-tab-budget",
            Tab::Money => "gov-tab-money",
            Tab::Laws => "gov-tab-laws",
        }
    }
}

/// What a tab gets to draw with.
pub struct TabContext<'a> {
    pub i18n: &'a I18n,
    pub snap: &'a Snapshot,
    /// Tag of the player's country.
    pub tag: &'a str,
    /// Commands for the simulation; they apply at the start of the next tick.
    pub commands: &'a mut Vec<GameCommand>,
}

#[derive(Default)]
pub struct GovernmentResponse {
    pub close: bool,
    pub commands: Vec<GameCommand>,
}

/// Window size: wide enough for two columns of figures, short enough for 1600 × 900.
const WINDOW: Vec2 = Vec2::new(1040.0, 700.0);

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    i18n: &I18n,
    snap: &Snapshot,
    tab: &mut Tab,
) -> GovernmentResponse {
    let mut out = GovernmentResponse::default();
    let top = screen.top() + super::top_bar::RIGHT_BOTTOM + space::SM;
    let size = WINDOW.min(Vec2::new(
        screen.width() - 32.0,
        screen.bottom() - top - space::LG,
    ));
    let rect = Rect::from_min_size(egui::pos2(screen.center().x - size.x / 2.0, top), size);
    bf_ui_kit::paint_overlay(ui, rect);
    ui.interact(
        rect,
        ui.make_persistent_id("government"),
        Sense::click_and_drag(),
    );
    let Some(tag) = snap.player.as_deref() else {
        return out;
    };
    region(ui, rect.shrink(space::XL), |ui| {
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("gov-title"));
            ui.add_space(space::LG);
            let options: Vec<(Tab, String)> = Tab::ALL
                .iter()
                .map(|t| (*t, i18n.tr(t.label_key())))
                .collect();
            bf_ui_kit::segmented(ui, tab, &options);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if Button::ghost(&i18n.tr("gov-close"))
                    .size(ButtonSize::Small)
                    .show(ui)
                    .clicked()
                {
                    out.close = true;
                }
            });
        });
        bf_ui_kit::divider(ui);
        let mut cx = TabContext {
            i18n,
            snap,
            tag,
            commands: &mut out.commands,
        };
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match *tab {
                Tab::Budget => super::budget::show(ui, &mut cx),
                Tab::Money => super::money::show(ui, &mut cx),
                Tab::Laws => super::laws::show(ui, &mut cx),
            });
    });
    out
}
