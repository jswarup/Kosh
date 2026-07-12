//-- shardtree.rs -------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // --- Helper Rules ---

    // Helper to resolve leaves
    ( @resolve [ $s:literal ] ) => {
        $crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) }
    };
    ( @resolve $l:literal ) => {
        $crate::shard::leaves::StrShard { _Val: $l }
    };
    ( @resolve ( $( $inner:tt )+ ) ) => {
        $crate::ShardTree!( $( $inner )+ )
    };
    ( @resolve $l:ident ) => {
        $l
    };
    ( @resolve $leaf:expr ) => {
        $leaf
    };

    // Helper to construct binary operators
    ( @bin Sequence, $left:expr, $( $rest:tt )+ ) => {
        $crate::shard::binshard::BinShard {
            _Left: $left,
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( @bin Choice, $left:expr, $( $rest:tt )+ ) => {
        $crate::shard::binshard::BinShard {
            _Left: $left,
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // Helper to construct actions
    ( @action $child:expr, $p:ident, $( $body:tt )+ ) => {
        $crate::shard::actionshard::ActionShard {
            _Child: $child,
            _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
        }
    };

    // Helper to construct repeat shards
    ( @repeat $child:expr, $min:expr ) => {
        $crate::shard::repeatshard::RepeatShard {
            _Child: $child,
            _USeg: $crate::silo::USeg::NewInf( $min ),
        }
    };


    // --- Public Parsing Rules ---

    // 1. Repeat with action and remainder: `* leaf [ |p| body ] < rest`
    ( * $l:tt [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Sequence, $crate::ShardTree!( @action $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 0 ), $p, $( $body )+ ), $( $rest )+ )
    };
    ( * $l:tt [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Choice, $crate::ShardTree!( @action $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 0 ), $p, $( $body )+ ), $( $rest )+ )
    };
    ( + $l:tt [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Sequence, $crate::ShardTree!( @action $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 1 ), $p, $( $body )+ ), $( $rest )+ )
    };
    ( + $l:tt [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Choice, $crate::ShardTree!( @action $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 1 ), $p, $( $body )+ ), $( $rest )+ )
    };

    // 2. Repeat with action (no remainder): `* leaf [ |p| body ]`
    ( * $l:tt [ | $p:ident | $( $body:tt )+ ] ) => {
        $crate::ShardTree!( @action $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 0 ), $p, $( $body )+ )
    };
    ( + $l:tt [ | $p:ident | $( $body:tt )+ ] ) => {
        $crate::ShardTree!( @action $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 1 ), $p, $( $body )+ )
    };

    // 3. Action with remainder: `leaf [ |p| body ] < rest`
    ( $l:tt [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Sequence, $crate::ShardTree!( @action $crate::ShardTree!( @resolve $l ), $p, $( $body )+ ), $( $rest )+ )
    };
    ( $l:tt [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Choice, $crate::ShardTree!( @action $crate::ShardTree!( @resolve $l ), $p, $( $body )+ ), $( $rest )+ )
    };

    // 4. Action base case (no remainder): `leaf [ |p| body ]`
    ( $l:tt [ | $p:ident | $( $body:tt )+ ] ) => {
        $crate::ShardTree!( @action $crate::ShardTree!( @resolve $l ), $p, $( $body )+ )
    };

    // 5. Repeat with remainder: `* leaf < rest`
    ( * $l:tt < $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Sequence, $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 0 ), $( $rest )+ )
    };
    ( * $l:tt | $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Choice, $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 0 ), $( $rest )+ )
    };
    ( + $l:tt < $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Sequence, $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 1 ), $( $rest )+ )
    };
    ( + $l:tt | $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Choice, $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 1 ), $( $rest )+ )
    };

    // 6. Repeat base case (no remainder): `* leaf`
    ( * $l:tt ) => {
        $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 0 )
    };
    ( + $l:tt ) => {
        $crate::ShardTree!( @repeat $crate::ShardTree!( @resolve $l ), 1 )
    };

    // 7. Binary operators with remainder: `leaf < rest`
    ( $l:tt < $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Sequence, $crate::ShardTree!( @resolve $l ), $( $rest )+ )
    };
    ( $l:tt | $( $rest:tt )+ ) => {
        $crate::ShardTree!( @bin Choice, $crate::ShardTree!( @resolve $l ), $( $rest )+ )
    };

    // 8. Base Case: resolve the leaf/expression
    ( $l:tt ) => {
        $crate::ShardTree!( @resolve $l )
    };
}
