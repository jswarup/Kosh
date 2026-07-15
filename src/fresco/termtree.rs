//-- termtree.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::{
    flux::{ IFluxImportSource, IFluxExportSource, fluximport::FieldImp, fluxexport::FieldExp },
    stalks::{ DynIWorker, IWork, BinNode, INode, BinOp },
};
use	std::fmt;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub enum Term {
    Null,
    String( String),
    Real( f64),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Term
{
    fn	default() -> Self
    {
        Self::Null
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for Term
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut step = 0u32;
        let  	term = self;
        *field = FieldExp::Obj( Box::new( move |key, item| {
            if step == 0 {
                match term {
                    Term::Null => {
                        *key = "Null".to_string();
                        *item = FieldExp::Null;
                    }
                    Term::String( s) => {
                        *key = "String".to_string();
                        *item = FieldExp::Str( s);
                    }
                    Term::Real( v) => {
                        *key = "Real".to_string();
                        *item = FieldExp::F64( *v);
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
            Self::Null => print!( "Null "),
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
            Self::Null => write!( f, "Term( Null)"),
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

pub trait ITermNode: INode
{
    fn	ChildrenCount( &self) -> usize;
    fn	Child( &self, idx: usize) -> &dyn ITermNode;
    fn	Op( &self) -> BinOp;
    fn	AsLeaf( &self) -> &Term;
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

    fn	Op( &self) -> BinOp
    {
        BinOp::None
    }

    fn	AsLeaf( &self) -> &Term
    {
        self
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

    fn	Op( &self) -> BinOp
    {
        ( **self).Op()
    }

    fn	AsLeaf( &self) -> &Term
    {
        ( **self).AsLeaf()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type TermBinNode< L, R> = BinNode< L, R>;

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> ITermNode for BinNode< L, R>
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

    fn	Op( &self) -> BinOp
    {
        self._Op
    }

    fn	AsLeaf( &self) -> &Term
    {
        static NULL_TERM: Term = Term::Null;
        &NULL_TERM
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait AsTermNode
{
    type Node: ITermNode;
    fn	AsTermNode( self) -> Self::Node;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: ITermNode> AsTermNode for T
{
    type Node = T;
    fn	AsTermNode( self) -> Self::Node
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AsTermNode for char
{
    type Node = Term;
    fn	AsTermNode( self) -> Self::Node
    {
        Term::from( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AsTermNode for &str
{
    type Node = Term;
    fn	AsTermNode( self) -> Self::Node
    {
        Term::from( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AsTermNode for String
{
    type Node = Term;
    fn	AsTermNode( self) -> Self::Node
    {
        Term::from( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AsTermNode for f64
{
    type Node = Term;
    fn	AsTermNode( self) -> Self::Node
    {
        Term::from( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! TermTree {
    // 1. leaf rule
    ( @leaf $( $leaf:tt )+ ) => {
        {
            use	$crate::fresco::termtree::AsTermNode;
            ( $( $leaf )+ ).AsTermNode()
        }
    };

    // 2. Delegate to NodeTree
    ( $( $tt:tt )+ ) => {
        $crate::NodeTree!( @parse TermTree, $( $tt )+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxImportSource for Term {
    fn FetchFieldImp<'b>(&'b mut self, field: &mut FieldImp<'b>) {
        let ptr = self as *mut Self;
        *field = FieldImp::Obj(Box::new(move |key, item| {
            let term = unsafe { &mut *ptr };
            if key == "Null" {
                *term = Term::Null;
                *item = FieldImp::Null;
                return true;
            } else if key == "String" {
                *term = Term::String(String::new());
                if let Term::String(s) = term {
                    *item = FieldImp::String(s);
                }
                return true;
            } else if key == "Real" {
                *term = Term::Real(0.0);
                if let Term::Real(v) = term {
                    *item = FieldImp::F64(v);
                }
                return true;
            }
            false
        }));
    }
}
