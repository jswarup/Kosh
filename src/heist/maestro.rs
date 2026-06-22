//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::Atelier;
use	crate::silo::{ Buff, IAccess, IArr, Stash, Stk, USeg, U16, U32 };
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
        let  	tailStash = Stash::<U16>::New( U32( 1024), 0, U16( 0));                 
        let  	tailStk = tailStash.Stk();
        let  	jobStash = Stash::<(U16, USeg)>::New( U32( 1024), 0, (U16( 0), USeg::New( 0, 0)));
        let  	mut jobStk = jobStash.Stk();
        let  	opStash = Stash::<(ChildOp, U32)>::New( U32( 1024), 0, (ChildOp::None, U32( 0)));
        let  	opStk = opStash.Stk();
        let  	currentSucc = self.CurSuccId();

        node.DiveDf( &mut |probe, enterFlg| {
            let  	curNode = probe.CurNode().unwrap();
            let  	curOp = curNode.ChildOp();
            if enterFlg { 
                if  curOp != ChildOp::None {
                    opStk.Push( ( curOp, jobStk.Size()));
                    return;
                }
                let  	job = curNode.Value().unwrap();
                let  	docStr = curNode.DocStr();
                let  	jobId = self.ConstructJob( U16( 0), job,  docStr); 
                jobStk.Push( (jobId, USeg::New( tailStk.Size(), 1)));
                tailStk.Push( jobId);
                return;
            }
            // Post-visit: Leaf nodes have already been pushed on entry.
            if  curOp == ChildOp::None {                                
                return;
            }
            let  	mut biOpTuple = ( ChildOp::None, U32( 0));
            let  	_res =  opStk.Pop( &mut biOpTuple);
            let  	opArr = opStk.Arr();
            assert!( biOpTuple.0 == curOp);
            let  	parentOp = if opArr.Size() != 0 { opArr.Last().0 } else { ChildOp::None}; 
            if parentOp == biOpTuple.0  {
                return;
            }

            let  	startSz = biOpTuple.1;
            assert!( jobStk.Size() - startSz != U32( 0));
            let  	arr = jobStk.Arr().Subset( startSz, jobStk.Size() - startSz);
            let     arrSz = arr.Size();
            jobStk.SetSize( startSz);
            if curOp == ChildOp::Less {  
                USeg::New( 0, arrSz -1).Traverse( |i| { 
                    let  	headNext = arr.At( i +1).0;
                    arr.At( i).1.Traverse( |tailIdx| {
                        self.Atelier().SetSucc( *tailStk.Arr().At( tailIdx), headNext);
                    });
                }); 
                jobStk.Push( ( arr.First().0, arr.Last().1));
            } 
            if curOp == ChildOp::Bor {  
                let  	mut headsBuff = Buff::<U16>::NewEmpty();
                let  	newTailStart = tailStk.Size();
                let  	mut newTailSz = U32( 0);
                arr.USeg().Traverse( |i| {
                    let  	(head, tails) = *arr.At( i);
                    headsBuff.Push( head);
                    tails.Traverse( |tailIdx| {
                        tailStk.Push( *tailStk.Arr().At( tailIdx));
                        newTailSz = newTailSz + 1;
                    });
                });
                let  	enqueJobId = self.ConstructEnqueArr( U16( 0), headsBuff.clone(), "BorEnq");  
                let  	newTails = USeg::New( newTailStart, newTailSz);
                jobStk.Push( (enqueJobId, newTails));
            }
            return;
        });
         
        jobStk.Arr().Traverse( |j| { 
            j.1.Traverse( |tailIdx| {
                self.Atelier().SetSucc( *tailStk.Arr().At( tailIdx), currentSucc); 
            });
        });  
        jobStk.Arr().Traverse( |j| { 
            self.EnqueueJob( j.0);
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
        self.EnqueueJob( jobId);
    }
    fn	AsRaw( &self) -> *const ()
    {
        self as *const Self as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
