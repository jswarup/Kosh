//-- node.rs -------------------------------------------------------------------------------------------------------------------
use	crate::stalks::WorkPtr;
use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::stalks::work::DynIWork;
use	std::fmt;
use	crate::silo::{ Arr, IAccess, Stash, U32 };
 
//---------------------------------------------------------------------------------------------------------------------------------

pub enum Attrib
{
    Action( Box< DynIWork< 'static>>),
    Repeat( crate::silo::USeg),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for Attrib
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Attrib::Action( _ ) => f.write_str( "Action"),
            Attrib::Repeat( useg) => write!( f, "Repeat( {:?})", useg),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for Attrib
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	attribVal = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                match attribVal {
                    Attrib::Action( _ ) => {
                        *key = "Action".to_string();
                        *item = XField::Str( "Action");
                    }
                    Attrib::Repeat( useg) => {
                        *key = "Repeat".to_string();
                        *item = XField::FluxSource( useg);
                    }
                }
                step += 1;
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


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

impl< 'a> INode< 'a> for U32
{
}

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

pub trait INode< 'a>: Send + Sync + crate::flux::IXFluxSource
{
    fn	_Size( &self) -> U32 { U32( 0) }
    fn	_At( &self, _idx: U32) -> &DynINode< 'a> { panic!( "Leaf") }

    fn	Value( &self) -> Option< WorkPtr< 'a>> { None }

    fn	DocStr( &self) -> &'static str { "" }

    fn	BinOp( &self) -> BinOp { BinOp::None }

    fn	Action( &self) -> Option< *const DynIWork< 'static>> { None }

    fn	AsRawLeaf( &self) -> *const () { std::ptr::null() }

    fn	Attrib( &self) -> Option< &Attrib>
    {
        None
    }


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

impl< 'a, 'r> crate::flux::IXFluxSource for &'r DynINode< 'a>
{
    fn	ToXField< 'b>( &'b self, field: &mut crate::flux::xflux::XField< 'b>)
    {
        (**self).ToXField( field);
    }
}

impl< 'a, 'r, T: crate::flux::IXFluxSource> crate::flux::IXFluxSource for &'r T
{
    fn	ToXField< 'b>( &'b self, field: &mut crate::flux::xflux::XField< 'b>)
    {
        (**self).ToXField( field);
    }
}

impl< 'a, 'r> INode< 'a> for &'r DynINode< 'a>
{
    fn	_Size( &self) -> U32 { (**self)._Size() }
    fn	_At( &self, idx: U32) -> &DynINode< 'a> { (**self)._At( idx) }
    fn	Value( &self) -> Option< crate::stalks::WorkPtr< 'a>> { (**self).Value() }
    fn	DocStr( &self) -> &'static str { (**self).DocStr() }
    fn	BinOp( &self) -> BinOp { (**self).BinOp() }
    fn	Action( &self) -> Option< *const crate::stalks::work::DynIWork< 'static>> { (**self).Action() }
    fn	AsRawLeaf( &self) -> *const () { (**self).AsRawLeaf() }
    fn	Attrib( &self) -> Option< &Attrib> { (**self).Attrib() }
    fn	IsLeaf( &self) -> bool { (**self).IsLeaf() }
    fn	AsAny( &self) -> Option<&dyn core::any::Any> { (**self).AsAny() }
    fn	TraverseDF( &'a self, fnMut: &mut dyn FnMut( &'a DynINode< 'a>, TraversalEvent)) where Self: Sized { TraverseDepthFirst(&(**self), fnMut) }
}

impl< 'a, 'r, T: INode< 'a>> INode< 'a> for &'r T
{
    fn	_Size( &self) -> U32 { (**self)._Size() }
    fn	_At( &self, idx: U32) -> &DynINode< 'a> { (**self)._At( idx) }
    fn	Value( &self) -> Option< crate::stalks::WorkPtr< 'a>> { (**self).Value() }
    fn	DocStr( &self) -> &'static str { (**self).DocStr() }
    fn	BinOp( &self) -> BinOp { (**self).BinOp() }
    fn	Action( &self) -> Option< *const crate::stalks::work::DynIWork< 'static>> { (**self).Action() }
    fn	AsRawLeaf( &self) -> *const () { (**self).AsRawLeaf() }
    fn	Attrib( &self) -> Option< &Attrib> { (**self).Attrib() }
    fn	IsLeaf( &self) -> bool { (**self).IsLeaf() }
    fn	AsAny( &self) -> Option<&dyn core::any::Any> { (**self).AsAny() }
    fn	TraverseDF( &'a self, fnMut: &mut dyn FnMut( &'a DynINode< 'a>, TraversalEvent)) where Self: Sized { TraverseDepthFirst(*self as &DynINode<'a>, fnMut) }
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
    },
    ParNode {
        _Children: [Box<DynINode<'a>>; 2],
    },
    CatNode {
        _Children: [Box<DynINode<'a>>; 2],
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ParNode< L, R>
{
    pub _Left: L,
    pub _Right: R,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IXFluxSource for ParNode< L, R>
where
    L: IXFluxSource,
    R: IXFluxSource,
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Left".to_string();
                node._Left.ToXField( item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Right".to_string();
                node._Right.ToXField( item);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, L, R> INode< 'a> for ParNode< L, R>
where
    L: INode< 'a> + Send + Sync + 'a,
    R: INode< 'a> + Send + Sync + 'a,
{
    fn	_Size( &self) -> U32
    {
        U32( 2)
    }

    fn	_At( &self, idx: U32) -> &DynINode< 'a>
    {
        match idx.0 {
            0 => {
                &self._Left
            }
            1 => {
                &self._Right
            }
            _ => {
                panic!( "At called on ParNode with index > 1");
            }
        }
    }

    fn	BinOp( &self) -> BinOp
    {
        BinOp::Bor
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct CatNode< L, R>
{
    pub _Left: L,
    pub _Right: R,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IXFluxSource for CatNode< L, R>
where
    L: IXFluxSource,
    R: IXFluxSource,
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Left".to_string();
                node._Left.ToXField( item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Right".to_string();
                node._Right.ToXField( item);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, L, R> INode< 'a> for CatNode< L, R>
where
    L: INode< 'a> + Send + Sync + 'a,
    R: INode< 'a> + Send + Sync + 'a,
{
    fn	_Size( &self) -> U32
    {
        U32( 2)
    }

    fn	_At( &self, idx: U32) -> &DynINode< 'a>
    {
        match idx.0 {
            0 => {
                &self._Left
            }
            1 => {
                &self._Right
            }
            _ => {
                panic!( "At called on CatNode with index > 1");
            }
        }
    }

    fn	BinOp( &self) -> BinOp
    {
        BinOp::Less
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl<'a, T> Send for Nodule<'a, T> {}
unsafe impl<'a, T> Sync for Nodule<'a, T> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> Nodule<'a, T>
where
    T: INode<'a> + crate::flux::IXFluxSource + Send + Sync + 'a,
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
    pub fn NewParNode(left: Self, right: Self) -> Self {
        Nodule::ParNode { _Children: [Box::new(left), Box::new(right)] }
    }
    pub fn NewCatNode(left: Self, right: Self) -> Self {
        Nodule::CatNode { _Children: [Box::new(left), Box::new(right)] }
    }
    pub fn Children<'b>(&'b self) -> NodeChildren<'b, 'a> {
        NodeChildren(self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> crate::flux::IXFluxSource for Nodule<'a, T>
where
    T: crate::flux::IXFluxSource + 'a,
{
    fn ToXField<'b>(&'b self, field: &mut crate::flux::xflux::XField<'b>) {
        let mut step = crate::silo::U32(0);
        let node: &'b Nodule<'a, T> = self;
        *field = crate::flux::xflux::XField::Obj(Box::new(move |key, item| {
            match node {
                Nodule::Leaf { _Val, .. } => {
                    if step == crate::silo::U32(0) {
                        *key = "Leaf".to_string();
                        *item = crate::flux::xflux::XField::FluxSource(_Val);
                        step.0 += 1;
                        true
                    } else {
                        false
                    }
                },
                Nodule::UniNode { _Child, _Attrib } => {
                    if step == crate::silo::U32(0) {
                        *key = "Child".to_string();
                        let child = &**_Child;
                        child.ToXField(item);
                        step.0 += 1;
                        true
                    } else if step == crate::silo::U32(1) {
                        *key = "Attrib".to_string();
                        *item = crate::flux::xflux::XField::FluxSource(_Attrib);
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
                        child.ToXField(item);
                        step.0 += 1;
                        true
                    } else if step == crate::silo::U32(2) {
                        *key = "RightChild".to_string();
                        let child = &*_Children[1];
                        child.ToXField(item);
                        step.0 += 1;
                        true
                    } else {
                        false
                    }
                },
                Nodule::ParNode { _Children, .. } | Nodule::CatNode { _Children, .. } => {
                    if step == crate::silo::U32(0) {
                        *key = "LeftChild".to_string();
                        let child = &*_Children[0];
                        child.ToXField(item);
                        step.0 += 1;
                        true
                    } else if step == crate::silo::U32(1) {
                        *key = "RightChild".to_string();
                        let child = &*_Children[1];
                        child.ToXField(item);
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

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> INode<'a> for Nodule<'a, T>
where
    T: INode<'a> + 'a,
{
    fn _Size(&self) -> crate::silo::U32 {
        match self {
            Nodule::UniNode { .. } => crate::silo::U32(1),
            Nodule::BinNode { .. } | Nodule::ParNode { .. } | Nodule::CatNode { .. } => crate::silo::U32(2),
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
            Nodule::BinNode { _Children, .. } | Nodule::ParNode { _Children, .. } | Nodule::CatNode { _Children, .. } => &*_Children[idx.0 as usize],
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
    fn AsRawLeaf(&self) -> *const () {
        match self {
            Nodule::Leaf { _Val, .. } => _Val as *const _ as *const (),
            _ => std::ptr::null()
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
            Nodule::ParNode { .. } => BinOp::Bor,
            Nodule::CatNode { .. } => BinOp::Less,
        }
    }
    fn Action(&self) -> Option<*const DynIWork<'static>> {
        match self {
            Nodule::UniNode { _Attrib: Attrib::Action(func), .. } => Some(func.as_ref() as *const _),
            Nodule::Leaf { _Val, .. } => _Val.Action(),
            _ => None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T, I> IntoNodule<T, Nodule<'a, T>> for I
where
    I: Into<T>,
    T: INode<'a> + crate::flux::IXFluxSource + Send + Sync + 'a,
{
    fn IntoNodule(self) -> Nodule<'a, T> {
        Nodule::New(self.into())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct NodeWrapper<I, T>(pub I, pub std::marker::PhantomData<T>);

impl<'a, T> NodeWrapper<Nodule<'a, T>, T> {
    pub fn resolve(self) -> Nodule<'a, T> {
        self.0
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait FallbackResolveNode<'a, T> {
    fn resolve(self) -> Nodule<'a, T>;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T, I> FallbackResolveNode<'a, T> for NodeWrapper<I, T>
where
    I: Into<T>,
    T: INode<'a> + crate::flux::IXFluxSource + Send + Sync + 'a,
{
    fn resolve(self) -> Nodule<'a, T> {
        Nodule::New(self.0.into())
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
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEWLEAF [ $( $cb)* ], $Arg, $Arg::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, move | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEWLEAF [ $( $cb)* ], $Arg, $Arg::New( move | $( $body)+ ) ) };


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
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] +  $( $r:tt)+ ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] -  $( $r:tt)+ ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] *  $( $r:tt)+ ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] /  $( $r:tt)+ ) => { $( $cb)* !( @feature_DIV [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] ^  $( $r:tt)+ ) => { $( $cb)* !( @feature_POW [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, ( [ $s ] ), $( $r )+ ) };

    // ── Leaf Boxet ──────────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, [ $s:literal ] ) => {
        $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s )
    };

    // ── Prefix * with remainder ─────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( * ( $( $l)+ ) ) $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( * $l ) $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( * $l ) $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( * [ $s ] ) $( $r)+ ) };

    // ── Prefix + with remainder ─────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( + ( $( $l)+ ) ) $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( + $l ) $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( + $l ) $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] $( $r:tt)+ ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, ( + [ $s ] ) $( $r)+ ) };

    // ── Prefix * Leaf ───────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * ( $( $l:tt)+ ) ) => { $( $cb)* !( @feature_REPEAT_STAR [ $( $cb)* ], $Arg, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:ident ) => { $( $cb)* !( @feature_REPEAT_STAR [ $( $cb)* ], $Arg, $( $cb)* !( @feature_RESOLVE_LEAF [ $( $cb)* ], $Arg, $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * $l:literal ) => { $( $cb)* !( @feature_REPEAT_STAR [ $( $cb)* ], $Arg, $( $cb)* !( @feature_RESOLVE_LEAF [ $( $cb)* ], $Arg, $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, * [ $s:literal ] ) => { $( $cb)* !( @feature_REPEAT_STAR [ $( $cb)* ], $Arg, $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) };
    // ── Prefix + Leaf ───────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + ( $( $l:tt)+ ) ) => { $( $cb)* !( @feature_REPEAT_PLUS [ $( $cb)* ], $Arg, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:ident ) => { $( $cb)* !( @feature_REPEAT_PLUS [ $( $cb)* ], $Arg, $( $cb)* !( @feature_RESOLVE_LEAF [ $( $cb)* ], $Arg, $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + $l:literal ) => { $( $cb)* !( @feature_REPEAT_PLUS [ $( $cb)* ], $Arg, $( $cb)* !( @feature_RESOLVE_LEAF [ $( $cb)* ], $Arg, $l ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, + [ $s:literal ] ) => { $( $cb)* !( @feature_REPEAT_PLUS [ $( $cb)* ], $Arg, $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $s ) ) };
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
        $( $cb)* !( @feature_RESOLVE_LEAF [ $( $cb)* ], $Arg, $leaf )
    };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------
    // @bg : binary — (group) OP rhs
    ( @bg [ $( $cb:tt)* ], $Arg:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        $( $cb)* !( @feature_NEWBINNODE [ $( $cb)* ], $Arg, $op, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $r)+ ) )
    };
    // @bl : binary — leaf OP rhs
    ( @bl [ $( $cb:tt)* ], $Arg:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        $( $cb)* !( @feature_NEWBINNODE [ $( $cb)* ], $Arg, $op, $( $cb)* !( @feature_RESOLVE_LEAF [ $( $cb)* ], $Arg, $l ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $r)+ ) )
    };

    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Bor, $l:expr, $r:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewParNode( $l, $r )
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Less, $l:expr, $r:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewCatNode( $l, $r )
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, $op:ident, $l:expr, $r:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewBinNode( $crate::stalks::BinOp::$op, $l, $r )
    };
    ( @feature_NEWUNINODE [ $( $cb:tt)* ], $Arg:ident, $attrib:expr, $child:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $attrib, $child )
    };
    ( @feature_NEWLEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::New( $val )
    };
    ( @feature_RESOLVE_LEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        {
            #[allow(unused_imports)] use $crate::stalks::node::FallbackResolveNode;
            $crate::stalks::node::NodeWrapper( $val, std::marker::PhantomData::<$Arg> ).resolve()
        }
    };
    ( @feature_REPEAT_STAR [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg { _First: $crate::silo::U32(0), _Last: $crate::silo::U32(0) } ), $child )
    };
    ( @feature_REPEAT_PLUS [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Repeat( $crate::silo::USeg { _First: $crate::silo::U32(1), _Last: $crate::silo::U32(0) } ), $child )
    };
    
    // @feature_PostBoxet
    ( @feature_PostBoxet [ $( $cb:tt)* ], @bg $Arg:ident, ( $( $l:tt)+ ), [ | $arg:ident | $( $body:tt)+ ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg,
            Box::new( move | $arg: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $( $l)+ ) )
    };
    ( @feature_PostBoxet [ $( $cb:tt)* ], @bl $Arg:ident, $l:expr, [ | $arg:ident | $( $body:tt)+ ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg,
            Box::new( move | $arg: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            $( $cb)* !( @feature_RESOLVE_LEAF [ $( $cb)* ], $Arg, $l ) )
    };

    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $action:expr, $child:expr ) => {
        $crate::stalks::node::Nodule::<$Arg>::NewUniNode( $crate::stalks::node::Attrib::Action( $action ), $child )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_NEW $( $args:tt )* ) => { compile_error!( "Closure literal is not enabled for this tree") };
    ( @feature_BOXET  $( $args:tt )* ) => { compile_error!( "Boxet [ ... ] is not enabled for this tree") };

}

//---------------------------------------------------------------------------------------------------------------------------------

pub use crate::NodeTree;

//-----------------------------------------------------------------------------------------------------------------------------
