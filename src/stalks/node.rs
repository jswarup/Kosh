//-- node.rs -------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::xflux::XField, stalks::WorkPtr };
use	crate::stalks::work::IWork;
use	std::fmt;
use	crate::silo::{ Arr, IAccess, Stash, U32 };
 
//---------------------------------------------------------------------------------------------------------------------------------

pub enum Attrib
{
    Repeat( crate::silo::USeg),
    Action( Box< dyn IWork + 'static>),
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

pub enum Nodule<'a, T> {
    Leaf {
        _Val: T,
    },
    UniNode {
        _Child: Box<DynINode<'a>>,
        _Attrib: Attrib,
    },
    BinNode {
        _BinOp: BinOp,
        _Children: [Box<DynINode<'a>>; 2],
    }
}
unsafe impl<'a, T> Send for Nodule<'a, T> {}
unsafe impl<'a, T> Sync for Nodule<'a, T> {}

impl<'a, T> Nodule<'a, T>
where
    T: INode<'a> + crate::flux::IXFluxable + Send + Sync + 'a,
{
    pub fn New(value: T) -> Self {
        Nodule::Leaf { _Val: value }
    }
    pub fn NewUniNode(attrib: Attrib, child: Self) -> Self {
        Nodule::UniNode { _Child: Box::new(child), _Attrib: attrib }
    }
    pub fn NewBinNode(op: BinOp, left: Self, right: Self) -> Self {
        Nodule::BinNode { _BinOp: op, _Children: [Box::new(left), Box::new(right)] }
    }
    pub fn Children<'b>(&'b self) -> NodeChildren<'b, 'a> {
        NodeChildren(self)
    }
}

impl<'a, T> crate::flux::IXFluxable for Nodule<'a, T>
where
    T: crate::flux::IXFluxable + 'a,
{
    fn ToXFlux<'b>(&'b self, field: &mut crate::flux::xflux::XField<'b>) {
        let mut step = crate::silo::U32(0);
        let node: &'b Nodule<'a, T> = self;
        *field = crate::flux::xflux::XField::Obj(Box::new(move |key, item| {
            match node {
                Nodule::Leaf { _Val, .. } => {
                    if step == crate::silo::U32(0) {
                        *key = "Leaf".to_string();
                        *item = crate::flux::xflux::XField::Fluxable(_Val);
                        step.0 += 1;
                        true
                    } else {
                        false
                    }
                },
                Nodule::UniNode { _Child, .. } => {
                    if step == crate::silo::U32(0) {
                        *key = "Child".to_string();
                        let child = &**_Child;
                        FluxDynINode(child, item);
                        step.0 += 1;
                        true
                    } else {
                        false
                    }
                },
                Nodule::BinNode { _BinOp, _Children, .. } => {
                    if step == crate::silo::U32(0) {
                        *key = "Op".to_string();
                        *item = crate::flux::xflux::XField::U64(*_BinOp as u64);
                        step.0 += 1;
                        true
                    } else if step == crate::silo::U32(1) {
                        *key = "LeftChild".to_string();
                        let child = &*_Children[0];
                        FluxDynINode(child, item);
                        step.0 += 1;
                        true
                    } else if step == crate::silo::U32(2) {
                        *key = "RightChild".to_string();
                        let child = &*_Children[1];
                        FluxDynINode(child, item);
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

impl<'a, T> INode<'a> for Nodule<'a, T>
where
    T: INode<'a>,
{
    fn _Size(&self) -> crate::silo::U32 {
        match self {
            Nodule::UniNode { .. } => crate::silo::U32(1),
            Nodule::BinNode { .. } => crate::silo::U32(2),
            _ => crate::silo::U32(0),
        }
    }
    fn _At(&self, idx: crate::silo::U32) -> &DynINode<'a> {
        match self {
            Nodule::UniNode { _Child, .. } => {
                if idx.0 == 0 {
                    &**_Child
                } else {
                    panic!("At called on UniNode with index > 0")
                }
            }
            Nodule::BinNode { _Children, .. } => &*_Children[idx.0 as usize],
            _ => panic!("At called on Leaf"),
        }
    }
    fn Value(&self) -> Option<WorkPtr<'a>> {
        match self {
            Nodule::Leaf { _Val, .. } => _Val.Value(),
            _ => None
        }
    }
    fn AsAny(&self) -> Option<&dyn core::any::Any> {
        match self {
            Nodule::Leaf { _Val, .. } => _Val.AsAny(),
            _ => None
        }
    }
    fn DocStr(&self) -> &'static str {
        match self {
            Nodule::Leaf { _Val, .. } => _Val.DocStr(),
            _ => ""
        }
    }
    fn Attrib(&self) -> Option<&Attrib> {
        match self {
            Nodule::UniNode { _Attrib, .. } => Some(_Attrib),
            _ => None,
        }
    }
    fn BinOp(&self) -> BinOp {
        match self {
            Nodule::Leaf { .. } => BinOp::None,
            Nodule::UniNode { .. } => BinOp::None,
            Nodule::BinNode { _BinOp, .. } => *_BinOp,
        }
    }
}

impl<'a, T, I> IntoNodule<T, Nodule<'a, T>> for I
where
    I: Into<T>,
    T: INode<'a> + crate::flux::IXFluxable + Send + Sync + 'a,
{
    fn IntoNodule(self) -> Nodule<'a, T> {
        Nodule::New(self.into())
    }
}



//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! NodeTree {
    // ---- FEATURE OPT-INS FOR NodeTree ITSELF ----------------------------------------------------------------------------
    ( @feature_PLUS [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Sum, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_PLUS [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Sum, $l, $( $r)+ ) };
    ( @feature_STAR [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Prod, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_STAR [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Prod, $l, $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Shl, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Shl, $l, $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Shr, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Shr, $l, $( $r)+ ) };
    ( @feature_MINUS [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Sub, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_MINUS [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Sub, $l, $( $r)+ ) };
    ( @feature_DIV [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Div, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_DIV [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Div, $l, $( $r)+ ) };
    ( @feature_POW [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Pow, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_POW [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Pow, $l, $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Less, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Less, $l, $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::NodeTree!( @bg [ $( $cb)* ], $Arg, Bor, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, $( $r:tt)+ ) => { $crate::NodeTree!( @bl [ $( $cb)* ], $Arg, Bor, $l, $( $r)+ ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, | $( $body:tt)+ ) => { $crate::stalks::node::Nodule::<$Arg>::New( $Arg::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, move | $( $body:tt)+ ) => { $crate::stalks::node::Nodule::<$Arg>::New( $Arg::New( move | $( $body)+ ) ) };


    //-----------------------------------------------------------------------------------------------------------------------------

    ( @define [ $( $cb:tt )* ], $Arg:ident, $( $inner:tt )+ ) => {
        $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $inner )+ )
    };

    //-----------------------------------------------------------------------------------------------------------------------------

    ( $Arg:ident, $( $inner:tt )+ ) => {
        $crate::NodeTree!( @define [ $crate::NodeTree ], $Arg, $( $inner )+ )
    };

    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $inner)+ ) };

    // ── Binary: [ boxet ] OP rhs ────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ), $( $r )+ ) };

    // ── Leaf Boxet ──────────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] ) => {
        $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s )
    };

    // ── Prefix * with Binary OP ─────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    // ── Prefix + with Binary OP ─────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] + $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] - $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] * $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] / $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] ^ $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] < $( $r:tt)+ ) => { $( $cb)* !( @feature_LT [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] | $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, ( $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) ), $( $r)+ ) };
    // ── Prefix * Leaf ───────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 0.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) };
    // ── Prefix + Leaf ───────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] ) => { $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg( 1.into(), 0.into() ) ), $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) };
    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), $( $r)+ ) };

    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $l, $( $r)+ ) };

    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, move | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, move | $( $body)+ ) };

    // ── Postfix Boxet ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, ( $( $l:tt)+ ) [ $( $body:tt)+ ] ) => { $( $cb)* !( @feature_PostBoxet [ $( $cb)* ], @bg $Arg, ( $( $l)+ ), [ $( $body)+ ] ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:ident [ $( $body:tt)+ ] ) => { $( $cb)* !( @feature_PostBoxet [ $( $cb)* ], @bl $Arg, $l, [ $( $body)+ ] ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $l:literal [ $( $body:tt)+ ] ) => { $( $cb)* !( @feature_PostBoxet [ $( $cb)* ], @bl $Arg, $l, [ $( $body)+ ] ) };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $leaf:expr ) => {
        $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $leaf )
    };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------
    // @bg : binary — (group) OP rhs
    ( @bg [ $( $cb:tt)* ], $Arg:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewBinNode(
            $crate::stalks::BinOp::$op,
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $r)+ ) )
    };
    // @bl : binary — leaf OP rhs
    ( @bl [ $( $cb:tt)* ], $Arg:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewBinNode(
            $crate::stalks::BinOp::$op,
            $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $r)+ ) )
    };
    
    // @feature_PostBoxet
    ( @feature_PostBoxet [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), [ | $arg:ident | $( $body:tt)+ ] ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewUniNode(
            $crate::stalks::node::Attrib::Action( Box::new( move | $arg: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ) ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) )
    };
    ( @feature_PostBoxet [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, [ | $arg:ident | $( $body:tt)+ ] ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewUniNode(
            $crate::stalks::node::Attrib::Action( Box::new( move | $arg: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ) ),
            $crate::stalks::node::IntoNodule::< $Arg, $crate::stalks::node::Nodule<$Arg> >::IntoNodule( $l ) )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_NEW $( $args:tt )* ) => { compile_error!( "Closure literal is not enabled for this tree") };
    ( @feature_BOXET  $( $args:tt )* ) => { compile_error!( "Boxet [ ... ] is not enabled for this tree") };

}

pub use crate::NodeTree;

//-----------------------------------------------------------------------------------------------------------------------------
