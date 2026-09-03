//-- reg.rs -------------------------------------------------------------------------------------------------------------------------
use	std::fmt;
use	std::ops::{ BitAnd, BitOr, BitXor, Not };
use	crate::{
    rube::port::PortType,
    silo::U32,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Unified 16-byte bit-packed register value ( 3-state / 4-state logic).
/// `_Val`: Data bits ( Bool at bit 0, U8 at 0..7, U16 at 0..15, U32 at 0..31, U64 at 0..63).
/// `_X`: Unknown mask bits ( 1 = bit is unknown X, 0 = bit is known valid).
#[derive( Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg
{
    pub _Val: u64,
    pub _X: u64,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type RegVal = Reg;

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Reg
{
    #[inline]
    fn	default() -> Self
    {
        return Self::X;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Reg
{
    pub const TRUE: Self = Self { _Val: 1, _X: 0 };
    pub const FALSE: Self = Self { _Val: 0, _X: 0 };
    pub const X: Self = Self { _Val: 0, _X: 1 };
    pub const X_BOOL: Self = Self { _Val: 0, _X: 1 };
    pub const X_U8: Self = Self { _Val: 0, _X: 0xFF };
    pub const X_U16: Self = Self { _Val: 0, _X: 0xFFFF };
    pub const X_U32: Self = Self { _Val: 0, _X: 0xFFFF_FFFF };
    pub const X_U64: Self = Self { _Val: 0, _X: 0xFFFF_FFFF_FFFF_FFFF };

    #[inline]
    pub const fn	Known( val: u64) -> Self
    {
        return Self { _Val: val, _X: 0 };
    }

    #[inline]
    pub const fn	Unknown( xMask: u64) -> Self
    {
        return Self { _Val: 0, _X: xMask };
    }

    #[inline]
    pub const fn	Val( &self) -> u64
    {
        return self._Val;
    }

    #[inline]
    pub const fn	IsX( &self) -> bool
    {
        return self._X != 0;
    }

    #[inline]
    pub const fn	IsValid( &self) -> bool
    {
        return self._X == 0;
    }

    #[inline]
    pub const fn	IsTrue( &self) -> bool
    {
        return ( self._X & 1) == 0 && ( self._Val & 1) != 0;
    }

    #[inline]
    pub const fn	IsFalse( &self) -> bool
    {
        return ( self._X & 1) == 0 && ( self._Val & 1) == 0;
    }

    #[inline]
    pub const fn	AsBool( &self) -> Self
    {
        if ( self._X & 1) != 0 {
            return Self::X;
        }
        if ( self._Val & 1) != 0 {
            return Self::TRUE;
        }
        return Self::FALSE;
    }

    #[inline]
    pub const fn	GetU32( &self) -> U32
    {
        return U32( ( self._Val & 0xFFFF_FFFF) as u32);
    }

    #[inline]
    pub const fn	Masked( &self, mask: u64) -> Self
    {
        return Self {
            _Val: self._Val & mask,
            _X: self._X & mask,
        };
    }

    #[inline]
    pub const fn	FromBool( val: bool) -> Self
    {
        return if val { Self::TRUE } else { Self::FALSE };
    }

    #[inline]
    pub const fn	FromU32( val: U32) -> Self
    {
        return Self::Known( val.0 as u64);
    }

    #[inline]
    pub fn	DefaultTyped( portType: PortType) -> Self
    {
        return match portType {
            PortType::Bool => Self::FALSE,
            PortType::U8Val | PortType::U16Val | PortType::U32Val | PortType::U64Val | PortType::Custom( _) => {
                Self::Known( 0)
            }
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Not for Reg
{
    type Output = Self;

    #[inline]
    fn	not( self) -> Self::Output
    {
        return Self {
            _Val: ( !self._Val) & ( !self._X),
            _X: self._X,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BitAnd for Reg
{
    type Output = Self;

    #[inline]
    fn	bitand( self, rhs: Self) -> Self::Output
    {
        let  	zeros = ( !self._Val & !self._X) | ( !rhs._Val & !rhs._X);
        let  	ones = ( self._Val & !self._X) & ( rhs._Val & !rhs._X);
        let  	x = !zeros & !ones;
        return Self {
            _Val: ones,
            _X: x,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BitOr for Reg
{
    type Output = Self;

    #[inline]
    fn	bitor( self, rhs: Self) -> Self::Output
    {
        let  	ones = ( self._Val & !self._X) | ( rhs._Val & !rhs._X);
        let  	zeros = ( !self._Val & !self._X) & ( !rhs._Val & !rhs._X);
        let  	x = !zeros & !ones;
        return Self {
            _Val: ones,
            _X: x,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BitXor for Reg
{
    type Output = Self;

    #[inline]
    fn	bitxor( self, rhs: Self) -> Self::Output
    {
        let  	x = self._X | rhs._X;
        let  	val = ( self._Val ^ rhs._Val) & !x;
        return Self {
            _Val: val,
            _X: x,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< bool> for Reg
{
    #[inline]
    fn	from( val: bool) -> Self
    {
        return Self::FromBool( val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< U32> for Reg
{
    #[inline]
    fn	from( val: U32) -> Self
    {
        return Self::FromU32( val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for Reg
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._X != 0 {
            return write!( f, "Reg(Val: 0x{:X}, X: 0x{:X})", self._Val, self._X);
        }
        return write!( f, "Reg(0x{:X})", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for Reg
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._X != 0 {
            return write!( f, "X");
        }
        return write!( f, "0x{:X}", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxSource!( Reg, _Val, _X);
