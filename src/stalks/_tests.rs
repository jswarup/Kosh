//-- _tests.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, IArr, U32 };
use	crate::stalks::{ Atm, BudTree};
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
    use	crate::stalks::bud::{ Bud, TraversalEvent, BudBinOp };
    let  	rootFromLiterals = BudTree!( U32, 10 < ( 20 | 30));
    assert_eq!( rootFromLiterals.CountLeaves(), 3);
    println!( "Tree structure: {:#?}", rootFromLiterals);
    let  	root: &dyn Bud< U32> = &rootFromLiterals;
    assert_eq!( root.Val(), None);
    assert_eq!( root.BinOp(), Some( BudBinOp::LT));
    assert_eq!( root.UniOp(), None);
    assert!( root.Left().is_some());
    assert_eq!( root.Left().unwrap().Val(), Some( &U32( 10)));
    assert_eq!( root.Left().unwrap().BinOp(), None);
    assert!( root.Right().is_some());
    assert_eq!( root.Right().unwrap().BinOp(), Some( BudBinOp::BOR));
    assert_eq!( root.Right().unwrap().Left().unwrap().Val(), Some( &U32( 20)));
    assert_eq!( root.Right().unwrap().Right().unwrap().Val(), Some( &U32( 30)));
    let  	mut visited = Buff::NewEmpty();
    root.TraverseDF( &mut |node, event| {
        let  	node_repr = if let  	Some( val) = node.Val() {
            format!( "{}", val.0)
        } else if let  	Some( op) = node.BinOp() {
            op.as_str().to_string()
        } else if let  	Some( op) = node.UniOp() {
            op.as_str().to_string()
        } else {
            "".to_string()
        };
        visited.Push( ( node_repr, event));
    });
    assert_eq!(
        &visited[..],
        &[
            ( "<".to_string(), TraversalEvent::Entry),
            ( "10".to_string(), TraversalEvent::Entry),
            ( "10".to_string(), TraversalEvent::Exit),
            ( "|".to_string(), TraversalEvent::Entry),
            ( "20".to_string(), TraversalEvent::Entry),
            ( "20".to_string(), TraversalEvent::Exit),
            ( "30".to_string(), TraversalEvent::Entry),
            ( "30".to_string(), TraversalEvent::Exit),
            ( "|".to_string(), TraversalEvent::Exit),
            ( "<".to_string(), TraversalEvent::Exit),
        ]
    );
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBudUnary()
{
    use	crate::stalks::bud::{ Bud, BudUniOp, BudBinOp };
    let  	rootFromLiterals = BudTree!( U32, ! ( 10 | 20 ));
    assert_eq!( rootFromLiterals.CountLeaves(), 2);

    let  	root: &dyn Bud< U32> = &rootFromLiterals;
    assert_eq!( root.Val(), None);
    assert_eq!( root.BinOp(), None);
    assert_eq!( root.UniOp(), Some( BudUniOp::BANG));

    assert!( root.Left().is_some());
    let  	child = root.Left().unwrap();
    assert_eq!( child.BinOp(), Some( BudBinOp::BOR));
    assert_eq!( child.Left().unwrap().Val(), Some( &U32( 10)));
    assert_eq!( child.Right().unwrap().Val(), Some( &U32( 20)));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBudMut()
{
    use	crate::stalks::bud::Bud;
    let  	mut root = BudTree!( U32, 10 < ( 20 | 30 ));

    // Mutate left child leaf value
    {
        let  	root_mut: &mut dyn Bud< U32> = &mut root;
        let  	left_mut = root_mut.LeftMut().unwrap();
        let  	val_ref = left_mut.ValMut().unwrap();
        assert_eq!( val_ref, &mut U32( 10));
        *val_ref = U32( 99);
    }
    assert_eq!( root.Left().unwrap().Val(), Some( &U32( 99)));
    // Mutate nested right child leaf value
    {
        let  	root_mut: &mut dyn Bud< U32> = &mut root;
        let  	right_mut = root_mut.RightMut().unwrap();
        let  	nested_right_mut = right_mut.RightMut().unwrap();
        let  	nested_val_ref = nested_right_mut.ValMut().unwrap();
        assert_eq!( nested_val_ref, &mut U32( 30));
        *nested_val_ref = U32( 100);
    }
    assert_eq!( root.Right().unwrap().Right().unwrap().Val(), Some( &U32( 100)));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBudDiveDf()
{
    use	crate::stalks::bud::Bud;
    let  	rootFromLiterals = BudTree!( U32, 10 < ( 20 | 30));
    let  	root: &dyn Bud< U32> = &rootFromLiterals;
    let  	mut visited = Buff::NewEmpty();
    root.DiveDf( &mut |probe| {
        let mut path_str = String::new();
        let arr = probe.Arr();
        for i in 0..arr.Size().0 {
            let node = *arr.At(i);
            let repr = if let Some(val) = node.Val() {
                format!("{}", val.0)
            } else if let Some(op) = node.BinOp() {
                op.as_str().to_string()
            } else if let Some(op) = node.UniOp() {
                op.as_str().to_string()
            } else {
                "".to_string()
            };
            if !path_str.is_empty() {
                path_str.push_str(" -> ");
            }
            path_str.push_str(&repr);
        }
        println!( "{}", path_str);
        visited.Push(path_str);
    });
    assert_eq!(
        &visited[..],
        &[
            "< -> 10".to_string(),
            "< -> | -> 20".to_string(),
            "< -> | -> 30".to_string(),
            "< -> |".to_string(),
            "<".to_string(),
        ]
    );
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestINodeTraverse()
{
    use	crate::stalks::{ INode, Attrib, TraversalEvent as NodeTraversalEvent };
    use	crate::silo::Arr;

    struct TestNode<'a> {
        id: u32,
        children: &'a [&'a dyn INode],
        attrib: Option<Attrib>,
    }
    impl<'a> INode for TestNode<'a> {
        fn  Attrib(&self) -> Option<&Attrib> {
            self.attrib.as_ref()
        }
        fn	Children<'b>( &'b self) -> Arr< 'b, &'b dyn INode> {
            Arr::from( self.children)
        }
    }

    let  	leaf1 = TestNode { id: 1, children: &[], attrib: Some(Attrib::default()) };
    let  	leaf2 = TestNode { id: 2, children: &[], attrib: None };
    let  	root = TestNode {
        id: 0,
        children: &[ &leaf1, &leaf2],
        attrib: None,
    };

    assert!(matches!(leaf1.Attrib(), Some(Attrib::Empty)));
    assert!(leaf2.Attrib().is_none());

    let  	mut visited = Buff::NewEmpty();
    root.TraverseDF( &mut |node, event| {
        let  	test_node = unsafe { &*( node as *const dyn INode as *const TestNode< '_>) };
        visited.Push( ( test_node.id, event));
    });

    assert_eq!(
        &visited[..],
        &[
            ( 0, NodeTraversalEvent::Entry( U32( 0))),
            ( 1, NodeTraversalEvent::Entry( U32( 0))),
            ( 1, NodeTraversalEvent::Exit),
            ( 0, NodeTraversalEvent::Entry( U32( 1))),
            ( 2, NodeTraversalEvent::Entry( U32( 0))),
            ( 2, NodeTraversalEvent::Exit),
            ( 0, NodeTraversalEvent::Entry( U32( 2))),
            ( 0, NodeTraversalEvent::Exit),
        ]
    );

    // Test DiveDf
    let mut visited2 = Buff::NewEmpty();
    (&root as &dyn INode).DiveDf(&mut |probe| {
        let mut path_str = String::new();
        let arr = probe.Arr();
        for i in 0..arr.Size().0 {
            let node = *arr.At(i);
            let test_node = unsafe { &*(node as *const dyn INode as *const TestNode<'_>) };
            if !path_str.is_empty() {
                path_str.push_str(" -> ");
            }
            path_str.push_str(&format!("{}", test_node.id));
        }
        visited2.Push(path_str);
    });

    assert_eq!(
        &visited2[..],
        &[
            "0 -> 1".to_string(),
            "0 -> 2".to_string(),
            "0".to_string(),
        ]
    );
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBiNodeTree()
{
    use	crate::stalks::{ BiNodeTree, ChildOp, INode };
 
    let  	root = BiNodeTree!( U32, 10 < ( 20 | 30 ));

    assert_eq!( root.ChildOp(), Some( ChildOp::Less));

    let  	children = root.Children();
    assert_eq!( children.Size().0, 2);

    let  	left = *children.At( 0);
    let  	right = *children.At( 1);

    assert_eq!( left.ChildOp(), None);
    assert_eq!( right.ChildOp(), Some( ChildOp::Bor));

    let  	right_children = right.Children();
    assert_eq!( right_children.Size().0, 2);
    assert_eq!( right_children.At( 0).ChildOp(), None);
    assert_eq!( right_children.At( 1).ChildOp(), None);

    // Test Clone implementation
    let  	cloned_root = root.clone();
    assert_eq!( cloned_root.ChildOp(), Some( ChildOp::Less));
    let  	cloned_children = cloned_root.Children();
    assert_eq!( cloned_children.Size().0, 2);
    assert_eq!( cloned_children.At( 1).ChildOp(), Some( ChildOp::Bor));

}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBiNodeTreeBoxetAction()
{
    use	crate::stalks::{ INode, Attrib };
    use	crate::segue::Shard;
    use	std::sync::atomic::{ AtomicBool, Ordering };
    use	std::sync::Arc;

    macro_rules! ShardBiNodeTree {
        ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $s:literal ) => {
            $crate::stalks::node::IntoBiNode::< Shard, $Node >::IntoBiNode( Shard::NewCharset( crate::segue::Charset::FromBoxet( crate::silo::U8::FromArr( crate::silo::Arr::from( $s.as_bytes() ) ) ) ) )
        };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, | $( $body)+ ) };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, || $( $body)+ ) };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, move | $( $body)+ ) };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, move || $( $body)+ ) };
        ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal [ $( $closure:tt )* ] ) => { $crate::BiNodeTree!( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $l [ $( $closure )* ] ) };
        ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $expr:tt)+ ) [ $( $closure:tt )* ] ) => { $crate::BiNodeTree!( @feature_ACTION [ $( $cb)* ], $Arg, $Node, ( $( $expr )+ ) [ $( $closure )* ] ) };
        ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] [ $( $closure:tt )* ] ) => { $crate::BiNodeTree!( @feature_ACTION [ $( $cb)* ], $Arg, $Node, [ $s ] [ $( $closure )* ] ) };
        ( @ $( $inner:tt )+ ) => {
            $crate::BiNodeTree!( @ $( $inner )+ )
        };
        ( $( $inner:tt)+ ) => {
            $crate::BiNodeTree!( @define [ ShardBiNodeTree ], Shard, $( $inner)+ )
        };
    }

    let  	triggered = Arc::new(AtomicBool::new(false));
    let  	triggered_clone = triggered.clone();
    
    // Construct tree with a boxet leaf and an action suffix
    let  	root = ShardBiNodeTree!( [ "a" ] [ move || {
        triggered_clone.store(true, Ordering::SeqCst);
    } ] );

    // Check that we can get the action attribute
    if let  	Some(Attrib::Action(action)) = root.Attrib() {
        action();
    } else {
        panic!("Action attribute not found on root node!");
    }

    assert!(triggered.load(Ordering::SeqCst));
}

//---------------------------------------------------------------------------------------------------------------------------------
