//! Native game client of Between Frontiers (egui over wgpu).

pub mod app;
pub mod flags;
pub mod government;
pub mod laws_view;
pub mod live;
pub mod map;
pub mod paths;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod sim;
pub mod text;
pub mod world;

pub use app::App;
