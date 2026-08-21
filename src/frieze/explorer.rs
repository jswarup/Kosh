//-- frieze/explorer.rs -------------------------------------------------------------------------------------------------------------
use	std::path::{ Path, PathBuf };
use	egui::{
    Ui, Color32, RichText, Frame, Margin, Vec2,
    CollapsingHeader, ScrollArea, Stroke, Window, Context,
};
use	crate::frieze::state::{ AppState, OpenTab, ExplorerViewMode };
use	crate::fresco::ExprRepos;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Renders the sidebar file and Fresco explorer tree.
pub fn	RenderExplorer( ui: &mut Ui, state: &mut AppState)
{
    ui.vertical( |ui| {
        // 1. Windows Explorer Address & Breadcrumb Header
        Frame::new()
            .fill( Color32::from_rgb( 30, 30, 46))
            .stroke( Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)))
            .inner_margin( Margin::symmetric( 8, 6))
            .show( ui, |ui| {
                ui.horizontal( |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new( 6.0, 0.0);

                    if ui.button( RichText::new( "⬆").size( 12.0)).on_hover_text( "Up to Parent Directory").clicked() {
                        if let Some( parent) = state._RootPath.parent() {
                            state._RootPath = parent.to_path_buf();
                        }
                    }

                    if ui.button( RichText::new( "📂 Open").size( 11.0)).on_hover_text( "Choose Folder").clicked() {
                        if let Some( folder) = rfd::FileDialog::new().pick_folder() {
                            state._RootPath = folder;
                        }
                    }

                    if ui.button( RichText::new( "🪟 Win").size( 11.0)).on_hover_text( "Open Floating Explorer Window").clicked() {
                        state._IsExplorerWindowOpen = true;
                    }

                    let  	pathDisplay = state._RootPath.to_string_lossy();
                    let  	shortPath = if pathDisplay.len() > 18 {
                        format!( "...{}", &pathDisplay[pathDisplay.len() - 15..])
                    } else {
                        pathDisplay.to_string()
                    };

                    ui.label(
                        RichText::new( format!( "📁 {}", shortPath))
                            .monospace()
                            .size( 11.0)
                            .color( Color32::from_rgb( 137, 180, 250))
                    ).on_hover_text( &*pathDisplay);
                });
            });

        ui.add_space( 4.0);

        // 2. Windows Explorer Quick Access & Tree Navigation
        ScrollArea::vertical()
            .auto_shrink( [false, false])
            .show( ui, |ui| {
                // Quick Access Section
                CollapsingHeader::new( RichText::new( "⭐ Quick Access").strong().size( 12.0).color( Color32::from_rgb( 249, 226, 175)))
                    .default_open( true)
                    .show( ui, |ui| {
                        render_quick_access_item( ui, "📦 workbench (3D Models)", &PathBuf::from( "workbench"), state);
                        render_quick_access_item( ui, "🦀 src (Source Tree)", &PathBuf::from( "src"), state);
                        render_quick_access_item( ui, "📖 wiki (Docs)", &PathBuf::from( "wiki"), state);
                    });

                ui.add_space( 6.0);
                ui.separator();
                ui.add_space( 4.0);

                // This PC / Local Workspace Section
                let  	folderName = state._RootPath.file_name().and_then( |n| n.to_str()).unwrap_or( "Workspace");
                CollapsingHeader::new( RichText::new( format!( "💻 This PC > {}", folderName)).strong().size( 12.0).color( Color32::from_rgb( 205, 214, 244)))
                    .default_open( true)
                    .show( ui, |ui| {
                        let  	root = state._RootPath.clone();
                        render_dir_entries( ui, &root, state, 0);
                    });

                ui.add_space( 6.0);
                ui.separator();
                ui.add_space( 4.0);

                // Fresco Symbolic Repositories Section
                CollapsingHeader::new( RichText::new( "∫ Fresco Symbolic Repos").strong().size( 12.0).color( Color32::from_rgb( 137, 220, 235)))
                    .default_open( true)
                    .show( ui, |ui| {
                        render_fresco_tree( ui, state);
                    });
            });
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Renders the floating, movable Windows File Explorer window over the workspace.
pub fn	RenderFloatingExplorerWindow( ctx: &Context, state: &mut AppState)
{
    if !state._IsExplorerWindowOpen {
        return;
    }

    let  	mut isOpen = state._IsExplorerWindowOpen;

    Window::new( "📁 Windows File Explorer")
        .open( &mut isOpen)
        .default_size( [760.0, 480.0])
        .min_size( [450.0, 300.0])
        .resizable( true)
        .collapsible( true)
        .show( ctx, |ui| {
            render_explorer_body( ui, state);
        });

    state._IsExplorerWindowOpen = isOpen;
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Renders the main Windows File Explorer view tab.
pub fn	RenderExplorerViewTab( ui: &mut Ui, state: &mut AppState)
{
    render_explorer_body( ui, state);
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	render_explorer_body( ui: &mut Ui, state: &mut AppState)
{
    ui.vertical( |ui| {
        // Top Windows Explorer Ribbon / Toolbar
        Frame::new()
            .fill( Color32::from_rgb( 24, 24, 37))
            .stroke( Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)))
            .inner_margin( Margin::symmetric( 10, 6))
            .show( ui, |ui| {
                ui.horizontal( |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new( 6.0, 0.0);

                    // Navigation buttons
                    if ui.button( RichText::new( "⬆ Up").size( 11.0)).clicked() {
                        if let Some( parent) = state._RootPath.parent() {
                            state._RootPath = parent.to_path_buf();
                        }
                    }

                    if ui.button( RichText::new( "📂 Open Folder...").size( 11.0)).clicked() {
                        if let Some( folder) = rfd::FileDialog::new().pick_folder() {
                            state._RootPath = folder;
                        }
                    }

                    // Breadcrumb Address Bar
                    let  	pathStr = state._RootPath.to_string_lossy().to_string();
                    ui.label(
                        RichText::new( format!( "📁 This PC > {}", pathStr))
                            .monospace()
                            .size( 11.5)
                            .color( Color32::from_rgb( 137, 180, 250))
                    );

                    // View Mode Toggles & Search Bar
                    ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new( 4.0, 0.0);

                        ui.selectable_value( &mut state._ExplorerViewMode, ExplorerViewMode::Details, "📋 Details");
                        ui.selectable_value( &mut state._ExplorerViewMode, ExplorerViewMode::Grid, "🗂 Large Icons");

                        ui.separator();
                        ui.add( egui::TextEdit::singleline( &mut state._ExplorerSearch).hint_text( "🔍 Search files...").desired_width( 130.0));
                    });
                });
            });

        ui.add_space( 6.0);

        // File Content List / Grid
        let  	dir = state._RootPath.clone();
        let  	entries = match std::fs::read_dir( &dir) {
            Ok( e)  => e,
            Err( _) => return,
        };

        let  	mut paths: Vec< PathBuf> = entries.filter_map( |e| e.ok().map( |e| e.path())).collect();
        paths.sort_by( |a, b| {
            let  	aIsDir = a.is_dir();
            let  	bIsDir = b.is_dir();
            if aIsDir != bIsDir {
                bIsDir.cmp( &aIsDir)
            } else {
                a.cmp( b)
            }
        });

        // Apply search query filter
        let  	searchQuery = state._ExplorerSearch.to_lowercase();
        let  	filteredPaths: Vec< PathBuf> = paths.into_iter().filter( |p| {
            let  	name = p.file_name().and_then( |n| n.to_str()).unwrap_or( "").to_lowercase();
            if name.starts_with( '.') || name == "target" {
                return false;
            }
            if searchQuery.is_empty() {
                true
            } else {
                name.contains( &searchQuery)
            }
        }).collect();

        ScrollArea::vertical().show( ui, |ui| {
            match state._ExplorerViewMode {
                ExplorerViewMode::Grid => {
                    render_grid_view( ui, &filteredPaths, state);
                }
                ExplorerViewMode::Details => {
                    render_details_view( ui, &filteredPaths, state);
                }
            }
        });
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	render_grid_view( ui: &mut Ui, paths: &[PathBuf], state: &mut AppState)
{
    let  	cardWidth = 110.0;
    let  	cardHeight = 90.0;

    ui.horizontal_wrapped( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 10.0, 10.0);

        for path in paths {
            let  	name = path.file_name().and_then( |n| n.to_str()).unwrap_or( "file").to_string();
            let  	isDir = path.is_dir();
            let  	isPts = name.to_lowercase().ends_with( ".pts");
            let  	isObj = name.to_lowercase().ends_with( ".obj");
            let  	(icon, color, typeName) = if isDir {
                ("📁", Color32::from_rgb( 249, 226, 175), "Folder")
            } else {
                get_file_icon_and_color( &name)
            };

            let  	(rect, response) = ui.allocate_exact_size( Vec2::new( cardWidth, cardHeight), egui::Sense::click());
            let  	painter = ui.painter();

            let  	bgFill = if response.hovered() {
                Color32::from_rgb( 49, 50, 68)
            } else {
                Color32::from_rgb( 30, 30, 46)
            };

            let  	borderStroke = if response.hovered() {
                Stroke::new( 1.0, Color32::from_rgb( 137, 180, 250))
            } else {
                Stroke::new( 1.0, Color32::from_rgb( 40, 40, 60))
            };

            painter.rect( rect, 6.0, bgFill, borderStroke, egui::StrokeKind::Outside);

            // Icon
            painter.text(
                rect.center_top() + Vec2::new( 0.0, 26.0),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional( 24.0),
                color,
            );

            // File / Folder Name (truncated)
            let  	shortName = if name.len() > 14 {
                format!( "{}...", &name[..11])
            } else {
                name.clone()
            };

            painter.text(
                rect.center_bottom() + Vec2::new( 0.0, -20.0),
                egui::Align2::CENTER_CENTER,
                &shortName,
                egui::FontId::proportional( 11.0),
                Color32::from_rgb( 205, 214, 244),
            );

            // Type badge
            painter.text(
                rect.center_bottom() + Vec2::new( 0.0, -8.0),
                egui::Align2::CENTER_CENTER,
                typeName,
                egui::FontId::monospace( 9.0),
                Color32::from_rgb( 108, 112, 134),
            );

            if response.double_clicked() || response.clicked() {
                if isDir {
                    state._RootPath = path.clone();
                } else {
                    open_file_tab( state, path, isPts, isObj, false);
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	render_details_view( ui: &mut Ui, paths: &[PathBuf], state: &mut AppState)
{
    // Table Header
    Frame::new()
        .fill( Color32::from_rgb( 20, 20, 30))
        .inner_margin( Margin::symmetric( 8, 4))
        .show( ui, |ui| {
            ui.horizontal( |ui| {
                ui.label( RichText::new( "Name").strong().size( 11.5).color( Color32::from_rgb( 166, 173, 200)));
                ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                    ui.label( RichText::new( "Size").strong().size( 11.5).color( Color32::from_rgb( 166, 173, 200)));
                    ui.add_space( 30.0);
                    ui.label( RichText::new( "Type").strong().size( 11.5).color( Color32::from_rgb( 166, 173, 200)));
                    ui.add_space( 40.0);
                });
            });
        });

    ui.separator();

    for path in paths {
        let  	name = path.file_name().and_then( |n| n.to_str()).unwrap_or( "file").to_string();
        let  	isDir = path.is_dir();
        let  	isPts = name.to_lowercase().ends_with( ".pts");
        let  	isObj = name.to_lowercase().ends_with( ".obj");
        let  	(icon, color, typeName) = if isDir {
            ("📁", Color32::from_rgb( 249, 226, 175), "File Folder")
        } else {
            get_file_icon_and_color( &name)
        };

        let  	fileSizeStr = if isDir {
            "-".to_string()
        } else if let Ok( meta) = std::fs::metadata( path) {
            let  	sz = meta.len();
            if sz > 1_048_576 {
                format!( "{:.1} MB", sz as f64 / 1_048_576.0)
            } else if sz > 1024 {
                format!( "{:.0} KB", sz as f64 / 1024.0)
            } else {
                format!( "{} B", sz)
            }
        } else {
            "-".to_string()
        };

        ui.horizontal( |ui| {
            ui.spacing_mut().item_spacing = Vec2::new( 6.0, 0.0);
            ui.label( RichText::new( icon).color( color).size( 12.0));
            let  	btn = ui.selectable_label( false, RichText::new( &name).size( 11.5).color( Color32::from_rgb( 205, 214, 244)));

            ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                ui.label( RichText::new( fileSizeStr).size( 11.0).color( Color32::from_rgb( 108, 112, 134)));
                ui.add_space( 30.0);
                ui.label( RichText::new( typeName).size( 11.0).color( Color32::from_rgb( 137, 180, 250)));
            });

            if btn.clicked() {
                if isDir {
                    state._RootPath = path.clone();
                } else {
                    open_file_tab( state, path, isPts, isObj, false);
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	render_quick_access_item( ui: &mut Ui, label: &str, path: &Path, state: &mut AppState)
{
    ui.horizontal( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 6.0, 0.0);
        let  	btn = ui.selectable_label( false, RichText::new( label).size( 11.5).color( Color32::from_rgb( 186, 194, 222)));
        if btn.clicked() {
            if path.exists() {
                state._RootPath = path.to_path_buf();
            }
        }
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	get_file_icon_and_color( name: &str) -> (&'static str, Color32, &'static str)
{
    let  	lower = name.to_lowercase();
    if lower.ends_with( ".pts") {
        ("☁", Color32::from_rgb( 137, 180, 250), "3D Points")
    } else if lower.ends_with( ".obj") {
        ("🧊", Color32::from_rgb( 203, 166, 247), "3D Mesh")
    } else if lower.ends_with( ".rs") {
        ("🦀", Color32::from_rgb( 250, 179, 135), "Rust Source")
    } else if lower.ends_with( ".toml") {
        ("⚙", Color32::from_rgb( 186, 194, 222), "Config")
    } else if lower.ends_with( ".md") {
        ("📄", Color32::from_rgb( 137, 220, 235), "Markdown")
    } else if lower.ends_with( ".json") {
        ("{}", Color32::from_rgb( 249, 226, 175), "JSON")
    } else if lower.ends_with( ".html") || lower.ends_with( ".css") || lower.ends_with( ".js") {
        ("🌐", Color32::from_rgb( 180, 190, 254), "Web")
    } else {
        ("📄", Color32::from_rgb( 147, 153, 178), "File")
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	render_dir_entries( ui: &mut Ui, dir: &Path, state: &mut AppState, depth: usize)
{
    if depth > 8 {
        return;
    }

    let  	entries = match std::fs::read_dir( dir) {
        Ok( e)  => e,
        Err( _) => return,
    };

    let  	mut paths: Vec< PathBuf> = entries.filter_map( |e| e.ok().map( |e| e.path())).collect();
    paths.sort_by( |a, b| {
        let  	aIsDir = a.is_dir();
        let  	bIsDir = b.is_dir();
        if aIsDir != bIsDir {
            bIsDir.cmp( &aIsDir)
        } else {
            a.cmp( b)
        }
    });

    for path in paths {
        let  	name = path.file_name().and_then( |n| n.to_str()).unwrap_or( "unnamed").to_string();
        if name.starts_with( '.') || name == "target" {
            continue;
        }

        if path.is_dir() {
            let  	headerText = format!( "📁 {}", name);
            CollapsingHeader::new( RichText::new( headerText).size( 11.5).color( Color32::from_rgb( 205, 214, 244)))
                .show( ui, |ui| {
                    render_dir_entries( ui, &path, state, depth + 1);
                });
        } else {
            let  	isPts = name.to_lowercase().ends_with( ".pts");
            let  	isObj = name.to_lowercase().ends_with( ".obj");
            let  	(icon, color, _typeBadge) = get_file_icon_and_color( &name);

            let  	fileSizeStr = if let Ok( meta) = std::fs::metadata( &path) {
                let  	sz = meta.len();
                if sz > 1_048_576 {
                    format!( "{:.1} MB", sz as f64 / 1_048_576.0)
                } else if sz > 1024 {
                    format!( "{:.0} KB", sz as f64 / 1024.0)
                } else {
                    format!( "{} B", sz)
                }
            } else {
                "-".to_string()
            };

            let  	isActive = state._ActiveTabId.as_ref().and_then( |activeId| {
                state._OpenTabs.iter().find( |t| &t._Id == activeId).map( |t| t._Path == path)
            }).unwrap_or( false);

            ui.horizontal( |ui| {
                ui.spacing_mut().item_spacing = Vec2::new( 4.0, 0.0);
                ui.label( RichText::new( icon).color( color).size( 11.0));

                let  	labelColor = if isActive { Color32::from_rgb( 137, 180, 250) } else { Color32::from_rgb( 205, 214, 244) };
                let  	itemBtn = ui.selectable_label( isActive, RichText::new( &name).color( labelColor).size( 11.5));

                ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                    ui.label( RichText::new( fileSizeStr).color( Color32::from_rgb( 108, 112, 134)).size( 10.0));
                });

                if itemBtn.clicked() {
                    open_file_tab( state, &path, isPts, isObj, false);
                }
            });
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	render_fresco_tree( ui: &mut Ui, state: &mut AppState)
{
    let  	_repos = ExprRepos::New();
    let  	terms = [
        ("fresco://poly/quadratic", "P(x) = 3x² + 2x + 1", "Polynomial"),
        ("fresco://poly/cubic", "Q(x,y) = x³ - 2xy + y²", "TermTree"),
        ("fresco://surface/saddle", "Z(u,v) = u² - v² (Hyperbolic)", "Surface"),
        ("fresco://series/fourier", "S(t) = ∑ aₙ sin(nωt)", "Fourier Series"),
    ];

    for (uri, label, badge) in terms {
        ui.horizontal( |ui| {
            ui.spacing_mut().item_spacing = Vec2::new( 5.0, 0.0);
            ui.label( RichText::new( "∫").color( Color32::from_rgb( 137, 220, 235)).strong().size( 12.0));
            let  	btn = ui.selectable_label( false, RichText::new( label).size( 11.5).color( Color32::from_rgb( 205, 214, 244)));
            ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                ui.label( RichText::new( badge).color( Color32::from_rgb( 108, 112, 134)).size( 10.0));
            });

            if btn.clicked() {
                open_file_tab( state, &PathBuf::from( uri), false, false, true);
            }
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	open_file_tab( state: &mut AppState, path: &Path, is_pts: bool, is_obj: bool, is_fresco: bool)
{
    let  	pathStr = path.to_string_lossy().to_string();
    if let Some( existing) = state._OpenTabs.iter().find( |t| t._Path == *path) {
        state._ActiveTabId = Some( existing._Id.clone());
        return;
    }

    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( &pathStr).to_string();
    let  	tabId = format!( "tab_{}", state._OpenTabs.len() + 1);

    let  	(content, lineCount, size) = if !is_pts && !is_obj && !is_fresco {
        if let Ok( text) = std::fs::read_to_string( path) {
            let  	lines = text.lines().count();
            let  	sz = text.len() as u64;
            (text, lines, sz)
        } else {
            (String::new(), 0, 0)
        }
    } else {
        (String::new(), 0, 0)
    };

    let  	newTab = OpenTab {
        _Id:         tabId.clone(),
        _Path:       path.to_path_buf(),
        _Name:       fileName,
        _Content:    content,
        _LineCount:  lineCount,
        _Size:       size,
        _IsPts:      is_pts,
        _IsObj:      is_obj,
        _IsFresco:   is_fresco,
        _IsExplorer: false,
    };

    state._OpenTabs.push( newTab);
    state._ActiveTabId = Some( tabId);
}

// ---------------------------------------------------------------------------------------------------------------------------------
