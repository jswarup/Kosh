//-- term.rs -------------------------------------------------------------------------------------------------------------------------
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

impl crate::flux::IXFluxable for Term
{
    fn	ToXFlux< 'b>( &'b self, field: &mut crate::flux::xflux::XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	term = self;
        *field = crate::flux::xflux::XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                match term {  
                    Term::String( s) => {
                        *key = "String".to_string();
                        *item = crate::flux::xflux::XField::Str( s);
                    } 
                    Term::Real( v) => {
                        *key = "Real".to_string();
                        *item = crate::flux::xflux::XField::F64( *v);
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

impl std::fmt::Display for Term
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
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

impl< 'a> crate::stalks::INode< 'a> for Term
{
    fn	_Size( &self) -> crate::silo::U32 { crate::silo::U32(0) }
    fn	_At( &self, _idx: crate::silo::U32) -> &crate::stalks::DynINode< 'a> { panic!("Leaf") }
    fn	Value( &self) -> Option< crate::stalks::WorkPtr< 'a>>
    {
        Some( crate::stalks::IntoWorkPtr::IntoWorkPtr( self.clone()))
    }
    fn	AsAny( &self) -> Option<&dyn core::any::Any>
    {
        Some( self)
    }
    fn	DocStr( &self) -> &'static str { "" }
    fn	Attrib( &self) -> Option< &crate::stalks::Attrib> { None }
    fn	ChildOp( &self) -> crate::stalks::ChildOp { crate::stalks::ChildOp::None }
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
