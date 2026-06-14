//-- node.rs -------------------------------------------------------------------------------------------------------------------
use crate::silo::{Arr, U32};

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

    ( @wrap $leaf:expr ) => {
        Into::into( $leaf )
    };
    ( @define [ $( $cb:tt )* ], $Arg:ident, $( $inner:tt )+ ) => {
        {
            paste::paste! {
                #[derive( Debug )]
                #[allow(dead_code)]
                enum [<$Arg BiNode>] {
                    Leaf( $Arg),
                    Node {
                        _Op: $crate::stalks::ChildOp,
                        _Children: [Box< [<$Arg BiNode>]>; 2],
                        _Refs: [*const dyn $crate::stalks::INode; 2],
                    }
                }
                unsafe impl Send for [<$Arg BiNode>] {}
                unsafe impl Sync for [<$Arg BiNode>] {}

                #[allow(dead_code)]
                impl [<$Arg BiNode>]
                {
                    fn	New( value: $Arg) -> Self
                    {
                        [<$Arg BiNode>]::Leaf( value)
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
                        }
                    }
                }
                impl Clone for [<$Arg BiNode>]
                {
                    fn	clone( &self) -> Self
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf( val) => [<$Arg BiNode>]::Leaf( val.clone()),
                            [<$Arg BiNode>]::Node { _Op, _Children, .. } => {
                                let left = _Children[0].as_ref().clone();
                                let right = _Children[1].as_ref().clone();
                                [<$Arg BiNode>]::NewBranch( *_Op, left, right)
                            }
                        }
                    }
                }
                impl $crate::stalks::INode for [<$Arg BiNode>]
                {
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
                }
                impl $crate::stalks::node::IntoBiNode< $Arg, [<$Arg BiNode>]> for [<$Arg BiNode>]
                {
                    fn	IntoBiNode( self) -> [<$Arg BiNode>]
                    {
                        self
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
}
pub use crate::BiNodeTree;
