//-- xplrcmds.rs ------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	tauri::Manager;
use	kosh::fenst::{ XplrEntry, XplrContent, XplrNodeDto, StreamChunkDto, PtsPointsDto, CreateDefaultRegistry };
use	kosh::silo::Buff;
use	serde::Serialize;
use	std::collections::HashMap;
use	std::sync::Mutex;
use	std::sync::LazyLock;


static GCOMP_SPV: &[u8] = include_bytes!( env!( "GCOMP_SPV_PATH"));

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
pub fn	XplrListEntries( path: String) -> Result< Buff< XplrEntry>, String>
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
pub fn	XplrChildren( uri: String) -> Result< Buff< XplrNodeDto>, String>
{
    let  	registry = CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    let  	( scheme, root) = guard.OpenRoot( &uri)?;
    let  	children = root.Children()?;
    let  	mut dtos = Buff::New();

    for child in children {
        dtos.Push( child.ToDto( &scheme));
    }

    Ok( dtos)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns a list of supported Xplr provider schemes.
#[tauri::command]
pub fn	XplrListProviders() -> Result< Buff< String>, String>
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

struct PtsSessionState
{
    _Points:       Buff< [f32; 3]>,
    _BboxMin:     [f32; 3],
    _BboxMax:     [f32; 3],
    _AngleX:      f32,
    _AngleY:      f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

static PTS_STATE: LazyLock< Mutex< HashMap< String, PtsSessionState>>> = LazyLock::new( || {
    Mutex::new( HashMap::new())
});

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
#[serde( rename_all = "snake_case")]
pub struct ProjectedPoint
{
    pub _X:            f32,
    pub _Y:            f32,
    pub _Radius:       f32,
    pub _CoreRadius:  f32,
    pub _Color:        String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
#[serde( rename_all = "snake_case")]
pub struct ProjectedLine
{
    pub _X1:   f32,
    pub _Y1:   f32,
    pub _X2:   f32,
    pub _Y2:   f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
#[serde( rename_all = "snake_case")]
pub struct PtsFrameDto
{
    pub _Points:         Buff< ProjectedPoint>,
    pub _BoxLines:      Buff< ProjectedLine>,
    pub _FileName:      String,
    pub _Count:          usize,
    pub _BboxLabel:     String,
    pub _ShaderStatus:  String,
    pub _OverlayText1:  String,
    pub _OverlayText2:  String,
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

fn	ParseHexColor( hex: &str) -> ( u8, u8, u8)
{
    let  	clean = hex.trim_start_matches( '#');
    if clean.len() == 6 {
        let  	r = u8::from_str_radix( &clean[0..2], 16).unwrap_or( 0);
        let  	g = u8::from_str_radix( &clean[2..4], 16).unwrap_or( 243);
        let  	b = u8::from_str_radix( &clean[4..6], 16).unwrap_or( 255);
        ( r, g, b)
    } else {
        ( 0, 243, 255)
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Transforms and projects 3D point cloud coordinates and its bounding box to 2D screen coordinates, managing rotation state.
#[tauri::command]
pub fn	XplrProjectPts(
    path: String,
    width: f32,
    height: f32,
    dpr: f32,
    speed: f32,
    color: String,
) -> Result< PtsFrameDto, String>
{
    let  	mut guard = PTS_STATE.lock().map_err( |e| e.to_string())?;
    let  	state = guard.entry( path.clone()).or_insert_with( || {
        let  	dto = kosh::fenst::XplrFetchPtsPoints( GCOMP_SPV).unwrap_or_else( |_| PtsPointsDto {
            _Points: Buff::New(),
            _Count: 0,
            _BboxMin: [ 0.0, 0.0, 0.0 ],
            _BboxMax: [ 0.0, 0.0, 0.0 ],
        });

        PtsSessionState {
            _Points: dto._Points,
            _BboxMin: dto._BboxMin,
            _BboxMax: dto._BboxMax,
            _AngleX: 0.4,
            _AngleY: 0.6,
        }
    });

    let  	speedRad = speed / 1000.0;
    state._AngleY += speedRad;
    state._AngleX += speedRad * 0.5;

    let  	angleX = state._AngleX;
    let  	angleY = state._AngleY;
    let  	bboxMin = state._BboxMin;
    let  	bboxMax = state._BboxMax;

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

    let  	mut projectedBox = Buff::New();
    for v in &bboxVerts {
        let  	( px, py, _) = Project3d( v[0], v[1], v[2], angleX, angleY, width, height);
        projectedBox.Push( ( px, py));
    }

    let  	mut box_lines = Buff::New();
    for ( i, j) in &bboxEdges {
        let  	p1 = projectedBox[*i];
        let  	p2 = projectedBox[*j];
        box_lines.Push( ProjectedLine {
            _X1: p1.0,
            _Y1: p1.1,
            _X2: p2.0,
            _Y2: p2.1,
        });
    }

    let  	( r, g, b) = ParseHexColor( &color);
    let  	mut projectedPoints = Buff::New();
    for pt in &state._Points {
        let  	( px, py, pz) = Project3d( pt[0], pt[1], pt[2], angleX, angleY, width, height);
        let  	depthFactor = 0.3f32.max( 1.0f32.min( ( 300.0 - pz) / 400.0));
        let  	radius = ( 3.0 + depthFactor * 4.0) * dpr;
        let  	alpha = 0.5 + depthFactor * 0.5;
        let  	core_radius = ( 1.0 + depthFactor * 1.5) * dpr;

        let  	colorStr = format!( "rgba({}, {}, {}, {:.3})", r, g, b, alpha);

        projectedPoints.Push( ProjectedPoint {
            _X: px,
            _Y: py,
            _Radius: radius,
            _CoreRadius: core_radius,
            _Color: colorStr,
        });
    }

    let  	fileName = std::path::Path::new( &path)
        .file_name()
        .map( |n| n.to_string_lossy().into_owned())
        .unwrap_or_else( || "Block.pts".to_string());

    let  	bboxLabel = format!( "[{}, {}, {}] → [{}, {}, {}]",
        bboxMin[0] as i32, bboxMin[1] as i32, bboxMin[2] as i32,
        bboxMax[0] as i32, bboxMax[1] as i32, bboxMax[2] as i32
    );

    let  	overlay1 = format!( "Points: {} | BBox: {}", state._Points.len(), bboxLabel);
    let  	overlay2 = "Shader Backend: Rust-GPU (gcomp::pts_pointcloud_cs)".to_string();

    Ok( PtsFrameDto {
        _Points: projectedPoints,
        _BoxLines: box_lines,
        _FileName: fileName,
        _Count: state._Points.len(),
        _BboxLabel: bboxLabel,
        _ShaderStatus: "Shader Active: gcomp::pts_pointcloud_cs".to_string(),
        _OverlayText1: overlay1,
        _OverlayText2: overlay2,
    })
}


