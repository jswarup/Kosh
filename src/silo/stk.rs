//-- stk.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::arr::Arr;
use	crate::stalks::atm::Atm;
use	crate::silo::uint::U32;
use	crate::silo::useg::USeg;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Stk< 'a, 'b, T>
{
    _Size: &'a Atm<U32>,
    _Arr: Arr< 'b, T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'b, T> Stk< 'a, 'b, T>
{
    pub fn	Create( _Size: &'a Atm<U32>, _Arr: Arr<'b, T>) -> Self
    {
        Self
        { _Size, _Arr }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Size( &self) -> U32
    {
        self._Size.Get()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SzVoid( &self) -> U32
    {
        self._Arr.Size() - self.Size()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	USeg( &self) -> USeg
    {
        USeg::Create( U32( 0), self.Size())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Arr( &self) -> Arr< 'b, T> {
        self._Arr.RSnip( self._Arr.Size() - self.Size())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// CAS-decrement _Size (Acquire), then read. Pairs with Push's Release.
    pub fn	Pop( &self, val: &mut T) -> bool
    where
        T: Default + Clone,
    {
		let  	sz = self.Size();
        if ( sz == U32( 0))
            || ( self._Size.CompareExchange( sz, sz - U32( 1),
                    std::sync::atomic::Ordering::Acquire,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_err()) {
            return false;
        }
        *val = self._Arr.At( sz - 1).clone();
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Write-then-publish: MoveAt data first, CAS-increment _Size (Release).
    /// On CAS failure, MoveAt rolls back the speculative write.
    pub fn	Push( &self, val: &mut T) -> bool
    where
        T: Default,
    {
		let  	sz = self.Size();
        if sz >= self._Arr.Size() {
            return false;
        }
        self._Arr.MoveAt( sz, val);                                     // Write data BEFORE publishing
        if self
            ._Size
            .CompareExchange(
                sz,
                sz + U32( 1),
                std::sync::atomic::Ordering::Release,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err() {
            self._Arr.MoveAt( sz, val);                                 // Rollback: swap original value back
            return false;
        }
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Import< M: Into< U32>>( &self, stk: &Stk< '_, '_, T>, maxMov: M) -> U32
    where
        T: Clone,
    {
		let  	max_mov = maxMov.into();
		let  	( szAlloc, oldSz) = loop {
			let  	sz = self.Size();
			let  	szCacheVoid = self._Arr.Size() - sz;
			let  	mut szAlloc = if szCacheVoid < stk.Size() {
                szCacheVoid
            } else {
                stk.Size()
            };
            if szAlloc > max_mov {
                szAlloc = max_mov
            }
            if szAlloc == U32( 0) {
                break ( U32( 0), sz);
            }
            if self
                ._Size
                .CompareExchange( sz, sz + szAlloc,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok() {
                break ( szAlloc, sz);
            }
        };
        if szAlloc == U32( 0) {
            return U32( 0);
        }
		let  	stkSz = stk._Size.FetchAdd( U32( 0) - szAlloc, std::sync::atomic::Ordering::SeqCst) - szAlloc;
        USeg::Create( U32( 0), szAlloc).Span( |i| {
            self._Arr.SetAt( oldSz + i, stk._Arr.At( stkSz + i));
            true
        });
        szAlloc
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Export< M: Into< U32>>( &self, stk: &Stk< '_, '_, T>, maxMov: M) -> U32
    where
        T: Clone,
    {
		let  	max_mov = maxMov.into();
		let  	( szAlloc, oldSz) = loop {
			let  	szStk = stk.Size();
			let  	szStkVoid = stk._Arr.Size() - szStk;
			let  	sz = self.Size();
			let  	mut szAlloc = if szStkVoid < sz {
                szStkVoid
            } else  {
                sz
            };
            if szAlloc > max_mov {
                szAlloc = max_mov
            }
            if szAlloc == U32( 0) {
                break ( U32( 0), sz);
            }
            if self._Size.CompareExchange( sz, sz - szAlloc,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok() {
                break ( szAlloc, sz);
            }
        };
        if szAlloc == U32( 0) {
            return U32( 0);
        }
		let  	szStk = stk._Size.FetchAdd( U32( 0) + szAlloc, std::sync::atomic::Ordering::SeqCst);
        USeg::Create( U32( 0), szAlloc).Span( |i| {
            stk._Arr.SetAt( szStk + i, self._Arr.At( oldSz - szAlloc + i));
            true
        });
        szAlloc
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
