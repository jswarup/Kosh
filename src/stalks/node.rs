//-- node.rs -------------------------------------------------------------------------------------------------------------------
use	crate::silo::{Arr, U32};

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

pub trait INode<'a>: Send + Sync {
    fn Attrib(&self) -> Option<&Attrib> {
        None
    }

    fn ChildOp(&self) -> Option<ChildOp> {
        None
    }

    fn Children(&self) -> &[&'a (dyn INode<'a> + Send + Sync)];

    fn IsLeaf(&self) -> bool {
        self.Children().is_empty()
    }

    fn TraverseDF(&'a self, fnMut: &mut dyn FnMut(&'a (dyn INode<'a> + Send + Sync), TraversalEvent))
    where
        Self: Sized,
    {
        traverse_df(self, fnMut);
    }
}

pub struct NodeArena<'a> {
    _Buff: std::cell::UnsafeCell<crate::silo::Buff<Box<dyn INode<'a> + Send + Sync + 'a>>>,
}

impl<'a> NodeArena<'a> {
    pub fn New() -> Self {
        Self { _Buff: std::cell::UnsafeCell::new(crate::silo::Buff::NewEmpty()) }
    }
    pub fn Alloc<T: INode<'a> + Send + Sync + 'a>(&self, node: T) -> &'a (dyn INode<'a> + Send + Sync) {
        let b = Box::new(node);
        let ptr = unsafe { &*(&*b as *const (dyn INode<'a> + Send + Sync)) };
        unsafe {
            (&mut *self._Buff.get()).Push(b as Box<dyn INode<'a> + Send + Sync + 'a>);
        }
        ptr
    }
}

unsafe impl<'a> Send for NodeArena<'a> {}
unsafe impl<'a> Sync for NodeArena<'a> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> dyn INode<'a> + Send + Sync + 'a {
    pub fn TraverseDF(&'a self, fnMut: &mut dyn FnMut(&'a (dyn INode<'a> + Send + Sync), TraversalEvent)) {
        traverse_df(self, fnMut);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn traverse_df<'a>(node: &'a (dyn INode<'a> + Send + Sync), fnMut: &mut dyn FnMut(&'a (dyn INode<'a> + Send + Sync), TraversalEvent)) {
    use crate::silo::U32;
    let mut stash = crate::silo::Stash::New(1024, 1, (node, 0usize));
    while stash.Size() > U32(0) {
        let mut curr = (node, 0usize);
        let _res = stash.Pop(&mut curr);
        let (n, idx) = curr;
        let children = n.Children();
        let num_children = children.len();
        if idx < num_children {
            fnMut(n, TraversalEvent::Entry(crate::silo::U32(idx as u32)));
            stash.Push((n, idx + 1));
            let child = children[idx];
            stash.Push((child, 0usize));
        } else {
            fnMut(n, TraversalEvent::Entry(crate::silo::U32(num_children as u32)));
            fnMut(n, TraversalEvent::Exit);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct NodeProbe<'a> {
    _NodeStash: crate::silo::Stash<&'a (dyn INode<'a> + Send + Sync)>,
}

impl<'a> NodeProbe<'a> {
    pub fn New< Sz: Into<U32>>( sz: Sz, node: &'a (dyn INode<'a> + Send + Sync)) -> Self {
        Self {
            _NodeStash: crate::silo::Stash::Create( sz, U32(0), |_| node),
        }
    }

    pub fn Push(&self, node: &'a (dyn INode<'a> + Send + Sync)) {
        let mut temp = node;
        self._NodeStash.Stk().Push(&mut temp);
    }

    pub fn Pop(&self, node: &'a (dyn INode<'a> + Send + Sync)) {
        let mut temp = node;
        self._NodeStash.Stk().Pop(&mut temp);
    }

    pub fn Arr(&self) -> Arr<'_, &'a (dyn INode<'a> + Send + Sync)> {
        self._NodeStash.Stk().Arr()
    }
}

impl<'a> dyn INode<'a> + Send + Sync + 'a {
    pub fn DiveDf(&'a self, fnMut: &mut dyn FnMut(&NodeProbe<'a>)) {
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
    ( @feature_SHL [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, $arena, Shl, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $arena:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, $arena, Shl, $l, $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, $arena, Shr, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $arena:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, $arena, Shr, $l, $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, $arena, Less, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $arena:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, $arena, Less, $l, $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BiNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, $arena, Bor, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $arena:ident, $l:expr, $( $r:tt)+ ) => { $crate::BiNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, $arena, Bor, $l, $( $r)+ ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, | $( $body:tt)+ ) => { $Node::New( $Arg::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, || $( $body:tt)+ ) => { $Node::New( $Arg::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, move | $( $body:tt)+ ) => { $Node::New( $Arg::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, move || $( $body:tt)+ ) => { $Node::New( $Arg::New( move || $( $body)+ ) ) };

    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:literal [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $arena, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $arena, $l ), $( $closure )* )
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $expr:tt)+ ) [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $arena, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $arena, ( $( $expr )+ ) ), $( $closure )* )
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $arena, $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $arena, $s ), $( $closure )* )
    };

    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $base:expr, | $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move | $( $closure)*
        )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $base:expr, || $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move || $( $closure)*
        )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $base:expr, move | $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move | $( $closure)*
        )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $base:expr, move || $( $closure:tt )* ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNodeAction(
            $base,
            move || $( $closure)*
        )
    };

    ( @wrap $leaf:expr ) => {
        Into::into( $leaf )
    };
    ( @define [ $( $cb:tt )* ], $Arg:ident, $arena:ident, $( $inner:tt )+ ) => {
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
                        _Children: [&'a (dyn $crate::stalks::INode<'a> + Send + Sync); 2],
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
                    fn	NewBranch( arena: & $crate::stalks::node::NodeArena<'a>, op: $crate::stalks::ChildOp, left: Self, right: Self) -> Self
                    {
                        let left_ref = arena.Alloc(left);
                        let right_ref = arena.Alloc(right);
                        [<$Arg BiNode>]::Node {
                            _Op: op,
                            _Children: [left_ref, right_ref],
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

                impl<'a> $crate::stalks::INode<'a> for [<$Arg BiNode>]<'a>
                {
                    fn	Attrib( &self) -> Option<& $crate::stalks::Attrib>
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { _Attrib, .. } => _Attrib.as_ref(),
                            [<$Arg BiNode>]::Node { _Attrib, .. } => _Attrib.as_ref(),
                        }
                    }
                    fn Children(&self) -> &[&'a (dyn $crate::stalks::INode<'a> + Send + Sync)] {
                        match self {
                            [<$Arg BiNode>]::Node { _Children, .. } => _Children,
                            _ => &[],
                        }
                    }
                    fn	ChildOp( &self) -> Option< $crate::stalks::ChildOp>
                    {
                        match self {
                            [<$Arg BiNode>]::Leaf { .. } => None,
                            [<$Arg BiNode>]::Node { _Op, .. } => Some( *_Op),
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
                $crate::BiNodeTree!( @cb [ $( $cb)* ], $Arg, [<$Arg BiNode>], $arena, $( $inner )+ )
            }
        }
    };
    ( $Arg:ident, $arena:ident, $( $inner:tt )+ ) => {
        $crate::BiNodeTree!( @define [ $crate::BiNodeTree ], $Arg, $arena, $( $inner )+ )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $arena, $( $inner)+ ) };

    // ── Leaf [ action ] ────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:literal [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $arena, $l [ $( $inner )* ] )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $expr:tt)+ ) [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $arena, ( $( $expr )+ ) [ $( $inner )* ] )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $arena, [ $s ] [ $( $inner )* ] )
    };

    // ── Binary: [ boxet ] OP rhs ────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $arena, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $arena, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $arena, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $arena, $s ) ), $( $r )+ ) };

    // ── Leaf Boxet ──────────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, [ $s:literal ] ) => {
        $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $arena, $s )
    };

    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, $arena, ( $( $l)+ ), $( $r)+ ) };

    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:ident <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:ident |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:literal <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $l:literal |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $arena, $l, $( $r)+ ) };

    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, || $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, move | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, move | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, move || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, $arena, move || $( $body)+ ) };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $leaf:expr ) => {
        $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNode( $leaf )
    };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------
    // @bg : binary — (group) OP rhs
    ( @bg [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        $Node::NewBranch(
            &$arena,
            $crate::stalks::ChildOp::$op,
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $arena, $( $l)+ ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $arena, $( $r)+ ) )
    };
    // @bl : binary — leaf OP rhs
    ( @bl [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $arena:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        $Node::NewBranch(
            &$arena,
            $crate::stalks::ChildOp::$op,
            $crate::stalks::node::IntoBiNode::< $Arg, $Node >::IntoBiNode( $l ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $arena, $( $r)+ ) )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_NEW $( $args:tt )* ) => { compile_error!( "Closure literal is not enabled for this tree") };
    ( @feature_BOXET  $( $args:tt )* ) => { compile_error!( "Boxet [ ... ] is not enabled for this tree") };
    ( @feature_ACTION $( $args:tt )* ) => { compile_error!( "Action suffix [ closure ] is not enabled for this tree") };
}
pub use crate::BiNodeTree;
