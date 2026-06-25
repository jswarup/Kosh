//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, Buff, IAccess, IArr, Stash, Stk, USeg, U8, U16, U32, U64 };
use	crate::stalks::{ Atm, Worker };
use	std::sync::Arc;
use	std::thread;

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
    // Test creation and push/pop on Buff
    let  	mut buffFromVec = Buff::NewEmpty();
    assert_eq!( buffFromVec.len(), 0);
    buffFromVec.Push( 40);
    buffFromVec.Push( 50);
    assert_eq!( buffFromVec.len(), 2);
    assert_eq!( buffFromVec[0], 40);
    assert_eq!( buffFromVec[1], 50);
    assert_eq!( buffFromVec.Pop(), Some( 50));
    assert_eq!( buffFromVec.Pop(), Some( 40));
    assert_eq!( buffFromVec.Pop(), None);
    // Test creation from an array directly
    let  	buffFromArr = Buff::from( [100, 200, 300, 400]);
    assert_eq!( buffFromArr.len(), 4);
    assert_eq!( buffFromArr[2], 300);
    let  	buff1 = Buff::New( 10, ());
    assert_eq!( buff1.Size(), 10);
    assert_eq!( buff1[5], ());
    let  	buff2 = Buff::Create( 5, |_| 42);
    let  	handle = std::thread::spawn( move || {
        assert_eq!( buff2.len(), 5);
        assert_eq!( buff2[0], 42);
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
        // Test SwapAt
        let  	mut val = 300;
        arr.SwapAt( 0, &mut val);
        assert_eq!( *arr.At( 0), 300);
        // Test Swap
        arr.Swap( 0, 2);
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
fn	TestArrFromArr()
{
    // Test creating an Arr from an array reference
    let  	arrData = [10u32, 20u32, 30u32];
    let  	arr = Arr::from( &arrData);
    assert_eq!( arr.Size(), 3);
    assert_eq!( *arr.At( 0), 10);
    assert_eq!( *arr.At( 1), 20);
    assert_eq!( *arr.At( 2), 30);
    // Test creating a mutable Arr from a mutable array reference
    let  	mut arrDataMut = [100u32, 200u32, 300u32];
    let  	arrMut = Arr::from( &mut arrDataMut);
    assert_eq!( arrMut.len(), 3);
    arrMut.SetAt( 1, &250u32);
    assert_eq!( *arrMut.At( 1), 250);
    assert_eq!( arrDataMut[1], 250);
    // Test creating an Arr from a slice
    let  	sliceData: &[u32] = &[1, 2, 3, 4];
    let  	arrSlice = Arr::from( sliceData);
    assert_eq!( arrSlice.len(), 4);
    assert_eq!( *arrSlice.At( 3), 4);
    // Test Arr<'a, U8>::Str()
    let  	arrU8Data = [U8( b'h'), U8( b'e'), U8( b'l'), U8( b'l'), U8( b'o')];
    let  	arrU8 = Arr::from( &arrU8Data);
    assert_eq!( arrU8.Str(), "hello");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	USegBasicOpsTest()
{
    let  	seg = USeg::New( 10, 11);
    assert_eq!( seg.First(), 10);
    assert_eq!( seg.Last(), 20);
    assert_eq!( seg.Size(), 11);
    assert!( !seg.IsEmpty());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	USegSnipTest()
{
    let  	seg = USeg::New( 10, 11);
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
    let  	seg = USeg::New( 10, 6);
    // Case 1: All values return true
    let  	mut visited = Buff::NewEmpty();
    seg.Traverse( |val| {
        visited.Push( val);
    });
    assert_eq!( &*visited, &[U32( 10), U32( 11), U32( 12), U32( 13), U32( 14), U32( 15)]);
    // Case 2: One value returns false ( early termination)
    let  	mut visited2 = Buff::NewEmpty();
    let  	result2 = seg.Span( |val| {
        visited2.Push( val);
        val < 13
    });
    assert!( !result2);
    assert_eq!( &*visited2, &[U32( 10), U32( 11), U32( 12), U32( 13)]);
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
        let     val = i;
        assert!( stack.Push( val), "push failed at {}", i);
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
    let  	mut x = U32::from( 10u32);
    x += b;
    assert_eq!( x, 13);
    x -= b;
    assert_eq!( x, 10);
    x *= b;
    assert_eq!( x, 30);
    x /= b;
    assert_eq!( x, 10);
    x %= b;
    assert_eq!( x, 1);
    let  	mut y = U32::from( 6u32);
    y &= b;
    assert_eq!( y, 2);
    y |= b;
    assert_eq!( y, 3);
    y ^= U32::from( 1u32);
    assert_eq!( y, 2);
    y <<= 1u32;
    assert_eq!( y, 4);
    y >>= 1u32;
    assert_eq!( y, 2);
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
fn	UInt8TestFrom()
{
    let  	_q = U8( 0);
    let  	a: U8 = 5u8.into();
    assert_eq!( a, 5);
    let  	b: U8 = ( -3i32).into();
    assert_eq!( b, ( -3i32) as u8);
    let  	c: U8 = ( 10usize).into();
    assert_eq!( c, 10);
}
fn	UInt8TestArith()
{
    let  	a = U8( 10);
    let  	b = U8( 3);
    assert_eq!( ( a + b), 13);
    assert_eq!( ( a - b), 7);
    assert_eq!( ( a * b), 30);
    assert_eq!( ( a / b), 3);
    assert_eq!( ( a % b), 1);
}
fn	UInt8TestNegNot()
{
    let  	a = U8( 0);
    assert_eq!( ( -a), 0);
    let  	b = U8( 5);
    assert_eq!( ( -b), 0u8.wrapping_sub( 5));
    assert_eq!( ( !b), !5u8);
}
fn	UInt64TestFrom()
{
    let  	_q = U64( 0);
    let  	a: U64 = 5u64.into();
    assert_eq!( a, 5);
    let  	b: U64 = ( -3i32).into();
    assert_eq!( b, ( -3i32) as u64);
    let  	c: U64 = ( 10usize).into();
    assert_eq!( c, 10);
}
fn	UInt64TestArith()
{
    let  	a = U64( 10);
    let  	b = U64( 3);
    assert_eq!( ( a + b), 13);
    assert_eq!( ( a - b), 7);
    assert_eq!( ( a * b), 30);
    assert_eq!( ( a / b), 3);
    assert_eq!( ( a % b), 1);
}
fn	UInt64TestNegNot()
{
    let  	a = U64( 0);
    assert_eq!( ( -a), 0);
    let  	b = U64( 5);
    assert_eq!( ( -b), 0u64.wrapping_sub( 5));
    assert_eq!( ( !b), !5u64);
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
    UInt8TestFrom();
    UInt8TestArith();
    UInt8TestNegNot();
    UInt64TestFrom();
    UInt64TestArith();
    UInt64TestNegNot();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
#[allow( dead_code)]
fn	StackExportImportOps()
{
    let  	srcStash = Stash::< U32>::New( 10, 0, U32( 0));
    let  	srcStk = srcStash.Stk();
    for i in 1..=5u32 {
        let     val = U32( i);
        assert!( srcStk.Push( val));
    }
    assert_eq!( srcStk.Size(), 5);
    // Destination stack initially empty
    let  	dstStash = Stash::< U32>::New( 10, 0, U32( 0));
    let  	dstStk = dstStash.Stk();
    assert_eq!( dstStk.Size(), 0);
    // Export from source to destination ( move all 5 elements)
    let  	moved = srcStk.Export( &dstStk, 5);
    assert_eq!( moved, 5);
    assert_eq!( srcStk.Size(), 0);
    assert_eq!( dstStk.Size(), 5);
    // Verify order in destination stack ( should be LIFO 5..=1)
    for expected in ( 1..=5u32).rev() {
        let  	mut out = U32( 0);
        assert!( dstStk.Pop( &mut out));
        assert_eq!( out, expected);
    }
    assert_eq!( dstStk.Size(), 0);
    // Refill source stack for Import test
    for i in 10..=14u32 {
        let     v = U32( i);
        assert!( srcStk.Push( v));
    }
    assert_eq!( srcStk.Size(), 5);
    // Import from source into destination ( move all 5 elements)
    let  	imported = dstStk.Import( &srcStk, 5);
    // Import uses a mutable reference, srcStk remains usable.
    assert_eq!( imported, 5);
    assert_eq!( dstStk.Size(), 5);
    // Verify imported order ( LIFO, should be 14..=10)
    for expected in ( 10..=14u32).rev() {
        let  	mut out = U32( 0);
        assert!( dstStk.Pop( &mut out));
        assert_eq!( out, expected);
    }
    assert_eq!( dstStk.Size(), 0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestConcurrentStackOps()
{
    // Create a shared destination stack of size 1000
    let  	dstStash = Arc::new( Stash::< U32>::New( 1000, 0, U32( 0)));
    let  	mut handles = Buff::NewEmpty();
    for t in 0..10 {
        let  	dstStkClone = dstStash.clone();
        let  	handle = thread::spawn( move || {
            // Create a thread-local source stack
            let  	srcStash = Stash::< U32>::New( 10, 0, U32( 0));
            let  	srcStk = srcStash.Stk();
            for i in 0..10 {
                let  	v = U32( t * 10 + i);
                srcStk.Push( v);
            }
            // Import elements from local srcStk to shared dstStk
            dstStkClone.Stk().Import( &srcStk, 10);
        });
        handles.Push( handle);
    }
    while let  	Some( handle) = handles.Pop() {
        handle.join().unwrap();
    }
    // Since 10 threads imported 10 elements each, dstStk size must be exactly 100
    assert_eq!( dstStash.Size(), 100);
    // Collect all elements and verify they are exactly 0..100 ( in some order)
    let  	mut values = Buff::NewEmpty();
    let  	dstStk = dstStash.Stk();
    let  	mut out = U32( 0);
    while dstStk.Pop( &mut out) {
        values.Push( out);
    }
    assert_eq!( values.len(), 100);
    values.sort();
    for i in 0..100 {
        assert_eq!( values[i], i as u32);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	QSortTest()
{
    let  	buff = Buff::Create( U32( 256), |_| rand::random::< f64>());
    //let     buff =  Buff::New( 5, | i| i);
    let  	arr = buff.Arr();
    arr.USeg().QSort( |i, j| arr.At( i) > arr.At( j), |i, j| {
            arr.Swap( i, j);
        });
    print! { "{:?}\n", arr};
    let  	res = arr.USeg().RSnip( 1).Span( |k| arr.At( k) > arr.At( k + 1));
    assert!( res);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestQSortBoundaries()
{
    // Test QSort with empty segment
    let  	buffEmpty = Buff::Create( U32( 0), |_| 0);
    let  	arrEmpty = buffEmpty.Arr();
    arrEmpty
        .USeg()
        .QSort( |i, j| arrEmpty.At( i) > arrEmpty.At( j), |i, j| {
            arrEmpty.Swap( i, j);
        });
    assert_eq!( arrEmpty.len(), 0);
    // Test QSort with size 1
    let  	buffOne = Buff::Create( U32( 1), |_| 42);
    let  	arrOne = buffOne.Arr();
    arrOne
        .USeg()
        .QSort( |i, j| arrOne.At( i) > arrOne.At( j), |i, j| {
            arrOne.Swap( i, j);
        });
    assert_eq!( arrOne[0], 42);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestDoQSort()
{
    let  	buff = Buff::Create( U32( 100), |_| rand::random::< f64>());
    let  	arr = buff.Arr();
    let  	worker = Worker::New();
    arr.USeg().DoQSort(
        &worker,
        &|i, j| arr.At( i) > arr.At( j),
        &|i, j| {
            arr.Swap( i, j);
        },
    );
    let  	res = arr.USeg().RSnip( 1).Span( |k| arr.At( k) > arr.At( k + 1));
    arr.USeg().Traverse( |i| {
        print!( "{} ", arr.At( i));
    });
    assert!( res);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestStashDynamicPushback()
{
    // Test with initial size 2
    let  	mut stash = Stash::< U32>::New( 2, 0, U32( 8));
    assert_eq!( stash.Size(), 0);
    assert_eq!( stash.Size() + stash.Stk().SzVoid(), 2);
    // Push first element
    let  	mut val1 = U32( 10);
    stash.PushX( &mut val1);
    assert_eq!( stash.Size(), 1);
    assert_eq!( stash.Size() + stash.Stk().SzVoid(), 2);
    // Push second element
    let  	mut val2 = U32( 20);
    stash.PushX( &mut val2);
    assert_eq!( stash.Size(), 2);
    assert_eq!( stash.Size() + stash.Stk().SzVoid(), 2);
    // Push third element ( should trigger resize to 4)
    let  	mut val3 = U32( 30);
    stash.PushX( &mut val3);
    assert_eq!( stash.Size(), 3);
    assert_eq!( stash.Size() + stash.Stk().SzVoid(), 4);
    // Push fourth element
    let  	mut val4 = U32( 40);
    stash.PushX( &mut val4);
    assert_eq!( stash.Size(), 4);
    assert_eq!( stash.Size() + stash.Stk().SzVoid(), 4);
    // Push fifth element ( should trigger resize to 8)
    let  	mut val5 = U32( 50);
    stash.PushX( &mut val5);
    assert_eq!( stash.Size(), 5);
    assert_eq!( stash.Size() + stash.Stk().SzVoid(), 8);
    // Pop and verify LIFO order and contents
    let  	stk = stash.Stk();
    let  	mut out = U32( 0);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 50);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 40);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 30);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 20);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 10);
    assert!( !stk.Pop( &mut out));
    // Test with initial size 0
    let  	mut stash0 = Stash::< U32>::New( 0, 0, U32( 0));
    assert_eq!( stash0.Size(), 0);
    assert_eq!( stash0.Size() + stash0.Stk().SzVoid(), 0);
    let  	mut v = U32( 100);
    stash0.PushX( &mut v);
    assert_eq!( stash0.Size(), 1);
    assert_eq!( stash0.Size() + stash0.Stk().SzVoid(), 1);
    let  	mut v2 = U32( 200);
    stash0.PushX( &mut v2);
    assert_eq!( stash0.Size(), 2);
    assert_eq!( stash0.Size() + stash0.Stk().SzVoid(), 2);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestStashAppend()
{
    // Test with initial size 2
    let  	mut stash = Stash::< U32>::New( 2, 0, U32( 0));
    // Create an Arr of elements to append
    let  	mut data = [U32( 10), U32( 20), U32( 30)];
    let  	arr = Arr::from( &mut data);
    stash.Append( arr);
    assert_eq!( stash.Size(), 3);
    // Buffer should resize to exactly 3 since neededSz ( 3) > current capacity ( 2)
    assert_eq!( stash.Size() + stash.Stk().SzVoid(), 3);
    // Pop and verify elements ( LIFO order: 30, 20, 10)
    let  	stk = stash.Stk();
    let  	mut out = U32( 0);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 30);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 20);
    assert!( stk.Pop( &mut out));
    assert_eq!( out, 10);
    assert!( !stk.Pop( &mut out));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, PartialEq, Debug)]
struct NonDefaultStruct
{
    value: i32,
}
#[test]
fn	TestBuffResize()
{
    // Create a buffer of size 3 initialized with NonDefaultStruct
    let  	mut buff = Buff::New( 3, NonDefaultStruct { value: 42 });
    assert_eq!( buff.len(), 3);
    assert_eq!( buff[0].value, 42);
    // Resize using Resize
    buff.Resize( U32( 5), |_| NonDefaultStruct { value: 100 });
    assert_eq!( buff.len(), 5);
    assert_eq!( buff[2].value, 42);
    assert_eq!( buff[3].value, 100);
    assert_eq!( buff[4].value, 100);
}


