//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------
use	std::sync::{ Arc, OnceLock };
use	std::sync::atomic::Ordering;
use	std::{ hint::spin_loop, thread::{ self, scope, yield_now } };
use	crate::heist::{ Maestro, IMaestro };
use	crate::silo::{ Arr, Buff, IAccess, IArr, Stash, USeg, U16, U32 };
use	crate::stalks::{ Atm, Spinlock, WorkPtr };
use	crate::swarm::SwarmEngine;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IAtelier< 'a>
{
    fn	MainMaestro( &self) -> &Maestro< 'a>;
    fn	Maestros( &self) -> Arr< 'a, Maestro< 'a>>;
    fn	FusionThres( &self) -> U32;
    fn	SetFusionThres< T: Into< U32>>( &mut self, val: T)
    where
        Self: Sized;
    fn	SetSwarm( &mut self, swarm: SwarmEngine);
    fn	Swarm( &self) -> Option< &SwarmEngine>;
    fn	SetSucc< J: Into< U16>, S: Into< U16>>( &self, jobId: J, succId: S)
    where
        Self: Sized;
    fn	ConstructJob< M: Into< U32>, S: Into< U16>>( &self, maestroIdx: M, succId: S, job: WorkPtr< 'a>, docStr: &'static str) -> U16
    where
        Self: Sized;
    fn	DoLaunch( &self);
    fn	SetWorkerCount< S: Into< U32>>( &self, newSz: S)
    where
        Self: Sized;
    fn	ActiveWorkers( &self) -> U32;
}

//---------------------------------------------------------------------------------------------------------------------------------

static GLOBAL_ATELIER: OnceLock< Atelier< 'static>> = OnceLock::new();

pub struct Atelier< 'a>
{
    _SzSchedJob:   Atm< U32>,                                           // Count of cumulative jobs in flight
    _Maestros:     Buff< Maestro< 'a>>,
    _SzPreds:      Buff< Atm< U16>>,                                    // Count of predecessors for job at the jobId
    _SuccIds:      Buff< U16>,
    _FreeJobLock:  Spinlock,
    _FreeJobStash: Stash< U16>,                                         // A Stack of free jobIds
    _JobBuff:      Buff< WorkPtr< 'a>>,
    _JobDocBuff:   Buff< &'static str>,
    _Terminal:     U16,
    _FusionThres:  U32,
    _Swarm:        Option< Arc< SwarmEngine>>,                          // Shared heterogeneous compute runtime instance
    _WorkerThreads:Buff< OnceLock< thread::Thread>>,
    _ActiveWorkers:Atm< U32>,
}


//---------------------------------------------------------------------------------------------------------------------------------

impl Atelier< 'static>
{
    pub fn	Init< S: Into< U32>>( szMaestro: S)
    {
        let  	sz = szMaestro.into();
        let  	isSet = GLOBAL_ATELIER.get().is_some();
        if !isSet {
            let  	atelier = Atelier::New( sz);
            if GLOBAL_ATELIER.set( atelier).is_ok() {
                let  	globalAtelier = GLOBAL_ATELIER.get().unwrap();
                globalAtelier._Maestros.Arr().USeg().Traverse( |mIdx| {
                    globalAtelier._Maestros.Arr().MutAt( mIdx).SetAtelier( globalAtelier);
                });
            }
        }
        let  	globalAtelier = GLOBAL_ATELIER.get().unwrap();
        globalAtelier.SetWorkerCount( sz);
    }

    pub fn	Get() -> &'static Atelier< 'static>
    {
        if GLOBAL_ATELIER.get().is_none() {
            Self::Init( U32( 0)); // Standardize on szMaestro = 0
        }
        GLOBAL_ATELIER.get().unwrap()
    }

