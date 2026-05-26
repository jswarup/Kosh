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
    fn  IncrPredAt( &mut self, jobId: U16, inc : U16) -> U16;
    fn  GrabJob( &mut self) -> U16 ;
    fn  AllocJob( &mut self) -> U16 ;
    fn  AllocJobs( &mut self, stk: &mut Stk< U16>) -> U32;
    fn  FreeJobs( &mut self, stk: &mut Stk< U16>) -> U32;
    fn  IncrSzSchedJob( &mut self,  inc : U32) -> U32;
    fn  ExecuteJob( &mut self,  mavenInd : U16,  jobId : U16);
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

    pub fn AllocJob( &mut self) -> U16
    {
        loop {
            let mut     jobId = U16( 0);
            let	mut     stk = self._JobCache.Stk();
            if stk.Size() > 0 && stk.Pop( &mut jobId)
            {
                break jobId;
            }
            unsafe {
                if (*self._Atelier).AllocJobs(&mut stk) == 0
                {
                    break jobId;
                }
            }
        }
    }

    pub fn FreeJob( &mut self, jobId : &mut U16) -> bool
    {
        let mut     stk = self._JobCache.Stk();
        loop {
            if stk.SzVoid() != 0
            {
                stk.Push( jobId);
                return true;
            }
            unsafe {
                if (*self._Atelier).FreeJobs(&mut stk) == 0
                {
                    return false;
                }
            }
        }
    }

    pub fn IncrSzSchedJob( &mut self, inc: U32) -> U32
    {
        unsafe {
            return (*self._Atelier).IncrSzSchedJob( inc);
        }
    }

    pub fn IncrPredAt( &mut self, inc: U16) -> U16
    {
        unsafe {
            return (*self._Atelier).IncrPredAt( self._CurSuccId, inc);
        }
    }

    pub fn GrabJob( &mut self) -> U16
    {
        unsafe {
            return (*self._Atelier).GrabJob();
        }
    }

    pub fn ExecuteJob( &mut self, jobId: U16)
    {
        unsafe {
            (*self._Atelier).ExecuteJob( self._Index, jobId);
        }
    }
    pub fn EnqueueJob( &mut self, jobId : &mut U16)
    {
        self.IncrSzSchedJob( U32(1));
        let _guard = self._RunQlock.Lock();
        self._RunQueue.Stk().Push( jobId);
    }

    pub fn PopJob( &mut self)  -> U16
    {
        let mut xStk = self._RunQueue.Stk();
        let mut jobId = U16( 0);
        if xStk.Size() != 0
        {
            let _guard = self._RunQlock.Lock();
            if xStk.Size() != 0 && xStk.Pop( &mut jobId)
            {
                return jobId;
            }
        }
        return jobId;
    }


    pub fn ExecuteLoop(  &mut self)
    {
        while self.IncrSzSchedJob( U32( 0) ) != 0
        {
            let mut szPred = U16( 0);
            if self._CurSuccId != 0 {
                szPred = self.IncrPredAt( U16::_0  - U16( 1) );
            }
            let mut jobId = if szPred == 0 { self._CurSuccId } else { U16( 0) };
            if jobId == 0 {
                jobId = self.PopJob();
            }
            if jobId == 0 {
                jobId = self.GrabJob();
            }
            if jobId == 0 {
                return;
            }
            self.ExecuteJob( jobId );
        }
        println!("{}: {} Done", self._Index, self._SzProcessed);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
