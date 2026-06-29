//-- term.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::segue::Charset;
use	crate::stalks::{ DynIWorker, IWork };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub enum Term {
    Closure( fn( &DynIWorker< '_>)),
    Char( char),
    String( String),
    Charset( Charset),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Term
{
    fn	default() -> Self
    {
        Self::Closure( |_| {})
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::segue::IXFluxable for Term
{
    fn	ToXFlux< 'b>( &'b self, field: &mut crate::segue::xflux::XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	term = self;
        *field = crate::segue::xflux::XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                match term {
                    Term::Closure( _) => {
                        *key = "Type".to_string();
                        *item = crate::segue::xflux::XField::Str( "Closure");
                    }
                    Term::Char( c) => {
                        *key = "Char".to_string();
                        *item = crate::segue::xflux::XField::String( c.to_string());
                    }
                    Term::String( s) => {
                        *key = "String".to_string();
                        *item = crate::segue::xflux::XField::Str( s);
                    }
                    Term::Charset( c) => {
                        *key = "Charset".to_string();
                        *item = crate::segue::xflux::XField::Fluxable( c);
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
    pub fn	New( f: fn( &DynIWorker< '_>)) -> Self
    {
        Self::Closure( f)
    }
    pub fn	NewChar( c: char) -> Self
    {
        Self::Char( c)
    }
    pub fn	NewString( s: String) -> Self
    {
        Self::String( s)
    }
    pub fn	NewCharset( cs: Charset) -> Self
    {
        Self::Charset( cs)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Term
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        match self {
            Self::Closure( f) => ( f)( worker),
            Self::Char( c) => print!( "{} ", c),
            Self::String( s) => print!( "{} ", s),
            Self::Charset( cs) => print!( "{} ", cs),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Term
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        match self {
            Self::Char( c) => write!( f, "Term( {})", c),
            Self::String( s) => write!( f, "Term( {})", s),
            Self::Charset( cs) => write!( f, "Term( {})", cs),
            Self::Closure( _) => write!( f, "Term"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< fn( &DynIWorker< '_>) > for Term
{
    fn	from( f: fn( &DynIWorker< '_>)) -> Self
    {
        Self::New( f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< char> for Term
{
    fn	from( c: char) -> Self
    {
        Self::Char( c)
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

impl From< Charset> for Term
{
    fn	from( cs: Charset) -> Self
    {
        Self::Charset( cs)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! TermTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_STAR   $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_STAR   $( $args)* ) };
    ( @feature_PLUS   $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_PLUS   $( $args)* ) }; 
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

impl< 'a> crate::stalks::INode< 'a> for Term
{
    fn	_Size( &self) -> crate::silo::U32 { crate::silo::U32(0) }
    fn	_At( &self, _idx: crate::silo::U32) -> &crate::stalks::DynINode< 'a> { panic!("Leaf") }
    fn	Value( &self) -> Option< crate::stalks::WorkPtr< 'a>>
    {
        Some( crate::stalks::IntoWorkPtr::IntoWorkPtr( self.clone()))
    }
    fn	DocStr( &self) -> &'static str { "" }
    fn	Attrib( &self) -> Option< &crate::stalks::Attrib> { None }
    fn	ChildOp( &self) -> crate::stalks::ChildOp { crate::stalks::ChildOp::None }
}

//---------------------------------------------------------------------------------------------------------------------------------
