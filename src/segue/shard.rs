use	crate::silo::U32;
use	crate::stalks::{ BinOp, DynINode, INode };
use	crate::flux::{ IXFluxSource, xflux::XField };
use	std::fmt;
use	crate::segue::{ Charset, IGrammar, Parser };
use	std::io::Read;

//---------------------------------------------------------------------------------------------------------------------------------

pub enum Shard<'a> {
    String( String),
    Charset( Charset),
    Repeat( Box<DynINode<'a>>, crate::silo::USeg),
    Action( Box<DynINode<'a>>, crate::segue::parser::ActionFn),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> Default for Shard<'a>
{
    fn	default() -> Self
    {
        Self::String( String::new())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IXFluxSource for Shard<'a>
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
                Shard::Action( child, _action) => {
                    if step == 0 {
                        *key = "Child".to_string();
                        let child_ref = &**child;
                        child_ref.ToXField( item);
                        step += 1;
                        true
                    } else if step == 1 {
                        *key = "Action".to_string();
                        *item = XField::Str( "Action");
                        step += 1;
                        true
                    } else { false }
                }
            }
        }));
    }
}
 
//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> fmt::Display for Shard<'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Self::String( s) => write!( f, "Shard( {})", s),
            Self::Charset( cs) => write!( f, "Shard( {})", cs),
            Self::Repeat( _, useg) => write!( f, "Repeat( {:?})", useg),
            Self::Action( _, _) => write!( f, "Action"),
        }
    }
}
 
//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> From< char> for Shard<'a>
{
    fn	from( c: char) -> Self
    {
        Self::String( c.to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> From< String> for Shard<'a>
{
    fn	from( s: String) -> Self
    {
        Self::String( s)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> From< &str> for Shard<'a>
{
    fn	from( s: &str) -> Self
    {
        Self::String( s.to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> From< Charset> for Shard<'a>
{
    fn	from( cs: Charset) -> Self
    {
        Self::Charset( cs)
    }
}
//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for Shard<'a>
{
    fn	_Size( &self) -> U32 {
        match self {
            Self::Repeat( _, _) | Self::Action( _, _) => U32(1),
            _ => U32(0),
        }
    }
    fn	_At( &self, idx: U32) -> &DynINode< 'a> {
        match self {
            Self::Repeat( child, _) | Self::Action( child, _) => {
                if idx.0 == 0 {
                    &**child
                } else {
                    panic!("At called on unary node with index > 0")
                }
            },
            _ => panic!("At called on Leaf"),
        }
    }
    fn	Value( &self) -> Option< crate::stalks::WorkPtr< 'a>> { None }
    fn	AsAny( &self) -> Option< &dyn core::any::Any>
    {
        None
    }
    fn	AsRawLeaf( &self) -> *const ()
    {
        self as *const _ as *const ()
    }
    fn	DocStr( &self) -> &'static str { "" }
    fn	BinOp( &self) -> BinOp {
        BinOp::None
    }
    fn	Attrib( &self) -> Option<&crate::stalks::node::Attrib> {
        None
    }

    fn	Action( &self) -> Option<*mut core::ffi::c_void> {
        match self {
            Self::Action( _, func) => {
                let func_ref: &crate::segue::parser::ActionFn = func;
                Some(func_ref as *const crate::segue::parser::ActionFn as *mut crate::segue::parser::ActionFn as *mut core::ffi::c_void)
            },
            _ => None,
        }
    }

    fn  Repeat( &self) -> Option<crate::silo::USeg> {
        match self {
            Self::Repeat( _, useg) => Some( *useg),
            _ => None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IGrammar for Shard<'a>
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        match self {
            Self::String( s) => s.as_str().Match( parser),
            Self::Charset( cs) => cs.Match( parser),
            Self::Repeat( _, _) | Self::Action( _, _) => {
                panic!("Use ParseTree + MatchNode for composite Shard nodes")
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'node> crate::segue::parser::IForgeable for Shard<'node> {
    fn Forge<'a, 'p, 's, R: std::io::Read + 'p>(
        &'a self, 
        parser: *mut Parser<'p, 's, R>
    ) -> *mut (dyn crate::segue::parser::IForge<'p, 'p, 's, R> + 'p) 
    where 's: 'p 
    {
        use crate::silo::cast::IAllocRawExt;
        let  	shardPtr = unsafe { Some(&*(self as *const _ as *const Shard<'p>)) };
        crate::segue::parser::ForgeNode {
            _Parent: None,
            _Parser: parser,
            _Kind: crate::segue::parser::ForgeKind::Leaf( shardPtr),
        }.AllocRaw()
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_CLOSURE [ $( $cb:tt)* ], $arg:ident, { $( $body:tt)* } ) => {
        Box::new( move | _raw: *mut core::ffi::c_void | {
            let $arg = unsafe { &mut *(_raw as *mut $crate::segue::Parser<'_, '_, std::io::Empty>) };
            $( $body )*
        } ) as $crate::segue::parser::ActionFn
    };

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
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $action:expr, $child:expr ) => {
        Shard::Action( Box::new( $child ), $action )
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

impl<'a> fmt::Debug for Shard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
