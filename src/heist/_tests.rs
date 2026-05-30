//- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::
    { atelier::Atelier, maestro::Maestro },
    silo::uint::
    { U16, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffBasicAtelierTest()
{
    fn	trialJob( m: &Maestro< '_>)
    {
        println!( "Trial {}", 0);
    }
	let  	atelier = Atelier::New( U32( 4));
	let  	maven = atelier.Mavens().At( 0);
	let  	mut jobId = atelier.ConstructJob( maven.Index(), trialJob);
    atelier.PostJob( maven.Index(), true, &mut jobId);
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
		let  	shared_clone = shared.clone();
		let  	handle = thread::spawn( move || {
			let  	mut val = shared_clone.lock().unwrap();
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
	let  	counter_clone3 = counter.clone();
	let  	job3 = atelier.ConstructJob( U32( 0), move |_m| {
        counter_clone3.fetch_add( 100, Ordering::SeqCst);
    });
    // Construct Job 1
	let  	counter_clone1 = counter.clone();
	let  	job1 = atelier.ConstructJob( U32( 0), move |_m| {
        counter_clone1.fetch_add( 1, Ordering::SeqCst);
    });
    // Construct Job 2
	let  	counter_clone2 = counter.clone();
	let  	job2 = atelier.ConstructJob( U32( 0), move |_m| {
        counter_clone2.fetch_add( 1, Ordering::SeqCst);
    });
    // Set dependencies:
    // Job 3 has 2 predecessors
    atelier._SzPreds.Arr().At( job3).Set( U16( 2));
    // Job 1 and Job 2 have Job 3 as successor
    atelier._SuccIds.Arr().SetAt( job1, &job3);
    atelier._SuccIds.Arr().SetAt( job2, &job3);
    // Enqueue the starting jobs (Job 1 and Job 2)
	let  	mut j1 = job1;
	let  	mut j2 = job2;
    atelier.PostJob( U32( 0), true, &mut j1);
    atelier.PostJob( U32( 1), true, &mut j2);
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
