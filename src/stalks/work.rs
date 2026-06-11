//-- work.rs -------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IWork: Send + Sync {
    fn	DoWork( &mut self, worker: &dyn IWorker);
}
impl< F> IWork for F
where
    F: for< 'r> FnMut(&'r ( dyn IWorker + 'r)) + Send + Sync,
{
    fn	DoWork( &mut self, worker: &dyn IWorker)
    {
        self( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type WorkFn< 'a> = dyn IWork + 'a;
pub type JobFn = for< 'r> fn(data: *mut (), worker: &'r ( dyn IWorker + 'r));
#[derive( Copy, Clone)]
pub struct WorkPtr< 'a> {
    pub data: *mut (),
    pub func: JobFn,
    _marker: std::marker::PhantomData< &'a ()>,
}
unsafe impl< 'a> Send for WorkPtr<'a>
{ }
unsafe impl< 'a> Sync for WorkPtr<'a>
{ }
impl< 'a> WorkPtr<'a>
{
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
    pub fn	FromRef< T: IWork + 'a>(inner: &'a mut T) -> Self
    {
        let  	data = inner as *mut T as *mut ();
        let  	func: JobFn = |dataPtr, worker| unsafe {
            let  	actual = &mut *( dataPtr as *mut T);
            actual.DoWork( worker);
        };
        Self {
            data,
            func,
            _marker: std::marker::PhantomData,
        }
    }
}
pub trait IntoWorkPtr< 'a> {
    fn	IntoWorkPtr( self) -> WorkPtr< 'a>;
}
impl< 'a> IntoWorkPtr<'a> for WorkPtr< 'a> {
    fn	IntoWorkPtr( self) -> WorkPtr< 'a> {
        self
    }
}
impl< 'a, T> IntoWorkPtr<'a> for T
where
    T: IWork + 'a,
{
    fn	IntoWorkPtr( self) -> WorkPtr< 'a> {
        WorkSlot::New( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct WorkSlot< T: IWork>
{
    _Inner: T,
}
unsafe impl< T: IWork> Send for WorkSlot< T>
{ }
unsafe impl< T: IWork> Sync for WorkSlot< T>
{ }
impl< T: IWork> IWork for WorkSlot< T>
{
    fn	DoWork( &mut self, worker: &dyn IWorker)
    {
        self._Inner.DoWork( worker);
        unsafe {
            let  	_owned = Box::from_raw( self as *mut Self);
        }
    }
}
impl< T: IWork> WorkSlot< T>
{
    pub fn	New< 'a>(inner: T) -> WorkPtr<'a>
    where
        T: 'a,
    {
        let  	boxed = Box::new( Self { _Inner: inner });
        let  	data = Box::into_raw( boxed) as *mut ();
        let  	func: JobFn = |dataPtr, worker| unsafe {
            let  	actual = &mut *( dataPtr as *mut Self);
            actual.DoWork( worker);
        };
        WorkPtr {
            data,
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IWorker {
    fn	PostJob( &self, job: WorkPtr< '_>);
    fn	AsRaw( &self) -> *const ()
    {
        std::ptr::null()
    }
    fn	IsSequential( &self) -> bool
    {
        false
    }
    fn	Tender< 'a, J: IntoWorkPtr<'a>>( &self, job: J)
    where
        Self: Sized
    {
        self.PostJob( job.IntoWorkPtr());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl dyn IWorker + '_ {
    pub fn	Post< 'a, J: IntoWorkPtr<'a>>( &self, job: J)
    {
        self.PostJob( job.IntoWorkPtr());
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
    fn	PostJob( &self, job: WorkPtr< '_>) {
        if !job.is_null() {
            ( job.func)( job.data, self);
        }
    }
    fn	AsRaw( &self) -> *const ()
    {
        self as *const Self as *const ()
    }
    fn	IsSequential( &self) -> bool
    {
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
