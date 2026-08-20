//-- styew/explorer.rs --------------------------------------------------------------------------------------------------------------
use	std::path::{ Path, PathBuf };
use	egui::{ Ui, RichText, Color32, CollapsingHeader };
use	crate::styew::state::{ AppState, OpenTab };
use	crate::fresco::ExprRepos;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Renders the sidebar file and Fresco explorer tree.
pub fn	RenderExplorer( ui: &mut Ui, state: &mut AppState)
{
    ui.vertical( |ui| {
        ui.horizontal( |ui| {
            ui.label( RichText::new( "EXPLORER").strong().size( 11.0).color( Color32::from_rgb( 148, 163, 184)));
            ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                if ui.small_button( RichText::new( "📁 Open Folder").color( Color32::from_rgb( 0, 243, 255))).clicked() {
                    if let  	Some( folder) = rfd::FileDialog::new().pick_folder() {
                        state._RootPath = folder;
                    }
                }
            });
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink( [false, false])
            .show( ui, |ui| {
                // 1. Filesystem Tree
                CollapsingHeader::new( RichText::new( format!( "📁 Workspace: {}", state._RootPath.display())).strong().size( 12.0))
                    .default_open( true)
                    .show( ui, |ui| {
                        let  	root = state._RootPath.clone();
                        render_dir_entries( ui, &root, state);
                    });

                ui.add_space( 8.0);

                // 2. Fresco Symbolic Math Repository
                CollapsingHeader::new( RichText::new( "ƒ Fresco Symbolic Repos").strong().size( 12.0).color( Color32::from_rgb( 59, 130, 246)))
                    .default_open( true)
                    .show( ui, |ui| {
                        render_fresco_tree( ui, state);
                    });
            });
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	render_dir_entries( ui: &mut Ui, dir: &Path, state: &mut AppState)
{
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
            CollapsingHeader::new( RichText::new( format!( "📁 {}", name)).size( 12.0))
                .show( ui, |ui| {
                    render_dir_entries( ui, &path, state);
                });
        } else {
            let  	isPts = name.to_lowercase().ends_with( ".pts");
            let  	isObj = name.to_lowercase().ends_with( ".obj");

            let  	(icon, color) = if isPts {
                ("•", Color32::from_rgb( 0, 243, 255))
            } else if isObj {
                ("◬", Color32::from_rgb( 168, 85, 247))
            } else {
                ("📄", Color32::from_rgb( 148, 163, 184))
            };

            ui.horizontal( |ui| {
                ui.label( RichText::new( icon).color( color).size( 11.0));
                let  	itemBtn = ui.selectable_label( false, RichText::new( &name).color( Color32::from_rgb( 226, 232, 240)).size( 12.0));
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
        ("fresco://poly/quadratic", "P(x) = 3x² + 2x + 1", "PolyExpr"),
        ("fresco://poly/cubic", "Q(x,y) = x³ - 2xy + y²", "TermTree"),
        ("fresco://surface/saddle", "Z(u,v) = u² - v² (Hyperbolic)", "Surface"),
        ("fresco://series/fourier", "S(t) = ∑ aₙ sin(nωt)", "Series"),
    ];

    for (uri, label, badge) in terms {
        ui.horizontal( |ui| {
            ui.label( RichText::new( "ƒ").color( Color32::from_rgb( 59, 130, 246)).strong());
            let  	btn = ui.selectable_label( false, RichText::new( label).size( 12.0).color( Color32::from_rgb( 226, 232, 240)));
            ui.label( RichText::new( badge).color( Color32::from_rgb( 100, 116, 139)).size( 10.0));

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
    if let  	Some( existing) = state._OpenTabs.iter().find( |t| t._Path == *path) {
        state._ActiveTabId = Some( existing._Id.clone());
        return;
    }

    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( &pathStr).to_string();
    let  	tabId = format!( "tab_{}", state._OpenTabs.len() + 1);

    let  	(content, lineCount, size) = if !is_pts && !is_obj && !is_fresco {
        if let  	Ok( text) = std::fs::read_to_string( path) {
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
        _Id:        tabId.clone(),
        _Path:      path.to_path_buf(),
        _Name:      fileName,
        _Content:   content,
        _LineCount: lineCount,
        _Size:      size,
        _IsPts:     is_pts,
        _IsObj:     is_obj,
        _IsFresco:  is_fresco,
    };

    state._OpenTabs.push( newTab);
    state._ActiveTabId = Some( tabId);
}

// ---------------------------------------------------------------------------------------------------------------------------------
