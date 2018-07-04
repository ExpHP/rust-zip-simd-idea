use ::std::ops;
use ::std::cmp;
use ::std::iter::FusedIterator;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cons<V, Vs>(pub V, pub Vs);
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Nil;

//-------------------------------------------------------------------------
// std::ops boilerplate

macro_rules! impl_std_binop {
    (ops::$Add:ident::$add:ident) => {
        impl ops::$Add<Nil> for Nil {
            type Output = Nil;

            fn $add(self, Nil: Nil) -> Nil { Nil }
        }

        impl<H1, H2, HOut, T1, T2, TOut> ops::$Add<Cons<H2, T2>> for Cons<H1, T1>
        where
            H1: ops::$Add<H2, Output=HOut>,
            T1: ops::$Add<T2, Output=TOut>,
        {
            type Output = Cons<HOut, TOut>;

            fn $add(self, other: Cons<H2, T2>) -> Self::Output {
                Cons(
                    ops::$Add::$add(self.0, other.0),
                    ops::$Add::$add(self.1, other.1),
                )
            }
        }
    }
}

macro_rules! impl_std_unop {
    (ops::$Neg:ident::$neg:ident) => {
        impl ops::$Neg for Nil {
            type Output = Nil;

            fn $neg(self) -> Nil { Nil }
        }

        impl<H, HOut, T, TOut> ops::$Neg for Cons<H, T>
        where
            H: ops::$Neg<Output=HOut>,
            T: ops::$Neg<Output=TOut>,
        {
            type Output = Cons<HOut, TOut>;

            fn $neg(self) -> Self::Output {
                Cons(
                    ops::$Neg::$neg(self.0),
                    ops::$Neg::$neg(self.1),
                )
            }
        }
    }
}

impl_std_binop!{ops::Add::add}
impl_std_binop!{ops::Sub::sub}
impl_std_binop!{ops::Mul::mul}
impl_std_binop!{ops::Div::div}
impl_std_binop!{ops::Rem::rem}
impl_std_binop!{ops::BitAnd::bitand}
impl_std_binop!{ops::BitOr::bitor}
impl_std_binop!{ops::BitXor::bitxor}
impl_std_binop!{ops::Shl::shl}
impl_std_binop!{ops::Shr::shr}
impl_std_unop!{ops::Neg::neg}
impl_std_unop!{ops::Not::not}

//-------------------------------------------------------------------------
// iterator boilerplate

// A VList of iterators behaves a zipped iterator;
// it produces VLists of the items.
//
// (alternatively Iterators could have their own set of cons-list types, but it seems wise to let
//  users reuse the same set of vlist macros)

impl<H, T, HItem, TItem> Iterator for Cons<H, T>
where
    H: Iterator<Item=HItem>,
    T: Iterator<Item=TItem>,
{
    type Item = Cons<HItem, TItem>;

    fn next(&mut self) -> Option<Self::Item> {
        let head = self.0.next()?;
        let tail = self.1.next()?;
        Some(Cons(head, tail))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // copied from std::Zip
        let (a_lower, a_upper) = self.0.size_hint();
        let (b_lower, b_upper) = self.1.size_hint();

        let lower = cmp::min(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(cmp::min(x,y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None
        };

        (lower, upper)
    }
}

impl<H, T> FusedIterator for Cons<H, T>
where
    H: FusedIterator,
    T: FusedIterator,
{}

impl<H, T> ExactSizeIterator for Cons<H, T>
where
    H: ExactSizeIterator,
    T: ExactSizeIterator,
{}

impl<H, T> DoubleEndedIterator for Cons<H, T>
where
    H: DoubleEndedIterator,
    T: DoubleEndedIterator
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let head = self.0.next_back()?;
        let tail = self.1.next_back()?;
        Some(Cons(head, tail))
    }
}

//-------------------------------------------------------------------------

// TODO: equivalent to Packable, for automatically picking the largest type.
//       When zipping two types of mismatched "largest SIMD widths", it should
//       chose the minimum width; this sounds like some pretty difficult typelevel programming.
//
//pub trait Packable {
//    type Vector: IsVector<Scalar=Self>;
//    type Size: IsSize;
//    const SIZE: Self::Size;
//}
//
//impl Packable for Nil {
//    type Vector = Nil;
//    type Size = Nil;
//    const SIZE: Self::Size = Nil;
//}
//
//impl<H, T> Packable for Cons<H, T>
//where
//    H: Packable,
//    T: Packable,
//{
//    type Vector = Cons<H::Vector, T::Vector>;
//    type Size = Cons<H::Size, T::Size>;
//    const SIZE: Self::Size = Nil;
//}
