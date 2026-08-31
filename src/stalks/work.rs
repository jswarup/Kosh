//-- work.rs -------------------------------------------------------------------------------------------------------------------------
use	std::{ marker::PhantomData, ptr::null_mut };
//---------------------------------------------------------------------------------------------------------------------------------
/// Represents a unit of work that can be executed concurrently.
pub trait IWork: Send + Sync
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>);
}

impl< F> IWork for F
where
    F: for< 'r> FnMut( &'r DynIWorker< 'r>) + Send + Sync,
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        self( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Function pointer type for executing a type-erased job.
pub type JobFn = for< 'r> fn(data: *mut (), worker: &'r DynIWorker< 'r>);

/// A type-erased pointer to a job and its associated execution function.
#[derive( Copy, Clone)]
pub struct WorkPtr< 'a>
{
    pub     _Data: *mut (),
    pub     _Func: JobFn,
    _Marker: PhantomData< &'a ()>,
}

unsafe impl< 'a> Send for WorkPtr< 'a>
{ }
unsafe impl< 'a> Sync for WorkPtr< 'a>
{ }

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> WorkPtr< 'a>
{
    pub fn	Null() -> Self
    {
        Self {
            _Data: null_mut(),
            _Func: |_, _| {},
            _Marker: PhantomData,
        }
    }
    pub fn	Dummy() -> Self
    {
        Self {
            _Data: 1 as *mut (),
            _Func: |_, _| {},
            _Marker: PhantomData,
        }
    }
    pub fn	IsNull( &self) -> bool
    {
        self._Data.is_null()
    }
    pub fn	FromRef< T: IWork + 'a>( inner: &'a mut T) -> Self
    {
        let  	data = inner as *mut T as *mut ();
        let  	func: JobFn = |dataPtr, worker| unsafe {
            let  	actual = &mut *( dataPtr as *mut T);
            actual.DoWork( worker);
        };
        Self::New( data, func)
    }

    pub fn	New( data: *mut (), func: JobFn) -> Self
    {
        Self {
            _Data:   data,
            _Func:   func,
            _Marker: PhantomData,
        }
    }

    pub fn	DoWork( &self, worker: &DynIWorker< '_>)
    {
        (self._Func)( self._Data, worker);
    }


}
//---------------------------------------------------------------------------------------------------------------------------------
/// Trait for converting objects (like closures or `IWork` implementations) into a `WorkPtr`.
/// NOTE: `WorkPtr` must NOT implement `IWork`, otherwise the two blanket impls below would conflict.
pub trait IntoWorkPtr< 'a>
{
    fn	IntoWorkPtr( self) -> WorkPtr< 'a>;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IntoWorkPtr< 'a> for WorkPtr< 'a>
{
    fn	IntoWorkPtr( self) -> WorkPtr< 'a>
    {
        self
    }
}
impl< 'a, T> IntoWorkPtr< 'a> for T
where
    T: IWork + 'a,
{
    fn	IntoWorkPtr( self) -> WorkPtr< 'a>
    {
        let  	boxed = Box::new( self);
        let  	data = Box::into_raw( boxed) as *mut ();
        let  	func: JobFn = |dataPtr, worker| unsafe {
            let  	mut owned = Box::from_raw( dataPtr as *mut T);
            owned.DoWork( worker);
        };
        WorkPtr {
            _Data: data,
            _Func: func,
            _Marker: PhantomData,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
/// A dynamic trait object for an `IWorker`. Send + Sync are already supertraits of IWorker.
pub type DynIWorker< 'a> = dyn IWorker + 'a;

/// A dynamic trait object for an `IWork`.
pub type DynIWork< 'a> = dyn IWork + 'a;

//---------------------------------------------------------------------------------------------------------------------------------
/// Represents an entity capable of receiving and executing jobs.
pub trait IWorker: Send + Sync
{
    fn	PostJob( &self, job: WorkPtr< '_>);

    // Allows unsafe downcasting to the underlying worker type (e.g. Parser)
    fn	AsRawWorker( &self) -> *const () { std::ptr::null() }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl DynIWorker< '_>
{
    pub fn	Post< 'a, J: IntoWorkPtr< 'a>>( &self, job: J)
    {
        self.PostJob( job.IntoWorkPtr());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
/// A simple, sequential implementation of `IWorker` that executes jobs immediately on the current thread.
pub struct Worker;
impl Worker
{
    pub fn	New() -> Self
    {
        Self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWorker for Worker
{
    fn	PostJob( &self, job: WorkPtr< '_>)
    {
        if !job.IsNull() {
            ( job._Func)( job._Data, self);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
