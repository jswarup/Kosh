//-- styew/tab_bar.rs ---------------------------------------------------------------------------------------------------------------
use	egui::{ Ui, Color32, Vec2, RichText, Frame, Stroke, Margin };
use	crate::styew::state::AppState;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Renders the top tab bar in immediate mode.
pub fn	RenderTabBar( ui: &mut Ui, state: &mut AppState)
{
    if state._OpenTabs.is_empty() {
        return;
    }

    let  	mut tabToClose: Option< String> = None;
    let  	mut tabToSelect: Option< String> = None;

    ui.horizontal( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 4.0, 0.0);

        for tab in state._OpenTabs.iter() {
            let  	isActive = state._ActiveTabId.as_ref() == Some( &tab._Id);
            let  	tabId = tab._Id.clone();

            let  	bg = if isActive {
                Color32::from_rgb( 17, 24, 38)
            } else {
                Color32::from_rgba_premultiplied( 11, 15, 25, 180)
            };

            let  	stroke = if isActive {
                Stroke::new( 1.0, Color32::from_rgb( 0, 243, 255))
            } else {
                Stroke::new( 1.0, Color32::from_rgba_premultiplied( 255, 255, 255, 15))
            };

            Frame::new()
                .fill( bg)
                .stroke( stroke)
                .inner_margin( Margin::symmetric( 8, 4))
                .corner_radius( 4)
                .show( ui, |ui| {
                    ui.horizontal( |ui| {
                        // Badge icon
                        if tab._IsPts {
                            ui.label( RichText::new( "• PTS").color( Color32::from_rgb( 0, 243, 255)).strong().size( 11.0));
                        } else if tab._IsObj {
                            ui.label( RichText::new( "◬ OBJ").color( Color32::from_rgb( 168, 85, 247)).strong().size( 11.0));
                        } else if tab._IsFresco {
                            ui.label( RichText::new( "ƒ FRESCO").color( Color32::from_rgb( 59, 130, 246)).strong().size( 11.0));
                        } else {
                            ui.label( RichText::new( "📄").size( 11.0));
                        }

                        // Tab label
                        let  	tabBtn = ui.selectable_label( false, RichText::new( &tab._Name).color( if isActive { Color32::WHITE } else { Color32::from_rgb( 148, 163, 184) }).size( 12.0));
                        if tabBtn.clicked() {
                            tabToSelect = Some( tabId.clone());
                        }

                        // Close button
                        if ui.small_button( RichText::new( "×").color( Color32::from_rgb( 100, 116, 139))).clicked() {
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
