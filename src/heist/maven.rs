//-- maven.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::heist::atelier::Atelier;
use	crate::silo::stash::Stash;
use	crate::silo::stk::Stk;
use	crate::silo::uint::{ U16, U32 };
use	crate::stalks::atm::{ Atm, Spinlock };
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maven 
{
    _SzProcessed: U32,
    _JobCache: Stash< U16>,
    _RunQueue: Stash< U16>,
    _RunQlock: Spinlock,
    _CurSuccId: Atm< U16>,
    _TempQueue: Stash< U16>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Maven 
{

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( _mavenInd: U32) -> Self 
    {
        Self {
            _SzProcessed: U32::_0,
            _JobCache: Stash::< U16>::New( U32( 256)),
            _RunQueue: Stash::< U16>::New( U32( 1024)),
            _RunQlock: Spinlock::New(),
            _CurSuccId: Atm::New( U16::_0),
            _TempQueue: Stash::< U16>::New( U32( 64)),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	JobCacheStk( &self) -> Stk< '_, '_, U16> 
    {
        self._JobCache.Stk()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	TempQueueStk( &self) -> Stk< '_, '_, U16> 
    {
        self._TempQueue.Stk()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FlushTempQueue( &self, atelier: &Atelier< '_>, mavenIdx: U32) 
    {
        let  	arr = self._TempQueue.Stk().Arr();
        arr.USeg().Traverse( |i| {
            let  	mut jobId = *arr.At( i);
            if jobId != 0 {
                atelier.EnqueueJob( mavenIdx, &mut jobId);
            }
        });
        self._TempQueue.Clear();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SzProcessed( &self) -> U32 
    {
        self._SzProcessed
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	IncrSzProcessed< K: Into< U32>>( &mut self, k: K) 
    {
        self._SzProcessed = self._SzProcessed + k.into();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, jobId: &mut U16) 
    {
        let  	_guard = self._RunQlock.Lock();
        self._RunQueue.Stk().Push( jobId);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	PopJob( &self) -> U16 
    {
        let  	xStk = self._RunQueue.Stk();
        let  	mut jobId = U16( 0);
        if xStk.Size() != 0 {
            let  	_guard = self._RunQlock.Lock();
            if xStk.Size() != 0 && xStk.Pop( &mut jobId) {
                return jobId;
            }
        }
        jobId
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	CurSuccId( &self) -> U16 
    {
        self._CurSuccId.Load( Ordering::Acquire)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SetCurSuccId< K: Into< U16>>( &self, val: K) 
    {
        self._CurSuccId.Store( val, Ordering::Release);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------
