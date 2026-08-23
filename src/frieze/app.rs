//-- frieze/app.rs ------------------------------------------------------------------------------------------------------------------
//! Assembles the native wxDragon Kosh application: main frame, menu bar, status bar, and the
//! Explorer + document Notebook tab strip. This is the wxDragon-based replacement entry point
//! for the egui/eframe `frieze` module.
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::anyhow;
use wxdragon::widgets::{AuiManager, AuiPaneInfo};
use wxdragon::event::menu_events::MenuEvents;
use wxdragon::id::ID_EXIT;
use crate::frieze::desktop::{ID_OPEN, ID_CLOSE};
use wxdragon::prelude::*;

use crate::frieze::desktop::{
    build_menu_bar, ID_THEME_CYBERPUNK, ID_THEME_DARK, ID_THEME_LIGHT, ID_THEME_NORD,
};
use crate::frieze::explorer::build_explorer_panel;
use crate::frieze::fresco_view::build_fresco_view_panel;
use crate::frieze::img_view::build_img_view_panel;
use crate::frieze::geom_view::build_geom_view_panel;
use crate::frieze::state::{AppState, AppTheme, OpenTab, SharedState};
use crate::frieze::tab_bar::close_active_tab;

/// Launches the native wxDragon-based Kosh desktop application window.
pub fn run() -> anyhow::Result<()> {
    let result = wxdragon::main(|_handle| {
        let state: SharedState = Rc::new(RefCell::new(AppState::default()));

        let frame = Frame::builder()
            .with_title("Kosh - Native 3D GPU Workspace (wxDragon)")
            .with_size(Size::new(1360, 840))
            .build();

        frame.set_menu_bar(build_menu_bar());
        let status_bar = frame.create_status_bar(1, 0, wxdragon::id::ID_ANY as i32, "");
        status_bar.set_status_text("Ready - Native Rust (wxDragon + wgpu + swarm)", 0);

        if let Some(toolbar) = frame.create_tool_bar(None, wxdragon::id::ID_ANY as i32) {
            let dummy_icon = wxdragon::bitmap::Bitmap::from_rgba(&vec![255; 16*16*4], 16, 16).unwrap();
            toolbar.add_tool(ID_OPEN, "Open Folder", &dummy_icon, "Open a folder in the explorer");
            toolbar.realize();
        }

        let manager = AuiManager::builder(&frame).build();

        let notebook = Notebook::builder(&frame).build();

        let on_open_file: Rc<dyn Fn(PathBuf)> = {
            let state = state.clone();
            let notebook = notebook.clone();
            Rc::new(move |path: PathBuf| {
                open_tab_for_path(&notebook, &state, &path);
            })
        };

        let explorer_panel = build_explorer_panel(&frame, state.clone(), on_open_file);
        
        let output_log = TextCtrl::builder(&frame)
            .with_style(wxdragon::widgets::textctrl::TextCtrlStyle::MultiLine | wxdragon::widgets::textctrl::TextCtrlStyle::ReadOnly)
            .build();
        output_log.set_value("Console Output\n");

        manager.add_pane_with_info(&notebook, AuiPaneInfo::new().with_name("CentralCanvas").center_pane().pane_border(false));
        manager.add_pane_with_info(&explorer_panel, AuiPaneInfo::new().with_name("LeftExplorer").with_caption("Explorer").left().best_size(240, -1).min_size(150, -1));
        manager.add_pane_with_info(&output_log, AuiPaneInfo::new().with_name("OutputLog").with_caption("Output").bottom().best_size(-1, 150).min_size(-1, 80));

        manager.update();

        {
            let state = state.clone();
            frame.on_menu_selected(move |evt| match evt.get_id() {
                ID_OPEN => {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        state.borrow_mut()._RootPath = folder;
                    }
                }
                ID_CLOSE => close_active_tab(&notebook, &state),
                ID_EXIT => {
                    frame.close(false);
                }
                ID_THEME_DARK => state.borrow_mut()._Theme = AppTheme::Dark,
                ID_THEME_LIGHT => state.borrow_mut()._Theme = AppTheme::Light,
                ID_THEME_CYBERPUNK => state.borrow_mut()._Theme = AppTheme::Cyberpunk,
                ID_THEME_NORD => state.borrow_mut()._Theme = AppTheme::Nord,
                _ => {}
            });
        }

        frame.show(true);
        frame.centre();
    });

    result.map_err(|e| anyhow!("Failed to launch wxDragon application: {e:?}"))
}

/// Opens (or focuses) a document tab for the given file path, choosing the right native
/// viewport (points / mesh / fresco / plain text) based on file extension.
fn open_tab_for_path(notebook: &Notebook, state: &SharedState, path: &Path) {
    let path_buf = path.to_path_buf();
    let already_open = state.borrow()._OpenTabs.iter().position(|tab| tab._Path == path_buf);
    if let Some(idx) = already_open {
        notebook.set_selection(idx);
        return;
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();

    let is_pts = ext == "pts";
    let is_obj = ext == "obj";
    let is_fresco = ext == "fresco" || ext == "frsc";
    let is_img = ext == "png" || ext == "jpg" || ext == "jpeg";

    let label = if is_pts {
        format!("PTS  {name}")
    } else if is_obj {
        format!("OBJ  {name}")
    } else if is_fresco {
        format!("FRESCO  {name}")
    } else if is_img {
        format!("IMG  {name}")
    } else {
        format!("{name}")
    };

    if is_pts || is_obj {
        let page = build_geom_view_panel(notebook, state.clone(), path.to_path_buf());
        notebook.add_page(&page, &label, true, None);
    } else if is_fresco {
        let page = build_fresco_view_panel(notebook, &path.to_string_lossy());
        notebook.add_page(&page, &label, true, None);
    } else if is_img {
        let page = build_img_view_panel(notebook, state.clone(), path.to_path_buf());
        notebook.add_page(&page, &label, true, None);
    } else {
        let page = build_text_view_panel(notebook, path);
        notebook.add_page(&page, &label, true, None);
    }

    state.borrow_mut()._OpenTabs.push(OpenTab {
        _Path: path.to_path_buf(),
        _Name: name,
        _IsPts: is_pts,
        _IsObj: is_obj,
        _IsFresco: is_fresco,
        _IsImg: is_img,
    });
}

/// Fallback plain-text viewer for files that aren't `.pts`/`.obj`/fresco documents.
fn build_text_view_panel(parent: &Notebook, path: &Path) -> Panel {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let content = std::fs::read_to_string(path).unwrap_or_else(|e| format!("Failed to read file: {e}"));
    let text = TextCtrl::builder(&panel)
        .with_style(wxdragon::widgets::textctrl::TextCtrlStyle::MultiLine | wxdragon::widgets::textctrl::TextCtrlStyle::ReadOnly)
        .build();
    text.set_value(&content);

    sizer.add(&text, 1, SizerFlag::Expand, 0);
    panel.set_sizer(sizer, true);
    panel
}




