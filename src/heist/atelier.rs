//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------

use crate::heist::maven::{AtelierT, Maven};
use crate::silo::atm::{Atm, Spinlock};
use crate::silo::buff::Buff;
use crate::silo::stash::Stash;
use crate::silo::stk::Stk;
use crate::silo::uint::{U16, U32};
use std::sync::atomic::Ordering;

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
    _JobBuffs: Buff< Box< dyn FnMut( &mut Maven) + Send + Sync>>,
}

impl Atelier
{
    pub fn	New() -> Self
    {
        let    mx = U32( 1 << 16);
        let mut atelier = Self {

            _StartCount: U32::_0,
            _SzSchedJob: Atm::New( U32::_0),
            _SzQueue: Atm::New( U32::_0),
            _Lock: Spinlock::New(),
            _Mavens: Buff::Create( U32( 16), |_i| {
                Maven::New( std::ptr::null_mut::< Atelier>() as *mut dyn AtelierT)
            }),
            _LockedMark: U32::_0,

            _SzPreds: Buff::< U16>::New( mx, U16::_0),
            _SuccIds: Buff::< U16>::New( mx, U16::_0),
            _JobStash: Stash::< U16>::New( mx),
            _JobBuffs: Buff::Create( mx, |_i| {
                let cb: Box< dyn FnMut( &mut Maven) + Send + Sync> = Box::new( |_m| {});
                cb
            }),
        };

        atelier._JobStash.DoIndexSetup();
        let atelier_ptr = &mut atelier as *mut Atelier as *mut dyn AtelierT;
        for i in 0..16 {
            atelier._Mavens[i as usize].SetAtelier( atelier_ptr);
        }

        atelier
    }
}

impl AtelierT for Atelier
{
    fn	IncrSzSchedJob( &mut self, inc: U32) -> U32
    {
        self._SzSchedJob.FetchAdd( inc, Ordering::SeqCst)
    }

    fn	IncrPredAt( &mut self, jobId: U16, inc: U16) -> U16
    {
        let idx = U32::from_U16( jobId);
        let arr = self._SzPreds.AsMutArr();
        let old = *arr.At( idx);
        let new = old + inc;
        arr.SetAt( idx, &new);
        old
    }

    fn	AllocJob( &mut self) -> U16
    {
        let mut stk = self._JobStash.Stk();
        let mut jobId = U16( 0);
        if stk.Size() != 0 && stk.Pop( &mut jobId) {
            jobId
        } else {
            jobId
        }
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

    fn	ExecuteJob( &mut self, mavenInd: U16, jobId: U16)
    {
        let     maven = self._Mavens.AsMutArr().At( U32::from_U16( mavenInd));
        //let     jobArr = self._JobBuff.AsMutArr();
        loop {
            if jobId != 0 {
                return;
            }
        };
    }
}
