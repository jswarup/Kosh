//-- stk.rs -------------------------------------------------------------------------------------------------------------------------
use	std::fmt;
use	crate::silo::{ Arr, IAccess, IArr, U32, USeg };
use	crate::stalks::Atm;
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Stk< 'a, 'b, T>
{
    _Size: &'a Atm< U32>,
    _Arr: Arr< 'b, T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'b, T> Stk< 'a, 'b, T>
{
    pub fn	Create( _Size: &'a Atm< U32>, _Arr: Arr< 'b, T>) -> Self
    {
        Self { _Size, _Arr }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Size( &self) -> U32
    {
        self._Size.Load( Ordering::Acquire)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SetSize( &mut self, size: U32)
    {
        self._Size.Store( size, Ordering::Release)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SzVoid( &self) -> U32
    {
        self._Arr.Size() - self.Size()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	USeg( &self) -> USeg
    {
        USeg::New( U32( 0), self.Size())
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    /// Requires exclusive access (not thread-safe to call concurrently).

    pub fn	Arr( &self) -> Arr< 'b, T>
    {
        self._Arr.RSnip( self._Arr.Size() - self.Size())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Pop( &self, val: &mut T) -> bool
    {
        let  	sz = self.Size();
        if ( sz == U32( 0))
            || ( self
                ._Size
                .CompareExchange(
                    sz,
                    sz - U32( 1),
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_err()) {
            return false;
        }
        self._Arr.SwapAt( sz - 1, val);
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    /// Requires exclusive access (not thread-safe to call concurrently).

    pub fn	PushX( &self, val: &mut T) -> bool
    {
        let  	sz = self.Size();
        if sz >= self._Arr.Size() {
            return false;
        }
        self._Arr.SwapAt( sz, val);                                    // Write data BEFORE publishing
        if self
            ._Size
            .CompareExchange(
                sz,
                sz + U32( 1),
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_err() {
            self._Arr.SwapAt( sz, val);                                // Rollback: swap original value back
            return false;
        }
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn  Push( &self, v:  T) -> bool
    {
        let  	mut val = v;
        self.PushX( &mut val)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Import< M: Into< U32>>( &self, stk: &Stk< '_, '_, T>, maxMov: M) -> U32
    where
        T: Copy,
    {
        let  	maxMov = maxMov.into();
        let  	( szAlloc, oldSz) = loop {
            let  	sz = self.Size();
            let  	szCacheVoid = self._Arr.Size() - sz;
            let  	mut szAlloc = if szCacheVoid < stk.Size() {
                szCacheVoid
            } else {
                stk.Size()
            };
            if szAlloc > maxMov {
                szAlloc = maxMov
            }
            if szAlloc == U32( 0) {
                break ( U32( 0), sz);
            }
            if self
                ._Size
                .CompareExchange( sz, sz + szAlloc, Ordering::AcqRel, Ordering::Acquire)
                .is_ok() {
                break ( szAlloc, sz);
            }
        };
        if szAlloc == U32( 0) {
            return U32( 0);
        }
        let  	stkSz = stk._Size.FetchAdd( U32( 0) - szAlloc, Ordering::AcqRel) - szAlloc;
        self._Arr.SwapFrom( oldSz, &stk._Arr, stkSz, szAlloc);
        szAlloc
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Export< M: Into< U32>>( &self, stk: &Stk< '_, '_, T>, maxMov: M) -> U32
    where
        T: Copy,
    {
        let  	maxMov = maxMov.into();
        let  	( szAlloc, oldSz) = loop {
            let  	szStk = stk.Size();
            let  	szStkVoid = stk._Arr.Size() - szStk;
            let  	sz = self.Size();
            let  	mut szAlloc = if szStkVoid < sz { szStkVoid } else { sz };
            if szAlloc > maxMov {
                szAlloc = maxMov
            }
            if szAlloc == U32( 0) {
                break ( U32( 0), sz);
            }
            if self
                ._Size
                .CompareExchange( sz, sz - szAlloc, Ordering::AcqRel, Ordering::Acquire)
                .is_ok() {
                break ( szAlloc, sz);
            }
        };
        if szAlloc == U32( 0) {
            return U32( 0);
        }
        let  	szStk = stk._Size.FetchAdd( U32( 0) + szAlloc, Ordering::AcqRel);
        stk._Arr.SwapFrom( szStk, &self._Arr, oldSz - szAlloc, szAlloc);
        szAlloc
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'b, T: fmt::Display> fmt::Display for Stk< 'a, 'b, T>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        fmt::Display::fmt( &self.Arr(), f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
