//-- maven.rs -----------------------------------------------------------------------------------------------------------------------

use crate::silo::stk::Stk;
use crate::silo::uint::{U32, U16};
use crate::silo::stash::Stash;
use crate::silo::atm::SpinLock;

//---------------------------------------------------------------------------------------------------------------------------------
/// Trait to abstract Atelier

#[allow(dead_code)]
trait AtelierT
{
    fn  IncrPredAt( &mut self, jobId: u16, inc : u16) -> u16;
    fn  GrabJob( &mut self) -> u16 ;
    fn  AllocJob( &mut self) -> u16 ;
    fn  AllocJobs( &mut self, stk: &mut Stk< u16>) -> u32;
    fn  FreeJobs( &mut self, stk: &mut Stk< u16>) -> u32;
    fn  IncrSzSchedJob( &mut self,  inc : u32) -> u32;
    fn  ExecuteJob( &mut self,  mavenInd : u16,  jobId : u16);
}


struct Maven
{
    _Index: U16,
    _CurSuccId: U16,
    _SzProcessed : U32,
    _RunQueue : Stash< U16>,
    _RunQlock : SpinLock,
    _JobCache : Stash< U16>,
    _TJobSilo : Stash< U16>,
}

//---------------------------------------------------------------------------------------------------------------------------------
