//-- atelierinfo.rs ------------------------------------------------------------------------------------------------------------------
use	std::collections::HashSet;
use	std::fmt;
use	std::sync::atomic::Ordering;
use	crate::heist::{ Atelier, IAtelier };
use	crate::silo::{ Arr, Buff, IAccess, IArr, Stash, U16, U32 };
use	crate::silo::uint::Xplod;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Default)]
pub struct JobInfo
{
    pub _JobId:  U16,
    pub _SuccId: U16,
    pub _SzPred: U16,
    pub _DocStr: &'static str,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl JobInfo
{
    pub fn	New< 'a>( atelier: *const Atelier< 'a>, jobId: U16) -> Self
    {
        unsafe {
            let  	succId = *( *atelier).SuccIds().At( jobId);
            let  	szPred = ( *atelier).SzPred( jobId).Load( Ordering::SeqCst);
            let  	docStr = *( *atelier).JobDocBuff().Arr().At( jobId);
            Self {
                _JobId:  jobId,
                _SuccId: succId,
                _SzPred: szPred,
                _DocStr: docStr,
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct AtelierInfo
{
    pub _HookedStash: Stash< JobInfo>,
    pub _JobRefBuff:  Buff< U16>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AtelierInfo
{
    pub fn	FetchConnectedJobs( atelier: &Atelier< '_>, jobIds: Arr< U16>, jobStash: &mut Stash< JobInfo>)
    {
        let  	mut jobSet = HashSet::< U16>::new();
        let  	mut processStash = Stash::< U16>::Create( U32( 1024), 0, |_| U16( 0));
        jobIds.Traverse( |jobId| {
            processStash.Push( *jobId);
        });

        processStash.Stk().Arr().Traverse( |jobId| {
            if jobSet.insert( *jobId) {
                let  	succId = *atelier.SuccIds().At( *jobId);
                if succId != U16( 0) {
                    processStash.Stk().Push( succId);
                }
                jobStash.Push( JobInfo::New( atelier as *const _, *jobId));
            }
        });
    }

    pub fn	TraceJobs( atelier: &Atelier< '_>) -> AtelierInfo
    {
        let  	docArr = atelier.JobDocBuff().Arr();
        let  	freeDoc = atelier.FreeDocStr();
        let  	mut info = AtelierInfo {
            _HookedStash: Stash::Create( U32( 1024), U32::_0, |_| JobInfo::default()),
            _JobRefBuff:  Buff::Create( U32::_16Sz, |i| if ( *docArr.At( i)).as_ptr() == ( *freeDoc).as_ptr() { i.Xplod()[ 0] } else { U16::_X }),
        };

        let  	maestros = atelier.Maestros();
        maestros.Traverse( |maestro| {
            let  	runQueue = maestro.RunQueueArr();
            Self::FetchConnectedJobs( atelier, runQueue, &mut info._HookedStash);
        });

        let  	jobRefs = info._JobRefBuff.Arr();
        let  	jSeg = jobRefs.USeg();
        jSeg.QSort( |i, j| *jobRefs.At( i) < *jobRefs.At( j), |i, j| jobRefs.Swap( i, j));
        let  	hookedArr = info._HookedStash.Stk().Arr();
        let  	mut tempStash = Stash::Create( U32( 1024), U32::_0, |_| U16::_X);

        hookedArr.Traverse( |job| {
            let  	lInd = jSeg.LowerBound( |i| *jobRefs.At( i) < job._JobId);
            if jSeg.IsWithin( lInd) && ( *jobRefs.At( lInd) == job._JobId) {
                tempStash.Push( lInd.Xplod()[ 0]);
            }
        });
        tempStash.Stk().Arr().Traverse( |jobId| {
            jobRefs.SetAt( *jobId, &U16::_X);
        });

        jSeg.QSort( |i, j| *jobRefs.At( i) < *jobRefs.At( j), |i, j| jobRefs.Swap( i, j));

        let  	lInd = jSeg.LowerBound( |i| *jobRefs.At( i) < U16::_X);
        jobRefs.RSnip( jobRefs.Size() - lInd);
        info
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for JobInfo
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "{{ JobId: {},  {}, {}, {}}} ", self._JobId, self._SuccId, self._SzPred, self._DocStr)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for AtelierInfo
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "Atel[ Hooked:")?;
        self._HookedStash.Stk().Arr().Traverse( |job| {
            let  	_ = write!( f, " {}", *job);
        });
        write!( f, " FreeJobs:")?;
        self._JobRefBuff.Arr().Traverse( |jobId| {
            let  	_ = write!( f, " {}", *jobId);
        });
        write!( f, "] ")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
