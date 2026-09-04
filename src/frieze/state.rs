//-- frieze/state.rs ----------------------------------------------------------------------------------------------------------------
use	std::cell::RefCell;
use	std::path::{ Path, PathBuf };
use	std::rc::Rc;

use	crate::frieze::gpu_cache::GpuMeshCache;
use	crate::swarm::viewport::ObjRenderMode;
use	crate::swarm::Camera;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Document kind for an open tab.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabKind
{
    Pts,
    Obj,
    Fresco,
    Image,
    Vcd,
    Text,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl TabKind
{
    pub fn	FromPath( path: &Path) -> Self
    {
        let  	ext = path.extension().and_then( |e| e.to_str()).unwrap_or( "").to_lowercase();
        if ext == "pts" {
            TabKind::Pts
        } else if ext == "obj" {
            TabKind::Obj
        } else if ext == "fresco" || ext == "frsc" {
            TabKind::Fresco
        } else if ext == "png" || ext == "jpg" || ext == "jpeg" {
            TabKind::Image
        } else if ext == "vcd" {
            TabKind::Vcd
        } else {
            TabKind::Text
        }
    }

    pub fn	Badge( &self) -> &str
    {
        match self {
            TabKind::Pts => "PTS",
            TabKind::Obj => "OBJ",
            TabKind::Fresco => "FRESCO",
            TabKind::Image => "IMG",
            TabKind::Vcd => "VCD",
            TabKind::Text => "TXT",
        }
    }

    pub fn	TabLabel( &self, name: &str) -> String
    {
        match self {
            TabKind::Text => name.to_string(),
            _ => format!( "{}  {}", self.Badge(), name),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Active color theme preset for the wxDragon workspace.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTheme
{
    Dark,
    Light,
    Cyberpunk,
    Nord,
}

impl AppTheme
{
    pub fn	panel_rgb( &self) -> ( u8, u8, u8)
    {
        match self {
            AppTheme::Dark => ( 24, 24, 37),
            AppTheme::Light => ( 239, 241, 245),
            AppTheme::Cyberpunk => ( 13, 2, 33),
            AppTheme::Nord => ( 46, 52, 64),
        }
    }

    pub fn	viewport_rgb( &self) -> ( u8, u8, u8)
    {
        ( 11, 15, 25)
    }

    pub fn	accent_rgb( &self) -> ( u8, u8, u8)
    {
        match self {
            AppTheme::Dark => ( 137, 180, 250),
            AppTheme::Light => ( 30, 102, 245),
            AppTheme::Cyberpunk => ( 0, 255, 204),
            AppTheme::Nord => ( 136, 192, 208),
        }
    }

    pub fn	text_rgb( &self) -> ( u8, u8, u8)
    {
        match self {
            AppTheme::Dark | AppTheme::Cyberpunk | AppTheme::Nord => ( 205, 214, 244),
            AppTheme::Light => ( 76, 79, 105),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Represents an open document tab in the native workspace.
#[derive( Clone, Debug)]
pub struct OpenTab
{
    pub _Path: PathBuf,
    pub _Name: String,
    pub _Kind: TabKind,
}

impl OpenTab
{
    pub fn	New( path: PathBuf) -> Self
    {
        let  	name = path.file_name().and_then( |n| n.to_str()).unwrap_or( "file").to_string();
        let  	kind = TabKind::FromPath( &path);
        Self {
            _Path: path,
            _Name: name,
            _Kind: kind,
        }
    }

    pub fn	IsPts( &self) -> bool { self._Kind == TabKind::Pts }
    pub fn	IsObj( &self) -> bool { self._Kind == TabKind::Obj }
    pub fn	IsFresco( &self) -> bool { self._Kind == TabKind::Fresco }
    pub fn	IsImg( &self) -> bool { self._Kind == TabKind::Image }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Primary application state, shared across widgets via Rc<RefCell<_>>.
pub struct AppState
{
    pub _Theme:          AppTheme,
    pub _RootPath:       PathBuf,
    pub _OpenTabs:       Vec< OpenTab>,
    pub _StatusMessage:  String,
    pub _MeshCache:      GpuMeshCache,
    pub _Camera:         Camera,
    pub _ActiveObjMode:  ObjRenderMode,
}

impl Default for AppState
{
    fn	default() -> Self
    {
        Self {
            _Theme:         AppTheme::Dark,
            _RootPath:      std::env::current_dir().unwrap_or_default(),
            _OpenTabs:      Vec::new(),
            _StatusMessage: "Ready".to_string(),
            _MeshCache:     GpuMeshCache::New(),
            _Camera:        Camera::New(),
            _ActiveObjMode: ObjRenderMode::Facets,
        }
    }
}

pub type SharedState = Rc< RefCell< AppState>>;
