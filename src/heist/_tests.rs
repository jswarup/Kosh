//- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::
    { atelier::Atelier, maestro::Maestro },
    silo::{
        buff::Buff,
        arr::Arr,
        uint::
        { U16, U32 },
    },
    stalks::work::{ IWorker, Worker},
};

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffBasicAtelierTest()
{
    fn	trialJob( worker: &dyn IWorker)
    {
        let  	maestro = Maestro::FromWorker( worker);
        let  	mut jobId = maestro.CurSuccId();
        jobId = maestro.ConstructJob(
            jobId,
            |w1: &dyn IWorker| {
                println!( "Trial1 {}", Maestro::FromWorker( w1).MavenIndex());
            },
        );
        jobId = maestro.ConstructJob(
            jobId,
            |w2: &dyn IWorker| {
                println!( "Trial2 {}", Maestro::FromWorker( w2).MavenIndex());
            },
        );
        maestro.EnqueueJob( &mut jobId);
        println!( "Trial {}", maestro.MavenIndex());
    }
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    let  	mut jobId = mainMaestro.ConstructJob( U16( 0), trialJob);
    mainMaestro.EnqueueJob( &mut jobId);
    drop( mainMaestro);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestThreadSharedInteger()
{
    use	std::sync::{ Arc, Mutex };
    use	std::thread;
    let  	shared = Arc::new( Mutex::new( 0));
    let  	mut handles = Buff::NewEmpty();
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
    atelier.Mavens().At( 2).SetCurSuccId( U16( 42));
    let  	maestro = Maestro::New( &atelier, U32( 2));
    assert_eq!( maestro.MavenIndex(), U32( 2));
    assert_eq!( maestro.CurSuccId(), U16( 42));
}
//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestChoreBuds()
{
    use	crate::heist::chore::Chore;
    let  	aChore = Chore::New( |_m| {
        print!( "{} ", 10);
    });
    let  	bChore = Chore::New( |_m| {
        print!( "{} ", 20);
    });
    let  	cChore = Chore::New( |_m| {
        print!( "{} ", 40);
    });
    let  	budTree = crate::ChoreTree!(
        ( cChore
            < ( bChore
                | aChore
                | ( |_m| {
                    print!( "{} ", 50);
                })))
    );
    budTree.Print();

    let     worker = Worker::New();

    budTree.Post( &worker);

    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    budTree.Post( &mainMaestro);
    //let  	mut jobId = mainMaestro.ConstructJob( U16( 0), Box::new( chore));
    //mainMaestro.EnqueueJob( &mut jobId);
    drop( mainMaestro);
    atelier.DoLaunch();
}


//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestDoQSort()
{
    let printArr = | arr: Arr< '_, U32>| {
        arr.USeg().Traverse( |i| { print!( "{} ", arr.At( i)); }); println!();
    };

    let mut     buff0 = Buff::Create( U32( 100), |_| U32( rand::random::< u32>() % 128));
    let mut     buff1 = buff0.clone();
    let sortWorkStealing = |buff: &mut Buff< U32>| {

        let     quickSorter = buff.Arr().QuickSorter( |a, b| { a > b});
        let  	atelier = Atelier::New( U32( 4));
        {
            let  	mainMaestro = atelier.MainMaestro();
            let  	mut jobId = mainMaestro.ConstructJob( atelier.Terminal(),  quickSorter);
            mainMaestro.EnqueueJob( &mut jobId);
        }
        atelier.DoLaunch();
    };
    sortWorkStealing(  &mut buff0);
    assert!( buff0.Arr().SortSanity(|a, b| { a > b} ));
    printArr( buff0.Arr());

    let sortSequential = |buff: &mut Buff< U32>| {
        print!( "Sequential[ ");
        let     quickSorter = buff.Arr().QuickSorter( |a, b| { a > b});
        let     worker = Worker::New();
        quickSorter( &worker);
        println!( "]")
    };
    sortSequential( &mut buff1);
    assert!( buff1.Arr().SortSanity(|a, b| { a > b} ));
    printArr( buff1.Arr());
    return;
}

//---------------------------------------------------------------------------------------------------------------------------------
