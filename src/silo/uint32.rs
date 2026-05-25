//-- uint32.rs -------------------------------------------------------------------------------------------------------------------------
//!  a thin wrapper around `u32` providing seamless integer operations.

use std::ops::{Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Not, Shl, Shr, Neg, Deref};
use std::sync::atomic::{AtomicU32, Ordering};

//---------------------------------------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct UInt32(pub u32);

impl UInt32 {
    /// Returns the inner value.
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Performs a wrapping subtraction, returning a new `UInt32`.
    #[inline]
    pub const fn wrapping_sub(self, other: UInt32) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Conversions
impl From<u32> for UInt32 {
    #[inline]
    fn from(v: u32) -> Self {
        Self(v)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From<i32> for UInt32 {
    #[inline]
    fn from(v: i32) -> Self {
        Self(v as u32)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From<usize> for UInt32 {
    #[inline]
    fn from(v: usize) -> Self {
        Self(v as u32)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// Implement common arithmetic operations between `UInt32` and any type that can be
// converted into `UInt32`. This provides seamless usage like `a + b` where `a`
// and `b` may be `u32`, `i32`, or `usize`.

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T> $trait<T> for UInt32
        where
            T: Into<UInt32>,
        {
            type Output = UInt32;
            #[inline]
            fn $method(self, rhs: T) -> Self::Output {
                UInt32(self.0 $op rhs.into().0)
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
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
impl Neg for UInt32 {
    type Output = UInt32;
    #[inline]
    fn neg(self) -> Self::Output {
        UInt32(0u32.wrapping_sub(self.0))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Not for UInt32 {
    type Output = UInt32;
    #[inline]
    fn not(self) -> Self::Output {
        UInt32(!self.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Display & formatting
impl std::fmt::Display for UInt32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// Allow `UInt32` to be used where `u32` is expected via `Deref`.

impl Deref for UInt32 {
    type Target = u32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// Optional: Implement `From<UInt32>` for `u32` for explicit conversion.

impl From<UInt32> for u32 {
    #[inline]
    fn from(v: UInt32) -> Self {
        v.0
    }
}

use crate::silo::atm::AtomicInt;

impl AtomicInt for UInt32 {
    type AtomicType = AtomicU32;
    fn IntoAtomic(self) -> Self::AtomicType { AtomicU32::new(self.0) }
    fn Get(a: &Self::AtomicType, order: Ordering) -> Self { UInt32(a.load(order)) }
    fn Set(a: &Self::AtomicType, val: Self, order: Ordering) { a.store(val.0, order); }
    fn FetchAdd(a: &Self::AtomicType, val: Self, order: Ordering) -> Self { UInt32(a.fetch_add(val.0, order)) }
    fn CompareExchange(a: &Self::AtomicType, current: Self, newVal: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
        a.compare_exchange(current.0, newVal.0, success, failure).map(UInt32).map_err(UInt32)
    }
}


//---------------------------------------------------------------------------------------------------------------------------------
