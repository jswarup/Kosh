//-- shardtree.rs -------------------------------------------------------------------------------------------------------------------
use crate::stalks::DynINode;
use crate::shard::Charset;

pub trait IntoDynNode<'a> {
    fn IntoDynNode(self) -> Box<DynINode<'a>>;
}

impl<'a> IntoDynNode<'a> for &'static str {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self.to_string())
    }
}

impl<'a> IntoDynNode<'a> for String {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self)
    }
}

impl<'a> IntoDynNode<'a> for char {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self.to_string())
    }
}

impl<'a> IntoDynNode<'a> for Charset {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self)
    }
}

impl<'a> IntoDynNode<'a> for Box<DynINode<'a>> {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        self
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
        $crate::shard::shardtree::IntoDynNode::IntoDynNode($val)
    };
    ( @feature_NEWLEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        $crate::shard::shardtree::IntoDynNode::IntoDynNode($val)
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Bor, $l:expr, $r:expr ) => {
        Box::new($crate::shard::parshard::ParShard { _Left: $l, _Right: $r }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Less, $l:expr, $r:expr ) => {
        Box::new($crate::shard::catshard::CatShard { _Left: $l, _Right: $r }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, $op:ident, $l:expr, $r:expr ) => {
        compile_error!("ShardTree only supports ParShard (Bor) and CatShard (Less).")
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $action:expr, $child:expr ) => {
        Box::new($crate::shard::actionshard::ActionShard { _Child: $child, _Action: $action }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_REPEAT_STAR [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        Box::new($crate::shard::repeatshard::RepeatShard { _Child: $child, _USeg: $crate::silo::USeg::NewInf( 0) }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_REPEAT_PLUS [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        Box::new($crate::shard::repeatshard::RepeatShard { _Child: $child, _USeg: $crate::silo::USeg::NewInf( 1) }) as Box<$crate::stalks::DynINode<'static>>
    };

    // ── Custom: Boxet stringification (overrides NodeTree default) ─────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $s:literal ) => {
        Box::new(<$crate::shard::Charset>::from( $s.as_bytes() )) as Box<$crate::stalks::DynINode<'static>>
    };

    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    ( @ $( $inner:tt )+ ) => {
        $crate::NodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::NodeTree!( @define [ $crate::ShardTree ], DynINode, $( $inner)+ )
    };
}
