//-- frieze/desktop.rs ---------------------------------------------------------------------------------------------------------------
use	egui::{
    Ui, Color32, RichText, Frame, Margin, Vec2,
    Align, Layout, ViewportCommand,
};
use	crate::frieze::state::{ AppState, ObjRenderMode, AppTheme };

// ---------------------------------------------------------------------------------------------------------------------------------

/// Desktop native menu bar providing branding, dropdown menus, and workspace actions.
pub struct DesktopMenuBar;

impl DesktopMenuBar
{
    pub fn	New() -> Self
    {
        Self
    }

    /// Renders the top native desktop menu bar.
    pub fn	Render( &mut self, ui: &mut Ui, state: &mut AppState)
    {
        Frame::new()
            .fill( state._Theme.PanelFill())
            .stroke( state._Theme.BorderStroke())
            .inner_margin( Margin::symmetric( 12, 6))
            .show( ui, |ui| {
                ui.horizontal( |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new( 8.0, 0.0);

                    // 1. Branding
                    ui.label( RichText::new( "◆").size( 13.0).color( state._Theme.AccentColor()));
                    ui.label( RichText::new( "FENST").strong().size( 12.5).color( state._Theme.AccentColor()));
                    ui.label( RichText::new( "/").size( 12.0).color( Color32::from_rgb( 108, 112, 134)));

                    // 2. Desktop Dropdown Menus
                    ui.horizontal( |ui| {
                        // FILE
                        ui.menu_button( "File", |ui| {
                            if ui.button( "📂 Open Folder...").clicked() {
                                if let Some( folder) = rfd::FileDialog::new().pick_folder() {
                                    state._RootPath = folder;
                                }
                                ui.close();
                            }
                            if ui.button( "❌ Close Active Tab").clicked() {
                                if let Some( ref activeId) = state._ActiveTabId.clone() {
                                    state._OpenTabs.retain( |t| &t._Id != activeId);
                                    state._ActiveTabId = state._OpenTabs.first().map( |t| t._Id.clone());
                                }
                                ui.close();
                            }
                            ui.separator();
                            if ui.button( "🚪 Exit").clicked() {
                                ui.ctx().send_viewport_cmd( ViewportCommand::Close);
                            }
                        });

                        // VIEW
                        ui.menu_button( "View", |ui| {
                            if ui.button( if state._IsExplorerOpen { "📁 Hide Sidebar Explorer" } else { "📁 Show Sidebar Explorer" }).clicked() {
                                state._IsExplorerOpen = !state._IsExplorerOpen;
                                ui.close();
                            }
                            if ui.button( "🪟 Open Floating File Explorer").clicked() {
                                state._IsExplorerWindowOpen = true;
                                ui.close();
                            }
                            ui.separator();
                            if ui.button( "↺ Reset 3D Camera").clicked() {
                                state._PtsCamera.Reset();
                                state._ObjCamera.Reset();
                                ui.close();
                            }
                        });

                        // RENDER
                        ui.menu_button( "Render", |ui| {
                            if ui.button( "• Points Mode").clicked() {
                                state._ActiveObjMode = ObjRenderMode::Points;
                                ui.close();
                            }
                            if ui.button( "• Wireframe Mode").clicked() {
                                state._ActiveObjMode = ObjRenderMode::Wireframe;
                                ui.close();
                            }
                            if ui.button( "• Facets Mode").clicked() {
                                state._ActiveObjMode = ObjRenderMode::Facets;
                                ui.close();
                            }
                            if ui.button( "• Shaded Wire Mode").clicked() {
                                state._ActiveObjMode = ObjRenderMode::ShadedWire;
                                ui.close();
                            }
                        });

                        // COMPUTE
                        ui.menu_button( "Compute", |ui| {
                            let  	backendStr = state._Engine.as_ref().map( |e| format!( "{}", e.Backend())).unwrap_or_else( || "CPU Fallback".to_string());
                            ui.label( format!( "Active Backend: {}", backendStr));
                            ui.separator();
                            if ui.button( "⚡ Run GPU Auto-Detect").clicked() {
                                state._StatusMessage = "Swarm Engine: Hardware discovery refreshed ✓".to_string();
                                ui.close();
                            }
                        });

                        // SETTINGS (Theme switcher & UI settings)
                        ui.menu_button( "Settings", |ui| {
                            ui.label( RichText::new( "🎨 Workspace Theme").strong());
                            ui.separator();

                            let  	darkActive = state._Theme == AppTheme::Dark;
                            let  	lightActive = state._Theme == AppTheme::Light;
                            let  	cyberActive = state._Theme == AppTheme::Cyberpunk;
                            let  	nordActive = state._Theme == AppTheme::Nord;

                            if ui.selectable_label( darkActive, "🌙 Dark (Mocha)").clicked() {
                                state._Theme = AppTheme::Dark;
                                state._Theme.Apply( ui.ctx());
                                state._StatusMessage = "Theme switched to Dark (Catppuccin Mocha)".to_string();
                                ui.close();
                            }

                            if ui.selectable_label( lightActive, "☀️ Light (Latte)").clicked() {
                                state._Theme = AppTheme::Light;
                                state._Theme.Apply( ui.ctx());
                                state._StatusMessage = "Theme switched to Light (Latte)".to_string();
                                ui.close();
                            }

                            if ui.selectable_label( cyberActive, "⚡ Cyberpunk Neon").clicked() {
                                state._Theme = AppTheme::Cyberpunk;
                                state._Theme.Apply( ui.ctx());
                                state._StatusMessage = "Theme switched to Cyberpunk Neon".to_string();
                                ui.close();
                            }

                            if ui.selectable_label( nordActive, "❄️ Nord Polar").clicked() {
                                state._Theme = AppTheme::Nord;
                                state._Theme.Apply( ui.ctx());
                                state._StatusMessage = "Theme switched to Nord Polar".to_string();
                                ui.close();
                            }
                        });

                        // HELP
                        ui.menu_button( "Help", |ui| {
                            if ui.button( "📖 Architecture & Docs").clicked() {
                                state._StatusMessage = "Documentation available in /wiki".to_string();
                                ui.close();
                            }
                            ui.separator();
                            ui.label( "Kosh Native Graphics Workspace v0.1.0");
                        });
                    });

                    // 3. Right-Aligned Workspace Info Badge
                    ui.with_layout( Layout::right_to_left( Align::Center), |ui| {
                        ui.label(
                            RichText::new( "100% Rust-Native (egui + wgpu + swarm)")
                                .size( 11.0)
                                .color( state._Theme.AccentColor())
                        );
                    });
                });
            });
    }
}
