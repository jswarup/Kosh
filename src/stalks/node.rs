//-- node.rs -------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, U32 };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub enum Attrib {
    Inv(bool),
    Repeat(U32, U32),
    Action(Box<dyn Fn()>),
    #[default]
    Empty,
}

impl std::fmt::Debug for Attrib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attrib::Inv(value) => f.debug_tuple("Inv").field(value).finish(),
            Attrib::Repeat(left, right) => f.debug_tuple("Repeat").field(left).field(right).finish(),
            Attrib::Action(_) => f.write_str("Action(<closure>)"),
            Attrib::Empty => f.write_str("Empty"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildOp {
    Sum,
    Prod,
    Less,
    Bor,
    Shl,
    Shr,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalEvent {
    Entry(U32),
    Exit,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INode {
    fn Attrib(&self) -> Option<&Attrib> {
        None
    }

    fn ChildOp(&self) -> Option<ChildOp> {
        None
    }

    fn Children<'a>(&'a self) -> Arr<'a, &'a dyn INode>;

    fn IsLeaf(&self) -> bool {
        self.Children().Size() == U32(0)
    }

    fn TraverseDF(&self, fnMut: &mut dyn FnMut(&dyn INode, TraversalEvent))
    where
        Self: Sized,
    {
        traverse_df(self, fnMut);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> dyn INode + 'a {
    pub fn TraverseDF(&self, fnMut: &mut dyn FnMut(&dyn INode, TraversalEvent)) {
        traverse_df(self, fnMut);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn traverse_df(node: &dyn INode, fnMut: &mut dyn FnMut(&dyn INode, TraversalEvent)) {
    let mut stash = crate::silo::Stash::New(1024, 1, (node, U32(0)));
    while stash.Size() > U32(0) {
        let mut curr = (node, U32(0));
        let _res = stash.Pop(&mut curr);
        let (n, idx) = curr;
        let children = n.Children();
        let sz = children.Size();
        if idx < sz {
            fnMut(n, TraversalEvent::Entry(idx));
            stash.Push((n, idx + U32(1)));
            let child = *children.At(idx);
            stash.Push((child, U32(0)));
        } else {
            fnMut(n, TraversalEvent::Entry(sz));
            fnMut(n, TraversalEvent::Exit);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct NodeProbe<'a> {
    _NodeStash: crate::silo::Stash<&'a dyn INode>,
}

impl<'a> NodeProbe<'a> {
    pub fn New< Sz: Into<U32>>( sz: Sz, node: &'a dyn INode) -> Self {
        Self {
            _NodeStash: crate::silo::Stash::Create( sz, U32(0), |_| node),
        }
    }

    pub fn Push(&self, node: &'a dyn INode) {
        let mut temp = node;
        self._NodeStash.Stk().Push(&mut temp);
    }

    pub fn Pop(&self, node: &'a dyn INode) {
        let mut temp = node;
        self._NodeStash.Stk().Pop(&mut temp);
    }

    pub fn Arr(&self) -> Arr<'_, &'a dyn INode> {
        self._NodeStash.Stk().Arr()
    }
}

impl<'a> dyn INode + 'a {
    pub fn DiveDf(&self, fnMut: &mut dyn FnMut(&NodeProbe<'_,>)) {
        let nodeProbe = NodeProbe::New(1024, self);
        traverse_df(self, &mut |node, event| match event {
            TraversalEvent::Entry(idx) => {
                if idx == U32(0) {
                    nodeProbe.Push(node);
                }
            }
            TraversalEvent::Exit => {
                fnMut(&nodeProbe);
                nodeProbe.Pop(node);
            }
        });
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IntoBiNode< T, N: Sized > {
    fn	IntoBiNode( self ) -> N;
    fn	IntoBiNodeAction< F >( self, _f: F ) -> N
    where
        Self: Sized,
        F: Fn() + 'static,
    {
        self.IntoBiNode()
    }
}

pub fn clone_attrib(attr: &Option<Attrib>) -> Option<Attrib> {
    match attr {
        None => None,
        Some(Attrib::Inv(val)) => Some(Attrib::Inv(*val)),
        Some(Attrib::Repeat(l, r)) => Some(Attrib::Repeat(*l, *r)),
        Some(Attrib::Empty) => Some(Attrib::Empty),
        Some(Attrib::Action(_)) => {
            panic!("Cannot clone an INode with an Action attribute");
        }
    }
}

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
    ( @define [ $( $cb:tt )* ], $Arg:ident, $( $inner:tt )+ ) => {
        {
            paste::paste! {
                #[derive( Debug )]
                #[allow(dead_code)]
                enum [<$Arg BiNode>] {
                    Leaf {
                        _Val: $Arg,
                        _Attrib: Option< $crate::stalks::Attrib >,
                    },
                    Node {
                        _Op: $crate::stalks::ChildOp,
                        _Children: [Box< [<$Arg BiNode>]>; 2],
                        _Refs: [*const dyn $crate::stalks::INode; 2],
                        _Attrib: Option< $crate::stalks::Attrib >,
                    }
                }
                unsafe impl Send for [<$Arg BiNode>] {}
                unsafe impl Sync for [<$Arg BiNode>] {}

                #[allow(dead_code)]
                impl [<$Arg BiNode>]
                {
                    fn	New( value: $Arg) -> Self
                    {
                        [<$Arg BiNode>]::Leaf {
                            _Val: value,
                            _Attrib: None,
                        }
                    }
                    fn	NewBranch( op: $crate::stalks::ChildOp, left: Self, right: Self) -> Self
                    {
                        let left_box = Box::new( left);
                        let right_box = Box::new( right);
                        let left_ptr = &*left_box as *const dyn $crate::stalks::INode;
                        let right_ptr = &*right_box as *const dyn $crate::stalks::INode;
                        [<$Arg BiNode>]::Node {
                            _Op: op,
                            _Children: [left_box, right_box],
                            _Refs: [left_ptr, right_ptr],
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
                impl Clone for [<$Arg BiNode>]
                {
                    fn	clone( &self) -> Self
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { _Val, _Attrib } => [<$Arg BiNode>]::Leaf {
                                _Val: _Val.clone(),
                                _Attrib: $crate::stalks::node::clone_attrib( _Attrib ),
                            },
                            [<$Arg BiNode>]::Node { _Op, _Children, _Attrib, .. } => {
                                let left = _Children[0].as_ref().clone();
                                let right = _Children[1].as_ref().clone();
                                let mut node = [<$Arg BiNode>]::NewBranch( *_Op, left, right);
                                node.SetAttrib( $crate::stalks::node::clone_attrib( _Attrib ) );
                                node
                            }
                        }
                    }
                }
                impl $crate::stalks::INode for [<$Arg BiNode>]
                {
                    fn	Attrib( &self) -> Option<& $crate::stalks::Attrib>
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { _Attrib, .. } => _Attrib.as_ref(),
                            [<$Arg BiNode>]::Node { _Attrib, .. } => _Attrib.as_ref(),
                        }
                    }
                    fn	ChildOp( &self) -> Option<$crate::stalks::ChildOp>
                    {
                        match self {
                            [<$Arg BiNode>]::Node { _Op, .. } => Some( *_Op),
                            _ => None,
                        }
                    }
                    fn	Children<'a>( &'a self) -> $crate::silo::Arr< 'a, &'a dyn $crate::stalks::INode>
                    {
                        match self {
                            [<$Arg BiNode>]::Node { _Refs, .. } => {
                                unsafe {
                                    let slice = std::slice::from_raw_parts(
                                        _Refs.as_ptr() as *const &'a dyn $crate::stalks::INode,
                                        2
                                    );
                                    $crate::silo::Arr::from(slice)
                                }
                            }
                            _ => $crate::silo::Arr::from(&[][..]),
                        }
                    }
                }
                impl std::ops::Deref for [<$Arg BiNode>]
                {
                    type Target = dyn $crate::stalks::INode;
                    fn	deref( &self) -> &Self::Target
                    {
                        self
                    }
                }
                impl< I > $crate::stalks::node::IntoBiNode< $Arg, [<$Arg BiNode>]> for I
                where
                    I: Into< $Arg >,
                {
                    fn	IntoBiNode( self) -> [<$Arg BiNode>]
                    {
                        [<$Arg BiNode>]::New( self.into() )
                    }
                    fn	IntoBiNodeAction< F >( self, f: F) -> [<$Arg BiNode>]
                    where
                        F: Fn() + 'static,
                    {
                        [<$Arg BiNode>]::New( self.into() ).WithAttrib(Some($crate::stalks::Attrib::Action(Box::new(f))))
                    }
                }
                impl $crate::stalks::node::IntoBiNode< $Arg, [<$Arg BiNode>]> for [<$Arg BiNode>]
                {
                    fn	IntoBiNode( self) -> [<$Arg BiNode>]
                    {
                        self
                    }
                    fn	IntoBiNodeAction< F >( self, f: F) -> [<$Arg BiNode>]
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
