//-- xplrcmds.rs ------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	tauri::Manager;
use	kosh::fenst::{ XplrEntry, XplrContent, XplrNodeDto, StreamChunkDto, PtsPointsDto, CreateDefaultRegistry };
use	serde::Serialize;

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

fn	OpenWindowHelper(
    app: &tauri::AppHandle,
    path: &str,
    labelPrefix: &str,
    urlTemplate: &str,
    title: String,
    width: f64,
    height: f64,
) -> Result< (), String>
{
    let  	mut hashVal: u64 = 5381;
    for b in path.bytes() {
        hashVal = hashVal.wrapping_mul( 33).wrapping_add( b as u64);
    }
    let  	label = format!( "{}{:x}", labelPrefix, hashVal);

    if let Some( win) = app.get_webview_window( &label) {
        let  	_ = win.set_focus();
        return Ok( ());
    }

    let  	encodedPath = UrlEncode( path);
    let  	url = format!( "{}{}", urlTemplate, encodedPath);

    let  	builder = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App( url.into())
    )
    .title( title)
    .inner_size( width, height);

    builder.build().map_err( |e| e.to_string())?;

    Ok( ())
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Opens a file content view in a new or existing separate window.
#[tauri::command]
pub fn	XplrOpenContentWindow( app: tauri::AppHandle, path: String) -> Result< (), String>
{
    if kosh::fenst::IsPtsFile( &path) {
        return XplrOpenPtsGraphicsWindow( app, path);
    }

    OpenWindowHelper( &app, &path, "win_", "index.html?file=", format!( "Fenst — {}", path), 900.0, 700.0)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Opens a dedicated rust-gpu graphics shader window displaying a 100x100x100 wireframe block for .pts files.
#[tauri::command]
pub fn	XplrOpenPtsGraphicsWindow( app: tauri::AppHandle, path: String) -> Result< (), String>
{
    if !std::path::Path::new( &path).exists() {
        return Err( "File does not exist".to_string());
    }

    if !kosh::fenst::IsPtsFile( &path) {
        return Err( "Not a .pts file".to_string());
    }

    let  	fileName = std::path::Path::new( &path);

    OpenWindowHelper( &app, &path, "pts_win_", "pts_viewer.html?file=", format!( "Fenst — Point Cloud Viewer — {}", fileName.display()), 960.0, 720.0)
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
pub struct ProjectedPoint
{
    pub x:            f32,
    pub y:            f32,
    pub radius:       f32,
    pub alpha:        f32,
    pub core_radius:  f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
pub struct ProjectedLine
{
    pub x1:   f32,
    pub y1:   f32,
    pub x2:   f32,
    pub y2:   f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
pub struct PtsFrameDto
{
    pub points:       Vec< ProjectedPoint>,
    pub box_lines:    Vec< ProjectedLine>,
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	Project3d(
    x: f32,
    y: f32,
    z: f32,
    angleX: f32,
    angleY: f32,
    width: f32,
    height: f32,
) -> ( f32, f32, f32)
{
    let  	cosY = angleY.cos();
    let  	sinY = angleY.sin();
    let  	x1 = x * cosY + z * sinY;
    let  	z1 = -x * sinY + z * cosY;

    let  	cosX = angleX.cos();
    let  	sinX = angleX.sin();
    let  	y2 = y * cosX - z1 * sinX;
    let  	z2 = y * sinX + z1 * cosX;

    let  	fov = 350.0;
    let  	distance = 250.0;
    let  	scale = fov / ( distance + z2);

    let  	projX = width / 2.0 + x1 * scale;
    let  	projY = height / 2.0 - y2 * scale;

    ( projX, projY, z2)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Transforms and projects 3D point cloud coordinates and its bounding box to 2D screen coordinates.
#[tauri::command]
pub fn	XplrProjectPts(
    points: Vec< [f32; 3]>,
    bboxMin: [f32; 3],
    bboxMax: [f32; 3],
    angleX: f32,
    angleY: f32,
    width: f32,
    height: f32,
    dpr: f32,
) -> Result< PtsFrameDto, String>
{
    let  	bboxVerts = [
        [ bboxMin[0], bboxMin[1], bboxMin[2] ],
        [ bboxMax[0], bboxMin[1], bboxMin[2] ],
        [ bboxMax[0], bboxMax[1], bboxMin[2] ],
        [ bboxMin[0], bboxMax[1], bboxMin[2] ],
        [ bboxMin[0], bboxMin[1], bboxMax[2] ],
        [ bboxMax[0], bboxMin[1], bboxMax[2] ],
        [ bboxMax[0], bboxMax[1], bboxMax[2] ],
        [ bboxMin[0], bboxMax[1], bboxMax[2] ],
    ];

    let  	bboxEdges = [
        ( 0, 1), ( 1, 2), ( 2, 3), ( 3, 0),
        ( 4, 5), ( 5, 6), ( 6, 7), ( 7, 4),
        ( 0, 4), ( 1, 5), ( 2, 6), ( 3, 7),
    ];

    let  	mut projectedBox = Vec::with_capacity( 8);
    for v in &bboxVerts {
        let  	( px, py, _) = Project3d( v[0], v[1], v[2], angleX, angleY, width, height);
        projectedBox.push( ( px, py));
    }

    let  	mut box_lines = Vec::with_capacity( 12);
    for ( i, j) in &bboxEdges {
        let  	p1 = projectedBox[*i];
        let  	p2 = projectedBox[*j];
        box_lines.push( ProjectedLine {
            x1: p1.0,
            y1: p1.1,
            x2: p2.0,
            y2: p2.1,
        });
    }

    let  	mut projectedPoints = Vec::with_capacity( points.len());
    for pt in &points {
        let  	( px, py, pz) = Project3d( pt[0], pt[1], pt[2], angleX, angleY, width, height);
        let  	depthFactor = 0.3f32.max( 1.0f32.min( ( 300.0 - pz) / 400.0));
        let  	radius = ( 3.0 + depthFactor * 4.0) * dpr;
        let  	alpha = 0.5 + depthFactor * 0.5;
        let  	core_radius = ( 1.0 + depthFactor * 1.5) * dpr;

        projectedPoints.push( ProjectedPoint {
            x: px,
            y: py,
            radius,
            alpha,
            core_radius,
        });
    }

    Ok( PtsFrameDto {
        points: projectedPoints,
        box_lines,
    })
}


