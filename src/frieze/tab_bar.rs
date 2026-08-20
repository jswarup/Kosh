//-- frieze/tab_bar.rs ---------------------------------------------------------------------------------------------------------------
use	egui::{ Ui, Color32, Vec2, RichText, Frame, Stroke, Margin };
use	crate::frieze::state::AppState;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Renders the top tab bar matching Aura's exact visual specifications.
pub fn	RenderTabBar( ui: &mut Ui, state: &mut AppState)
{
    if state._OpenTabs.is_empty() {
        return;
    }

    let  	mut tabToClose: Option< String> = None;
    let  	mut tabToSelect: Option< String> = None;

    ui.horizontal( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 3.0, 0.0);

        for tab in state._OpenTabs.iter() {
            let  	isActive = state._ActiveTabId.as_ref() == Some( &tab._Id);
            let  	tabId = tab._Id.clone();

            // Background colors matching Aura (--tab-bg / --tab-active-bg)
            let  	bg = if isActive {
                Color32::from_rgb( 30, 30, 46)                          // #1e1e2e
            } else {
                Color32::from_rgb( 24, 24, 37)                          // #181825
            };

            let  	stroke = if isActive {
                Stroke::new( 1.0, Color32::from_rgb( 137, 180, 250))   // #89b4fa
            } else {
                Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68))      // #313244
            };

            Frame::new()
                .fill( bg)
                .stroke( stroke)
                .inner_margin( Margin::symmetric( 10, 5))
                .corner_radius( 4)
                .show( ui, |ui| {
                    ui.horizontal( |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new( 6.0, 0.0);

                        // Badge icon
                        if tab._IsPts {
                            ui.label( RichText::new( "• PTS").color( Color32::from_rgb( 137, 180, 250)).strong().size( 11.0));
                        } else if tab._IsObj {
                            ui.label( RichText::new( "◬ OBJ").color( Color32::from_rgb( 203, 166, 247)).strong().size( 11.0));
                        } else if tab._IsFresco {
                            ui.label( RichText::new( "ƒ FRESCO").color( Color32::from_rgb( 137, 220, 235)).strong().size( 11.0));
                        } else {
                            ui.label( RichText::new( "📄").size( 11.0));
                        }

                        // Tab title
                        let  	tabTextColor = if isActive {
                            Color32::from_rgb( 205, 214, 244)           // #cdd6f4
                        } else {
                            Color32::from_rgb( 166, 173, 200)           // #a6adc8
                        };

                        let  	tabBtn = ui.selectable_label( false, RichText::new( &tab._Name).color( tabTextColor).size( 12.0));
                        if tabBtn.clicked() {
                            tabToSelect = Some( tabId.clone());
                        }

                        // Close button
                        if ui.small_button( RichText::new( "×").color( Color32::from_rgb( 108, 112, 134)).size( 12.0)).clicked() {
                            tabToClose = Some( tabId.clone());
                        }
                    });
                });
        }
    });

    if let  	Some( id) = tabToSelect {
        state._ActiveTabId = Some( id);
    }

    if let  	Some( id) = tabToClose {
        let  	idx = state._OpenTabs.iter().position( |t| t._Id == id);
        if let  	Some( i) = idx {
            state._OpenTabs.remove( i);
            if state._ActiveTabId.as_ref() == Some( &id) {
                if !state._OpenTabs.is_empty() {
                    let  	nextIdx = i.min( state._OpenTabs.len() - 1);
                    state._ActiveTabId = Some( state._OpenTabs[nextIdx]._Id.clone());
                } else {
                    state._ActiveTabId = None;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
