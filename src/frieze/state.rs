//-- frieze/state.rs -----------------------------------------------------------------------------------------------------------------
use	std::path::PathBuf;
use	std::sync::Arc;
use	egui::{ Context, Color32, Stroke, Theme };
use	crate::swarm::{ Camera, SwarmEngine, ViewportRenderer };
use	crate::frieze::gpu_cache::GpuMeshCache;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Represents the active color theme preset of the workspace.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTheme
{
    Dark,
    Light,
    Cyberpunk,
    Nord,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Theme mode setting: follow OS or explicitly select a theme preset.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode
{
    System,
    Explicit( AppTheme),
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Font family preference override.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFamilyPreference
{
    Default,
    Proportional,
    Monospace,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Typography semantic levels.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypographyLevel
{
    SmallCaption,
    Small,
    Body,
    Heading,
    Title,
    Monospace,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Unified appearance and styling configuration.
#[derive( Clone, Debug, PartialEq)]
pub struct AppearanceSettings
{
    pub _ThemeMode:           ThemeMode,
    pub _ExplicitTheme:       AppTheme,
    pub _FontFamily:          FontFamilyPreference,
    pub _FontSizeOffset:      f32,
    pub _CustomTextColor:     Option< Color32>,
}

impl Default for AppearanceSettings
{
    fn	default() -> Self
    {
        Self {
            _ThemeMode:           ThemeMode::System,
            _ExplicitTheme:       AppTheme::Dark,
            _FontFamily:          FontFamilyPreference::Default,
            _FontSizeOffset:      0.0,
            _CustomTextColor:     None,
        }
    }
}

impl AppearanceSettings
{
    /// Resolves the active theme based on OS primacy and theme mode.
    pub fn	ResolveTheme( &self, ctx: &Context) -> AppTheme
    {
        match self._ThemeMode {
            ThemeMode::System => {
                match ctx.system_theme() {
                    Some( Theme::Light) => AppTheme::Light,
                    _                  => AppTheme::Dark,
                }
            }
            ThemeMode::Explicit( theme) => theme,
        }
    }

    /// Computes the font size for a given typography semantic level with offset.
    pub fn	FontSize( &self, level: TypographyLevel) -> f32
    {
        let  	base = match level {
            TypographyLevel::SmallCaption => 11.0,
            TypographyLevel::Small        => 13.0,
            TypographyLevel::Body         => 15.0,
            TypographyLevel::Heading      => 19.0,
            TypographyLevel::Title        => 26.0,
            TypographyLevel::Monospace    => 14.0,
        };
        ( base + self._FontSizeOffset).max( 6.0)
    }

    /// Applies the unified appearance settings (visuals + typography) to the egui context.
    pub fn	Apply( &self, ctx: &Context)
    {
        let  	activeTheme = self.ResolveTheme( ctx);

        // 1. Build Base Visuals from resolved theme
        let  	mut visuals = match activeTheme {
            AppTheme::Dark => {
                let  	mut v = egui::Visuals::dark();
                v.panel_fill = Color32::from_rgb( 24, 24, 37);
                v.window_fill = Color32::from_rgb( 30, 30, 46);
                v.override_text_color = Some( Color32::from_rgb( 205, 214, 244));
                v.widgets.noninteractive.bg_fill = Color32::from_rgb( 30, 30, 46);
                v.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68));
                v.selection.bg_fill = Color32::from_rgb( 137, 180, 250);
                v
            }
            AppTheme::Light => {
                let  	mut v = egui::Visuals::light();
                v.panel_fill = Color32::from_rgb( 239, 241, 245);
                v.window_fill = Color32::from_rgb( 230, 233, 239);
                v.override_text_color = Some( Color32::from_rgb( 76, 79, 105));
                v.widgets.noninteractive.bg_fill = Color32::from_rgb( 230, 233, 239);
                v.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 204, 208, 218));
                v.selection.bg_fill = Color32::from_rgb( 30, 102, 245);
                v
            }
            AppTheme::Cyberpunk => {
                let  	mut v = egui::Visuals::dark();
                v.panel_fill = Color32::from_rgb( 13, 2, 33);
                v.window_fill = Color32::from_rgb( 23, 10, 48);
                v.override_text_color = Some( Color32::from_rgb( 0, 255, 204));
                v.widgets.noninteractive.bg_fill = Color32::from_rgb( 23, 10, 48);
                v.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 255, 0, 127));
                v.selection.bg_fill = Color32::from_rgb( 255, 0, 127);
                v
            }
            AppTheme::Nord => {
                let  	mut v = egui::Visuals::dark();
                v.panel_fill = Color32::from_rgb( 46, 52, 64);
                v.window_fill = Color32::from_rgb( 59, 66, 82);
                v.override_text_color = Some( Color32::from_rgb( 236, 239, 244));
                v.widgets.noninteractive.bg_fill = Color32::from_rgb( 59, 66, 82);
                v.widgets.noninteractive.bg_stroke = Stroke::new( 1.0, Color32::from_rgb( 76, 86, 106));
                v.selection.bg_fill = Color32::from_rgb( 136, 192, 208);
                v
            }
        };

        // 2. Primacy Override: Specific text color override if provided
        if let  	Some( customColor) = self._CustomTextColor {
            visuals.override_text_color = Some( customColor);
        }

        ctx.set_visuals( visuals);

        // 3. Apply Typography and Font Scaling to egui context
        for theme in [Theme::Dark, Theme::Light] {
            let  	mut style = (*ctx.style_of( theme)).clone();
            style.text_styles.insert( egui::TextStyle::Small, egui::FontId::proportional( self.FontSize( TypographyLevel::Small)));
            style.text_styles.insert( egui::TextStyle::Body, egui::FontId::proportional( self.FontSize( TypographyLevel::Body)));
            style.text_styles.insert( egui::TextStyle::Button, egui::FontId::proportional( self.FontSize( TypographyLevel::Body)));
            style.text_styles.insert( egui::TextStyle::Heading, egui::FontId::proportional( self.FontSize( TypographyLevel::Heading)));
            style.text_styles.insert( egui::TextStyle::Monospace, egui::FontId::monospace( self.FontSize( TypographyLevel::Monospace)));
            ctx.set_style_of( theme, style);
        }
    }
}

impl AppTheme
{
    pub fn	Apply( &self, ctx: &Context)
    {
        let  	appearance = AppearanceSettings {
            _ThemeMode:           ThemeMode::Explicit( *self),
            _ExplicitTheme:       *self,
            _FontFamily:          FontFamilyPreference::Default,
            _FontSizeOffset:      0.0,
            _CustomTextColor:     None,
        };
        appearance.Apply( ctx);
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
    pub _Appearance:            AppearanceSettings,
    pub _IsSettingsWindowOpen:  bool,
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
            _Appearance:            AppearanceSettings::default(),
            _IsSettingsWindowOpen:  false,
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
