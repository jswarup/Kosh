//-- work.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::heist::maestro::Maestro;
use	crate::silo::arr::Arr;

//---------------------------------------------------------------------------------------------------------------------------------

pub type WorkFn< 'a> = dyn FnMut( &dyn IWorker) + Send + Sync + 'a;

pub trait IWorker
{
    fn	PostJob( &self, job: Box< WorkFn< '_>>);

    fn	PostJobs( &self, jobs: Arr< '_, Box< WorkFn< '_>>>);

    fn	AsMaestro( &self) -> Option< &Maestro< '_>>
    {
        None
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
    fn	PostJob( &self, mut job: Box< WorkFn< '_>>)
    {
        job( self);
    }

    fn	PostJobs( &self, mut jobs: Arr< '_, Box< WorkFn< '_>>>)
    {
        for job in jobs.iter_mut() {
            job( self);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
