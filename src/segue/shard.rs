//-- shard.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::segue::Charset;
use	crate::stalks::{ Bud, IWork, IWorker };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
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

impl Bud< Shard> for Shard
{
    fn	Val( &self) -> Shard
    {
        self.clone()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::BudOp for Shard
{
    fn	IsOpAllowed( op: crate::stalks::BudBinOp) -> bool
    {
        matches!( 
            op,
            crate::stalks::BudBinOp::LT | crate::stalks::BudBinOp::BOR | crate::stalks::BudBinOp::SHL
        )
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

impl crate::stalks::IntoBud< Shard> for fn( &dyn IWorker)
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::New( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::IntoBud< Shard> for char
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::IntoBud< Shard> for String
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::IntoBud< Shard> for &'static str
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
    }
    fn	IntoBudAction( self, _act: Box< dyn Bud< Shard>>) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
    }
    fn	IntoBudUniOp( self, _op: crate::stalks::BudUniOp) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
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
    ( @feature_STAR   $( $args:tt)* ) => { $crate::BudTree!( @feature_STAR   $( $args)* ) };
    ( @feature_PLUS   $( $args:tt)* ) => { $crate::BudTree!( @feature_PLUS   $( $args)* ) };
    ( @feature_BANG   $( $args:tt)* ) => { $crate::BudTree!( @feature_BANG   $( $args)* ) };
    ( @feature_LT     $( $args:tt)* ) => { $crate::BudTree!( @feature_LT     $( $args)* ) };
    ( @feature_SHL    $( $args:tt)* ) => { $crate::BudTree!( @feature_SHL    $( $args)* ) };
    ( @feature_BOR    $( $args:tt)* ) => { $crate::BudTree!( @feature_BOR    $( $args)* ) };
    ( @feature_NEW    $( $args:tt)* ) => { $crate::BudTree!( @feature_NEW    $( $args)* ) };
    ( @feature_ACTION $( $args:tt)* ) => { $crate::BudTree!( @feature_ACTION $( $args)* ) };
    // ── Custom: Boxet stringification (overrides BudTree default) ───────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $type:ident, $s:literal ) => {
        $crate::stalks::IntoBud::IntoBud( $type::NewCharset( $crate::segue::Charset::FromBoxet( $crate::silo::U8::FromArr( $crate::silo::Arr::from( $s.as_bytes() ) ) ) ) )
    };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    // Forward unhandled internal callbacks to BudTree (e.g., disallowed features like @feature_SHR)
    ( @ $( $inner:tt )+ ) => {
        $crate::BudTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => { $crate::ShardTree!( @cb [ $crate::ShardTree ], Shard, $( $inner)+ ) };
}

//---------------------------------------------------------------------------------------------------------------------------------