    pub fn	Post( job: impl crate::stalks::IntoWorkPtr< 'static> + 'static)
    {
        let  	atelier = Self::Get();
        let  	maestroIdx = Maestro::GetCurrentIndex();
        let  	maestro = atelier._Maestros.Arr().MutAt( maestroIdx);
        let  	jobId = atelier.ConstructJob( maestroIdx, U16::_0, job.IntoWorkPtr(), "Post");
        maestro.EnqueueJob( jobId);
        maestro.FlushTempQueue();

        if atelier._ActiveWorkers.Load( Ordering::Acquire) == U32( 0) {
            atelier.ExecuteLoop( maestroIdx);
        } else {
            atelier.WakeWorker();
        }
    }

    pub fn	PostChoreTree( jobTree: impl crate::heist::choretree::IChoreNode + 'static)
    {
        let  	atelier = Self::Get();
        let  	maestroIdx = Maestro::GetCurrentIndex();
        let  	maestro = atelier._Maestros.Arr().MutAt( maestroIdx);
        maestro.PostChoreTree( &jobTree);
        maestro.FlushTempQueue();
        if atelier._ActiveWorkers.Load( Ordering::Acquire) == U32( 0) {
            atelier.ExecuteLoop( maestroIdx);
        } else {
            atelier.WakeWorker();
        }
    }

    pub fn	Wait()
    {
        let  	atelier = Self::Get();
        let  	maestroIdx = Maestro::GetCurrentIndex();
        atelier.ExecuteLoop( maestroIdx);
    }

    pub fn	Pump()
    {
        Self::Wait();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Atelier< 'a>
{
    pub fn	New< S: Into< U32>>( szMaestro: S) -> Atelier< 'a>
    {
        let  	sz = szMaestro.into();
        let  	maxSz = U32( 64);
        let  	mut atelier = Self {
            _SzSchedJob:   Atm::New( U32::_0),
            _Maestros:     Buff::Create( maxSz, Maestro::New),
            _SzPreds:      Buff::Create( U32::_16Sz, |_i| Atm::New( U16::_0)),
            _SuccIds:      Buff::< U16>::Create( U32::_16Sz, |_| U16::_0),
            _FreeJobLock:  Spinlock::New(),
            _FreeJobStash: Stash::< U16>::Create( U32::_16Sz, U32::_0, |_| U16::_0),
            _JobBuff:      Buff::Create( U32::_16Sz, |_| WorkPtr::Null()),
            _JobDocBuff:   Buff::Create( U32::_16Sz, |_| "Free"),
            _Terminal:     U16::_0,
            _FusionThres:  U32( 2),
            _Swarm:        None,
            _WorkerThreads:Buff::Create( maxSz, |_| OnceLock::new()),
            _ActiveWorkers:Atm::New( sz),
        };
        atelier._FreeJobStash.DoIndexSetup();
        atelier._Terminal = atelier.ConstructJob( U32::_0, U16::_0, WorkPtr::Dummy(), "Terminal");
        atelier._Maestros.Arr().MutAt( U32::_0).SetCurSuccId( atelier._Terminal);
        atelier
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub( crate) fn	IncSchedJob( &self)
    {
        self._SzSchedJob.Add( U32( 1));
    }

    pub( crate) fn	SuccIds( &self) -> Arr< '_, U16>
    {
        self._SuccIds.Arr()
    }

    pub( crate) fn	SzPred< J: Into< U16>>( &self, jobId: J) -> &Atm< U16>
    {
        self._SzPreds.Arr().At( jobId.into())
    }

    pub( crate) fn	JobDocBuff( &self) -> &Buff< &'static str>
    {
        &self._JobDocBuff
    }

    pub( crate) fn	FreeDocStr( &self) -> &'static str
    {
        let  	docStr = *self._JobDocBuff.Arr().At( 0);
        assert!( docStr == "Free");
        docStr
    }

    pub fn	Terminal( &self) -> U16
    {
        self._Terminal
    }


    pub fn	WithFusionThres< T: Into< U32>>( mut self, thres: T) -> Self
    {
        self._FusionThres = thres.into();
        self
    }

    pub( crate) fn	WakeWorker( &self)
    {
        let  	sz = self._ActiveWorkers.Load( Ordering::Acquire);
        if sz > U32( 1) {
            USeg::New( U32( 1), sz - U32( 1)).Traverse( |mIdx| {
                if let Some( t) = self._WorkerThreads.Arr().At( mIdx.AsU32()).get() {
                    t.unpark();
                }
            });
        }
    }


    //-----------------------------------------------------------------------------------------------------------------------------

    fn	AllocJob( &self, maestroIdx: U32) -> U16
    {
        let  	maestro = self._Maestros.Arr().At( maestroIdx);
        let  	jobCacheStk = maestro.JobCacheStk();
        loop {
            let  	mut jobId = U16( 0);
            if jobCacheStk.Size() != 0 && jobCacheStk.Pop( &mut jobId) {
                return jobId;
            }
            if self._FreeJobStash.Size() == 0 {
                yield_now();
                continue;
            }
            let  	_guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Export( &jobCacheStk, U32::_X);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	FreeJob( &self, maestroIdx: U32, mut jobId: U16) -> bool
    {
        self._JobDocBuff.Arr().SetAt( jobId, &self.FreeDocStr());

        let  	maestro = self._Maestros.Arr().At( maestroIdx);

        maestro.FlushTempQueue();
        let  	jobCacheStk = maestro.JobCacheStk();
        loop {
            if jobCacheStk.SzVoid() != 0 && jobCacheStk.PushX( &mut jobId) {
                return true;
            }
            let  	_guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Import( &jobCacheStk, U32::_X);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	GrabJob( &self, idx: U32, stealSeed: &mut u32) -> U16
    {
        let  	maestros = self._Maestros.Arr();
        let  	active = self._ActiveWorkers.Load( Ordering::Acquire);
        let  	sz = if active == U32( 0) { U32( 1) } else { active };
        let  	knuthMultHash = 2654435761u32;
        *stealSeed = stealSeed.wrapping_mul( knuthMultHash).wrapping_add( 1u32);
        let  	seed = *stealSeed;
        let  	mut foundJobId = U16::_0;
        USeg::New( U32::_0, sz).Traverse( |mIdx| {
            if foundJobId != 0 {
                return;
            }
            let  	maestroIdx = U32( ( seed.wrapping_add( mIdx.AsU32()) ) % sz.AsU32());
            if maestroIdx == idx {
                return;
            }
            let  	maestro = maestros.At( maestroIdx);
            let  	jobId = maestro.PopJob();
            if jobId != 0 {
                foundJobId = jobId;
            }
        });
        foundJobId
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	ExecuteLoop< M: Into< U32>>( &self, maestroIdx: M)
    {
        let  	mIdx = maestroIdx.into();
        let  	maestro = self._Maestros.Arr().MutAt( mIdx);
        maestro.SetAtelier( self);
        maestro.FlushTempQueue();
        let  	mut jobId = U16( 0);
        let  	mut stealSeed = mIdx.AsU32();
        while self._SzSchedJob.Load( Ordering::Acquire) != 0 {
            while jobId != 0 {
                maestro.SetCurSuccId( *self._SuccIds.Arr().At( jobId));
                let  	job = *self._JobBuff.Arr().At( jobId);
                assert!( !job.IsNull(), "jobId {} is null!", jobId.AsU16());

                ( job._Func)( job._Data, maestro);                   // Run job
                self._JobBuff.Arr().SetAt( jobId, &WorkPtr::Null());
                maestro.IncProcessed();

                let  	_res = self.FreeJob( mIdx, jobId);
                let  	succId = maestro.CurSuccId();
                if succId != U16( 0) {
                    let  	szPred: U16 = self.SzPred( succId).Add( -U16( 1));
                    if szPred == U16( 1) {
                        jobId = succId;
                        self._SzSchedJob.Add( U32( 1));
                    } else {
                        jobId = U16::_0;
                    }
                } else {
                    jobId = U16::_0;
                }

                self._SzSchedJob.Add( -U32( 1));
            }
            jobId = maestro.PopJob();
            if jobId == 0 {
                jobId = self.GrabJob( mIdx, &mut stealSeed);
            }
            if jobId == 0 {
                spin_loop();
                yield_now();
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IAtelier< 'a> for Atelier< 'a>
{
    fn	MainMaestro( &self) -> &Maestro< 'a>
    {
        let  	maestro = self._Maestros.Arr().MutAt( U32( 0));
        maestro.SetAtelier( self);
        maestro
    }

    fn	Maestros( &self) -> Arr< 'a, Maestro< 'a>>
    {
        self._Maestros.Arr()
    }

    fn	FusionThres( &self) -> U32
    {
        self._FusionThres
    }

    fn	SetFusionThres< T: Into< U32>>( &mut self, thres: T)
    {
        self._FusionThres = thres.into();
    }

    fn	SetSwarm( &mut self, swarm: SwarmEngine)
    {
        self._Swarm = Some( Arc::new( swarm));
    }

    fn	Swarm( &self) -> Option< &SwarmEngine>
    {
        self._Swarm.as_deref()
    }

    fn	SetSucc< J: Into< U16>, S: Into< U16>>( &self, jobId: J, succId: S)
    {
        let  	j = jobId.into();
        let  	s = succId.into();
        self._SuccIds.Arr().SetAt( j, &s);
        if s != U16::_0 {
            self.SzPred( s).Add( 1);
        }
    }

    fn	ConstructJob< M: Into< U32>, S: Into< U16>>( &self, maestroIdx: M, succId: S, job: WorkPtr< 'a>, docStr: &'static str) -> U16
    {
        let  	mIdx = maestroIdx.into();
        let  	sId = succId.into();
        let  	jobId = self.AllocJob( mIdx);
        if jobId == 0 {
            return jobId;
        }
        self._JobBuff.Arr().SetAt( jobId, &job);
        self._JobDocBuff.Arr().SetAt( jobId, &docStr);
        self.SzPred( jobId).Store( U16::_0, Ordering::Release);
        if sId != 0 {
            self.SetSucc( jobId, sId);
        } else {
            self._SuccIds.Arr().SetAt( jobId, &U16::_0);
        }
        jobId
    }

    fn	DoLaunch( &self)
    {
        let  	maestros = self._Maestros.Arr();
        let  	active = self._ActiveWorkers.Load( Ordering::Acquire);
        let  	sz = if active == U32( 0) { U32( 1) } else { active };
        scope( |s| {
            if sz > U32( 1) {
                USeg::New( U32( 1), sz - U32( 1)).Traverse( |maestroIdx| {
                    s.spawn( move || {
                        self.ExecuteLoop( maestroIdx);
                    });
                });
            }
            self.ExecuteLoop( U32( 0));
        });
        println!();
        print!( "Atelier[ ");
        USeg::New( U32( 0), sz).Traverse( |maestroIdx| {
            print!( "( Maestro-{}: {})", maestroIdx, maestros.At( maestroIdx).SzProcessed());
        });
        println!( "]");
    }

    fn	SetWorkerCount< S: Into< U32>>( &self, newSz: S)
    {
        let  	sz = newSz.into();
        let  	maxSz = U32( 64);
        let  	targetSz = if sz > maxSz { maxSz } else { sz };

        self._ActiveWorkers.Store( targetSz, Ordering::Release);

        if targetSz > U32( 1) {
            USeg::New( U32( 1), targetSz - U32( 1)).Traverse( |maestroIdx| {
                let  	threadSlot = self._WorkerThreads.Arr().At( maestroIdx.AsU32());
                if threadSlot.get().is_none() {
                    let  	handle = thread::spawn( move || {
                        Maestro::SetCurrentIndex( maestroIdx);
                        loop {
                            let  	active = Atelier::Get()._ActiveWorkers.Load( Ordering::Acquire);
                            if maestroIdx < active {
                                Atelier::Get().ExecuteLoop( maestroIdx);
                            }
                            thread::park();
                        }
                    });
                    threadSlot.set( handle.thread().clone()).unwrap();
                }
            });
        }
        self.WakeWorker();
    }

    fn	ActiveWorkers( &self) -> U32
    {
        self._ActiveWorkers.Load( Ordering::Acquire)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
