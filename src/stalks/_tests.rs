//-- _tests.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::{ segue::Charset, silo::{ Arr, U8 }, stalks::WorkPtr };
use	crate::silo::{ Buff, IAccess, U32 };
use	crate::stalks::{ Atm, INode, DynINode, Attrib, TraversalEvent as NodeTraversalEvent, BiNodeTree, ChildOp };
use	crate::segue::shard::Shard;
use	std::sync::Arc;
use	std::sync::atomic::{ AtomicBool, Ordering };

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
    let  	prevVal = atmVar.Add( 8);
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
    atmVar1.Add( 1);
    assert_eq!( atmVar1.Get(), 1);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestINodeTraverse()
{
    struct TestNode< 'a>
    {
        id: u32,
        children: &'a [&'a DynINode< 'a>],
        attrib: Option< Attrib>,
    }
    unsafe impl< 'a> Send for TestNode< 'a>
    { }
    unsafe impl< 'a> Sync for TestNode< 'a>
    { }

    impl< 'a> INode< 'a> for TestNode< 'a>
    {
        fn	Attrib( &self) -> Option< &Attrib>
        {
            self.attrib.as_ref()
        }
        fn	_Size( &self) -> U32
        {
            U32( self.children.len() as u32)
        }
        fn	_At( &self, idx: U32) -> &DynINode< 'a>
        {
            self.children[idx.0 as usize]
        }
        fn	Value( &self) -> Option< WorkPtr< 'a>>
        {
            None
        }
        fn	DocStr( &self) -> &'static str
        {
            ""
        }
        fn	ChildOp( &self) -> ChildOp
        {
            ChildOp::None
        }
    }

    let  	leaf1 = TestNode { id: 1, children: &[], attrib: Some( Attrib::default()) };
    let  	leaf2 = TestNode { id: 2, children: &[], attrib: None };
    let  	root = TestNode {
        id: 0,
        children: &[ &leaf1 as &DynINode< '_>, &leaf2 as &DynINode< '_>],
        attrib: None,
    };

    assert!( matches!( leaf1.Attrib(), Some( Attrib::Empty)));
    assert!( leaf2.Attrib().is_none());

    let  	mut visited = Buff::NewEmpty();
    root.TraverseDF( &mut |node, event| {
        let  	test_node = unsafe { &*( node as *const DynINode< '_> as *const TestNode< '_>) };
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
    let  	mut visited2 = Buff::NewEmpty();
    ( &root as &DynINode< '_>).DiveDf( &mut |probe, enterFlg| {
        if enterFlg {
            return;
        }
        let  	mut pathStr = String::new();
        let  	arr = probe.Arr();
        for i in 0..arr.Size().0 {
            let  	node = *arr.At( i);
            let  	testNode = unsafe { &*( node as *const DynINode< '_> as *const TestNode< '_>) };
            if !pathStr.is_empty() {
                pathStr.push_str( " -> ");
            }
            pathStr.push_str( &format!( "{}", testNode.id));
        }
        println!( "{}", pathStr);
        visited2.Push( pathStr);
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
    let  	root = BiNodeTree!( U32, 10 < ( 20 | 30 ));

    assert_eq!( root.ChildOp(), ChildOp::Less);

    assert_eq!( root.Children().Size(), U32(2));

    let  	left = root.Children().At(U32(0));
    let  	right = root.Children().At(U32(1));

    assert_eq!( left.ChildOp(), ChildOp::None);
    assert_eq!( right.ChildOp(), ChildOp::Bor);

    let  	_left1: &DynINode< '_> = root.Children().At(U32(0));

    assert_eq!( right.Children().Size(), U32(2));
    assert_eq!( right.Children().At(U32(0)).ChildOp(), ChildOp::None);
    assert_eq!( right.Children().At(U32(1)).ChildOp(), ChildOp::None);
}

//---------------------------------------------------------------------------------------------------------------------------------



//---------------------------------------------------------------------------------------------------------------------------------
