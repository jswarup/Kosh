//-- xplrcmds.rs ------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	kosh::fenst::{ XplrEntry, XplrContent };

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a directory and returns sorted entries (directories first, then files).
#[tauri::command]
pub fn	XplrListEntries( path: String) -> Result< Vec< XplrEntry>, String>
{
    kosh::fenst::XplrListEntries( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads the text content of a file, with a size guard.
#[tauri::command]
pub fn	XplrFetchContent( path: String) -> Result< XplrContent, String>
{
    kosh::fenst::XplrFetchContent( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns metadata about a file or directory.
#[tauri::command]
pub fn	XplrLeafInfo( path: String) -> Result< kosh::fenst::XplrLeafInfo, String>
{
    kosh::fenst::XplrLeafInfo( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Shows a native dialog to pick a folder.
#[tauri::command]
pub fn	XplrSelectBranch() -> Result< Option< String>, String>
{
    let  	fileDialog = rfd::FileDialog::new()
        .set_title( "Select Folder to Open");

    let  	folderPath = fileDialog.pick_folder();

    Ok( folderPath.map( |p| p.to_string_lossy().into_owned()))
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Fetches children of a given URI using the registered XplrProviders.
#[tauri::command]
pub fn	XplrChildren( uri: String) -> Result< Vec< kosh::fenst::XplrNodeDto>, String>
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
pub fn	XplrListProviders() -> Result< Vec< String>, String>
{
    let  	registry = kosh::fenst::CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    Ok( guard.Schemes())
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a windowed chunk of a file using stream buffering.
#[tauri::command]
pub fn	XplrFetchChunk( path: String, offset: u64, size: usize) -> Result< kosh::fenst::StreamChunkDto, String>
{
    kosh::fenst::XplrFetchChunk( path, offset, size)
}

// ---------------------------------------------------------------------------------------------------------------------------------
