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
        jobId =  maestro.ConstructJob(  jobId, Box::new(|w1| {
            println!( "Trial1 {}", w1.AsMaestro().unwrap().MavenIndex());
        }));

        jobId =  maestro.ConstructJob(  jobId, Box::new(|w2| {
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
fn	TestConcurrentDAG()
{
    use	std::sync::Arc;
    use	std::sync::atomic::{ AtomicU32, Ordering };
    // A test DAG:
    // Job 1 (starts) -> Job 3 (successor)
    // Job 2 (starts) -> Job 3 (successor)
    // When Job 1 and Job 2 both finish, Job 3 is executed.
	let  	counter = Arc::new( AtomicU32::new( 0));
	let  	atelier = Atelier::New( U32( 4));
    // Construct Job 3 (which waits for Job 1 and Job 2)
	let  	counterClone3 = counter.clone();
	let  	job3 = atelier.ConstructJob( U32( 0), U16( 0), Box::new(move |_m| {
        counterClone3.fetch_add( 100, Ordering::SeqCst);
    }));
    // Construct Job 1
	let  	counterClone1 = counter.clone();
	let  	job1 = atelier.ConstructJob( U32( 0), U16( 0), Box::new(move |_m| {
        counterClone1.fetch_add( 1, Ordering::SeqCst);
    }));
    // Construct Job 2
	let  	counterClone2 = counter.clone();
	let  	job2 = atelier.ConstructJob( U32( 0), U16( 0), Box::new(move |_m| {
        counterClone2.fetch_add( 1, Ordering::SeqCst);
    }));
    // Set dependencies:
    // Job 3 has 2 predecessors
    atelier._SzPreds.Arr().At( job3).Set( U16( 2));
    // Job 1 and Job 2 have Job 3 as successor
    atelier._SuccIds.Arr().SetAt( job1, &job3);
    atelier._SuccIds.Arr().SetAt( job2, &job3);
    // Enqueue the starting jobs (Job 1 and Job 2)
	let  	mut j1 = job1;
	let  	mut j2 = job2;
    atelier.EnqueueJob( U32( 0), &mut j1);
    atelier.EnqueueJob( U32( 1), &mut j2);
    // Launch the processing queues
    atelier.DoLaunch();
    // Verify: Job 1 (+1), Job 2 (+1), and Job 3 (+100) must all run!
    // Total should be exactly 102.
    assert_eq!( counter.load( Ordering::SeqCst), 102);
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

	let  	mut jobId = mainMaestro.ConstructJob(  U16( 0), Box::new(move |worker| {
        arr.USeg().DoQSort( worker, move |i, j| arr.At( i) > arr.At( j), move |i, j| { arr.SwapAt( i, j); });
    }));
    mainMaestro.EnqueueJob( &mut jobId);

    atelier.DoLaunch();

	let  	res = arr.USeg().RSnip( 1).Span( |k| arr.At( k) > arr.At( k + 1));

    arr.USeg().Span( |i| {
        print!( "{} ", arr.At( i));
        true
    });
    assert!( res);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestChoreBuds()
{
    use crate::heist::chore::Chore;

    let atelier = Atelier::New( U32( 4));
    let mainMaestro = atelier.MainMaestro();
    let mut job = Chore::New( U32( 42) );
    let mut jobId = mainMaestro.ConstructJob( U16( 0), Box::new( move |worker| job.execute( worker)));
    mainMaestro.EnqueueJob( &mut jobId);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------
