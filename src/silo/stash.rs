//-- stash.rs -------------------------------------------------------------------------------------------------------------------------

use crate::silo::atm::Atm;
use crate::silo::buff::Buff;
use crate::silo::stk::Stk;
use crate::silo::uint::U32;

//---------------------------------------------------------------------------------------------------------------------------------
pub struct Stash< T>
{
    _Buff: Buff< T>,
    _Sz: Atm< U32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Default> Stash< T>
{
    pub fn	New< S: Into< U32>>( sz: S) -> Self
    {
        Self {
            _Buff: Buff::Create( sz, |_| T::default()),
            _Sz: Atm::New( U32( 0)),
        }
    }

    pub fn	Create< S1: Into< U32>, S2: Into< U32>, Dispenser>( sz: S1, szStk: S2, dispenser: Dispenser) -> Self
    where
        Dispenser: Fn( U32) -> T,
    {
        Self {
            _Buff: Buff::Create( sz, dispenser),
            _Sz: Atm::New( szStk.into()),
        }
    }

    pub fn	Size( &self) -> U32
    {
        self._Sz.Get()
    }

    pub fn	Stk( &self) -> Stk< '_, '_, T>
    {
        Stk::Create( &self._Sz, self._Buff.Arr())
    }

    pub fn	DoIndexSetup( &mut self)
    where
        T: From< usize> + Clone,
    {
        let arr = self._Buff.Arr();
        arr.USeg().Span( |i: U32| {
            arr.SetAt( i, &T::from( i.as_usize()));
            true
        });
        self._Sz.Set( arr.Size());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
