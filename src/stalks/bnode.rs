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
    pub fn	as_str( &self) -> &'static str
    {
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
    pub fn	as_str( &self) -> &'static str
    {
        match self {
            BNodeUniOp::STAR => "*",
            BNodeUniOp::PLUS => "+",
            BNodeUniOp::MINUS => "-",
            BNodeUniOp::BANG => "!",
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Debug, PartialEq, Clone)]
pub enum BNode< T> {
    Leaf( T),
    Node {
        _BinOp : BNodeBinOp,
        _Left: Box< BNode< T>>,
        _Right: Box< BNode< T>>,
    },
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> BNode< T>
{

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	NewLeaf ( value: T) -> Self
    {
        BNode::Leaf( value)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New( op: BNodeBinOp,  _Left: BNode< T>, _Right: BNode< T>) -> Self
    {
        BNode::Node {
            _BinOp: op,
            _Left: Box::new( _Left),
            _Right: Box::new( _Right),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	CountLeaves( &self) -> usize
    {
        match self {
            BNode::Leaf( _ ) => 1,
            BNode::Node { _BinOp, _Left, _Right } => {
                _Left.CountLeaves() + _Right.CountLeaves()
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BNodeTree {
    ( @cb [ $( $cb:tt)* ], ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $( $inner)+ ) };
    ( @cb [ $( $cb:tt)* ], ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::SHL, $( $cb)* !( @cb [ $( $cb)* ], $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::SHR, $( $cb)* !( @cb [ $( $cb)* ], $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::LT,  $( $cb)* !( @cb [ $( $cb)* ], $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::BOR, $( $cb)* !( @cb [ $( $cb)* ], $( $l)+ ), $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $l:ident << $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::SHL, $l, $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $l:ident >> $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::SHR, $l, $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $l:ident <  $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::LT,  $l, $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $l:ident |  $( $r:tt)+ ) => { $crate::stalks::BNode::New( $crate::stalks::bnode::BNodeBinOp::BOR, $l, $( $cb)* !( @cb [ $( $cb)* ], $( $r)+ ) ) };
    ( @cb [ $( $cb:tt)* ], $leaf:expr ) => { $leaf };
    ( $( $inner:tt )+ ) => {
        $crate::BNodeTree!( @cb [ $crate::BNodeTree ], $( $inner )+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
