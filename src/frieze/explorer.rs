//-- frieze/explorer.rs -------------------------------------------------------------------------------------------------------------
//! Sidebar/tab file explorer: a toolbar (Up / Open Folder) plus a native directory tree.
use std::path::PathBuf;
use std::rc::Rc;

use wxdragon::prelude::*;
use wxdragon::widgets::treectrl::{TreeCtrl, TreeCtrlStyle, TreeItemId};
use wxdragon::HasItemData;

use crate::frieze::state::SharedState;

pub fn build_explorer_panel(
    parent: &impl WxWidget,
    state: SharedState,
    on_open_file: Rc<dyn Fn(PathBuf)>,
) -> Panel {
    let panel = Panel::builder(parent).build();
    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    let toolbar = Panel::builder(&panel).build();
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let path_label = StaticText::builder(&toolbar).with_label("").build();
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
        tree.on_selection_changed(move |evt| {
            let Some(item_id) = evt.get_item() else { return };
            let Some(data) = tree.get_custom_data(&item_id) else { return };
            let Some(path) = data.downcast_ref::<PathBuf>() else { return };
            if path.is_dir() {
                if tree.is_expanded(&item_id) {
                    tree.collapse(&item_id);
                } else {
                    tree.expand(&item_id);
                }
            } else {
                on_open_file(path.clone());
            }
        });
    }

    panel
}

fn refresh_path_label(label: &StaticText, state: &SharedState) {
    let path = state.borrow()._RootPath.to_string_lossy().to_string();
    label.set_label(&format!("Workspace: {path}"));
}

fn populate_node(tree: &TreeCtrl, parent_item: &TreeItemId, current_path: &PathBuf) {
    let Ok(entries) = std::fs::read_dir(current_path) else { return; };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort_by(|a, b| {
        let a_dir = a.is_dir();
        let b_dir = b.is_dir();
        if a_dir != b_dir { b_dir.cmp(&a_dir) } else { a.cmp(b) }
    });

    for path in paths {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
        if name.starts_with('.') || name == "target" || name == "out" {
            continue;
        }
        let is_dir = path.is_dir();
        if let Some(item) = tree.append_item_with_data(parent_item, &name, path.clone(), None, None) {
            if is_dir {
                populate_node(tree, &item, &path);
            }
        }
    }
}

fn populate_tree(tree: &TreeCtrl, state: &SharedState) {
    tree.delete_all_items();
    let root_path = state.borrow()._RootPath.clone();
    let root_name = root_path.file_name().and_then(|n| n.to_str()).unwrap_or("Workspace").to_string();

    if let Some(root_item) = tree.add_root_with_data(&root_name, root_path.clone(), None, None) {
        populate_node(tree, &root_item, &root_path);
        tree.expand(&root_item);
    }
}