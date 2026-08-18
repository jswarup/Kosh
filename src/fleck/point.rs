//-- point.rs -------------------------------------------------------------------------------------------------------------------

/// Represents a 3D point with 32-bit floating-point coordinates (x, y, z).
#[derive( Clone, Copy, Debug, Default, PartialEq)]
pub struct Pt3f
{
    pub _X: f32,
    pub _Y: f32,
    pub _Z: f32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Pt3f
{
    pub fn	New( x: f32, y: f32, z: f32) -> Self
    {
        Self {
            _X: x,
            _Y: y,
            _Z: z,
        }
    }

    pub fn	Pos( &self) -> [f32; 3]
    {
        [self._X, self._Y, self._Z]
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a 4D point/homogeneous vertex with 32-bit floating-point coordinates (x, y, z, w).
#[derive( Clone, Copy, Debug, Default, PartialEq)]
pub struct Pt4f
{
    pub _X: f32,
    pub _Y: f32,
    pub _Z: f32,
    pub _W: f32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Pt4f
{
    pub fn	New( x: f32, y: f32, z: f32) -> Self
    {
        Self {
            _X: x,
            _Y: y,
            _Z: z,
            _W: 1.0,
        }
    }

    pub fn	WithW( x: f32, y: f32, z: f32, w: f32) -> Self
    {
        Self {
            _X: x,
            _Y: y,
            _Z: z,
            _W: w,
        }
    }

    pub fn	Pos( &self) -> [f32; 3]
    {
        [self._X, self._Y, self._Z]
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a 3D direction / normal vector with 32-bit floating-point coordinates (x, y, z).
#[derive( Clone, Copy, Debug, Default, PartialEq)]
pub struct Dir3f
{
    pub _X: f32,
    pub _Y: f32,
    pub _Z: f32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Dir3f
{
    pub fn	New( x: f32, y: f32, z: f32) -> Self
    {
        Self {
            _X: x,
            _Y: y,
            _Z: z,
        }
    }

    pub fn	Vec( &self) -> [f32; 3]
    {
        [self._X, self._Y, self._Z]
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
