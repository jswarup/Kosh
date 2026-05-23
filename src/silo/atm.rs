//-- atm.rs -----------------------------------------------------------------------------------------------------------------------

use std::sync::atomic::*;

//---------------------------------------------------------------------------------------------------------------------------------
/// Trait to abstract over standard atomic integer types

pub trait AtomicInt: Sized
{
    type AtomicType;

    fn  IntoAtomic(self) -> Self::AtomicType;
    fn  Get(a: &Self::AtomicType, order: Ordering) -> Self;
    fn  Set(a: &Self::AtomicType, val: Self, order: Ordering);
    fn  FetchAdd(a: &Self::AtomicType, val: Self, order: Ordering) -> Self;

    fn  CompareExchange( a: &Self::AtomicType, current: Self, newVal: Self, success: Ordering, failure: Ordering, ) -> Result<Self, Self>
            where Self: Sized;
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! impl_atomic_int
{
    ($prim:ty, $atomic:ty) =>
    {
        impl AtomicInt for $prim
        {
            type AtomicType = $atomic;

            fn IntoAtomic(self) -> Self::AtomicType
            {
                <$atomic>::new(self)
            }

            fn Get(a: &Self::AtomicType, order: Ordering) -> Self
            {
                a.load(order)
            }

            fn Set(a: &Self::AtomicType, val: Self, order: Ordering)
            {
                a.store(val, order);
            }

            fn FetchAdd(a: &Self::AtomicType, val: Self, order: Ordering) -> Self
            {
                a.fetch_add(val, order)
            }

            fn CompareExchange( a: &Self::AtomicType, current: Self, newVal: Self, success: Ordering, failure: Ordering, ) -> Result<Self, Self>
            {
                a.compare_exchange(current, newVal, success, failure)
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

impl_atomic_int!(usize, AtomicUsize);
impl_atomic_int!(isize, AtomicIsize);
impl_atomic_int!(u32, AtomicU32);
impl_atomic_int!(i32, AtomicI32);
impl_atomic_int!(u64, AtomicU64);
impl_atomic_int!(i64, AtomicI64);
impl_atomic_int!(u8, AtomicU8);
impl_atomic_int!(i8, AtomicI8);
impl_atomic_int!(u16, AtomicU16);
impl_atomic_int!(i16, AtomicI16);

//---------------------------------------------------------------------------------------------------------------------------------
/// A generic wrapper that encapsulates an atomic variable.
pub struct Atm<T: AtomicInt>
{
    _Val: T::AtomicType,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T: AtomicInt> Atm<T>
{
    /// Creates a new `Atm` with the given initial value.
    pub fn New(v: T) -> Self
    {
        Self
        {
            _Val: v.IntoAtomic(),
        }
    }

    /// Loads the value using the provided ordering.
    pub fn Load(&self, order: Ordering) -> T
    {
        T::Get(&self._Val, order)
    }

    /// Stores a value using the provided ordering.
    pub fn Store(&self, v: T, order: Ordering)
    {
        T::Set(&self._Val, v, order);
    }

    /// Convenience for sequential consistency load.
    pub fn Get(&self) -> T
    {
        self.Load(Ordering::SeqCst)
    }

    /// Convenience for sequential consistency store.
    pub fn Set(&self, v: T)
    {
        self.Store(v, Ordering::SeqCst);
    }

    /// Adds to the current value, returning the previous value.
    pub fn FetchAdd(&self, v: T, order: Ordering) -> T
    {
        T::FetchAdd(&self._Val, v, order)
    }

    /// Stores a value into the atomic integer if the current value is the same as the `current` value.
    pub fn CompareExchange( &self, current: T, newVal: T, success: Ordering, failure: Ordering, ) -> Result<T, T>
    {
        T::CompareExchange(&self._Val, current, newVal, success, failure)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

