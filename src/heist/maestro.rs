//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::atelier::Atelier;
use	crate::silo::arr::Arr;
use	crate::silo::uint::{ U16, U32};
use	crate::stalks::work::{ IWorker, WorkFn };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maestro< 'a>
{
    _Atelier: &'a Atelier< 'a>,
    _MavenIndex: U32,
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

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob< F>( &self,  succId: U16, jobFn: F) -> U16
    where
        F: FnMut( &dyn IWorker) + Send + Sync + 'a,
    {
        self._Atelier.ConstructJob( self._MavenIndex, succId, jobFn)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, jobId: &mut U16)
    {
        self._Atelier.EnqueueJob( self._MavenIndex, jobId)
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
        for i in 0..jobs.len() {
			let  	mut job = Box::new( |_w: &dyn IWorker| {}) as Box< WorkFn< '_>>;
            jobs.MoveAt( i, &mut job);
            self.PostJob( job);
        }
    }

    fn	AsMaestro( &self) -> Option< &Maestro< 'a>>
    {
        Some( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

