//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::
    { Atelier, Chore, Maestro },
    silo::
    { Buff, IAccess, IArr, U16, U32 },
    stalks::
    { DynIWorker, IWorker, Worker },
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
        }, "TestJob1");
        jobId = maestro.ConstructJob( jobId, |w2: &DynIWorker< '_>| {
            println!( "Trial2 {}", Maestro::FromWorker( w2).MaestroIndex());
        }, "TestJob2");
        maestro.EnqueRunJob( &mut jobId);
        println!( "Trial {}", maestro.MaestroIndex());
    }
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    let  	mut jobId = mainMaestro.ConstructJob( U16( 0), trialJob, "TrialJob");
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
    let  	aChore = crate::Chore!( |_m| {
        print!( "{} ", 10);
    }, "10S");
    let  	bChore = crate::Chore!( |_m| {
        print!( "{} ", 20);
    }, "20S");
    let  	cChore = crate::Chore!( |_m| {
        print!( "{} ", 40);
    }, "40S");
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
    let  	aChore = crate::Chore!( |_m| {
        print!( "{} ", "A");
    }, "A");
    let  	bChore = crate::Chore!( |_m| {
        print!( "{} ", "B");
    }, "B");
    let  	cChore = crate::Chore!( |_m| {
        print!( "{} ", "C");
    }, "C"); 
    let  	dChore = crate::Chore!( |_m| {
        print!( "{} ", "D");
    }, "D"); 
 
    let  	eChore = crate::Chore!( |_m| {
        print!( "{} ", "E");
    }, "E"); 
 
    let  	fChore = crate::Chore!( |_m| {
        print!( "{} ", "F");
    }, "F"); 
 
    let  	gChore = crate::Chore!( |_m| {
        print!( "{} ", "G");
    }, "G"); 
 
    let  	choreTree= crate::ChoreTree!( ((( ( aChore < bChore ) | ( cChore <  dChore)) < eChore) | fChore) < gChore);
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro(); 

    // Note: Calling DoWork manually in DiveDf consumes the job, which causes 
    // use-after-free panics when DoLaunch actually runs them asynchronously.
    
    mainMaestro.PostNode(  &choreTree);
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
    mainMaestro.Tender( quickSorter);
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
    worker.Tender( quickSorter);
    assert!( buff.Arr().SortSanity( |a, b| { a > b })); 
    println!( "{} ", buff.Arr());  
}

//---------------------------------------------------------------------------------------------------------------------------------
