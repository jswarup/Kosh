//-- uint.rs -------------------------------------------------------------------------------------------------------------------------
//!  thin wrappers around unsigned integers providing seamless operations.
use	std::{ cmp, fmt, ops::{ AddAssign, SubAssign } };
use	crate::silo::cast::ICastExt;
use	crate::stalks::atm::AtomicInt;
use	std::ops::{ Add, BitAnd, BitOr, BitXor, Deref, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub };
use	std::sync::atomic::{ AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering };
use	crate::silo::Arr;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr( transparent)]
pub struct U8( pub u8);
impl U8 
{
    pub const _X: Self = U8( u8::MAX);
    pub const _0: Self = U8( 0u8);
    pub const _1: Self = U8( 1u8);
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
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplUIntTraits {
    ( $type:ident, $prim:ty, $atomic:ty, $asPrim:ident) => {
        impl $type
        {
            #[inline]
            pub const fn	From( v: $prim) -> Self
            {
                $type( v)
            }
            #[inline]
            pub const fn	$asPrim( self) -> $prim
            {
                self.0
            }
            #[inline]
            pub const fn	AsUsize( self) -> usize
            {
                self.0 as usize
            }
            #[inline]
            pub const fn	FromUsize( v: usize) -> Self
            {
                $type( v as $prim)
            }
            #[inline]
            pub const fn	Max() -> Self
            {
                $type( <$prim>::MAX)
            }
            #[inline]
            pub const fn	Get( self) -> $prim
            {
                self.0
            }
            #[inline]
            pub fn	FromArr< 'a>(arr: Arr<'a, $prim>) -> Arr< 'a, Self> {
                arr.Cast()
            }
        }
        macro_rules! impl_binop {
            ( $trait:ident, $method:ident, $op:tt) => {
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
        impl_binop!( Mul, mul, *);
        impl_binop!( Div, div, /);
        impl_binop!( Rem, rem, %);
        impl_binop!( BitAnd, bitand, &);
        impl_binop!( BitOr, bitor, |);
        impl_binop!( BitXor, bitxor, ^);
        impl_binop!( Shl, shl, <<);
        impl_binop!( Shr, shr, >>);
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
        impl fmt::Display for $type
        {
            fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result {
                write!( f, "{}", self.0)
            }
        }
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
            {
                <$atomic>::new( self.0)
            }
            #[inline]
            fn	Get( a: &Self::AtomicType, order: Ordering) -> Self
            {
                $type( a.load( order))
            }
            #[inline]
            fn	Set( a: &Self::AtomicType, val: Self, order: Ordering)
            {
                a.store( val.0, order);
            }
            #[inline]
            fn	FetchAdd( a: &Self::AtomicType, val: Self, order: Ordering) -> Self
            {
                $type( a.fetch_add( val.0, order))
            }
            #[inline]
            fn	CompareExchange( 
                a: &Self::AtomicType,
                current: Self,
                newVal: Self,
                success: Ordering,
                failure: Ordering,
            ) -> Result< Self, Self> {
                a.compare_exchange( current.0, newVal.0, success, failure)
                    .map( $type)
                    .map_err( $type)
            }
        }
        impl From< $type> for $prim
        {
            #[inline]
            fn	from( v: $type) -> Self
            {
                v.0
            }
        }
        impl From< $type> for i32
        {
            #[inline]
            fn	from( v: $type) -> Self
            {
                v.0 as i32
            }
        }
        impl From< $type> for usize
        {
            #[inline]
            fn	from( v: $type) -> Self
            {
                v.0 as usize
            }
        }
        impl From< $prim> for $type
        {
            #[inline]
            fn	from( v: $prim) -> Self
            {
                $type( v)
            }
        }
        impl From< i32> for $type
        {
            #[inline]
            fn	from( v: i32) -> Self
            {
                $type( v as $prim)
            }
        }
        impl From< usize> for $type
        {
            #[inline]
            fn	from( v: usize) -> Self
            {
                $type( v as $prim)
            }
        }
        impl PartialEq< $prim> for $type
        {
            #[inline]
            fn	eq( &self, other: &$prim) -> bool
            {
                self.0 == *other
            }
        }
        impl PartialEq< $type> for $prim
        {
            #[inline]
            fn	eq( &self, other: &$type) -> bool
            {
                *self == other.0
            }
        }
        impl PartialOrd< $prim> for $type
        {
            #[inline]
            fn	partial_cmp( &self, other: &$prim) -> Option< cmp::Ordering>
            {
                self.0.partial_cmp( other)
            }
        }
        impl PartialOrd< $type> for $prim
        {
            #[inline]
            fn	partial_cmp( &self, other: &$type) -> Option< cmp::Ordering>
            {
                self.partial_cmp( &other.0)
            }
        }
        macro_rules! impl_assignop {
            ( $trait:ident, $method:ident, $op:tt) => {
                impl< T> std::ops::$trait< T> for $type
                where
                    T: Into< $type>,
                {
                    #[inline]
                    fn	$method( &mut self, rhs: T)
                    {
                        self.0 = self.0 $op rhs.into().0;
                    }
                }
            };
        }
        impl< T> AddAssign< T> for $type
        where
            T: Into< $type>,
        {
            #[inline]
            fn	add_assign( &mut self, rhs: T)
            {
                self.0 = self.0.wrapping_add( rhs.into().0);
            }
        }
        impl< T> SubAssign< T> for $type
        where
            T: Into< $type>,
        {
            #[inline]
            fn	sub_assign( &mut self, rhs: T)
            {
                self.0 = self.0.wrapping_sub( rhs.into().0);
            }
        }
        impl_assignop!( MulAssign, mul_assign, *);
        impl_assignop!( DivAssign, div_assign, /);
        impl_assignop!( RemAssign, rem_assign, %);
        impl_assignop!( BitAndAssign, bitand_assign, &);
        impl_assignop!( BitOrAssign, bitor_assign, |);
        impl_assignop!( BitXorAssign, bitxor_assign, ^);
        impl_assignop!( ShlAssign, shl_assign, <<);
        impl_assignop!( ShrAssign, shr_assign, >>);
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplUIntTraits!( U8, u8, AtomicU8, AsU8);
ImplUIntTraits!( U16, u16, AtomicU16, AsU16);
ImplUIntTraits!( U32, u32, AtomicU32, AsU32);
ImplUIntTraits!( U64, u64, AtomicU64, AsU64);

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplWiden {
    ( $src:ident, $dst:ident, $method:ident) => {
        impl From< $src> for $dst
        {
            #[inline]
            fn	from( v: $src) -> Self
            {
                $dst( v.0 as _)
            }
        }
        impl $src
        {
            #[inline]
            pub const fn $method( self) -> $dst
            {
                $dst( self.0 as _)
            }
        }
    };
}
ImplWiden!( U8, U16, AsU16);
ImplWiden!( U8, U32, AsU32);
ImplWiden!( U8, U64, AsU64);
ImplWiden!( U16, U32, AsU32);
ImplWiden!( U16, U64, AsU64);
ImplWiden!( U32, U64, AsU64);

//---------------------------------------------------------------------------------------------------------------------------------

pub trait Xplod< Dst, const N: usize> {
    fn	Xplod( self) -> [Dst; N];
}

macro_rules! Xplod {
    ( $src:ident, $dst:ident, $sz:expr) => {
        impl Xplod< $dst, $sz> for $src
        {
            #[inline]
            fn	Xplod( self) -> [$dst; $sz]
            {
                self.Cast()
            }
        }
    };
}

Xplod!( U16, U8, 2);
Xplod!( U32, U8, 4);
Xplod!( U64, U8, 8);
Xplod!( U32, U16, 2);
Xplod!( U64, U16, 4);
Xplod!( U64, U32, 2);

//---------------------------------------------------------------------------------------------------------------------------------
