//-- frieze/mod.rs -----------------------------------------------------------------------------------------------------------------
//! Native wxDragon (wxWidgets) desktop GUI facade replacement for the egui/eframe based rieze module.
pub mod state;
pub mod gpu_cache;
pub mod desktop;
pub mod tab_bar;
pub mod explorer;
pub mod geom_view;
pub mod fresco_view;
pub mod img_view;
pub mod wave_view;
pub mod app;

pub use app::run;
