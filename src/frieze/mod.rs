//-- frieze/mod.rs -------------------------------------------------------------------------------------------------------------------
pub mod state;
pub mod tab_bar;
pub mod explorer;
pub mod pts_view;
pub mod obj_view;
pub mod fresco_view;
pub mod gpu_cache;
pub mod app;

pub use	app::KoshApp;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Launches the native eframe/egui Kosh application window.
pub fn	run() -> eframe::Result
{
    let  	nativeOptions = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title( "Kosh — Native 3D GPU Workspace")
            .with_inner_size( [1360.0, 840.0])
            .with_min_inner_size( [800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Kosh Native",
        nativeOptions,
        Box::new( |cc| Ok( Box::new( KoshApp::new( cc)))),
    )
}

// ---------------------------------------------------------------------------------------------------------------------------------
