//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------

use crate::heist::maven::{AtelierT, Maven, JobFn};
use crate::silo::atm::{Atm, Spinlock};
use crate::silo::{ buff::Buff, arr::Arr, stk::Stk, stash::Stash, uint::{U16, U32}};
use std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------
pub struct Atelier
{
    _StartCount: U32, // Count of Processing Queue started, used for startup and shutdown
    _SzSchedJob: Atm< U32>, // Count of cumulative scheduled jobs in Works and Queues
    _SzQueue: Atm< U32>,
    _Lock: Spinlock,
    _LockedMark: U32,

    _Mavens: Buff< Maven>,
    _SzPreds: Buff< U16>, // Count of predessors for job at the jobId
    _SuccIds: Buff< U16>,
    _JobStash: Stash< U16>, // A Stack of free jobIds
    _JobBuff: Buff< Box< JobFn>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Atelier
{
    pub fn	New( szMaven: U32 ) -> Self
    {
        let mut atelier = Self {

            _StartCount: U32::_0,
            _SzSchedJob: Atm::New( U32::_0),
            _SzQueue: Atm::New( U32::_0),
            _Lock: Spinlock::New(),
            _Mavens: Buff::Create( szMaven, | i| {
                Maven::New( std::ptr::null_mut::< Atelier>() as *mut dyn AtelierT, i)
            }),
            _LockedMark: U32::_0,

            _SzPreds: Buff::< U16>::New( U32::_16Sz, U16::_0),
            _SuccIds: Buff::< U16>::New( U32::_16Sz, U16::_0),
            _JobStash: Stash::< U16>::New( U32::_16Sz),
            _JobBuff: Buff::Create( U32::_16Sz, |_i| {
                let cb: Box< JobFn> = Box::new( |_m| {});
                cb
            }),
        };

        atelier._JobStash.DoIndexSetup();
        let atelier_ptr = &mut atelier as *mut dyn AtelierT;
        for i in 0..szMaven.as_usize() {
            atelier._Mavens[i as usize].SetAtelier( atelier_ptr);
        }
        atelier
    }

    pub fn	Mavens< 'a>( &self) -> Arr< 'a, Maven> { self._Mavens.Arr() }


    fn	AllocJob( &mut self, mavenInd: U32) -> U16
    {
        let  mavens = self._Mavens.Arr();
        let  jobCacheStk = mavens.At( mavenInd).JobCacheStk();

        let mut jobId = U16( 0);
        if jobCacheStk.Size() > 0 && jobCacheStk.Pop( &mut jobId) && jobId != 0 {
            return jobId
        }
        let freeStk = self._JobStash.Stk();

        if freeStk.Size() != 0 && freeStk.Export( &jobCacheStk, U32::_X) != 0 &&
                        jobCacheStk.Pop( &mut jobId) && jobId != 0 {
        }
        jobId
    }
    pub fn DoLaunch( &self)
    {
        let  mavens = self._Mavens.Arr();
        std::thread::scope(|s| {
            for mavenIdx in 1..mavens.len() {
                let     maven = mavens.At( mavenIdx);
                s.spawn(move || {
                    maven.ExecuteLoop();
                });
            }
        });
        mavens.At( 0).ExecuteLoop();
        print!( "DoLaunch Over")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AtelierT for Atelier
{
    fn	IncrSzSchedJob( &mut self, inc: U32) -> U32
    {
        self._SzSchedJob.FetchAdd( inc, Ordering::SeqCst)
    }

    fn	IncrPredAt( &mut self, jobId: U16, inc: U16) -> U16
    {
        let arr = self._SzPreds.Arr();
        let old = *arr.At( jobId);
        let new = old + inc;
        arr.SetAt( jobId, &new);
        old
    }


    fn	AllocJobs( &mut self, stk: &Stk< U16>) -> U32
    {
        let freeJobs = self._JobStash.Stk();
        freeJobs.Export( stk, U32::_X)
    }

    fn	FreeJobs( &mut self, stk: &Stk< U16>) -> U32
    {
        let freeJobs = self._JobStash.Stk();
        freeJobs.Import( stk, U32::_X)
    }

    fn	GrabJob( &self) -> U16
    {
        let mut jobId = U16( 0);
        for maven in self._Mavens.iter() {
            jobId = maven.PopJob();
            if jobId != 0 {
                return jobId;
            }
        }
        return jobId;
    }

    fn  StoreJob( &mut self, jobId: U16, job: Box< JobFn>)
    {
        self._JobBuff[jobId.as_usize()] = job;
    }

    fn	ExecuteJob( &mut self, mavenInd: U32, jId: U16)
    {
        let     maven = self.Mavens().At( mavenInd);
        let mut jobId = jId;
        loop {
            if jobId == 0 {
                return;
            }
            let succId;
            {
                maven.SetCurSuccId( *self._SuccIds.Arr().At( jobId));
                let     job = self._JobBuff.Arr().At( jobId);
                job( maven);
                let     _res = maven.FreeJob( jobId);
                succId = maven.CurSuccId();
                maven.SetCurSuccId( U16::_0);
            }
            let     szPred = self.IncrPredAt( succId, -U16(1));
            jobId = if  szPred == 0 { succId } else { U16::_0};
            self.IncrSzSchedJob( -U32(1));
        };
    }

}

//---------------------------------------------------------------------------------------------------------------------------------
