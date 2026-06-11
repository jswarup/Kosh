//-- bnode.rs -------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Debug, Clone, Copy, PartialEq, Eq)]
pub( crate) enum BNodeBinOp {
    LT,
    BOR,
    SHL,
    SHR,
}
impl BNodeBinOp
{
    pub( crate) fn	as_str( &self) -> &'static str {
        match self {
            BNodeBinOp::LT => "<",
            BNodeBinOp::BOR => "|",
            BNodeBinOp::SHL => "<<",
            BNodeBinOp::SHR => ">>",
        }
    }
}
#[derive( Debug, Clone, Copy, PartialEq, Eq)]
pub( crate) enum BNodeUniOp {
    STAR,
    PLUS,
    MINUS,
    BANG,
}
impl BNodeUniOp
{
    pub( crate) fn	as_str( &self) -> &'static str {
        match self {
            BNodeUniOp::STAR => "*",
            BNodeUniOp::PLUS => "+",
            BNodeUniOp::MINUS => "-",
            BNodeUniOp::BANG => "!",
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BNodeTree {
    ( $Arg:ident, $( $inner:tt )+ ) => {
        {
            #[derive( Debug, PartialEq, Clone)]
            enum ArgBNode {
                Leaf( $Arg),
                Node {
                    _BinOp: $crate::stalks::bnode::BNodeBinOp,
                    _Left: Box< ArgBNode>,
                    _Right: Box< ArgBNode>,
                }
            }
            impl ArgBNode
            {
                fn	New( value: $Arg) -> Self
                {
                    ArgBNode::Leaf( value)
                }
                fn	NewBranch( op: $crate::stalks::bnode::BNodeBinOp, left: Self, right: Self) -> Self
                {
                    ArgBNode::Node {
                        _BinOp: op,
                        _Left: Box::new( left),
                        _Right: Box::new( right),
                    }
                }
                pub( crate) fn	CountLeaves( &self) -> usize
                {
                    match self {
                        ArgBNode::Leaf( _) => 1,
                        ArgBNode::Node { _Left, _Right, .. } => _Left.CountLeaves() + _Right.CountLeaves(),
                    }
                }
            }
            $crate::BNodeTree!( @cb [ $crate::BNodeTree ], ArgBNode, $( $inner )+ )
        }
    };
    ( @cb [ $( $cb:tt)* ], $Node:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $inner)+ ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHL, $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHR, $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::LT,  $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::BOR, $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:ident << $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHL, $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:ident >> $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHR, $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:ident <  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::LT,  $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:ident |  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::BOR, $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:literal << $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHL, $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:literal >> $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHR, $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:literal <  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::LT,  $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $l:literal |  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::BOR, $Node::New( $l ), $( $cb)* !( @cb [ $( $cb)* ], $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Node:ident, $leaf:expr ) => { $Node::New( $leaf ) };
}
pub use	crate::BNodeTree;

//---------------------------------------------------------------------------------------------------------------------------------
