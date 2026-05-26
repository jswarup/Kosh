//-- uint32.rs -------------------------------------------------------------------------------------------------------------------------
//!  a thin wrapper around `u32` providing seamless integer operations.

use std::ops::{Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Not, Shl, Shr, Neg, Deref};
use std::sync::atomic::{AtomicU32, Ordering};

//---------------------------------------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct U32(pub u32);

impl U32 {
    /// Create a `U32` from a primitive `u32` (inherent method).
    #[inline]
    pub const fn from(v: u32) -> Self {
        U32(v)
    }
    /// Create a `U32` from a primitive `u32`.
    #[inline]
    pub const fn from_u32(v: u32) -> Self {
        U32(v)
    }

    /// Get the inner `u32` value.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Get the inner `usize` value.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Maximum value for `U32`.
    #[inline]
    pub const fn Max() -> Self {
        U32(u32::MAX)
    }

    pub const fn Get(self) -> u32 {
        self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// Implement common arithmetic operations between `U32` and any type that can be converted into `U32`.
// This provides seamless usage like `a + b` where `a` and `b` may be `u32`, `i32`, or `usize`.

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T> $trait<T> for U32
        where
            T: Into<U32>,
        {
            type Output = U32;
            #[inline]
            fn $method(self, rhs: T) -> Self::Output {
                U32(self.0 $op rhs.into().0)
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Add for U32
{
    type Output = U32;
    fn add(self, rhs: U32) -> Self::Output {
        U32(self.0.wrapping_add( rhs.0))
    }
}

impl Sub for U32
{
    type Output = U32;
    fn sub(self, rhs: U32) -> Self::Output {
        U32(self.0.wrapping_sub( rhs.0))
    }
}

impl_op!(Mul, mul, *);
impl_op!(Div, div, /);
impl_op!(Rem, rem, %);
impl_op!(BitAnd, bitand, &);
impl_op!(BitOr, bitor, |);
impl_op!(BitXor, bitxor, ^);
impl_op!(Shl, shl, <<);
impl_op!(Shr, shr, >>);

//---------------------------------------------------------------------------------------------------------------------------------
// Unary `-` (negation) for unsigned values is defined as wrapping subtraction from zero.
impl Neg for U32 {
    type Output = U32;
    #[inline]
    fn neg(self) -> Self::Output {
        U32(0u32.wrapping_sub(self.0))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Not for U32 {
    type Output = U32;
    #[inline]
    fn not(self) -> Self::Output {
        U32(!self.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Display & formatting
impl std::fmt::Display for U32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// Allow `U32` to be used where `u32` is expected via `Deref`.

impl Deref for U32 {
    type Target = u32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

use crate::silo::atm::AtomicInt;

impl AtomicInt for U32 {
    type AtomicType = AtomicU32;
    fn IntoAtomic(self) -> Self::AtomicType { AtomicU32::new(self.0) }
    fn Get(a: &Self::AtomicType, order: Ordering) -> Self { U32(a.load(order)) }
    fn Set(a: &Self::AtomicType, val: Self, order: Ordering) { a.store(val.0, order); }
    fn FetchAdd(a: &Self::AtomicType, val: Self, order: Ordering) -> Self { U32(a.fetch_add(val.0, order)) }
    fn CompareExchange(a: &Self::AtomicType, current: Self, newVal: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
        a.compare_exchange(current.0, newVal.0, success, failure).map(U32).map_err(U32)
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

impl From<U32> for u32 {
    #[inline]
    fn from(v: U32) -> Self { v.0 }
}

impl From<U32> for i32 {
    #[inline]
    fn from(v: U32) -> Self { v.0 as i32 }
}

impl From<U32> for usize {
    #[inline]
    fn from(v: U32) -> Self { v.0 as usize }
}
// Additional convenience implementations for testing and interoperability
impl From<u32> for U32 {
    #[inline]
    fn from(v: u32) -> Self { U32(v) }
}
impl From<i32> for U32 {
    #[inline]
    fn from(v: i32) -> Self { U32(v as u32) }
}
impl From<usize> for U32 {
    #[inline]
    fn from(v: usize) -> Self { U32(v as u32) }
}
impl PartialEq<u32> for U32 {
    fn eq(&self, other: &u32) -> bool { self.0 == *other }
}
impl PartialEq<U32> for u32 {
    fn eq(&self, other: &U32) -> bool { *self == other.0 }
}
impl PartialOrd<u32> for U32 {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> { self.0.partial_cmp(other) }
}
impl PartialOrd<U32> for u32 {
    fn partial_cmp(&self, other: &U32) -> Option<std::cmp::Ordering> { self.partial_cmp(&other.0) }
}
