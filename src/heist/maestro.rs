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

    pub fn	ConstructJob< K: Into<U16>>( &self, succId: K, job: impl IntoWorkPtr< 'a>, docStr: &'static str) -> U16
    {
        self.Atelier().ConstructJob( self._Index, succId.into(), job.IntoWorkPtr(), docStr)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueTempJob( &self, jobId: U16)
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
                maestro.EnqueTempJob( *jobId);
            });
        }, docStr)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	JobCacheStk( &self) -> Stk< '_, '_, U16>
    {
        self._JobCache.Stk()
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
     
    pub fn	PostChoreTree( &self, node: &DynINode< 'a>)
    {
        let         nodeStash = Stash::New( 1024, 1, ( node, U32( 0), U32( 0)));
        let         nodeStk = nodeStash.Stk();
        let         jobStash = Stash::< U16>::New( U32( 1024), 0, U16( 0));
        let  mut    jobStk = jobStash.Stk();
        let  mut    currentSucc = self.CurSuccId();
        while nodeStk.Size() > U32( 0) {
            let  	mut curr = ( node, U32( 0), U32( 0));
            let  	_res = nodeStk.Pop( &mut curr);
            let  	( curNode, idx, jobStkFirst) = curr;
            let  	szChild = curNode.Children().Size();
            if szChild == 0 {
                let         job = curNode.Value().unwrap();
                let         docStr = curNode.DocStr();
                let         jobId = self.ConstructJob( 0,  job,  docStr);
                jobStk.Push( jobId);
                continue;
            }  
            let     curOp = curNode.ChildOp();  
            if idx < szChild { 
                let     nxIdx = idx + U32( 1); 
                nodeStk.Push( ( curNode, nxIdx, jobStk.Size())); 
                let     k = if curOp == ChildOp::Less { szChild -idx -1} else { idx};
                let     child = curNode.Children().At( k);
                nodeStk.Push( ( child, U32( 0), jobStk.Size()));
                continue;
            }   
            // When all children been visited
            let     parentOp = if nodeStk.Size() != 0 { nodeStk.Arr().Last().0.ChildOp() } else { ChildOp::None} ;  
            if curOp == parentOp {
                continue;
            }
            let     startSz = if nodeStk.Size() != 0 { nodeStk.Arr().Last().2 } else { U32( 0)} ;  
            let     arr = jobStk.Arr().Subset( startSz, jobStk.Size() - startSz); 
            jobStk.SetSize( startSz);
            if curOp == ChildOp::Bor {
                arr.Traverse( | jobId| {
                    self.Atelier().SetSucc( *jobId, currentSucc);
                }); 
                let     head = self.ConstructEnqueArr( currentSucc, arr.into(), "BorEnq"); 
                println!( "{}: {} {}", curOp, head, self.Atelier().TraceJobs( arr));
                jobStk.Push( head);
            }
            if curOp == ChildOp::Less {
                arr.USeg().Traverse( | i| {
                    let     jobId = arr.At( i);
                    self.Atelier().SetSucc( *jobId, currentSucc);
                    currentSucc = *jobId;
                });
                println!( "{}: {} {}", curOp, currentSucc, self.Atelier().TraceJobs( arr));
                jobStk.Push( currentSucc);
            } 
                
        }
        jobStk.Arr().Traverse( | jobId| {
            let     oldSuccId = self.Atelier().SuccId( *jobId);
            if oldSuccId == 0 {
                self.Atelier().SetSucc( *jobId, self.CurSuccId());
            }
        }); 
        println!( "Enq: {}" , self.Atelier().TraceJobs( jobStk.Arr()));
        jobStk.Arr().Traverse( |j| { 
            self.EnqueTempJob( *j);
        });
        return; 
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
                    self.Atelier().SetSucc( tail_i, head_next);
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
                    self.Atelier().SetSucc( tail, joinJobId);
                });
                let     enqueJobId = self.ConstructEnqueArr( U16( 0), headsBuff.clone(), "BorEnq"); 
                println!( "{}: {} {}", curOp, enqueJobId, self.Atelier().TraceJobs( headsBuff.Arr()));
                jobStk.Push( (enqueJobId, joinJobId));
            }
            return;
        });
        
        let mut traceBuff = Buff::<U16>::NewEmpty();
        jobStk.Arr().Traverse( |j| { 
            self.Atelier().SetSucc( j.1, currentSucc);
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
