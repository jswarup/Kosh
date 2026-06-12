//-- shard.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::segue::Charset;
use	crate::stalks::{ IWork, IWorker };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq)]
pub enum Shard {
    Closure( fn( &dyn IWorker)),
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

impl Shard
{
    pub fn	New( f: fn( &dyn IWorker)) -> Self
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
    fn	DoWork( &mut self, worker: &dyn IWorker)
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

impl From< fn( &dyn IWorker) > for Shard
{
    fn	from( f: fn( &dyn IWorker)) -> Self
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
macro_rules! ShardNodeTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_STAR   $( $args:tt)* ) => { $crate::BNodeTree!( @feature_STAR   $( $args)* ) };
    ( @feature_PLUS   $( $args:tt)* ) => { $crate::BNodeTree!( @feature_PLUS   $( $args)* ) };
    ( @feature_BANG   $( $args:tt)* ) => { $crate::BNodeTree!( @feature_BANG   $( $args)* ) };
    ( @feature_LT     $( $args:tt)* ) => { $crate::BNodeTree!( @feature_LT     $( $args)* ) };
    ( @feature_SHL    $( $args:tt)* ) => { $crate::BNodeTree!( @feature_SHL    $( $args)* ) };
    ( @feature_BOR    $( $args:tt)* ) => { $crate::BNodeTree!( @feature_BOR    $( $args)* ) };
    ( @feature_NEW    $( $args:tt)* ) => { $crate::BNodeTree!( @feature_NEW    $( $args)* ) };
    ( @feature_ACTION $( $args:tt)* ) => { $crate::BNodeTree!( @feature_ACTION $( $args)* ) };
    // ── Custom: Boxet stringification (overrides BNodeTree default) ─────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $s:literal ) => {
        $crate::stalks::bnode::IntoBNode::< Shard, $Node >::IntoBNode( Shard::NewCharset( $crate::segue::Charset::FromBoxet( $crate::silo::U8::FromArr( $crate::silo::Arr::from( $s.as_bytes() ) ) ) ) )
    };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    ( @ $( $inner:tt )+ ) => {
        $crate::BNodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::BNodeTree!( @define [ $crate::ShardNodeTree ], Shard, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
