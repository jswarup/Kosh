//-- frieze/desktop.rs --------------------------------------------------------------------------------------------------------------
//! Top-level application menu bar (File / Settings) for the wxDragon workspace.
use	wxdragon::id::{ ID_EXIT, ID_HIGHEST };
use	wxdragon::menus::menuitem::ItemKind;
use	wxdragon::menus::{ Menu, MenuBar };

pub const ID_OPEN: i32 = 5000;
pub const ID_CLOSE: i32 = 5001;

pub const ID_THEME_DARK: i32 = ID_HIGHEST + 10;
pub const ID_THEME_LIGHT: i32 = ID_HIGHEST + 11;
pub const ID_THEME_CYBERPUNK: i32 = ID_HIGHEST + 12;
pub const ID_THEME_NORD: i32 = ID_HIGHEST + 13;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Builds the top-level File / Settings menu bar.
pub fn	build_menu_bar() -> MenuBar
{
    let  	file_menu = Menu::builder().build();
    file_menu.append( ID_OPEN, "Open Folder...\tCtrl+O", "Open a folder in the explorer", ItemKind::Normal);
    file_menu.append( ID_CLOSE, "Close Tab\tCtrl+W", "Close the active document tab", ItemKind::Normal);
    file_menu.append_separator();
    file_menu.append( ID_EXIT, "Exit\tAlt+F4", "Exit the application", ItemKind::Normal);

    let  	theme_menu = Menu::builder().build();
    theme_menu.append( ID_THEME_DARK, "Dark", "", ItemKind::Radio);
    theme_menu.append( ID_THEME_LIGHT, "Light", "", ItemKind::Radio);
    theme_menu.append( ID_THEME_CYBERPUNK, "Cyberpunk", "", ItemKind::Radio);
    theme_menu.append( ID_THEME_NORD, "Nord", "", ItemKind::Radio);

    let  	settings_menu = Menu::builder().build();
    settings_menu.append_submenu( theme_menu, "Theme", "Choose the workspace color theme");

    MenuBar::builder()
        .append( file_menu, "File")
        .append( settings_menu, "Settings")
        .build()
}

// ---------------------------------------------------------------------------------------------------------------------------------
