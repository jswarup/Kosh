//-- atelier.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::heist::Maestro;
use	crate::silo::{ Arr, Buff, IAccess, IArr, Stash, U16, U32 };
use	crate::stalks::{ Atm, Spinlock, WorkPtr };
use	std::sync::atomic::Ordering;
use std::collections::HashSet;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Atelier< 'a>
{
    pub( crate) _SzSchedJob: Atm< U32>,                                 // Count of cumulative jobs in flight
    _Maestros: Buff< Maestro< 'a>>,
    _SzPreds: Buff< Atm< U16>>,                                        // Count of predessors for job at the jobId
    _SuccIds: Buff< U16>,
    _FreeJobLock: Spinlock,
    _FreeJobStash: Stash< U16>,                                        // A Stack of free jobIds
    _JobBuff: Buff< WorkPtr< 'a>>,
    _JobDocBuff: Buff< &'static str>,
    _Terminal: U16,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Atelier< 'a>
{

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( szMaestro: U32) -> Atelier< 'a>
    {
        let  	mut atelier = Self {
            _SzSchedJob: Atm::New( U32::_0),
            _Maestros: Buff::Create( szMaestro, Maestro::New),
            _SzPreds: Buff::Create( U32::_16Sz, |_i| Atm::New( U16::_0)),
            _SuccIds: Buff::< U16>::New( U32::_16Sz, U16::_0),
            _FreeJobLock: Spinlock::New(),
            _FreeJobStash: Stash::< U16>::New( U32::_16Sz, 0, U16( 0)),
            _JobBuff: Buff::New( U32::_16Sz, WorkPtr::Null()),
            _JobDocBuff: Buff::New( U32::_16Sz, "Free"),
            _Terminal: U16( 0),
        };
        atelier._FreeJobStash.DoIndexSetup();
        atelier._Terminal = atelier.ConstructJob( U32( 0), U16( 0), WorkPtr::Dummy(), "Terminal");
        atelier._Maestros.Arr().MutAt( 0).SetCurSuccId( atelier._Terminal);
        atelier
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MainMaestro( &self) -> &Maestro< 'a>
    {
        let  	maestro = self._Maestros.Arr().MutAt( U32( 0));
        maestro.SetAtelier( self);
        maestro
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Terminal( &self) -> U16
    {
        self._Terminal
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Maestros( &self) -> Arr< 'a, Maestro< 'a>>
    {
        self._Maestros.Arr()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn  SuccId( &self, jobId: U16) -> U16
    {
        *self._SuccIds.Arr().At( jobId)
    }
 
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SzPred( &self, jobId: U16) -> &Atm< U16>
    {
        self._SzPreds.Arr().At( jobId)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FreeDocStr( &self) -> &'static str
    {
        let     docStr = *self._JobDocBuff.Arr().At( 0);
        assert!( docStr == "Free");
        docStr
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	AllocJob( &self, maestroIdx: U32) -> U16
    {
        let  	maestro = self._Maestros.Arr().At( maestroIdx);
        let  	jobCacheStk = maestro.JobCacheStk();
        loop {
            let  	mut jobId = U16( 0);
            if jobCacheStk.Size() != 0 && jobCacheStk.Pop( &mut jobId) {
                return jobId;
            }
            if self._FreeJobStash.Size() == 0 {
                std::thread::yield_now();
                continue;
            }
            let  	_guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Export( &jobCacheStk, U32::_X);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	FreeJob( &self, maestroIdx: U32, mut jobId: U16) -> bool
    {
        self._JobDocBuff.Arr().SetAt( jobId, &self.FreeDocStr());

        let  	maestro = self._Maestros.Arr().At( maestroIdx);
        
        maestro.FlushTempQueue();
        let  	jobCacheStk = maestro.JobCacheStk();
        loop {
            if jobCacheStk.SzVoid() != 0 && jobCacheStk.PushX( &mut jobId) {
                return true;
            }
            let  	_guard = self._FreeJobLock.Lock();
            self._FreeJobStash.Stk().Import( &jobCacheStk, U32::_X);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SetSucc( &self, jobId: U16, succId: U16)  
    {  
        self._SuccIds.Arr().SetAt( jobId, &succId);
        self.SzPred( succId).Add( 1); 
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConstructJob( &self, maestroIdx: U32, succId: U16, job: WorkPtr< 'a>, docStr: &'static str) -> U16
    {
        let  	jobId = self.AllocJob( maestroIdx);
        if jobId == 0 {
            return jobId;
        }
        self._JobBuff.Arr().SetAt( jobId, &job); 
        self._JobDocBuff.Arr().SetAt( jobId, &docStr);
        if succId != 0 {
             self.SetSucc( jobId, succId);
        }
        jobId
    }


    //-----------------------------------------------------------------------------------------------------------------------------

    fn	GrabJob( &self, idx: U32, stealSeed: &mut u32) -> U16
    {
        let  	maestros = self._Maestros.Arr();
        let  	sz = maestros.len() as u32;
        let     knuthMultHash = 2654435761u32;
        *stealSeed = stealSeed.wrapping_mul( knuthMultHash).wrapping_add( 1u32);
        for mIdx in 0..sz {
            let  	maestroIdx = U32( stealSeed.wrapping_add( mIdx) % sz);
            if maestroIdx == idx {
                continue;
            }
            let  	maestro = maestros.At( maestroIdx);
            let  	jobId = maestro.PopJob();
            if jobId != 0 {
                return jobId;
            }
        }
        U16( 0)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ExecuteLoop( &self, maestroIdx: U32)
    {
        let  	maestro = self._Maestros.Arr().MutAt( maestroIdx);
        maestro.SetAtelier( self );
        maestro.FlushTempQueue();
        let  	mut jobId = U16( 0);
        let  	mut stealSeed = maestroIdx.AsU32();
        while self._SzSchedJob.Load( Ordering::Acquire) != 0 {
            while jobId != 0 {
                maestro.SetCurSuccId( *self._SuccIds.Arr().At( jobId));
                let  	job = *self._JobBuff.Arr().At( jobId);
                assert!( !job.IsNull(), "jobId {} is null!", jobId.AsU16());

                ( job.func)( job.data, maestro);                   // Run job
                self._JobBuff.Arr().SetAt( jobId, &WorkPtr::Null());
                maestro._SzProcessed += 1;

                let  	_res = self.FreeJob( maestroIdx, jobId);
                let  	succId = maestro.CurSuccId();
                if succId != U16( 0) {
                    let  	szPred: U16 = self.SzPred( succId).Add( -U16( 1));
                    if szPred == U16( 1) {
                        jobId = succId;
                        self._SzSchedJob.Add( U32( 1));
                    } else {
                        jobId = U16::_0;
                    }
                } else {
                    jobId = U16::_0;
                }

                self._SzSchedJob.Add( -U32( 1));
            }
            jobId = maestro.PopJob();
            if jobId == 0 {
                jobId = self.GrabJob( maestroIdx, &mut stealSeed);
            }
            if jobId == 0 {
                std::hint::spin_loop();
                std::thread::yield_now();
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DoLaunch( &self)
    {
        let  	maestros = self._Maestros.Arr();
        std::thread::scope( |s| {
            for maestroIdx in 1..maestros.len() {
                s.spawn( move || {
                    self.ExecuteLoop( U32( maestroIdx as u32));
                });
            }
            self.ExecuteLoop( U32( 0));
        });
        println!();
        print!( "Atelier[ ");
        maestros.USeg().Traverse( |maestroIdx| {
            print!( "( Maestro-{}: {})", maestroIdx, maestros.At( maestroIdx)._SzProcessed);
        });
        println!( "]")
    }



}


//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Default)]
pub struct JobInfo
{
    pub _JobId: U16, 
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
            let  	succId = *( *atelier)._SuccIds.Arr().At( jobId);
            let  	szPred = ( *atelier)._SzPreds.Arr().At( jobId).Load( Ordering::SeqCst);
            let  	docStr = *( *atelier)._JobDocBuff.Arr().At( jobId);
            Self { _JobId: jobId, _SuccId: succId, _SzPred: szPred, _DocStr: docStr }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct AtelierInfo
{ 
    
    pub _HookedStash: Stash< JobInfo>,
    pub _OrphanStash: Stash< JobInfo>
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AtelierInfo
{ 

    pub fn	TraceJobs( atelier: &Atelier< '_>, jobIds: Arr< U16>, jobStash: &mut Stash< JobInfo>  )
    {        
        let  	mut jobSet = HashSet::< U16>::new(); 
        let  	mut processStash = Stash::< U16>::New( U32( 1024), 0, U16( 0));
        jobIds.Traverse( |jobId| {
            processStash.Push( *jobId);
        });
          
        for jobId in processStash.Stk().Arr()  {
            if !jobSet.insert( *jobId) {
                continue;
            }
            
            let  	succId = *atelier._SuccIds.Arr().At( *jobId);
            if succId != U16( 0) {
                processStash.Stk().Push( succId);
            }  
            jobStash.Push( JobInfo::New( atelier as *const _, *jobId));
        } 
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for JobInfo
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        write!( f, "{{ JobId: {},  {}, {}, {}}} ", self._JobId, self._SuccId, self._SzPred, self._DocStr)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for AtelierInfo
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        write!( f, "Atel[ Hooked:")?;
        self._HookedStash.Stk().Arr().Traverse( |job| { 
            let  	_ = write!( f, " {}", *job);
        });
        write!( f, " Orphan:")?;
        self._OrphanStash.Stk().Arr().Traverse( |job| { 
            let  	_ = write!( f, " {}", *job);
        });
        write!( f, "] ") 
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
