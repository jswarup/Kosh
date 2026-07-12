//-- termtree.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IXFluxSource, xflux::XField };
use	std::fmt;
use	crate::stalks::{ DynIWorker, IWork };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub enum Term {
    String( String),
    Real( f64),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Term
{
    fn	default() -> Self
    {
        Self::String( "".to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for Term
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	term = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                match term {
                    Term::String( s) => {
                        *key = "String".to_string();
                        *item = XField::Str( s);
                    }
                    Term::Real( v) => {
                        *key = "Real".to_string();
                        *item = XField::F64( *v);
                    }
                }
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}



impl IWork for Term
{
    fn	DoWork( &mut self, _worker: &DynIWorker< '_>)
    {
        match self {
            Self::String( s) => print!( "{} ", s),
            Self::Real( v) => print!( "{} ", v),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for Term
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Self::String( s) => write!( f, "Term( {})", s),
            Self::Real( v) => write!( f, "Term( {})", v),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< char> for Term
{
    fn	from( c: char) -> Self
    {
        Self::String( c.to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< String> for Term
{
    fn	from( s: String) -> Self
    {
        Self::String( s)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< &str> for Term
{
    fn	from( s: &str) -> Self
    {
        Self::String( s.to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< f64> for Term
{
    fn	from( v: f64) -> Self
    {
        Self::Real( v)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u64)]
pub enum TermOp
{
    Sum = 0,
    Prod = 1,
    Sub = 2,
    Div = 3,
    Pow = 4,
    None = 5,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ITermNode
{
    fn	ChildrenCount( &self) -> usize;
    fn	Child( &self, idx: usize) -> &dyn ITermNode;
    fn	Op( &self) -> TermOp;
    fn	AsLeaf( &self) -> Option< &Term>;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITermNode for Term
{
    fn	ChildrenCount( &self) -> usize
    {
        0
    }

    fn	Child( &self, _idx: usize) -> &dyn ITermNode
    {
        panic!( "Leaf has no children");
    }

    fn	Op( &self) -> TermOp
    {
        TermOp::None
    }

    fn	AsLeaf( &self) -> Option< &Term>
    {
        Some( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: ITermNode + ?Sized> ITermNode for &T
{
    fn	ChildrenCount( &self) -> usize
    {
        ( **self).ChildrenCount()
    }

    fn	Child( &self, idx: usize) -> &dyn ITermNode
    {
        ( **self).Child( idx)
    }

    fn	Op( &self) -> TermOp
    {
        ( **self).Op()
    }

    fn	AsLeaf( &self) -> Option< &Term>
    {
        ( **self).AsLeaf()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct TermBinNode< L, R>
{
    pub _Left: L,
    pub _Right: R,
    pub _Op: TermOp,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IXFluxSource for TermBinNode< L, R>
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
                *key = "Op".to_string();
                *item = XField::U64( node._Op as u64);
                step += 1;
                
                true
            } else if step == 1 {
                *key = "LeftChild".to_string();
                node._Left.ToXField( item);
                step += 1;
                
                true
            } else if step == 2 {
                *key = "RightChild".to_string();
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

impl< L, R> ITermNode for TermBinNode< L, R>
where
    L: ITermNode,
    R: ITermNode,
{
    fn	ChildrenCount( &self) -> usize
    {
        2
    }

    fn	Child( &self, idx: usize) -> &dyn ITermNode
    {
        match idx {
            0 => {
                &self._Left
            }
            1 => {
                &self._Right
            }
            _ => {
                panic!( "Index out of bounds");
            }
        }
    }

    fn	Op( &self) -> TermOp
    {
        self._Op
    }

    fn	AsLeaf( &self) -> Option< &Term>
    {
        None
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! TermTree {
    // Helper to construct binary nodes
    ( @bin $op:ident, $l:expr, $( $r:tt )+ ) => {
        &$crate::fresco::termtree::TermBinNode {
            _Left: $l,
            _Right: $crate::TermTree!( $( $r )+ ),
            _Op: $crate::fresco::termtree::TermOp::$op,
        }
    };

    // Group with remainder
    ( ( $( $inner:tt )+ ) + $( $rest:tt )+ ) => { $crate::TermTree!( @bin Sum,  $crate::TermTree!( ( $( $inner )+ ) ), $( $rest )+ ) };
    ( ( $( $inner:tt )+ ) * $( $rest:tt )+ ) => { $crate::TermTree!( @bin Prod, $crate::TermTree!( ( $( $inner )+ ) ), $( $rest )+ ) };
    ( ( $( $inner:tt )+ ) - $( $rest:tt )+ ) => { $crate::TermTree!( @bin Sub,  $crate::TermTree!( ( $( $inner )+ ) ), $( $rest )+ ) };
    ( ( $( $inner:tt )+ ) / $( $rest:tt )+ ) => { $crate::TermTree!( @bin Div,  $crate::TermTree!( ( $( $inner )+ ) ), $( $rest )+ ) };
    ( ( $( $inner:tt )+ ) ^ $( $rest:tt )+ ) => { $crate::TermTree!( @bin Pow,  $crate::TermTree!( ( $( $inner )+ ) ), $( $rest )+ ) };

    // Ident with remainder
    ( $l:ident + $( $rest:tt )+ ) => { $crate::TermTree!( @bin Sum,  $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:ident * $( $rest:tt )+ ) => { $crate::TermTree!( @bin Prod, $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:ident - $( $rest:tt )+ ) => { $crate::TermTree!( @bin Sub,  $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:ident / $( $rest:tt )+ ) => { $crate::TermTree!( @bin Div,  $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:ident ^ $( $rest:tt )+ ) => { $crate::TermTree!( @bin Pow,  $crate::TermTree!( $l ), $( $rest )+ ) };

    // Literal with remainder
    ( $l:literal + $( $rest:tt )+ ) => { $crate::TermTree!( @bin Sum,  $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:literal * $( $rest:tt )+ ) => { $crate::TermTree!( @bin Prod, $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:literal - $( $rest:tt )+ ) => { $crate::TermTree!( @bin Sub,  $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:literal / $( $rest:tt )+ ) => { $crate::TermTree!( @bin Div,  $crate::TermTree!( $l ), $( $rest )+ ) };
    ( $l:literal ^ $( $rest:tt )+ ) => { $crate::TermTree!( @bin Pow,  $crate::TermTree!( $l ), $( $rest )+ ) };

    // Base Case: Group
    ( ( $( $inner:tt )+ ) ) => {
        $crate::TermTree!( $( $inner )+ )
    };

    // Base Case: Leaf
    ( $leaf:expr ) => {
        &< Term as From< _>>::from( $leaf )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
