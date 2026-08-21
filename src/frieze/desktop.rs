//-- frieze/desktop.rs ---------------------------------------------------------------------------------------------------------------
use	egui::{ Ui, Context, Window, Color32, RichText };
use	crate::frieze::state::{ AppState, AppTheme, ThemeMode, FontFamilyPreference };

// ---------------------------------------------------------------------------------------------------------------------------------

/// Desktop Top Menu Bar component providing system menus and settings access.
pub struct DesktopMenuBar;

impl DesktopMenuBar
{
    pub fn	New() -> Self
    {
        Self
    }

    pub fn	Render( &mut self, ui: &mut Ui, state: &mut AppState)
    {
        ui.horizontal( |ui| {
            ui.menu_button( "File", |ui| {
                if ui.button( "Open File Explorer").clicked() {
                    state._IsExplorerWindowOpen = true;
                    ui.close();
                }
                ui.separator();
                if ui.button( "Toggle Explorer Sidebar").clicked() {
                    state._IsExplorerOpen = !state._IsExplorerOpen;
                    ui.close();
                }
            });

            ui.menu_button( "Settings", |ui| {
                if ui.button( "Preferences & Appearance...").clicked() {
                    state._IsSettingsWindowOpen = true;
                    ui.close();
                }

                ui.separator();

                ui.menu_button( "Quick Theme Mode", |ui| {
                    let  	mut changed = false;

                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::System, "Auto (Follow OS)").clicked() {
                        changed = true;
                    }

                    ui.separator();

                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Dark), "Dark").clicked() {
                        state._Theme = AppTheme::Dark;
                        state._Appearance._ExplicitTheme = AppTheme::Dark;
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Light), "Light").clicked() {
                        state._Theme = AppTheme::Light;
                        state._Appearance._ExplicitTheme = AppTheme::Light;
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Cyberpunk), "Cyberpunk").clicked() {
                        state._Theme = AppTheme::Cyberpunk;
                        state._Appearance._ExplicitTheme = AppTheme::Cyberpunk;
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Nord), "Nord").clicked() {
                        state._Theme = AppTheme::Nord;
                        state._Appearance._ExplicitTheme = AppTheme::Nord;
                        changed = true;
                    }

                    if changed {
                        state._Appearance.Apply( ui.ctx());
                    }
                });

                ui.menu_button( "Quick Font Size Offset", |ui| {
                    let  	mut changed = false;
                    if ui.button( "+2 pt Larger").clicked() {
                        state._Appearance._FontSizeOffset += 2.0;
                        changed = true;
                    }
                    if ui.button( "-2 pt Smaller").clicked() {
                        state._Appearance._FontSizeOffset -= 2.0;
                        changed = true;
                    }
                    if ui.button( "Reset (0 pt)").clicked() {
                        state._Appearance._FontSizeOffset = 0.0;
                        changed = true;
                    }
                    if changed {
                        state._Appearance.Apply( ui.ctx());
                    }
                });
            });

            ui.menu_button( "Help", |ui| {
                if ui.button( "About Kosh Native").clicked() {
                    state._StatusMessage = "Kosh Native Workspace v0.1.0 (egui + wgpu + swarm)".to_string();
                    ui.close();
                }
            });
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Floating, movable Settings & Preferences Dialog Window.
pub fn	RenderSettingsWindow( ctx: &Context, state: &mut AppState)
{
    if !state._IsSettingsWindowOpen {
        return;
    }

    let  	mut open = state._IsSettingsWindowOpen;
    let  	mut changed = false;
    let  	mut closeRequested = false;

    Window::new( "Settings & Preferences")
        .open( &mut open)
        .resizable( true)
        .default_size( [440.0, 480.0])
        .show( ctx, |ui| {
            ui.heading( "Appearance & Theme Cascade");
            ui.label( RichText::new( "Primacy: OS System Preference -> Selected Theme Preset -> Custom Overrides").small().color( Color32::from_rgb( 166, 173, 200)));
            ui.add_space( 8.0);

            // Section 1: OS Primacy & Theme Mode
            ui.group( |ui| {
                ui.label( RichText::new( "Theme & OS Integration").strong());
                ui.add_space( 4.0);

                if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::System, "Auto (Follow OS System Theme)").clicked() {
                    changed = true;
                }

                ui.add_space( 4.0);
                ui.label( RichText::new( "Explicit Theme Preset:").small());
                ui.horizontal( |ui| {
                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Dark), "Dark").clicked() {
                        state._Theme = AppTheme::Dark;
                        state._Appearance._ExplicitTheme = AppTheme::Dark;
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Light), "Light").clicked() {
                        state._Theme = AppTheme::Light;
                        state._Appearance._ExplicitTheme = AppTheme::Light;
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Cyberpunk), "Cyberpunk").clicked() {
                        state._Theme = AppTheme::Cyberpunk;
                        state._Appearance._ExplicitTheme = AppTheme::Cyberpunk;
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._ThemeMode, ThemeMode::Explicit( AppTheme::Nord), "Nord").clicked() {
                        state._Theme = AppTheme::Nord;
                        state._Appearance._ExplicitTheme = AppTheme::Nord;
                        changed = true;
                    }
                });

                let  	activeResolved = state._Appearance.ResolveTheme( ui.ctx());
                ui.label( RichText::new( format!( "Active Resolved Theme: {:?}", activeResolved)).small().color( state._Theme.AccentColor()));
            });

            ui.add_space( 8.0);

            // Section 2: Typography & Baselined Font Scaling
            ui.group( |ui| {
                ui.label( RichText::new( "Typography & Font Scaling").strong());
                ui.label( RichText::new( "All font levels scale together proportionally relative to baseline.").small().color( Color32::from_rgb( 166, 173, 200)));
                ui.add_space( 4.0);

                ui.horizontal( |ui| {
                    ui.label( "Font Size Offset:");
                    if ui.add( egui::Slider::new( &mut state._Appearance._FontSizeOffset, -4.0..=10.0).suffix( " pt")).changed() {
                        changed = true;
                    }
                    if ui.button( "Reset").clicked() {
                        state._Appearance._FontSizeOffset = 0.0;
                        changed = true;
                    }
                });

                ui.add_space( 4.0);
                ui.horizontal( |ui| {
                    ui.label( "Font Family:");
                    if ui.radio_value( &mut state._Appearance._FontFamily, FontFamilyPreference::Default, "Default").clicked() {
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._FontFamily, FontFamilyPreference::Proportional, "Proportional").clicked() {
                        changed = true;
                    }
                    if ui.radio_value( &mut state._Appearance._FontFamily, FontFamilyPreference::Monospace, "Monospace").clicked() {
                        changed = true;
                    }
                });
            });

            ui.add_space( 8.0);

            // Section 3: Text Color Override (Tertiary Primacy)
            ui.group( |ui| {
                ui.label( RichText::new( "Text Color Override (High Primacy)").strong());
                ui.label( RichText::new( "Overrides default text color across all themes.").small().color( Color32::from_rgb( 166, 173, 200)));
                ui.add_space( 4.0);

                ui.horizontal( |ui| {
                    if ui.selectable_label( state._Appearance._CustomTextColor.is_none(), "Theme Default").clicked() {
                        state._Appearance._CustomTextColor = None;
                        changed = true;
                    }

                    let  	swatches = [
                        ( "Bright White", Color32::from_rgb( 255, 255, 255)),
                        ( "Amber Gold",   Color32::from_rgb( 255, 191, 0)),
                        ( "Cyan Cyber",    Color32::from_rgb( 0, 255, 204)),
                        ( "Soft Mint",     Color32::from_rgb( 166, 227, 161)),
                        ( "Rose Pink",     Color32::from_rgb( 245, 194, 231)),
                    ];

                    for ( name, color) in swatches {
                        let  	isSelected = state._Appearance._CustomTextColor == Some( color);
                        if ui.selectable_label( isSelected, name).clicked() {
                            state._Appearance._CustomTextColor = Some( color);
                            changed = true;
                        }
                    }
                });
            });

            ui.add_space( 12.0);
            ui.separator();
            ui.horizontal( |ui| {
                ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                    if ui.button( "Close").clicked() {
                        closeRequested = true;
                    }
                });
            });
        });

    if closeRequested {
        open = false;
    }

    state._IsSettingsWindowOpen = open;

    if changed {
        state._Appearance.Apply( ctx);
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
