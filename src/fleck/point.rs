//-- point.rs -------------------------------------------------------------------------------------------------------------------

use	crate::fleck::vex::{ Vex2f, Vex3f, Vex4f };

//---------------------------------------------------------------------------------------------------------------------------------

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

impl From< [f32; 3]> for Pt3f
{
    fn	from( p: [f32; 3]) -> Self
    {
        Self::New( p[0], p[1], p[2])
    }
}

impl From< Pt3f> for [f32; 3]
{
    fn	from( pt: Pt3f) -> Self
    {
        [pt._X, pt._Y, pt._Z]
    }
}

impl From< Vex3f> for Pt3f
{
    fn	from( v: Vex3f) -> Self
    {
        Self::New( v._Data[0], v._Data[1], v._Data[2])
    }
}

impl From< Pt3f> for Vex3f
{
    fn	from( pt: Pt3f) -> Self
    {
        Vex3f::New3( pt._X, pt._Y, pt._Z)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a 3D weighted point / homogeneous vertex with coordinates (x, y, z, w).
#[derive( Clone, Copy, Debug, Default, PartialEq)]
pub struct WPt3f
{
    pub _X: f32,
    pub _Y: f32,
    pub _Z: f32,
    pub _W: f32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl WPt3f
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

impl From< Vex4f> for WPt3f
{
    fn	from( v: Vex4f) -> Self
    {
        Self::WithW( v._Data[0], v._Data[1], v._Data[2], v._Data[3])
    }
}

impl From< WPt3f> for Vex4f
{
    fn	from( pt: WPt3f) -> Self
    {
        Vex4f::New4( pt._X, pt._Y, pt._Z, pt._W)
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

impl From< [f32; 3]> for Dir3f
{
    fn	from( d: [f32; 3]) -> Self
    {
        Self::New( d[0], d[1], d[2])
    }
}

impl From< Dir3f> for [f32; 3]
{
    fn	from( d: Dir3f) -> Self
    {
        [d._X, d._Y, d._Z]
    }
}

impl From< Vex3f> for Dir3f
{
    fn	from( v: Vex3f) -> Self
    {
        Self::New( v._Data[0], v._Data[1], v._Data[2])
    }
}

impl From< Dir3f> for Vex3f
{
    fn	from( d: Dir3f) -> Self
    {
        Vex3f::New3( d._X, d._Y, d._Z)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a 2D weighted point / parameter coordinate (u, v, w).
#[derive( Clone, Copy, Debug, Default, PartialEq)]
pub struct WPt2f
{
    pub _U: f32,
    pub _V: f32,
    pub _W: f32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl WPt2f
{
    pub fn	New( u: f32, v: f32) -> Self
    {
        Self {
            _U: u,
            _V: v,
            _W: 0.0,
        }
    }

    pub fn	WithW( u: f32, v: f32, w: f32) -> Self
    {
        Self {
            _U: u,
            _V: v,
            _W: w,
        }
    }

    pub fn	Pos( &self) -> [f32; 2]
    {
        [self._U, self._V]
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< Vex2f> for WPt2f
{
    fn	from( v: Vex2f) -> Self
    {
        Self::New( v._Data[0], v._Data[1])
    }
}

impl From< WPt2f> for Vex2f
{
    fn	from( pt: WPt2f) -> Self
    {
        Vex2f::New2( pt._U, pt._V)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
