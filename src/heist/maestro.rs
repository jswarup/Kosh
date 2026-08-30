//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	std::ptr::null;
use	std::sync::atomic::Ordering;
use	crate::heist::{ Atelier, IAtelier, choretree::IChoreNode };
use	crate::silo::{ Arr, Buff, IAccess, Stash, Stk, U16, U32 };
use	crate::stalks::{ Atm, DynIWorker, IWorker, IntoWorkPtr, Spinlock, WorkPtr };
use	crate::swarm::SwarmEngine;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IMaestro< 'a>: IWorker
{
    fn	Atelier( &self) -> &Atelier< 'a>;
    fn	MaestroIndex( &self) -> U32;
    fn	Swarm( &self) -> Option< &SwarmEngine>;
    fn	CurSuccId( &self) -> U16;

    fn	SetCurSuccId< K: Into< U16>>( &self, val: K)
    where
        Self: Sized;

    fn	ConstructJob< S: Into< U16>>( &self, succId: S, job: impl IntoWorkPtr< 'a>, docStr: &'static str) -> U16
    where
        Self: Sized;

    fn	EnqueueJob< J: Into< U16>>( &self, jobId: J)
    where
        Self: Sized;

    fn	ConstructEnqueArr< S: Into< U16>>( &self, succId: S, buff: Buff< U16>, docStr: &'static str) -> U16
    where
        Self: Sized,
    {
        self.ConstructJob( succId.into(), move |worker: &DynIWorker< '_>| {
            let  	maestro = Maestro::FromWorker( worker);
            let  	arr = buff.Arr();
            arr.Traverse( |jobId| {
                maestro.EnqueueJob( *jobId);
            });
        }, docStr)
    }

    fn	FlushTempQueue( &self);

    fn	PostChoreTree< T: IChoreNode>( &self, node: &T)
    where
        Self: Sized,
    {
        let  	mut tails = Stash::New();
        let  	head = node.Post( self, &mut tails);
        let  	succId = self.CurSuccId();
        while let  	Some( tail) = tails.Pop() {
            self.Atelier().SetSucc( tail, succId);
        }
        self.EnqueueJob( head);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maestro< 'a>
{
    _Index:       U32,
    _Atelier:     *const Atelier< 'a>,
    _SzProcessed: U32,
    _JobCache:    Stash< U16>,
    _RunQueue:    Stash< U16>,
    _RunQlock:    Spinlock,
    _CurSuccId:   Atm< U16>,
    _TempQueue:   Stash< U16>,
}

unsafe impl< 'a> Send for Maestro< 'a>
{ }

unsafe impl< 'a> Sync for Maestro< 'a>
{ }

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Maestro< 'a>
{
    pub fn	New< I: Into< U32>>( maestroInd: I) -> Self
    {
        Self {
            _Index: maestroInd.into(),
            _Atelier: null(),
            _SzProcessed: U32::_0,
            _JobCache: Stash::< U16>::Create( U32( 256), U32::_0, |_| U16::_0),
            _RunQueue: Stash::< U16>::Create( U32( 1024), U32::_0, |_| U16::_0),
            _RunQlock: Spinlock::New(),
            _CurSuccId: Atm::New( U16::_0),
            _TempQueue: Stash::< U16>::Create( U32( 64), U32::_0, |_| U16::_0),
        }
    }

    pub fn	FromWorker< 'w>( worker: &'w DynIWorker< '_>) -> &'w Self
    {
        let  	ptr = worker as *const DynIWorker< '_> as *const ();
        assert!( !ptr.is_null());
        unsafe { &*( ptr as *const Self) }
    }

    pub fn	SzProcessed( &self) -> U32
    {
        self._SzProcessed
    }

    pub( crate) fn	IncProcessed( &mut self)
    {
        self._SzProcessed += 1;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub( crate) fn	SetAtelier( &mut self, atelier: *const Atelier< 'a>)
    {
        self._Atelier = atelier;
    }

    pub( crate) fn	JobCacheStk( &self) -> Stk< '_, '_, U16>
    {
        self._JobCache.Stk()
    }

    pub( crate) fn	RunQueueArr( &self) -> Arr< '_, U16>
    {
        self._RunQueue.Stk().Arr()
    }

    pub( crate) fn	PopJob( &self) -> U16
    {
        let  	xStk = self._RunQueue.Stk();
        let  	mut jobId = U16( 0);
        if xStk.Size() != 0 {
            let  	_guard = self._RunQlock.Lock();
            if xStk.Size() != 0 && xStk.Pop( &mut jobId) {
                return jobId;
            }
        }
        return jobId;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IMaestro< 'a> for Maestro< 'a>
{
    fn	Atelier( &self) -> &Atelier< 'a>
    {
        unsafe { &*self._Atelier }
    }

    fn	MaestroIndex( &self) -> U32
    {
        self._Index
    }

    fn	Swarm( &self) -> Option< &SwarmEngine>
    {
        self.Atelier().Swarm()
    }

    fn	ConstructJob< S: Into< U16>>( &self, succId: S, job: impl IntoWorkPtr< 'a>, docStr: &'static str) -> U16
    {
        self.Atelier().ConstructJob( self._Index, succId.into(), job.IntoWorkPtr(), docStr)
    }

    fn	EnqueueJob< J: Into< U16>>( &self, jobId: J)
    {
        let  	res = self._TempQueue.Stk().Push( jobId.into());
        assert!( res);
    }

    fn	FlushTempQueue( &self)
    {
        let  	arr = self._TempQueue.Stk().Arr();
        let  	sz = arr.Size();
        if sz == U32::_0 {
            return;
        }
        let  	_guard = self._RunQlock.Lock();
        arr.USeg().Traverse( |i| {
            let  	jobId = *arr.At( i);
            if jobId != 0 {
                self.Atelier().IncSchedJob();
                assert!( self._RunQueue.Stk().Push( jobId), "RunQueue overflow!");
            }
        });
        self._TempQueue.ClearConcurrent();
    }

    fn	CurSuccId( &self) -> U16
    {
        self._CurSuccId.Load( Ordering::Acquire)
    }

    fn	SetCurSuccId< K: Into< U16>>( &self, val: K)
    {
        self._CurSuccId.Store( val, Ordering::Release);
    }
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
