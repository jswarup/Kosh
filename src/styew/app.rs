//-- styew/app.rs -------------------------------------------------------------------------------------------------------------------
use	egui::{ Ui, Color32, RichText, Frame, Margin, Panel, Vec2 };
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
    pub fn	new( cc: &eframe::CreationContext<'_>) -> Self
    {
        // Apply crisp dark theme visuals matching Aura (Catppuccin Mocha / VS Code Dark)
        let  	mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb( 24, 24, 37);            // --bg-surface: #181825
        visuals.window_fill = Color32::from_rgb( 30, 30, 46);           // --bg-base: #1e1e2e
        visuals.override_text_color = Some( Color32::from_rgb( 205, 214, 244)); // --text-primary: #cdd6f4
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb( 30, 30, 46);
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)); // --border: #313244
        cc.egui_ctx.set_visuals( visuals);

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
        // 1. Top Header & Tab Bar Panel (Height & Styling matching Aura)
        Panel::top( "top_panel")
            .frame(
                Frame::new()
                    .fill( Color32::from_rgb( 24, 24, 37))
                    .stroke( egui::Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)))
                    .inner_margin( Margin::symmetric( 12, 6))
            )
            .show( ui, |ui| {
                ui.horizontal( |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new( 8.0, 0.0);

                    // Brand Title
                    ui.label( RichText::new( "FENST").strong().size( 12.5).color( Color32::from_rgb( 137, 180, 250)));
                    ui.label( RichText::new( "/").size( 12.0).color( Color32::from_rgb( 108, 112, 134)));
                    ui.label( RichText::new( "Rust-Native Graphics Workspace").size( 12.0).color( Color32::from_rgb( 166, 173, 200)));

                    ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new( "⚡ 100% Rust-Native (egui + wgpu)")
                                .size( 11.0)
                                .color( Color32::from_rgb( 203, 166, 247))
                        );
                    });
                });

                ui.add_space( 4.0);
                RenderTabBar( ui, &mut self._State);
            });

        // 2. Bottom Status Bar Panel (Height: 24px, #11111b matching Aura)
        Panel::bottom( "bottom_panel")
            .frame(
                Frame::new()
                    .fill( Color32::from_rgb( 17, 17, 27))
                    .stroke( egui::Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)))
                    .inner_margin( Margin::symmetric( 12, 4))
            )
            .show( ui, |ui| {
                ui.horizontal( |ui| {
                    ui.label(
                        RichText::new( &self._State._StatusMessage)
                            .monospace()
                            .size( 11.5)
                            .color( Color32::from_rgb( 166, 173, 200))
                    );

                    ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new( "In-Process Direct GPU Pipeline")
                                .monospace()
                                .size( 11.0)
                                .color( Color32::from_rgb( 137, 180, 250))
                        );
                    });
                });
            });

        // 3. Left Sidebar Explorer Panel (Width: 260px, #181825 matching Aura)
        if self._State._IsExplorerOpen {
            Panel::left( "left_panel")
                .resizable( true)
                .default_size( 260.0)
                .frame(
                    Frame::new()
                        .fill( Color32::from_rgb( 24, 24, 37))
                        .stroke( egui::Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)))
                        .inner_margin( Margin::same( 10))
                )
                .show( ui, |ui| {
                    RenderExplorer( ui, &mut self._State);
                });
        }

        // 4. Central Document Viewport Area (#1e1e2e matching Aura)
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
                        ui.horizontal( |ui| {
                            ui.label( RichText::new( &tab._Name).strong().size( 13.0).color( Color32::from_rgb( 205, 214, 244)));
                            ui.label( RichText::new( format!( "{} lines | {} bytes", tab._LineCount, tab._Size)).size( 11.0).color( Color32::from_rgb( 108, 112, 134)));
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().show( ui, |ui| {
                            ui.label(
                                RichText::new( &tab._Content)
                                    .monospace()
                                    .size( 12.0)
                                    .color( Color32::from_rgb( 205, 214, 244))
                            );
                        });
                    });
                }
            } else {
                // Empty state matching Aura
                ui.centered_and_justified( |ui| {
                    ui.vertical_centered( |ui| {
                        ui.label( RichText::new( "❖").size( 42.0).color( Color32::from_rgba_premultiplied( 205, 214, 244, 30)));
                        ui.add_space( 8.0);
                        ui.label( RichText::new( "Select a file from the explorer to open").strong().size( 13.5).color( Color32::from_rgb( 166, 173, 200)));
                        ui.add_space( 4.0);
                        ui.label( RichText::new( "Supports .pts point clouds, .obj 3D meshes, and fresco:// symbolic trees").size( 11.5).color( Color32::from_rgb( 108, 112, 134)));
                    });
                });
            }
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
