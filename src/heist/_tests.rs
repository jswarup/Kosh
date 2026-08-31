//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	std::sync::atomic::Ordering;
use	std::sync::{ Arc, Mutex };
use	std::thread;
use	crate::{
    Chore,
    ChoreTree,
    CoroChore,
    CpuChore,
    CpuMapCollect,
    GpuAutoChore,
    WeightedChore,
    heist::{ Atelier, IAtelier, IChore, ChoreTarget, IChoreNode, IMaestro, Maestro, Chore },
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

#[test]
fn	TestChoreWeightCalculation()
{
    let  	choreA = Chore::NewDoc( "A", |_| {}).WithWeight( U32( 10));
    let  	choreB = WeightedChore!( U32( 20), "B", |_| {});
    let  	choreC = CpuChore!( "C", |_| {}).WithWeight( U32( 30));

    let  	tree = ChoreTree!( choreA | ( choreB < choreC));
    assert_eq!( tree.Weight(), U32( 60));
    println!( "TestChoreWeightCalculation: Tree weight calculated correctly ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

static MAP_COUNT_BASIC: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestMapCollectCpuBasic()
{
    MAP_COUNT_BASIC.Store( U32( 0), Ordering::Release);
    let  	buff = Buff::Create( U32( 10000), |_| 1u32);

    let  	mapCollect = CpuMapCollect!(
        buff.Arr(),
        |seg, _w| {
            let  	mut sum = 0;
            seg.Traverse( |_| sum += 1);
            MAP_COUNT_BASIC.FetchAdd( U32( sum), Ordering::Relaxed);
        },
        |_w| {
            println!( "Collect phase!");
        }
    );

    let  	atelier = Atelier::New( 4);
    atelier.MainMaestro().PostChoreTree( &mapCollect);
    atelier.DoLaunch();

    assert_eq!( MAP_COUNT_BASIC.Load( Ordering::Acquire), U32( 10000));
    println!( "TestMapCollectCpuBasic: Distributed map and collect completed ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

static MAP_COUNT_ADAPTIVE: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestMapCollectAdaptiveChunking()
{
    MAP_COUNT_ADAPTIVE.Store( U32( 0), Ordering::Release);
    let  	buff = Buff::Create( U32( 100), |_| 1u32); // Small enough to be lumped

    let  	mapCollect = CpuMapCollect!(
        buff.Arr(),
        |seg, _w| {
            let  	mut sum = 0;
            seg.Traverse( |_| sum += 1);
            MAP_COUNT_ADAPTIVE.FetchAdd( U32( sum), Ordering::Relaxed);
        },
        |_w| {}
    );

    let  	atelier = Atelier::New( 4).WithFusionThres( 1000);
    atelier.MainMaestro().PostChoreTree( &mapCollect);
    atelier.DoLaunch();

    assert_eq!( MAP_COUNT_ADAPTIVE.Load( Ordering::Acquire), U32( 100));
    println!( "TestMapCollectAdaptiveChunking: Coalesced small workload correctly ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

static FUSED_SEQ_VALUE: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestAutomaticSequentialChoreFusion()
{
    FUSED_SEQ_VALUE.Store( U32( 0), Ordering::Release);

    let  	choreA = CpuChore!( "StepA", |_w| {
        FUSED_SEQ_VALUE.FetchAdd( U32( 5), Ordering::Relaxed);
    }).WithWeight( U32( 10));

    let  	choreB = CpuChore!( "StepB", |_w| {
        let  	cur = FUSED_SEQ_VALUE.Load( Ordering::Acquire);
        FUSED_SEQ_VALUE.Store( cur * U32( 2), Ordering::Release);
    }).WithWeight( U32( 20));

    let  	choreC = CpuChore!( "StepC", |_w| {
        FUSED_SEQ_VALUE.FetchAdd( U32( 1), Ordering::Relaxed);
    }).WithWeight( U32( 30));

    let  	pipeline = ChoreTree!(
        choreA < choreB < choreC
    );

    assert_eq!( pipeline.Weight(), U32( 60));

    let  	atelier = Atelier::New( 2).WithFusionThres( 100);
    atelier.MainMaestro().PostChoreTree( &pipeline);
    atelier.DoLaunch();

    // StepA (+5) -> StepB (*2 => 10) -> StepC (+1 => 11)
    assert_eq!( FUSED_SEQ_VALUE.Load( Ordering::Acquire), U32( 11));
    println!( "TestAutomaticSequentialChoreFusion: Automatic fused DAG completed ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

static DEFAULT_FUSION_VAL: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestDefaultFusionThreshold()
{
    DEFAULT_FUSION_VAL.Store( U32( 0), Ordering::Release);

    let  	atelier = Atelier::New( 2);
    assert_eq!( atelier.FusionThres(), U32( 2));

    // Two default-weight chores (1 + 1 = 2 <= 2) will fuse by default
    let  	chore1 = CpuChore!( "Chore1", |_w| {
        DEFAULT_FUSION_VAL.FetchAdd( U32( 10), Ordering::Relaxed);
    });
    let  	chore2 = CpuChore!( "Chore2", |_w| {
        DEFAULT_FUSION_VAL.FetchAdd( U32( 20), Ordering::Relaxed);
    });

    let  	tree = ChoreTree!( chore1 < chore2);
    assert_eq!( tree.Weight(), U32( 2));

    atelier.MainMaestro().PostChoreTree( &tree);
    atelier.DoLaunch();

    assert_eq!( DEFAULT_FUSION_VAL.Load( Ordering::Acquire), U32( 30));
    println!( "TestDefaultFusionThreshold: Default _FusionThres = 2 verified ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

static CORO_STAGE: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestCoroGeneratorBasic()
{
    use crate::stalks::{ Coro, CoroRes, ICoro };
    let  	mut coro = Coro::New( |yielder, _input: ()| {
        yielder.Suspend( 10);
        yielder.Suspend( 20);
        30
    });

    assert_eq!( coro.IsDone(), false);

    if let CoroRes::Yield( y) = coro.Resume( ()) {
        assert_eq!( y, 10);
    } else { panic!(); }

    if let CoroRes::Yield( y) = coro.Resume( ()) {
        assert_eq!( y, 20);
    } else { panic!(); }

    if let CoroRes::Done( r) = coro.Resume( ()) {
        assert_eq!( r, 30);
    } else { panic!(); }

    assert_eq!( coro.IsDone(), true);
    println!( "TestCoroGeneratorBasic: Yield/Resume cycle verified ?");
}

#[test]
fn	TestHeistCoroChoreYieldAndResume()
{
    CORO_STAGE.Store( U32( 0), Ordering::Release);

    let  	coroChore = CoroChore!( "CoroTask", |yielder, _wPtr| {
        CORO_STAGE.Store( U32( 1), Ordering::Release);
        yielder.Suspend( ());
        CORO_STAGE.Store( U32( 2), Ordering::Release);
        yielder.Suspend( ());
        CORO_STAGE.Store( U32( 3), Ordering::Release);
    });

    let  	atelier = Atelier::New( 2);
    atelier.MainMaestro().PostChoreTree( &coroChore);
    atelier.DoLaunch();

    assert_eq!( CORO_STAGE.Load( Ordering::Acquire), U32( 3));
    println!( "TestHeistCoroChoreYieldAndResume: Coroutine suspended and resumed on worker ?");
}

static DAG_SEQ_VAL: Atm< U32> = U32::_0.IntoAtm();

#[test]
fn	TestHeistCoroChoreDAG()
{
    DAG_SEQ_VAL.Store( U32( 1), Ordering::Release);

    let  	c1 = CpuChore!( "Step1", |_| {
        DAG_SEQ_VAL.FetchAdd( U32( 1), Ordering::Relaxed);
    });

    let  	c2 = CoroChore!( "CoroStep2", |yielder, _wPtr| {
        let  	v = DAG_SEQ_VAL.Load( Ordering::Acquire);
        DAG_SEQ_VAL.Store( v * U32( 2), Ordering::Release);
        yielder.Suspend( ());
        let  	v2 = DAG_SEQ_VAL.Load( Ordering::Acquire);
        DAG_SEQ_VAL.Store( v2 * U32( 2), Ordering::Release);
    });

    let  	c3 = CpuChore!( "Step3", |_| {
        DAG_SEQ_VAL.FetchAdd( U32( 5), Ordering::Relaxed);
    });

    let  	tree = ChoreTree!( c1 < c2 < c3 );

    let  	atelier = Atelier::New( 2);
    atelier.MainMaestro().PostChoreTree( &tree);
    atelier.DoLaunch();

    // (1 + 1) * 2 * 2 + 5 = 13
    assert_eq!( DAG_SEQ_VAL.Load( Ordering::Acquire), U32( 13));
    println!( "TestHeistCoroChoreDAG: DAG execution with CoroChore intermediate step completed ?");
}

