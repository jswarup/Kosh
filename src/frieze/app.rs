//-- frieze/app.rs -------------------------------------------------------------------------------------------------------------------
use	std::sync::Arc;
use	egui::{ Ui, Color32, RichText, Frame, Margin, Panel };
use	crate::swarm::{ SwarmEngine, ViewportRenderer };
use	crate::frieze::state::AppState;
use	crate::frieze::desktop::DesktopMenuBar;
use	crate::frieze::tab_bar::RenderTabBar;
use	crate::frieze::explorer::{ RenderExplorer, RenderFloatingExplorerWindow, RenderExplorerViewTab };
use	crate::frieze::pts_view::RenderPtsView;
use	crate::frieze::obj_view::RenderObjView;
use	crate::frieze::fresco_view::RenderFrescoView;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Primary native application struct implementing eframe::App with native desktop look and feel.
pub struct KoshApp
{
    pub _State:   AppState,
    pub _MenuBar: DesktopMenuBar,
}

impl KoshApp
{
    pub fn	new( cc: &eframe::CreationContext<'_>) -> Self
    {
        let  	mut appState = AppState::default();
        appState._Theme.Apply( &cc.egui_ctx);

        if let Some( ref renderState) = cc.wgpu_render_state {
            let  	devArc = Arc::new( renderState.device.clone());
            let  	queueArc = Arc::new( renderState.queue.clone());
            let  	eng = SwarmEngine::FromSharedGpu( devArc.clone(), queueArc.clone());
            let  	rend = Arc::new( ViewportRenderer::New( devArc, queueArc, renderState.target_format));
            appState._Engine = Some( eng);
            appState._ViewportRenderer = Some( rend);
        }

        Self {
            _State:   appState,
            _MenuBar: DesktopMenuBar::New(),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl eframe::App for KoshApp
{
    fn	ui( &mut self, ui: &mut Ui, _frame: &mut eframe::Frame)
    {
        // 0. Render Floating Windows File Explorer if open
        RenderFloatingExplorerWindow( ui.ctx(), &mut self._State);

        // 1. Top Desktop Menu Bar
        self._MenuBar.Render( ui, &mut self._State);

        // 2. Tab Bar Header Panel
        Panel::top( "tab_bar_panel")
            .frame(
                Frame::new()
                    .fill( self._State._Theme.PanelFill())
                    .stroke( self._State._Theme.BorderStroke())
                    .inner_margin( Margin::symmetric( 12, 6))
            )
            .show( ui, |ui| {
                RenderTabBar( ui, &mut self._State);
            });

        // 3. Bottom Status Bar Panel
        Panel::bottom( "bottom_panel")
            .frame(
                Frame::new()
                    .fill( self._State._Theme.BottomFill())
                    .stroke( self._State._Theme.BorderStroke())
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
                            RichText::new( "Pure Native Rust (egui + wgpu + swarm)")
                                .monospace()
                                .size( 11.0)
                                .color( self._State._Theme.AccentColor())
                        );
                    });
                });
            });

        // 4. Left Sidebar Explorer Panel
        if self._State._IsExplorerOpen {
            Panel::left( "left_panel")
                .resizable( true)
                .default_size( 260.0)
                .frame(
                    Frame::new()
                        .fill( self._State._Theme.PanelFill())
                        .stroke( self._State._Theme.BorderStroke())
                        .inner_margin( Margin::same( 10))
                )
                .show( ui, |ui| {
                    RenderExplorer( ui, &mut self._State);
                });
        }

        // 5. Central Document Viewport Area
        let  	activeTab = self._State._OpenTabs.iter().find( |t| Some( &t._Id) == self._State._ActiveTabId.as_ref()).cloned();

        ui.vertical( |ui| {
            if let  	Some( tab) = activeTab {
                if tab._IsExplorer {
                    RenderExplorerViewTab( ui, &mut self._State);
                } else if tab._IsPts {
                    RenderPtsView( ui, &tab._Path, &mut self._State);
                } else if tab._IsObj {
                    RenderObjView( ui, &tab._Path, &mut self._State);
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
                ui.centered_and_justified( |ui| {
                    ui.vertical_centered( |ui| {
                        ui.label( RichText::new( "KOSH").size( 42.0).color( Color32::from_rgba_premultiplied( 205, 214, 244, 30)));
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
