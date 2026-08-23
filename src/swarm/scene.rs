//-- scene.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, Stash, U32 };
use	crate::swarm::{ SwarmEngine, SwarmCluster, SwarmError };
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
    /// Calculates 4x4 View-Projection matrix for GPU vertex shaders.
    pub fn	CalcViewProjMatrix( &self, width: f32, height: f32) -> [f32; 16]
    {
        let  	aspect = if height > 0.0 { width / height } else { 1.0 };
        let  	cosX = self._RotX.cos();
        let  	sinX = self._RotX.sin();
        let  	cosY = self._RotY.cos();
        let  	sinY = self._RotY.sin();

        let  	scale = self._Zoom * 0.005;
        let  	sx = scale / aspect;
        let  	sy = scale;
        let  	sz = scale * 0.1;

        let  	tx = self._PanX / ( width * 0.5 + 1.0);
        let  	ty = -self._PanY / ( height * 0.5 + 1.0);

        [
            cosY * sx,  sinX * sinY * sy, -cosX * sinY * sz,  0.0,
            0.0,        cosX * sy,        sinX * sz,         0.0,
            sinY * sx, -sinX * cosY * sy,  cosX * cosY * sz,  0.0,
            tx,         ty,               0.5,               1.0,
        ]
    }

    pub fn Fit(&mut self) {
        self._PanX = 0.0;
        self._PanY = 0.0;
        self._Zoom = 0.4;
        self._RotX = 0.0;
        self._RotY = 0.0;
    }

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

impl Default for SceneGraph
{
    fn	default() -> Self
    {
        Self::New()
    }
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
        let  	scaleNorm = if maxDim > 1e-4 { 240.0 / maxDim } else { 1.0 };

