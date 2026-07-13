//-- shardtree.rs -------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // Leaf / node creation calls
    ( @leaf [ $s:literal ] ) => {
        $crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) }
    };
    ( @leaf $l:literal ) => {
        $crate::shard::leaves::StrShard { _Val: $l }
    };
    ( @leaf ( $( $inner:tt )+ ) ) => {
        $crate::ShardTree!( $( $inner )+ )
    };
    ( @leaf $l:ident ) => {
        $l
    };
    ( @leaf $leaf:expr ) => {
        $leaf
    };

    ( @action $child:expr, $p:ident, $( $body:tt )+ ) => {
        $crate::stalks::UniNode {
            _Child: $child,
            _Op: $crate::shard::actionshard::ActionOp {
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
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
