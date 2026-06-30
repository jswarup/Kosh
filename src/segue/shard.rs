//-- shard.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::segue::Charset;
use	crate::stalks::{ DynIWorker, IWork };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub enum Shard {
    Closure( fn( &DynIWorker< '_>)),
    Char( char),
    String( String),
    Charset( Charset),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Shard
{
    fn	default() -> Self
    {
        Self::Closure( |_| {})
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::flux::IXFluxable for Shard
{
    fn	ToXFlux< 'b>( &'b self, field: &mut crate::flux::xflux::XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	shard = self;
        *field = crate::flux::xflux::XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                match shard {
                    Shard::Closure( _) => {
                        *key = "Type".to_string();
                        *item = crate::flux::xflux::XField::Str( "Closure");
                    }
                    Shard::Char( c) => {
                        *key = "Char".to_string();
                        *item = crate::flux::xflux::XField::String( c.to_string());
                    }
                    Shard::String( s) => {
                        *key = "String".to_string();
                        *item = crate::flux::xflux::XField::Str( s);
                    }
                    Shard::Charset( c) => {
                        *key = "Charset".to_string();
                        *item = crate::flux::xflux::XField::Fluxable( c);
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

impl Shard
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

impl IWork for Shard
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

impl std::fmt::Display for Shard
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        match self {
            Self::Char( c) => write!( f, "Shard( {})", c),
            Self::String( s) => write!( f, "Shard( {})", s),
            Self::Charset( cs) => write!( f, "Shard( {})", cs),
            Self::Closure( _) => write!( f, "Shard"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< fn( &DynIWorker< '_>) > for Shard
{
    fn	from( f: fn( &DynIWorker< '_>)) -> Self
    {
        Self::New( f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< char> for Shard
{
    fn	from( c: char) -> Self
    {
        Self::Char( c)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< String> for Shard
{
    fn	from( s: String) -> Self
    {
        Self::String( s)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< &str> for Shard
{
    fn	from( s: &str) -> Self
    {
        Self::String( s.to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< Charset> for Shard
{
    fn	from( cs: Charset) -> Self
    {
        Self::Charset( cs)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_STAR   $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_STAR   $( $args)* ) };
    ( @feature_PLUS   $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_PLUS   $( $args)* ) };
    ( @feature_BANG   $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_BANG   $( $args)* ) };
    ( @feature_LT     $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_LT     $( $args)* ) };
    ( @feature_SHL    $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_SHL    $( $args)* ) };
    ( @feature_BOR    $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_BOR    $( $args)* ) };
    ( @feature_NEW    $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_NEW    $( $args)* ) };
    ( @feature_ACTION $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_ACTION $( $args)* ) };
    // ── Custom: Boxet stringification (overrides BudTree default) ─────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $s:literal ) => {
        $crate::stalks::node::IntoBiNode::< Shard, $Node >::IntoBiNode( Shard::NewCharset( $crate::segue::Charset::FromBoxet( $crate::silo::U8::FromArr( $crate::silo::Arr::from( $s.as_bytes() ) ) ) ) )
    };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    ( @ $( $inner:tt )+ ) => {
        $crate::BiNodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::BiNodeTree!( @define [ $crate::ShardTree ], Shard, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> crate::stalks::INode< 'a> for Shard
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
