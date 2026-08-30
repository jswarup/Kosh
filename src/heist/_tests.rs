//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	std::sync::atomic::Ordering;
use	std::sync::{ Arc, Mutex };
use	std::thread;
use	crate::{
    Chore,
    ChoreTree,
    CpuChore,
    GpuAutoChore,
    heist::{ Atelier, IAtelier, IChore, ChoreTarget, IChoreNode, IMaestro, Maestro },
    silo::{ Buff, IAccess, IArr, Stash, U16, U32 },
    stalks::{ Atm, DynIWorker, IntoWorkPtr, IWorker, Worker },
    swarm::SwarmEngine,
};

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffBasicAtelierTest()
{
    fn	trialJob( worker: &DynIWorker< '_>)
    {
        let  	maestro = Maestro::FromWorker( worker);
        let  	mut jobId = maestro.CurSuccId();
        jobId = maestro.ConstructJob( jobId, |w1: &DynIWorker< '_>| {
            println!( "Trial1 {}", Maestro::FromWorker( w1).MaestroIndex());
        }, "TestJob1");
        jobId = maestro.ConstructJob( jobId, |w2: &DynIWorker< '_>| {
            println!( "Trial2 {}", Maestro::FromWorker( w2).MaestroIndex());
        }, "TestJob2");
        maestro.EnqueueJob( jobId);
        println!( "Trial {}", maestro.MaestroIndex());
    }
    let  	atelier = Atelier::New( 4);
    let  	mainMaestro = atelier.MainMaestro();
    let  	jobId = mainMaestro.ConstructJob( 0, trialJob, "TrialJob");
    mainMaestro.EnqueueJob( jobId);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestThreadSharedInteger()
{
    let  	shared = Arc::new( Mutex::new( 0));
    let  	mut handles = Stash::New();
    for i in 0..4 {
        let  	sharedClone = shared.clone();
        let  	handle = thread::spawn( move || {
            let  	mut val = sharedClone.lock().unwrap();
            *val += 1;
            println!( "Thread {} incremented shared integer to: {}", i, *val);
        });
        handles.Push( handle);
    }
    while let Some( handle) = handles.Pop() {
        handle.join().unwrap();
    }
    assert_eq!( *shared.lock().unwrap(), 4);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestMaestroBasicOps()
{
    let  	atelier = Atelier::New( U32( 4));
    {
        let  	maestro = atelier.Maestros().MutAt( 2);
        maestro.SetAtelier( &atelier);
        maestro.SetCurSuccId( U16( 42));
    }
    let  	maestro = atelier.Maestros().At( 2);
    assert_eq!( maestro.MaestroIndex(), U32( 2));
    assert_eq!( maestro.CurSuccId(), U16( 42));
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	TestChoreHelper() -> impl IChoreNode
{
    let  	aChore = Chore!( "10S", |_m| {
        print!( "{} ", 10);
    });
    let  	bChore = Chore!( "20S", |_m| {
        print!( "{} ", 20);
    });
    let  	cChore = Chore!( "40S", |_m| {
        print!( "{} ", 40);
    });
    ChoreTree!( ( cChore
            < ( bChore
                | aChore
                | |_m| {
                    print!( "{} ", 50);
                }))
    )
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestChoreBuds()
{
    let  	choreTree  = TestChoreHelper();
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    mainMaestro.PostChoreTree( &choreTree);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestChoreTree()
{
    let  	a  = Chore!( "A", |_m| {
        print!( "{} ", "A");
    });
    let  	b  = Chore!( "B", |_m| {
        print!( "{} ", "B");
    });
    let  	c = Chore!( "C", |_m| {
        print!( "{} ", "C");
    });
    let  	d = Chore!( "D", |_m| {
        print!( "{} ", "D");
    });

    let  	e = Chore!( "E", |_m| {
        print!( "{} ", "E");
    });

    let  	f = Chore!( "F", |_m| {
        print!( "{} ", "F");
    });

    let  	g = Chore!( "G", |_m| {
        print!( "{} ", "G");
    });

    let  	h = Chore!( "H", |_m| {
        print!( "{} ", "H");
    });

    let  	i = Chore!( "i", |_m| {
        print!( "{} ", "I");
    });

    let  	j = Chore!( "J", |_m| {
        print!( "{} ", "J");
    });

    let  	choreTree = ChoreTree!( ((( ( a < b ) | ( c <  d)) < e) | ( ( f | g) < h)  | i) < j);

    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();

    mainMaestro.PostChoreTree( &choreTree);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestDoQSortWorkStealing()
{
    let  	buff = Buff::Create( U32( 100), |_| U32( rand::random::<u32>() % 128));
    let  	arr = buff.Arr();
    let  	quickSorter = arr.QuickSorter( |a, b| a > b);
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    mainMaestro.PostJob( quickSorter.IntoWorkPtr());
    atelier.DoLaunch();
    assert!( arr.SortSanity( |a, b| { a > b }));
    println!( "{} ", arr);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestDoQSortSequential()
{
    let  	buff = Buff::Create( U32( 100), |_| U32( rand::random::<u32>() % 128));
    let  	quickSorter = buff.Arr().QuickSorter( |a, b| a > b);
    let  	worker = Worker::New();
    worker.PostJob( quickSorter.IntoWorkPtr());
    assert!( buff.Arr().SortSanity( |a, b| { a > b }));
    println!( "{} ", buff.Arr());
}

//---------------------------------------------------------------------------------------------------------------------------------

static CPU_COUNT: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestHeistSwarmCpuChore()
{
    CPU_COUNT.Store( U32::_0, Ordering::Release);

    fn	stepA( _w: &DynIWorker< '_>)
    {
        CPU_COUNT.FetchAdd( U32( 10), Ordering::AcqRel);
    }

    fn	stepB( _w: &DynIWorker< '_>)
    {
        CPU_COUNT.FetchAdd( U32( 20), Ordering::AcqRel);
    }

    let  	choreA = CpuChore!( "CpuStepA", stepA);
    let  	choreB = CpuChore!( "CpuStepB", stepB);

    assert_eq!( choreA.Target(), ChoreTarget::Cpu);
    assert_eq!( choreB.Target(), ChoreTarget::Cpu);

    let  	choreTree = ChoreTree!( choreA < choreB);
    let  	atelier = Atelier::New( 2);
    let  	mainMaestro = atelier.MainMaestro();
    mainMaestro.PostChoreTree( &choreTree);
    atelier.DoLaunch();

    assert_eq!( CPU_COUNT.Load( Ordering::Acquire), 30);
    println!( "TestHeistSwarmCpuChore: Sequential CPU chores executed via Heist ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

static GPU_RESULT_SUM: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestHeistSwarmGpuChore()
{
    GPU_RESULT_SUM.Store( U32::_0, Ordering::Release);

    fn	gpuWork( w: &DynIWorker< '_>)
    {
        let  	maestro = Maestro::FromWorker( w);
        if let Some( swarm) = maestro.Swarm() {
            let  	input = Buff![1.0f32, 2.0, 3.0, 4.0, 5.0];
            let  	res = swarm.RunDouble( &input).unwrap();
            let  	sum: u32 = res.iter().map( |x| *x as u32).sum();
            GPU_RESULT_SUM.Store( U32( sum), Ordering::Release);
        }
    }

    let  	engine = SwarmEngine::Auto();
    let  	mut atelier = Atelier::New( 2);
    atelier.SetSwarm( engine);

    let  	gpuChore = GpuAutoChore!( "GpuDouble", gpuWork);
    assert_eq!( gpuChore.Target(), ChoreTarget::GpuAuto);

    let  	mainMaestro = atelier.MainMaestro();
    mainMaestro.PostChoreTree( &gpuChore);
    atelier.DoLaunch();

    assert_eq!( GPU_RESULT_SUM.Load( Ordering::Acquire), 30);
    println!( "TestHeistSwarmGpuChore: GPU chore executed via Heist Atelier ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

static STAGE1_A: Atm< U32> = U32::_0.IntoAtm();
static STAGE1_B: Atm< U32> = U32::_0.IntoAtm();
static STAGE2_SUM: Atm< U32> = U32::_0.IntoAtm();
static STAGE3_FINAL: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestHeistSwarmHeterogeneousPipeline()
{
    STAGE1_A.Store( U32::_0, Ordering::Release);
    STAGE1_B.Store( U32::_0, Ordering::Release);
    STAGE2_SUM.Store( U32::_0, Ordering::Release);
    STAGE3_FINAL.Store( U32::_0, Ordering::Release);

    fn	produceA( _w: &DynIWorker< '_>)
    {
        STAGE1_A.Store( U32( 10), Ordering::Release);
    }

    fn	produceB( _w: &DynIWorker< '_>)
    {
        STAGE1_B.Store( U32( 20), Ordering::Release);
    }

    fn	mergeChunks( _w: &DynIWorker< '_>)
    {
        let  	valA = STAGE1_A.Load( Ordering::Acquire);
        let  	valB = STAGE1_B.Load( Ordering::Acquire);
        STAGE2_SUM.Store( valA + valB, Ordering::Release);
    }

    fn	gpuCompute( w: &DynIWorker< '_>)
    {
        let  	maestro = Maestro::FromWorker( w);
        if let Some( swarm) = maestro.Swarm() {
            let  	sumVal = STAGE2_SUM.Load( Ordering::Acquire).AsU32() as f32;
            let  	input = Buff![sumVal];
            let  	res = swarm.RunDouble( &input).unwrap();
            STAGE3_FINAL.Store( U32( res[0] as u32), Ordering::Release);
        }
    }

    let  	engine = SwarmEngine::Auto();
    let  	mut atelier = Atelier::New( 4);
    atelier.SetSwarm( engine);

    let  	choreA = CpuChore!( "ProduceA", produceA);
    let  	choreB = CpuChore!( "ProduceB", produceB);
    let  	mergeChore = CpuChore!( "Merge", mergeChunks);
    let  	computeChore = GpuAutoChore!( "GpuCompute", gpuCompute);

    let  	pipeline = ChoreTree!(
        ( choreA | choreB ) < mergeChore < computeChore
    );

    let  	mainMaestro = atelier.MainMaestro();
    mainMaestro.PostChoreTree( &pipeline);
    atelier.DoLaunch();

    assert_eq!( STAGE3_FINAL.Load( Ordering::Acquire), 60);
    println!( "TestHeistSwarmHeterogeneousPipeline: Parallel CPU -> Merge -> GPU pipeline completed ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------
