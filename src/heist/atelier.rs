//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::Maestro;
use	crate::silo::{ Arr, Buff, ISlice, IArr, Stash, U16, U32 };
use	crate::stalks::{ Atm, Spinlock, WorkPtr };
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Atelier< 'a>
{
    _SzSchedJob: Atm< U32>,                                            // Count of cumulative jobs in flight
    _Maestros: Buff< Maestro< 'a>>,
    _SzPreds: Buff< Atm< U16>>,                                        // Count of predessors for job at the jobId
    _SuccIds: Buff< U16>,
    _FreeJobLock: Spinlock,
    _FreeJobStash: Stash< U16>,                                        // A Stack of free jobIds
    _JobBuff: Buff< WorkPtr< 'a>>,
    _Terminal: U16,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Atelier< 'a>
{

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( szMaestro: U32) -> Atelier< 'a>
    {
        let  	mut atelier = Self {
            _SzSchedJob: Atm::New( U32::_0),
            _Maestros: Buff::Create( szMaestro, Maestro::New),
            _SzPreds: Buff::Create( U32::_16Sz, |_i| Atm::New( U16::_0)),
            _SuccIds: Buff::< U16>::New( U32::_16Sz, U16::_0),
            _FreeJobLock: Spinlock::New(),
            _FreeJobStash: Stash::< U16>::New( U32::_16Sz, 0, U16( 0)),
            _JobBuff: Buff::New( U32::_16Sz, WorkPtr::null()),
            _Terminal: U16( 0),
        };
        atelier._FreeJobStash.DoIndexSetup();
        atelier._Terminal = atelier.ConstructJob( U32( 0), U16( 0), WorkPtr::null());
        atelier
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MainMaestro( &self) -> &Maestro< 'a>
    {
        let  	maestro = self._Maestros.Arr().MutAt( U32( 0));
        maestro.SetAtelier( self);
        maestro
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Terminal( &self) -> U16
    {
        self._Terminal
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Maestros( &self) -> Arr< 'a, Maestro< 'a>>
    {
        self._Maestros.Arr()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	IncrSzSchedJob( &self, inc: U32) -> U32
    {
        self._SzSchedJob.FetchAdd( inc, Ordering::SeqCst)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	SzSchedJob( &self) -> U32
    {
        self._SzSchedJob.Load( Ordering::Acquire)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	IncrPredAt< K: Into< U16>>( &self, jobId: U16, inc: K) -> U16
    {
        self._SzPreds
            .Arr()
            .At( jobId)
            .FetchAdd( inc, Ordering::SeqCst)
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
                std::thread::yield_now();
                continue;
            }
            let  	_guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Export( &jobCacheStk, U32::_X);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	FreeJob( &self, maestroIdx: U32, mut jobId: U16) -> bool
    {
        let  	maestro = self._Maestros.Arr().At( maestroIdx);
        let  	jobCacheStk = maestro.JobCacheStk();
        loop {
            if jobCacheStk.SzVoid() != 0 && jobCacheStk.Push( &mut jobId) {
                return true;
            }
            let  	_guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Import( &jobCacheStk, U32::_X);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob( &self, maestroIdx: U32, succId: U16, job: WorkPtr< 'a>) -> U16
    {
        let  	jobId = self.AllocJob( maestroIdx);
        if jobId == 0 {
            return jobId;
        }
        self._JobBuff.Arr().SetAt( jobId, &job);
        self._SuccIds.Arr().SetAt( jobId, &succId);
        self.IncrPredAt( succId, 1);
        jobId
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, maestroIdx: U32, jobId: &mut U16)
    {
        self.IncrSzSchedJob( U32( 1));
        let  	maestro = self._Maestros.Arr().At( maestroIdx);
        maestro.EnqueueActiveJob( jobId);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	GrabJob( &self, idx: U32, stealSeed: &mut u32) -> U16
    {
        let  	maestros = self._Maestros.Arr();
        let  	sz = maestros.len() as u32;
        *stealSeed = stealSeed.wrapping_mul( 2654435761).wrapping_add( 1);
        for mIdx in 0..sz {
            let  	maestroIdx = U32( stealSeed.wrapping_add( mIdx) % sz);
            if maestroIdx == idx {
                continue;
            }
            let  	maestro = maestros.At( maestroIdx);
            let  	jobId = maestro.PopJob();
            if jobId != 0 {
                return jobId;
            }
        }
        U16( 0)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ExecuteLoop( &self, maestroIdx: U32)
    {
        let  	maestro = self._Maestros.Arr().MutAt( maestroIdx);
        maestro.SetAtelier( self );
        let  	mut jobId = U16( 0);
        let  	mut stealSeed = maestroIdx.AsU32();
        while self.SzSchedJob() != 0 {
            while jobId != 0 {
                maestro.SetCurSuccId( *self._SuccIds.Arr().At( jobId));
                let  	job = *self._JobBuff.Arr().At( jobId);
                assert!( !job.is_null(), "jobId {} is null!", jobId.AsU16());
                
                ( job.func)( job.data, maestro);                   // Run job
                self._JobBuff.Arr().SetAt( jobId, &WorkPtr::null());
                maestro.IncrSzProcessed( 1);
                maestro.FlushTempQueue();
                
                let  	_res = self.FreeJob( maestroIdx, jobId);
                let  	succId = maestro.CurSuccId();
                if succId != U16( 0) {
                    let  	szPred: U16 = self.IncrPredAt( succId, -U16( 1));
                    if szPred == U16( 1) {
                        jobId = succId;
                        self.IncrSzSchedJob( U32( 1));
                    } else {
                        jobId = U16::_0;
                    }
                } else {
                    jobId = U16::_0;
                }
                
                self.IncrSzSchedJob( -U32( 1));
            }
            jobId = maestro.PopJob();
            if jobId == 0 {
                jobId = self.GrabJob( maestroIdx, &mut stealSeed);
            }
            if jobId == 0 {
                std::hint::spin_loop();
                std::thread::yield_now();
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DoLaunch( &self)
    {
        {
            let  	maestro0 = self._Maestros.Arr().MutAt( U32( 0));
            maestro0.SetAtelier( self );
            maestro0.FlushTempQueue();
        }
        let  	maestros = self._Maestros.Arr();
        std::thread::scope( |s| {
            for maestroIdx in 1..maestros.len() {
                s.spawn( move || {
                    self.ExecuteLoop( U32( maestroIdx as u32));
                });
            }
            self.ExecuteLoop( U32( 0));
        });
        println!();
        print!( "Atelier[ ");
        maestros.USeg().Traverse( |maestroIdx| {
            print!( "( Maestro-{}: {})", maestroIdx, maestros.At( maestroIdx).SzProcessed());
        });
        println!( "]")
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------
