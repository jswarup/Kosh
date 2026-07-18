//-- shardtree.rs -------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // Leaf / node creation calls
    ( @leaf [ $s:literal ] ) => {
        <$crate::shard::Charset>::from( $s.as_bytes() )
    };
    ( @leaf $l:literal ) => {
        $l
    };
    ( @leaf ( $( $inner:tt )+ ) ) => {
        $crate::ShardTree!( $( $inner )+ )
    };
    ( @leaf Str ) => {
        $crate::shard::Str
    };
    ( @leaf $l:ident ) => {
        $l
    };
    ( @leaf $leaf:expr ) => {
        $leaf
    };

    ( @action_expr $child:expr, $work:expr ) => {
        $crate::stalks::UniNode {
            _Child: $child,
            _Op: $crate::shard::actionshard::ActionOp {
                _Action: $work,
            },
        }
    };

    ( @action $child:expr, $p:ident, $( $body:tt )+ ) => {
        $crate::stalks::UniNode {
            _Child: $child,
            _Op: $crate::shard::actionshard::ActionOp {
                _Action: $crate::shard::actionshard::Coerce( | mut $p: $crate::silo::Arr<'_, $crate::silo::U8> | { $( $body )+ } ),
            },
        }
    };

    ( @repeat $child:expr, $min:expr ) => {
        $crate::stalks::UniNode {
            _Child: $child,
            _Op: $crate::silo::USeg::NewInf( $min ),
        }
    };

    // Delegate recursive parsing to NodeTree
    ( $( $tt:tt )+ ) => {
        $crate::NodeTree!( @parse ShardTree, $( $tt )+ )
    };
}
