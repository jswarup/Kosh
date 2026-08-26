//-- regval.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	std::ops::{ BitAnd, BitOr, BitXor, Not };
use	crate::{
    rube::{ port::PortType, reg::Reg },
    silo::{ U16, U32, U64, U8 },
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Unified 16-byte bit-packed 3-state register value ( AoS cell).
/// `_Val`: Data bits ( Bool at bit 0, U8 at 0..7, U16 at 0..15, U32 at 0..31, U64 at 0..63).
/// `_X`: Unknown mask bits ( 1 = bit is unknown X, 0 = bit is known valid).
#[derive( Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RegVal
{
    pub _Val: u64,
    pub _X: u64,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RegVal
{
    pub const TRUE: Self = Self { _Val: 1, _X: 0 };
    pub const FALSE: Self = Self { _Val: 0, _X: 0 };
    pub const X_BOOL: Self = Self { _Val: 0, _X: 1 };
    pub const X_U8: Self = Self { _Val: 0, _X: 0xFF };
    pub const X_U16: Self = Self { _Val: 0, _X: 0xFFFF };
    pub const X_U32: Self = Self { _Val: 0, _X: 0xFFFF_FFFF };
    pub const X_U64: Self = Self { _Val: 0, _X: 0xFFFF_FFFF_FFFF_FFFF };

    #[inline]
    pub const fn	New( val: u64, xMask: u64) -> Self
    {
        return Self {
            _Val: val & !xMask,
            _X: xMask,
        };
    }

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
    pub fn	Val( &self) -> u64
    {
        return self._Val;
    }

    #[inline]
    pub fn	XMask( &self) -> u64
    {
        return self._X;
    }

    #[inline]
    pub fn	IsX( &self) -> bool
    {
        return self._X != 0;
    }

    #[inline]
    pub fn	IsValid( &self) -> bool
    {
        return self._X == 0;
    }

    #[inline]
    pub fn	IsTrue( &self) -> bool
    {
        return ( self._X & 1) == 0 && ( self._Val & 1) != 0;
    }

    #[inline]
    pub fn	IsFalse( &self) -> bool
    {
        return ( self._X & 1) == 0 && ( self._Val & 1) == 0;
    }

    #[inline]
    pub fn	AsBool( &self) -> Reg< bool>
    {
        return if ( self._X & 1) != 0 {
            Reg::X
        } else {
            Reg::FromBool( ( self._Val & 1) != 0)
        };
    }

    #[inline]
    pub fn	AsU8( &self) -> Reg< U8>
    {
        return if ( self._X & 0xFF) != 0 {
            Reg::Unknown( U8( ( self._Val & 0xFF) as u8))
        } else {
            Reg::Known( U8( ( self._Val & 0xFF) as u8))
        };
    }

    #[inline]
    pub fn	AsU16( &self) -> Reg< U16>
    {
        return if ( self._X & 0xFFFF) != 0 {
            Reg::Unknown( U16( ( self._Val & 0xFFFF) as u16))
        } else {
            Reg::Known( U16( ( self._Val & 0xFFFF) as u16))
        };
    }

    #[inline]
    pub fn	AsU32( &self) -> Reg< U32>
    {
        return if ( self._X & 0xFFFF_FFFF) != 0 {
            Reg::Unknown( U32( ( self._Val & 0xFFFF_FFFF) as u32))
        } else {
            Reg::Known( U32( ( self._Val & 0xFFFF_FFFF) as u32))
        };
    }

    #[inline]
    pub fn	AsU64( &self) -> Reg< U64>
    {
        return if self._X != 0 {
            Reg::Unknown( U64( self._Val))
        } else {
            Reg::Known( U64( self._Val))
        };
    }

    #[inline]
    pub fn	FromBool( val: bool) -> Self
    {
        return if val { Self::TRUE } else { Self::FALSE };
    }

    #[inline]
    pub fn	FromRegBool( reg: Reg< bool>) -> Self
    {
        return if reg.IsX() {
            Self::X_BOOL
        } else if *reg.Val() {
            Self::TRUE
        } else {
            Self::FALSE
        };
    }

    #[inline]
    pub fn	FromU8( val: U8) -> Self
    {
        return Self::Known( u64::from( u8::from( val)));
    }

    #[inline]
    pub fn	FromRegU8( reg: Reg< U8>) -> Self
    {
        return if reg.IsX() {
            Self::X_U8
        } else {
            Self::Known( u64::from( u8::from( *reg.Val())))
        };
    }

    #[inline]
    pub fn	FromU16( val: U16) -> Self
    {
        return Self::Known( u64::from( u16::from( val)));
    }

    #[inline]
    pub fn	FromRegU16( reg: Reg< U16>) -> Self
    {
        return if reg.IsX() {
            Self::X_U16
        } else {
            Self::Known( u64::from( u16::from( *reg.Val())))
        };
    }

    #[inline]
    pub fn	FromU32( val: U32) -> Self
    {
        return Self::Known( u64::from( u32::from( val)));
    }

    #[inline]
    pub fn	FromRegU32( reg: Reg< U32>) -> Self
    {
        return if reg.IsX() {
            Self::X_U32
        } else {
            Self::Known( u64::from( u32::from( *reg.Val())))
        };
    }

    #[inline]
    pub fn	FromU64( val: U64) -> Self
    {
        return Self::Known( u64::from( val));
    }

    #[inline]
    pub fn	FromRegU64( reg: Reg< U64>) -> Self
    {
        return if reg.IsX() {
            Self::X_U64
        } else {
            Self::Known( u64::from( *reg.Val()))
        };
    }

    #[inline]
    pub fn	DefaultTyped( portType: PortType) -> Self
    {
        return match portType {
            PortType::Bool => Self::FALSE,
            PortType::U8Val => Self::FromU8( U8( 0)),
            PortType::U16Val => Self::FromU16( U16( 0)),
            PortType::U32Val => Self::FromU32( U32( 0)),
            PortType::U64Val => Self::FromU64( U64( 0)),
            PortType::Custom( _) => Self::Known( 0),
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Not for RegVal
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

impl BitAnd for RegVal
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

impl BitOr for RegVal
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

impl BitXor for RegVal
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

impl fmt::Debug for RegVal
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._X != 0 {
            return write!( f, "RegVal(Val: 0x{:X}, X: 0x{:X})", self._Val, self._X);
        }
        return write!( f, "RegVal(0x{:X})", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for RegVal
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
