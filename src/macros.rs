#[macro_export]
macro_rules! vlist {
    () => { $crate::Nil };
    (...$rest:expr) => { $rest };
    ($a:expr) => { vlist![$a,] };
    ($a:expr, $($tok:tt)*) => {
        $crate::Cons(
            $a,
            vlist![$($tok)*],
        )
    };
}

#[macro_export]
macro_rules! vlist_pat {
    () => { $crate::Nil };
    (...) => { _ };
    (...$rest:pat) => { $rest };
    ($a:pat) => { vlist_pat![$a,] };
    ($a:pat, $($tok:tt)*) => {
        $crate::Cons(
            $a,
            vlist_pat![$($tok)*],
        )
    };
}

#[macro_export]
macro_rules! VList {
    () => { $crate::Nil };
    (...$Rest:ty) => { $Rest };
    ($A:ty) => { VList![$A,] };
    ($A:ty, $($tok:tt)*) => {
        $crate::Cons<$A, VList![$($tok)*]>
    };
}
