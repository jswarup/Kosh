//- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::silo::arr::Arr;
use	crate::stalks::work::{ IWorker, Worker };
use	std::sync::Arc;
use	std::sync::atomic::{ AtomicBool, Ordering };

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWorkerPost()
{
	let  	run1 = Arc::new( AtomicBool::new( false));
	let  	run2 = Arc::new( AtomicBool::new( false));
	let  	run1_c = run1.clone();
	let  	run2_c = run2.clone();

	let  	job1: Box< dyn FnMut( &dyn IWorker) + Send + Sync> = Box::new( move |_worker| {
        run1_c.store( true, Ordering::SeqCst);
    });

	let  	job2: Box< dyn FnMut( &dyn IWorker) + Send + Sync> = Box::new( move |_worker| {
        run2_c.store( true, Ordering::SeqCst);
    });

	let  	mut jobs_vec = vec![ job1, job2];
	let  	arr = Arr::from( &mut jobs_vec[..]);

	let  	worker = Worker::New();
    worker.Post( arr);

    assert!( run1.load( Ordering::SeqCst));
    assert!( run2.load( Ordering::SeqCst));
}

//---------------------------------------------------------------------------------------------------------------------------------
