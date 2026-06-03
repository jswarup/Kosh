
//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::uint::{ U16, U32 };
use	crate::silo::{ stash::Stash};
use	crate::heist::maestro::Maestro;
use	crate::stalks::work::{ IWorker, IWork };
use	crate::stalks::bud::{ Bud, IntoBud };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Chore
{
    pub Ind: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Chore
{
    pub fn	New( ind: U32) -> Self
    {
        Self {
            Ind: ind,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Chore
{
    fn	DoWork( &mut self, _worker: &dyn IWorker)
    {
        println!( "{}", self.Ind);
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

impl IntoBud< Chore> for Chore
{
    fn	IntoBud( self) -> Box< dyn Bud< Chore>>
    {
        Box::new( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Chore
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        write!( f, "{}", self.Ind)
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

impl dyn Bud<Chore>+ '_
{
    pub fn	Post( &self, worker:  &dyn IWorker )
    {
        let  	maestro: &Maestro<'_> = worker.AsMaestro().unwrap();

        struct ForkJob
        {
            _RStarts: Stash< U16>,
        }

        impl IWork for ForkJob
        {
            fn	DoWork( &mut self, worker: &dyn IWorker)
            {
                let  	maestro = worker.AsMaestro().unwrap();
                for rStart in &mut *self._RStarts.Stk().Arr() {
                    maestro.EnqueueJob( rStart);
                }
            }
        }

        fn	Compile( node: &dyn Bud< Chore>, maestro: &Maestro< '_>, succId: U16) -> Stash< U16>
        {
            if node.Left().is_none() && node.Right().is_none() {
                let  	choreJob: Box< dyn IWork> = Box::new( node.Val());
                let  	mut jobId = maestro.ConstructJob( succId, choreJob);
                let  	mut stash = Stash::New( 1);
                stash.Pushback( &mut jobId);
                stash
            } else if node.Op() == "|" {
                let  	mut startsL = Compile( node.Left().unwrap(), maestro, succId);
                let  	startsR = Compile( node.Right().unwrap(), maestro, succId);
                startsL.Append( startsR.Stk().Arr());
                startsL
            } else if node.Op() == "<" {
                let  	rStarts = Compile( node.Right().unwrap(), maestro, succId);
                if rStarts.Size() == 1 {
                    Compile( node.Left().unwrap(), maestro, *rStarts.Stk().Arr().At( 0))
                } else {
                    let  	forkJob = Box::new( ForkJob { _RStarts: rStarts }) as Box< dyn IWork>;
                    let  	forkJobId = maestro.ConstructJob( U16( 0), forkJob);
                    Compile( node.Left().unwrap(), maestro, forkJobId)
                }
            } else {
                Stash::New( 4)
            }
        }

        let  	succId = maestro.CurSuccId();
        let  	starts = Compile( self, maestro, succId);
        for startId in &mut *starts.Stk().Arr() {
            maestro.EnqueueJob( startId);
        }

        #[allow(dead_code)]
        struct JobStash
        {
            _JobStash: Stash< U16>,
        }

        impl IWork for JobStash
        {
            fn	DoWork( &mut self, worker: &dyn IWorker)
            {
                let  	maestro = worker.AsMaestro().unwrap();
                for headJob in &mut *self._JobStash.Stk().Arr() {
                    maestro.EnqueueJob( headJob);
                }
            }
        }
        #[allow(dead_code)]
        impl  JobStash
        {
            fn	Process( &mut self, node: &dyn Bud< Chore>, maestro: &Maestro< '_>, succId: U16) -> U16
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
                    return branchId;
                } else {
                    assert!( false);
                    return U16( 0);
                }
            }

        }
        let  	succId = maestro.CurSuccId();
        let  	starts = Compile( self, maestro, succId);
        for startId in &mut *starts.Stk().Arr() {
            maestro.EnqueueJob( startId);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


