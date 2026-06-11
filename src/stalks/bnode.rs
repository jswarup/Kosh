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

#[macro_export]
macro_rules! BNodeTree {
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
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHL, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHR, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::LT,  $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::BOR, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident << $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHL, $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident >> $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHR, $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident <  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::LT,  $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident |  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::BOR, $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal << $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHL, $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal >> $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::SHR, $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal <  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::LT,  $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal |  $( $r:tt)+ ) => { $Node::NewBranch( $crate::stalks::bnode::BNodeBinOp::BOR, $Node::New( $( $cb)* !( @wrap $l ) ), $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $leaf:expr ) => { $Node::New( $( $cb)* !( @wrap $leaf ) ) };
}
pub use	crate::BNodeTree;

//---------------------------------------------------------------------------------------------------------------------------------
