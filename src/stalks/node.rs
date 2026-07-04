//-- node.rs -------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::xflux::XField, stalks::WorkPtr };
use	std::fmt;
use	crate::silo::{ Arr, IAccess, Stash, U32 };
 
//---------------------------------------------------------------------------------------------------------------------------------

pub enum Attrib
{
    Repeat( crate::silo::USeg),
    Action( Box< dyn Fn() + 'static>),
}

impl fmt::Display for Attrib
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Attrib::Repeat( _ ) => f.write_str( "Repeat"),
            Attrib::Action( _ ) => f.write_str( "Action"),
        }
    }
}

#[derive( Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp
{
    Sum,
    Prod,
    Less,
    Bor,
    Shl,
    Shr,
    Sub,
    Div,
    Pow,
    None
}


//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for BinOp
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            BinOp::Sum => write!( f, "+"),
            BinOp::Prod => write!( f, "*"),
            BinOp::Less => write!( f, "<"),
            BinOp::Bor => write!( f, "|"),
            BinOp::Shl => write!( f, "<<"),
            BinOp::Shr => write!( f, "Shr"),
            BinOp::Sub => write!( f, "Sub"),
            BinOp::Div => write!( f, "Div"),
            BinOp::Pow => write!( f, "Pow"),
            BinOp::None => write!( f, "None"),
        }
    }
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

