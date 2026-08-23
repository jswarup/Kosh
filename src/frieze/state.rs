//-- frieze/state.rs ----------------------------------------------------------------------------------------------------------------
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::swarm::viewport::ObjRenderMode;
use crate::swarm::Camera;
use crate::frieze::gpu_cache::GpuMeshCache;

/// Active color theme preset for the wxDragon workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTheme {
    Dark,
    Light,
    Cyberpunk,
    Nord,
}

impl AppTheme {
    pub fn panel_rgb(&self) -> (u8, u8, u8) {
        match self {
            AppTheme::Dark => (24, 24, 37),
            AppTheme::Light => (239, 241, 245),
            AppTheme::Cyberpunk => (13, 2, 33),
            AppTheme::Nord => (46, 52, 64),
        }
    }

    pub fn viewport_rgb(&self) -> (u8, u8, u8) {
        (11, 15, 25)
    }

    pub fn accent_rgb(&self) -> (u8, u8, u8) {
        match self {
            AppTheme::Dark => (137, 180, 250),
            AppTheme::Light => (30, 102, 245),
            AppTheme::Cyberpunk => (0, 255, 204),
            AppTheme::Nord => (136, 192, 208),
        }
    }

    pub fn text_rgb(&self) -> (u8, u8, u8) {
        match self {
            AppTheme::Dark | AppTheme::Cyberpunk | AppTheme::Nord => (205, 214, 244),
            AppTheme::Light => (76, 79, 105),
        }
    }
}

/// Represents an open document tab in the native workspace.
#[derive(Clone, Debug)]
pub struct OpenTab {
    pub _Path: PathBuf,
    pub _Name: String,
    pub _IsPts: bool,
    pub _IsObj: bool,
    pub _IsFresco: bool,
    pub _IsImg: bool,
}

/// Primary application state, shared across widgets via `Rc<RefCell<_>>`.
pub struct AppState {
    pub _Theme: AppTheme,
    pub _RootPath: PathBuf,
    pub _OpenTabs: Vec<OpenTab>,
    pub _StatusMessage: String,
    pub _MeshCache: GpuMeshCache,
    pub _PtsCamera: Camera,
    pub _ObjCamera: Camera,
    pub _ActiveObjMode: ObjRenderMode,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            _Theme: AppTheme::Dark,
            _RootPath: std::env::current_dir().unwrap_or_default(),
            _OpenTabs: Vec::new(),
            _StatusMessage: "Ready".to_string(),
            _MeshCache: GpuMeshCache::New(),
            _PtsCamera: Camera::New(),
            _ObjCamera: Camera::New(),
            _ActiveObjMode: ObjRenderMode::Facets,
        }
    }
}

pub type SharedState = Rc<RefCell<AppState>>;

