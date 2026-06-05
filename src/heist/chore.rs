
//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::uint::{ U16, U32 };
use	crate::silo::{ stash::Stash};
use	crate::heist::maestro::Maestro;
use	crate::stalks::work::{ IWorker, IWork };
use	crate::stalks::bud::Bud;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Chore;

//---------------------------------------------------------------------------------------------------------------------------------

impl Chore
{
    pub fn	New() -> Self
    {
        Self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Chore
{
    fn	DoWork( &mut self, _worker: &dyn IWorker)
    {
        println!( "Chore");
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

impl Bud< Chore> for Chore
{
    fn	Val( &self) -> Chore
    {
        *self
    }

    fn	Left( &self) -> Option< &dyn Bud< Chore>>
    {
        None
    }

    fn	Right( &self) -> Option< &dyn Bud< Chore>>
    {
        None
    }

    fn	Op( &self) -> &str
    {
        ""
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Chore
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        write!( f, "Chore")
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

impl< T> dyn Bud< T> + '_
where
    T: IWork + Clone + Default + 'static,
{
    pub fn	Post( &self, worker:  &dyn IWorker )
    {
        let  	maestro: &Maestro<'_> = worker.AsMaestro().unwrap();

        struct JobStash
        {
            _JobStash: Stash< U16>,
        }

        impl  JobStash
        {
            fn	Process< T>( &mut self, node: &dyn Bud< T>, maestro: &Maestro< '_>, succId: U16) -> U16
            where
                T: IWork + Clone + Default + 'static,
            {
                if node.Left().is_none() && node.Right().is_none() {
                    let  	choreJob: Box< dyn IWork> = Box::new( node.Val());
                    let  	mut jobId = maestro.ConstructJob( succId, choreJob);
                    self._JobStash.Pushback( &mut jobId);
                    return jobId;
                }
                if node.Op() == "|" {
                    let     _succR = self.Process( node.Right().unwrap(), maestro, succId);
                    let  	_succL = self.Process( node.Left().unwrap(), maestro, succId);
                    return succId;
                }
                if node.Op() == "<" {
                    let     mark = self._JobStash.Size();
                    let  	rJobId = self.Process( node.Right().unwrap(), maestro, succId);
                    let     jStk = self._JobStash.Stk();
                    let     rSz =  jStk.Size() -mark;
                    if rSz == 1 {
                        return self.Process( node.Left().unwrap(), maestro, rJobId);
                    }

                    let mut rXStash : Stash< U16> = Stash::New( U32(0));
                    self._JobStash.Stk().Export( &rXStash.Stk(), rSz);
                    let     rBuff = rXStash.BuffOut();
                    let  	branchJob  = Box::new( move | worker: &dyn IWorker| {
                        let  	maestro = worker.AsMaestro().unwrap();
                        let  	arr = rBuff.Arr();
                        arr.USeg().Span( | i| {
                            maestro.EnqueueJob( arr.MutAt( i));
                            true
                        });
                    });
                    let  	branchId = maestro.ConstructJob( succId, branchJob);
                    let  	succL = self.Process( node.Left().unwrap(), maestro, branchId);
                    return succL;
                } else {
                    assert!( false);
                    return U16( 0);
                }
            }

        }

        let  	succId = maestro.CurSuccId();
        let  	mut jobStash = JobStash { _JobStash: Stash::New( 0) };

        jobStash.Process( self, maestro, succId);
        let     jobArr = jobStash._JobStash.Stk().Arr();
        jobArr.USeg().Span( |i| {
            maestro.EnqueueJob( jobArr.MutAt( i));
            true
        });
        return;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


