//-- stash.rs -------------------------------------------------------------------------------------------------------------------------

use crate::silo::buff::Buff;
use crate::silo::uint::U32;
use crate::silo::atm::Atm;
use crate::silo::stk::Stk;

//---------------------------------------------------------------------------------------------------------------------------------
pub struct Stash<T>
{
    _Buff: Buff< T>,
    _Sz: Atm< U32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T: Default> Stash<T>
{
    pub fn New( sz: U32) -> Self
    {
        Self
        {
            _Buff: Buff::Create(sz , |_| T::default()),
            _Sz: Atm::New(U32(0)),
        }
    }


    pub fn Create< Dispenser>( sz: U32, szStk : U32, dispenser: Dispenser) -> Self
        where
            Dispenser: Fn( U32) -> T
    {   Self
        {
            _Buff: Buff::Create(sz, dispenser),
            _Sz: Atm::New( szStk)
        }
    }

    pub fn Size( &self) -> U32 { self._Sz.Get() }

    pub fn Stk( &mut self) -> Stk<'_, '_, T>
    {
        Stk::Create(&mut self._Sz, self._Buff.AsMutArr())
    }

    pub fn DoIndexSetup( &mut self)
    where T: From<usize> + Clone
    {
        let  arr = self._Buff.AsArr();
        arr.USeg().Span( | i: U32 | {
            arr.SetAt(i, &T::from(i.as_usize()));
            true
        });
        self._Sz.Set( arr.Size());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

