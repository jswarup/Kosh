//- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist:: { atelier::Atelier, maestro::Maestro },
    silo:: {
        uint:: { U16, U32},
        buff::Buff,
    },
    stalks::work::IWorker
};

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffBasicAtelierTest()
{
    fn	trialJob( worker: &dyn IWorker)
    {
		let  	maestro = worker.AsMaestro().unwrap();
		let  	mut jobId = maestro.CurSuccId();
        jobId =  maestro.ConstructJob(  jobId, Box::new(|w1: &dyn IWorker| {
            println!( "Trial1 {}", w1.AsMaestro().unwrap().MavenIndex());
        }));

        jobId =  maestro.ConstructJob(  jobId, Box::new(|w2: &dyn IWorker| {
            println!( "Trial2 {}", w2.AsMaestro().unwrap().MavenIndex());
        }));
        maestro.EnqueueJob( &mut jobId);
        println!( "Trial {}", maestro.MavenIndex());
    }
	let  	atelier = Atelier::New( U32( 4));
	let  	mainMaestro = atelier.MainMaestro();
	let  	mut jobId = mainMaestro.ConstructJob(  U16( 0), Box::new(trialJob));
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
	let  	mut handles = vec![];
    for i in 0..4 {
		let  	sharedClone = shared.clone();
		let  	handle = thread::spawn( move || {
			let  	mut val = sharedClone.lock().unwrap();
            *val += 1;
            println!( "Thread {} incremented shared integer to: {}", i, *val);
        });
        handles.push( handle);
    }
    for handle in handles {
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
fn	TestDoQSort()
{
	let  	buff = Buff::Create( U32( 1000), |_| rand::random::<f64>());
	let  	arr = buff.Arr();

	let  	atelier = Atelier::New( U32( 4));
	let  	mainMaestro = atelier.MainMaestro();
    let  	mut jobId = U16( 0);
	jobId = mainMaestro.ConstructJob(  jobId, Box::new( |_worker: &dyn IWorker| {
        let  	_res = arr.USeg().RSnip( 1).Span( |k| arr.At( k) > arr.At( k + 1));

        arr.USeg().Span( |i| {
            print!( "{} ", arr.At( i));
            true
        });
        println!();
    }));
	jobId = mainMaestro.ConstructJob(  jobId, Box::new( |worker: &dyn IWorker| {
        arr.USeg().DoQSort( worker, move |i, j| arr.At( i) > arr.At( j), move |i, j| { arr.SwapAt( i, j); });
    }));
    mainMaestro.EnqueueJob( &mut jobId);

    atelier.DoLaunch();

}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestChoreBuds()
{
    use crate::heist::chore::Chore;


    let  	aChore: Chore = Chore::New( U32( 10));
    let  	bChore = Chore::New( U32( 20));
    let  	cChore = Chore::New( U32( 30));
    let  	budTree = crate::BudTree!( Chore, ( cChore < ( bChore | aChore ) ));
    budTree.Print();

    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();

    budTree.Post( &mainMaestro);
    //let  	mut jobId = mainMaestro.ConstructJob( U16( 0), Box::new( chore));
    //mainMaestro.EnqueueJob( &mut jobId);
    atelier.DoLaunch();
}


//---------------------------------------------------------------------------------------------------------------------------------
