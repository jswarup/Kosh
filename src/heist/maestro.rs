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

    pub fn	ConstructJob( &self, succId: U16, job: impl IntoWorkPtr< 'a>) -> U16
    {
        self.Atelier().ConstructJob( self._Index, succId, job.IntoWorkPtr())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueTempJob( &self, jobId: &mut U16)
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
        })
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

    pub fn	EnqueRunJob( &self, jobId: &mut U16)
    {
        let  	_guard = self._RunQlock.Lock();
        assert!( self._RunQueue.Stk().Push( jobId), "RunQueue overflow!");
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
        let         jobStash = Stash::<U16>::New( U32( 1024), 0, U16( 0));
        let mut     jobStk = jobStash.Stk();
        let         opStash = Stash::<(ChildOp, U32)>::New( U32( 1024), 0, (ChildOp::None, U32( 0)));
        let         opStk = opStash.Stk();
        let mut     currentSucc = self.CurSuccId();

        node.DiveDf( &mut |probe, enterFlg| {
            let  	    curNode = probe.CurNode().unwrap();
            let         curOp = curNode.ChildOp();
            if enterFlg {
                if  curOp != ChildOp::None {
                    opStk.PushX( ( curOp, jobStk.Size()));
                    return;
                }
                let         job = curNode.Value().unwrap();
                let mut     jobId = self.ConstructJob( U16( 0), job);
                jobStk.Push( &mut jobId);
                return;
            }
            if  curOp == ChildOp::None {
                return;
            }
            let mut biOpTuple = ( ChildOp::None, U32( 0));
            let     _res =  opStk.Pop( &mut biOpTuple);
            let     opArr = opStk.Arr();
            assert!( biOpTuple.0 == curOp);
            let     parentOp = if opArr.Size() != 0 { opArr.Last().0 } else { ChildOp::None};
            if parentOp == biOpTuple.0  {
                return;
            }
            let     startSz = biOpTuple.1;
            assert!( jobStk.Size() - startSz != U32( 0));
            let     arr = jobStk.Arr().Subset( startSz, jobStk.Size() - startSz);
            jobStk.SetSize( startSz);
            if curOp == ChildOp::Less {
                arr.USeg().TraverseRev( |i| {
                    let     jobId = *arr.At( i);
                    self.Atelier().SetAfter( jobId, currentSucc);
                    currentSucc = jobId;
                });
                jobStk.PushX( currentSucc);
            } else {
                assert!( curOp == ChildOp::Bor);
                arr.USeg().TraverseRev( |i| {
                    let     jobId = *arr.At( i);
                    self.Atelier().SetAfter( jobId, currentSucc);
                });
                let     jobId = self.ConstructEnqueBulk( U16( 0), arr.into());
                jobStk.PushX( jobId);
            }
            return;
        });
        let      ( jobStash, succStash,     predStash) = self.Atelier().TraceJobs( jobStk.Arr());
        let     jobArr = jobStash.Stk().Arr();
        let     succArr = succStash.Stk().Arr();
        let     predArr = predStash.Stk().Arr();
        jobArr.USeg().Traverse( |i| {
            println!( "{} {} {}", *jobArr.At( i), *succArr.At( i), *predArr.At( i));
        });
        
        jobStk.Arr().Traverse( |jId| {
            let mut     jobId = *jId;
           // self.EnqueTempJob( &mut jobId);
        });
        return;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	PostNode1( &self, node: &DynINode< 'a>)
    {
        let         jobStash = Stash::<U16>::New( U32( 1024), 0, U16( 0));
        let  mut    succId = self.CurSuccId();
        let  mut    stk = jobStash.Stk();
        let mut     groupOp = ChildOp::None;
        node.DiveDf( &mut |probe, enterFlg| {
            let  	curNode = probe.CurNode().unwrap();
            let     curOp = curNode.ChildOp();
            if enterFlg {
                let mut jobId = if curOp == ChildOp::None  {
                    self.ConstructJob( U16( 0), curNode.Value().unwrap())
                } else {
                    U16( 0)
                };
                stk.Push( &mut jobId);
                return;
            }
            if curOp == ChildOp::Less {
                  //
            }
            match curOp {
                ChildOp::None => {
                    let  	job = curNode.Value().unwrap();
                    let mut jobId = self.ConstructJob( succId, job);
                    if groupOp == ChildOp::Less {
                        succId = jobId;
                    }
                    stk.Push( &mut jobId);
                }
                _ => {
                    if ( groupOp != ChildOp::None) && ( groupOp != curOp) && ( groupOp == ChildOp::Less) {
                        assert!( curOp == ChildOp::Bor);
                        let     arr = stk.Arr();
                        let     buff = Buff::Create( arr.Size(), |i| *arr.At( i));
                        succId = self.ConstructEnqueBulk( U16( 0),  buff);
                        stk.SetSize( U32( 0));
                    }
                    groupOp = curOp
                }
            }
        });
        let     arr = stk.Arr();
        arr.USeg().Traverse( |i| {
            let  	mut jobId = *arr.At( i);
            self.EnqueRunJob( &mut jobId);
        });
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IWorker for Maestro< 'a>
{
    fn	PostJob( &self, job: WorkPtr< '_>)
    {
        let  	mut jobId = self.CurSuccId();
        jobId = self.ConstructJob( jobId, job);
        self.EnqueTempJob( &mut jobId);
    }
    fn	AsRaw( &self) -> *const ()
    {
        self as *const Self as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
