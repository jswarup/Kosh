//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	std::ptr::null;
use	std::sync::atomic::Ordering;
use	crate::heist::{ Atelier, choretree::IChoreNode };
use	crate::silo::{ Arr, Buff, IAccess, Stash, Stk, U16, U32 };
use	crate::stalks::{ Atm, DynIWorker, IWorker, IntoWorkPtr, Spinlock, WorkPtr };
use	crate::swarm::{ SwarmDevice, SwarmEngine };

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IMaestro< 'a>: IWorker
{
    fn	SetAtelier( &mut self, atelier: *const Atelier< 'a>);
    fn	Atelier( &self) -> &Atelier< 'a>;
    fn	MaestroIndex( &self) -> U32;
    fn	Swarm( &self) -> Option< &SwarmEngine>;
    fn	Device( &self) -> Option< &SwarmDevice>;

    fn	ConstructJob< S: Into< U16>>( &self, succId: S, job: impl IntoWorkPtr< 'a>, docStr: &'static str) -> U16
    where
        Self: Sized;

    fn	EnqueueJob< J: Into< U16>>( &self, jobId: J)
    where
        Self: Sized;

    fn	ConstructEnqueArr< S: Into< U16>>( &self, succId: S, buff: Buff< U16>, docStr: &'static str) -> U16
    where
        Self: Sized;

    fn	JobCacheStk( &self) -> Stk< '_, '_, U16>;
    fn	RunQueueArr( &self) -> Arr< '_, U16>;
    fn	FlushTempQueue( &self);
    fn	EnqueRunJob( &self, jobId: &U16);
    fn	PopJob( &self) -> U16;
    fn	CurSuccId( &self) -> U16;

    fn	SetCurSuccId< K: Into< U16>>( &self, val: K)
    where
        Self: Sized;

    fn	PostChoreTree< T: IChoreNode>( &self, node: &T)
    where
        Self: Sized;
}

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
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IMaestro< 'a> for Maestro< 'a>
{
    fn	SetAtelier( &mut self, atelier: *const Atelier< 'a>)
    {
        self._Atelier = atelier;
    }

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

    fn	Device( &self) -> Option< &SwarmDevice>
    {
        self.Swarm().map( |s| s.Device())
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

    fn	ConstructEnqueArr< S: Into< U16>>( &self, succId: S, buff: Buff< U16>, docStr: &'static str) -> U16
    {
        self.ConstructJob( succId.into(), move |worker: &DynIWorker< '_>| {
            let  	maestro = Maestro::FromWorker( worker);
            let  	arr = buff.Arr();
            arr.Traverse( |jobId| {
                maestro.EnqueueJob( *jobId);
            });
        }, docStr)
    }

    fn	JobCacheStk( &self) -> Stk< '_, '_, U16>
    {
        self._JobCache.Stk()
    }

    fn	RunQueueArr( &self) -> Arr< '_, U16>
    {
        self._RunQueue.Stk().Arr()
    }

    fn	FlushTempQueue( &self)
    {
        let  	arr = self._TempQueue.Stk().Arr();
        arr.USeg().Traverse( |i| {
            let  	mut jobId = *arr.At( i);
            if jobId != 0 {
                self.Atelier()._SzSchedJob.Add( U32( 1));
                self.EnqueRunJob( &mut jobId);
            }
        });
        self._TempQueue.ClearConcurrent();
    }

    fn	EnqueRunJob( &self, jobId: &U16)
    {
        let  	_guard = self._RunQlock.Lock();
        assert!( self._RunQueue.Stk().Push( *jobId), "RunQueue overflow!");
    }

    fn	PopJob( &self) -> U16
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

    fn	CurSuccId( &self) -> U16
    {
        self._CurSuccId.Load( Ordering::Acquire)
    }

    fn	SetCurSuccId< K: Into< U16>>( &self, val: K)
    {
        self._CurSuccId.Store( val, Ordering::Release);
    }

    fn	PostChoreTree< T: IChoreNode>( &self, node: &T)
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
