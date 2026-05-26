//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------

use crate::silo::stk::Stk;
use crate::silo::buff::Buff;
use crate::silo::uint::{ U16, U32};
use crate::silo::stash::Stash;
use crate::silo::atm::{ Atm, Spinlock};
use crate::heist::maven::{AtelierT, Maven};

pub struct Atelier {
    _Mavens: Buff<Maven>,
    _StartCount: U32,                           // Count of Processing Queue started, used for startup and shutdown
    _SzSchedJob: Atm< U32>,                     // Count of cumulative scheduled jobs in Works and Queues
    _SzQueue: Atm< U32>,
    _Lock: Spinlock,
    _LockedMark: U32,
    _JobSilo: Stash< U16>,                      // A Stack of free jobIds

    _SzPreds: Buff< U16>,                       // Count of predessors for job at the jobId
    _SuccIds: Buff< U16>,                       // Successor job for the job at the jobId

}

impl Atelier {
    pub fn New() -> Self {
        let mut atelier = Self {
            _Mavens: Buff::Create(U32(16), |_i| Maven::New(std::ptr::null_mut::<Atelier>() as *mut dyn AtelierT)),
            _StartCount: U32::_0,
            _SzSchedJob: Atm::New(U32::_0),
            _SzQueue: Atm::New(U32::_0),
            _Lock: Spinlock::New(),
            _LockedMark: U32::_0,
            _JobSilo: Stash::<U16>::New(U32(1024)),
            _SzPreds: Buff::<U16>::New(U32(1024), U16::_0),
            _SuccIds: Buff::<U16>::New(U32(1024), U16::_0),
        };

        atelier._JobSilo.DoIndexSetup();
        let atelier_ptr = &mut atelier as *mut Atelier as *mut dyn AtelierT;
        for i in 0..16 {
            atelier._Mavens[i as usize].SetAtelier(atelier_ptr);
        }

        atelier
    }
}

impl AtelierT for Atelier {
    fn IncrPredAt(&mut self, _jobId: u16, _inc: u16) -> u16 {
        0
    }

    fn GrabJob(&mut self) -> u16 {
        0
    }

    fn AllocJob(&mut self) -> u16 {
        0
    }

    fn AllocJobs(&mut self, _stk: &mut Stk<u16>) -> u32 {
        0
    }

    fn FreeJobs(&mut self, _stk: &mut Stk<u16>) -> u32 {
        0
    }

    fn IncrSzSchedJob(&mut self, _inc: u32) -> u32 {
        0
    }

    fn ExecuteJob(&mut self, _mavenInd: u16, _jobId: u16) {
    }
}
