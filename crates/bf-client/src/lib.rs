//! Native game client of Between Frontiers (egui over wgpu).

pub mod agreements_view;
pub mod app;
pub mod army_view;
pub mod blocs_view;
pub mod crisis_view;
pub mod demands_view;
pub mod flags;
pub mod forces_view;
pub mod government;
pub mod hud;
pub mod influence_view;
pub mod journal;
pub mod keys;
pub mod laws_view;
pub mod ledger_view;
pub mod live;
pub mod map;
pub mod nuclear_view;
pub mod orders_view;
pub mod paths;
pub mod points;
pub mod population;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod sim;
pub mod tariff_view;
pub mod text;
pub mod trade_view;
pub mod treaty_view;
pub mod ui_scale;
pub mod war_view;
pub mod works_view;
pub mod world;

pub use app::App;
