//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------
use	std::sync::atomic::Ordering;
use	crate::heist::maestro::Maestro;
use	crate::heist::maven::Maven;
use	crate::stalks::atm::{ Atm, Spinlock };
use	crate::stalks::work::WorkFn;
use	crate::silo::{
    arr::Arr,
    buff::Buff,
    stash::Stash,
    uint::
    { U16, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

type JobFn< 'a> = WorkFn< 'a>;
pub struct Atelier< 'a>
{
    _StartCount: U32,                                                  // Count of Processing Queue started, used for startup and shutdown
    _SzSchedJob: Atm< U32>,                                            // Count of cumulative jobs in flight
    _LockedMark: U32,
    _Mavens: Buff< Maven>,
    pub( crate) _SzPreds: Buff< Atm< U16>>,                            // Count of predessors for job at the jobId
    pub( crate) _SuccIds: Buff< U16>,
    _FreeJobLock: Spinlock,
    _FreeJobStash: Stash< U16>,                                        // A Stack of free jobIds
    _JobBuff: Buff< Box< JobFn< 'a>>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Atelier< 'a>
{
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( szMaven: U32) -> Atelier< 'a>
    {
		let  	atelier = Self {
            _StartCount: U32::_0,
            _SzSchedJob: Atm::New( U32::_0),
            _Mavens: Buff::Create( szMaven, Maven::New),
            _LockedMark: U32::_0,
            _SzPreds: Buff::Create( U32::_16Sz, |_i| Atm::New( U16::_0)),
            _SuccIds: Buff::<U16>::New( U32::_16Sz, U16::_0),
            _FreeJobLock: Spinlock::New(),
            _FreeJobStash: Stash::<U16>::New( U32::_16Sz),
            _JobBuff: Buff::Create( U32::_16Sz, |_i| {
			let  	cb: Box< JobFn< 'static>> = Box::new( |_m|
				{ });
                cb
            }),
        };
        atelier._FreeJobStash.DoIndexSetup();
        atelier
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MainMaestro( &self) -> Maestro< '_>
    {
        Maestro::New( self, U32( 0))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Mavens( &self) -> Arr<'a, Maven>
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
        self._SzPreds.Arr().At( jobId).FetchAdd( inc, Ordering::SeqCst)
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

    pub fn	ConstructJob( &self, mavenIdx: U32, succId: U16, mut jobBox: Box< JobFn< 'a>>) -> U16
    {
        let   jobId = self.AllocJob( mavenIdx);
        if jobId == 0 {
            return jobId;
        }
        self._JobBuff.Arr().MoveAt( jobId, &mut jobBox);

        self._SuccIds.Arr().SetAt( jobId, &succId);
        self.IncrPredAt( succId, 1);
        jobId
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, mavenIdx: U32, jobId: &mut U16)
    {
        self.IncrSzSchedJob( U32( 1));
        let     maven = self._Mavens.Arr().At( mavenIdx);
        maven.EnqueueJob( jobId);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	GrabJob( &self, idx: U32) -> U16
    {
		let  	mavens = self._Mavens.Arr();
        for mIdx in 0..mavens.len() {
			let  	mavenIdx = ( idx + mIdx + 1) % mavens.len();
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
        while self.SzSchedJob() != 0 {
            while jobId != 0 {
                maven.SetCurSuccId( *self._SuccIds.Arr().At( jobId));
				let  	maestro = Maestro::New( self, mavenIdx);
                self._JobBuff.Arr().MutAt( jobId)( &maestro);          // Run job
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
                jobId = self.GrabJob( mavenIdx);
            }
            if jobId == 0 {
                std::thread::yield_now();
            }
        }
        println!( "{}: {} Done", mavenIdx, maven.SzProcessed());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DoLaunch( &self)
    {
		let  	mavens = self._Mavens.Arr();
        std::thread::scope( |s| {
            for mavenIdx in 1..mavens.len() {
                s.spawn( move || {
                    self.ExecuteLoop( U32( mavenIdx as u32));
                });
            }
        });
        self.ExecuteLoop( U32( 0));
        print!( "DoLaunch Over")
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------
