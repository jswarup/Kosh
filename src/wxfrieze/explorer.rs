//-- wxfrieze/explorer.rs -------------------------------------------------------------------------------------------------------------
//! Sidebar/tab file explorer: a toolbar (Up / Open Folder) plus a native directory tree.
use std::path::PathBuf;
use std::rc::Rc;

use wxdragon::prelude::*;
use wxdragon::widgets::treectrl::{TreeCtrl, TreeCtrlStyle};
use wxdragon::HasItemData;

use crate::wxfrieze::state::SharedState;

/// Builds the file/directory explorer panel. `on_open_file` is invoked with the path
/// whenever the user activates (double-clicks / presses Enter on) a non-directory item.
pub fn build_explorer_panel(
    parent: &Notebook,
    state: SharedState,
    on_open_file: Rc<dyn Fn(PathBuf)>,
) -> Panel {
    let panel = Panel::builder(parent).build();
    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    let toolbar = Panel::builder(&panel).build();
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let up_btn = Button::builder(&toolbar).with_label("Up").build();
    let open_btn = Button::builder(&toolbar).with_label("Open Folder...").build();
    let path_label = StaticText::builder(&toolbar).with_label("").build();
    toolbar_sizer.add(&up_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    toolbar_sizer.add(&open_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    toolbar_sizer.add(&path_label, 1, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    toolbar.set_sizer(toolbar_sizer, true);

    let tree = TreeCtrl::builder(&panel)
        .with_style(TreeCtrlStyle::HasButtons | TreeCtrlStyle::LinesAtRoot)
        .build();

    root_sizer.add(&toolbar, 0, SizerFlag::Expand, 0);
    root_sizer.add(&tree, 1, SizerFlag::Expand, 0);
    panel.set_sizer(root_sizer, true);

    refresh_path_label(&path_label, &state);
    populate_tree(&tree, &state);

    {
        let state = state.clone();
        let path_label = path_label;
        up_btn.on_click(move |_| {
            let parent_path = state.borrow()._RootPath.parent().map(|p| p.to_path_buf());
            if let Some(p) = parent_path {
                state.borrow_mut()._RootPath = p;
                refresh_path_label(&path_label, &state);
                populate_tree(&tree, &state);
            }
        });
    }

    {
        let state = state.clone();
        open_btn.on_click(move |_| {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                state.borrow_mut()._RootPath = folder;
                refresh_path_label(&path_label, &state);
                populate_tree(&tree, &state);
            }
        });
    }

    {
        let state = state.clone();
        tree.on_item_activated(move |evt| {
            let Some(item_id) = evt.get_item() else { return };
            let Some(data) = tree.get_custom_data(&item_id) else { return };
            let Some(path) = data.downcast_ref::<PathBuf>() else { return };
            if path.is_dir() {
                state.borrow_mut()._RootPath = path.clone();
                refresh_path_label(&path_label, &state);
                populate_tree(&tree, &state);
            } else {
                on_open_file(path.clone());
            }
        });
    }

    panel
}

fn refresh_path_label(label: &StaticText, state: &SharedState) {
    let path = state.borrow()._RootPath.to_string_lossy().to_string();
    label.set_label(&format!("This PC > {path}"));
}

fn populate_tree(tree: &TreeCtrl, state: &SharedState) {
    tree.delete_all_items();
    let root_path = state.borrow()._RootPath.clone();
    let root_name = root_path.file_name().and_then(|n| n.to_str()).unwrap_or("Workspace").to_string();

    let Some(root_item) = tree.add_root_with_data(&root_name, root_path.clone(), None, None) else {
        return;
    };

    let Ok(entries) = std::fs::read_dir(&root_path) else {
        return;
    };

    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort_by(|a, b| {
        let a_dir = a.is_dir();
        let b_dir = b.is_dir();
        if a_dir != b_dir { b_dir.cmp(&a_dir) } else { a.cmp(b) }
    });

    for path in paths {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        let is_dir = path.is_dir();
        if let Some(item) = tree.append_item_with_data(&root_item, &name, path, None, None) {
            if is_dir {
                tree.set_item_has_children(&item, true);
            }
        }
    }

    tree.expand(&root_item);
}
