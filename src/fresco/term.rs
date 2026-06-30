//-- term.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::{ IXFluxable, xflux::XField }, silo::U32, stalks::{ Attrib, ChildOp, DynINode, INode, IntoWorkPtr, WorkPtr } };
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

impl IXFluxable for Term
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
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

//---------------------------------------------------------------------------------------------------------------------------------

impl Term
{
    pub fn	NewString( s: String) -> Self
    {
        Self::String( s)
    }
    pub fn	NewReal( v: f64) -> Self
    {
        Self::Real( v)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

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

impl< 'a> INode< 'a> for Term
{
    fn	_Size( &self) -> U32 { U32(0) }
    fn	_At( &self, _idx: U32) -> &DynINode< 'a> { panic!("Leaf") }
    fn	Value( &self) -> Option< WorkPtr< 'a>>
    {
        Some( IntoWorkPtr::IntoWorkPtr( self.clone()))
    }
    fn	AsAny( &self) -> Option<&dyn core::any::Any>
    {
        Some( self)
    }
    fn	DocStr( &self) -> &'static str { "" }
    fn	Attrib( &self) -> Option< &Attrib> { None }
    fn	ChildOp( &self) -> ChildOp { ChildOp::None }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! TermTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_STAR   $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_STAR   $( $args)* ) };
    ( @feature_PLUS   $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_PLUS   $( $args)* ) }; 
    ( @feature_MINUS  $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_MINUS  $( $args)* ) }; 
    ( @feature_DIV    $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_DIV    $( $args)* ) }; 
    ( @feature_POW    $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_POW    $( $args)* ) }; 
    ( @feature_NEW    $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_NEW    $( $args)* ) }; 
    
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    ( @ $( $inner:tt )+ ) => {
        $crate::BiNodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::BiNodeTree!( @define [ $crate::TermTree ], Term, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
