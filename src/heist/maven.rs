//-- maven.rs -----------------------------------------------------------------------------------------------------------------------

use crate::silo::stk::Stk;

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

//---------------------------------------------------------------------------------------------------------------------------------
