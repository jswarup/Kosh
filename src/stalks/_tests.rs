//- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::silo::arr::Arr;
use	crate::silo::uint::U32;
use	crate::stalks::work::{ IWorker, WorkFn, Worker };
use	std::sync::Arc;
use	std::sync::atomic::{ AtomicBool, Ordering };
use	crate::stalks::atm::Atm;
use	crate::stalks::bud::TraversalEvent;

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestAtmBasicOps()
{
	let  	atmVar = Atm::New( 10i32);
    // Test Get and Set
    assert_eq!( atmVar.Get(), 10);
    atmVar.Set( 42);
    assert_eq!( atmVar.Get(), 42);
    // Test FetchAdd
	let  	prevVal = atmVar.FetchAdd( 8, Ordering::SeqCst);
    assert_eq!( prevVal, 42);
    assert_eq!( atmVar.Get(), 50);
    // Test CompareExchange (success)
	let  	successRes = atmVar.CompareExchange( 50, 100, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!( successRes, Ok( 50));
    assert_eq!( atmVar.Get(), 100);
    // Test CompareExchange (failure)
	let  	failureRes = atmVar.CompareExchange( 50, 200, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!( failureRes, Err( 100));
    assert_eq!( atmVar.Get(), 100);

    let  	atmVar1: Atm<U32> = Atm::New( U32( 0));
    atmVar1.FetchAdd( 1, Ordering::SeqCst);
    assert_eq!( atmVar1.Get(), 1);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWorkerPost()
{
	let  	run1 = Arc::new( AtomicBool::new( false));
	let  	run2 = Arc::new( AtomicBool::new( false));
	let  	run1C = run1.clone();
	let  	run2C = run2.clone();

	let  	job1: Box< WorkFn< '_>> = Box::new( move |_worker| {
        run1C.store( true, Ordering::SeqCst);
    });

	let  	job2: Box< WorkFn< '_>> = Box::new( move |_worker| {
        run2C.store( true, Ordering::SeqCst);
    });

	let  	mut jobsVec = vec![ job1, job2];
	let  	arr = Arr::from( &mut jobsVec[..]);

	let  	worker = Worker::New();
    worker.PostJobs( arr);

    assert!( run1.load( Ordering::SeqCst));
    assert!( run2.load( Ordering::SeqCst));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBudBasicOps()
{
	let  	a = U32( 10);
	let  	b = U32( 5);
	let  	c = U32( 12);
	let  	d = U32( 20);
	let  	e = U32( 1);
	let  	f = U32( 8);

	let  	x  = crate::bud!( ((( a | b) < c) | ( d | ( e < f))) );

    assert_eq!( x.Id(), U32( 21));

	let  	xLeft = x.Left().unwrap();
    assert_eq!( xLeft.Id(), U32( 0));

	let  	xRight = x.Right().unwrap();
    assert_eq!( xRight.Id(), U32( 21));

	let  	xLeftLeft = xLeft.Left().unwrap();
    assert_eq!( xLeftLeft.Id(), U32( 15));

	let  	mut visitedEvents = Vec::new();
    x.TraverseDFS( &mut |node, event| {
        visitedEvents.push( ( node.Id(), event));
    });

    assert_eq!( visitedEvents.len(), 16);
    assert_eq!( visitedEvents[0], ( U32( 21), TraversalEvent::Entry));
    assert_eq!( visitedEvents[1], ( U32( 0), TraversalEvent::Entry));
    assert_eq!( visitedEvents[2], ( U32( 15), TraversalEvent::Entry));
    assert_eq!( visitedEvents[3], ( U32( 10), TraversalEvent::Entry));
    assert_eq!( visitedEvents[4], ( U32( 5), TraversalEvent::Entry));
    assert_eq!( visitedEvents[5], ( U32( 15), TraversalEvent::Exit));
    assert_eq!( visitedEvents[6], ( U32( 12), TraversalEvent::Entry));
    assert_eq!( visitedEvents[7], ( U32( 0), TraversalEvent::Exit));
    assert_eq!( visitedEvents[8], ( U32( 21), TraversalEvent::Entry));
    assert_eq!( visitedEvents[9], ( U32( 20), TraversalEvent::Entry));
    assert_eq!( visitedEvents[10], ( U32( 1), TraversalEvent::Entry));
    assert_eq!( visitedEvents[11], ( U32( 1), TraversalEvent::Entry));
    assert_eq!( visitedEvents[12], ( U32( 8), TraversalEvent::Entry));
    assert_eq!( visitedEvents[13], ( U32( 1), TraversalEvent::Exit));
    assert_eq!( visitedEvents[14], ( U32( 21), TraversalEvent::Exit));
    assert_eq!( visitedEvents[15], ( U32( 21), TraversalEvent::Exit));
    x.Print();
}

//---------------------------------------------------------------------------------------------------------------------------------
