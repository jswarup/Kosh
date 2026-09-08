//-- reg.rs -------------------------------------------------------------------------------------------------------------------------
use	std::fmt;
use	std::ops::{ BitAnd, BitOr, BitXor, Not };
use	crate::{
    rube::port::PortType,
    silo::{ U32, U64 },
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Unified bit-packed register value with 4-state logic ( 0, 1, X, Z/I).
/// `_Val`: Data bits as U64.
/// `_X`: Unknown boolean flag ( true = register value is unknown X).
/// `_I`: High-Impedance boolean flag ( true = register value is high-impedance Z/I).
#[derive( Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg
{
    pub _Val: U64,
    pub _X:   bool,
    pub _I:   bool,
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
    pub const TRUE:   Self = Self { _Val: U64::_1, _X: false, _I: false };
    pub const FALSE:  Self = Self { _Val: U64::_0, _X: false, _I: false };
    pub const X:      Self = Self { _Val: U64::_0, _X: true,  _I: false };
    pub const Z:      Self = Self { _Val: U64::_0, _X: false, _I: true  };
    pub const I:      Self = Self::Z;
    pub const HIGH_Z: Self = Self::Z;

    pub const X_BOOL: Self = Self::X;
    pub const X_U8:   Self = Self::X;
    pub const X_U16:  Self = Self::X;
    pub const X_U32:  Self = Self::X;
    pub const X_U64:  Self = Self::X;

    pub const Z_BOOL: Self = Self::Z;
    pub const Z_U8:   Self = Self::Z;
    pub const Z_U16:  Self = Self::Z;
    pub const Z_U32:  Self = Self::Z;
    pub const Z_U64:  Self = Self::Z;

    #[inline]
    pub const fn	Known( val: u64) -> Self
    {
        return Self { _Val: U64( val), _X: false, _I: false };
    }

    #[inline]
    pub const fn	Unknown( _dummy: u64) -> Self
    {
        return Self::X;
    }

    #[inline]
    pub const fn	HighZ( _dummy: u64) -> Self
    {
        return Self::Z;
    }

    #[inline]
    pub const fn	Val( &self) -> u64
    {
        return self._Val.0;
    }

    #[inline]
    pub const fn	GetU64( &self) -> U64
    {
        return self._Val;
    }

    #[inline]
    pub const fn	IsX( &self) -> bool
    {
        return self._X;
    }

    #[inline]
    pub const fn	IsZ( &self) -> bool
    {
        return self._I;
    }

    #[inline]
    pub const fn	IsI( &self) -> bool
    {
        return self._I;
    }

    #[inline]
    pub const fn	IsValid( &self) -> bool
    {
        return !self._X && !self._I;
    }

    #[inline]
    pub const fn	IsTrue( &self) -> bool
    {
        return !self._X && !self._I && ( self._Val.0 & 1) != 0;
    }

    #[inline]
    pub const fn	IsFalse( &self) -> bool
    {
        return !self._X && !self._I && ( self._Val.0 & 1) == 0;
    }

    #[inline]
    pub const fn	AsBool( &self) -> Self
    {
        if self._I {
            return Self::Z;
        }
        if self._X {
            return Self::X;
        }
        if ( self._Val.0 & 1) != 0 {
            return Self::TRUE;
        }
        return Self::FALSE;
    }

    #[inline]
    pub const fn	GetU32( &self) -> U32
    {
        return U32( ( self._Val.0 & 0xFFFF_FFFF) as u32);
    }

    #[inline]
    pub const fn	Masked( &self, mask: u64) -> Self
    {
        return Self {
            _Val: U64( self._Val.0 & mask),
            _X:   self._X,
            _I:   self._I,
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
    pub const fn	FromU64( val: U64) -> Self
    {
        return Self { _Val: val, _X: false, _I: false };
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
        if self._X || self._I {
            return Self::X;
        }
        if self._Val == U64::_1 {
            return Self::FALSE;
        }
        if self._Val == U64::_0 {
            return Self::TRUE;
        }
        return Self {
            _Val: !self._Val,
            _X:   false,
            _I:   false,
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
        if self.IsFalse() || rhs.IsFalse() {
            return Self::FALSE;
        }
        if self.IsX() || self.IsI() || rhs.IsX() || rhs.IsI() {
            return Self::X;
        }
        return Self {
            _Val: self._Val & rhs._Val,
            _X:   false,
            _I:   false,
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
        if self.IsTrue() || rhs.IsTrue() {
            return Self::TRUE;
        }
        if self.IsX() || self.IsI() || rhs.IsX() || rhs.IsI() {
            return Self::X;
        }
        return Self {
            _Val: self._Val | rhs._Val,
            _X:   false,
            _I:   false,
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
        if self.IsX() || self.IsI() || rhs.IsX() || rhs.IsI() {
            return Self::X;
        }
        return Self {
            _Val: self._Val ^ rhs._Val,
            _X:   false,
            _I:   false,
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

impl From< U64> for Reg
{
    #[inline]
    fn	from( val: U64) -> Self
    {
        return Self::FromU64( val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for Reg
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._I {
            return write!( f, "Reg(Z)");
        }
        if self._X {
            return write!( f, "Reg(X)");
        }
        return write!( f, "Reg(0x{:X})", self._Val.0);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for Reg
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._I {
            return write!( f, "Z");
        }
        if self._X {
            return write!( f, "X");
        }
        return write!( f, "0x{:X}", self._Val.0);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxSource!( Reg, _Val, _X, _I);
