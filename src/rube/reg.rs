//-- reg.rs -------------------------------------------------------------------------------------------------------------------------
/// Register representation with generic value and unknown ( X) state flag.
///
/// Encoding:
/// - `_Val`: Underlying value ( e.g. `bool`, `U32`, numeric or bit vector)
/// - `_X`: `true` indicates unknown/uninitialized ( X state); `false` indicates valid/known value.

use	std::fmt;
use	std::ops::{ BitAnd, BitOr, BitXor, Not };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg< Val = bool>
{
    pub _Val: Val,
    pub _X: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val: Default> Default for Reg< Val>
{
    /// Default initialization is X ( unknown).
    #[inline]
    fn	default() -> Self
    {
        Self {
            _Val: Val::default(),
            _X: true,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IReg< Val>
{
    fn	Val( &self) -> &Val;
    fn	IsX( &self) -> bool;
    fn	IsValid( &self) -> bool;
    fn	ConvertX( &mut self);
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val> Reg< Val>
{
    #[inline]
    pub const fn	New( val: Val, x: bool) -> Self
    {
        Self { _Val: val, _X: x }
    }

    #[inline]
    pub const fn	Known( val: Val) -> Self
    {
        Self { _Val: val, _X: false }
    }

    #[inline]
    pub const fn	Unknown( val: Val) -> Self
    {
        Self { _Val: val, _X: true }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val> IReg< Val> for Reg< Val>
{
    #[inline]
    fn	Val( &self) -> &Val
    {
        return &self._Val;
    }

    #[inline]
    fn	IsX( &self) -> bool
    {
        return self._X;
    }

    #[inline]
    fn	IsValid( &self) -> bool
    {
        return !self._X;
    }

    #[inline]
    fn	ConvertX( &mut self)
    {
        self._X = true;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IRegBool
{
    fn	IsFalse( &self) -> bool;
    fn	IsTrue( &self) -> bool;
    fn	GetBool( &self) -> bool;
    fn	ToChar( &self) -> char;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Reg< bool>
{
    pub const FALSE: Self = Self { _Val: false, _X: false };
    pub const TRUE: Self = Self { _Val: true, _X: false };
    pub const X: Self = Self { _Val: false, _X: true };

    #[inline]
    pub const fn	FromBool( val: bool) -> Self
    {
        if val {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }

    #[inline]
    pub fn	FromChar( c: char) -> Option< Self>
    {
        match c {
            '0' => Some( Self::FALSE),
            '1' => Some( Self::TRUE),
            'x' | 'X' => Some( Self::X),
            _ => None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IRegBool for Reg< bool>
{
    #[inline]
    fn	IsFalse( &self) -> bool
    {
        return !self._X && !self._Val;
    }

    #[inline]
    fn	IsTrue( &self) -> bool
    {
        return !self._X && self._Val;
    }

    #[inline]
    fn	GetBool( &self) -> bool
    {
        return self._Val;
    }

    #[inline]
    fn	ToChar( &self) -> char
    {
        if self._X {
            return 'X';
        }
        if self._Val {
            return '1';
        }
        return '0';
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Not for Reg< bool>
{
    type Output = Self;

    #[inline]
    fn	not( self) -> Self::Output
    {
        if self._X {
            return Self::X;
        }
        return Self::New( !self._Val, false);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BitAnd for Reg< bool>
{
    type Output = Self;

    #[inline]
    fn	bitand( self, rhs: Self) -> Self::Output
    {
        if ( !self._X && !self._Val) || ( !rhs._X && !rhs._Val) {
            return Self::FALSE;
        }
        if self._X || rhs._X {
            return Self::X;
        }
        return Self::New( self._Val && rhs._Val, false);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BitOr for Reg< bool>
{
    type Output = Self;

    #[inline]
    fn	bitor( self, rhs: Self) -> Self::Output
    {
        if ( !self._X && self._Val) || ( !rhs._X && rhs._Val) {
            return Self::TRUE;
        }
        if self._X || rhs._X {
            return Self::X;
        }
        return Self::New( self._Val || rhs._Val, false);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BitXor for Reg< bool>
{
    type Output = Self;

    #[inline]
    fn	bitxor( self, rhs: Self) -> Self::Output
    {
        if self._X || rhs._X {
            return Self::X;
        }
        return Self::New( self._Val ^ rhs._Val, false);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val: fmt::Debug> fmt::Debug for Reg< Val>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._X {
            return write!( f, "X");
        }
        return write!( f, "{:?}", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val: fmt::Display> fmt::Display for Reg< Val>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._X {
            return write!( f, "X");
        }
        return write!( f, "{}", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val> From< Val> for Reg< Val>
{
    #[inline]
    fn	from( val: Val) -> Self
    {
        return Self { _Val: val, _X: false };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
