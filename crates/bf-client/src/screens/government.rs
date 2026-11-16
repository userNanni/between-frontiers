//! Government: one window over the map with a tab per area the player governs (budget, money
//! and debt, laws). Each tab is a function in its own module that draws into the window's body
//! and pushes the commands the player gave; this module owns the window, the tabs and the
//! hand-off of the commands to the simulation.

use bf_i18n::I18n;
use bf_sim::game::GameCommand;
use bf_ui_kit::tokens::space;
use bf_ui_kit::{Button, ButtonSize};
use egui::{Rect, Ui, Vec2};

use super::region;
use crate::government::GovernmentView;
use crate::keys::{self, Tip, id};
use crate::sim::Snapshot;
use crate::world::World;
use bf_sim::econ::Catalogue;
use bf_sim::econ::migration::MigrationLaw;
use bf_sim::econ::money::CentralBank;
use bf_sim::econ::state::{PUBLIC_CATEGORIES, TAX_CLASSES};

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

    /// The command that opens the window on this tab.
    pub fn command(self) -> &'static str {
        match self {
            Tab::Budget => id::TAB_BUDGET,
            Tab::Money => id::TAB_MONEY,
            Tab::Laws => id::TAB_LAWS,
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            Tab::Budget => "gov-tab-budget",
            Tab::Money => "gov-tab-money",
            Tab::Laws => "gov-tab-laws",
        }
    }
}

/// What the player changed and the simulation has not applied yet: commands apply at the start
/// of the next tick, so a paused game would otherwise show the old value. Cleared once a later
/// tick is published.
#[derive(Clone, Debug, Default)]
pub struct Drafts {
    /// Tick the drafts were made on.
    pub tick: u64,
    pub tax: Option<Option<[u16; TAX_CLASSES]>>,
    pub priority: Option<[u8; PUBLIC_CATEGORIES]>,
    pub migration: Option<MigrationLaw>,
    pub rate: Option<Option<u16>>,
    pub bank: Option<CentralBank>,
}

impl Drafts {
    /// Records a draft made on `tick`.
    pub fn touch(&mut self, tick: u64) {
        if self.tick != tick {
            *self = Self {
                tick,
                ..Self::default()
            };
        }
    }
}

/// The open window: its tab and the player's pending changes.
#[derive(Clone, Debug, Default)]
pub struct Window {
    pub tab: Tab,
    pub drafts: Drafts,
}

/// What a tab gets to draw with.
pub struct TabContext<'a> {
    pub i18n: &'a I18n,
    pub snap: &'a Snapshot,
    /// The player's treasury, currency and debt; `None` without an economy.
    pub view: Option<&'a GovernmentView>,
    pub catalogue: &'a Catalogue,
    /// Map states and countries, for names; `None` before the map loads.
    pub world: Option<&'a World>,
    pub drafts: &'a mut Drafts,
    /// Tag of the player's country.
    pub tag: &'a str,
    /// Commands for the simulation; they apply at the start of the next tick.
    pub commands: &'a mut Vec<GameCommand>,
}

#[derive(Default)]
pub struct GovernmentResponse {
    pub close: bool,
    /// The player finished moving or resizing the window.
    pub place_done: bool,
    pub commands: Vec<GameCommand>,
}

/// Space kept clear of the scroll bar on the right of the window's body.
const SCROLL_GUTTER: f32 = 16.0;
/// Window size: wide enough for two columns of figures, short enough for 1600 × 900.
const WINDOW: Vec2 = Vec2::new(1040.0, 700.0);

/// The smallest the player may make the window.
const MIN_WINDOW: Vec2 = Vec2::new(720.0, 420.0);

/// What the window reads from the campaign.
pub struct Sources<'a> {
    pub i18n: &'a I18n,
    pub catalogue: &'a Catalogue,
    pub world: Option<&'a World>,
    pub snap: &'a Snapshot,
    /// Tag of the player's country, from the campaign.
    pub tag: Option<&'a str>,
}

pub fn show(
    ui: &mut Ui,
    screen: Rect,
    src: Sources<'_>,
    window: &mut Window,
    place: &mut crate::hud::WindowPlace,
) -> GovernmentResponse {
    let Sources {
        i18n,
        catalogue,
        world,
        snap,
        tag,
    } = src;
    let mut out = GovernmentResponse::default();
    let top = screen.top() + super::top_bar::LEFT_BOTTOM;
    let size = WINDOW.min(Vec2::new(
        screen.width() - 32.0,
        screen.bottom() - top - space::LG,
    ));
    let default = Rect::from_min_size(egui::pos2(screen.center().x - size.x / 2.0, top), size);
    let (rect, moved) = super::window_frame(ui, "government", default, screen, place, MIN_WINDOW);
    out.place_done |= moved;
    // A later tick carries what the player changed.
    if snap.tick > window.drafts.tick {
        window.drafts = Drafts::default();
    }
    let Window { tab, drafts } = window;
    region(ui, rect.shrink(space::XL), |ui| {
        ui.horizontal(|ui| {
            bf_ui_kit::title(ui, &i18n.tr("gov-title"));
            ui.add_space(space::LG);
            let options: Vec<(Tab, String)> = Tab::ALL
                .iter()
                .map(|t| (*t, i18n.tr(t.label_key())))
                .collect();
            let tips = Tab::ALL.map(|t| Tip::Command(t.command()));
            keys::segmented(ui, tab, &options, &tips);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close = Button::ghost(&i18n.tr("gov-close"))
                    .size(ButtonSize::Small)
                    .show(ui);
                let close =
                    close.on_hover_text(keys::tip(ui.ctx(), &i18n.tr("gov-close"), id::BACK));
                if close.clicked() || keys::single(ui, "gov.close", 'X', close.rect) {
                    out.close = true;
                }
            });
        });
        bf_ui_kit::divider(ui);
        let Some(tag) = tag else {
            bf_ui_kit::muted(ui, &i18n.tr("gov-no-player"));
            return;
        };
        let mut cx = TabContext {
            i18n,
            snap,
            view: snap.government.as_deref(),
            catalogue,
            world,
            drafts,
            tag,
            commands: &mut out.commands,
        };
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Room on the right for the scroll bar.
                let width = ui.available_width() - SCROLL_GUTTER;
                ui.set_max_width(width);
                match *tab {
                    Tab::Budget => super::budget::show(ui, &mut cx),
                    Tab::Money => super::money::show(ui, &mut cx),
                    Tab::Laws => super::laws::show(ui, &mut cx),
                }
            });
    });
    out.place_done |= super::window_grip(ui, "government", rect, place, MIN_WINDOW);
    out
}

/// What a tab says when the simulation has no figures for the player yet: before the first
/// day of a new campaign the player is not in the world yet.
pub fn waiting_key(snap: &Snapshot) -> &'static str {
    if snap.player.is_none() {
        "gov-waiting"
    } else {
        "gov-no-economy"
    }
}
