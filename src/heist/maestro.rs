//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::Atelier;
use	crate::silo::{ Buff, IAccess, IArr, Stash, Stk, U16, U32 };
use	crate::stalks::{ Atm, DynINode, DynIWorker, IWorker, IntoWorkPtr, Spinlock, WorkPtr, ChildOp};
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
        let  	ptr = worker.AsRaw();
        assert!( !ptr.is_null());
        unsafe { &*( ptr as *const Self) }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob( &self, succId: U16, job: impl IntoWorkPtr< 'a>, docStr: &'static str) -> U16
    {
        self.Atelier().ConstructJob( self._Index, succId, job.IntoWorkPtr(), docStr)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueTempJob( &self, jobId: U16)
    {
        assert!( self.TempQueueStk().Push( jobId));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructEnqueArr( &self, succId: U16, buff: Buff< U16>, docStr: &'static str) -> U16
    { 
        self.ConstructJob( succId, move |worker: &DynIWorker< '_>| {
            let  	maestro = Maestro::FromWorker( worker);
            let  	arr = buff.Arr();
            arr.USeg().Traverse( |i| {
                maestro.Atelier().EnqueRunJob( maestro._Index, arr.MutAt( i));
            });
        }, docStr)
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
                self.Atelier().EnqueRunJob( self._Index, &mut jobId);
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
     
    pub fn	PostNode( &self, node: &DynINode< 'a>)
    {
        let         jobStash = Stash::<(U16, U16)>::New( U32( 1024), 0, (U16( 0), U16( 0)));
        let mut     jobStk = jobStash.Stk();
        let         opStash = Stash::<(ChildOp, U32)>::New( U32( 1024), 0, (ChildOp::None, U32( 0)));
        let         opStk = opStash.Stk();
        let         currentSucc = self.CurSuccId();

        node.DiveDf( &mut |probe, enterFlg| {
            let  	    curNode = probe.CurNode().unwrap();
            let         curOp = curNode.ChildOp();
            if enterFlg {
                // Pre-visit: Push operator and current job stack size, or construct job for leaf nodes.
                if  curOp != ChildOp::None {
                    opStk.Push( ( curOp, jobStk.Size()));
                    return;
                }
                let         job = curNode.Value().unwrap();
                let         docStr = curNode.DocStr();
                let         jobId = self.ConstructJob( U16( 0), job,  docStr);
                jobStk.PushX( &mut (jobId, jobId));
                return;
            }
            // Post-visit: Leaf nodes have already been pushed on entry.
            if  curOp == ChildOp::None {                                
                return;
            }
            let mut biOpTuple = ( ChildOp::None, U32( 0));
            let     _res =  opStk.Pop( &mut biOpTuple);
            let     opArr = opStk.Arr();
            assert!( biOpTuple.0 == curOp);
            let     parentOp = if opArr.Size() != 0 { opArr.Last().0 } else { ChildOp::None};
            // Flatten identical adjacent operators (e.g. A < (B < C) becomes A < B < C)
            if parentOp == biOpTuple.0  {
                return;
            }
            let     startSz = biOpTuple.1;
            assert!( jobStk.Size() - startSz != U32( 0));
            let     arr = jobStk.Arr().Subset( startSz, jobStk.Size() - startSz);
            jobStk.SetSize( startSz);
            if curOp == ChildOp::Less {
                // Sequential: Chain jobs such that each completes before the next one starts.
                let     n = arr.Size().0;
                for i in 0..n-1 {
                    let  tail_i = arr.At( U32( i)).1;
                    let  head_next = arr.At( U32( i+1)).0;
                    self.Atelier().SetAfter( tail_i, head_next);
                }
                let     head = arr.At( U32( 0)).0;
                let     tail = arr.At( U32( n-1)).1;
                
                let mut traceBuff = Buff::<U16>::NewEmpty();
                arr.USeg().Traverse( |i| { traceBuff.Push( arr.At( i).0); });
                println!( "{}: {} {}", curOp, head, self.Atelier().TraceJobs( traceBuff.Arr()));
                jobStk.Push( (head, tail));
            } 
            if curOp == ChildOp::Bor { 
                // Parallel: All jobs run concurrently and share the same successor.
                let     joinJobId = self.ConstructJob( U16( 0), |_worker: &DynIWorker< '_>| {}, "Join");
                let mut headsBuff = Buff::<U16>::NewEmpty();
                arr.USeg().Traverse( |i| {
                    let  (head, tail) = *arr.At( i);
                    headsBuff.Push( head);
                    self.Atelier().SetAfter( tail, joinJobId);
                });
                let     enqueJobId = self.ConstructEnqueArr( U16( 0), headsBuff.clone(), "BorEnq"); 
                println!( "{}: {} {}", curOp, enqueJobId, self.Atelier().TraceJobs( headsBuff.Arr()));
                jobStk.Push( (enqueJobId, joinJobId));
            }
            return;
        });
        
        let mut traceBuff = Buff::<U16>::NewEmpty();
        jobStk.Arr().Traverse( |j| { 
            self.Atelier().SetAfter( j.1, currentSucc);
            traceBuff.Push( j.0);
        });
        println!( "{}", self.Atelier().TraceJobs( traceBuff.Arr()));
        
        jobStk.Arr().Traverse( |j| { 
            self.EnqueTempJob( j.0);
        });
        return;
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
        self.EnqueTempJob( jobId);
    }
    fn	AsRaw( &self) -> *const ()
    {
        self as *const Self as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
