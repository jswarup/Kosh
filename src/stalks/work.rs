//-- work.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::arr::Arr;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct RawPtr<T>(pub *mut T);

unsafe impl<T> Send for RawPtr<T> {}
unsafe impl<T> Sync for RawPtr<T> {}

impl<T> Clone for RawPtr<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for RawPtr<T> {}

impl<T> RawPtr<T> {
    pub unsafe fn	as_mut( &self) -> &mut T
    {
        unsafe { &mut *self.0 }
    }
    pub unsafe fn	as_ref( &self) -> &T
    {
        unsafe { &*self.0 }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type WorkFn<'a> = dyn FnMut(&dyn IWorker) + Send + Sync + 'a;

pub trait IWorker
{
    fn	Post( &self, jobs: Arr<'_, Box<WorkFn<'_>>>);
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
    fn	Post( &self, mut jobs: Arr<'_, Box<WorkFn<'_>>>)
    {
        for job in jobs.iter_mut() {
            job( self);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
