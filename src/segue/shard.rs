use	crate::silo::U32;
use	crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use	crate::flux::{ IXFluxable, xflux::XField };
use	std::fmt;
use	crate::segue::{ Charset, IGrammar, Parser };
use	std::any::Any;
use	std::io::Read;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub enum Shard {
    String( String),
    Charset( Charset),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Shard
{
    fn	default() -> Self
    {
        Self::String( String::new())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for Shard
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	shard = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                match shard {
                    Shard::String( s) => {
                        *key = "String".to_string();
                        *item = XField::Str( s);
                    }
                    Shard::Charset( c) => {
                        *key = "Charset".to_string();
                        *item = XField::Fluxable( c);
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

impl fmt::Display for Shard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Self::String( s) => write!( f, "Shard( {})", s),
            Self::Charset( cs) => write!( f, "Shard( {})", cs),
        }
    }
}
 
//---------------------------------------------------------------------------------------------------------------------------------

impl From< char> for Shard
{
    fn	from( c: char) -> Self
    {
        Self::String( c.to_string())
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

impl< 'a> INode< 'a> for Shard
{
    fn	_Size( &self) -> U32 { U32(0) }
    fn	_At( &self, _idx: U32) -> &DynINode< 'a> { panic!("Leaf") }
    fn	Value( &self) -> Option< WorkPtr< 'a>> { None }
    fn	AsAny( &self) -> Option< &dyn Any>
    {
        Some( self)
    }
    fn	DocStr( &self) -> &'static str { "" }
    fn	BinOp( &self) -> BinOp { BinOp::None }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Shard
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        match self {
            Self::String( s) => s.as_str().Match( parser),
            Self::Charset( cs) => cs.Match( parser),
        }
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_STAR   $( $args:tt)* ) => { $crate::NodeTree!( @feature_STAR   $( $args)* ) };
    ( @feature_PLUS   $( $args:tt)* ) => { $crate::NodeTree!( @feature_PLUS   $( $args)* ) };

    ( @feature_LT     $( $args:tt)* ) => { $crate::NodeTree!( @feature_LT     $( $args)* ) };
    ( @feature_SHL    $( $args:tt)* ) => { $crate::NodeTree!( @feature_SHL    $( $args)* ) };
    ( @feature_BOR    $( $args:tt)* ) => { $crate::NodeTree!( @feature_BOR    $( $args)* ) };
    ( @feature_NEW    $( $args:tt)* ) => { $crate::NodeTree!( @feature_NEW    $( $args)* ) };
    ( @feature_PostBoxet $( $args:tt)* ) => { $crate::NodeTree!( @feature_PostBoxet $( $args)* ) };

    // ── Custom: Boxet stringification (overrides BudTree default) ─────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $s:literal ) => {
        $crate::stalks::node::IntoNodule::< Shard, $Node >::IntoNodule( Shard::from( <$crate::segue::Charset>::from( <$crate::silo::Arr<$crate::silo::U8>>::from( $crate::silo::Arr::from( $s.as_bytes() ) ) ) ) )
    };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    ( @ $( $inner:tt )+ ) => {
        $crate::NodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::NodeTree!( @define [ $crate::ShardTree ], Shard, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
