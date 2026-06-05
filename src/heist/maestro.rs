//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::atelier::Atelier;
use	crate::silo::{ arr::Arr, buff::Buff, stash::Stash};
use	crate::silo::uint::{ U16, U32};
use	crate::stalks::work::{ IWorker, WorkFn };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maestro< 'a>
{
    _Atelier: &'a Atelier< 'a>,
    _MavenIndex: U32,
    _Stash: Stash< U16 >,
}
//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Maestro< 'a>
{

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( atelier: &'a Atelier< 'a>, mavenIdx: U32) -> Self
    {
        Self {
            _Atelier: atelier,
            _MavenIndex: mavenIdx,
            _Stash: Stash::New( U32( 64)),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Atelier( &self) -> &'a Atelier< 'a>
    {
        self._Atelier
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MavenIndex( &self) -> U32
    {
        self._MavenIndex
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	CurSuccId( &self) -> U16
    {
        let     maven = self._Atelier.Mavens().At( self._MavenIndex);
        maven.CurSuccId()
    }

    pub fn	ConstructJob( &self, succId: U16, jobBox: Box< WorkFn< 'a>>) -> U16
    {
        self._Atelier.ConstructJob( self._MavenIndex, succId, jobBox)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, jobId: &mut U16)
    {
        assert!( self._Stash.Stk().Push( jobId));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructEnqueueBulk( &self, succId: U16, buff : Buff< U16>) ->  U16
    {
         return self.ConstructJob( succId, Box::new( move | worker: &dyn IWorker| {
            let  	maestro = worker.AsMaestro().unwrap();
            let  	arr = buff.Arr();
            arr.USeg().Traverse( | i| {
                maestro._Atelier.EnqueueJob( self._MavenIndex, arr.MutAt( i));
            });
        }));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}


//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IWorker for Maestro< 'a>
{
    fn	PostJob( &self, job: Box< WorkFn< '_>>)
    {
        let  	mut jobId = self.CurSuccId();
        jobId =  self.ConstructJob(  jobId, job);
        self.EnqueueJob( &mut jobId);
    }

    fn	PostJobs( &self, jobs: Arr< '_, Box< WorkFn< '_>>>)
    {
        if jobs.IsEmpty() {
            return;
        }
        if jobs.len() == 1 {
            let  	mut job = Box::new( |_w: &dyn IWorker| {}) as Box< WorkFn< '_>>;
            jobs.MoveAt( 0, &mut job);
            self.PostJob( job);
            return;
        }
        let  	buff = Buff::Create( jobs.Size(), | i| {
            let  	mut job = Box::new( |_w: &dyn IWorker| {}) as Box< WorkFn< '_>>;
            jobs.MoveAt( i, &mut job);
            job
        });
        let  	branchJob = Box::new( move | worker: &dyn IWorker| {
            let  	maestro = worker.AsMaestro().unwrap();
            let  	arr = buff.Arr();
            let  	succId = maestro.CurSuccId();
            arr.USeg().Span( | i| {
                let  	mut job = Box::new( |_w: &dyn IWorker| {}) as Box< WorkFn< '_>>;
                arr.MoveAt( i, &mut job);
                let  	mut jobId = maestro.ConstructJob( succId, job);
                maestro.EnqueueJob( &mut jobId);
                true
            });
        });
        self.PostJob( branchJob);
    }

    fn	AsMaestro( &self) -> Option< &Maestro< 'a>>
    {
        Some( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Drop for Maestro< 'a>
{
    fn	drop( &mut self)
    {
        let arr = self._Stash.Stk().Arr();
        arr.USeg().Traverse( |i| {
            let mut jobId = *arr.At( i);
            if jobId != 0 {
                self._Atelier.EnqueueJob( self._MavenIndex, &mut jobId);
            }
        });
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

