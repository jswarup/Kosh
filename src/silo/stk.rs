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

    pub fn Size( self) -> u32
    {
        self._Size.Get()
    }
    pub fn SzVoid( self) -> u32
    {
        self._Arr.Size() -self._Size.Get()
    }

    pub fn USeg( self) -> USeg
    {
        USeg::Create( 0, self._Size.Get())
    }

    pub fn Top( self) -> &'b T
    {
        self._Arr.At( self._Size.Get() - 1)
    }

    pub fn Arr( self) -> Arr< 'b, T>
    {
        self._Arr.RSnip( self._Arr.Size() - self._Size.Get() )
    }


}

//---------------------------------------------------------------------------------------------------------------------------------

