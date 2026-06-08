//-- shard.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::stalks::bud::Bud;
use	crate::stalks::work::{ IWork, IWorker };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct Shard
{
    _Closure: Option< fn( &dyn IWorker)>,
    _Char: Option< char>,
    _String: Option< String>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Shard
{
    fn	default() -> Self
    {
        Self {
            _Closure: Some( |_| {}),
            _Char: None,
            _String: None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Shard
{
    pub fn	New( f: fn( &dyn IWorker)) -> Self
    {
        Self {
            _Closure: Some( f),
            _Char: None,
            _String: None,
        }
    }
    pub fn	NewChar( c: char) -> Self
    {
        Self {
            _Closure: None,
            _Char: Some( c),
            _String: None,
        }
    }
    pub fn	NewString( s: String) -> Self
    {
        Self {
            _Closure: None,
            _Char: None,
            _String: Some( s),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Shard
{
    fn	DoWork( &mut self, worker: &dyn IWorker)
    {
        if let  	Some( f) = self._Closure {
            ( f)( worker);
        } else if let  	Some( c) = self._Char {
            print!( "{} ", c);
        } else if let  	Some( s) = &self._String {
            print!( "{} ", s);
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

impl crate::stalks::bud::BudOp for Shard
{
    fn	IsOpAllowed( op: crate::stalks::bud::BudBinOp) -> bool
    {
        matches!(
            op,
            crate::stalks::bud::BudBinOp::LT | crate::stalks::bud::BudBinOp::BOR | crate::stalks::bud::BudBinOp::SHL
        )
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Shard
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        if let  	Some( c) = self._Char {
            write!( f, "Shard( {})", c)
        } else if let  	Some( s) = &self._String {
            write!( f, "Shard( {})", s)
        } else {
            write!( f, "Shard")
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for fn( &dyn IWorker)
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::New( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for char
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for String
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for &'static str
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>>
    {
        Box::new( Shard::from( self))
    }
    fn	IntoBudAction( self, _act: Box<dyn Bud<Shard>>) -> Box<dyn Bud<Shard>>
    {
        Box::new( Shard::from( self))
    }

    fn	IntoBudUniOp( self, _op: crate::stalks::bud::BudUniOp) -> Box<dyn Bud<Shard>>
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< char> for Shard
{
    fn	from( c: char) -> Self
    {
        Self::NewChar( c)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< String> for Shard
{
    fn	from( s: String) -> Self
    {
        Self::NewString( s)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< &str> for Shard
{
    fn	from( s: &str) -> Self
    {
        Self::NewString( s.to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // ═══ OPT-IN FEATURES ════════════════════════════════════════════════════════════════════════════

    // Enable Unary Operators (*, +, !)
    ( @feature_STAR [ $($cb:tt)* ], $type:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $($cb)* ], $type, STAR, $l $( $r )* ) };
    ( @feature_PLUS [ $($cb:tt)* ], $type:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $($cb)* ], $type, PLUS, $l $( $r )* ) };
    ( @feature_BANG [ $($cb:tt)* ], $type:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $($cb)* ], $type, BANG, $l $( $r )* ) };

    // Enable LT (<)
    ( @feature_LT [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, LT, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, LT, $l, $( $r)+ ) };

    // Enable SHL (<<)
    ( @feature_SHL [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, SHL, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, SHL, $l, $( $r)+ ) };

    // Enable BOR (|)
    ( @feature_BOR [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, BOR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, BOR, $l, $( $r)+ ) };

    // Enable Boxet stringification
    ( @feature_BOXET [ $($cb:tt)* ], $type:ident, [ $( $inner:tt )* ] ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $type::from( stringify!( $($inner)* ).replace( " ", "" ) ) )
    };

    // Enable Closure literal (NEW)
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, | $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, || $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, move | $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, move || $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( move || $( $body)+ ) ) };

    // Enable Action Bracket [ closure ]
    ( @feature_ACTION [ $($cb:tt)* ], $type:ident, $l:literal [ $( $inner:tt )* ] ) => { $crate::BudTree!( @closure_match [ $($cb)* ], $type, $l, $( $inner )* ) };
    ( @feature_ACTION [ $($cb:tt)* ], $type:ident, ( $( $expr:tt)+ ) [ $( $inner:tt )* ] ) => { $crate::BudTree!( @closure_match [ $($cb)* ], $type, $($cb)* !( @cb [ $($cb)* ], $type, $( $expr)+ ), $( $inner )* ) };

    // ═══ FALLBACKS ══════════════════════════════════════════════════════════════════════════════════

    // Forward unhandled internal callbacks to BudTree (e.g., disallowed features like @feature_SHL)
    ( @ $( $inner:tt )+ ) => {
        $crate::BudTree!( @ $( $inner )+ )
    };

    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => { $crate::ShardTree!( @cb [ $crate::ShardTree ], Shard, $( $inner)+ ) };
}

//---------------------------------------------------------------------------------------------------------------------------------
