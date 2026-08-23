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

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents an axis-aligned 3D bounding box with 32-bit floating-point coordinates.
#[derive( Clone, Copy, Debug, PartialEq)]
pub struct BBox3f
{
    pub _Min: Pt3f,
    pub _Max: Pt3f,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for BBox3f
{
    fn	default() -> Self
    {
        Self::Empty()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BBox3f
{
    pub fn	New( min: Pt3f, max: Pt3f) -> Self
    {
        Self {
            _Min: min,
            _Max: max,
        }
    }

    pub fn	Empty() -> Self
    {
        Self {
            _Min: Pt3f::New( f32::MAX, f32::MAX, f32::MAX),
            _Max: Pt3f::New( f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn	FromPoints( points: &[ [f32; 3]]) -> Self
    {
        let  	mut bbox = Self::Empty();
        for &p in points {
            bbox.Extend( Pt3f::from( p));
        }
        if bbox.IsEmpty() {
            bbox = Self::New( Pt3f::New( -50.0, -50.0, -50.0), Pt3f::New( 50.0, 50.0, 50.0));
        }
        return bbox;
    }

    pub fn	Extend( &mut self, pt: Pt3f)
    {
        self._Min._X = self._Min._X.min( pt._X);
        self._Min._Y = self._Min._Y.min( pt._Y);
        self._Min._Z = self._Min._Z.min( pt._Z);
        self._Max._X = self._Max._X.max( pt._X);
        self._Max._Y = self._Max._Y.max( pt._Y);
        self._Max._Z = self._Max._Z.max( pt._Z);
    }

    pub fn	IsEmpty( &self) -> bool
    {
        self._Min._X > self._Max._X || self._Min._Y > self._Max._Y || self._Min._Z > self._Max._Z
    }

    pub fn	Center( &self) -> Pt3f
    {
        Pt3f::New(
            ( self._Min._X + self._Max._X) * 0.5,
            ( self._Min._Y + self._Max._Y) * 0.5,
            ( self._Min._Z + self._Max._Z) * 0.5,
        )
    }

    pub fn	Extent( &self) -> Pt3f
    {
        Pt3f::New(
            self._Max._X - self._Min._X,
            self._Max._Y - self._Min._Y,
            self._Max._Z - self._Min._Z,
        )
    }

    pub fn	MaxDim( &self) -> f32
    {
        let  	ext = self.Extent();
        ext._X.max( ext._Y).max( ext._Z)
    }

    pub fn	ScaleNorm( &self, targetExtent: f32) -> f32
    {
        let  	maxDim = self.MaxDim();
        if maxDim > 1e-4 {
            targetExtent / maxDim
        } else {
            1.0
        }
    }

    pub fn	Corners( &self) -> [Pt3f; 8]
    {
        [
            Pt3f::New( self._Min._X, self._Min._Y, self._Min._Z),
            Pt3f::New( self._Max._X, self._Min._Y, self._Min._Z),
            Pt3f::New( self._Max._X, self._Max._Y, self._Min._Z),
            Pt3f::New( self._Min._X, self._Max._Y, self._Min._Z),
            Pt3f::New( self._Min._X, self._Min._Y, self._Max._Z),
            Pt3f::New( self._Max._X, self._Min._Y, self._Max._Z),
            Pt3f::New( self._Max._X, self._Max._Y, self._Max._Z),
            Pt3f::New( self._Min._X, self._Max._Y, self._Max._Z),
        ]
    }

    pub const fn	BoxEdges() -> [( usize, usize); 12]
    {
        [
            ( 0, 1), ( 1, 2), ( 2, 3), ( 3, 0),
            ( 4, 5), ( 5, 6), ( 6, 7), ( 7, 4),
            ( 0, 4), ( 1, 5), ( 2, 6), ( 3, 7),
        ]
    }

    pub fn	Min( &self) -> [f32; 3]
    {
        self._Min.Pos()
    }

    pub fn	Max( &self) -> [f32; 3]
    {
        self._Max.Pos()
    }
}


//---------------------------------------------------------------------------------------------------------------------------------
