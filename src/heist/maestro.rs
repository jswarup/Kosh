//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::Atelier;
use	crate::silo::{ Buff, IAccess, IArr, Stash, Stk, U16, U32 };
use	crate::stalks::{ Atm, DynINode, DynIWorker, IWorker, IntoWorkPtr, Spinlock, WorkPtr, Worker, ChildOp};
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

    pub fn	ConstructEnqueBulk( &self, succId: U16, buff: Buff< U16>) -> U16
    {
        self.ConstructJob( succId, move |worker: &DynIWorker< '_>| {
            let  	maestro = Maestro::FromWorker( worker);
            let  	arr = buff.Arr();
            arr.USeg().Traverse( |i| {
                maestro.Atelier().EnqueRunJob( maestro._Index, arr.MutAt( i));
            });
        }, "EnqueBulk")
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
    // PostNode method:
    // This method traverses a dependency graph/tree (`DynINode`) using depth-first search (`DiveDf`) to convert it into
    // executable jobs with correct execution ordering (sequential vs parallel).
    //
    // - Leaf nodes (`ChildOp::None`): Constructed as jobs and pushed onto the `jobStk`.
    // - Operator nodes (`ChildOp::Less` for sequential, `ChildOp::Bor` for parallel):
    //   - On entry (pre-visit), the operator and the current size of `jobStk` are saved on `opStk`.
    //   - On exit (post-visit), the operator evaluates its children (all jobs added to `jobStk` since entry).
    //   - Optimization: If the parent operator is identical to the current one (`parentOp == curOp`), it delays
    //     processing to flatten the tree (e.g., A < (B < C) becomes A < B < C).
    //   - For Sequential (`ChildOp::Less`): Sets up a chain where each job must complete before the next begins. The first
    //     job in the chain is pushed back to `jobStk` to represent the entire sequence.
    //   - For Parallel (`ChildOp::Bor`): All jobs share the same successor. A "bulk" job is created to enqueue all these
    //     parallel jobs at once, and this bulk job is pushed back to `jobStk`.
    // - After traversal, any remaining jobs on `jobStk` are linked to the current Maestro successor (`self.CurSuccId()`).
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	PostNode( &self, node: &DynINode< 'a>)
    {
        let         jobStash = Stash::<U16>::New( U32( 1024), 0, U16( 0));
        let mut     jobStk = jobStash.Stk();
        let         opStash = Stash::<(ChildOp, U32)>::New( U32( 1024), 0, (ChildOp::None, U32( 0)));
        let         opStk = opStash.Stk();
        let mut     currentSucc = self.CurSuccId();

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
                let mut     jobId = self.ConstructJob( U16( 0), job, "PostNode");
                jobStk.PushX( &mut jobId);
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
                arr.USeg().TraverseRev( |i| {
                    let     jobId = *arr.At( i);
                    self.Atelier().SetAfter( jobId, currentSucc);
                    currentSucc = jobId;
                });
                jobStk.Push( currentSucc);
                self.Atelier().PrintTraceJobs( jobStk.Arr());
            } 
            if curOp == ChildOp::Bor { 
                // Parallel: All jobs run concurrently and share the same successor.
                arr.USeg().TraverseRev( |i| { 
                    self.Atelier().SetAfter( *arr.At( i), currentSucc);
                });
                // Create a bulk job to enqueue all parallel jobs at once.
                let     jobId = self.ConstructEnqueBulk( currentSucc, arr.into());
                jobStk.Push( jobId);
                self.Atelier().PrintTraceJobs( jobStk.Arr());
            }
            return;
        });
        jobStk.Arr().Traverse( |jId| { 
            self.Atelier().SetAfter( *jId, currentSucc);
        });
        self.Atelier().PrintTraceJobs( jobStk.Arr());
        
        jobStk.Arr().Traverse( |jobId| { 
            self.EnqueTempJob( *jobId);
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
