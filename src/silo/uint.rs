//-- uint.rs -------------------------------------------------------------------------------------------------------------------------
//!  thin wrappers around unsigned integers providing seamless operations.
use	crate::stalks::atm::AtomicInt;
use	std::ops::{ Add, BitAnd, BitOr, BitXor, Deref, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub };
use	std::sync::atomic::{ AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr( transparent)]
pub struct U8( pub u8);
impl U8 
{
    pub const _X: Self = U8( u8::MAX);
    pub const _0: Self = U8( 0u8);
    pub const _1: Self = U8( 1u8);

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Create a `U8` from a primitive `u8` ( inherent method).
    #[inline]
    pub const fn	from( v: u8) -> Self 
    {
        U8( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Create a `U8` from a primitive `u8`.
    #[inline]
    pub const fn	from_u8( v: u8) -> Self 
    {
        U8( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `u8` value.
    #[inline]
    pub const fn	as_u8( self) -> u8 
    {
        self.0
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `usize` value.
    #[inline]
    pub const fn	as_usize( self) -> usize 
    {
        self.0 as usize
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Maximum value for `U8`.
    #[inline]
    pub const fn	Max() -> Self 
    {
        U8( u8::MAX)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	Get( self) -> u8 
    {
        self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr( transparent)]
pub struct U32( pub u32);
impl U32 
{
    pub const _X: Self = U32( u32::MAX);
    pub const _0: Self = U32( 0u32);
    pub const _1: Self = U32( 1u32);
    pub const _16Sz: Self = U32( 1 << 16);

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Create a `U32` from a primitive `u32` ( inherent method).
    #[inline]
    pub const fn	from( v: u32) -> Self 
    {
        U32( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub const fn	from_u32( v: u32) -> Self 
    {
        U32( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_usize( v: usize) -> Self 
    {
        U32( v as u32)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_U8( v: U8) -> Self 
    {
        U32( v.0 as u32)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_U16( v: U16) -> Self 
    {
        U32( v.0 as u32)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `u32` value.
    #[inline]
    pub const fn	as_u32( self) -> u32 
    {
        self.0
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `usize` value.
    #[inline]
    pub const fn	as_usize( self) -> usize 
    {
        self.0 as usize
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	Get( self) -> u32 
    {
        self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr( transparent)]
pub struct U16( pub u16);
impl U16 
{
    pub const _X: Self = U16( u16::MAX);
    pub const _0: Self = U16( 0u16);
    pub const _1: Self = U16( 1u16);

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Create a `U16` from a primitive `u16` ( inherent method).
    #[inline]
    pub const fn	from( v: u16) -> Self 
    {
        U16( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Create a `U16` from a primitive `u16`.
    #[inline]
    pub const fn	from_u16( v: u16) -> Self 
    {
        U16( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_U8( v: U8) -> Self 
    {
        U16( v.0 as u16)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `u16` value.
    #[inline]
    pub const fn	as_u16( self) -> u16 
    {
        self.0
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `usize` value.
    #[inline]
    pub const fn	as_usize( self) -> usize 
    {
        self.0 as usize
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Maximum value for `U16`.
    #[inline]
    pub const fn	Max() -> Self 
    {
        U16( u16::MAX)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	Get( self) -> u16 
    {
        self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr( transparent)]
pub struct U64( pub u64);
impl U64 
{
    pub const _X: Self = U64( u64::MAX);
    pub const _0: Self = U64( 0u64);
    pub const _1: Self = U64( 1u64);

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Create a `U64` from a primitive `u64` ( inherent method).
    #[inline]
    pub const fn	from( v: u64) -> Self 
    {
        U64( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Create a `U64` from a primitive `u64`.
    #[inline]
    pub const fn	from_u64( v: u64) -> Self 
    {
        U64( v)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_usize( v: usize) -> Self 
    {
        U64( v as u64)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_U8( v: U8) -> Self 
    {
        U64( v.0 as u64)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_U16( v: U16) -> Self 
    {
        U64( v.0 as u64)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	from_U32( v: U32) -> Self 
    {
        U64( v.0 as u64)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `u64` value.
    #[inline]
    pub const fn	as_u64( self) -> u64 
    {
        self.0
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Get the inner `usize` value.
    #[inline]
    pub const fn	as_usize( self) -> usize 
    {
        self.0 as usize
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Maximum value for `U64`.
    #[inline]
    pub const fn	Max() -> Self 
    {
        U64( u64::MAX)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	Get( self) -> u64 
    {
        self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// Implement common arithmetic operations between the wrapper and any type that can be converted into the wrapper.
// This provides seamless usage like `a + b` where `a` and `b` may be primitives or wrappers.
macro_rules! ImplOp {
    ( $type:ident, $trait:ident, $method:ident, $op:tt) => {
        impl< T> $trait< T> for $type
        where
            T: Into< $type>,
        {
            type Output = $type;
            #[inline]
            fn	$method( self, rhs: T) -> Self::Output
            {
                $type( self.0 $op rhs.into().0)
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplUIntTraits {
    ( $type:ident, $prim:ty, $atomic:ty) => {
        impl< T> Add< T> for $type
        where
            T: Into< $type>,
        {
            type Output = $type;
            #[inline]
            fn	add( self, rhs: T) -> Self::Output
            {
                $type( self.0.wrapping_add( rhs.into().0))
            }
        }
        impl< T> Sub< T> for $type
        where
            T: Into< $type>,
        {
            type Output = $type;
            #[inline]
            fn	sub( self, rhs: T) -> Self::Output
            {
                $type( self.0.wrapping_sub( rhs.into().0))
            }
        }
        ImplOp!( $type, Mul, mul, *);
        ImplOp!( $type, Div, div, /);
        ImplOp!( $type, Rem, rem, %);
        ImplOp!( $type, BitAnd, bitand, &);
        ImplOp!( $type, BitOr, bitor, |);
        ImplOp!( $type, BitXor, bitxor, ^);
        ImplOp!( $type, Shl, shl, << );
        ImplOp!( $type, Shr, shr, >>);
        // Unary `-` ( negation) for unsigned values is defined as wrapping subtraction from zero.
        impl Neg for $type
        {
            type Output = $type;
            #[inline]
            fn	neg( self) -> Self::Output
            {
                $type( ( 0 as $prim).wrapping_sub( self.0))
            }
        }
        impl Not for $type
        {
            type Output = $type;
            #[inline]
            fn	not( self) -> Self::Output
            {
                $type( !self.0)
            }
        }
        // Display & formatting
        impl std::fmt::Display for $type
        {
            fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result 
            {
                write!( f, "{}", self.0)
            }
        }
        // Allow wrapping type to be used where its underlying primitive is expected via `Deref`.
        impl Deref for $type
        {
            type Target = $prim;
            #[inline]
            fn	deref( &self) -> &Self::Target
            {
                &self.0
            }
        }
        impl AtomicInt for $type
        {
            type AtomicType = $atomic;
            #[inline]
            fn	IntoAtomic( self) -> Self::AtomicType
            { < $atomic>::new( self.0) }
            #[inline]
            fn	Get( a: &Self::AtomicType, order: Ordering) -> Self
            { $type( a.load( order)) }
            #[inline]
            fn	Set( a: &Self::AtomicType, val: Self, order: Ordering)
            { a.store( val.0, order); }
            #[inline]
            fn	FetchAdd( a: &Self::AtomicType, val: Self, order: Ordering) -> Self
            { $type( a.fetch_add( val.0, order)) }
            #[inline]
            fn	CompareExchange( a: &Self::AtomicType, current: Self, newVal: Self, success: Ordering, failure: Ordering) -> Result< Self, Self>
            {
                a.compare_exchange( current.0, newVal.0, success, failure).map( $type).map_err( $type)
            }
        }
        impl From< $type> for $prim
        {
            #[inline]
            fn	from( v: $type) -> Self
            { v.0 }
        }
        impl From< $type> for i32
        {
            #[inline]
            fn	from( v: $type) -> Self
            { v.0 as i32 }
        }
        impl From< $type> for usize
        {
            #[inline]
            fn	from( v: $type) -> Self
            { v.0 as usize }
        }
        // Additional convenience implementations for testing and interoperability
        impl From< $prim> for $type
        {
            #[inline]
            fn	from( v: $prim) -> Self
            { $type( v) }
        }
        impl From< i32> for $type
        {
            #[inline]
            fn	from( v: i32) -> Self
            { $type( v as $prim) }
        }
        impl From< usize> for $type
        {
            #[inline]
            fn	from( v: usize) -> Self
            { $type( v as $prim) }
        }
        impl PartialEq< $prim> for $type
        {
            #[inline]
            fn	eq( &self, other: &$prim) -> bool
            { self.0 == *other }
        }
        impl PartialEq< $type> for $prim
        {
            #[inline]
            fn	eq( &self, other: &$type) -> bool
            { *self == other.0 }
        }
        impl PartialOrd< $prim> for $type
        {
            #[inline]
            fn	partial_cmp( &self, other: &$prim) -> Option< std::cmp::Ordering>
            { self.0.partial_cmp( other) }
        }
        impl PartialOrd< $type> for $prim
        {
            #[inline]
            fn	partial_cmp( &self, other: &$type) -> Option< std::cmp::Ordering>
            { self.partial_cmp( &other.0) }
        }
        impl $type {
            #[inline]
            pub fn	FromSlice( slice: &[$prim]) -> &[Self]
            {
                unsafe { std::mem::transmute( slice) }
            }
        }
        impl< T> std::ops::AddAssign< T> for $type
        where
            T: Into< $type>,
        {
            #[inline]
            fn	add_assign( &mut self, rhs: T)
            {
                self.0 = self.0.wrapping_add( rhs.into().0);
            }
        }
        impl< T> std::ops::SubAssign< T> for $type
        where
            T: Into< $type>,
        {
            #[inline]
            fn	sub_assign( &mut self, rhs: T)
            {
                self.0 = self.0.wrapping_sub( rhs.into().0);
            }
        }
        impl< T> std::ops::BitAndAssign< T> for $type
        where
            T: Into< $type>,
        {
            #[inline]
            fn	bitand_assign( &mut self, rhs: T)
            {
                self.0 &= rhs.into().0;
            }
        }
        impl< T> std::ops::BitOrAssign< T> for $type
        where
            T: Into< $type>,
        {
            #[inline]
            fn	bitor_assign( &mut self, rhs: T)
            {
                self.0 |= rhs.into().0;
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplUIntTraits!( U8, u8, AtomicU8);
ImplUIntTraits!( U16, u16, AtomicU16);
ImplUIntTraits!( U32, u32, AtomicU32);
ImplUIntTraits!( U64, u64, AtomicU64);

//---------------------------------------------------------------------------------------------------------------------------------

impl From< U8> for U16 
{
    #[inline]
    fn	from( v: U8) -> Self 
    {
        U16( v.0 as u16)
    }
}

impl From< U8> for U32 
{
    #[inline]
    fn	from( v: U8) -> Self 
    {
        U32( v.0 as u32)
    }
}

impl From< U8> for U64 
{
    #[inline]
    fn	from( v: U8) -> Self 
    {
        U64( v.0 as u64)
    }
}

impl From< U16> for U32 
{
    #[inline]
    fn	from( v: U16) -> Self 
    {
        U32( v.0 as u32)
    }
}

impl From< U16> for U64 
{
    #[inline]
    fn	from( v: U16) -> Self 
    {
        U64( v.0 as u64)
    }
}

impl From< U32> for U64 
{
    #[inline]
    fn	from( v: U32) -> Self 
    {
        U64( v.0 as u64)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
