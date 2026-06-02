//-- atm.rs -----------------------------------------------------------------------------------------------------------------------
use	std::sync::atomic::*;

//---------------------------------------------------------------------------------------------------------------------------------

/// Trait to abstract over standard atomic integer types
pub trait AtomicInt: Sized {
    type AtomicType;
    fn	IntoAtomic( self) -> Self::AtomicType;
    fn	Get( a: &Self::AtomicType, order: Ordering) -> Self;
    fn	Set( a: &Self::AtomicType, val: Self, order: Ordering);
    fn	FetchAdd( a: &Self::AtomicType, val: Self, order: Ordering) -> Self;
    fn	CompareExchange(
        a: &Self::AtomicType,
        current: Self,
        newVal: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result< Self, Self>
    where
        Self: Sized;
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplAtomicInt {
    ( $prim:ty, $atomic:ty) => {
        impl AtomicInt for $prim
        {
            type AtomicType = $atomic;
            fn	IntoAtomic( self) -> Self::AtomicType
            {
                <$atomic>::new( self)
            }
            fn	Get( a: &Self::AtomicType, order: Ordering) -> Self
            {
                a.load( order)
            }
            fn	Set( a: &Self::AtomicType, val: Self, order: Ordering)
            {
                a.store( val, order);
            }
            fn	FetchAdd( a: &Self::AtomicType, val: Self, order: Ordering) -> Self
            {
                a.fetch_add( val, order)
            }
            fn	CompareExchange(
                a: &Self::AtomicType,
                current: Self,
                newVal: Self,
                success: Ordering,
                failure: Ordering,
            ) -> Result< Self, Self> {
                a.compare_exchange( current, newVal, success, failure)
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplAtomicInt!( usize, AtomicUsize);
ImplAtomicInt!( isize, AtomicIsize);
ImplAtomicInt!( u32, AtomicU32);
ImplAtomicInt!( i32, AtomicI32);
ImplAtomicInt!( u64, AtomicU64);
ImplAtomicInt!( i64, AtomicI64);
ImplAtomicInt!( u8, AtomicU8);
ImplAtomicInt!( i8, AtomicI8);
ImplAtomicInt!( u16, AtomicU16);
ImplAtomicInt!( i16, AtomicI16);

//---------------------------------------------------------------------------------------------------------------------------------

/// A generic wrapper that encapsulates an atomic variable.
pub struct Atm< T: AtomicInt>
{
    _Val: T::AtomicType,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: AtomicInt> Atm< T>
{
    /// Creates a new `Atm` with the given initial value.
    pub fn	New( v: T) -> Self
    {
        Self {
            _Val: v.IntoAtomic(),
        }
    }
    /// Loads the value using the provided ordering.
    pub fn	Load( &self, order: Ordering) -> T
    {
        T::Get( &self._Val, order)
    }
    /// Stores a value using the provided ordering.
    pub fn	Store< K: Into< T>>( &self, v: K, order: Ordering)
    {
        T::Set( &self._Val, v.into(), order);
    }
    /// Convenience for sequential consistency load.
    pub fn	Get( &self) -> T
    {
        self.Load( Ordering::SeqCst)
    }
    /// Convenience for sequential consistency store.
    pub fn	Set< K: Into< T>>( &self, v: K)
    {
        self.Store( v, Ordering::SeqCst);
    }
    /// Adds to the current value, returning the previous value.
    pub fn	FetchAdd< K: Into< T>>( &self, v: K, order: Ordering) -> T
    {
        T::FetchAdd( &self._Val, v.into(), order)
    }
    /// Stores a value into the atomic integer if the current value is the same as the `current` value.
    pub fn	CompareExchange< K: Into< T>>(
        &self,
        current: K,
        newVal: K,
        success: Ordering,
        failure: Ordering,
    ) -> Result< T, T> {
        T::CompareExchange( &self._Val, current.into(), newVal.into(), success, failure)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// A simple spinlock.
pub struct Spinlock
{
    _Locked: AtomicBool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Spinlock
{

    //------------------------------------------------------------------------------------------------------------------------------

    /// Creates a new unlocked spinlock.
    pub const fn	New() -> Self
    {
        Self {
            _Locked: AtomicBool::new( false),
        }
    }

    //------------------------------------------------------------------------------------------------------------------------------

    /// Acquires the spinlock, blocking the current thread until it is able to do so.
    pub fn	Acquire( &self)
    {
        while self
            ._Locked
            .compare_exchange_weak( false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err() {
            while self._Locked.load( Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
    }

    //------------------------------------------------------------------------------------------------------------------------------

    /// Acquires the spinlock and returns a guard that releases the lock when dropped.
    pub fn	Lock( &self) -> SpinLockGuard< '_> {
        self.Acquire();
        SpinLockGuard
        { _Lock: self }
    }

    //------------------------------------------------------------------------------------------------------------------------------

    /// Releases the spinlock.
    pub fn	Release( &self)
    {
        self._Locked.store( false, Ordering::Release);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// An RAII implementation of a "scoped lock" of a spinlock.
pub struct SpinLockGuard< 'a> {
    _Lock: &'a Spinlock,
}
impl< 'a> Drop for SpinLockGuard<'a>
{
    fn	drop( &mut self)
    {
        self._Lock.Release();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
