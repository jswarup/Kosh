//-- lib.rs -----------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
pub mod xplrcmds;

use	tauri::menu::{ Menu, MenuItem, PredefinedMenuItem, Submenu };
use	tauri::Emitter;

// ---------------------------------------------------------------------------------------------------------------------------------

fn	build_menu( app: &tauri::App) -> Result< Menu< tauri::Wry>, tauri::Error>
{
    // File menu
    let  	openFolder = MenuItem::with_id( app, "open_folder", "Open Folder...", true, Some( "CmdOrCtrl+O"))?;
    let  	closeFolder = MenuItem::with_id( app, "close_folder", "Close Folder", true, None::< &str>)?;
    let  	separator1 = PredefinedMenuItem::separator( app)?;
    let  	quit = MenuItem::with_id( app, "quit", "Quit", true, Some( "CmdOrCtrl+Q"))?;

    let  	fileMenu = Submenu::with_items( app, "File", true, &[
        &openFolder,
        &closeFolder,
        &separator1,
        &quit,
    ])?;

    // Edit menu
    let  	cut = PredefinedMenuItem::cut( app, Some( "Cut"))?;
    let  	copy = PredefinedMenuItem::copy( app, Some( "Copy"))?;
    let  	paste = PredefinedMenuItem::paste( app, Some( "Paste"))?;
    let  	selectAll = PredefinedMenuItem::select_all( app, Some( "Select All"))?;

    let  	editMenu = Submenu::with_items( app, "Edit", true, &[
        &cut,
        &copy,
        &paste,
        &selectAll,
    ])?;

    // View menu
    let  	toggleExplorer = MenuItem::with_id( app, "toggle_explorer", "Toggle Explorer", true, Some( "CmdOrCtrl+B"))?;
    let  	toggleToolbar = MenuItem::with_id( app, "toggle_toolbar", "Toggle Toolbar", true, None::< &str>)?;
    let  	toggleWordWrap = MenuItem::with_id( app, "toggle_word_wrap", "Toggle Word Wrap", true, Some( "Alt+Z"))?;
    let  	toggleTheme = MenuItem::with_id( app, "toggle_theme", "Toggle Theme", true, Some( "CmdOrCtrl+T"))?;

    let  	viewMenu = Submenu::with_items( app, "View", true, &[
        &toggleExplorer,
        &toggleToolbar,
        &toggleWordWrap,
        &toggleTheme,
    ])?;

    // Help menu
    let  	about = MenuItem::with_id( app, "about", "About Fenst", true, None::< &str>)?;

    let  	helpMenu = Submenu::with_items( app, "Help", true, &[
        &about,
    ])?;

    // Build the complete menu bar
    let  	menu = Menu::with_items( app, &[
        &fileMenu,
        &editMenu,
        &viewMenu,
        &helpMenu,
    ])?;

    Ok( menu)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Run the Tauri application.
pub fn	run()
{
    tauri::Builder::default()
        .setup( |app| {
            let  	menu = build_menu( app)?;
            app.set_menu( menu)?;
            Ok( ())
        })
        .on_menu_event( |app, event| {
            let  	menuId = event.id().as_ref();
            match menuId {
                "quit" => {
                    app.exit( 0);
                }
                "open_folder" | "close_folder" | "toggle_explorer" | "toggle_toolbar" | "about" | "toggle_word_wrap" | "toggle_theme" => {
                    let  	_ = app.emit( "menu-event", menuId);
                }
                _ => {}
            }
        })
        .invoke_handler( tauri::generate_handler![
            xplrcmds::XplrListEntries,
            xplrcmds::XplrFetchContent,
            xplrcmds::XplrLeafInfo,
            xplrcmds::XplrSelectBranch,
            xplrcmds::XplrChildren,
            xplrcmds::XplrListProviders,
            xplrcmds::XplrFetchChunk,
        ])
        .run( tauri::generate_context!())
        .expect( "error while running Fenst application");
}

// ---------------------------------------------------------------------------------------------------------------------------------
