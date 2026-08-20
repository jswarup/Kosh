//-- frieze/explorer.rs --------------------------------------------------------------------------------------------------------------
use	std::path::{ Path, PathBuf };
use	egui::{ Ui, RichText, Color32, CollapsingHeader, Vec2 };
use	crate::frieze::state::{ AppState, OpenTab };
use	crate::fresco::ExprRepos;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Renders the sidebar file and Fresco explorer tree matching Aura's layout.
pub fn	RenderExplorer( ui: &mut Ui, state: &mut AppState)
{
    ui.vertical( |ui| {
        // Section Header
        ui.horizontal( |ui| {
            ui.label( RichText::new( "EXPLORER").strong().size( 11.0).color( Color32::from_rgb( 166, 173, 200)));
            ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                if ui.small_button( RichText::new( "📁 Open Folder").color( Color32::from_rgb( 137, 180, 250)).size( 11.0)).clicked() {
                    if let  	Some( folder) = rfd::FileDialog::new().pick_folder() {
                        state._RootPath = folder;
                    }
                }
            });
        });

        ui.add_space( 4.0);
        ui.separator();
        ui.add_space( 4.0);

        egui::ScrollArea::vertical()
            .auto_shrink( [false, false])
            .show( ui, |ui| {
                // 1. Filesystem Tree
                let  	folderLabel = state._RootPath.file_name().and_then( |n| n.to_str()).unwrap_or( "Workspace");
                CollapsingHeader::new( RichText::new( format!( "📁 {}", folderLabel)).strong().size( 12.5).color( Color32::from_rgb( 205, 214, 244)))
                    .default_open( true)
                    .show( ui, |ui| {
                        let  	root = state._RootPath.clone();
                        render_dir_entries( ui, &root, state);
                    });

                ui.add_space( 10.0);

                // 2. Fresco Symbolic Math Repository
                CollapsingHeader::new( RichText::new( "ƒ Fresco Symbolic Repos").strong().size( 12.5).color( Color32::from_rgb( 137, 220, 235)))
                    .default_open( true)
                    .show( ui, |ui| {
                        render_fresco_tree( ui, state);
                    });
            });
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	get_file_icon_and_color( name: &str) -> (&'static str, Color32, &'static str)
{
    let  	lower = name.to_lowercase();
    if lower.ends_with( ".pts") {
        ("◈", Color32::from_rgb( 137, 180, 250), "PTS")
    } else if lower.ends_with( ".obj") {
        ("◬", Color32::from_rgb( 203, 166, 247), "OBJ")
    } else if lower.ends_with( ".rs") {
        ("🦀", Color32::from_rgb( 250, 179, 135), "RS")
    } else if lower.ends_with( ".toml") {
        ("⚙", Color32::from_rgb( 186, 194, 222), "TOML")
    } else if lower.ends_with( ".md") {
        ("📝", Color32::from_rgb( 137, 220, 235), "MD")
    } else if lower.ends_with( ".json") {
        ("{}", Color32::from_rgb( 249, 226, 175), "JSON")
    } else if lower.ends_with( ".html") || lower.ends_with( ".css") || lower.ends_with( ".js") {
        ("🌐", Color32::from_rgb( 180, 190, 254), "WEB")
    } else {
        ("📄", Color32::from_rgb( 108, 112, 134), "FILE")
    }
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
            CollapsingHeader::new( RichText::new( format!( "📁 {}", name)).size( 12.0).color( Color32::from_rgb( 205, 214, 244)))
                .show( ui, |ui| {
                    render_dir_entries( ui, &path, state);
                });
        } else {
            let  	isPts = name.to_lowercase().ends_with( ".pts");
            let  	isObj = name.to_lowercase().ends_with( ".obj");
            let  	(icon, color, badge) = get_file_icon_and_color( &name);

            ui.horizontal( |ui| {
                ui.spacing_mut().item_spacing = Vec2::new( 5.0, 0.0);
                ui.label( RichText::new( icon).color( color).size( 11.5));
                let  	itemBtn = ui.selectable_label( false, RichText::new( &name).color( Color32::from_rgb( 205, 214, 244)).size( 12.0));
                ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                    ui.label( RichText::new( badge).color( Color32::from_rgb( 108, 112, 134)).size( 10.0));
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
        ("fresco://poly/quadratic", "P(x) = 3x² + 2x + 1", "PolyExpr"),
        ("fresco://poly/cubic", "Q(x,y) = x³ - 2xy + y²", "TermTree"),
        ("fresco://surface/saddle", "Z(u,v) = u² - v² (Hyperbolic)", "Surface"),
        ("fresco://series/fourier", "S(t) = ∑ aₙ sin(nωt)", "Series"),
    ];

    for (uri, label, badge) in terms {
        ui.horizontal( |ui| {
            ui.spacing_mut().item_spacing = Vec2::new( 5.0, 0.0);
            ui.label( RichText::new( "ƒ").color( Color32::from_rgb( 137, 220, 235)).strong().size( 12.0));
            let  	btn = ui.selectable_label( false, RichText::new( label).size( 12.0).color( Color32::from_rgb( 205, 214, 244)));
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