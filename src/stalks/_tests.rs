//-- _tests.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, ISlice, U32 };
use	crate::stalks::Atm;
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
fn	TestINodeTraverse()
{
    use	crate::stalks::{ INode, Attrib, TraversalEvent as NodeTraversalEvent };


    struct TestNode<'a> {
        id: u32,
        children: &'a [&'a (dyn INode<'a> + Send + Sync)],
        attrib: Option<Attrib>,
    }
    unsafe impl<'a> Send for TestNode<'a> {}
    unsafe impl<'a> Sync for TestNode<'a> {}

    impl<'a> INode<'a> for TestNode<'a> {
        fn  Attrib(&self) -> Option<&Attrib> {
            self.attrib.as_ref()
        }
        fn Children(&self) -> &[&'a (dyn INode<'a> + Send + Sync)] {
            self.children
        }
    }

    let  	leaf1 = TestNode { id: 1, children: &[], attrib: Some(Attrib::default()) };
    let  	leaf2 = TestNode { id: 2, children: &[], attrib: None };
    let  	root = TestNode {
        id: 0,
        children: &[ &leaf1 as &(dyn INode<'_> + Send + Sync), &leaf2 as &(dyn INode<'_> + Send + Sync)],
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
    (&root as &(dyn INode + Send + Sync)).DiveDf(&mut |probe| {
        let mut path_str = String::new();
        let arr = probe.Arr();
        for i in 0..arr.Size().0 {
            let node = *arr.At(i);
            let test_node = unsafe { &*(node as *const (dyn INode + Send + Sync) as *const TestNode<'_>) };
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
 
    let arena = crate::stalks::node::NodeArena::New();
    let  	root = BiNodeTree!( U32, arena, 10 < ( 20 | 30 ));

    assert_eq!( root.ChildOp(), Some( ChildOp::Less));

    assert_eq!( root.Children().len(), 2);

    let  	left = root.Children()[0];
    let  	right = root.Children()[1];

    assert_eq!( left.ChildOp(), None);
    assert_eq!( right.ChildOp(), Some( ChildOp::Bor));

    assert_eq!( right.Children().len(), 2);
    assert_eq!( right.Children()[0].ChildOp(), None);
    assert_eq!( right.Children()[1].ChildOp(), None);
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
        ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $s:literal ) => {
            $crate::stalks::node::IntoBiNode::< Shard, $Node >::IntoBiNode( Shard::NewCharset( crate::segue::Charset::FromBoxet( crate::silo::U8::FromArr( crate::silo::Arr::from( $s.as_bytes() ) ) ) ) )
        };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, | $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, | $( $body)+ ) };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, || $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, || $( $body)+ ) };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, move | $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, move | $( $body)+ ) };
        ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, move || $( $body:tt)+ ) => { $crate::BiNodeTree!( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, move || $( $body)+ ) };
        ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:literal [ $( $closure:tt )* ] ) => { $crate::BiNodeTree!( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $arena, $l [ $( $closure )* ] ) };
        ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $expr:tt)+ ) [ $( $closure:tt )* ] ) => { $crate::BiNodeTree!( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $arena, ( $( $expr )+ ) [ $( $closure )* ] ) };
        ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] [ $( $closure:tt )* ] ) => { $crate::BiNodeTree!( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $arena, [ $s ] [ $( $closure )* ] ) };
        ( @ $( $inner:tt )+ ) => {
            $crate::BiNodeTree!( @ $( $inner )+ )
        };
        ( $arena:ident, $( $inner:tt)+ ) => {
            $crate::BiNodeTree!( @define [ ShardBiNodeTree ], Shard, $arena, $( $inner)+ )
        };
    }

    let  	triggered = Arc::new(AtomicBool::new(false));
    let  	triggered_clone = triggered.clone();
    
    // Construct tree with a boxet leaf and an action suffix
    let _arena = crate::stalks::node::NodeArena::New();
    let  	root = ShardBiNodeTree!( arena, [ "a" ] [ move || {
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
