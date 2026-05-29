//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------

use crate::heist::maven::{ Maven, JobFn};
use crate::silo::atm::{Atm, Spinlock};
use crate::silo::{ buff::Buff, arr::Arr, stash::Stash, uint::{U16, U32}};
use std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------
pub struct Atelier
{
    _StartCount: U32,               // Count of Processing Queue started, used for startup and shutdown
    _SzSchedJob: Atm< U32>,         // Count of cumulative jobs in flight

    _LockedMark: U32,

    _Mavens: Buff< Maven>,
    _SzPreds: Buff< U16>,       // Count of predessors for job at the jobId
    _SuccIds: Buff< U16>,

    _FreeJobLock: Spinlock,
    _FreeJobStash: Stash< U16>, // A Stack of free jobIds

    _JobBuff: Buff< Box< JobFn>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Atelier
{
    pub fn	New( szMaven: U32 ) -> Self
    {
        let atelier = Self {

            _StartCount: U32::_0,
            _SzSchedJob: Atm::New( U32::_0),
            _Mavens: Buff::Create( szMaven, | i| {
                Maven::New( i)
            }),
            _LockedMark: U32::_0,

            _SzPreds: Buff::< U16>::New( U32::_16Sz, U16::_0),
            _SuccIds: Buff::< U16>::New( U32::_16Sz, U16::_0),
            _FreeJobLock: Spinlock::New(),
            _FreeJobStash: Stash::< U16>::New( U32::_16Sz),
            _JobBuff: Buff::Create( U32::_16Sz, |_i| {
                let cb: Box< JobFn> = Box::new( |_m| {});
                cb
            }),
        };

        atelier._FreeJobStash.DoIndexSetup();
        atelier
    }

    pub fn	Mavens< 'a>( &self) -> Arr< 'a, Maven> { self._Mavens.Arr() }


    fn	IncrSzSchedJob( &self, inc: U32) -> U32
    {
        self._SzSchedJob.FetchAdd( inc, Ordering::SeqCst)
    }

    fn	AllocJob( &self, mavenIdx: U32) -> U16
    {
        let     maven = self._Mavens.Arr().At( mavenIdx);
        let     jobCacheStk = maven.JobCacheStk(); 

        loop {
            let mut     jobId = U16( 0);
            if jobCacheStk.Size() != 0 && jobCacheStk.Pop( &mut jobId) {
                return jobId;
            } 
            let _guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Export( &jobCacheStk, U32::_X); 
        }
    }

    fn	FreeJob( &self, mavenIdx: U32, mut jobId : U16) -> bool
    {
        let     maven = self._Mavens.Arr().At( mavenIdx);
        let     jobCacheStk = maven.JobCacheStk(); 
        
        loop { 
            if jobCacheStk.SzVoid() != 0 && jobCacheStk.Push( &mut jobId) {
                return true;
            }
            let _guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Import( &jobCacheStk, U32::_X);
        }
    }

    fn	IncrPredAt( &self, jobId: U16, inc: U16) -> U16
    {
        let arr = self._SzPreds.Arr();
        let old = *arr.At( jobId);
        let new = old + inc;
        arr.SetAt( jobId, &new);
        old
    }



    pub fn	ConstructJob<F>( &self, mavenIdx: U32, jobFn : F) -> U16
    where
        F: FnMut( &mut Maven) + Send + Sync + 'static,
    {
        let     jobId = self.AllocJob( mavenIdx);
        if jobId == 0 {
            return jobId;
        }
        let mut jobBox: Box<JobFn> = Box::new( jobFn);
        self._JobBuff.Arr().MoveAt( jobId, &mut jobBox);
        return jobId;
    }

    pub fn	EnqueueJob( &self, mavenIdx: U32, jobId: &mut U16)
    {
        self.IncrSzSchedJob( U32( 1));
        self._Mavens.Arr().At( mavenIdx).EnqueueJob( jobId);
    }

    fn	GrabJob( &self, idx: U32) -> U16
    {
        let  mavens = self._Mavens.Arr();
        for mIdx in 0..mavens.len() {
            let     mavenIdx = ( idx + mIdx +1) % mavens.len();
            let     maven = mavens.At( mavenIdx);
            let     jobId = maven.PopJob();
            if jobId != 0 {
                return jobId;
            }
        }
        return U16( 0);
    }

    pub fn  FetchJob( &self, mavenIdx: U32) -> U16
    {
        let     maven = self._Mavens.Arr().MutAt( mavenIdx);
        let     curSuccId = maven.CurSuccId();
        if curSuccId != 0 {
            if self.IncrPredAt( curSuccId, -U16(1)) == 0 {
                return curSuccId;
            }
        }

        let     jobId = maven.PopJob();
        if jobId != 0 {
            return jobId;
        }
        self.GrabJob( mavenIdx)
    }

    fn	ExecuteJob( &self, mavenIdx: U32, jId: U16)
    {
        let     maven = self.Mavens().MutAt( mavenIdx);
        let mut jobId = jId;
        while jobId != 0 {
            let succId;
            {
                maven.SetCurSuccId( *self._SuccIds.Arr().At( jobId));       // for user-jobs
                self._JobBuff.Arr().MutAt( jobId)( maven);                          // Run job
                maven.IncrSzProcessed( 1);
                let     _res = self.FreeJob( mavenIdx, jobId);
                succId = maven.CurSuccId();
                maven.SetCurSuccId( U16::_0);
            }
            let     szPred = self.IncrPredAt( succId, -U16(1));
            jobId = if  szPred == 0 { succId } else { U16::_0};
            self.IncrSzSchedJob( -U32(1));
        };
    }
    pub fn	ExecuteLoop( &self, mavenIdx: U32)
    {
        let     maven = self._Mavens.Arr().MutAt( mavenIdx);
        while self.IncrSzSchedJob( U32( 0)) != 0 {
            let jobId = self.FetchJob( mavenIdx);
            if jobId != 0 {
                self.ExecuteJob( mavenIdx, jobId);
            }
        }
        println!( "{}: {} Done", mavenIdx, maven.SzProcessed());
    }

    pub fn DoLaunch( &self)
    {
        let  mavens = self._Mavens.Arr(); 

        std::thread::scope(|s| {
            for mavenIdx in 1..mavens.len() {
                s.spawn(move || {
                    self.ExecuteLoop( U32( mavenIdx as u32));
                });
            }
        });
        self.ExecuteLoop( U32( 0));
        print!( "DoLaunch Over")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
