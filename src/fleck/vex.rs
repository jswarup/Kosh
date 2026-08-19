//-- vex.rs ----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	std::ops::{
    Add, AddAssign, Deref, DerefMut, Div, DivAssign, Index, IndexMut, Mul,
    MulAssign, Neg, Sub, SubAssign,
};
use	crate::silo::{ Buff, U8, U16, U32, U64 };

//---------------------------------------------------------------------------------------------------------------------------------

/// Supertrait defining algebraic and arithmetic capabilities for scalar types.
pub trait IScalar:
    Copy
    + Clone
    + PartialEq
    + PartialOrd
    + Default
    + fmt::Debug
    + fmt::Display
    + Add< Output = Self>
    + Sub< Output = Self>
    + Mul< Output = Self>
    + Div< Output = Self>
    + Neg< Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + 'static
{
    const ZERO: Self;
    const ONE: Self;

    fn	Sqrt( self) -> Self;
    fn	Abs( self) -> Self;
    fn	Cos( self) -> Self;
    fn	Sin( self) -> Self;
    fn	FromF32( val: f32) -> Self;
    fn	ToF64( self) -> f64;
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplScalar {
    ( float: $( $t:ty ),* ) => {
        $(
            impl IScalar for $t
            {
                const ZERO: Self = 0.0;
                const ONE: Self = 1.0;

                #[inline]
                fn	Sqrt( self) -> Self
                {
                    self.sqrt()
                }

                #[inline]
                fn	Abs( self) -> Self
                {
                    self.abs()
                }

                #[inline]
                fn	Cos( self) -> Self
                {
                    self.cos()
                }

                #[inline]
                fn	Sin( self) -> Self
                {
                    self.sin()
                }

                #[inline]
                fn	FromF32( val: f32) -> Self
                {
                    val as $t
                }

                #[inline]
                fn	ToF64( self) -> f64
                {
                    self as f64
                }
            }
        )*
    };
    ( int: $( $t:ty ),* ) => {
        $(
            impl IScalar for $t
            {
                const ZERO: Self = 0;
                const ONE: Self = 1;

                #[inline]
                fn	Sqrt( self) -> Self
                {
                    ( ( self as f64).sqrt()) as $t
                }

                #[inline]
                fn	Abs( self) -> Self
                {
                    self.abs()
                }

                #[inline]
                fn	Cos( self) -> Self
                {
                    ( ( self as f64).cos()) as $t
                }

                #[inline]
                fn	Sin( self) -> Self
                {
                    ( ( self as f64).sin()) as $t
                }

                #[inline]
                fn	FromF32( val: f32) -> Self
                {
                    val as $t
                }

                #[inline]
                fn	ToF64( self) -> f64
                {
                    self as f64
                }
            }
        )*
    };
    ( uint: $( $t:ident ),* ) => {
        $(
            impl IScalar for $t
            {
                const ZERO: Self = $t::_0;
                const ONE: Self = $t::_1;

                #[inline]
                fn	Sqrt( self) -> Self
                {
                    $t( ( ( self.0 as f64).sqrt()) as _)
                }

                #[inline]
                fn	Abs( self) -> Self
                {
                    self
                }

                #[inline]
                fn	Cos( self) -> Self
                {
                    $t( ( ( self.0 as f64).cos()) as _)
                }

                #[inline]
                fn	Sin( self) -> Self
                {
                    $t( ( ( self.0 as f64).sin()) as _)
                }

                #[inline]
                fn	FromF32( val: f32) -> Self
                {
                    $t( val as _)
                }

                #[inline]
                fn	ToF64( self) -> f64
                {
                    self.0 as f64
                }
            }
        )*
    };
}

ImplScalar!( float: f32, f64);
ImplScalar!( int: i8, i16, i32, i64, isize);
ImplScalar!( uint: U8, U16, U32, U64);

//---------------------------------------------------------------------------------------------------------------------------------

/// Supertrait defining linear vector space operations.
pub trait IVectorSpace:
    Sized
    + Clone
    + PartialEq
    + fmt::Debug
    + Add< Output = Self>
    + Sub< Output = Self>
    + Neg< Output = Self>
    + for< 'a> Add< &'a Self, Output = Self>
    + for< 'a> Sub< &'a Self, Output = Self>
    + AddAssign
    + SubAssign
{
    type Scalar: IScalar;

    fn	Dim( &self) -> usize;

    fn	Zero() -> Self;

    fn	IsZero( &self) -> bool;

    fn	Scale( &self, s: Self::Scalar) -> Self;

    fn	ScaleAssign( &mut self, s: Self::Scalar);

    fn	Lerp( &self, other: &Self, t: Self::Scalar) -> Self;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Supertrait extending `IVectorSpace` with Euclidean inner product operations.
pub trait IInnerProductSpace: IVectorSpace
{
    fn	Dot( &self, rhs: &Self) -> Self::Scalar;

    fn	MagnitudeSquared( &self) -> Self::Scalar;

    fn	Magnitude( &self) -> Self::Scalar;

    fn	Normalized( &self) -> Option< Self>;

    fn	DistanceSquared( &self, other: &Self) -> Self::Scalar;

    fn	Distance( &self, other: &Self) -> Self::Scalar;

    fn	Angle( &self, other: &Self) -> Self::Scalar;

    fn	Project( &self, onto: &Self) -> Option< Self>;

    fn	Reject( &self, from: &Self) -> Option< Self>;

    fn	Reflect( &self, normal: &Self) -> Self;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Supertrait for 3D vector spaces supporting the cross product.
pub trait ICrossProduct: IInnerProductSpace
{
    fn	Cross( &self, rhs: &Self) -> Self;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// N-dimensional geometric vector represented as a contiguous stack array.
#[derive( Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr( C)]
pub struct Vex< T, const N: usize>
{
    pub _Data: [T; N],
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type Vex2< T> = Vex< T, 2>;
pub type Vex3< T> = Vex< T, 3>;
pub type Vex4< T> = Vex< T, 4>;

pub type Vex2f = Vex< f32, 2>;
pub type Vex3f = Vex< f32, 3>;
pub type Vex4f = Vex< f32, 4>;

pub type Vex2d = Vex< f64, 2>;
pub type Vex3d = Vex< f64, 3>;
pub type Vex4d = Vex< f64, 4>;

pub type Vex2i = Vex< i32, 2>;
pub type Vex3i = Vex< i32, 3>;
pub type Vex4i = Vex< i32, 4>;

//---------------------------------------------------------------------------------------------------------------------------------

impl< T, const N: usize> Vex< T, N>
{
    pub const fn	New( data: [T; N]) -> Self
    {
        Self { _Data: data }
    }

    pub const fn	FromArray( data: [T; N]) -> Self
    {
        Self { _Data: data }
    }

    pub const fn	AsArray( &self) -> &[T; N]
    {
        &self._Data
    }

    pub fn	AsMutArray( &mut self) -> &mut [T; N]
    {
        &mut self._Data
    }

    pub fn	AsSlice( &self) -> &[T]
    {
        &self._Data
    }

    pub fn	AsMutSlice( &mut self) -> &mut [T]
    {
        &mut self._Data
    }

    pub const fn	Dim() -> usize
    {
        N
    }

    pub fn	Splat( val: T) -> Self
    where
        T: Copy,
    {
        Self { _Data: [val; N] }
    }

    pub fn	Map< U, F>( &self, f: F) -> Vex< U, N>
    where
        T: Copy,
        F: Fn( T) -> U,
        U: Default + Copy,
    {
        let  	mut result = [U::default(); N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = f( self._Data[idx]);
            idx += 1;
        }
        return Vex { _Data: result };
    }

    pub fn	ZipMap< U, R, F>( &self, other: &Vex< U, N>, f: F) -> Vex< R, N>
    where
        T: Copy,
        U: Copy,
        R: Default + Copy,
        F: Fn( T, U) -> R,
    {
        let  	mut result = [R::default(); N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = f( self._Data[idx], other._Data[idx]);
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Vex< T, 2>
{
    pub const fn	New2( x: T, y: T) -> Self
    {
        Self { _Data: [x, y] }
    }

    pub fn	X( &self) -> T
    where
        T: Copy,
    {
        self._Data[0]
    }

    pub fn	Y( &self) -> T
    where
        T: Copy,
    {
        self._Data[1]
    }

    pub fn	SetX( &mut self, val: T)
    {
        self._Data[0] = val;
    }

    pub fn	SetY( &mut self, val: T)
    {
        self._Data[1] = val;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Vex< T, 3>
{
    pub const fn	New3( x: T, y: T, z: T) -> Self
    {
        Self { _Data: [x, y, z] }
    }

    pub fn	X( &self) -> T
    where
        T: Copy,
    {
        self._Data[0]
    }

    pub fn	Y( &self) -> T
    where
        T: Copy,
    {
        self._Data[1]
    }

    pub fn	Z( &self) -> T
    where
        T: Copy,
    {
        self._Data[2]
    }

    pub fn	SetX( &mut self, val: T)
    {
        self._Data[0] = val;
    }

    pub fn	SetY( &mut self, val: T)
    {
        self._Data[1] = val;
    }

    pub fn	SetZ( &mut self, val: T)
    {
        self._Data[2] = val;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Vex< T, 4>
{
    pub const fn	New4( x: T, y: T, z: T, w: T) -> Self
    {
        Self { _Data: [x, y, z, w] }
    }

    pub fn	X( &self) -> T
    where
        T: Copy,
    {
        self._Data[0]
    }

    pub fn	Y( &self) -> T
    where
        T: Copy,
    {
        self._Data[1]
    }

    pub fn	Z( &self) -> T
    where
        T: Copy,
    {
        self._Data[2]
    }

    pub fn	W( &self) -> T
    where
        T: Copy,
    {
        self._Data[3]
    }

    pub fn	SetX( &mut self, val: T)
    {
        self._Data[0] = val;
    }

    pub fn	SetY( &mut self, val: T)
    {
        self._Data[1] = val;
    }

    pub fn	SetZ( &mut self, val: T)
    {
        self._Data[2] = val;
    }

    pub fn	SetW( &mut self, val: T)
    {
        self._Data[3] = val;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IScalar, const N: usize> IVectorSpace for Vex< T, N>
{
    type Scalar = T;

    fn	Dim( &self) -> usize
    {
        N
    }

    fn	Zero() -> Self
    {
        Self { _Data: [T::ZERO; N] }
    }

    fn	IsZero( &self) -> bool
    {
        let  	mut idx = 0;
        while idx < N {
            if self._Data[idx] != T::ZERO {
                return false;
            }
            idx += 1;
        }
        return true;
    }

    fn	Scale( &self, s: Self::Scalar) -> Self
    {
        self * s
    }

    fn	ScaleAssign( &mut self, s: Self::Scalar)
    {
        *self *= s;
    }

    fn	Lerp( &self, other: &Self, t: Self::Scalar) -> Self
    {
        let  	oneMinusT = T::ONE - t;
        return ( self * oneMinusT) + ( other * t);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IScalar, const N: usize> IInnerProductSpace for Vex< T, N>
{
    fn	Dot( &self, rhs: &Self) -> Self::Scalar
    {
        let  	mut sum = T::ZERO;
        let  	mut idx = 0;
        while idx < N {
            sum += self._Data[idx] * rhs._Data[idx];
            idx += 1;
        }
        return sum;
    }

    fn	MagnitudeSquared( &self) -> Self::Scalar
    {
        self.Dot( self)
    }

    fn	Magnitude( &self) -> Self::Scalar
    {
        self.MagnitudeSquared().Sqrt()
    }

    fn	Normalized( &self) -> Option< Self>
    {
        let  	mag = self.Magnitude();
        if mag == T::ZERO {
            return None;
        }
        return Some( self / mag);
    }

    fn	DistanceSquared( &self, other: &Self) -> Self::Scalar
    {
        ( self - other).MagnitudeSquared()
    }

    fn	Distance( &self, other: &Self) -> Self::Scalar
    {
        ( self - other).Magnitude()
    }

    fn	Angle( &self, other: &Self) -> Self::Scalar
    {
        let  	denom = self.Magnitude() * other.Magnitude();
        if denom == T::ZERO {
            return T::ZERO;
        }
        let  	cosTheta = self.Dot( other) / denom;
        let  	clampedCos = if cosTheta > T::ONE {
            T::ONE
        } else if cosTheta < -T::ONE {
            -T::ONE
        } else {
            cosTheta
        };
        let  	rad = ( clampedCos.ToF64()).acos();
        return T::FromF32( rad as f32);
    }

    fn	Project( &self, onto: &Self) -> Option< Self>
    {
        let  	ontoMagSq = onto.MagnitudeSquared();
        if ontoMagSq == T::ZERO {
            return None;
        }
        let  	scaleFactor = self.Dot( onto) / ontoMagSq;
        return Some( onto * scaleFactor);
    }

    fn	Reject( &self, from: &Self) -> Option< Self>
    {
        let  	proj = self.Project( from)?;
        return Some( self - &proj);
    }

    fn	Reflect( &self, normal: &Self) -> Self
    {
        let  	scale = T::FromF32( 2.0) * self.Dot( normal);
        return self - &( normal * scale);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IScalar> ICrossProduct for Vex< T, 3>
{
    fn	Cross( &self, rhs: &Self) -> Self
    {
        let  	cx = ( self._Data[1] * rhs._Data[2]) - ( self._Data[2] * rhs._Data[1]);
        let  	cy = ( self._Data[2] * rhs._Data[0]) - ( self._Data[0] * rhs._Data[2]);
        let  	cz = ( self._Data[0] * rhs._Data[1]) - ( self._Data[1] * rhs._Data[0]);
        return Vex::New3( cx, cy, cz);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Vector + Vector (4 permutations: val+val, ref+ref, ref+val, val+ref)

impl< T: IScalar, const N: usize> Add< Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	add( self, rhs: Vex< T, N>) -> Self::Output
    {
        &self + &rhs
    }
}

impl< 'a, 'b, T: IScalar, const N: usize> Add< &'b Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	add( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        let  	mut result = [T::ZERO; N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = self._Data[idx] + rhs._Data[idx];
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

impl< 'a, T: IScalar, const N: usize> Add< Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	add( self, rhs: Vex< T, N>) -> Self::Output
    {
        self + &rhs
    }
}

impl< 'b, T: IScalar, const N: usize> Add< &'b Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	add( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        &self + rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Vector - Vector (4 permutations: val-val, ref-ref, ref-val, val-ref)

impl< T: IScalar, const N: usize> Sub< Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	sub( self, rhs: Vex< T, N>) -> Self::Output
    {
        &self - &rhs
    }
}

impl< 'a, 'b, T: IScalar, const N: usize> Sub< &'b Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	sub( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        let  	mut result = [T::ZERO; N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = self._Data[idx] - rhs._Data[idx];
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

impl< 'a, T: IScalar, const N: usize> Sub< Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	sub( self, rhs: Vex< T, N>) -> Self::Output
    {
        self - &rhs
    }
}

impl< 'b, T: IScalar, const N: usize> Sub< &'b Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	sub( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        &self - rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Negation (-Vex, -&Vex)

impl< T: IScalar, const N: usize> Neg for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	neg( self) -> Self::Output
    {
        -&self
    }
}

impl< 'a, T: IScalar, const N: usize> Neg for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	neg( self) -> Self::Output
    {
        let  	mut result = [T::ZERO; N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = -self._Data[idx];
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Component-wise Multiplication (Hadamard product)

impl< T: IScalar, const N: usize> Mul< Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: Vex< T, N>) -> Self::Output
    {
        &self * &rhs
    }
}

impl< 'a, 'b, T: IScalar, const N: usize> Mul< &'b Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        let  	mut result = [T::ZERO; N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = self._Data[idx] * rhs._Data[idx];
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

impl< 'a, T: IScalar, const N: usize> Mul< Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: Vex< T, N>) -> Self::Output
    {
        self * &rhs
    }
}

impl< 'b, T: IScalar, const N: usize> Mul< &'b Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        &self * rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Component-wise Division

impl< T: IScalar, const N: usize> Div< Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: Vex< T, N>) -> Self::Output
    {
        &self / &rhs
    }
}

impl< 'a, 'b, T: IScalar, const N: usize> Div< &'b Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        let  	mut result = [T::ZERO; N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = self._Data[idx] / rhs._Data[idx];
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

impl< 'a, T: IScalar, const N: usize> Div< Vex< T, N>> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: Vex< T, N>) -> Self::Output
    {
        self / &rhs
    }
}

impl< 'b, T: IScalar, const N: usize> Div< &'b Vex< T, N>> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: &'b Vex< T, N>) -> Self::Output
    {
        &self / rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Compound Assignments (AddAssign, SubAssign, MulAssign, DivAssign)

impl< T: IScalar, const N: usize> AddAssign< Vex< T, N>> for Vex< T, N>
{
    fn	add_assign( &mut self, rhs: Vex< T, N>)
    {
        *self += &rhs;
    }
}

impl< 'a, T: IScalar, const N: usize> AddAssign< &'a Vex< T, N>> for Vex< T, N>
{
    fn	add_assign( &mut self, rhs: &'a Vex< T, N>)
    {
        let  	mut idx = 0;
        while idx < N {
            self._Data[idx] += rhs._Data[idx];
            idx += 1;
        }
    }
}

impl< T: IScalar, const N: usize> SubAssign< Vex< T, N>> for Vex< T, N>
{
    fn	sub_assign( &mut self, rhs: Vex< T, N>)
    {
        *self -= &rhs;
    }
}

impl< 'a, T: IScalar, const N: usize> SubAssign< &'a Vex< T, N>> for Vex< T, N>
{
    fn	sub_assign( &mut self, rhs: &'a Vex< T, N>)
    {
        let  	mut idx = 0;
        while idx < N {
            self._Data[idx] -= rhs._Data[idx];
            idx += 1;
        }
    }
}

impl< T: IScalar, const N: usize> MulAssign< Vex< T, N>> for Vex< T, N>
{
    fn	mul_assign( &mut self, rhs: Vex< T, N>)
    {
        *self *= &rhs;
    }
}

impl< 'a, T: IScalar, const N: usize> MulAssign< &'a Vex< T, N>> for Vex< T, N>
{
    fn	mul_assign( &mut self, rhs: &'a Vex< T, N>)
    {
        let  	mut idx = 0;
        while idx < N {
            self._Data[idx] *= rhs._Data[idx];
            idx += 1;
        }
    }
}

impl< T: IScalar, const N: usize> DivAssign< Vex< T, N>> for Vex< T, N>
{
    fn	div_assign( &mut self, rhs: Vex< T, N>)
    {
        *self /= &rhs;
    }
}

impl< 'a, T: IScalar, const N: usize> DivAssign< &'a Vex< T, N>> for Vex< T, N>
{
    fn	div_assign( &mut self, rhs: &'a Vex< T, N>)
    {
        let  	mut idx = 0;
        while idx < N {
            self._Data[idx] /= rhs._Data[idx];
            idx += 1;
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Vector * Scalar (Vex * T, &Vex * T, &Vex * &T, Vex * &T)

impl< T: IScalar, const N: usize> Mul< T> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: T) -> Self::Output
    {
        &self * &rhs
    }
}

impl< 'a, T: IScalar, const N: usize> Mul< T> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: T) -> Self::Output
    {
        self * &rhs
    }
}

impl< 'a, 'b, T: IScalar, const N: usize> Mul< &'b T> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: &'b T) -> Self::Output
    {
        let  	mut result = [T::ZERO; N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = self._Data[idx] * *rhs;
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

impl< 'b, T: IScalar, const N: usize> Mul< &'b T> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	mul( self, rhs: &'b T) -> Self::Output
    {
        &self * rhs
    }
}

impl< T: IScalar, const N: usize> MulAssign< T> for Vex< T, N>
{
    fn	mul_assign( &mut self, rhs: T)
    {
        *self *= &rhs;
    }
}

impl< 'a, T: IScalar, const N: usize> MulAssign< &'a T> for Vex< T, N>
{
    fn	mul_assign( &mut self, rhs: &'a T)
    {
        let  	mut idx = 0;
        while idx < N {
            self._Data[idx] *= *rhs;
            idx += 1;
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Vector / Scalar (Vex / T, &Vex / T, &Vex / &T, Vex / &T)

impl< T: IScalar, const N: usize> Div< T> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: T) -> Self::Output
    {
        &self / &rhs
    }
}

impl< 'a, T: IScalar, const N: usize> Div< T> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: T) -> Self::Output
    {
        self / &rhs
    }
}

impl< 'a, 'b, T: IScalar, const N: usize> Div< &'b T> for &'a Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: &'b T) -> Self::Output
    {
        let  	mut result = [T::ZERO; N];
        let  	mut idx = 0;
        while idx < N {
            result[idx] = self._Data[idx] / *rhs;
            idx += 1;
        }
        return Vex { _Data: result };
    }
}

impl< 'b, T: IScalar, const N: usize> Div< &'b T> for Vex< T, N>
{
    type Output = Vex< T, N>;

    fn	div( self, rhs: &'b T) -> Self::Output
    {
        &self / rhs
    }
}

impl< T: IScalar, const N: usize> DivAssign< T> for Vex< T, N>
{
    fn	div_assign( &mut self, rhs: T)
    {
        *self /= &rhs;
    }
}

impl< 'a, T: IScalar, const N: usize> DivAssign< &'a T> for Vex< T, N>
{
    fn	div_assign( &mut self, rhs: &'a T)
    {
        let  	mut idx = 0;
        while idx < N {
            self._Data[idx] /= *rhs;
            idx += 1;
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Scalar * Vector (T * Vex, &T * &Vex, &T * Vex, T * &Vex)

macro_rules! ImplScalarMulVex {
    ( $ScalarType:ty) => {
        impl< const N: usize> Mul< Vex< $ScalarType, N>> for $ScalarType
        {
            type Output = Vex< $ScalarType, N>;

            fn	mul( self, rhs: Vex< $ScalarType, N>) -> Self::Output
            {
                &self * &rhs
            }
        }

        impl< 'a, 'b, const N: usize> Mul< &'b Vex< $ScalarType, N>> for &'a $ScalarType
        {
            type Output = Vex< $ScalarType, N>;

            fn	mul( self, rhs: &'b Vex< $ScalarType, N>) -> Self::Output
            {
                let  	mut result = [<$ScalarType>::ZERO; N];
                let  	mut idx = 0;
                while idx < N {
                    result[idx] = *self * rhs._Data[idx];
                    idx += 1;
                }
                return Vex { _Data: result };
            }
        }

        impl< 'a, const N: usize> Mul< Vex< $ScalarType, N>> for &'a $ScalarType
        {
            type Output = Vex< $ScalarType, N>;

            fn	mul( self, rhs: Vex< $ScalarType, N>) -> Self::Output
            {
                self * &rhs
            }
        }

        impl< 'b, const N: usize> Mul< &'b Vex< $ScalarType, N>> for $ScalarType
        {
            type Output = Vex< $ScalarType, N>;

            fn	mul( self, rhs: &'b Vex< $ScalarType, N>) -> Self::Output
            {
                &self * rhs
            }
        }
    };
}

ImplScalarMulVex!( f32);
ImplScalarMulVex!( f64);
ImplScalarMulVex!( i32);
ImplScalarMulVex!( i64);

//---------------------------------------------------------------------------------------------------------------------------------

// Indexing and Deref Implementations

impl< T, const N: usize> Index< usize> for Vex< T, N>
{
    type Output = T;

    fn	index( &self, index: usize) -> &Self::Output
    {
        &self._Data[index]
    }
}

impl< T, const N: usize> IndexMut< usize> for Vex< T, N>
{
    fn	index_mut( &mut self, index: usize) -> &mut Self::Output
    {
        &mut self._Data[index]
    }
}

impl< T, const N: usize> Index< U32> for Vex< T, N>
{
    type Output = T;

    fn	index( &self, index: U32) -> &Self::Output
    {
        &self._Data[index.AsUsize()]
    }
}

impl< T, const N: usize> IndexMut< U32> for Vex< T, N>
{
    fn	index_mut( &mut self, index: U32) -> &mut Self::Output
    {
        &mut self._Data[index.AsUsize()]
    }
}

impl< T, const N: usize> Index< U64> for Vex< T, N>
{
    type Output = T;

    fn	index( &self, index: U64) -> &Self::Output
    {
        &self._Data[index.AsUsize()]
    }
}

impl< T, const N: usize> IndexMut< U64> for Vex< T, N>
{
    fn	index_mut( &mut self, index: U64) -> &mut Self::Output
    {
        &mut self._Data[index.AsUsize()]
    }
}

impl< T, const N: usize> Deref for Vex< T, N>
{
    type Target = [T; N];

    fn	deref( &self) -> &Self::Target
    {
        &self._Data
    }
}

impl< T, const N: usize> DerefMut for Vex< T, N>
{
    fn	deref_mut( &mut self) -> &mut Self::Target
    {
        &mut self._Data
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Default + Copy, const N: usize> Default for Vex< T, N>
{
    fn	default() -> Self
    {
        Self { _Data: [T::default(); N] }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: fmt::Display, const N: usize> fmt::Display for Vex< T, N>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "[")?;
        let  	mut idx = 0;
        while idx < N {
            if idx > 0 {
                write!( f, ", ")?;
            }
            write!( f, "{}", self._Data[idx])?;
            idx += 1;
        }
        write!( f, "]")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T, const N: usize> From< [T; N]> for Vex< T, N>
{
    fn	from( data: [T; N]) -> Self
    {
        Self { _Data: data }
    }
}

impl< T, const N: usize> From< Vex< T, N>> for [T; N]
{
    fn	from( v: Vex< T, N>) -> Self
    {
        v._Data
    }
}

impl< T> From< ( T, T)> for Vex< T, 2>
{
    fn	from( ( x, y): ( T, T)) -> Self
    {
        Self { _Data: [x, y] }
    }
}

impl< T> From< ( T, T, T)> for Vex< T, 3>
{
    fn	from( ( x, y, z): ( T, T, T)) -> Self
    {
        Self { _Data: [x, y, z] }
    }
}

impl< T> From< ( T, T, T, T)> for Vex< T, 4>
{
    fn	from( ( x, y, z, w): ( T, T, T, T)) -> Self
    {
        Self { _Data: [x, y, z, w] }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Copy, const N: usize> IntoIterator for Vex< T, N>
{
    type Item = T;
    type IntoIter = std::array::IntoIter< T, N>;

    fn	into_iter( self) -> Self::IntoIter
    {
        self._Data.into_iter()
    }
}

impl< 'a, T, const N: usize> IntoIterator for &'a Vex< T, N>
{
    type Item = &'a T;
    type IntoIter = std::slice::Iter< 'a, T>;

    fn	into_iter( self) -> Self::IntoIter
    {
        self._Data.iter()
    }
}

impl< 'a, T, const N: usize> IntoIterator for &'a mut Vex< T, N>
{
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut< 'a, T>;

    fn	into_iter( self) -> Self::IntoIter
    {
        self._Data.iter_mut()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IScalar> IVectorSpace for Buff< T>
{
    type Scalar = T;

    fn	Dim( &self) -> usize
    {
        self.len()
    }

    fn	Zero() -> Self
    {
        Buff::New()
    }

    fn	IsZero( &self) -> bool
    {
        let  	mut idx = 0;
        while idx < self.len() {
            if self[idx] != T::ZERO {
                return false;
            }
            idx += 1;
        }
        return true;
    }

    fn	Scale( &self, s: Self::Scalar) -> Self
    {
        self * s
    }

    fn	ScaleAssign( &mut self, s: Self::Scalar)
    {
        *self *= s;
    }

    fn	Lerp( &self, other: &Self, t: Self::Scalar) -> Self
    {
        assert_eq!( self.len(), other.len(), "Dimension mismatch in Buff Lerp");
        let  	oneMinusT = T::ONE - t;
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = self[idx] * oneMinusT + other[idx] * t;
            idx += 1;
        }
        return result;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IScalar> IInnerProductSpace for Buff< T>
{
    fn	Dot( &self, rhs: &Self) -> Self::Scalar
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff Dot product");
        let  	mut sum = T::ZERO;
        let  	mut idx = 0;
        while idx < self.len() {
            sum += self[idx] * rhs[idx];
            idx += 1;
        }
        return sum;
    }

    fn	MagnitudeSquared( &self) -> Self::Scalar
    {
        self.Dot( self)
    }

    fn	Magnitude( &self) -> Self::Scalar
    {
        self.MagnitudeSquared().Sqrt()
    }

    fn	Normalized( &self) -> Option< Self>
    {
        let  	mag = self.Magnitude();
        if mag == T::ZERO {
            return None;
        }
        return Some( self / mag);
    }

    fn	DistanceSquared( &self, other: &Self) -> Self::Scalar
    {
        ( self - other).MagnitudeSquared()
    }

    fn	Distance( &self, other: &Self) -> Self::Scalar
    {
        ( self - other).Magnitude()
    }

    fn	Angle( &self, other: &Self) -> Self::Scalar
    {
        let  	denom = self.Magnitude() * other.Magnitude();
        if denom == T::ZERO {
            return T::ZERO;
        }
        let  	cosTheta = self.Dot( other) / denom;
        let  	clampedCos = if cosTheta > T::ONE {
            T::ONE
        } else if cosTheta < -T::ONE {
            -T::ONE
        } else {
            cosTheta
        };
        let  	rad = ( clampedCos.ToF64()).acos();
        return T::FromF32( rad as f32);
    }

    fn	Project( &self, onto: &Self) -> Option< Self>
    {
        let  	ontoMagSq = onto.MagnitudeSquared();
        if ontoMagSq == T::ZERO {
            return None;
        }
        let  	scaleFactor = self.Dot( onto) / ontoMagSq;
        return Some( onto * scaleFactor);
    }

    fn	Reject( &self, from: &Self) -> Option< Self>
    {
        let  	proj = self.Project( from)?;
        return Some( self - &proj);
    }

    fn	Reflect( &self, normal: &Self) -> Self
    {
        let  	scale = T::FromF32( 2.0) * self.Dot( normal);
        return self - &( normal * scale);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IScalar> Buff< T>
{
    pub fn	ZeroVec( dim: usize) -> Self
    {
        Buff::Create( dim as u32, |_| T::ZERO)
    }

    pub fn	Splat( val: T, dim: usize) -> Self
    {
        Buff::Create( dim as u32, |_| val)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Buff<T> + Buff<T>

impl< T: IScalar> Add< Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	add( self, rhs: Buff< T>) -> Self::Output
    {
        &self + &rhs
    }
}

impl< 'a, 'b, T: IScalar> Add< &'b Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	add( self, rhs: &'b Buff< T>) -> Self::Output
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff addition");
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = self[idx] + rhs[idx];
            idx += 1;
        }
        return result;
    }
}

impl< 'a, T: IScalar> Add< Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	add( self, rhs: Buff< T>) -> Self::Output
    {
        self + &rhs
    }
}

impl< 'b, T: IScalar> Add< &'b Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	add( self, rhs: &'b Buff< T>) -> Self::Output
    {
        &self + rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Buff<T> - Buff<T>

impl< T: IScalar> Sub< Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	sub( self, rhs: Buff< T>) -> Self::Output
    {
        &self - &rhs
    }
}

impl< 'a, 'b, T: IScalar> Sub< &'b Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	sub( self, rhs: &'b Buff< T>) -> Self::Output
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff subtraction");
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = self[idx] - rhs[idx];
            idx += 1;
        }
        return result;
    }
}

impl< 'a, T: IScalar> Sub< Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	sub( self, rhs: Buff< T>) -> Self::Output
    {
        self - &rhs
    }
}

impl< 'b, T: IScalar> Sub< &'b Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	sub( self, rhs: &'b Buff< T>) -> Self::Output
    {
        &self - rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: -Buff<T>

impl< T: IScalar> Neg for Buff< T>
{
    type Output = Buff< T>;

    fn	neg( self) -> Self::Output
    {
        -&self
    }
}

impl< 'a, T: IScalar> Neg for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	neg( self) -> Self::Output
    {
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = -self[idx];
            idx += 1;
        }
        return result;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Buff<T> * Buff<T> (Hadamard element-wise multiplication)

impl< T: IScalar> Mul< Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: Buff< T>) -> Self::Output
    {
        &self * &rhs
    }
}

impl< 'a, 'b, T: IScalar> Mul< &'b Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: &'b Buff< T>) -> Self::Output
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff Hadamard multiplication");
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = self[idx] * rhs[idx];
            idx += 1;
        }
        return result;
    }
}

impl< 'a, T: IScalar> Mul< Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: Buff< T>) -> Self::Output
    {
        self * &rhs
    }
}

impl< 'b, T: IScalar> Mul< &'b Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: &'b Buff< T>) -> Self::Output
    {
        &self * rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Operator Overloads: Buff<T> / Buff<T> (Hadamard element-wise division)

impl< T: IScalar> Div< Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: Buff< T>) -> Self::Output
    {
        &self / &rhs
    }
}

impl< 'a, 'b, T: IScalar> Div< &'b Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: &'b Buff< T>) -> Self::Output
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff Hadamard division");
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = self[idx] / rhs[idx];
            idx += 1;
        }
        return result;
    }
}

impl< 'a, T: IScalar> Div< Buff< T>> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: Buff< T>) -> Self::Output
    {
        self / &rhs
    }
}

impl< 'b, T: IScalar> Div< &'b Buff< T>> for Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: &'b Buff< T>) -> Self::Output
    {
        &self / rhs
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// In-place Assignment Operators: Buff<T> +=, -=, *=, /= Buff<T>

impl< T: IScalar> AddAssign< Buff< T>> for Buff< T>
{
    fn	add_assign( &mut self, rhs: Buff< T>)
    {
        *self += &rhs;
    }
}

impl< 'a, T: IScalar> AddAssign< &'a Buff< T>> for Buff< T>
{
    fn	add_assign( &mut self, rhs: &'a Buff< T>)
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff AddAssign");
        let  	mut idx = 0;
        while idx < self.len() {
            self[idx] += rhs[idx];
            idx += 1;
        }
    }
}

impl< T: IScalar> SubAssign< Buff< T>> for Buff< T>
{
    fn	sub_assign( &mut self, rhs: Buff< T>)
    {
        *self -= &rhs;
    }
}

impl< 'a, T: IScalar> SubAssign< &'a Buff< T>> for Buff< T>
{
    fn	sub_assign( &mut self, rhs: &'a Buff< T>)
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff SubAssign");
        let  	mut idx = 0;
        while idx < self.len() {
            self[idx] -= rhs[idx];
            idx += 1;
        }
    }
}

impl< T: IScalar> MulAssign< Buff< T>> for Buff< T>
{
    fn	mul_assign( &mut self, rhs: Buff< T>)
    {
        *self *= &rhs;
    }
}

impl< 'a, T: IScalar> MulAssign< &'a Buff< T>> for Buff< T>
{
    fn	mul_assign( &mut self, rhs: &'a Buff< T>)
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff MulAssign");
        let  	mut idx = 0;
        while idx < self.len() {
            self[idx] *= rhs[idx];
            idx += 1;
        }
    }
}

impl< T: IScalar> DivAssign< Buff< T>> for Buff< T>
{
    fn	div_assign( &mut self, rhs: Buff< T>)
    {
        *self /= &rhs;
    }
}

impl< 'a, T: IScalar> DivAssign< &'a Buff< T>> for Buff< T>
{
    fn	div_assign( &mut self, rhs: &'a Buff< T>)
    {
        assert_eq!( self.len(), rhs.len(), "Dimension mismatch in Buff DivAssign");
        let  	mut idx = 0;
        while idx < self.len() {
            self[idx] /= rhs[idx];
            idx += 1;
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Scalar Scaling Operators: Buff<T> * T, Buff<T> / T

impl< T: IScalar> Mul< T> for Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: T) -> Self::Output
    {
        &self * rhs
    }
}

impl< 'a, T: IScalar> Mul< T> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: T) -> Self::Output
    {
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = self[idx] * rhs;
            idx += 1;
        }
        return result;
    }
}

impl< 'a, 'b, T: IScalar> Mul< &'b T> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: &'b T) -> Self::Output
    {
        self * *rhs
    }
}

impl< 'b, T: IScalar> Mul< &'b T> for Buff< T>
{
    type Output = Buff< T>;

    fn	mul( self, rhs: &'b T) -> Self::Output
    {
        &self * *rhs
    }
}

impl< T: IScalar> Div< T> for Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: T) -> Self::Output
    {
        &self / rhs
    }
}

impl< 'a, T: IScalar> Div< T> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: T) -> Self::Output
    {
        let  	mut result = Buff::Create( self.len() as u32, |_| T::ZERO);
        let  	mut idx = 0;
        while idx < self.len() {
            result[idx] = self[idx] / rhs;
            idx += 1;
        }
        return result;
    }
}

impl< 'a, 'b, T: IScalar> Div< &'b T> for &'a Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: &'b T) -> Self::Output
    {
        self / *rhs
    }
}

impl< 'b, T: IScalar> Div< &'b T> for Buff< T>
{
    type Output = Buff< T>;

    fn	div( self, rhs: &'b T) -> Self::Output
    {
        &self / *rhs
    }
}

impl< T: IScalar> MulAssign< T> for Buff< T>
{
    fn	mul_assign( &mut self, rhs: T)
    {
        let  	mut idx = 0;
        while idx < self.len() {
            self[idx] *= rhs;
            idx += 1;
        }
    }
}

impl< 'a, T: IScalar> MulAssign< &'a T> for Buff< T>
{
    fn	mul_assign( &mut self, rhs: &'a T)
    {
        *self *= *rhs;
    }
}

impl< T: IScalar> DivAssign< T> for Buff< T>
{
    fn	div_assign( &mut self, rhs: T)
    {
        let  	mut idx = 0;
        while idx < self.len() {
            self[idx] /= rhs;
            idx += 1;
        }
    }
}

impl< 'a, T: IScalar> DivAssign< &'a T> for Buff< T>
{
    fn	div_assign( &mut self, rhs: &'a T)
    {
        *self /= *rhs;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Explicit free function for dot product computation.
pub fn	Dot< T: IScalar, const N: usize>( a: &Vex< T, N>, b: &Vex< T, N>) -> T
{
    a.Dot( b)
}

/// Explicit free function for 3D cross product computation.
pub fn	Cross< T: IScalar>( a: &Vex< T, 3>, b: &Vex< T, 3>) -> Vex< T, 3>
{
    a.Cross( b)
}

/// Explicit free function for linear interpolation between two vectors.
pub fn	Lerp< T: IScalar, const N: usize>( a: &Vex< T, N>, b: &Vex< T, N>, t: T) -> Vex< T, N>
{
    a.Lerp( b, t)
}

//---------------------------------------------------------------------------------------------------------------------------------
