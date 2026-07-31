//-- commands.rs ------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	kosh::fenst::{ FileEntry, FileContents, FileInfo };

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a directory and returns sorted entries (directories first, then files).
#[tauri::command]
pub fn	read_directory( path: String) -> Result< Vec< FileEntry>, String>
{
    kosh::fenst::read_directory( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads the text content of a file, with a size guard.
#[tauri::command]
pub fn	read_file_contents( path: String) -> Result< FileContents, String>
{
    kosh::fenst::read_file_contents( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns metadata about a file or directory.
#[tauri::command]
pub fn	get_file_info( path: String) -> Result< FileInfo, String>
{
    kosh::fenst::get_file_info( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Shows a native dialog to pick a folder.
#[tauri::command]
pub fn	select_directory() -> Result< Option< String>, String>
{
    let  	fileDialog = rfd::FileDialog::new()
        .set_title( "Select Folder to Open");

    let  	folderPath = fileDialog.pick_folder();

    Ok( folderPath.map( |p| p.to_string_lossy().into_owned()))
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Fetches children of a given URI using the registered XplrProviders.
#[tauri::command]
pub fn	xplr_children( uri: String) -> Result< Vec< kosh::fenst::XplrNodeDto>, String>
{
    let  	registry = kosh::fenst::CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    let  	( scheme, root) = guard.OpenRoot( &uri)?;
    let  	children = root.Children()?;
    let  	mut dtos = Vec::new();

    for child in children {
        dtos.push( child.ToDto( &scheme));
    }

    Ok( dtos)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns a list of supported Xplr provider schemes.
#[tauri::command]
pub fn	xplr_providers() -> Result< Vec< String>, String>
{
    let  	registry = kosh::fenst::CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    Ok( guard.Schemes())
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a windowed chunk of a file using stream buffering.
#[tauri::command]
pub fn	read_file_chunk( path: String, offset: u64, size: usize) -> Result< kosh::fenst::StreamChunkDto, String>
{
    kosh::fenst::read_file_chunk( path, offset, size)
}

// ---------------------------------------------------------------------------------------------------------------------------------
