//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::
    { Atelier, Chore, Maestro },
    silo::
    { Buff, IAccess, IArr, U16, U32 },
    stalks::
    { DynINode, DynIWorker, IWorker, Worker },
};
use	std::sync::{ Arc, Mutex };
use	std::thread;

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
        });
        jobId = maestro.ConstructJob( jobId, |w2: &DynIWorker< '_>| {
            println!( "Trial2 {}", Maestro::FromWorker( w2).MaestroIndex());
        });
        maestro.EnqueRunJob( &mut jobId);
        println!( "Trial {}", maestro.MaestroIndex());
    }
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    let  	mut jobId = mainMaestro.ConstructJob( U16( 0), trialJob);
    mainMaestro.EnqueRunJob( &mut jobId);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestThreadSharedInteger()
{
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
    let  	aChore = Chore::New( |_m| {
        print!( "{} ", 10);
    });
    let  	bChore = Chore::New( |_m| {
        print!( "{} ", 20);
    });
    let  	cChore = Chore::New( |_m| {
        print!( "{} ", 40);
    });
    let  	_choreTreeMacro = crate::ChoreTree!( ( cChore
            < ( bChore
                | aChore
                | ( |_m| {
                    print!( "{} ", 50);
                })))
    );
    let  	worker = Worker::New();
    worker.Tender( aChore);
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    mainMaestro.Tender( aChore);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestChoreTree()
{
    let  	aChore = Chore::New( |_m| {
        print!( "{} ", 10);
    });
    let  	bChore = Chore::New( |_m| {
        print!( "{} ", 20);
    });
    let  	cChore = Chore::New( |_m| {
        print!( "{} ", 40);
    });
    let  	choreTree = crate::ChoreTree!( ( cChore
            < ( bChore
                | aChore
                | ( |_m| {
                    print!( "{} ", 50);
                })))
    );
 
    ( &choreTree as &DynINode< '_>).DiveDf( &mut |probe| {
        let  	arr = probe.Arr();
        println!( "DiveDf reached a path of depth: {}", arr.Size().0);
    });

    
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro(); 
    mainMaestro.PostNode(  &choreTree);
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
    mainMaestro.Tender( quickSorter);
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
    worker.Tender( quickSorter);
    assert!( buff.Arr().SortSanity( |a, b| { a > b }));
    buff.Arr().USeg().Traverse( |i| {
        print!( "{} ", buff.Arr().At( i));
    });
    println!();
}

//---------------------------------------------------------------------------------------------------------------------------------
