//-- scene.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::silo::Buff;
use	serde::{ Serialize, Deserialize };

// ---------------------------------------------------------------------------------------------------------------------------------

/// 3D Viewport Camera managing Pan, Zoom, and Rotation transformations.
#[derive( Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Camera
{
    #[serde( rename = "pan_x")]
    pub _PanX:        f32,
    #[serde( rename = "pan_y")]
    pub _PanY:        f32,
    #[serde( rename = "zoom")]
    pub _Zoom:        f32,
    #[serde( rename = "rot_x")]
    pub _RotX:        f32,
    #[serde( rename = "rot_y")]
    pub _RotY:        f32,
    #[serde( rename = "fov")]
    pub _Fov:         f32,
    #[serde( rename = "distance")]
    pub _Distance:    f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Default for Camera
{
    fn	default() -> Self
    {
        Self::New()
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Camera
{
    pub fn	New() -> Self
    {
        Self {
            _PanX:      0.0,
            _PanY:      0.0,
            _Zoom:      1.0,
            _RotX:      0.4,
            _RotY:      0.6,
            _Fov:       350.0,
            _Distance:  250.0,
        }
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Translates camera viewport by (dx, dy) screen delta.
    pub fn	Pan( &mut self, dx: f32, dy: f32)
    {
        self._PanX += dx;
        self._PanY += dy;
    }

    /// Multiplies current zoom level by a scaling factor.
    pub fn	Zoom( &mut self, factor: f32)
    {
        self._Zoom = ( self._Zoom * factor).clamp( 0.05, 50.0);
    }

    /// Sets absolute zoom level.
    pub fn	SetZoom( &mut self, zoom: f32)
    {
        self._Zoom = zoom.clamp( 0.05, 50.0);
    }

    /// Increments pitch and yaw rotation angles (in radians).
    pub fn	Rotate( &mut self, dRotX: f32, dRotY: f32)
    {
        self._RotX += dRotX;
        self._RotY += dRotY;
    }

    /// Sets absolute rotation angles (in radians).
    pub fn	SetRotation( &mut self, rotX: f32, rotY: f32)
    {
        self._RotX = rotX;
        self._RotY = rotY;
    }

    /// Sets absolute pan offset.
    pub fn	SetPan( &mut self, panX: f32, panY: f32)
    {
        self._PanX = panX;
        self._PanY = panY;
    }

    /// Resets camera to default perspective view and pan offset.
    pub fn	Reset( &mut self)
    {
        self._PanX = 0.0;
        self._PanY = 0.0;
        self._Zoom = 1.0;
        self._RotX = 0.4;
        self._RotY = 0.6;
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Projects 3D world coordinate (x, y, z) to 2D screen coordinate (projX, projY, depthZ)
    /// incorporating camera rotation, zoom, pan, and perspective division.
    pub fn	Project(
        &self,
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
    ) -> ( f32, f32, f32)
    {
        let  	cosY = self._RotY.cos();
        let  	sinY = self._RotY.sin();
        let  	x1 = x * cosY + z * sinY;
        let  	z1 = -x * sinY + z * cosY;

        let  	cosX = self._RotX.cos();
        let  	sinX = self._RotX.sin();
        let  	y2 = y * cosX - z1 * sinX;
        let  	z2 = y * sinX + z1 * cosX;

        let  	scale = ( self._Fov * self._Zoom) / ( self._Distance + z2);

        let  	projX = width / 2.0 + self._PanX + x1 * scale;
        let  	projY = height / 2.0 + self._PanY - y2 * scale;

        ( projX, projY, z2)
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// SceneGraph representing the active 3D visualization scene (camera, points, bounding boxes).
#[derive( Clone, Debug)]
pub struct SceneGraph
{
    pub _Camera:     Camera,
    pub _Points:     Buff< [f32; 3]>,
    pub _BboxMin:    [f32; 3],
    pub _BboxMax:    [f32; 3],
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl SceneGraph
{
    pub fn	New() -> Self
    {
        Self {
            _Camera:   Camera::New(),
            _Points:   Buff::New(),
            _BboxMin:  [ -20.0, -20.0, -20.0 ],
            _BboxMax:  [ 20.0, 20.0, 20.0 ],
        }
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    pub fn	WithPoints( points: Buff< [f32; 3]>, bboxMin: [f32; 3], bboxMax: [f32; 3]) -> Self
    {
        Self {
            _Camera:   Camera::New(),
            _Points:   points,
            _BboxMin:  bboxMin,
            _BboxMax:  bboxMax,
        }
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Returns a mutable reference to the scene camera.
    pub fn	CameraMut( &mut self) -> &mut Camera
    {
        &mut self._Camera
    }

    /// Returns a reference to the scene camera.
    pub fn	Camera( &self) -> &Camera
    {
        &self._Camera
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Calculates center and scale normalization factors for bounding box fitting.
    pub fn	CalcNormalization( &self) -> ( [f32; 3], f32)
    {
        let  	cx = ( self._BboxMin[0] + self._BboxMax[0]) * 0.5;
        let  	cy = ( self._BboxMin[1] + self._BboxMax[1]) * 0.5;
        let  	cz = ( self._BboxMin[2] + self._BboxMax[2]) * 0.5;
        let  	dx = self._BboxMax[0] - self._BboxMin[0];
        let  	dy = self._BboxMax[1] - self._BboxMin[1];
        let  	dz = self._BboxMax[2] - self._BboxMin[2];
        let  	maxDim = dx.max( dy).max( dz);
        let  	scaleNorm = if maxDim > 1e-4 { 35.0 / maxDim } else { 1.0 };

        ( [ cx, cy, cz ], scaleNorm)
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Projects the 12 edges of the bounding box wireframe to 2D line segments.
    pub fn	ProjectBoundingBox( &self, width: f32, height: f32) -> Buff< ( ( f32, f32), ( f32, f32))>
    {
        let  	( center, scaleNorm) = self.CalcNormalization();
        let  	bMin = self._BboxMin;
        let  	bMax = self._BboxMax;

        let  	verts = [
            [ ( bMin[0] - center[0]) * scaleNorm, ( bMin[1] - center[1]) * scaleNorm, ( bMin[2] - center[2]) * scaleNorm ],
            [ ( bMax[0] - center[0]) * scaleNorm, ( bMin[1] - center[1]) * scaleNorm, ( bMin[2] - center[2]) * scaleNorm ],
            [ ( bMax[0] - center[0]) * scaleNorm, ( bMax[1] - center[1]) * scaleNorm, ( bMin[2] - center[2]) * scaleNorm ],
            [ ( bMin[0] - center[0]) * scaleNorm, ( bMax[1] - center[1]) * scaleNorm, ( bMin[2] - center[2]) * scaleNorm ],
            [ ( bMin[0] - center[0]) * scaleNorm, ( bMin[1] - center[1]) * scaleNorm, ( bMax[2] - center[2]) * scaleNorm ],
            [ ( bMax[0] - center[0]) * scaleNorm, ( bMin[1] - center[1]) * scaleNorm, ( bMax[2] - center[2]) * scaleNorm ],
            [ ( bMax[0] - center[0]) * scaleNorm, ( bMax[1] - center[1]) * scaleNorm, ( bMax[2] - center[2]) * scaleNorm ],
            [ ( bMin[0] - center[0]) * scaleNorm, ( bMax[1] - center[1]) * scaleNorm, ( bMax[2] - center[2]) * scaleNorm ],
        ];

        let  	edges = [
            ( 0, 1), ( 1, 2), ( 2, 3), ( 3, 0),
            ( 4, 5), ( 5, 6), ( 6, 7), ( 7, 4),
            ( 0, 4), ( 1, 5), ( 2, 6), ( 3, 7),
        ];

        let  	mut projectedVerts = Buff::New();
        for v in &verts {
            let  	( px, py, _) = self._Camera.Project( v[0], v[1], v[2], width, height);
            projectedVerts.Push( ( px, py));
        }

        let  	mut lines = Buff::New();
        for ( i, j) in &edges {
            let  	p1 = projectedVerts[*i];
            let  	p2 = projectedVerts[*j];
            lines.Push( ( p1, p2));
        }

        lines
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
