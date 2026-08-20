//-- styew/app.rs -------------------------------------------------------------------------------------------------------------------
use	egui::{ Ui, Color32, RichText, Frame, Margin, Panel };
use	crate::styew::state::AppState;
use	crate::styew::tab_bar::RenderTabBar;
use	crate::styew::explorer::RenderExplorer;
use	crate::styew::pts_view::RenderPtsView;
use	crate::styew::obj_view::RenderObjView;
use	crate::styew::fresco_view::RenderFrescoView;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Primary native application struct implementing eframe::App.
pub struct KoshApp
{
    pub _State: AppState,
}

impl KoshApp
{
    pub fn	new( _cc: &eframe::CreationContext<'_>) -> Self
    {
        Self {
            _State: AppState::default(),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl eframe::App for KoshApp
{
    fn	ui( &mut self, ui: &mut Ui, _frame: &mut eframe::Frame)
    {
        let  	ctx = ui.ctx().clone();
        let  	mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb( 11, 15, 25);
        visuals.window_fill = Color32::from_rgb( 11, 15, 25);
        visuals.override_text_color = Some( Color32::from_rgb( 248, 250, 252));
        ctx.set_visuals( visuals);

        // 1. Top Header & Tab Bar Panel
        Panel::top( "top_panel")
            .frame( Frame::new().fill( Color32::from_rgb( 8, 12, 22)).inner_margin( Margin::symmetric( 12, 6)))
            .show( ui, |ui| {
                ui.horizontal( |ui| {
                    ui.label( RichText::new( "KOSH / FENST").strong().size( 14.0).color( Color32::from_rgb( 0, 243, 255)));
                    ui.label( RichText::new( "• Native 3D GPU Workspace").size( 11.0).color( Color32::from_rgb( 148, 163, 184)));

                    ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                        ui.label( RichText::new( "⚡ 100% Rust-Native (egui + wgpu)").size( 11.0).color( Color32::from_rgb( 168, 85, 247)));
                    });
                });

                ui.add_space( 4.0);
                RenderTabBar( ui, &mut self._State);
            });

        // 2. Bottom Status Bar Panel
        Panel::bottom( "bottom_panel")
            .frame( Frame::new().fill( Color32::from_rgb( 6, 9, 17)).inner_margin( Margin::symmetric( 12, 4)))
            .show( ui, |ui| {
                ui.horizontal( |ui| {
                    ui.label( RichText::new( &self._State._StatusMessage).monospace().size( 11.0).color( Color32::from_rgb( 100, 116, 139)));
                    ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                        ui.label( RichText::new( "In-Process GPU Direct Context").monospace().size( 11.0).color( Color32::from_rgb( 0, 243, 255)));
                    });
                });
            });

        // 3. Left Sidebar Explorer Panel
        if self._State._IsExplorerOpen {
            Panel::left( "left_panel")
                .resizable( true)
                .default_size( 260.0)
                .frame( Frame::new().fill( Color32::from_rgb( 11, 15, 25)).inner_margin( Margin::same( 10)))
                .show( ui, |ui| {
                    RenderExplorer( ui, &mut self._State);
                });
        }

        // 4. Central Document Viewport Area (Remaining Space in UI)
        let  	activeTab = self._State._OpenTabs.iter().find( |t| Some( &t._Id) == self._State._ActiveTabId.as_ref()).cloned();

        ui.vertical( |ui| {
            if let  	Some( tab) = activeTab {
                if tab._IsPts {
                    RenderPtsView( ui, &tab._Path, &mut self._State._PtsCamera);
                } else if tab._IsObj {
                    RenderObjView( ui, &tab._Path, &mut self._State._ObjCamera, &mut self._State._ActiveObjMode);
                } else if tab._IsFresco {
                    RenderFrescoView( ui, &tab._Path.to_string_lossy());
                } else {
                    // Text Document Viewer
                    ui.vertical( |ui| {
                        ui.label( RichText::new( &tab._Name).strong().size( 13.0));
                        ui.separator();
                        egui::ScrollArea::vertical().show( ui, |ui| {
                            ui.label( RichText::new( &tab._Content).monospace().size( 12.0).color( Color32::from_rgb( 226, 232, 240)));
                        });
                    });
                }
            } else {
                // Empty state
                ui.centered_and_justified( |ui| {
                    ui.vertical_centered( |ui| {
                        ui.label( RichText::new( "❖").size( 48.0).color( Color32::from_rgba_premultiplied( 255, 255, 255, 30)));
                        ui.add_space( 12.0);
                        ui.label( RichText::new( "Select a file or expression from the explorer to view").strong().size( 14.0).color( Color32::from_rgb( 148, 163, 184)));
                        ui.add_space( 6.0);
                        ui.label( RichText::new( "Supports .pts point clouds, .obj 3D models, and fresco:// symbolic trees").size( 11.0).color( Color32::from_rgb( 100, 116, 139)));
                    });
                });
            }
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
