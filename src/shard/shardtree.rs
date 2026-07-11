//-- shardtree.rs -------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // 1. Postfix action with remainder: `(expr) [ |p| body ] < rest` or `(expr) [ |p| body ] | rest`
    ( ( $( $inner:tt )+ ) [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( ( $( $inner:tt )+ ) [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( $l:ident [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: $crate::ShardTree!( $l ),
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( $l:ident [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: $crate::ShardTree!( $l ),
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( $l:literal [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::leaves::StrShard { _Val: $l },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( $l:literal [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::leaves::StrShard { _Val: $l },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( [ $s:literal ] [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( [ $s:literal ] [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // 2. Postfix action base cases (no rest):
    ( ( $( $inner:tt )+ ) [ | $p:ident | $( $body:tt )+ ] ) => {
        &$crate::shard::actionshard::ActionShard {
            _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
            _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
        }
    };
    ( $l:ident [ | $p:ident | $( $body:tt )+ ] ) => {
        &$crate::shard::actionshard::ActionShard {
            _Child: $crate::ShardTree!( $l ),
            _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
        }
    };
    ( $l:literal [ | $p:ident | $( $body:tt )+ ] ) => {
        &$crate::shard::actionshard::ActionShard {
            _Child: &$crate::shard::leaves::StrShard { _Val: $l },
            _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
        }
    };
    ( [ $s:literal ] [ | $p:ident | $( $body:tt )+ ] ) => {
        &$crate::shard::actionshard::ActionShard {
            _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
            _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
        }
    };

    // 2b. Repeat with action (no remainder/rest):
    ( * $l:ident [ | $p:ident | $( $body:tt )+ ] ) => {
        &$crate::shard::actionshard::ActionShard {
            _Child: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( $l ),
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
        }
    };
    ( + $l:ident [ | $p:ident | $( $body:tt )+ ] ) => {
        &$crate::shard::actionshard::ActionShard {
            _Child: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( $l ),
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
        }
    };

    // 2c. Repeat with action and remainder:
    ( * $l:ident [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::repeatshard::RepeatShard {
                    _Child: $crate::ShardTree!( $l ),
                    _USeg: $crate::silo::USeg::NewInf( 0 ),
                },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( * $l:ident [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::repeatshard::RepeatShard {
                    _Child: $crate::ShardTree!( $l ),
                    _USeg: $crate::silo::USeg::NewInf( 0 ),
                },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( + $l:ident [ | $p:ident | $( $body:tt )+ ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::repeatshard::RepeatShard {
                    _Child: $crate::ShardTree!( $l ),
                    _USeg: $crate::silo::USeg::NewInf( 1 ),
                },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( + $l:ident [ | $p:ident | $( $body:tt )+ ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::actionshard::ActionShard {
                _Child: &$crate::shard::repeatshard::RepeatShard {
                    _Child: $crate::ShardTree!( $l ),
                    _USeg: $crate::silo::USeg::NewInf( 1 ),
                },
                _Action: $crate::shard::actionshard::Coerce( | $p: &crate::stalks::work::DynIWorker<'_> | { $( $body )+ } ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // 3. Binary operators < and | with group LHS:
    ( ( $( $inner:tt )+ ) < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: $crate::ShardTree!( ( $( $inner )+ ) ),
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( ( $( $inner:tt )+ ) | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: $crate::ShardTree!( ( $( $inner )+ ) ),
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // 4. Prefix Repeat star/plus with remainder:
    ( * [ $s:literal ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( * [ $s:literal ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( + [ $s:literal ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( + [ $s:literal ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    ( * ( $( $inner:tt )+ ) < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( * ( $( $inner:tt )+ ) | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( + ( $( $inner:tt )+ ) < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( + ( $( $inner:tt )+ ) | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    ( * $l:ident < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( $l ),
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( * $l:ident | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( $l ),
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( + $l:ident < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( $l ),
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( + $l:ident | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: $crate::ShardTree!( $l ),
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    ( * $l:literal < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::StrShard { _Val: $l },
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( * $l:literal | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::StrShard { _Val: $l },
                _USeg: $crate::silo::USeg::NewInf( 0 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };
    ( + $l:literal < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::StrShard { _Val: $l },
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( + $l:literal | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::repeatshard::RepeatShard {
                _Child: &$crate::shard::leaves::StrShard { _Val: $l },
                _USeg: $crate::silo::USeg::NewInf( 1 ),
            },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // 5. Boxet (charset) with remainder:
    ( [ $s:literal ] < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( [ $s:literal ] | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // 6. String literal with remainder:
    ( $l:literal < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::leaves::StrShard { _Val: $l },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( $l:literal | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: &$crate::shard::leaves::StrShard { _Val: $l },
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // 7. Ident with remainder:
    ( $l:ident < $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: $l,
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Sequence,
        }
    };
    ( $l:ident | $( $rest:tt )+ ) => {
        &$crate::shard::binshard::BinShard {
            _Left: $l,
            _Right: $crate::ShardTree!( $( $rest )+ ),
            _Op: $crate::shard::binshard::BinShardOp::Choice,
        }
    };

    // Base Case: Group
    ( ( $( $inner:tt )+ ) ) => {
        $crate::ShardTree!( $( $inner )+ )
    };

    // Base Case: Repeat star/plus
    ( * [ $s:literal ] ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
            _USeg: $crate::silo::USeg::NewInf( 0 ),
        }
    };
    ( + [ $s:literal ] ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) },
            _USeg: $crate::silo::USeg::NewInf( 1 ),
        }
    };

    ( * ( $( $inner:tt )+ ) ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
            _USeg: $crate::silo::USeg::NewInf( 0 ),
        }
    };
    ( + ( $( $inner:tt )+ ) ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: $crate::ShardTree!( ( $( $inner )+ ) ),
            _USeg: $crate::silo::USeg::NewInf( 1 ),
        }
    };
    ( * $l:ident ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: $crate::ShardTree!( $l ),
            _USeg: $crate::silo::USeg::NewInf( 0 ),
        }
    };
    ( + $l:ident ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: $crate::ShardTree!( $l ),
            _USeg: $crate::silo::USeg::NewInf( 1 ),
        }
    };
    ( * $l:literal ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: &$crate::shard::leaves::StrShard { _Val: $l },
            _USeg: $crate::silo::USeg::NewInf( 0 ),
        }
    };
    ( + $l:literal ) => {
        &$crate::shard::repeatshard::RepeatShard {
            _Child: &$crate::shard::leaves::StrShard { _Val: $l },
            _USeg: $crate::silo::USeg::NewInf( 1 ),
        }
    };

    // Base Case: Boxet
    ( [ $s:literal ] ) => {
        &$crate::shard::leaves::CharsetShard { _Val: <$crate::shard::Charset>::from( $s.as_bytes() ) }
    };

    // Base Case: Literal
    ( $l:literal ) => {
        &$crate::shard::leaves::StrShard { _Val: $l }
    };

    // Base Case: Expr/Ident
    ( $leaf:expr ) => {
        $leaf
    };
}
