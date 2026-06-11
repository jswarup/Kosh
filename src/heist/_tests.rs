//- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::
    { atelier::Atelier, maestro::Maestro },
    silo::{
        buff::Buff,
        uint::
        { U16, U32 },
    },
    stalks::work::
    { IWorker, Worker },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffBasicAtelierTest()
{
    fn	trialJob( worker: &dyn IWorker)
    {
        let  	maestro = Maestro::FromWorker( worker);
        let  	mut jobId = maestro.CurSuccId();
        jobId = maestro.ConstructJob( jobId, |w1: &dyn IWorker| {
            println!( "Trial1 {}", Maestro::FromWorker( w1).MaestroIndex());
        });
        jobId = maestro.ConstructJob( jobId, |w2: &dyn IWorker| {
            println!( "Trial2 {}", Maestro::FromWorker( w2).MaestroIndex());
        });
        maestro.EnqueueJob( &mut jobId);
        println!( "Trial {}", maestro.MaestroIndex());
    }
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    let  	mut jobId = mainMaestro.ConstructJob( U16( 0), trialJob);
    mainMaestro.EnqueueJob( &mut jobId);
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
    while let  	Some( handle) = handles.Pop() {
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
    let  	worker = Worker::New();
    budTree.Post( &worker);
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    budTree.Post( mainMaestro);
    //let  	mut jobId = mainMaestro.ConstructJob( U16( 0), Box::new( chore));
    //mainMaestro.EnqueueJob( &mut jobId);
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
    let  	mut jobId = mainMaestro.ConstructJob( atelier.Terminal(), quickSorter);
    mainMaestro.EnqueueJob( &mut jobId);
    atelier.DoLaunch();
    assert!( arr.SortSanity( |a, b| { a > b }));
    arr.USeg().Traverse( |i| {
        print!( "{} ", arr.At( i));
    });
    println!();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestDoQSortSequential()
{
    let  	buff = Buff::Create( U32( 100), |_| U32( rand::random::<u32>() % 128));
    let  	quickSorter = buff.Arr().QuickSorter( |a, b| a > b);
    let  	worker = Worker::New();
    quickSorter( &worker);
    assert!( buff.Arr().SortSanity( |a, b| { a > b }));
    buff.Arr().USeg().Traverse( |i| {
        print!( "{} ", buff.Arr().At( i));
    });
    println!();
}

//---------------------------------------------------------------------------------------------------------------------------------
