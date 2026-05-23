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
        self._Arr.Size() -self._Size.Get()
    }

    pub fn USeg( &self) -> USeg
    {
        USeg::Create( 0, self._Size.Get())
    }

    pub fn Arr( &self) -> Arr< 'b, T>
    {
        self._Arr.RSnip( self._Arr.Size() - self._Size.Get() )
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

}

//---------------------------------------------------------------------------------------------------------------------------------

