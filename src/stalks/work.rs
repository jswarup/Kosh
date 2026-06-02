//-- work.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::heist::maestro::Maestro;
use	crate::silo::arr::Arr;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IWork: Send + Sync
{
    fn	run( &mut self, worker: &dyn IWorker);
}

impl< F> IWork for F
where
    F: for<'r> FnMut( &'r (dyn IWorker + 'r)) + Send + Sync,
{
    fn	run( &mut self, worker: &dyn IWorker)
    {
        self( worker);
    }
}

pub type WorkFn< 'a> = dyn IWork + 'a;

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
        job.run( self);
    }

    fn	PostJobs( &self, mut jobs: Arr< '_, Box< WorkFn< '_>>>)
    {
        for job in jobs.iter_mut() {
            job.run( self);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
