//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::{ JsonOutStream, xflux::XField, IXFlux }, ChoreTree };
use	std::fs;
use	crate::{
    heist::
    { Atelier, Chore, Maestro },
    silo::
    { Buff, IAccess, IArr, U16, U32 },
    stalks::
    { DynIWorker, IWorker, Worker, IntoWorkPtr },
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
    let  	aChore = crate::Chore!( "10S", |_m| {
        print!( "{} ", 10);
    });
    let  	bChore = crate::Chore!( "20S", |_m| {
        print!( "{} ", 20);
    });
    let  	cChore = crate::Chore!( "40S", |_m| {
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
    worker.PostJob( aChore.IntoWorkPtr());
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    mainMaestro.PostJob( aChore.IntoWorkPtr());
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestChoreTree()
{
    let  	a  = crate::Chore!( "A", |_m| {
        print!( "{} ", "A");
    });
    let  	b  = crate::Chore!( "B", |_m| {
        print!( "{} ", "B");
    });
    let  	c = crate::Chore!( "C", |_m| {
        print!( "{} ", "C");
    }); 
    let  	d = crate::Chore!( "D", |_m| {
        print!( "{} ", "D");
    }); 
 
    let  	e = crate::Chore!( "E", |_m| {
        print!( "{} ", "E");
    }); 
 
    let  	f = crate::Chore!( "F", |_m| {
        print!( "{} ", "F");
    }); 
 
    let  	g = crate::Chore!( "G", |_m| {
        print!( "{} ", "G");
    }); 
 
    let  	h = crate::Chore!( "H", |_m| {
        print!( "{} ", "H");
    }); 
 
    let  	i = crate::Chore!( "i", |_m| {
        print!( "{} ", "I");
    }); 
 
    let  	j = crate::Chore!( "J", |_m| {
        print!( "{} ", "J");
    }); 
    
    let  	choreTree= crate::ChoreTree!( ((( ( a < b ) | ( c <  d)) < e) | ( ( f | g) < h)  | i) < j);
    //let  	choreTree= crate::ChoreTree!( ((( ( aChore | bChore ) < gChore))));
    
    let  	mut jsonStr = String::new();
    {
        let  	mut jsonOutStream = JsonOutStream::New( &mut jsonStr, true);
        IXFlux::Field( &mut jsonOutStream, XField::Fluxable( &choreTree));
    }
    fs::write( "a.json", jsonStr).unwrap();

    
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro(); 

    // Note: Calling DoWork manually in DiveDf consumes the job, which causes 
    // use-after-free panics when DoLaunch actually runs them asynchronously.
     
    mainMaestro.PostChoreTree(  &choreTree);
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
