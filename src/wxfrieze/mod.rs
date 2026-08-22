//-- wxfrieze/mod.rs -----------------------------------------------------------------------------------------------------------------
//! Native wxDragon (wxWidgets) desktop GUI facade â€” replacement for the egui/eframe based `frieze` module.
pub mod state;
pub mod gpu_cache;
pub mod desktop;
pub mod tab_bar;
pub mod explorer;
pub mod pts_view;
pub mod obj_view;
pub mod img_view;
pub mod fresco_view;
pub mod app;

pub use app::run;

