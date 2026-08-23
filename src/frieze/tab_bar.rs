//-- frieze/tab_bar.rs --------------------------------------------------------------------------------------------------------------
//! Small helpers around the native wxNotebook tab strip used as Kosh's document tab bar.
use wxdragon::prelude::*;
use crate::frieze::state::SharedState;

/// Closes the currently active tab, unless it is the fixed Explorer tab (index 0).
pub fn close_active_tab(notebook: &Notebook, state: &SharedState) {
    let selection = notebook.selection();
    if selection >= 0 {
        let idx = selection as usize;
        let mut st = state.borrow_mut();
        if idx < st._OpenTabs.len() {
            st._OpenTabs.remove(idx);
        }
        notebook.remove_page(idx);
    }
}