//-- maestro.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::atelier::Atelier;
use	crate::silo::uint::{ U16, U32 };
use	crate::silo::{ buff::Buff, stash::Stash };
use	crate::stalks::work::{ IWorker, WorkPtr, IntoWorkPtr };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Maestro< 'a> 
{
    _Atelier: &'a Atelier< 'a>,
    _MavenIndex: U32,
    _Stash: Stash< U16>,
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

    pub fn	MavenIndex( &self) -> U32 
    {
        self._MavenIndex
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FromWorker( worker: &dyn IWorker) -> &Self 
    {
        let  	ptr = worker.AsRaw();
        assert!( !ptr.is_null());
        unsafe { &*( ptr as *const Self) }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	CurSuccId( &self) -> U16 
    {
        let  	maven = self._Atelier.Mavens().At( self._MavenIndex);
        maven.CurSuccId()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob( &self, succId: U16, job: impl IntoWorkPtr< 'a>) -> U16 
    {
        self._Atelier.ConstructJob( self._MavenIndex, succId, job.IntoWorkPtr())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EnqueueJob( &self, jobId: &mut U16) 
    {
        assert!( self._Stash.Stk().Push( jobId));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructEnqueueBulk( &self, succId: U16, buff: Buff< U16>) -> U16 
    {
        self.ConstructJob(
            succId,
            move |worker: &dyn IWorker| {
                let  	maestro = Maestro::FromWorker( worker);
                let  	arr = buff.Arr();
                arr.USeg().Traverse( |i| {
                    maestro._Atelier.EnqueueJob( self._MavenIndex, arr.MutAt( i));
                });
            },
        )
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IWorker for Maestro< 'a> 
{
    fn	PostJob( &self, job: WorkPtr< '_>) 
    {
        let  	mut jobId = self.CurSuccId();
        jobId = self.ConstructJob( jobId, job);
        self.EnqueueJob( &mut jobId);
    }
    fn	AsRaw( &self) -> *const () 
    {
        self as *const Self as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Drop for Maestro< 'a> 
{
    fn	drop( &mut self) 
    {
        let  	arr = self._Stash.Stk().Arr();
        arr.USeg().Traverse( |i| {
            let  	mut jobId = *arr.At( i);
            if jobId != 0 {
                self._Atelier.EnqueueJob( self._MavenIndex, &mut jobId);
            }
        });
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
