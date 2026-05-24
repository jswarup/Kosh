//-- stk.rs -------------------------------------------------------------------------------------------------------------------------

use crate::silo::arr::Arr;
use crate::silo::atm::Atm;
use crate::silo::useg::USeg;

//---------------------------------------------------------------------------------------------------------------------------------
pub struct Stk<'a, 'b, T>
{
    _Size: &'a mut Atm< u32>,
    _Arr: &'b mut Arr<'b, T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'b, T> Stk<'a, 'b, T>
{
    pub fn Create(_Size: &'a mut Atm<u32>, _Arr: &'b mut Arr<'b, T>) -> Self
    {
        Self { _Size, _Arr }
    }

    pub fn Size( &self) -> u32
    {
        self._Size.Get()
    }
    pub fn SzVoid( &self) -> u32
    {
        self._Arr.Size() -self.Size()
    }

    pub fn USeg( &self) -> USeg
    {
        USeg::Create( 0, self.Size())
    }

    pub fn Arr( &self) -> Arr< 'b, T>
    {
        self._Arr.RSnip( self._Arr.Size() - self.Size() )
    }

    pub fn Pop( &mut self, val: &mut T) -> bool where T: Default + Clone
    {
        let sz = self.Size();
        if ( sz == 0) ||
            ( self._Size.CompareExchange(sz, sz - 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_err())
        {
            return false;
        }
        *val = self._Arr.At( sz - 1).clone();
        true
    }

    pub fn Push( &mut self, val: &mut T) -> bool where T: Default
    {
        let sz = self.Size();
        if (sz >= self._Arr.Size()) ||
            self._Size.CompareExchange(sz, sz + 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_err()
        {
            return false;
        }
        self._Arr.MoveAt(sz, val);
        true
    }

    pub fn Import(&mut self, stk: &mut Stk<'_, '_, T>, maxMov: u32) -> u32 where T: Clone
    {
        let szAlloc = loop {
            let     sz = self.Size();
            let     szCacheVoid = self._Arr.Size() - sz;
            let mut szAlloc = if szCacheVoid < stk.Size() { szCacheVoid } else { stk.Size() };
            szAlloc = if szAlloc > maxMov { maxMov } else { szAlloc };

            if szAlloc == 0 ||
                self._Size.CompareExchange( sz, sz + szAlloc, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_ok()
            {
                break szAlloc;
            }
        };
        let sz = self.Size();
        let stkSz = stk._Size.FetchAdd( (0u32).wrapping_sub(szAlloc), std::sync::atomic::Ordering::SeqCst) - szAlloc;
        USeg::Create(0, szAlloc).Span( | i| {
            self._Arr.SetAt( sz - szAlloc + i, stk._Arr.At( stkSz + i));
            true
        });
        szAlloc
    }

    pub fn Export(&mut self, stk: &mut Stk<'_, '_, T>, maxMov: u32) -> u32 where T: Clone
    {
        let szAlloc = loop {
            let     sz = stk.Size();
            let     szCacheVoid = stk._Arr.Size() - sz;
            let mut szAlloc = if szCacheVoid < self.Size() { szCacheVoid } else { self.Size() };
            szAlloc = if szAlloc > maxMov { maxMov } else { szAlloc };

            if szAlloc == 0 ||
                self._Size.CompareExchange( sz, sz - szAlloc, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_ok()
            {
                break szAlloc;
            }
        };
        let sz = self.Size();
        let stkSz = stk._Size.FetchAdd( (0u32).wrapping_sub(szAlloc), std::sync::atomic::Ordering::SeqCst) - szAlloc;
        USeg::Create(0, szAlloc).Span( | i| {
            stk._Arr.SetAt( stkSz - szAlloc + i, self._Arr.At( sz + i));
            true
        });
        szAlloc
    }

}

//---------------------------------------------------------------------------------------------------------------------------------

