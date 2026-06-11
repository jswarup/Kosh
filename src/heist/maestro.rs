//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::Atelier;
use	crate::silo::{ Buff, Stash, Stk, U16, U32 };
use	crate::stalks::{ Atm, IWorker, IntoWorkPtr, Spinlock, WorkPtr };
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maestro< 'a> 
{
    _Index: U32,
    _Atelier: *const Atelier< 'a>,
    _SzProcessed: U32,
    _JobCache: Stash< U16>,
    _RunQueue: Stash< U16>,
    _RunQlock: Spinlock,
    _CurSuccId: Atm< U16>,
    _TempQueue: Stash< U16>,
}
unsafe impl< 'a> Send for Maestro< 'a>
{ }
unsafe impl< 'a> Sync for Maestro< 'a>
{ }

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Maestro< 'a> 
{

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( maestroInd: U32) -> Self 
    {
        Self {
            _Index: maestroInd,
            _Atelier: std::ptr::null(),
            _SzProcessed: U32::_0,
            _JobCache: Stash::< U16>::New( U32( 256)),
            _RunQueue: Stash::< U16>::New( U32( 1024)),
            _RunQlock: Spinlock::New(),
            _CurSuccId: Atm::New( U16::_0),
            _TempQueue: Stash::< U16>::New( U32( 64)),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SetAtelier( &mut self, atelier: *const Atelier< 'a>) 
    {
        self._Atelier = atelier;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Atelier( &self) -> &Atelier< 'a> 
    {
        assert!( !self._Atelier.is_null());
        unsafe { &*self._Atelier }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MaestroIndex( &self) -> U32 
    {
        self._Index
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FromWorker( worker: &dyn IWorker) -> &Self 
    {
        let  	ptr = worker.AsRaw();
        assert!( !ptr.is_null());
        unsafe { &*( ptr as *const Self) }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob( &self, succId: U16, job: impl IntoWorkPtr< 'a>) -> U16 
    {
        self.Atelier().ConstructJob( self._Index, succId, job.IntoWorkPtr())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, jobId: &mut U16) 
    {
        assert!( self.TempQueueStk().Push( jobId));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructEnqueueBulk( &self, succId: U16, buff: Buff< U16>) -> U16 
    {
        self.ConstructJob( 
            succId,
            move |worker: &dyn IWorker| {
                let  	maestro = Maestro::FromWorker( worker);
                let  	arr = buff.Arr();
                arr.USeg().Traverse( |i| {
                    maestro.Atelier().EnqueueJob( maestro._Index, arr.MutAt( i));
                });
            },
        )
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

    pub fn	FlushTempQueue( &self) 
    {
        let  	arr = self._TempQueue.Stk().Arr();
        arr.USeg().Traverse( |i| {
            let  	mut jobId = *arr.At( i);
            if jobId != 0 {
                self.Atelier().EnqueueJob( self._Index, &mut jobId);
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

    pub fn	EnqueueActiveJob( &self, jobId: &mut U16) 
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

impl< 'a> IWorker for Maestro< 'a> 
{
    fn	PostJob( &self, job: WorkPtr< '_>) 
    {
        let  	mut jobId = self.CurSuccId();
        jobId = self.ConstructJob( jobId, job);
        self.EnqueueJob( &mut jobId);
    }
    fn	AsRaw( &self) -> *const () 
    {
        self as *const Self as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
