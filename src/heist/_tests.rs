//- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use crate::{
    heist::{
        atelier::Atelier,
        maven::Maven,
    },
    silo::uint::U32
};

//---------------------------------------------------------------------------------------------------------------------------------
#[test]
fn	BuffBasicAtelierTest()
{
    fn trialJob( m : &mut Maven)
    {
        println!( "Trial {}", m.Index());
    }
    let mut atelier = Atelier::New( U32( 4));
    let     maven = atelier.Mavens().At( 0);
    let mut jobId = atelier.ConstructJob( maven.Index(), trialJob);
    atelier.EnqueueJob( maven.Index(), &mut jobId);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestThreadSharedInteger()
{
    use std::sync::{Arc, Mutex};
    use std::thread;

    let shared = Arc::new( Mutex::new( 0));
    let mut handles = vec![];

    for i in 0..4 {
        let shared_clone = shared.clone();
        let handle = thread::spawn( move || {
            let mut val = shared_clone.lock().unwrap();
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

