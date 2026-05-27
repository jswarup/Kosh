//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------

use crate::heist::maven::{AtelierT, Maven, JobFn};
use crate::silo::atm::{Atm, Spinlock};
use crate::silo::buff::Buff;
use crate::silo::stash::Stash;
use crate::silo::stk::Stk;
use crate::silo::uint::{U16, U32};
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
            _Mavens: Buff::Create( szMaven, |_i| {
                Maven::New( std::ptr::null_mut::< Atelier>() as *mut dyn AtelierT)
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

    pub fn  Maven< 'a>( &self, mavenInd : U16) -> &'a mut Maven
    {
        self._Mavens.AsArr().At( mavenInd)
    }

    pub fn DoLaunch( &mut self)
    {
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
        let arr = self._SzPreds.AsArr();
        let old = *arr.At( jobId);
        let new = old + inc;
        arr.SetAt( jobId, &new);
        old
    }

    fn	AllocJob( &mut self) -> U16
    {
        let mut stk = self._JobStash.Stk();
        let mut jobId = U16( 0);
        if stk.Size() != 0 && stk.Pop( &mut jobId) {
        }
        jobId
    }

    fn	AllocJobs( &mut self, stk: &mut Stk< U16>) -> U32
    {
        let mut freeJobs = self._JobStash.Stk();
        freeJobs.Export( stk, U32::_X)
    }

    fn	FreeJobs( &mut self, stk: &mut Stk< U16>) -> U32
    {
        let mut freeJobs = self._JobStash.Stk();
        freeJobs.Import( stk, U32::_X)
    }

    fn	GrabJob( &mut self) -> U16
    {
        let mut jobId = U16( 0);
        for maven in self._Mavens.iter_mut() {
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

    fn	ExecuteJob( &mut self, mavenInd: U16, jId: U16)
    {
        let mut jobId = jId;
        loop {
            if jobId == 0 {
                return;
            }
            let succId;
            {
                let     maven = self.Maven( mavenInd);
                maven._CurSuccId = *self._SuccIds.AsArr().At( jobId);
                let     job = &mut self._JobBuff[jobId.as_usize()];
                job( maven);
                let     _res = maven.FreeJob( jobId);
                succId = maven._CurSuccId;
                maven._CurSuccId = U16::_0;
            }
            let     szPred = self.IncrPredAt( succId, -U16(1));
            jobId = if  szPred == 0 { succId } else { U16::_0};
            self.IncrSzSchedJob( -U32(1));
        };
    }

}

//---------------------------------------------------------------------------------------------------------------------------------
