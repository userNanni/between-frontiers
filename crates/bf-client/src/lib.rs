//! Native game client of Between Frontiers (egui over wgpu).

pub mod app;
pub mod flags;
pub mod government;
pub mod hud;
pub mod journal;
pub mod keys;
pub mod laws_view;
pub mod ledger_view;
pub mod live;
pub mod map;
pub mod paths;
pub mod points;
pub mod population;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod sim;
pub mod text;
pub mod ui_scale;
pub mod world;

pub use app::App;
