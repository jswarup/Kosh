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
    fn	BinOp( &self) -> Option< BNodeBinOp>
    {
        None
    }
    fn	UniOp( &self) -> Option< BNodeUniOp>
    {
        None
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
pub trait IntoBNode< T, N > {
    fn	IntoBNode( self ) -> N;
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

    ( @feature_STAR  [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BNodeTree!( @uni [ $( $cb)* ], $Arg, $Node, STAR, $l $( $r )* ) };
    ( @feature_PLUS  [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BNodeTree!( @uni [ $( $cb)* ], $Arg, $Node, PLUS, $l $( $r )* ) };
    ( @feature_MINUS [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BNodeTree!( @uni [ $( $cb)* ], $Arg, $Node, MINUS, $l $( $r )* ) };
    ( @feature_BANG  [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BNodeTree!( @uni [ $( $cb)* ], $Arg, $Node, BANG, $l $( $r )* ) };

    ( @wrap $leaf:expr ) => {
        Into::into( $leaf )
    };
    ( @define [ $( $cb:tt )* ], $Arg:ident, $( $inner:tt )+ ) => {
        {
            paste::paste! {
                #[derive( Debug, PartialEq, Clone)]
                enum [<$Arg BNode>] {
                    Leaf( $Arg),
                    Node {
                        _BinOp: $crate::stalks::bnode::BNodeBinOp,
                        _Left: Box< [<$Arg BNode>]>,
                        _Right: Box< [<$Arg BNode>]>,
                    },
                    UniNode {
                        _UniOp: $crate::stalks::bnode::BNodeUniOp,
                        _Child: Box< [<$Arg BNode>]>,
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
                    fn	NewUni( op: $crate::stalks::bnode::BNodeUniOp, child: Self) -> Self
                    {
                        [<$Arg BNode>]::UniNode {
                            _UniOp: op,
                            _Child: Box::new( child),
                        }
                    }
                    pub fn	CountLeaves( &self) -> usize
                    {
                        match self {
                            [<$Arg BNode>]::Leaf( _) => 1,
                            [<$Arg BNode>]::Node { _Left, _Right, .. } => _Left.CountLeaves() + _Right.CountLeaves(),
                            [<$Arg BNode>]::UniNode { _Child, .. } => _Child.CountLeaves(),
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
                            [<$Arg BNode>]::Node { _Left, .. } => Some( &**_Left),
                            [<$Arg BNode>]::UniNode { _Child, .. } => Some( &**_Child),
                            _ => None,
                        }
                    }
                    fn	Right( &self) -> Option< &dyn $crate::stalks::bnode::IBNode< $Arg>>
                    {
                        match self {
                            [<$Arg BNode>]::Node { _Right, .. } => Some( &**_Right),
                            _ => None,
                        }
                    }
                    fn	BinOp( &self) -> Option< $crate::stalks::bnode::BNodeBinOp>
                    {
                        match self {
                            [<$Arg BNode>]::Node { _BinOp, .. } => Some( *_BinOp),
                            _ => None,
                        }
                    }
                    fn	UniOp( &self) -> Option< $crate::stalks::bnode::BNodeUniOp>
                    {
                        match self {
                            [<$Arg BNode>]::UniNode { _UniOp, .. } => Some( *_UniOp),
                            _ => None,
                        }
                    }
                }
                impl< I > $crate::stalks::bnode::IntoBNode< $Arg, [<$Arg BNode>]> for I
                where
                    I: Into< $Arg >,
                {
                    fn	IntoBNode( self) -> [<$Arg BNode>] {
                        [<$Arg BNode>]::New( self.into() )
                    }
                }
                impl $crate::stalks::bnode::IntoBNode< $Arg, [<$Arg BNode>]> for [<$Arg BNode>] {
                    fn	IntoBNode( self) -> [<$Arg BNode>] {
                        self
                    }
                }
                $crate::BNodeTree!( @cb [ $( $cb)* ], $Arg, [<$Arg BNode>], $( $inner )+ )
            }
        }
    };
    ( $Arg:ident, $( $inner:tt )+ ) => {
        $crate::BNodeTree!( @define [ $crate::BNodeTree ], $Arg, $( $inner )+ )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $inner)+ ) };

    // ── Unary operators ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, * $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, + $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, - $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ! $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_BANG [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    
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
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $leaf:expr ) => {
        $crate::stalks::bnode::IntoBNode::< $Arg, $Node >::IntoBNode( $leaf )
    };

    // @uni : unary - OP rhs
    ( @uni [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, $l:tt $( $r:tt)* ) => {
        $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, ( $Node::NewUni( $crate::stalks::bnode::BNodeUniOp::$op, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $l ) ) ) $( $r)* )
    };

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
    ( @feature_STAR  $( $args:tt )* ) => { compile_error!( "Unary STAR (*) is not enabled for this tree"); };
    ( @feature_PLUS  $( $args:tt )* ) => { compile_error!( "Unary PLUS (+) is not enabled for this tree"); };
    ( @feature_MINUS $( $args:tt )* ) => { compile_error!( "Unary MINUS (-) is not enabled for this tree"); };
    ( @feature_BANG  $( $args:tt )* ) => { compile_error!( "Unary BANG (!) is not enabled for this tree"); };
}
pub use	crate::BNodeTree;

//---------------------------------------------------------------------------------------------------------------------------------
