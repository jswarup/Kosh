//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	std::ptr::null;
use	crate::heist::Atelier;
use	crate::silo::{ Arr, Buff, IAccess, Stash, Stk, U16, U32 };
use	crate::stalks::{ Atm, DynIWorker, IWorker, IntoWorkPtr, Spinlock, WorkPtr};
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maestro< 'a>
{
    _Index: U32,
    _Atelier: *const Atelier< 'a>,
    pub( crate) _SzProcessed: U32,
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
            _Atelier: null(),
            _SzProcessed: U32::_0,
            _JobCache: Stash::<U16>::New( U32( 256), 0, U16( 0)),
            _RunQueue: Stash::<U16>::New( U32( 1024),0, U16( 0)),
            _RunQlock: Spinlock::New(),
            _CurSuccId: Atm::New( U16::_0),
            _TempQueue: Stash::<U16>::New( U32( 64),0, U16( 0)),
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

    pub fn	FromWorker< 'w>( worker: &'w DynIWorker< '_>) -> &'w Self
    {
        let  	ptr = worker as *const DynIWorker< '_> as *const ();
        assert!( !ptr.is_null());
        unsafe { &*( ptr as *const Self) }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob( &self, succId: U16, job: impl IntoWorkPtr< 'a>, docStr: &'static str) -> U16
    {
        self.Atelier().ConstructJob( self._Index, succId, job.IntoWorkPtr(), docStr)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, jobId: U16)
    {
        let     res = self._TempQueue.Stk().Push( jobId);
        assert!( res);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructEnqueArr( &self, succId: U16, buff: Buff< U16>, docStr: &'static str) -> U16
    {
        self.ConstructJob( succId, move |worker: &DynIWorker< '_>| {
            let  	maestro = Maestro::FromWorker( worker);
            let  	arr = buff.Arr();
            arr.Traverse( |jobId| {
                maestro.EnqueueJob( *jobId);
            });
        }, docStr)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	JobCacheStk( &self) -> Stk< '_, '_, U16>
    {
        self._JobCache.Stk()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RunQueueArr( &self) -> Arr< '_, U16>
    {
        self._RunQueue.Stk().Arr()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FlushTempQueue( &self)
    {
        let  	arr = self._TempQueue.Stk().Arr();
        arr.USeg().Traverse( |i| {
            let  	mut jobId = *arr.At( i);
            if jobId != 0 {
                self.Atelier()._SzSchedJob.Add( U32( 1));
                self.EnqueRunJob( &mut jobId);
            }
        });
        self._TempQueue.Clear();
    }



    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueRunJob( &self, jobId: & U16)
    {
        let  	_guard = self._RunQlock.Lock();
        assert!( self._RunQueue.Stk().Push( *jobId), "RunQueue overflow!");
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

    pub fn	PostChoreTree< T: crate::heist::choretree::IChoreNode>( &self, node: &T)
    {
        let  	mut tails = Buff::NewEmpty();
        let  	head = node.Post( self, &mut tails);
        let  	succId = self.CurSuccId();
        while let  	Some( tail) = tails.Pop() {
            self.Atelier().SetSucc( tail, succId);
        }
        self.EnqueueJob( head);
    }

    //-----------------------------------------------------------------------------------------------------------------------------
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IWorker for Maestro< 'a>
{
    fn	PostJob( &self, job: WorkPtr< '_>)
    {
        let  	mut jobId = self.CurSuccId();
        jobId = self.ConstructJob( jobId, job, "PostJob");
        self.EnqueueJob( jobId);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
