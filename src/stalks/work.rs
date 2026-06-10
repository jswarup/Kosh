//-- work.rs -------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IWork: Send + Sync {
    fn	DoWork( &mut self, worker: &dyn IWorker);
}
impl< F> IWork for F
where
    F: for< 'r> FnMut( &'r ( dyn IWorker + 'r)) + Send + Sync,
    {
    fn	DoWork( &mut self, worker: &dyn IWorker) 
    {
        self( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type WorkFn< 'a> = dyn IWork + 'a;

pub type JobFn = for< 'r > fn( data: *mut (), worker: &'r ( dyn IWorker + 'r ) );

#[derive( Copy, Clone )]
pub struct JobPtr< 'a> {
    pub data: *mut (),
    pub func: JobFn,
    _marker: std::marker::PhantomData< &'a () >,
}

unsafe impl< 'a> Send for JobPtr< 'a> {}
unsafe impl< 'a> Sync for JobPtr< 'a> {}

impl< 'a> JobPtr< 'a> {
    pub fn	null() -> Self 
    {
        Self {
            data: std::ptr::null_mut(),
            func: |_, _| {},
            _marker: std::marker::PhantomData,
        }
    }
    pub fn	is_null( &self) -> bool 
    {
        self.data.is_null()
    }
    pub fn	FromRef< T: IWork + 'a>( inner: &'a mut T) -> Self 
    {
        let  	data = inner as *mut T as *mut ();
        let  	func: JobFn = |data_ptr, worker| unsafe {
            let  	actual = &mut *( data_ptr as *mut T );
            actual.DoWork( worker );
        };
        Self {
            data,
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

pub trait IntoJobPtr< 'a> 
{
    fn	into_job_ptr( self) -> JobPtr< 'a>;
}

impl< 'a> IntoJobPtr< 'a> for JobPtr< 'a> 
{
    fn	into_job_ptr( self) -> JobPtr< 'a> 
    {
        self
    }
}

impl< 'a, T> IntoJobPtr< 'a> for T 
where
    T: IWork + 'a,
{
    fn	into_job_ptr( self) -> JobPtr< 'a> 
    {
        AutoFreeJob::New( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct AutoFreeJob< T: IWork> 
{
    _Inner: T,
    _Ptr: *mut Self,
}

unsafe impl< T: IWork> Send for AutoFreeJob< T> {}
unsafe impl< T: IWork> Sync for AutoFreeJob< T> {}

impl< T: IWork> IWork for AutoFreeJob< T> 
{
    fn	DoWork( &mut self, worker: &dyn IWorker) 
    {
        self._Inner.DoWork( worker);
        unsafe {
            let _owned = Box::from_raw( self._Ptr);
        }
    }
}

impl< T: IWork> AutoFreeJob< T> 
{
    pub fn	New< 'a>( inner: T) -> JobPtr< 'a> 
    where
        T: 'a,
    {
        let mut boxed = Box::new( Self {
            _Inner: inner,
            _Ptr: std::ptr::null_mut(),
        });
        let ptr = &mut *boxed as *mut Self;
        boxed._Ptr = ptr;
        let data = Box::into_raw( boxed ) as *mut ();
        let func: JobFn = |data_ptr, worker| unsafe {
            let actual = &mut *( data_ptr as *mut Self );
            actual.DoWork( worker );
        };
        JobPtr {
            data,
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IWorker {
    fn	PostJob( &self, job: JobPtr< '_>);
    fn	AsRaw( &self) -> *const () 
    {
        std::ptr::null()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

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
    fn	PostJob( &self, job: JobPtr< '_>) 
    {
        if !job.is_null() {
            ( job.func)( job.data, self);
        }
    }
    fn	AsRaw( &self) -> *const () 
    {
        self as *const Self as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
