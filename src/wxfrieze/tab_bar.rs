//-- wxfrieze/tab_bar.rs --------------------------------------------------------------------------------------------------------------
//! Small helpers around the native `wxNotebook` tab strip used as Kosh's document tab bar.
use wxdragon::prelude::*;

/// Closes the currently active tab, unless it is the fixed Explorer tab (index 0).
pub fn close_active_tab(notebook: &Notebook) {
    let selection = notebook.selection();
    if selection > 0 {
        notebook.remove_page(selection as usize);
    }
}
