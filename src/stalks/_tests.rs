//-- _tests.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::silo::U32;
use	crate::stalks::{ Atm, BudTree, Bud };
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
    let  	x = BudTree!( f64, ( ( ( a | b) < c) | ( d | ( e < f))));
    x.Print();
    let  	combined = BudTree!( f64, ( a | b) | ( c | d));
    combined.Print();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBud()
{
    use crate::stalks::bud::{Bud, TraversalEvent, BudBinOp};
    let  	rootFromLiterals = BudTree!( U32, 10 < ( 20 | 30));
    assert_eq!( rootFromLiterals.CountLeaves(), 3);
    println!( "Tree structure: {:#?}", rootFromLiterals);

    let root: &dyn Bud<U32> = &rootFromLiterals;
    assert_eq!( root.Val(), None);
    assert_eq!( root.BinOp(), Some(BudBinOp::LT));
    assert_eq!( root.UniOp(), None);
    assert!( root.Left().is_some());
    assert_eq!( root.Left().unwrap().Val(), Some(&U32(10)));
    assert_eq!( root.Left().unwrap().BinOp(), None);
    assert!( root.Right().is_some());
    assert_eq!( root.Right().unwrap().BinOp(), Some(BudBinOp::BOR));
    assert_eq!( root.Right().unwrap().Left().unwrap().Val(), Some(&U32(20)));
    assert_eq!( root.Right().unwrap().Right().unwrap().Val(), Some(&U32(30)));

    let mut visited = Vec::new();
    root.TraverseDFS(&mut |node, event| {
        let node_repr = if let Some(val) = node.Val() {
            format!("{}", val.0)
        } else if let Some(op) = node.BinOp() {
            op.as_str().to_string()
        } else if let Some(op) = node.UniOp() {
            op.as_str().to_string()
        } else {
            "".to_string()
        };
        visited.push((node_repr, event));
    });

    assert_eq!(
        visited,
        vec![
            ("<".to_string(), TraversalEvent::Entry),
            ("10".to_string(), TraversalEvent::Entry),
            ("|".to_string(), TraversalEvent::Entry),
            ("20".to_string(), TraversalEvent::Entry),
            ("30".to_string(), TraversalEvent::Entry),
            ("|".to_string(), TraversalEvent::Exit),
            ("<".to_string(), TraversalEvent::Exit),
        ]
    );
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBudUnary()
{
    use crate::stalks::bud::{Bud, BudUniOp, BudBinOp};
    let  	rootFromLiterals = BudTree!( U32, ! ( 10 | 20 ));
    assert_eq!( rootFromLiterals.CountLeaves(), 2);
    
    let root: &dyn Bud<U32> = &rootFromLiterals;
    assert_eq!( root.Val(), None);
    assert_eq!( root.BinOp(), None);
    assert_eq!( root.UniOp(), Some(BudUniOp::BANG));
    
    assert!( root.Left().is_some());
    let child = root.Left().unwrap();
    assert_eq!( child.BinOp(), Some(BudBinOp::BOR));
    assert_eq!( child.Left().unwrap().Val(), Some(&U32(10)));
    assert_eq!( child.Right().unwrap().Val(), Some(&U32(20)));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBudMut()
{
    use crate::stalks::bud::Bud;
    let mut root = BudTree!( U32, 10 < ( 20 | 30 ));
    
    // Mutate left child leaf value
    {
        let root_mut: &mut dyn Bud<U32> = &mut root;
        let left_mut = root_mut.LeftMut().unwrap();
        let val_ref = left_mut.ValMut().unwrap();
        assert_eq!( val_ref, &mut U32(10));
        *val_ref = U32(99);
    }
    assert_eq!( root.Left().unwrap().Val(), Some( &U32(99)));

    // Mutate nested right child leaf value
    {
        let root_mut: &mut dyn Bud<U32> = &mut root;
        let right_mut = root_mut.RightMut().unwrap();
        let nested_right_mut = right_mut.RightMut().unwrap();
        let nested_val_ref = nested_right_mut.ValMut().unwrap();
        assert_eq!( nested_val_ref, &mut U32(30));
        *nested_val_ref = U32(100);
    }
    assert_eq!( root.Right().unwrap().Right().unwrap().Val(), Some( &U32(100)));
}

//---------------------------------------------------------------------------------------------------------------------------------
