//! Native game client of Between Frontiers (egui over wgpu).

pub mod app;
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
pub mod orders_view;
pub mod paths;
pub mod points;
pub mod population;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod sim;
pub mod text;
pub mod treaty_view;
pub mod ui_scale;
pub mod works_view;
pub mod world;

pub use app::App;
