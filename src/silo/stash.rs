//-- stash.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, Buff, Stk, U32 };
use	crate::stalks::Atm;
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Stash< T> 
{
    _Buff: Buff< T>,
    _Sz: Atm< U32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Stash< T> 
{
    
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Create< Sz: Into< U32>, SzStk: Into< U32>, Dispenser>( 
        sz: Sz,
        szStk: SzStk,
        dispenser: Dispenser,
    ) -> Self
    where
        Dispenser: Fn( U32) -> T,
    {
        Self {
            _Buff: Buff::Create( sz, dispenser),
            _Sz: Atm::New( szStk.into()),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Size( &self) -> U32 
    {
        self._Sz.Load( Ordering::Acquire)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Clear( &self) 
    {
        self._Sz.Store( U32( 0), Ordering::Release);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Stk( &self) -> Stk< '_, '_, T> 
    {
        Stk::Create( &self._Sz, self._Buff.Arr())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	BuffOut( &mut self) -> Buff< T>
    {
        let  	mut buff = Buff::NewEmpty();
        self._Buff.Swap( &mut buff);
        buff
    }

    //-----------------------------------------------------------------------------------------------------------------------------
 
    pub fn	Pushback( &mut self, val: &mut T) 
    where
        T: Default,
    {
        while !self.Stk().Push( val) {
            if self.Size() == self._Buff.Size() {
                let  	newSz = if self._Buff.Size() == U32( 0) {
                    U32( 1)
                } else {
                    self._Buff.Size() * U32( 2)
                };
                self._Buff.Resize( newSz, |_| T::default());
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    pub fn	Append( &mut self, arr: Arr< '_, T>) 
    where
        T: Default,
    {
        let  	n = arr.Size();
        if n == U32( 0) {
            return;
        }
        let  	neededSz = self.Size() + n;
        if neededSz > self._Buff.Size() {
            self._Buff.Resize( neededSz, |_| T::default());
        }
        let  	startSz = self.Size();
        let  	arrBuff = self._Buff.Arr();
        for i in 0..usize::from( n) {
            arrBuff.MoveAt( startSz + U32( i as u32), arr.MutAt( U32( i as u32)));
        }
        self._Sz.Set( startSz + n);
    }
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
}
//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Stash< T>
where
    T: From< usize> + Clone,
{
    //-----------------------------------------------------------------------------------------------------------------------------
    pub fn	DoIndexSetup( &self)
    {
        let  	arr = self._Buff.Arr();
        arr.USeg().Traverse( |i: U32| {
            arr.SetAt( i, &T::from( i.AsUsize()));
        });
        self._Sz.Store( arr.Size(), Ordering::Release);
    }
    //-----------------------------------------------------------------------------------------------------------------------------
}

//---------------------------------------------------------------------------------------------------------------------------------