        ( [ cx, cy, cz ], scaleNorm)
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Projects all points in the scene to 2D screen coordinates with depth-scaled radius and alpha.
    pub fn	ProjectPoints( &self, width: f32, height: f32, dpr: f32) -> Buff< ( f32, f32, f32, f32, f32)>
    {
        let  	( center, scaleNorm) = self.CalcNormalization();
        let  	mut resultStash = Stash::WithCapacity( U32( self._Points.len() as u32));

        for pt in &self._Points {
            let  	nx = ( pt[0] - center[0]) * scaleNorm;
            let  	ny = ( pt[1] - center[1]) * scaleNorm;
            let  	nz = ( pt[2] - center[2]) * scaleNorm;

            let  	( px, py, z2) = self._Camera.Project( nx, ny, nz, width, height);

            let  	depthFactor = ( ( 300.0 - z2) / 400.0).clamp( 0.3, 1.0);
            let  	radius = ( 2.5 + depthFactor * 2.5) * dpr;
            let  	coreRadius = ( 1.0 + depthFactor * 1.0) * dpr;
            let  	alpha = 0.5 + depthFactor * 0.5;

            resultStash.Push( ( px, py, radius, coreRadius, alpha));
        }

        resultStash.IntoBuff()
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

        let  	mut projectedVerts = [ ( 0.0f32, 0.0f32); 8 ];
        for ( idx, v) in verts.iter().enumerate() {
            let  	( px, py, _) = self._Camera.Project( v[0], v[1], v[2], width, height);
            projectedVerts[idx] = ( px, py);
        }

        let  	mut linesStash = Stash::WithCapacity( U32( 12));
        for ( i, j) in &edges {
            let  	p1 = projectedVerts[*i];
            let  	p2 = projectedVerts[*j];
            linesStash.Push( ( p1, p2));
        }

        linesStash.IntoBuff()
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Encodes camera parameters and scene normalization into a 13-float uniform array for Swarm GPU kernels.
    pub fn	CameraParams( &self, width: f32, height: f32) -> [f32; 13]
    {
        let  	( center, scaleNorm) = self.CalcNormalization();
        [
            self._Camera._RotX,
            self._Camera._RotY,
            self._Camera._Zoom,
            self._Camera._PanX,
            self._Camera._PanY,
            self._Camera._Fov,
            self._Camera._Distance,
            width,
            height,
            center[0],
            center[1],
            center[2],
            scaleNorm,
        ]
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Dispatches point cloud transformation to the Swarm compute framework (GPU/CPU)
    /// and formats the resulting 2D coordinates with depth-scaled radius and alpha.
    pub fn	ProjectPointsSwarm(
        &self,
        engine: &SwarmEngine,
        width: f32,
        height: f32,
        dpr: f32,
        spirvBytes: Option< &[u8]>,
    ) -> Result< Buff< ( f32, f32, f32, f32, f32)>, SwarmError>
    {
        let  	camParams = self.CameraParams( width, height);
        let  	rawProjected = engine.RunCameraTransform( &self._Points, &camParams, spirvBytes)?;

        let  	mut resultStash = Stash::WithCapacity( U32( rawProjected.len() as u32));
        for proj in &rawProjected {
            let  	px = proj[0];
            let  	py = proj[1];
            let  	radius = proj[2] * dpr;
            let  	coreRadius = proj[3] * dpr;
            let  	alpha = proj[4];
            resultStash.Push( ( px, py, radius, coreRadius, alpha));
        }

        Ok( resultStash.IntoBuff())
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Dispatches bounding box wireframe vertex transformation to the Swarm compute GPU engine
    /// and connects the projected vertices into 12 line segments.
    pub fn	ProjectBoundingBoxSwarm(
        &self,
        engine: &SwarmEngine,
        width: f32,
        height: f32,
        spirvBytes: Option< &[u8]>,
    ) -> Result< Buff< ( ( f32, f32), ( f32, f32))>, SwarmError>
    {
        let  	bMin = self._BboxMin;
        let  	bMax = self._BboxMax;
        let  	corners = [
            [ bMin[0], bMin[1], bMin[2] ],
            [ bMax[0], bMin[1], bMin[2] ],
            [ bMax[0], bMax[1], bMin[2] ],
            [ bMin[0], bMax[1], bMin[2] ],
            [ bMin[0], bMin[1], bMax[2] ],
            [ bMax[0], bMin[1], bMax[2] ],
            [ bMax[0], bMax[1], bMax[2] ],
            [ bMin[0], bMax[1], bMax[2] ],
        ];

        let  	camParams = self.CameraParams( width, height);
        let  	rawProjected = engine.RunCameraTransform( &corners, &camParams, spirvBytes)?;

        let  	edges = [
            ( 0, 1), ( 1, 2), ( 2, 3), ( 3, 0),
            ( 4, 5), ( 5, 6), ( 6, 7), ( 7, 4),
            ( 0, 4), ( 1, 5), ( 2, 6), ( 3, 7),
        ];

        let  	mut linesStash = Stash::WithCapacity( U32( 12));
        if rawProjected.len() >= 8 {
            for ( i, j) in &edges {
                let  	p1 = ( rawProjected[*i][0], rawProjected[*i][1]);
                let  	p2 = ( rawProjected[*j][0], rawProjected[*j][1]);
                linesStash.Push( ( p1, p2));
            }
        }

        Ok( linesStash.IntoBuff())
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Dispatches the complete graphics scene projection (point cloud and bounding box wireframe)
    /// to the Swarm compute GPU engine for display.
    pub fn	ProjectSceneSwarm(
        &self,
        engine: &SwarmEngine,
        width: f32,
        height: f32,
        dpr: f32,
        spirvBytes: Option< &[u8]>,
    ) -> Result< SceneDisplayFrame, SwarmError>
    {
        let  	points = self.ProjectPointsSwarm( engine, width, height, dpr, spirvBytes)?;
        let  	boxLines = self.ProjectBoundingBoxSwarm( engine, width, height, spirvBytes)?;

        Ok( SceneDisplayFrame {
            _Points:    points,
            _BoxLines:  boxLines,
        })
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Dispatches point cloud projection across a multi-GPU SwarmCluster.
    pub fn	ProjectPointsCluster(
        &self,
        cluster: &SwarmCluster,
        width: f32,
        height: f32,
        dpr: f32,
        spirvBytes: Option< &[u8]>,
    ) -> Result< Buff< ( f32, f32, f32, f32, f32)>, SwarmError>
    {
        let  	camParams = self.CameraParams( width, height);
        let  	rawProjected = cluster.RunCameraTransformSharded( &self._Points, &camParams, spirvBytes)?;

        let  	mut resultStash = Stash::WithCapacity( U32( rawProjected.len() as u32));
        for proj in &rawProjected {
            let  	px = proj[0];
            let  	py = proj[1];
            let  	radius = proj[2] * dpr;
            let  	coreRadius = proj[3] * dpr;
            let  	alpha = proj[4];
            resultStash.Push( ( px, py, radius, coreRadius, alpha));
        }

        Ok( resultStash.IntoBuff())
    }

    // -----------------------------------------------------------------------------------------------------------------------------

    /// Dispatches full scene projection across a multi-GPU SwarmCluster.
    pub fn	ProjectSceneCluster(
        &self,
        cluster: &SwarmCluster,
        width: f32,
        height: f32,
        dpr: f32,
        spirvBytes: Option< &[u8]>,
    ) -> Result< SceneDisplayFrame, SwarmError>
    {
        let  	points = self.ProjectPointsCluster( cluster, width, height, dpr, spirvBytes)?;
        let  	engine = cluster.Primary();
        let  	boxLines = self.ProjectBoundingBoxSwarm( engine, width, height, spirvBytes)?;

        Ok( SceneDisplayFrame {
            _Points:    points,
            _BoxLines:  boxLines,
        })
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Frame output containing projected 2D points and bounding box line segments ready for Canvas rendering.
#[derive( Clone, Debug)]
pub struct SceneDisplayFrame
{
    pub _Points:    Buff< ( f32, f32, f32, f32, f32)>,
    pub _BoxLines:  Buff< ( ( f32, f32), ( f32, f32))>,
}

// ---------------------------------------------------------------------------------------------------------------------------------