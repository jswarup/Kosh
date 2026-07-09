//-- shardtree.rs -------------------------------------------------------------------------------------------------------------------

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
    ( @feature_RESOLVE_LEAF [ $( $cb:tt)* ], $Arg:ident, $val:literal ) => {
        &$crate::shard::strshard::StrShard { _Val: $val } as &$crate::stalks::DynINode
    };
    ( @feature_RESOLVE_LEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        $val
    };
    
    ( @feature_NEWLEAF [ $( $cb:tt)* ], $Arg:ident, $val:literal ) => {
        &$crate::shard::strshard::StrShard { _Val: $val } as &$crate::stalks::DynINode
    };
    ( @feature_NEWLEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        $val
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Bor, $l:expr, $r:expr ) => {
        &$crate::shard::parshard::ParShard { _Left: $l, _Right: $r } as &$crate::stalks::DynINode
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Less, $l:expr, $r:expr ) => {
        &$crate::shard::catshard::CatShard { _Left: $l, _Right: $r } as &$crate::stalks::DynINode
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, $op:ident, $l:expr, $r:expr ) => {
        compile_error!("ShardTree only supports ParShard (Bor) and CatShard (Less).")
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $action:expr, $child:expr ) => {
        &$crate::shard::actionshard::ActionShard { _Child: $child, _Action: $action } as &$crate::stalks::DynINode
    };
    ( @feature_REPEAT_STAR [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        &$crate::shard::repeatshard::RepeatShard { _Child: $child, _USeg: $crate::silo::USeg::NewInf( 0) } as &$crate::stalks::DynINode
    };
    ( @feature_REPEAT_PLUS [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        &$crate::shard::repeatshard::RepeatShard { _Child: $child, _USeg: $crate::silo::USeg::NewInf( 1) } as &$crate::stalks::DynINode
    };

    // ── Custom: Boxet stringification (overrides NodeTree default) ─────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $s:literal ) => {
        &$crate::shard::charsetshard::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) } as &$crate::stalks::DynINode
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

//---------------------------------------------------------------------------------------------------------------------------------
