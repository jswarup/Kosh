//-- corochore.rs ---------------------------------------------------------------------------------------------------------------------

use	crate::{
    heist::{ ChoreTarget, IChoreNode, IMaestro },
    silo::{ Stash, U16, U32 },
    stalks::{ Coro, CoroRes, CoroYielder, DynIWorker, ICoro, IntoWorkPtr, IWork, WorkPtr },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone)]
pub struct WorkerFatPtr
{
    pub _Ptr: *const DynIWorker< 'static>,
}

unsafe impl Send for WorkerFatPtr {}
unsafe impl Sync for WorkerFatPtr {}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct CoroWork
{
    _Coro: Coro< WorkerFatPtr, (), ()>,
}

//---------------------------------------------------------------------------------------------------------------------------------

fn CoroJobFunc( dataPtr: *mut (), worker: &DynIWorker< '_>)
{
    unsafe {
        let  	mut owned = Box::from_raw( dataPtr as *mut CoroWork);
        // Transmute worker lifetime to 'static to pass through the fat pointer
        let  	workerPtr = WorkerFatPtr { _Ptr: std::mem::transmute::< &DynIWorker< '_>, *const DynIWorker< 'static>>( worker) };
        match owned._Coro.Resume( workerPtr) {
            CoroRes::Yield( _) => {
                let  	newData = Box::into_raw( owned) as *mut ();
                let  	job = WorkPtr::New( newData, CoroJobFunc);
                worker.PostJob( job);
            }
            CoroRes::Done( _) => {}
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IntoWorkPtr< 'a> for CoroWork
{
    fn	IntoWorkPtr( self) -> WorkPtr< 'a>
    {
        let  	boxed = Box::new( self);
        let  	data = Box::into_raw( boxed) as *mut ();
        WorkPtr::New( data, CoroJobFunc)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone)]
pub struct CoroChore
{
    pub _DocStr:  &'static str,
    pub _Target:  ChoreTarget,
    pub _Weight:  U32,
    pub _Closure: fn( CoroYielder< '_, WorkerFatPtr, ()>, WorkerFatPtr),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ICoroChore: IWork + IChoreNode
{
    fn	Target( &self) -> ChoreTarget;
    fn	DocStr( &self) -> &'static str;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CoroChore
{
    pub fn	New( f: fn( CoroYielder< '_, WorkerFatPtr, ()>, WorkerFatPtr)) -> Self
    {
        Self {
            _DocStr:  "",
            _Target:  ChoreTarget::Cpu,
            _Weight:  U32( 1),
            _Closure: f,
        }
    }
    pub fn	NewDoc( docStr: &'static str, f: fn( CoroYielder< '_, WorkerFatPtr, ()>, WorkerFatPtr)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::Cpu,
            _Weight:  U32( 1),
            _Closure: f,
        }
    }

    pub fn	WithWeight< W: Into< U32>>( mut self, weight: W) -> Self
    {
        self._Weight = weight.into();
        self
    }

    pub fn	Weight( &self) -> U32
    {
        self._Weight
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IChoreNode for CoroChore
{
    fn	Weight( &self) -> U32
    {
        self._Weight
    }

    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    where
        Self: 'a,
    {
        let  	closure = self._Closure;
        let  	coroWork = CoroWork {
            _Coro: Coro::New( move |yielder, input| closure( yielder, input)),
        };
        let  	jobId = maestro.ConstructJob( U16::_0, coroWork, self._DocStr);
        tails.Push( jobId);
        jobId
    }

    fn	Exec( &self, worker: &DynIWorker< '_>)
    {
        // For synchronous direct execution (e.g. fused subtrees):
        let  	closure = self._Closure;
        let  	mut coroWork = CoroWork {
            _Coro: Coro::New( move |yielder, input| closure( yielder, input)),
        };
        let  	workerPtr = WorkerFatPtr { _Ptr: unsafe { std::mem::transmute::< &DynIWorker< '_>, *const DynIWorker< 'static>>( worker) } };
        loop {
            match coroWork._Coro.Resume( workerPtr) {
                CoroRes::Yield( _) => {},
                CoroRes::Done( _) => break,
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for CoroChore
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        self.Exec( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ICoroChore for CoroChore
{
    fn	Target( &self) -> ChoreTarget
    {
        self._Target
    }

    fn	DocStr( &self) -> &'static str
    {
        self._DocStr
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! CoroChore {
    ( | $yielder:ident, $workerPtr:ident | $body:block ) => {
        $crate::heist::CoroChore::New( |$yielder, $workerPtr| $body )
    };
    ( $doc:expr, | $yielder:ident, $workerPtr:ident | $body:block ) => {
        $crate::heist::CoroChore::NewDoc( $doc, |$yielder, $workerPtr| $body )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! WeightedCoroChore {
    ( $weight:expr, $doc:expr, | $yielder:ident, $workerPtr:ident | $body:block ) => {
        $crate::heist::CoroChore::NewDoc( $doc, |$yielder, $workerPtr| $body ).WithWeight( $weight)
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
