use	crate::silo::U32;
use	crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use	crate::stalks::work::DynIWork;
use	crate::flux::{ IXFluxSource, xflux::XField };
use	std::fmt;
use	crate::segue::{ Charset, IGrammar, Parser };


//---------------------------------------------------------------------------------------------------------------------------------

pub enum Shard {
    Leaf( Box<DynINode<'static>>),
    Repeat( Box<Shard>, crate::silo::USeg),
    Action( Box<Shard>, Box<DynIWork<'static>>),
    ParNode( Box<Shard>, Box<Shard>),
    CatNode( Box<Shard>, Box<Shard>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Shard
{
    pub fn	NewParNode( left: Self, right: Self) -> Self
    {
        Shard::ParNode( Box::new( left), Box::new( right))
    }
    pub fn	NewCatNode( left: Self, right: Self) -> Self
    {
        Shard::CatNode( Box::new( left), Box::new( right))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Shard
{
    fn	default() -> Self
    {
        Self::Leaf( Box::new( String::new()))
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
                Shard::Leaf( val) => {
                    if step == 0 {
                        *key = "Leaf".to_string();
                        val.ToXField( item);
                        step += 1;
                        true
                    } else { false }
                }
                Shard::Repeat( child, useg) => {
                    if step == 0 {
                        *key = "Child".to_string();
                        child.ToXField( item);
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
                        child.ToXField( item);
                        step += 1;
                        true
                    } else if step == 1 {
                        *key = "Action".to_string();
                        *item = XField::Str( "Action");
                        step += 1;
                        true
                    } else { false }
                }
                Shard::ParNode( left, right) | Shard::CatNode( left, right) => {
                    if step == 0 {
                        *key = "Left".to_string();
                        left.ToXField( item);
                        step += 1;
                        true
                    } else if step == 1 {
                        *key = "Right".to_string();
                        right.ToXField( item);
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
            Self::Leaf( val) => write!( f, "Shard( {})", val.DocStr()),
            Self::Repeat( _, useg) => write!( f, "Repeat( {:?})", useg),
            Self::Action( _, _) => write!( f, "Action"),
            Self::ParNode( _, _) => write!( f, "ParNode"),
            Self::CatNode( _, _) => write!( f, "CatNode"),
        }
    }
}
 
//---------------------------------------------------------------------------------------------------------------------------------

impl From< char> for Shard
{
    fn	from( c: char) -> Self
    {
        Self::Leaf( Box::new( c.to_string()))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< String> for Shard
{
    fn	from( s: String) -> Self
    {
        Self::Leaf( Box::new( s))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< &str> for Shard
{
    fn	from( s: &str) -> Self
    {
        Self::Leaf( Box::new( s.to_string()))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< Charset> for Shard
{
    fn	from( cs: Charset) -> Self
    {
        Self::Leaf( Box::new( cs))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for Shard
{
    fn	_Size( &self) -> U32 {
        match self {
            Self::Repeat( _, _) | Self::Action( _, _) => U32(1),
            Self::ParNode( _, _) | Self::CatNode( _, _) => U32(2),
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
            Self::ParNode( left, right) | Self::CatNode( left, right) => {
                match idx.0 {
                    0 => &**left,
                    1 => &**right,
                    _ => panic!("At called on BinNode with index > 1"),
                }
            },
            _ => panic!("At called on Leaf"),
        }
    }
    fn	Value( &self) -> Option< WorkPtr< 'a>> { None }
    fn	AsRawLeaf( &self) -> *const ()
    {
        self as *const _ as *const ()
    }
    fn	DocStr( &self) -> &'static str { "" }
    fn	BinOp( &self) -> BinOp {
        match self {
            Self::ParNode( _, _) => BinOp::Bor,
            Self::CatNode( _, _) => BinOp::Less,
            _ => BinOp::None,
        }
    }

    fn	Action( &self) -> Option<*const DynIWork<'static>> {
        match self {
            Self::Action( _, action) => Some( action.as_ref() as *const _),
            _ => None,
        }
    }

    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut crate::segue::Parser<'_>) };
        self.Match(p, crate::silo::U32(marker)).map(|u| u.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Shard {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        match self {
            Self::Leaf( val) => val.MatchGrammar(parser as *mut _ as *mut (), marker.0).map(crate::silo::U32),
            Self::Repeat( child, useg) => {
                let  	mut count = U32( 0);
                let  	first = useg.First();
                let  	last = if useg.IsEmpty() { U32( std::u32::MAX) } else { useg.Last() };
                let  	mut currMark = marker;

                while count < last {
                    if let Some( newMark) = child.Match( parser, currMark) {
                        if newMark == currMark {
                            count += U32( 1);
                            break;
                        }
                        currMark = newMark;
                        count += U32( 1);
                    } else {
                        break;
                    }
                }

                if count >= first {
                    Some(currMark)
                } else {
                    None
                }
            },
            Self::Action( child, action) => {
                if let Some( childMark) = child.Match( parser, marker) {
                    let  	actionPtr = &**action as *const DynIWork<'static>;
                    #[allow(invalid_reference_casting)]
                    let  	actionMut = unsafe { &mut *(actionPtr as *mut DynIWork<'static>) };
                    actionMut.DoWork( parser);
                    return Some( childMark);
                }
                None
            },
            Self::CatNode( left, right) => {
                if let Some( leftMark) = left.Match( parser, marker) {
                    if let Some( rightMark) = right.Match( parser, leftMark) {
                        return Some( rightMark);
                    }
                }
                None
            },
            Self::ParNode( left, right) => {
                if let Some( leftMark) = left.Match( parser, marker) {
                    return Some( leftMark);
                }
                if let Some( rightMark) = right.Match( parser, marker) {
                    return Some( rightMark);
                }
                None
            },
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

    ( @feature_NEW    $( $args:tt)* ) => { $crate::NodeTree!( @feature_NEW    $( $args)* ) };
    ( @feature_PostBoxet $( $args:tt)* ) => { $crate::NodeTree!( @feature_PostBoxet $( $args)* ) };

    // ── Shard AST Hooks (overrides NodeTree default) ──────────────────────────────────────────────
    ( @feature_RESOLVE_LEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        Shard::from( $val )
    };
    ( @feature_NEWLEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        Shard::from( $val )
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Bor, $l:expr, $r:expr ) => {
        $crate::segue::shard::Shard::NewParNode( $l, $r )
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Less, $l:expr, $r:expr ) => {
        $crate::segue::shard::Shard::NewCatNode( $l, $r )
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, $op:ident, $l:expr, $r:expr ) => {
        compile_error!("Shard only supports ParNode (Bor) and CatNode (Less).")
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

    // ── Custom: Boxet stringification (overrides NodeTree default) ─────────────────────────────────
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

//---------------------------------------------------------------------------------------------------------------------------------
