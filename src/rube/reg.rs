/// 4-State logic value representation ( 0, 1, X, Z) and edge sensitivity.
///
/// Direct Rust equivalent of `Ax_Regval` / `Fr_Regval` / `Fr_Bool`.
/// Encoding:
/// - a=0, b=0 -> False ( 0)
/// - a=1, b=0 -> True ( 1)
/// - a=1, b=1 -> X ( Unknown)
/// - a=0, b=1 -> Z ( High-Z)

use	std::fmt;
use	std::ops::{BitAnd, BitOr, BitXor, Not};

#[derive( Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg
{
    pub _A: bool,
    pub _B: bool,
}

impl Default for Reg
{
    /// Default initialization in Ferris is X ( a=true, b=true).
    #[inline]
    fn	default() -> Self
{
        Self::X
    }
}

impl Reg
{
    pub const FALSE: Self = Self { _A: false, _B: false };
    pub const TRUE: Self = Self { _A: true, _B: false };
    pub const X: Self = Self { _A: true, _B: true };
    pub const Z: Self = Self { _A: false, _B: true };

    #[inline]
    pub const fn	new( a: bool, b: bool) -> Self
{
        Self { _A: a, _B: b }
    }

    #[inline]
    pub const fn	from_bool( val: bool) -> Self
{
        if val {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }

    #[inline]
    pub fn	is_false( &self) -> bool
{
        !self._A && !self._B
    }

    #[inline]
    pub fn	is_true( &self) -> bool
{
        self._A && !self._B
    }

    #[inline]
    pub fn	is_x( &self) -> bool
{
        self._A && self._B
    }

    #[inline]
    pub fn	is_z( &self) -> bool
{
        !self._A && self._B
    }

    #[inline]
    pub fn	is_valid( &self) -> bool
{
        !self._B
    }

    #[inline]
    pub fn	get_bool( &self) -> bool
{
        self._A
    }

    #[inline]
    pub fn	convert_x( &mut self)
{
        self._A = true;
        self._B = true;
    }

    #[inline]
    pub fn	convert_z( &mut self)
{
        self._A = false;
        self._B = true;
    }

    #[inline]
    pub fn	to_char( &self) -> char
{
        if !self._B {
            if self._A { '1' } else { '0' }
        } else if self._A {
            'X'
        } else {
            'Z'
        }
    }

    pub fn	from_char( c: char) -> Option< Self>
{
        match c {
            '0' => Some( Self::FALSE),
            '1' => Some( Self::TRUE),
            'x' | 'X' => Some( Self::X),
            'z' | 'Z' => Some( Self::Z),
            _ => None,
        }
    }
}

impl Not for Reg
{
    type Output = Self;

    #[inline]
    fn	not( self) -> Self::Output
{
        if !self._B {
            Self::new( !self._A, false)
        } else {
            Self::new( self._A, true)
        }
    }
}

impl BitAnd for Reg
{
    type Output = Self;

    #[inline]
    fn	bitand( self, rhs: Self) -> Self::Output
{
        if self.is_z() || rhs.is_z() {
            return Self::Z;
        }
        if self.is_x() && rhs.is_x() {
            return Self::X;
        }
        // In 4-state logic: 0 & X = 0, X & 0 = 0
        if self.is_x() && !rhs._A && !rhs._B {
            return rhs; // 0
        }
        if rhs.is_x() && !self._A && !self._B {
            return self; // 0
        }
        if self.is_x() || rhs.is_x() {
            return Self::X;
        }
        Self::new( self._A && rhs._A, false)
    }
}

impl BitOr for Reg
{
    type Output = Self;

    #[inline]
    fn	bitor( self, rhs: Self) -> Self::Output
{
        if self.is_z() || rhs.is_z() {
            return Self::Z;
        }
        if self.is_x() && rhs.is_x() {
            return Self::X;
        }
        // In 4-state logic: 1 | X = 1, X | 1 = 1
        if self.is_x() && rhs._A && !rhs._B {
            return rhs; // 1
        }
        if rhs.is_x() && self._A && !self._B {
            return self; // 1
        }
        if self.is_x() || rhs.is_x() {
            return Self::X;
        }
        Self::new( self._A || rhs._A, false)
    }
}

impl BitXor for Reg
{
    type Output = Self;

    #[inline]
    fn	bitxor( self, rhs: Self) -> Self::Output
{
        if self.is_x() || self.is_z() || rhs.is_x() || rhs.is_z() {
            Self::X
        } else {
            Self::new( self._A ^ rhs._A, false)
        }
    }
}

impl fmt::Debug for Reg
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
{
        write!( f, "{}", self.to_char())
    }
}

impl fmt::Display for Reg
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
{
        write!( f, "{}", self.to_char())
    }
}

impl From< bool> for Reg
{
    #[inline]
    fn	from( val: bool) -> Self
{
        Self::from_bool( val)
    }
}
