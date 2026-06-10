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

#[derive( Copy, Clone )]
pub struct JobPtr< 'a>( pub Option< std::ptr::NonNull< WorkFn< 'a>>>);

unsafe impl< 'a> Send for JobPtr< 'a> {}
unsafe impl< 'a> Sync for JobPtr< 'a> {}

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
        let non_null = unsafe { std::ptr::NonNull::new_unchecked( Box::into_raw( boxed) as *mut WorkFn< 'a>) };
        JobPtr( Some( non_null ) )
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
        if let Some( mut non_null ) = job.0 {
            unsafe {
                non_null.as_mut().DoWork( self);
            }
        }
    }
    fn	AsRaw( &self) -> *const () 
    {
        self as *const Self as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
