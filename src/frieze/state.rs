//-- frieze/state.rs -----------------------------------------------------------------------------------------------------------------
use	std::path::PathBuf;
use	std::sync::Arc;
use	egui::{ Context, Color32, Stroke };
use	crate::swarm::{ Camera, SwarmEngine, ViewportRenderer };
use	crate::frieze::gpu_cache::GpuMeshCache;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Represents the active color theme of the workspace.
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
    /// Applies the selected theme visuals to the egui context.
    pub fn	Apply( &self, ctx: &Context)
    {
        match self {
            AppTheme::Dark => {
                let  	mut visuals = egui::Visuals::dark();
                visuals.panel_fill = Color32::from_rgb( 24, 24, 37);
                visuals.window_fill = Color32::from_rgb( 30, 30, 46);
                visuals.override_text_color = Some( Color32::from_rgb( 205, 214, 244));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb( 30, 30, 46);
                visuals.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68));
                visuals.selection.bg_fill = Color32::from_rgb( 137, 180, 250);
                ctx.set_visuals( visuals);
            }
            AppTheme::Light => {
                let  	mut visuals = egui::Visuals::light();
                visuals.panel_fill = Color32::from_rgb( 239, 241, 245);
                visuals.window_fill = Color32::from_rgb( 230, 233, 239);
                visuals.override_text_color = Some( Color32::from_rgb( 76, 79, 105));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb( 230, 233, 239);
                visuals.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 204, 208, 218));
                visuals.selection.bg_fill = Color32::from_rgb( 30, 102, 245);
                ctx.set_visuals( visuals);
            }
            AppTheme::Cyberpunk => {
                let  	mut visuals = egui::Visuals::dark();
                visuals.panel_fill = Color32::from_rgb( 13, 2, 33);
                visuals.window_fill = Color32::from_rgb( 23, 10, 48);
                visuals.override_text_color = Some( Color32::from_rgb( 0, 255, 204));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb( 23, 10, 48);
                visuals.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 255, 0, 127));
                visuals.selection.bg_fill = Color32::from_rgb( 255, 0, 127);
                ctx.set_visuals( visuals);
            }
            AppTheme::Nord => {
                let  	mut visuals = egui::Visuals::dark();
                visuals.panel_fill = Color32::from_rgb( 46, 52, 64);
                visuals.window_fill = Color32::from_rgb( 59, 66, 82);
                visuals.override_text_color = Some( Color32::from_rgb( 236, 239, 244));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb( 59, 66, 82);
                visuals.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 76, 86, 106));
                visuals.selection.bg_fill = Color32::from_rgb( 136, 192, 208);
                ctx.set_visuals( visuals);
            }
        }
    }

    pub fn	PanelFill( &self) -> Color32
    {
        match self {
            AppTheme::Dark      => Color32::from_rgb( 24, 24, 37),
            AppTheme::Light     => Color32::from_rgb( 239, 241, 245),
            AppTheme::Cyberpunk => Color32::from_rgb( 13, 2, 33),
            AppTheme::Nord      => Color32::from_rgb( 46, 52, 64),
        }
    }

    pub fn	BottomFill( &self) -> Color32
    {
        match self {
            AppTheme::Dark      => Color32::from_rgb( 17, 17, 27),
            AppTheme::Light     => Color32::from_rgb( 220, 224, 232),
            AppTheme::Cyberpunk => Color32::from_rgb( 7, 1, 18),
            AppTheme::Nord      => Color32::from_rgb( 36, 41, 51),
        }
    }

    pub fn	BorderStroke( &self) -> Stroke
    {
        match self {
            AppTheme::Dark      => Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)),
            AppTheme::Light     => Stroke::new( 1.0, Color32::from_rgb( 204, 208, 218)),
            AppTheme::Cyberpunk => Stroke::new( 1.0, Color32::from_rgb( 255, 0, 127)),
            AppTheme::Nord      => Stroke::new( 1.0, Color32::from_rgb( 76, 86, 106)),
        }
    }

    pub fn	AccentColor( &self) -> Color32
    {
        match self {
            AppTheme::Dark      => Color32::from_rgb( 137, 180, 250),
            AppTheme::Light     => Color32::from_rgb( 30, 102, 245),
            AppTheme::Cyberpunk => Color32::from_rgb( 0, 255, 204),
            AppTheme::Nord      => Color32::from_rgb( 136, 192, 208),
        }
    }
}

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

/// File explorer display mode (Large Icon Grid or Details List).
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplorerViewMode
{
    Grid,
    Details,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Represents an open document tab in the native workspace.
#[derive( Clone, Debug, PartialEq)]
pub struct OpenTab
{
    pub _Id:         String,
    pub _Path:       PathBuf,
    pub _Name:       String,
    pub _Content:    String,
    pub _LineCount:  usize,
    pub _Size:       u64,
    pub _IsPts:      bool,
    pub _IsObj:      bool,
    pub _IsFresco:   bool,
    pub _IsExplorer: bool,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Primary application state.
pub struct AppState
{
    pub _Theme:                 AppTheme,
    pub _RootPath:              PathBuf,
    pub _OpenTabs:              Vec< OpenTab>,
    pub _ActiveTabId:           Option< String>,
    pub _IsExplorerOpen:        bool,
    pub _IsExplorerWindowOpen:  bool,
    pub _ExplorerSearch:        String,
    pub _ExplorerViewMode:      ExplorerViewMode,
    pub _ActiveObjMode:         ObjRenderMode,
    pub _PtsCamera:             Camera,
    pub _ObjCamera:             Camera,
    pub _StatusMessage:         String,
    pub _MeshCache:             GpuMeshCache,
    pub _Engine:                Option< SwarmEngine>,
    pub _ViewportRenderer:      Option< Arc< ViewportRenderer>>,
}

impl Default for AppState
{
    fn	default() -> Self
    {
        Self {
            _Theme:                 AppTheme::Dark,
            _RootPath:              PathBuf::from( "."),
            _OpenTabs:              Vec::new(),
            _ActiveTabId:           None,
            _IsExplorerOpen:        true,
            _IsExplorerWindowOpen:  false,
            _ExplorerSearch:        String::new(),
            _ExplorerViewMode:      ExplorerViewMode::Grid,
            _ActiveObjMode:         ObjRenderMode::Facets,
            _PtsCamera:             Camera::New(),
            _ObjCamera:             Camera::New(),
            _StatusMessage:         "Ready - 100% In-Process Direct GPU (egui + wgpu + swarm)".to_string(),
            _MeshCache:             GpuMeshCache::New(),
            _Engine:                None,
            _ViewportRenderer:      None,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