impl< 'a> INode< 'a> for U32
{
    fn	_Size( &self) -> U32 { U32(0) }
    fn	_At( &self, _idx: U32) -> &DynINode< 'a> { panic!("Leaf") }
    fn	Value( &self) -> Option< WorkPtr< 'a>> { None }
    fn	DocStr( &self) -> &'static str { "" }
    fn	BinOp( &self) -> BinOp { BinOp::None }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INode< 'a>: Send + Sync
{
    fn	_Size( &self) -> U32;
    fn	_At( &self, idx: U32) -> &DynINode< 'a>;

    fn	Value( &self) -> Option< WorkPtr< 'a>>;

    fn	DocStr( &self) -> &'static str;

    fn	Attrib( &self) -> Option< &Attrib>
    {
        None
    }

    fn	BinOp( &self) -> BinOp;

    fn	IsLeaf( &self) -> bool
    {
        self._Size() == U32( 0)
    }

    fn	AsAny( &self) -> Option<&dyn core::any::Any>
    {
        None
    }

    fn	TraverseDF( &'a self, fnMut: &mut dyn FnMut( &'a DynINode< 'a>, TraversalEvent))
    where
        Self: Sized,
    {
        TraverseDepthFirst( self, fnMut);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn FluxDynINode< 'b, 'a>( node: &'b DynINode< 'a>, field: &mut XField< 'b>)
{
    let  	mut step = 0u32;
    *field = XField::Obj( Box::new( move |key, item| {
        if step == 0 {
            *key = "DocStr".to_string();
            *item = XField::Str( node.DocStr());
            step += 1;
            true
        } else if step == 1 {
            *key = "BinOp".to_string();
            *item = XField::U64( node.BinOp() as u64);
            step += 1;
            true
        } else if step == 2 {
            *key = "ChildrenSize".to_string();
            *item = XField::U64( node._Size().0 as u64);
            step += 1;
            true
        } else if step >= 3 && step < 3 + node._Size().0 {
            let  	childIdx = step - 3;
            *key = format!( "Child_{}", childIdx);
            let  	child = node._At( U32( childIdx));
            FluxDynINode( child, item);
            step += 1;
            true
        } else {
            false
        }
    }));
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

    pub fn	DiveDf< 'b>( &'b self, fnMut: &mut dyn FnMut( &NodeProbe< 'b, 'a>, bool))
    {
        let  	nodeProbe = NodeProbe::New( 1024, self);
        TraverseDepthFirst( self, &mut |node, event| match event {
            TraversalEvent::Entry( idx) => {
                if idx == U32( 0) {
                    nodeProbe.Push( node);
                    fnMut( &nodeProbe, true);
                }
            }
            TraversalEvent::Exit => {
                fnMut( &nodeProbe, false);
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
        self._NodeStash.Stk().Push( node);
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

pub trait IntoNodule< T, N: Sized>
{
    fn	IntoNodule( self) -> N;
}

//---------------------------------------------------------------------------------------------------------------------------------



//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! NodeTree {
    // ---- FEATURE OPT-INS FOR NodeTree ITSELF ----------------------------------------------------------------------------
    ( @feature_PLUS [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Sum, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_PLUS [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Sum, $l, $( $r)+ ) };
    ( @feature_STAR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Prod, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_STAR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Prod, $l, $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Shl, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Shl, $l, $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Shr, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Shr, $l, $( $r)+ ) };
    ( @feature_MINUS [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Sub, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_MINUS [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Sub, $l, $( $r)+ ) };
    ( @feature_DIV [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Div, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_DIV [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Div, $l, $( $r)+ ) };
    ( @feature_POW [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Pow, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_POW [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Pow, $l, $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Less, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Less, $l, $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, $Node, Bor, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, $Node, Bor, $l, $( $r)+ ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $Node::New( $Arg::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $Node::New( $Arg::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $Node::New( $Arg::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $Node::New( $Arg::New( move || $( $body)+ ) ) };


    ( @wrap $leaf:expr ) => {
        Into::into( $leaf )
    };

    //-----------------------------------------------------------------------------------------------------------------------------

    ( @define [ $( $cb:tt )* ], $Arg:ident, $( $inner:tt )+ ) => {
        {
            paste::paste! {
                #[allow(dead_code)]
                enum [<$Arg Nodule>]<'a> {
                    Leaf {
                        _Val: $Arg,
                    },
                    UniNode {
                        _Child: Box<$crate::stalks::DynINode<'a>>,
                        _Attrib: $crate::stalks::node::Attrib,
                    },
                    BinNode {
                        _BinOp: $crate::stalks::BinOp,
                        _Children: [Box<$crate::stalks::DynINode<'a>>; 2],
                    }
                }
                unsafe impl<'a> Send for [<$Arg Nodule>]<'a> {}
                unsafe impl<'a> Sync for [<$Arg Nodule>]<'a> {}

                #[allow(dead_code)]
                impl<'a> [<$Arg Nodule>]<'a>
                {
                    fn	New( value: $Arg) -> Self
                    {
                        [<$Arg Nodule>]::Leaf {
                            _Val: value,
                        }
                    }
                    fn  NewUniNode( attrib: $crate::stalks::node::Attrib, child: Self) -> Self
                    {
                        [<$Arg Nodule>]::UniNode {
                            _Child: Box::new(child),
                            _Attrib: attrib,
                        }
                    }
                    fn  NewBinNode( op: $crate::stalks::BinOp, left: Self, right: Self) -> Self
                    {
                        [<$Arg Nodule>]::BinNode {
                            _BinOp: op,
                            _Children: [Box::new(left), Box::new(right)],
                        }
                    }
                }

                impl<'a> [<$Arg Nodule>]<'a> {
                    #[allow(dead_code)]
                    pub fn Children<'b>(&'b self) -> $crate::stalks::node::NodeChildren<'b, 'a> {
                        $crate::stalks::node::NodeChildren(self)
                    }
                }

                impl<'a> $crate::flux::IXFluxable for [<$Arg Nodule>]<'a>
                where
                    $Arg: $crate::flux::IXFluxable + 'a,
                {
                    fn	ToXFlux< 'b>( &'b self, field: &mut $crate::flux::xflux::XField< 'b>)
                    {
                        let  	mut step = $crate::silo::U32( 0);
                        let  	node: &'b [<$Arg Nodule>]<'a> = self;
                        *field = $crate::flux::xflux::XField::Obj( Box::new( move |key, item| {
                            match node {
                                [<$Arg Nodule>]::Leaf { _Val, .. } => {
                                    if step == $crate::silo::U32( 0) {
                                        *key = "Leaf".to_string();
                                        *item = $crate::flux::xflux::XField::Fluxable( _Val);
                                        step.0 += 1;
                                        true
                                    } else {
                                        false
                                    }
                                },
                                [<$Arg Nodule>]::UniNode { _Child, .. } => {
                                    if step == $crate::silo::U32( 0) {
                                        *key = "Child".to_string();
                                        let  	child = &**_Child;
                                        $crate::stalks::node::FluxDynINode( child, item);
                                        step.0 += 1;
                                        true
                                    } else {
                                        false
                                    }
                                },
                                [<$Arg Nodule>]::BinNode { _BinOp, _Children, .. } => {
                                    if step == $crate::silo::U32( 0) {
                                        *key = "Op".to_string();
                                        *item = $crate::flux::xflux::XField::U64( *_BinOp as u64);
                                        step.0 += 1;
                                        true
                                    } else if step == $crate::silo::U32( 1) {
                                        *key = "LeftChild".to_string();
                                        let  	child = &*_Children[0];
                                        $crate::stalks::node::FluxDynINode( child, item);
                                        step.0 += 1;
                                        true
                                    } else if step == $crate::silo::U32( 2) {
                                        *key = "RightChild".to_string();
                                        let  	child = &*_Children[1];
                                        $crate::stalks::node::FluxDynINode( child, item);
                                        step.0 += 1;
                                        true
                                    } else {
                                        false
                                    }
                                }
                            }
                        }));
                    }
                }

                impl<'a> $crate::stalks::INode<'a> for [<$Arg Nodule>]<'a>
                {
                    fn _Size(&self) -> $crate::silo::U32 {
                        match self {
                            [<$Arg Nodule>]::UniNode { .. } => $crate::silo::U32(1),
                            [<$Arg Nodule>]::BinNode { .. } => $crate::silo::U32(2),
                            _ => $crate::silo::U32(0),
                        }
                    }
                    fn _At(&self, idx: $crate::silo::U32) -> &$crate::stalks::DynINode<'a> {
                        match self {
                            [<$Arg Nodule>]::UniNode { _Child, .. } => {
                                if idx.0 == 0 {
                                    &**_Child
                                } else {
                                    panic!("At called on UniNode with index > 0")
                                }
                            }
                            [<$Arg Nodule>]::BinNode { _Children, .. } => &*_Children[idx.0 as usize],
                            _ => panic!("At called on Leaf"),
                        }
                    }

                    fn	Value( &self) -> Option< $crate::stalks::WorkPtr< 'a>>
                    {
                        match self {
                            [<$Arg Nodule>]::Leaf { _Val, .. } => {
                                _Val.Value()
                            }
                            _ => None
                        }
                    }

                    fn	AsAny( &self) -> Option<&dyn core::any::Any>
                    {
                        match self {
                            [<$Arg Nodule>]::Leaf { _Val, .. } => {
                                _Val.AsAny()
                            }
                            _ => None
                        }
                    }

                    fn	DocStr( &self) -> &'static str
                    {
                        match self {
                            [<$Arg Nodule>]::Leaf { _Val, .. } => {
                                _Val.DocStr()
                            }
                            _ => ""
                        }
                    }


                    fn	Attrib( &self) -> Option<& $crate::stalks::node::Attrib>
                    {
                        match self {
                            [<$Arg Nodule>]::UniNode { _Attrib, .. } => Some(_Attrib),
                            _ => None,
                        }
                    }

                    fn	BinOp( &self) -> $crate::stalks::BinOp
                    {
                        match self {
                            [<$Arg Nodule>]::Leaf { .. } => $crate::stalks::BinOp::None,
                            [<$Arg Nodule>]::UniNode { .. } => $crate::stalks::BinOp::None,
                            [<$Arg Nodule>]::BinNode { _BinOp, .. } => *_BinOp,
                        }
                    }
                }
                impl<'a, I > $crate::stalks::node::IntoNodule< $Arg, [<$Arg Nodule>]<'a>> for I
                where
                    I: Into< $Arg >,
                {
                    fn	IntoNodule( self) -> [<$Arg Nodule>]<'a>
                    {
                        [<$Arg Nodule>]::New( self.into() )
                    }
                }
                impl<'a> $crate::stalks::node::IntoNodule< $Arg, [<$Arg Nodule>]<'a>> for [<$Arg Nodule>]<'a>
                {
                    fn	IntoNodule( self) -> [<$Arg Nodule>]<'a>
                    {
                        self
                    }
                }
                $crate::NodeTree!( @cb [ $( $cb)* ], $Arg, [<$Arg Nodule>], $( $inner )+ )
            }
        }
    };

    //-----------------------------------------------------------------------------------------------------------------------------

    ( $Arg:ident, $( $inner:tt )+ ) => {
        $crate::NodeTree!( @define [ $crate::NodeTree ], $Arg, $( $inner )+ )
    };

    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $inner)+ ) };

    // ── Binary: [ boxet ] OP rhs ────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };

    // ── Leaf Boxet ──────────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] ) => {
        $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s )
    };

    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };

    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };

    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, || $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move || $( $body)+ ) };

    // ── Postfix Boxet ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) [ $( $body:tt)+ ] ) => { $( $cb)* !( @feature_PostBoxet [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), [ $( $body)+ ] ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident [ $( $body:tt)+ ] ) => { $( $cb)* !( @feature_PostBoxet [ $( $cb)* ], @bl $Arg, $Node, $l, [ $( $body)+ ] ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal [ $( $body:tt)+ ] ) => { $( $cb)* !( @feature_PostBoxet [ $( $cb)* ], @bl $Arg, $Node, $l, [ $( $body)+ ] ) };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $leaf:expr ) => {
        $crate::stalks::node::IntoNodule::< $Arg, $Node >::IntoNodule( $leaf )
    };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------
    // @bg : binary — (group) OP rhs
    ( @bg [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        $Node::NewBinNode(
            $crate::stalks::BinOp::$op,
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };
    // @bl : binary — leaf OP rhs
    ( @bl [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        $Node::NewBinNode(
            $crate::stalks::BinOp::$op,
            $crate::stalks::node::IntoNodule::< $Arg, $Node >::IntoNodule( $l ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };
    
    // @feature_PostBoxet
    ( @feature_PostBoxet [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), [ $( $body:tt)+ ] ) => {
        $Node::NewUniNode(
            $crate::stalks::node::Attrib::Action( Box::new( $( $body)+ ) ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ) )
    };
    ( @feature_PostBoxet [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, [ $( $body:tt)+ ] ) => {
        $Node::NewUniNode(
            $crate::stalks::node::Attrib::Action( Box::new( $( $body)+ ) ),
            $crate::stalks::node::IntoNodule::< $Arg, $Node >::IntoNodule( $l ) )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_NEW $( $args:tt )* ) => { compile_error!( "Closure literal is not enabled for this tree") };
    ( @feature_BOXET  $( $args:tt )* ) => { compile_error!( "Boxet [ ... ] is not enabled for this tree") };

}

pub use crate::NodeTree;

//-----------------------------------------------------------------------------------------------------------------------------
