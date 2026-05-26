//-- maven.rs -----------------------------------------------------------------------------------------------------------------------

use crate::silo::stk::Stk;
use crate::silo::uint::{U32, U16};
use crate::silo::stash::Stash;
use crate::silo::atm::Spinlock;

//---------------------------------------------------------------------------------------------------------------------------------
/// Trait to abstract Atelier

#[allow(dead_code)]
pub trait AtelierT
{
    fn  IncrPredAt( &mut self, jobId: u16, inc : u16) -> u16;
    fn  GrabJob( &mut self) -> u16 ;
    fn  AllocJob( &mut self) -> u16 ;
    fn  AllocJobs( &mut self, stk: &mut Stk< u16>) -> u32;
    fn  FreeJobs( &mut self, stk: &mut Stk< u16>) -> u32;
    fn  IncrSzSchedJob( &mut self,  inc : u32) -> u32;
    fn  ExecuteJob( &mut self,  mavenInd : u16,  jobId : u16);
}


pub struct Maven
{
    _Atelier: *mut dyn AtelierT,
    _Index: U16,
    _CurSuccId: U16,
    _SzProcessed : U32,
    _RunQueue : Stash< U16>,
    _RunQlock : Spinlock,
    _JobCache : Stash< U16>,
    _TJobSilo : Stash< U16>,
}

unsafe impl Send for Maven {}
unsafe impl Sync for Maven {}

//---------------------------------------------------------------------------------------------------------------------------------

impl  Maven
{
    pub fn New( atelier: *mut dyn AtelierT) -> Self
    {
        Self
        {
            _Atelier: atelier,
            _Index: U16::_X,
            _CurSuccId: U16::_0,
            _SzProcessed: U32::_0,
            _RunQueue: Stash::<U16>::New( U32( 1024)),
            _RunQlock: Spinlock::New(),
            _JobCache: Stash::<U16>::New( U32( 64)),
            _TJobSilo: Stash::<U16>::New( U32( 1024)),
        }
    }

    pub fn SetAtelier(&mut self, atelier: *mut dyn AtelierT) {
        self._Atelier = atelier;
    }

    pub fn Index( &self) ->U16 { self._Index}

    pub fn CurSuccId( &self) ->U16 { self._CurSuccId}

}

//---------------------------------------------------------------------------------------------------------------------------------
