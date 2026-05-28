//-- maven.rs -----------------------------------------------------------------------------------------------------------------------

use crate::silo::atm::Spinlock;
use crate::silo::stash::Stash;
use crate::silo::stk::Stk;
use crate::silo::uint::{U16, U32};

//---------------------------------------------------------------------------------------------------------------------------------

pub type  JobFn = dyn FnMut( &mut Maven) + Send + Sync;

pub trait AtelierT
{
    fn	AllocJobs( &mut self, stk: &Stk< U16>) -> U32;
    fn	FreeJobs( &mut self, stk: &Stk< U16>) -> U32;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maven
{
    _Atelier: *mut dyn AtelierT,
    _Index: U32,
    _CurSuccId: U16,
    _SzProcessed: U32,
    _RunQueue: Stash< U16>,
    _RunQlock: Spinlock,
    _JobCache: Stash< U16>,
    _TJobSilo: Stash< U16>,
}

unsafe impl Send for Maven {}
unsafe impl Sync for Maven {}

//---------------------------------------------------------------------------------------------------------------------------------

impl Maven
{
    pub fn	New( atelier: *mut dyn AtelierT, mavenInd : U32) -> Self
    {
        Self {
            _Atelier: atelier,
            _Index: mavenInd,
            _CurSuccId: U16::_0,
            _SzProcessed: U32::_0,
            _RunQueue: Stash::< U16>::New( U32( 1024)),
            _RunQlock: Spinlock::New(),
            _JobCache: Stash::< U16>::New( U32( 64)),
            _TJobSilo: Stash::< U16>::New( U32( 1024)),
        }
    }

    pub fn	SetAtelier( &mut self, atelier: *mut dyn AtelierT)
    {
        self._Atelier = atelier;
    }

    pub fn	Index( &self) -> U32 { self._Index }

    pub fn	CurSuccId( &self) -> U16 { self._CurSuccId }

    pub fn	SetCurSuccId( &mut self, succId: U16) { self._CurSuccId = succId; }

    pub fn	JobCacheStk( &self) -> Stk< '_, '_, U16>
    {
        self._JobCache.Stk()
    }

    pub fn	SzProcessed( &self) -> U32 { self._SzProcessed }

    pub fn	IncrSzProcessed< K: Into< U32>>( &mut self, k: K)
    {
        self._SzProcessed = self._SzProcessed + k.into();
    }

    pub fn	AllocJob( &self) -> U16
    {
        loop {
            let mut jobId = U16( 0);
            let stk = self._JobCache.Stk();
            if stk.Size() > 0 && stk.Pop( &mut jobId) {
                break jobId;
            }
            unsafe {
                if ( *self._Atelier).AllocJobs( &stk) == 0 {
                    break jobId;
                }
            }
        }
    }

    pub fn	FreeJob( &self, jobId: U16) -> bool
    {
        let stk = self._JobCache.Stk();
        loop {
            let mut jId = jobId;
            if stk.SzVoid() != 0 {
                stk.Push( &mut jId);
                return true;
            }
            unsafe {
                if ( *self._Atelier).FreeJobs( &stk) == 0 {
                    return false;
                }
            }
        }
    }
 


    pub fn	ExecuteJob( &self, jobId: U16)
    {
    }
    pub fn	EnqueueJob( &mut self, jobId: &mut U16)
    {
        let _guard = self._RunQlock.Lock();
        self._RunQueue.Stk().Push( jobId);
    }

    pub fn	PopJob( &self) -> U16
    {
        let xStk = self._RunQueue.Stk();
        let mut jobId = U16( 0);
        if xStk.Size() != 0 {
            let _guard = self._RunQlock.Lock();
            if xStk.Size() != 0 && xStk.Pop( &mut jobId) {
                return jobId;
            }
        }
        return jobId;
    }



}

//---------------------------------------------------------------------------------------------------------------------------------
