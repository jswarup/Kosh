//- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::stalks::atm::Atm;
use	crate::silo::buff::Buff;
use	crate::silo::stash::Stash;
use	crate::silo::stk::Stk;
#[warn( unused_imports)]
use	crate::silo::uint::{ U16, U32 };
use	crate::silo::useg::USeg;
use	std::sync::atomic::Ordering;

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffBasicOpsTest()
{
	let  	mut buff = Buff::New( 10, 42);
    assert_eq!( buff.len(), 10);
    assert_eq!( buff[0], 42);
    assert_eq!( buff[1], 42);
    assert_eq!( buff[2], 42);
    buff[1] = 100;
    assert_eq!( buff[1], 100);
    // Test slice methods made available via Deref
    assert_eq!( buff.first(), Some( &42));
    assert_eq!( buff.last(), Some( &42));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffFromTest()
{
    // Test creation from a slice
	let  	sliceData = [10, 20, 30];
	let  	buffFromSlice = Buff::from( &sliceData[..]);
    assert_eq!( buffFromSlice.len(), 3);
    assert_eq!( buffFromSlice[0], 10);
    assert_eq!( buffFromSlice[1], 20);
    assert_eq!( buffFromSlice[2], 30);
    // Test creation from a Vec
	let  	vecData = vec![40, 50];
	let  	buffFromVec = Buff::from( vecData);
    assert_eq!( buffFromVec.len(), 2);
    assert_eq!( buffFromVec[0], 40);
    assert_eq!( buffFromVec[1], 50);
    // Test creation from an array directly
	let  	buffFromArr = Buff::from( [100, 200, 300, 400]);
    assert_eq!( buffFromArr.len(), 4);
    assert_eq!( buffFromArr[2], 300);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BufZSTTest()
{
	let  	buff = Buff::New( 10, ());
    assert_eq!( buff.Size(), 10);
    assert_eq!( buff[5], ());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	BuffSendSyncTest()
{
	let  	buff = Buff::Create( 5, |_| 42);
	let  	handle = std::thread::spawn( move || {
        assert_eq!( buff.len(), 5);
        assert_eq!( buff[0], 42);
    });
    handle.join().unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	ArrBasicOpsTest()
{
	let  	buff = Buff::New( 3, 42);
    {
		let  	mut arr = buff.Arr();
        assert_eq!( arr.len(), 3);
        assert_eq!( arr[0], 42);
        arr[1] = 100;
        // Test At
        assert_eq!( *arr.At( 1), 100);
        // Test SetAt
        arr.SetAt( 2, &200u32);
        assert_eq!( *arr.At( 2), 200);
        // Test MoveAt
		let  	mut val = 300;
        arr.MoveAt( 0, &mut val);
        assert_eq!( *arr.At( 0), 300);
        // Test SwapAt
        arr.SwapAt( 0, 2);
        assert_eq!( *arr.At( 0), 200);
        assert_eq!( *arr.At( 2), 300);
    }
    assert_eq!( buff[0], 200);
    assert_eq!( buff[1], 100);
    assert_eq!( buff[2], 300);
	let  	arr2 = buff.Arr();
    assert_eq!( arr2[1], 100);
    // Test Debug trait
    assert_eq!( format!( "{:?}", arr2), "[200, 100, 300]");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	USegBasicOpsTest()
{
	let  	seg = USeg::Create( 10, 11);
    assert_eq!( seg.First(), 10);
    assert_eq!( seg.Last(), 20);
    assert_eq!( seg.Size(), 11);
    assert!( !seg.IsEmpty());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	USegSnipTest()
{
	let  	seg = USeg::Create( 10, 11);
    // Test LSnip
	let  	lSnipped = seg.LSnip( 5);
    assert_eq!( lSnipped.First(), 15);
    assert_eq!( lSnipped.Last(), 20);
    assert_eq!( lSnipped.Size(), 6);
	let  	lEmpty = seg.LSnip( 11);
    assert!( lEmpty.IsEmpty());
	let  	lOverflow = seg.LSnip( 15);
    assert!( lOverflow.IsEmpty());
    // Test RSnip
	let  	rSnipped = seg.RSnip( 4);
    assert_eq!( rSnipped.First(), 10);
    assert_eq!( rSnipped.Last(), 16);
    assert_eq!( rSnipped.Size(), 7);
	let  	rEmpty = seg.RSnip( 11);
    assert!( rEmpty.IsEmpty());
	let  	rUnderflow = seg.RSnip( 20);
    assert!( rUnderflow.IsEmpty());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	USegSpanTest()
{
	let  	seg = USeg::Create( 10, 6);
    // Case 1: All values return true
	let  	mut visited = Vec::new();
	let  	result = seg.Span( |val| {
        visited.push( val);
        true
    });
    assert!( result);
    assert_eq!( visited, vec![10, 11, 12, 13, 14, 15]);
    // Case 2: One value returns false (early termination)
	let  	mut visited2 = Vec::new();
	let  	result2 = seg.Span( |val| {
        visited2.push( val);
        val < 13
    });
    assert!( !result2);
    assert_eq!( visited2, vec![10, 11, 12, 13]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	QSortTest()
{
	let  	buff = Buff::Create( U32( 256), |_| rand::random::<f64>());
    //let     buff =  Buff::New( 5, | i| i);
	let  	arr = buff.Arr();
    arr.USeg()
        .QSort( &|i, j| arr.At( i) > arr.At( j), &mut |i, j| {
            arr.SwapAt( i, j);
        });
    print! { "{:?}\n", arr};
	let  	res = arr.USeg().RSnip( 1).Span( |k| arr.At( k) > arr.At( k + 1));
    assert!( res);
}

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
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestAtmUsize()
{
	let  	atmVar: Atm<U32> = Atm::New( U32( 0));
    atmVar.FetchAdd( 1, Ordering::SeqCst);
    assert_eq!( atmVar.Get(), 1);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	StackBasicOps()
{
    // Create a buffer of size 10 initialized with zeros
	let  	buff = Buff::Create( U32( 10), |_| 0u32);
    // Atomic counter for size tracking
	let  	atm = Atm::New( U32( 0));
    // Obtain a mutable Arr view over the buffer
	let  	arr = buff.Arr();
    // Create the stack
	let  	stack = Stk::Create( &atm, arr);
    // Stack should start empty
    assert_eq!( stack.Size(), 0);
    // Push values 1..=5 onto the stack
    for i in 1..=5u32 {
		let  	mut val = i;
        assert!( stack.Push( &mut val), "push failed at {}", i);
    }
    assert_eq!( stack.Size(), 5);
    // Pop values and verify LIFO order
    for expected in ( 1..=5u32).rev() {
		let  	mut out = 0u32;
        assert!( stack.Pop( &mut out), "pop failed at {}", expected);
        assert_eq!( out, expected);
    }
    assert_eq!( stack.Size(), 0);
    // Popping from an empty stack should return false
	let  	mut out = 0u32;
    assert!( !stack.Pop( &mut out));
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	UIntTestFrom()
{
	let  	_q = U32::from( 0);
	let  	a: U32 = 5u32.into();
    assert_eq!( a, 5);
	let  	b: U32 = ( -3i32).into();
    assert_eq!( b, ( -3i32) as u32);
	let  	c: U32 = ( 10usize).into();
    assert_eq!( c, 10);
}
fn	UIntTestArith()
{
	let  	a = U32::from( 10u32);
	let  	b = U32::from( 3u32);
    assert_eq!( ( a + b), 13);
    assert_eq!( ( a - b), 7);
    assert_eq!( ( a * b), 30);
    assert_eq!( ( a / b), 3);
    assert_eq!( ( a % b), 1);
}
fn	UIntNegNotTest()
{
	let  	a = U32( 0);
    assert_eq!( ( -a), 0);
	let  	b = U32( 5);
    assert_eq!( ( -b), 0u32.wrapping_sub( 5));
    assert_eq!( ( !b), !5u32);
}
fn	UInt16TestFrom()
{
	let  	_q = U16( 0);
	let  	a: U16 = 5u16.into();
    assert_eq!( a, 5);
	let  	b: U16 = ( -3i32).into();
    assert_eq!( b, ( -3i32) as u16);
	let  	c: U16 = ( 10usize).into();
    assert_eq!( c, 10);
}
fn	UInt16TestArith()
{
	let  	a = U16( 10);
	let  	b = U16( 3);
    assert_eq!( ( a + b), 13);
    assert_eq!( ( a - b), 7);
    assert_eq!( ( a * b), 30);
    assert_eq!( ( a / b), 3);
    assert_eq!( ( a % b), 1);
}
fn	UInt16TestNegNot()
{
	let  	a = U16( 0);
    assert_eq!( ( -a), 0);
	let  	b = U16( 5);
    assert_eq!( ( -b), 0u16.wrapping_sub( 5));
    assert_eq!( ( !b), !5u16);
}
#[test]
fn	UIntBasicOps()
{
    UIntTestFrom();
    UIntTestArith();
    UIntNegNotTest();
    UInt16TestFrom();
    UInt16TestArith();
    UInt16TestNegNot();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
#[allow( dead_code)]
fn	StackExportImportOps()
{
	let  	srcStash = Stash::<U32>::New( 10);
	let  	srcStk = srcStash.Stk();
    for i in 1..=5u32 {
		let  	mut val = U32( i);
        assert!( srcStk.Push( &mut val));
    }
    assert_eq!( srcStk.Size(), 5);
    // Destination stack initially empty
	let  	dstStash = Stash::<U32>::New( 10);
	let  	dstStk = dstStash.Stk();
    assert_eq!( dstStk.Size(), 0);
    // Export from source to destination (move all 5 elements)
	let  	moved = srcStk.Export( &dstStk, 5);
    assert_eq!( moved, 5);
    assert_eq!( srcStk.Size(), 0);
    assert_eq!( dstStk.Size(), 5);
    // Verify order in destination stack (should be LIFO 5..=1)
    for expected in ( 1..=5u32).rev() {
		let  	mut out = U32( 0);
        assert!( dstStk.Pop( &mut out));
        assert_eq!( out, expected);
    }
    assert_eq!( dstStk.Size(), 0);
    // Refill source stack for Import test
    for i in 10..=14u32 {
		let  	mut v = U32( i);
        assert!( srcStk.Push( &mut v));
    }
    assert_eq!( srcStk.Size(), 5);
    // Import from source into destination (move all 5 elements)
	let  	imported = dstStk.Import( &srcStk, 5);
    // Import uses a mutable reference, srcStk remains usable.
    assert_eq!( imported, 5);
    assert_eq!( dstStk.Size(), 5);
    // Verify imported order (LIFO, should be 14..=10)
    for expected in ( 10..=14u32).rev() {
		let  	mut out = U32( 0);
        assert!( dstStk.Pop( &mut out));
        assert_eq!( out, expected);
    }
    assert_eq!( dstStk.Size(), 0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestQSortBoundaries()
{
    // Test QSort with empty segment
	let  	buff_empty = Buff::Create( U32( 0), |_| 0);
	let  	arr_empty = buff_empty.Arr();
    arr_empty
        .USeg()
        .QSort( &|i, j| arr_empty.At( i) > arr_empty.At( j), &mut |i, j| {
            arr_empty.SwapAt( i, j);
        });
    assert_eq!( arr_empty.len(), 0);
    // Test QSort with size 1
	let  	buff_one = Buff::Create( U32( 1), |_| 42);
	let  	arr_one = buff_one.Arr();
    arr_one
        .USeg()
        .QSort( &|i, j| arr_one.At( i) > arr_one.At( j), &mut |i, j| {
            arr_one.SwapAt( i, j);
        });
    assert_eq!( arr_one[0], 42);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestConcurrentStackOps()
{
    use	std::sync::Arc;
    use	std::thread;
    // Create a shared destination stack of size 1000
	let  	dstStash = Arc::new( Stash::<U32>::New( 1000));
	let  	mut handles = vec![];
    for t in 0..10 {
		let  	dstStk_clone = dstStash.clone();
		let  	handle = thread::spawn( move || {
            // Create a thread-local source stack
			let  	srcStash = Stash::<U32>::New( 10);
			let  	srcStk = srcStash.Stk();
            for i in 0..10 {
				let  	mut v = U32( t * 10 + i);
                srcStk.Push( &mut v);
            }
            // Import elements from local srcStk to shared dstStk
            dstStk_clone.Stk().Import( &srcStk, 10);
        });
        handles.push( handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    // Since 10 threads imported 10 elements each, dstStk size must be exactly 100
    assert_eq!( dstStash.Size(), 100);
    // Collect all elements and verify they are exactly 0..100 (in some order)
	let  	mut values = vec![];
	let  	dstStk = dstStash.Stk();
	let  	mut out = U32( 0);
    while dstStk.Pop( &mut out) {
        values.push( out.0);
    }
    assert_eq!( values.len(), 100);
    values.sort();
    for i in 0..100 {
        assert_eq!( values[i], i as u32);
    }
}
