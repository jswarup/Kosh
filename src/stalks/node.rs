//-- stalks/node.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IXFluxSource, xflux::XField };
use crate::silo::U64;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum BinOp
{
    Sum = 0,
    Prod = 1,
    Sub = 2,
    Div = 3,
    Pow = 4,
    None = 5,

    Less = 6,
    Bor = 7,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, PartialEq, Eq)]
pub struct BinNode< L, R, Op = BinOp>
{
    pub _Left: L,
    pub _Right: R,
    pub _Op: Op,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, PartialEq, Eq)]
pub struct UniNode< C, Op>
{
    pub _Child: C,
    pub _Op: Op,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INode: IXFluxSource {}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IXFluxSource + ?Sized> INode for T {}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IXFluxSource for BinNode< L, R>
where
    L: IXFluxSource,
    R: IXFluxSource,
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        let  	opVal = match node._Op {
            BinOp::Sum => 0,
            BinOp::Prod => 1,
            BinOp::Sub => 2,
            BinOp::Div => 3,
            BinOp::Pow => 4,
            BinOp::None => 5,
            BinOp::Less => 6,
            BinOp::Bor => 7,
        };
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Op".to_string();
                *item = XField::U64( U64::From(opVal));
                step += 1;
                return true;
            }
            if step == 1 {
                *key = "Left".to_string();
                node._Left.ToXField( item);
                step += 1;
                return true;
            }
            if step == 2 {
                *key = "Right".to_string();
                node._Right.ToXField( item);
                step += 1;
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! NodeTree {
    // Helper to construct binary nodes
    ( @bin $op:ident, $left:expr, $macro:ident, $( $rest:tt )+ ) => {
        $crate::stalks::BinNode {
            _Left: $left,
            _Right: $crate::$macro!( $( $rest )+ ),
            _Op: $crate::stalks::BinOp::$op,
        }
    };

    // 1. Closures with operators (for ChoreTree)
    ( @parse $macro:ident, | $arg:ident | $body:block < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::$macro!( @leaf | $arg | $body ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, | $arg:ident | $body:block | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor,  $crate::$macro!( @leaf | $arg | $body ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, move | $arg:ident | $body:block < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::$macro!( @leaf move | $arg | $body ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, move | $arg:ident | $body:block | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor,  $crate::$macro!( @leaf move | $arg | $body ), $macro, $( $rest )+ )
    };

    // 2. Repeat with action and operators (for ShardTree)
    ( @parse $macro:ident, * $l:tt [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::$macro!( @action $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 0 ), $p, $( $body )+ ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, * $l:tt [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor,  $crate::$macro!( @action $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 0 ), $p, $( $body )+ ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, + $l:tt [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::$macro!( @action $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 1 ), $p, $( $body )+ ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, + $l:tt [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor,  $crate::$macro!( @action $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 1 ), $p, $( $body )+ ), $macro, $( $rest )+ )
    };

    // 3. Repeat with action (no operators)
    ( @parse $macro:ident, * $l:tt [ | $p:ident | $( $body:tt )+ ] ) => {
        $crate::$macro!( @action $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 0 ), $p, $( $body )+ )
    };
    ( @parse $macro:ident, + $l:tt [ | $p:ident | $( $body:tt )+ ] ) => {
        $crate::$macro!( @action $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 1 ), $p, $( $body )+ )
    };

    // 4. Action with operators
    ( @parse $macro:ident, $l:tt [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::$macro!( @action $crate::$macro!( @leaf $l ), $p, $( $body )+ ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, $l:tt [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor,  $crate::$macro!( @action $crate::$macro!( @leaf $l ), $p, $( $body )+ ), $macro, $( $rest )+ )
    };

    // 5. Action base case
    ( @parse $macro:ident, $l:tt [ | $p:ident | $( $body:tt )+ ] ) => {
        $crate::$macro!( @action $crate::$macro!( @leaf $l ), $p, $( $body )+ )
    };

    // 6. Repeat with operators
    ( @parse $macro:ident, * $l:tt < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 0 ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, * $l:tt | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor,  $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 0 ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, + $l:tt < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 1 ), $macro, $( $rest )+ )
    };
    ( @parse $macro:ident, + $l:tt | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor,  $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 1 ), $macro, $( $rest )+ )
    };

    // 7. Repeat base case
    ( @parse $macro:ident, * $l:tt ) => {
        $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 0 )
    };
    ( @parse $macro:ident, + $l:tt ) => {
        $crate::$macro!( @repeat $crate::$macro!( @leaf $l ), 1 )
    };

    // 8. Infix operators with remainder
    // Group with remainder
    ( @parse $macro:ident, ( $( $inner:tt )+ ) + $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Sum,  $crate::$macro!( ( $( $inner )+ ) ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, ( $( $inner:tt )+ ) * $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Prod, $crate::$macro!( ( $( $inner )+ ) ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, ( $( $inner:tt )+ ) - $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Sub,  $crate::$macro!( ( $( $inner )+ ) ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, ( $( $inner:tt )+ ) / $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Div,  $crate::$macro!( ( $( $inner )+ ) ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, ( $( $inner:tt )+ ) ^ $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Pow,  $crate::$macro!( ( $( $inner )+ ) ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, ( $( $inner:tt )+ ) < $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Less, $crate::$macro!( ( $( $inner )+ ) ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, ( $( $inner:tt )+ ) | $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Bor,  $crate::$macro!( ( $( $inner )+ ) ), $macro, $( $rest )+ ) };

    // tt with remainder
    ( @parse $macro:ident, $l:tt + $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Sum,  $crate::$macro!( $l ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, $l:tt * $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Prod, $crate::$macro!( $l ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, $l:tt - $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Sub,  $crate::$macro!( $l ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, $l:tt / $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Div,  $crate::$macro!( $l ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, $l:tt ^ $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Pow,  $crate::$macro!( $l ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, $l:tt < $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Less, $crate::$macro!( $l ), $macro, $( $rest )+ ) };
    ( @parse $macro:ident, $l:tt | $( $rest:tt )+ ) => { $crate::NodeTree!( @bin Bor,  $crate::$macro!( $l ), $macro, $( $rest )+ ) };

    // 9. Group base case
    ( @parse $macro:ident, ( $( $inner:tt )+ ) ) => {
        $crate::$macro!( $( $inner )+ )
    };

    // 10. Fallback leaf rule
    ( @parse $macro:ident, $( $leaf:tt )+ ) => {
        $crate::$macro!( @leaf $( $leaf )+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> std::fmt::Display for BinNode< L, R>
where
    L: std::fmt::Display,
    R: std::fmt::Display,
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        match self._Op {
            BinOp::Sum => write!( f, "({} + {})", self._Left, self._Right),
            BinOp::Prod => write!( f, "({} * {})", self._Left, self._Right),
            BinOp::Sub => write!( f, "({} - {})", self._Left, self._Right),
            BinOp::Div => write!( f, "({} / {})", self._Left, self._Right),
            BinOp::Pow => write!( f, "({} ^ {})", self._Left, self._Right),
            BinOp::Less => write!( f, "({} < {})", self._Left, self._Right),
            BinOp::Bor => write!( f, "({} | {})", self._Left, self._Right),
            BinOp::None => write!( f, "({} ? {})", self._Left, self._Right),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> std::fmt::Debug for BinNode< L, R>
where
    L: std::fmt::Display,
    R: std::fmt::Display,
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        std::fmt::Display::fmt( self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
