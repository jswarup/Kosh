use	crate::silo::uint::U32;
use	crate::stalks::atm::Atm;
use	std::sync::atomic::Ordering;

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
    // Test CompareExchange ( success)
    let  	successRes = atmVar.CompareExchange( 50, 100, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!( successRes, Ok( 50));
    assert_eq!( atmVar.Get(), 100);
    // Test CompareExchange ( failure)
    let  	failureRes = atmVar.CompareExchange( 50, 200, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!( failureRes, Err( 100));
    assert_eq!( atmVar.Get(), 100);
    let  	atmVar1: Atm< U32> = Atm::New( U32( 0));
    atmVar1.FetchAdd( 1, Ordering::SeqCst);
    assert_eq!( atmVar1.Get(), 1);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBudBasicOps() 
{
    let  	a = 1.8;
    let  	b = 5.7;
    let  	c = 12.2;
    let  	d = 20.7;
    let  	e = 1.5;
    let  	f = 8.1;
    let  	x = crate::BudTree!( ( ( ( a | b) < c) | ( d | ( e < f))));
    x.Print();
    let  	left = crate::BudTree!( a | b);
    let  	right = crate::BudTree!( c | d);
    let  	combined = crate::BudTree!( left | right);
    combined.Print();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
#[should_panic( expected = "Binary operation not supported for this type")]
fn	TestUnsupportedOpPanic() 
{
    #[derive( Clone, Default)]
    struct Dummy;
    impl crate::stalks::bud::Bud< Dummy> for Dummy 
    {
        fn	Val( &self) -> Dummy 
        {
            Self
        }
    }
    impl crate::stalks::bud::BudOp for Dummy 
    {
        fn	IsOpAllowed( _op: crate::stalks::bud::BudBinOp) -> bool 
        {
            false
        }
    }
    let  	left = Box::new( Dummy) as Box< dyn crate::stalks::bud::Bud< Dummy>>;
    let  	right = Box::new( Dummy) as Box< dyn crate::stalks::bud::Bud< Dummy>>;
    let  	_combined = < Dummy as crate::stalks::bud::BudOp>::seq( left, right);
}

//---------------------------------------------------------------------------------------------------------------------------------
