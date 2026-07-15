//-- compre.rs -------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! StashC {
    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                $acc.Push($exp);
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            $acc.Push($exp);
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                $crate::StashC![@__ $acc, $exp; $($tail)+];
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            $crate::StashC![@__ $acc, $exp; $($tail)+];
        }
    );

    ($exp:expr; $($tail:tt)+) => ({
        let  	mut ret = $crate::silo::Stash::NewEmpty();
        $crate::StashC![@__ ret, $exp; $($tail)+];
        ret
    });
}
