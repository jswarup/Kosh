//-- bnode.rs -------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNodeBinOp {
    LT,
    BOR,
    SHL,
    SHR,
}
impl BNodeBinOp
{
    pub fn	as_str( &self) -> &'static str {
        match self {
            BNodeBinOp::LT => "<",
            BNodeBinOp::BOR => "|",
            BNodeBinOp::SHL => "<<",
            BNodeBinOp::SHR => ">>",
        }
    }
}
#[derive( Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNodeUniOp {
    STAR,
    PLUS,
    MINUS,
    BANG,
}
impl BNodeUniOp
{
    pub fn	as_str( &self) -> &'static str {
        match self {
            BNodeUniOp::STAR => "*",
            BNodeUniOp::PLUS => "+",
            BNodeUniOp::MINUS => "-",
            BNodeUniOp::BANG => "!",
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IBNode< T> {
    fn	Val( &self) -> Option< &T>;

    fn	Left( &self) -> Option< &dyn IBNode< T>>
    {
        None
    }
    fn	Right( &self) -> Option< &dyn IBNode< T>>
    {
        None
    }
    fn	Op( &self) -> &str
    {
        ""
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Debug, PartialEq, Eq)]
pub enum TraversalEvent {
    Entry,
    Exit,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> dyn IBNode< T> + '_
{
    pub fn	TraverseDFS( &self, f: &mut dyn FnMut( &dyn IBNode< T>, TraversalEvent))
    {
        let  	isLeaf = self.Left().is_none() && self.Right().is_none();
        f( self, TraversalEvent::Entry);
        if let  	Some( left) = self.Left() {
            left.TraverseDFS( f);
        }
        if let  	Some( right) = self.Right() {
            right.TraverseDFS( f);
        }
        if !isLeaf {
            f( self, TraversalEvent::Exit);
        }
    }
}
//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BNodeTree {
    // ---- FEATURE OPT-INS FOR BNodeTree ITSELF ----------------------------------------------------------------------------
    // BNodeTree explicitly opts in to all features by delegating back to its own builders.
    ( @feature_SHL [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, SHL, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, SHL, $l, $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, SHR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, SHR, $l, $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, LT, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, LT, $l, $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BNodeTree!( @bg [ $( $cb)* ], $Arg, $Node, BOR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BNodeTree!( @bl [ $( $cb)* ], $Arg, $Node, BOR, $l, $( $r)+ ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $Node::New( $Arg::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $Node::New( $Arg::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $Node::New( $Arg::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $Node::New( $Arg::New( move || $( $body)+ ) ) };
    ( @wrap $leaf:expr ) => {
        Into::into( $leaf )
    };
    ( $Arg:ident, $( $inner:tt )+ ) => {
        {
            paste::paste! {
                #[derive( Debug, PartialEq, Clone)]
                enum [<$Arg BNode>] {
                    Leaf( $Arg),
                    Node {
                        _BinOp: $crate::stalks::bnode::BNodeBinOp,
                        _Left: Box< [<$Arg BNode>]>,
                        _Right: Box< [<$Arg BNode>]>,
                    }
                }
                impl [<$Arg BNode>]
                {
                    fn	New( value: $Arg) -> Self
                    {
                        [<$Arg BNode>]::Leaf( value)
                    }
                    fn	NewBranch( op: $crate::stalks::bnode::BNodeBinOp, left: Self, right: Self) -> Self
                    {
                        [<$Arg BNode>]::Node {
                            _BinOp: op,
                            _Left: Box::new( left),
                            _Right: Box::new( right),
                        }
                    }
                    pub fn	CountLeaves( &self) -> usize
                    {
                        match self {
                            [<$Arg BNode>]::Leaf( _) => 1,
                            [<$Arg BNode>]::Node { _Left, _Right, .. } => _Left.CountLeaves() + _Right.CountLeaves(),
                        }
                    }
                }
                impl $crate::stalks::bnode::IBNode< $Arg> for [<$Arg BNode>]
                {
                    fn	Val( &self) -> Option< &$Arg>
                    {
                        match self {
                            [<$Arg BNode>]::Leaf( value) => Some( value),
                            _ => None,
                        }
                    }
                    fn	Left( &self) -> Option< &dyn $crate::stalks::bnode::IBNode< $Arg>>
                    {
                        match self {
                            [<$Arg BNode>]::Leaf( _) => None,
                            [<$Arg BNode>]::Node { _Left, .. } => Some( &**_Left),
                        }
                    }
                    fn	Right( &self) -> Option< &dyn $crate::stalks::bnode::IBNode< $Arg>>
                    {
                        match self {
                            [<$Arg BNode>]::Leaf( _) => None,
                            [<$Arg BNode>]::Node { _Right, .. } => Some( &**_Right),
                        }
                    }
                    fn	Op( &self) -> &str
                    {
                        match self {
                            [<$Arg BNode>]::Leaf( _) => "",
                            [<$Arg BNode>]::Node { _BinOp, .. } => _BinOp.as_str(),
                        }
                    }
                }
                $crate::BNodeTree!( @cb [ $crate::BNodeTree ], $Arg, [<$Arg BNode>], $( $inner )+ )
            }
        }
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $inner)+ ) };
    
    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    
    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    
    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, || $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move || $( $body)+ ) };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $leaf:expr ) => { $Node::New( $( $cb)* !( @wrap $leaf ) ) };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------
    // @bg : binary — (group) OP rhs
    ( @bg [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        $Node::NewBranch( 
            $crate::stalks::bnode::BNodeBinOp::$op,
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };
    // @bl : binary — leaf OP rhs
    ( @bl [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        $Node::NewBranch( 
            $crate::stalks::bnode::BNodeBinOp::$op,
            $Node::New( $( $cb)* !( @wrap $l ) ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_SHL $( $args:tt )* ) => { compile_error!( "Binary SHL (<<) is not enabled for this tree"); };
    ( @feature_SHR $( $args:tt )* ) => { compile_error!( "Binary SHR (>>) is not enabled for this tree"); };
    ( @feature_LT  $( $args:tt )* ) => { compile_error!( "Binary LT (<) is not enabled for this tree"); };
    ( @feature_BOR $( $args:tt )* ) => { compile_error!( "Binary BOR (|) is not enabled for this tree"); };
    ( @feature_NEW $( $args:tt )* ) => { compile_error!( "Closure literal is not enabled for this tree"); };
}
pub use	crate::BNodeTree;

//---------------------------------------------------------------------------------------------------------------------------------
