use	crate::silo::U32;
use	crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use	crate::flux::{ IXFluxSource, xflux::XField };
use	std::fmt;
use	crate::segue::{ Charset, IGrammar, Parser };
use	std::any::Any;
use	std::io::Read;

//---------------------------------------------------------------------------------------------------------------------------------

pub enum Shard {
    String( String),
    Charset( Charset),
    Repeat( Box<Shard>, crate::silo::USeg),
    UniNode( crate::stalks::node::Attrib, Box<Shard>),
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

impl IXFluxSource for Shard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	shard = self;
        *field = XField::Obj( Box::new( move |key, item| {
            match shard {
                Shard::String( s) => {
                    if step == 0 {
                        *key = "String".to_string();
                        *item = XField::Str( s);
                        step += 1;
                        true
                    } else { false }
                }
                Shard::Charset( c) => {
                    if step == 0 {
                        *key = "Charset".to_string();
                        *item = XField::FluxSource( c);
                        step += 1;
                        true
                    } else { false }
                }
                Shard::Repeat( child, useg) => {
                    if step == 0 {
                        *key = "Child".to_string();
                        let child_ref = &**child;
                        child_ref.ToXField( item);
                        step += 1;
                        true
                    } else if step == 1 {
                        *key = "Repeat".to_string();
                        *item = XField::FluxSource( useg);
                        step += 1;
                        true
                    } else { false }
                }
                Shard::UniNode( attrib, child) => {
                    if step == 0 {
                        *key = "Child".to_string();
                        let child_ref = &**child;
                        child_ref.ToXField( item);
                        step += 1;
                        true
                    } else if step == 1 {
                        *key = "Attrib".to_string();
                        *item = XField::FluxSource( attrib);
                        step += 1;
                        true
                    } else { false }
                }
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
            Self::Repeat( _, useg) => write!( f, "Repeat( {:?})", useg),
            Self::UniNode( _attrib, _) => write!( f, "UniNode"),
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
    fn	_Size( &self) -> U32 {
        match self {
            Self::Repeat( _, _) | Self::UniNode( _, _) => U32(1),
            _ => U32(0),
        }
    }
    fn	_At( &self, idx: U32) -> &DynINode< 'a> {
        match self {
            Self::Repeat( child, _) | Self::UniNode( _, child) => {
                if idx.0 == 0 {
                    &**child
                } else {
                    panic!("At called on unary node with index > 0")
                }
            },
            _ => panic!("At called on Leaf"),
        }
    }
    fn	Value( &self) -> Option< WorkPtr< 'a>> { None }
    fn	AsAny( &self) -> Option< &dyn Any>
    {
        Some( self)
    }
    fn	DocStr( &self) -> &'static str { "" }
    fn	BinOp( &self) -> BinOp {
        BinOp::None
    }
    fn	Attrib( &self) -> Option<&crate::stalks::node::Attrib> {
        match self {
            Self::UniNode( attrib, _) => Some( attrib),
            _ => None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Shard
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        match self {
            Self::String( s) => s.as_str().Match( parser),
            Self::Charset( cs) => cs.Match( parser),
            Self::Repeat( _child, _) => false, // TODO: Implement Repeat matching
            Self::UniNode( _, _child) => false, // TODO: Implement UniNode matching
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::segue::parser::IForgeable for Shard {
    fn Forge<'a, 'p, 's, R: std::io::Read + 'p>(
        &'a self, 
        parser: *mut Parser<'p, 's, R>
    ) -> *mut (dyn crate::segue::parser::IForge<'p, 'p, 's, R> + 'p) 
    where 's: 'p 
    {
        use crate::silo::cast::{IAllocRawExt, ICastExt};
        let shardPtr = Some(self).Cast::<Option<&'static Shard>>();
        crate::segue::parser::LeafForge {
            _Parent: None,
            _Parser: parser,
            _Shard: shardPtr,
        }.AllocRaw()
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

    // ── Shard AST Hooks (overrides NodeTree default) ──────────────────────────────────────────────
    ( @feature_RESOLVE_LEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        Shard::from( $val )
    };
    ( @feature_NEWLEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        Shard::from( $val )
    };
    ( @feature_NEWUNINODE [ $( $cb:tt)* ], $Arg:ident, $attrib:expr, $child:expr ) => {
        Shard::UniNode( $attrib, Box::new( $child ) )
    };
    ( @feature_NEWUNINODE [ $( $cb:tt)* ], $Arg:ident, $attrib:expr, $child:expr ) => {
        Shard::UniNode( $attrib, Box::new( $child ) )
    };
    ( @feature_REPEAT_STAR [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        Shard::Repeat( Box::new( $child ), $crate::silo::USeg::NewInf( 0) )
    };
    ( @feature_REPEAT_PLUS [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        Shard::Repeat( Box::new( $child ), $crate::silo::USeg::NewInf( 1) )
    };

    // ── Custom: Boxet stringification (overrides BudTree default) ─────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $s:literal ) => {
        Shard::from( <$crate::segue::Charset>::from( $s.as_bytes() ) )
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

impl fmt::Debug for Shard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
