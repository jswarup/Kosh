//-- styew/state.rs -----------------------------------------------------------------------------------------------------------------
use	std::path::PathBuf;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Represents a 3D OBJ render mode in the native viewport.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjRenderMode
{
    Points,
    Wireframe,
    Facets,
    ShadedWire,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Represents an open document tab in the native workspace.
#[derive( Clone, Debug, PartialEq)]
pub struct OpenTab
{
    pub _Id:        String,
    pub _Path:      PathBuf,
    pub _Name:      String,
    pub _Content:   String,
    pub _LineCount: usize,
    pub _Size:      u64,
    pub _IsPts:     bool,
    pub _IsObj:     bool,
    pub _IsFresco:  bool,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// 3D Camera navigation state for native viewports.
#[derive( Clone, Copy, Debug, PartialEq)]
pub struct CameraState
{
    pub _RotX:          f32,
    pub _RotY:          f32,
    pub _PanX:          f32,
    pub _PanY:          f32,
    pub _Zoom:          f32,
    pub _IsDragging:    bool,
    pub _IsPanning:     bool,
    pub _LastMouseX:    f32,
    pub _LastMouseY:    f32,
    pub _IsInteractive: bool,
}

impl Default for CameraState
{
    fn	default() -> Self
    {
        Self {
            _RotX:          0.2,
            _RotY:          0.0,
            _PanX:          0.0,
            _PanY:          0.0,
            _Zoom:          1.0,
            _IsDragging:    false,
            _IsPanning:     false,
            _LastMouseX:    0.0,
            _LastMouseY:    0.0,
            _IsInteractive: false,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Primary application state.
pub struct AppState
{
    pub _RootPath:         PathBuf,
    pub _OpenTabs:         Vec< OpenTab>,
    pub _ActiveTabId:      Option< String>,
    pub _IsExplorerOpen:   bool,
    pub _ActiveObjMode:    ObjRenderMode,
    pub _PtsCamera:        CameraState,
    pub _ObjCamera:        CameraState,
    pub _StatusMessage:    String,
}

impl Default for AppState
{
    fn	default() -> Self
    {
        Self {
            _RootPath:       PathBuf::from( "."),
            _OpenTabs:       Vec::new(),
            _ActiveTabId:    None,
            _IsExplorerOpen: true,
            _ActiveObjMode:  ObjRenderMode::Facets,
            _PtsCamera:      CameraState::default(),
            _ObjCamera:      CameraState::default(),
            _StatusMessage:  "Ready — 100% Pure Native Rust (egui + wgpu)".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
