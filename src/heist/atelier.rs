//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------
use	std::sync::atomic::Ordering;
use	crate::heist::maven::Maven;
use	crate::silo::atm::{ Atm, Spinlock };
use	crate::silo::{
    arr::Arr,
    buff::Buff,
    stash::Stash,
    uint::
    { U16, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

type JobFn = dyn FnMut( &Atelier) + Send + Sync;
pub struct Atelier
{
    _StartCount: U32, // Count of Processing Queue started, used for startup and shutdown
    _SzSchedJob: Atm< U32>, // Count of cumulative jobs in flight
    _LockedMark: U32,
    _Mavens: Buff< Maven>,
    pub( crate) _SzPreds: Buff< Atm< U16>>, // Count of predessors for job at the jobId
    pub( crate) _SuccIds: Buff< U16>,
    _FreeJobLock: Spinlock,
    _FreeJobStash: Stash< U16>, // A Stack of free jobIds
    _JobBuff: Buff< Box< JobFn>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Atelier
{

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( szMaven: U32) -> Self
    {
		let  	atelier = Self {
            _StartCount: U32::_0,
            _SzSchedJob: Atm::New( U32::_0),
            _Mavens: Buff::Create( szMaven, |i| Maven::New( i)),
            _LockedMark: U32::_0,
            _SzPreds: Buff::Create( U32::_16Sz, |_i| Atm::New( U16::_0)),
            _SuccIds: Buff::<U16>::New( U32::_16Sz, U16::_0),
            _FreeJobLock: Spinlock::New(),
            _FreeJobStash: Stash::<U16>::New( U32::_16Sz),
            _JobBuff: Buff::Create( U32::_16Sz, |_i| {
				let  	cb: Box< JobFn> = Box::new( |_m|
				{ });
                cb
            }),
        };
        atelier._FreeJobStash.DoIndexSetup();
        atelier
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Mavens< 'a>(&self) -> Arr<'a, Maven>
    {
        self._Mavens.Arr()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	IncrSzSchedJob( &self, inc: U32) -> U32
    {
        self._SzSchedJob.FetchAdd( inc, Ordering::SeqCst)
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
                std::hint::spin_loop();
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

    fn	IncrPredAt( &self, jobId: U16, inc: U16) -> U16
    {
		let  	arr = self._SzPreds.Arr();
        arr.At( jobId).FetchAdd( inc, Ordering::SeqCst)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob< F>( &self, mavenIdx: U32, jobFn: F) -> U16
    where
        F: FnMut( &Atelier) + Send + Sync + 'static,
    {
		let  	jobId = self.AllocJob( mavenIdx);
        if jobId == 0 {
            return jobId;
        }
		let  	mut jobBox: Box< JobFn> = Box::new( jobFn);
        self._JobBuff.Arr().MoveAt( jobId, &mut jobBox);
        return jobId;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, mavenIdx: U32, jobId: &mut U16)
    {
        self.IncrSzSchedJob( U32( 1));
        self._Mavens.Arr().At( mavenIdx).EnqueueJob( jobId);
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
        return U16( 0);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ExecuteLoop( &self, mavenIdx: U32)
    {
		let  	maven = self._Mavens.Arr().MutAt( mavenIdx);
		let  	mut jobId = U16( 0);
        while self.IncrSzSchedJob( U32( 0)) != 0 {
            while jobId != 0 {
                maven.SetCurSuccId( *self._SuccIds.Arr().At( jobId)); // for user-jobs
                self._JobBuff.Arr().MutAt( jobId)( self); // Run job
                maven.IncrSzProcessed( 1);
				let  	_res = self.FreeJob( mavenIdx, jobId);
				let  	succId = maven.CurSuccId();
                if succId != U16( 0) {
					let  	szPred = self.IncrPredAt( succId, -U16( 1));
                    if szPred == U16( 1) {
                        jobId = succId;
                        self.IncrSzSchedJob( U32( 1));
                    } else {
                        jobId = U16::_0;
                    }
                } else {
                    jobId = U16::_0;
                }
                maven.SetCurSuccId( U16::_0);
                self.IncrSzSchedJob( -U32( 1));
            }
            jobId = maven.PopJob();
            if jobId == 0 {
                jobId = self.GrabJob( mavenIdx);
            }
            if jobId == 0 {
                std::hint::spin_loop();
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
