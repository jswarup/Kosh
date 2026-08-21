//-- frieze/state.rs -----------------------------------------------------------------------------------------------------------------
use	std::path::PathBuf;
use	std::sync::Arc;
use	crate::swarm::{ Camera, SwarmEngine, ViewportRenderer };
use	crate::frieze::gpu_cache::GpuMeshCache;

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

/// Primary application state.
pub struct AppState
{
    pub _RootPath:         PathBuf,
    pub _OpenTabs:         Vec< OpenTab>,
    pub _ActiveTabId:      Option< String>,
    pub _IsExplorerOpen:   bool,
    pub _ActiveObjMode:    ObjRenderMode,
    pub _PtsCamera:        Camera,
    pub _ObjCamera:        Camera,
    pub _StatusMessage:    String,
    pub _MeshCache:        GpuMeshCache,
    pub _Engine:           Option< SwarmEngine>,
    pub _ViewportRenderer: Option< Arc< ViewportRenderer>>,
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
            _PtsCamera:      Camera::New(),
            _ObjCamera:      Camera::New(),
            _StatusMessage:  "Ready - 100% In-Process Direct GPU (egui + wgpu + swarm)".to_string(),
            _MeshCache:      GpuMeshCache::New(),
            _Engine:           None,
            _ViewportRenderer: None,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
