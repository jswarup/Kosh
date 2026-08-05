//-- xplrcmds.rs ------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	tauri::Manager;
use	kosh::fenst::{ XplrEntry, XplrContent, XplrNodeDto, StreamChunkDto, PtsPointsDto, CreateDefaultRegistry };

// ---------------------------------------------------------------------------------------------------------------------------------

fn	UrlEncode( input: &str) -> String
{
    let  	mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push( byte as char);
            }
            _ => {
                encoded.push_str( &format!( "%{:02X}", byte));
            }
        }
    }
    encoded
}

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
pub fn	XplrChildren( uri: String) -> Result< Vec< XplrNodeDto>, String>
{
    let  	registry = CreateDefaultRegistry();
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
    let  	registry = CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    Ok( guard.Schemes())
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a windowed chunk of a file using stream buffering.
#[tauri::command]
pub fn	XplrFetchChunk( path: String, offset: u64, size: usize) -> Result< StreamChunkDto, String>
{
    kosh::fenst::XplrFetchChunk( path, offset, size)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Generates 100 pseudo-random 3D points on the GPU using the rust-gpu pts_pointcloud_cs compute shader.
#[tauri::command]
pub fn	XplrFetchPtsPoints() -> Result< PtsPointsDto, String>
{
    static GCOMP_SPV: &[u8] = include_bytes!( env!( "GCOMP_SPV_PATH"));
    kosh::fenst::XplrFetchPtsPoints( GCOMP_SPV)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Opens a file content view in a new or existing separate window.
#[tauri::command]
pub fn	XplrOpenContentWindow( app: tauri::AppHandle, path: String) -> Result< (), String>
{
    if kosh::fenst::IsPtsFile( &path) {
        return XplrOpenPtsGraphicsWindow( app, path);
    }

    let  	fileName = std::path::Path::new( &path)
        .file_name()
        .map( |n| n.to_string_lossy().into_owned())
        .unwrap_or_else( || "Viewer".to_string());

    let  	mut hashVal: u64 = 5381;
    for b in path.bytes() {
        hashVal = hashVal.wrapping_mul( 33).wrapping_add( b as u64);
    }
    let  	label = format!( "win_{:x}", hashVal);

    if let Some( win) = app.get_webview_window( &label) {
        let  	_ = win.set_focus();
        return Ok( ());
    }

    let  	encodedPath = UrlEncode( &path);
    let  	url = format!( "index.html?file={}", encodedPath);

    let  	builder = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::App( url.into())
    )
    .title( format!( "Fenst — {}", fileName))
    .inner_size( 900.0, 700.0);

    builder.build().map_err( |e| e.to_string())?;

    Ok( ())
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Opens a dedicated rust-gpu graphics shader window displaying a 100x100x100 wireframe block for .pts files.
#[tauri::command]
pub fn	XplrOpenPtsGraphicsWindow( app: tauri::AppHandle, path: String) -> Result< (), String>
{
    let  	fileName = std::path::Path::new( &path)
        .file_name()
        .map( |n| n.to_string_lossy().into_owned())
        .unwrap_or_else( || "Block.pts".to_string());

    let  	mut hashVal: u64 = 5381;
    for b in path.bytes() {
        hashVal = hashVal.wrapping_mul( 33).wrapping_add( b as u64);
    }
    let  	label = format!( "pts_win_{:x}", hashVal);

    if let Some( win) = app.get_webview_window( &label) {
        let  	_ = win.set_focus();
        return Ok( ());
    }

    let  	encodedPath = UrlEncode( &path);
    let  	url = format!( "pts_viewer.html?file={}", encodedPath);

    let  	builder = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::App( url.into())
    )
    .title( format!( "Fenst — Point Cloud Viewer — {}", fileName))
    .inner_size( 960.0, 720.0);

    builder.build().map_err( |e| e.to_string())?;

    Ok( ())
}

// ---------------------------------------------------------------------------------------------------------------------------------


