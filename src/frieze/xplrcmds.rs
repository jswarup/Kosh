//-- xplrcmds.rs ------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	tauri::Manager;
use	crate::fenst::{ XplrEntry, XplrContent, XplrNodeDto, StreamChunkDto, PtsPointsDto, WaveObjMeshDto, CreateDefaultRegistry, Camera, SceneGraph };
use	crate::silo::Buff;
use	crate::swarm::SwarmCluster;
use	serde::Serialize;
use	std::collections::HashMap;
use	std::sync::Mutex;
use	std::sync::LazyLock;


static SYMPH_SPV: &[u8] = include_bytes!( env!( "SYMPH_SPV_PATH"));

static SWARM_CLUSTER: LazyLock< SwarmCluster> = LazyLock::new( || {
    SwarmCluster::Auto()
});

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
    crate::fenst::XplrListEntries( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads the text content of a file, with a size guard.
#[tauri::command]
pub fn	XplrFetchContent( path: String) -> Result< XplrContent, String>
{
    crate::fenst::XplrFetchContent( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns metadata about a file or directory.
#[tauri::command]
pub fn	XplrLeafInfo( path: String) -> Result< crate::fenst::XplrLeafInfo, String>
{
    crate::fenst::XplrLeafInfo( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Shows a native dialog to pick a folder.
#[tauri::command]
pub async fn	XplrSelectBranch() -> Result< Option< String>, String>
{
    let  	fileDialog = rfd::AsyncFileDialog::new()
        .set_title( "Select Folder to Open");

    let  	folderHandle = fileDialog.pick_folder().await;

    Ok( folderHandle.map( |h| h.path().to_string_lossy().into_owned()))
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

/// Returns the list of registered provider scheme prefixes.
#[tauri::command]
pub fn	XplrListProviders() -> Result< Buff< String>, String>
{
    let  	registry = CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    let  	schemes = guard.Schemes();
    let  	mut buff = Buff::New();
    for s in schemes {
        buff.Push( s);
    }
    Ok( buff)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a windowed chunk of a file using flux::BuffStream and silo::Buff.
#[tauri::command]
pub fn	XplrFetchChunk( path: String, offset: u64, size: usize) -> Result< StreamChunkDto, String>
{
    crate::fenst::XplrFetchChunk( path, offset, size)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Generates 3D points from a .pts file (using fleck::ParsePtsStream) or from GPU compute shader if path is empty/omitted.
#[tauri::command]
pub fn	XplrFetchPtsPoints( path: Option< String>) -> Result< PtsPointsDto, String>
{
    if let Some( ref filePath) = path {
        if !filePath.is_empty() && std::path::Path::new( filePath).exists() {
            return crate::fenst::XplrParsePtsFile( filePath);
        }
    }
    crate::fenst::XplrFetchPtsPoints( SYMPH_SPV)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Parses and returns a Wavefront .obj 3D mesh model from the specified file path.
#[tauri::command]
pub fn	XplrFetchWaveObj( path: Option< String>) -> Result< WaveObjMeshDto, String>
{
    if let Some( ref filePath) = path {
        if !filePath.is_empty() && std::path::Path::new( filePath).exists() {
            return crate::fenst::XplrParseWaveObjFile( filePath);
        }
    }
    let  	defaultObj = "workbench/blub/blub_control_mesh.obj";
    if std::path::Path::new( defaultObj).exists() {
        return crate::fenst::XplrParseWaveObjFile( defaultObj);
    }
    Err( "No valid .obj file path provided".to_string())
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
    if crate::fenst::IsPtsFile( &path) {
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

    if !crate::fenst::IsPtsFile( &path) {
        return Err( "Not a .pts file".to_string());
    }

    let  	fileName = std::path::Path::new( &path);

    OpenWindowHelper( &app, &path, "pts_win_", "pts_viewer.html?file=", format!( "Fenst — Point Cloud Viewer — {}", fileName.display()), 960.0, 720.0)
}

// ---------------------------------------------------------------------------------------------------------------------------------

struct PtsSessionState
{
    _Scene: SceneGraph,
}

// ---------------------------------------------------------------------------------------------------------------------------------

static PTS_STATE: LazyLock< Mutex< HashMap< String, PtsSessionState>>> = LazyLock::new( || {
    Mutex::new( HashMap::new())
});

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug, Clone, Copy)]
pub struct ProjectedPoint
{
    #[serde( rename = "x")]
    pub _X:            f32,
    #[serde( rename = "y")]
    pub _Y:            f32,
    #[serde( rename = "radius")]
    pub _Radius:       f32,
    #[serde( rename = "core_radius")]
    pub _CoreRadius:  f32,
    #[serde( rename = "alpha")]
    pub _Alpha:        f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
pub struct ProjectedLine
{
    #[serde( rename = "x1")]
    pub _X1:   f32,
    #[serde( rename = "y1")]
    pub _Y1:   f32,
    #[serde( rename = "x2")]
    pub _X2:   f32,
    #[serde( rename = "y2")]
    pub _Y2:   f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Debug)]
pub struct PtsFrameDto
{
    #[serde( rename = "points")]
    pub _Points:         Buff< ProjectedPoint>,
    #[serde( rename = "box_lines")]
    pub _BoxLines:      Buff< ProjectedLine>,
    #[serde( rename = "file_name")]
    pub _FileName:      String,
    #[serde( rename = "count")]
    pub _Count:          usize,
    #[serde( rename = "bbox_label")]
    pub _BboxLabel:     String,
    #[serde( rename = "shader_status")]
    pub _ShaderStatus:  String,
    #[serde( rename = "overlay_text1")]
    pub _OverlayText1:  String,
    #[serde( rename = "overlay_text2")]
    pub _OverlayText2:  String,
}

/// Transforms and projects 3D point cloud coordinates and its bounding box to 2D screen coordinates,
/// applying camera pan, zoom, and rotation state.
#[tauri::command]
pub fn	XplrProjectPts(
    path: String,
    width: f32,
    height: f32,
    dpr: f32,
    speed: f32,
    _color: String,
    pan_x: Option< f32>,
    pan_y: Option< f32>,
    zoom: Option< f32>,
    rot_x: Option< f32>,
    rot_y: Option< f32>,
    is_interactive: Option< bool>,
) -> Result< PtsFrameDto, String>
{
    let  	mut guard = PTS_STATE.lock().map_err( |e| e.to_string())?;
    let  	state = guard.entry( path.clone()).or_insert_with( || {
        let  	dto = if std::path::Path::new( &path).exists() {
            crate::fenst::XplrParsePtsFile( &path)
                .or_else( |_| crate::fenst::XplrFetchPtsPoints( SYMPH_SPV))
                .unwrap_or_else( |_| PtsPointsDto {
                    _Points: Buff::New(),
                    _Count: 0,
                    _BboxMin: [ 0.0, 0.0, 0.0 ],
                    _BboxMax: [ 0.0, 0.0, 0.0 ],
                })
        } else {
            crate::fenst::XplrFetchPtsPoints( SYMPH_SPV).unwrap_or_else( |_| PtsPointsDto {
                _Points: Buff::New(),
                _Count: 0,
                _BboxMin: [ 0.0, 0.0, 0.0 ],
                _BboxMax: [ 0.0, 0.0, 0.0 ],
            })
        };

        PtsSessionState {
            _Scene: SceneGraph::WithPoints( dto._Points, dto._BboxMin, dto._BboxMax),
        }
    });

    let  	cam = state._Scene.CameraMut();
    if let Some( px) = pan_x {
        cam._PanX = px;
    }
    if let Some( py) = pan_y {
        cam._PanY = py;
    }
    if let Some( z) = zoom {
        cam.SetZoom( z);
    }
    if let Some( rx) = rot_x {
        cam._RotX = rx;
    }
    if let Some( ry) = rot_y {
        cam._RotY = ry;
    }

    if is_interactive != Some( true) && speed > 0.0 {
        let  	speedRad = speed / 1000.0;
        cam._RotY += speedRad;
        cam._RotX += speedRad * 0.5;
    }

    let  	mut box_lines = Buff::New();
    let  	mut projectedPoints = Buff::New();
    let  	camRef = state._Scene.Camera();

    let  	sceneResult = state._Scene.ProjectSceneCluster(
        &SWARM_CLUSTER,
        width,
        height,
        dpr,
        None,
    );

    if let Ok( sceneFrame) = sceneResult {
        for line in &sceneFrame._BoxLines {
            box_lines.Push( ProjectedLine {
                _X1: line.0.0,
                _Y1: line.0.1,
                _X2: line.1.0,
                _Y2: line.1.1,
            });
        }
        for pt in &sceneFrame._Points {
            projectedPoints.Push( ProjectedPoint {
                _X: pt.0,
                _Y: pt.1,
                _Radius: pt.2,
                _CoreRadius: pt.3,
                _Alpha: pt.4,
            });
        }
    } else {
        // High performance CPU fallback if Swarm device is uninitialized
        let  	( center, scaleNorm) = state._Scene.CalcNormalization();
        let  	bboxLines = state._Scene.ProjectBoundingBox( width, height);
        for ( p1, p2) in &bboxLines {
            box_lines.Push( ProjectedLine {
                _X1: p1.0,
                _Y1: p1.1,
                _X2: p2.0,
                _Y2: p2.1,
            });
        }

        for pt in &state._Scene._Points {
            let  	nx = ( pt[0] - center[0]) * scaleNorm;
            let  	ny = ( pt[1] - center[1]) * scaleNorm;
            let  	nz = ( pt[2] - center[2]) * scaleNorm;
            let  	( px, py, pz) = camRef.Project( nx, ny, nz, width, height);
            let  	depthFactor = 0.3f32.max( 1.0f32.min( ( 300.0 - pz) / 400.0));
            let  	radius = ( 3.0 + depthFactor * 4.0) * dpr;
            let  	alpha = 0.5 + depthFactor * 0.5;
            let  	core_radius = ( 1.0 + depthFactor * 1.5) * dpr;

            projectedPoints.Push( ProjectedPoint {
                _X: px,
                _Y: py,
                _Radius: radius,
                _CoreRadius: core_radius,
                _Alpha: alpha,
            });
        }
    }

    let  	fileName = std::path::Path::new( &path)
        .file_name()
        .map( |n| n.to_string_lossy().into_owned())
        .unwrap_or_else( || "Block.pts".to_string());

    let  	bboxMin = state._Scene._BboxMin;
    let  	bboxMax = state._Scene._BboxMax;
    let  	bboxLabel = format!( "[{:.2}, {:.2}, {:.2}] → [{:.2}, {:.2}, {:.2}]",
        bboxMin[0], bboxMin[1], bboxMin[2],
        bboxMax[0], bboxMax[1], bboxMax[2]
    );

    let  	isParsedFile = std::path::Path::new( &path).exists();
    let  	overlay1 = format!(
        "Points: {} | Zoom: {:.2}x | Pan: ({:.0}, {:.0})",
        state._Scene._Points.len(),
        camRef._Zoom,
        camRef._PanX,
        camRef._PanY
    );
    let  	overlay2 = if isParsedFile {
        format!( "Source: {} | Rot: ({:.0}°, {:.0}°)", fileName, camRef._RotX.to_degrees(), camRef._RotY.to_degrees())
    } else {
        format!( "Swarm {} [{} dev] | Rot: ({:.0}°, {:.0}°)", SWARM_CLUSTER.Primary().Backend(), SWARM_CLUSTER.DeviceCount(), camRef._RotX.to_degrees(), camRef._RotY.to_degrees())
    };
    let  	shaderStatus = format!( "Swarm GPU [{}] ({} device{}) SceneGraph ({} pts)",
        SWARM_CLUSTER.Primary().Backend(),
        SWARM_CLUSTER.DeviceCount(),
        if SWARM_CLUSTER.DeviceCount() > 1 { "s" } else { "" },
        state._Scene._Points.len()
    );

    Ok( PtsFrameDto {
        _Points: projectedPoints,
        _BoxLines: box_lines,
        _FileName: fileName,
        _Count: state._Scene._Points.len(),
        _BboxLabel: bboxLabel,
        _ShaderStatus: shaderStatus,
        _OverlayText1: overlay1,
        _OverlayText2: overlay2,
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Resets camera pan, zoom, and rotation in the active SceneGraph to default centered view.
#[tauri::command]
pub fn	XplrResetCamera( path: String) -> Result< Camera, String>
{
    let  	mut guard = PTS_STATE.lock().map_err( |e| e.to_string())?;
    if let Some( state) = guard.get_mut( &path) {
        state._Scene.CameraMut().Reset();
        Ok( *state._Scene.Camera())
    } else {
        Ok( Camera::New())
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests
{
    use	super::*;
    use	std::fs::File;
    use	std::io::Write;

    #[test]
    fn	TestXplrFetchPtsPointsWithFile()
    {
        let  	tempPath = std::env::temp_dir().join( "test_fenst_cloud.pts");
        {
            let  	mut f = File::create( &tempPath).unwrap();
            writeln!( f, "3").unwrap();
            writeln!( f, "10.0 20.0 30.0").unwrap();
            writeln!( f, "40.0 50.0 60.0").unwrap();
            writeln!( f, "70.0 80.0 90.0").unwrap();
        }

        let  	res = XplrFetchPtsPoints( Some( tempPath.to_str().unwrap().to_string()));
        assert!( res.is_ok());

        let  	dto = res.unwrap();
        assert_eq!( dto._Count, 3);
        assert_eq!( dto._Points.len(), 3);
        assert_eq!( dto._Points[0], [10.0, 20.0, 30.0]);
        assert_eq!( dto._BboxMin, [10.0, 20.0, 30.0]);
        assert_eq!( dto._BboxMax, [70.0, 80.0, 90.0]);

        let  	_ = std::fs::remove_file( &tempPath);
    }

    #[test]
    fn	TestXplrProjectPtsWithFile()
    {
        let  	tempPath = std::env::temp_dir().join( "test_fenst_project.pts");
        {
            let  	mut f = File::create( &tempPath).unwrap();
            writeln!( f, "2").unwrap();
            writeln!( f, "0.0 0.0 0.0").unwrap();
            writeln!( f, "10.0 10.0 10.0").unwrap();
        }

        let  	pathStr = tempPath.to_str().unwrap().to_string();
        let  	res = XplrProjectPts(
            pathStr,
            800.0,
            600.0,
            1.0,
            30.0,
            "#00f3ff".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!( res.is_ok());

        let  	frame = res.unwrap();
        assert_eq!( frame._Count, 2);
        assert_eq!( frame._Points.len(), 2);
        assert_eq!( frame._BoxLines.len(), 12);
        assert!( frame._ShaderStatus.contains( "SceneGraph"));

        let  	_ = std::fs::remove_file( &tempPath);
    }

    #[test]
    fn	TestXplrProjectPtsWithCameraPanZoomRotate()
    {
        let  	tempPath = std::env::temp_dir().join( "test_fenst_camera_project.pts");
        {
            let  	mut f = File::create( &tempPath).unwrap();
            writeln!( f, "2").unwrap();
            writeln!( f, "0.0 0.0 0.0").unwrap();
            writeln!( f, "10.0 10.0 10.0").unwrap();
        }

        let  	pathStr = tempPath.to_str().unwrap().to_string();

        // Standard projection
        let  	res = XplrProjectPts(
            pathStr.clone(),
            800.0,
            600.0,
            1.0,
            30.0,
            "#00f3ff".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!( res.is_ok());
        let  	frame = res.unwrap();
        assert_eq!( frame._Count, 2);
        assert_eq!( frame._Points.len(), 2);
        assert_eq!( frame._BoxLines.len(), 12);
        assert!( frame._ShaderStatus.contains( "SceneGraph"));

        // Interactive projection with Pan, Zoom, and Rotation
        let  	resInteractive = XplrProjectPts(
            pathStr.clone(),
            800.0,
            600.0,
            1.0,
            0.0,
            "#00f3ff".to_string(),
            Some( 100.0),
            Some( -50.0),
            Some( 2.5),
            Some( 0.8),
            Some( 1.2),
            Some( true),
        );
        assert!( resInteractive.is_ok());
        let  	frameInteractive = resInteractive.unwrap();
        assert!( frameInteractive._OverlayText1.contains( "2.50x"));
        assert!( frameInteractive._OverlayText1.contains( "100, -50"));

        // Reset Camera
        let  	resetRes = XplrResetCamera( pathStr.clone());
        assert!( resetRes.is_ok());
        let  	cam = resetRes.unwrap();
        assert_eq!( cam._PanX, 0.0);
        assert_eq!( cam._PanY, 0.0);
        assert_eq!( cam._Zoom, 1.0);

        let  	_ = std::fs::remove_file( &tempPath);
    }

    #[test]
    fn	TestXplrCommandsGeneral()
    {
        let  	contentRes = XplrFetchContent( "Cargo.toml".to_string());
        assert!( contentRes.is_ok());

        let  	chunkRes = XplrFetchChunk( "Cargo.toml".to_string(), 0, 50);
        assert!( chunkRes.is_ok());
        assert_eq!( chunkRes.unwrap()._Length, 50);

        let  	infoRes = XplrLeafInfo( "Cargo.toml".to_string());
        assert!( infoRes.is_ok());
        assert_eq!( infoRes.unwrap()._Name, "Cargo.toml");
    }

    #[test]
    fn	TestWorkbenchBunnyDataParsing()
    {
        let  	bunnyPath = std::path::Path::new( "workbench/bunnyData.pts");
        if bunnyPath.exists() {
            let  	res = XplrFetchPtsPoints( Some( "workbench/bunnyData.pts".to_string()));
            assert!( res.is_ok());
            let  	dto = res.unwrap();
            assert_eq!( dto._Count, 30571);
            assert_eq!( dto._Points.len(), 30571);
        }
    }

    #[test]
    fn	TestWorkbenchBlubObjParsing()
    {
        let  	blubPath = std::path::Path::new( "workbench/blub/blub_control_mesh.obj");
        if blubPath.exists() {
            let  	res = XplrFetchWaveObj( Some( "workbench/blub/blub_control_mesh.obj".to_string()));
            assert!( res.is_ok());
            let  	dto = res.unwrap();
            assert!( dto._VertexCount > 0);
            assert!( dto._FaceCount > 0);
            assert_eq!( dto._Points.len(), dto._VertexCount);
            assert!( dto._Triangles.len() > 0);
            assert!( dto._Edges.len() > 0);
            assert_eq!( dto._Normals.len(), dto._Triangles.len());
        }
    }
}
