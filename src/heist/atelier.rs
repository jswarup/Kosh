//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::maestro::Maestro;
use	crate::heist::maven::Maven;
use	crate::silo::{
    arr::Arr,
    buff::Buff,
    stash::Stash,
    uint::
    { U16, U32 },
};
use	crate::stalks::atm::{ Atm, Spinlock };
use	crate::stalks::work::WorkPtr;
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Atelier< 'a>
{
    _SzSchedJob: Atm< U32>,                                            // Count of cumulative jobs in flight
    _Mavens: Buff< Maven>,
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

    pub fn	New( szMaven: U32) -> Atelier< 'a>
    {
        let  	mut atelier = Self {
            _SzSchedJob: Atm::New( U32::_0),
            _Mavens: Buff::Create( szMaven, Maven::New),
            _SzPreds: Buff::Create( U32::_16Sz, |_i| Atm::New( U16::_0)),
            _SuccIds: Buff::< U16>::New( U32::_16Sz, U16::_0),
            _FreeJobLock: Spinlock::New(),
            _FreeJobStash: Stash::< U16>::New( U32::_16Sz),
            _JobBuff: Buff::New( U32::_16Sz, WorkPtr::null()),
            _Terminal: U16( 0),
        };
        atelier._FreeJobStash.DoIndexSetup();
        atelier._Terminal = atelier.ConstructJob( U32( 0), U16( 0), WorkPtr::null());
        atelier
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MainMaestro( &self) -> Maestro< '_> 
    {
        Maestro::New( self, U32( 0))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Terminal( &self) -> U16
    {
        self._Terminal
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Mavens( &self) -> Arr< 'a, Maven>
    {
        self._Mavens.Arr()
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

    fn	AllocJob( &self, mavenIdx: U32) -> U16
    {
        let  	maven = self._Mavens.Arr().At( mavenIdx);
        let  	jobCacheStk = maven.JobCacheStk();
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

    fn	FreeJob( &self, mavenIdx: U32, mut jobId: U16) -> bool
    {
        let  	maven = self._Mavens.Arr().At( mavenIdx);
        let  	jobCacheStk = maven.JobCacheStk();
        loop {
            if jobCacheStk.SzVoid() != 0 && jobCacheStk.Push( &mut jobId) {
                return true;
            }
            let  	_guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Import( &jobCacheStk, U32::_X);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob( &self, mavenIdx: U32, succId: U16, job: WorkPtr< 'a>) -> U16
    {
        let  	jobId = self.AllocJob( mavenIdx);
        if jobId == 0 {
            return jobId;
        }
        self._JobBuff.Arr().SetAt( jobId, &job);
        self._SuccIds.Arr().SetAt( jobId, &succId);
        self.IncrPredAt( succId, 1);
        jobId
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, mavenIdx: U32, jobId: &mut U16)
    {
        self.IncrSzSchedJob( U32( 1));
        let  	maven = self._Mavens.Arr().At( mavenIdx);
        maven.EnqueueJob( jobId);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	GrabJob( &self, idx: U32, stealSeed: &mut u32) -> U16
    {
        let  	mavens = self._Mavens.Arr();
        let  	sz = mavens.len() as u32;
        *stealSeed = stealSeed.wrapping_mul( 2654435761).wrapping_add( 1);
        for mIdx in 0..sz {
            let  	mavenIdx = U32( ( *stealSeed + mIdx) % sz);
            if mavenIdx == idx {
                continue;
            }
            let  	maven = mavens.At( mavenIdx);
            let  	jobId = maven.PopJob();
            if jobId != 0 {
                return jobId;
            }
        }
        U16( 0)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ExecuteLoop( &self, mavenIdx: U32)
    {
        let  	maven = self._Mavens.Arr().MutAt( mavenIdx);
        let  	mut jobId = U16( 0);
        let  	mut stealSeed = mavenIdx.AsU32();
        while self.SzSchedJob() != 0 {
            while jobId != 0 {
                maven.SetCurSuccId( *self._SuccIds.Arr().At( jobId));
                let  	maestro = Maestro::New( self, mavenIdx);
                let  	job = *self._JobBuff.Arr().At( jobId);
                if !job.is_null() {
                    ( job.func)( job.data, &maestro);   // Run job
                    self._JobBuff.Arr().SetAt( jobId, &WorkPtr::null());
                    maven.FlushTempQueue( self, mavenIdx);
                }
                maven.IncrSzProcessed( 1);
                let  	_res = self.FreeJob( mavenIdx, jobId);
                let  	succId = maven.CurSuccId();
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
            jobId = maven.PopJob();
            if jobId == 0 {
                jobId = self.GrabJob( mavenIdx, &mut stealSeed);
            }
            if jobId == 0 {
                std::thread::yield_now();
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DoLaunch( &self) 
    {
        self._Mavens.Arr().At( U32( 0)).FlushTempQueue( self, U32( 0));
        let  	mavens = self._Mavens.Arr();
        std::thread::scope( |s| {
            for mavenIdx in 1..mavens.len() {
                s.spawn( move || {
                    self.ExecuteLoop( U32( mavenIdx as u32));
                });
            }
            self.ExecuteLoop( U32( 0));
        });
        println!();
        print!( "Atelier[ ");
        mavens.USeg().Traverse( |mavenIdx| {
            print!( "( Maven-{}: {})", mavenIdx, mavens.At( mavenIdx).SzProcessed());
        });
        println!( "]")
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------
