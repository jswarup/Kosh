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
    pub     data: *mut (),
    pub     func: JobFn,
    _marker: PhantomData< &'a ()>,
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
            data: null_mut(),
            func: |_, _| {},
            _marker: PhantomData,
        }
    }
    pub fn	Dummy() -> Self
    {
        Self {
            data: 1 as *mut (),
            func: |_, _| {},
            _marker: PhantomData,
        }
    }
    pub fn	IsNull( &self) -> bool
    {
        self.data.is_null()
    }
    pub fn	FromRef< T: IWork + 'a>( inner: &'a mut T) -> Self
    {
        let  	data = inner as *mut T as *mut ();
        let  	func: JobFn = |dataPtr, worker| unsafe {
            let  	actual = &mut *( dataPtr as *mut T);
            actual.DoWork( worker);
        };
        Self {
            data,
            func,
            _marker: PhantomData,
        }
    }

    pub fn	DoWork( &self, worker: &DynIWorker< '_>)
    {
        (self.func)( self.data, worker);
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
            data,
            func,
            _marker: PhantomData,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
/// A dynamic trait object for an `IWorker`. Send + Sync are already supertraits of IWorker.
pub type DynIWorker< 'a> = dyn IWorker + 'a;

//---------------------------------------------------------------------------------------------------------------------------------
/// Represents an entity capable of receiving and executing jobs.
pub trait IWorker: Send + Sync
{
    fn	PostJob( &self, job: WorkPtr< '_>);
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
            ( job.func)( job.data, self);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
