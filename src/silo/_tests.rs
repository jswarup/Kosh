//- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use crate::silo::buff::Buff;
use crate::silo::useg::USeg;
use crate::silo::atm::Atm;
use std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffBasicOpsTest()
{
    let mut buff = Buff::New( 10, 42);
    assert_eq!( buff.len(), 10);
    assert_eq!( buff[ 0], 42);
    assert_eq!( buff[ 1], 42);
    assert_eq!( buff[ 2], 42);

    buff[ 1] = 100;
    assert_eq!( buff[ 1], 100);

    // Test slice methods made available via Deref
    assert_eq!( buff.first(), Some( &42));
    assert_eq!( buff.last(), Some( &42));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffFromTest()
{
    // Test creation from a slice
    let sliceData = [ 10, 20, 30];
    let buffFromSlice = Buff::from( &sliceData[ ..]);
    assert_eq!( buffFromSlice.len(), 3);
    assert_eq!( buffFromSlice[ 0], 10);
    assert_eq!( buffFromSlice[ 1], 20);
    assert_eq!( buffFromSlice[ 2], 30);

    // Test creation from a Vec
    let vecData = vec![ 40, 50];
    let buffFromVec = Buff::from( vecData);
    assert_eq!( buffFromVec.len(), 2);
    assert_eq!( buffFromVec[ 0], 40);
    assert_eq!( buffFromVec[ 1], 50);

    // Test creation from an array directly
    let buffFromArr = Buff::from( [ 100, 200, 300, 400]);
    assert_eq!( buffFromArr.len(), 4);
    assert_eq!( buffFromArr[ 2], 300);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BufZSTTest()
{
    let buff = Buff::New( 10, ());
    assert_eq!( buff.Size(), 10);
    assert_eq!( buff[ 5], ());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffSendSyncTest()
{
    let buff = Buff::CreateD( 5, |_| {42});
    let handle = std::thread::spawn( move ||
    {
        assert_eq!( buff.len(), 5);
        assert_eq!( buff[ 0], 42);
    });

    handle.join().unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn ArrBasicOpsTest()
{
    let mut buff = Buff::New( 3, 42);
    {
        let mut arr = buff.AsMutArr();
        assert_eq!( arr.len(), 3);
        assert_eq!( arr[ 0], 42);
        arr[ 1] = 100;

        // Test At
        assert_eq!( *arr.At( 1), 100);

        // Test SetAt
        arr.SetAt( 2, &200);
        assert_eq!( *arr.At( 2), 200);

        // Test MoveAt
        let mut val = 300;
        arr.MoveAt( 0, &mut val);
        assert_eq!( *arr.At( 0), 300);
        assert_eq!( val, 0); // 0 is i32 default

        // Test SwapAt
        arr.SwapAt( 0, 2);
        assert_eq!( *arr.At( 0), 200);
        assert_eq!( *arr.At( 2), 300);
    }
    assert_eq!( buff[ 0], 200);
    assert_eq!( buff[ 1], 100);
    assert_eq!( buff[ 2], 300);

    let arr2 = buff.AsArr();
    assert_eq!( arr2[ 1], 100);

    // Test Debug trait
    assert_eq!( format!( "{:?}", arr2), "[200, 100, 300]");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegBasicOpsTest()
{
    let seg = USeg::Create( 10, 11);
    assert_eq!( seg.First(), 10);
    assert_eq!( seg.Last(), 20);
    assert_eq!( seg.Size(), 11);
    assert!( !seg.IsEmpty());

}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegSnipTest()
{
    let seg = USeg::Create( 10, 11);

    // Test LSnip
    let lSnipped = seg.LSnip( 5);
    assert_eq!( lSnipped.First(), 15);
    assert_eq!( lSnipped.Last(), 20);
    assert_eq!( lSnipped.Size(), 6);

    let lEmpty = seg.LSnip( 11);
    assert!( lEmpty.IsEmpty());

    let lOverflow = seg.LSnip( 15);
    assert!( lOverflow.IsEmpty());

    // Test RSnip
    let rSnipped = seg.RSnip( 4);
    assert_eq!( rSnipped.First(), 10);
    assert_eq!( rSnipped.Last(), 16);
    assert_eq!( rSnipped.Size(), 7);

    let rEmpty = seg.RSnip( 11);
    assert!( rEmpty.IsEmpty());

    let rUnderflow = seg.RSnip( 20);
    assert!( rUnderflow.IsEmpty());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegSpanTest()
{
    let seg = USeg::Create( 10, 6);

    // Case 1: All values return true
    let mut visited = Vec::new();
    let result = seg.Span( |val| {
        visited.push( val);
        true
    });
    assert!( result);
    assert_eq!( visited, vec![ 10, 11, 12, 13, 14, 15]);

    // Case 2: One value returns false (early termination)
    let mut visited2 = Vec::new();
    let result2 = seg.Span( |val| {
        visited2.push( val);
        val < 13
    });
    assert!( !result2);
    assert_eq!( visited2, vec![ 10, 11, 12, 13]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn QSortTest()
{
    let     buff =  Buff::CreateD( 256, |_| rand::random::<f64>());
    //let     buff =  Buff::CreateD( 5, | i| i);

    let     arr = buff.AsArr();
    arr.USeg().QSort( &| i, j| { arr.At( i) > arr.At( j) }, &mut | i, j| { arr.SwapAt(i, j);});
    print!{ "{:?}\n", arr};
    let     res = arr.USeg().RSnip(1).Span(| k|{ arr.At( k) > arr.At( k +1)});
    assert!( res);
}


//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn TestAtmBasicOps() {
    let atmVar = Atm::New(10i32);

    // Test Get and Set
    assert_eq!(atmVar.Get(), 10);
    atmVar.Set(42);
    assert_eq!(atmVar.Get(), 42);

    // Test FetchAdd
    let prevVal = atmVar.FetchAdd(8, Ordering::SeqCst);
    assert_eq!(prevVal, 42);
    assert_eq!(atmVar.Get(), 50);

    // Test CompareExchange (success)
    let successRes = atmVar.CompareExchange(50, 100, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!(successRes, Ok(50));
    assert_eq!(atmVar.Get(), 100);

    // Test CompareExchange (failure)
    let failureRes = atmVar.CompareExchange(50, 200, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!(failureRes, Err(100));
    assert_eq!(atmVar.Get(), 100);
}

#[test]
fn TestAtmUsize() {
    let atmVar = Atm::New(0usize);

    atmVar.FetchAdd(1, Ordering::SeqCst);
    assert_eq!(atmVar.Get(), 1);
}

//---------------------------------------------------------------------------------------------------------------------------------
