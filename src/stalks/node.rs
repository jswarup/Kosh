//-- node.rs -------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, IAccess, Stash, U32 };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub enum Attrib
{
    Inv( bool),
    Repeat( U32, U32),
    Action( Box< dyn Fn()>),
    #[default]
    Empty,
}

impl std::fmt::Debug for Attrib
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        match self {
            Attrib::Inv( value) => f.debug_tuple( "Inv").field( value).finish(),
            Attrib::Repeat( left, right) => f.debug_tuple( "Repeat").field( left).field( right).finish(),
            Attrib::Action( _) => f.write_str( "Action(<closure>)"),
            Attrib::Empty => f.write_str( "Empty"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildOp
{
    Sum,
    Prod,
    Less,
    Bor,
    Shl,
    Shr,
    None
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalEvent
{
    Entry( U32),
    Exit,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type DynINode< 'a> = dyn INode< 'a> + Send + Sync + 'a;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct NodeChildren< 'b, 'a>( pub &'b DynINode< 'a>);

impl< 'b, 'a> IAccess< 'b, DynINode< 'a>> for NodeChildren< 'b, 'a>
{
    fn	Size( &self) -> U32
    {
        self.0._Size()
    }
    fn	At< K: Into< U32>>( &self, k: K) -> &'b DynINode< 'a>
    {
        self.0._At( k.into())
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

pub trait AsWorkPtr< 'a>
{
    fn	AsWorkPtr( &self) -> Option< crate::stalks::WorkPtr< 'a>>;
}

impl< 'a> AsWorkPtr< 'a> for crate::silo::U32
{
    fn	AsWorkPtr( &self) -> Option< crate::stalks::WorkPtr< 'a>>
    {
        None
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INode< 'a>: Send + Sync
{
    fn	_Size( &self) -> U32;
    fn	_At( &self, idx: U32) -> &DynINode< 'a>;

    fn	Value( &self) -> Option< crate::stalks::WorkPtr< 'a>>
    {
        None
    }

    fn	Attrib( &self) -> Option< &Attrib>
    {
        None
    }

    fn	ChildOp( &self) -> ChildOp
    {
        ChildOp::None
    }

    fn	IsLeaf( &self) -> bool
    {
        self._Size() == U32( 0)
    }

    fn	TraverseDF( &'a self, fnMut: &mut dyn FnMut( &'a DynINode< 'a>, TraversalEvent))
    where
        Self: Sized,
    {
        TraverseDepthFirst( self, fnMut);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> DynINode< 'a>
{
    pub fn	Children< 'b>( &'b self) -> NodeChildren< 'b, 'a>
    {
        NodeChildren( self)
    }

    pub fn	TraverseDF< 'b>( &'b self, fnMut: &mut dyn FnMut( &'b DynINode< 'a>, TraversalEvent))
    {
        TraverseDepthFirst( self, fnMut);
    }

    pub fn	DiveDf< 'b>( &'b self, fnMut: &mut dyn FnMut( &NodeProbe< 'b, 'a>))
    {
        let  	nodeProbe = NodeProbe::New( 1024, self);
        TraverseDepthFirst( self, &mut |node, event| match event {
            TraversalEvent::Entry( idx) => {
                if idx == U32( 0) {
                    nodeProbe.Push( node);
                }
            }
            TraversalEvent::Exit => {
                fnMut( &nodeProbe);
                nodeProbe.Pop( node);
            }
        });
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn	TraverseDepthFirst< 'b, 'a>( node: &'b DynINode< 'a>, fnMut: &mut dyn FnMut( &'b DynINode< 'a>, TraversalEvent))
{
    let  	mut stash = Stash::New( 1024, 1, ( node, U32( 0)));
    while stash.Size() > U32( 0) {
        let  	mut curr = ( node, U32( 0));
        let  	_res = stash.Pop( &mut curr);
        let  	( currNode, idx) = curr;
        let  	numChildren = currNode.Children().Size();
        if idx < numChildren {
            fnMut( currNode, TraversalEvent::Entry( idx));
            stash.Push( ( currNode, idx + U32( 1)));
            let  	child = currNode.Children().At( idx);
            stash.Push( ( child, U32( 0)));
        } else {
            fnMut( currNode, TraversalEvent::Entry( numChildren));
            fnMut( currNode, TraversalEvent::Exit);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct NodeProbe< 'b, 'a>
{
    _NodeStash: Stash< &'b DynINode< 'a>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'b, 'a> NodeProbe< 'b, 'a>
{
    pub fn	New< Sz: Into< U32>>( sz: Sz, node: &'b DynINode< 'a>) -> Self
    {
        Self {
            _NodeStash: Stash::Create( sz, U32( 0), |_| node),
        }
    }

    pub fn	Push( &self, node: &'b DynINode< 'a>)
    {
        let  	mut temp = node;
        self._NodeStash.Stk().Push( &mut temp);
    }

    pub fn	Pop( &self, node: &'b DynINode< 'a>)
    {
        let  	mut temp = node;
        self._NodeStash.Stk().Pop( &mut temp);
    }

    pub fn	Arr( &self) -> Arr< '_, &'b DynINode< 'a>>
    {
        self._NodeStash.Stk().Arr()
    }
    pub fn  CurNode( &self) -> Option< &'b DynINode< 'a>>
    {
        let  	sz = self._NodeStash.Size();
        if sz > U32( 0) {
            return Some( *self.Arr().At( sz - U32( 1)));
        }
        None
    } 
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IntoBiNode< T, N: Sized>
{
    fn	IntoBiNode( self) -> N;
    fn	IntoBiNodeAction< F>( self, _f: F) -> N
    where
        Self: Sized,
        F: Fn() + 'static,
    {
        self.IntoBiNode()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn	clone_attrib( attr: &Option< Attrib>) -> Option< Attrib>
{
    match attr {
        None => None,
        Some( Attrib::Inv( val)) => Some( Attrib::Inv( *val)),
        Some( Attrib::Repeat( l, r)) => Some( Attrib::Repeat( *l, *r)),
        Some( Attrib::Empty) => Some( Attrib::Empty),
        Some( Attrib::Action( _)) => {
            panic!( "Cannot clone an INode with an Action attribute");
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BiNodeTree {
    // ---- FEATURE OPT-INS FOR BiNodeTree ITSELF ----------------------------------------------------------------------------
    ( @feature_SHL [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Shl, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Shl, $l, $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Shr, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Shr, $l, $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Less, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Less, $l, $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Bor, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Bor, $l, $( $r)+ ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $Node::New( $Arg::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $Node::New( $Arg::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $Node::New( $Arg::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $Node::New( $Arg::New( move || $( $body)+ ) ) };

    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $l ), $( $closure )* )
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $expr:tt)+ ) [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, ( $( $expr )+ ) ), $( $closure )* )
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ), $( $closure )* )
    };

    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, | $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move | $( $closure)*
        )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, || $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move || $( $closure)*
        )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, move | $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move | $( $closure)*
        )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, move || $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move || $( $closure)*
        )
    };

    ( @wrap $leaf:expr ) => {
        Into::into( $leaf )
    };

    //-----------------------------------------------------------------------------------------------------------------------------

    ( @define [ $( $cb:tt )* ], $Arg:ident, $( $inner:tt )+ ) => {
        {
            paste::paste! {
                #[allow(dead_code)]
                enum [<$Arg BiNode>]<'a> {
                    Leaf {
                        _Val: $Arg,
                        _Attrib: Option< $crate::stalks::Attrib >,
                    },
                    Node {
                        _Op: $crate::stalks::ChildOp,
                        _Children: [Box<$crate::stalks::DynINode<'a>>; 2],
                        _Attrib: Option< $crate::stalks::Attrib >,
                    }
                }
                unsafe impl<'a> Send for [<$Arg BiNode>]<'a> {}
                unsafe impl<'a> Sync for [<$Arg BiNode>]<'a> {}

                #[allow(dead_code)]
                impl<'a> [<$Arg BiNode>]<'a>
                {
                    fn	New( value: $Arg) -> Self
                    {
                        [<$Arg BiNode>]::Leaf {
                            _Val: value,
                            _Attrib: None,
                        }
                    }
                    fn  NewBranch( op: $crate::stalks::ChildOp, left: Self, right: Self) -> Self
                    {
                        [<$Arg BiNode>]::Node {
                            _Op: op,
                            _Children: [Box::new(left), Box::new(right)],
                            _Attrib: None,
                        }
                    }
                    fn	SetAttrib( &mut self, attr: Option< $crate::stalks::Attrib >)
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { _Attrib, .. } => {
                                *_Attrib = attr;
                            }
                            [<$Arg BiNode>]::Node { _Attrib, .. } => {
                                *_Attrib = attr;
                            }
                        }
                    }
                    fn	WithAttrib( mut self, attr: Option< $crate::stalks::Attrib >) -> Self
                    {
                        self.SetAttrib( attr);
                        self
                    }
                }

                impl<'a> [<$Arg BiNode>]<'a> {
                    #[allow(dead_code)]
                    pub fn Children<'b>(&'b self) -> $crate::stalks::node::NodeChildren<'b, 'a> {
                        $crate::stalks::node::NodeChildren(self)
                    }
                }

                impl<'a> $crate::stalks::INode<'a> for [<$Arg BiNode>]<'a>
                {
                    fn _Size(&self) -> $crate::silo::U32 {
                        match self {
                            [<$Arg BiNode>]::Node { .. } => $crate::silo::U32(2),
                            _ => $crate::silo::U32(0),
                        }
                    }
                    fn _At(&self, idx: $crate::silo::U32) -> &$crate::stalks::DynINode<'a> {
                        match self {
                            [<$Arg BiNode>]::Node { _Children, .. } => &*_Children[idx.0 as usize],
                            _ => panic!("At called on Leaf"),
                        }
                    }

                    fn	Value( &self) -> Option< $crate::stalks::WorkPtr< 'a>>
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { _Val, .. } => {
                                $crate::stalks::node::AsWorkPtr::AsWorkPtr( _Val)
                            }
                            _ => None
                        }
                    }

                    fn	Attrib( &self) -> Option<& $crate::stalks::Attrib>
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { _Attrib, .. } => _Attrib.as_ref(),
                            [<$Arg BiNode>]::Node { _Attrib, .. } => _Attrib.as_ref(),
                        }
                    }
                    fn	ChildOp( &self) -> $crate::stalks::ChildOp
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { .. } => $crate::stalks::ChildOp::None,
                            [<$Arg BiNode>]::Node { _Op, .. } => *_Op,
                        }
                    }
                }
                impl<'a, I > $crate::stalks::node::IntoBiNode< $Arg, [<$Arg BiNode>]<'a>> for I
                where
                    I: Into< $Arg >,
                {
                    fn	IntoBiNode( self) -> [<$Arg BiNode>]<'a>
                    {
                        [<$Arg BiNode>]::New( self.into() )
                    }
                    fn	IntoBiNodeAction< F >( self, f: F) -> [<$Arg BiNode>]<'a>
                    where
                        F: Fn() + 'static,
                    {
                        [<$Arg BiNode>]::New( self.into() ).WithAttrib(Some($crate::stalks::Attrib::Action(Box::new(f))))
                    }
                }
                impl<'a> $crate::stalks::node::IntoBiNode< $Arg, [<$Arg BiNode>]<'a>> for [<$Arg BiNode>]<'a>
                {
                    fn	IntoBiNode( self) -> [<$Arg BiNode>]<'a>
                    {
                        self
                    }
                    fn	IntoBiNodeAction< F >( self, f: F) -> [<$Arg BiNode>]<'a>
                    where
                        F: Fn() + 'static,
                    {
                        self.WithAttrib(Some($crate::stalks::Attrib::Action(Box::new(f))))
                    }
                }
                $crate::BiNodeTree!( @cb [ $( $cb)* ], $Arg, [<$Arg BiNode>], $( $inner )+ )
            }
        }
    };
    
    //-----------------------------------------------------------------------------------------------------------------------------

    ( $Arg:ident, $( $inner:tt )+ ) => {
        $crate::BiNodeTree!( @define [ $crate::BiNodeTree ], $Arg, $( $inner )+ )
    };
    
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $inner)+ ) };

    // ── Leaf [ action ] ────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $l [ $( $inner )* ] )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $expr:tt)+ ) [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, ( $( $expr )+ ) [ $( $inner )* ] )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, [ $s ] [ $( $inner )* ] )
    };

    // ── Binary: [ boxet ] OP rhs ────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };

    // ── Leaf Boxet ──────────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] ) => {
        $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s )
    };

    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };

    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };

    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, || $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move || $( $body)+ ) };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $leaf:expr ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNode( $leaf )
    };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------
    // @bg : binary — (group) OP rhs
    ( @bg [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        $Node::NewBranch(
            $crate::stalks::ChildOp::$op,
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };
    // @bl : binary — leaf OP rhs
    ( @bl [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        $Node::NewBranch(
            $crate::stalks::ChildOp::$op,
            $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNode( $l ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_NEW $( $args:tt )* ) => { compile_error!( "Closure literal is not enabled for this tree") };
    ( @feature_BOXET  $( $args:tt )* ) => { compile_error!( "Boxet [ ... ] is not enabled for this tree") };
    ( @feature_ACTION $( $args:tt )* ) => { compile_error!( "Action suffix [ closure ] is not enabled for this tree") };
}

pub use crate::BiNodeTree;

//-----------------------------------------------------------------------------------------------------------------------------
