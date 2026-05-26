//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------

use crate::silo::stk::Stk;
use crate::silo::buff::Buff;
use crate::silo::uint::U32;
use crate::heist::maven::{AtelierT, Maven};

pub struct Atelier {
    _Mavens: Buff<Maven>,
}

impl Atelier {
    pub fn New() -> Self {
        Self {
            _Mavens: Buff::Create(U32::from(16), |_i| Maven::New(U32::_0)),
        }
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
