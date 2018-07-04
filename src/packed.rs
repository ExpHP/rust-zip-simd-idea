use ::vlist::{Cons, Nil};
use ::faster::vecs::f64x2;

pub trait TypeLevelUsize {
    const VALUE: usize;
}

// TODO could use typenum (or it could be overkill),
//      or maybe some sort of "exponential Peano" encoding
//      (type P8 = Double<Double<Double<P1>>>;)
pub enum P1 {}
pub enum P2 {}
pub enum P4 {}
pub enum P8 {}
pub enum P16 {}
pub enum P32 {}
impl TypeLevelUsize for P1 { const VALUE: usize = 1; }
impl TypeLevelUsize for P2 { const VALUE: usize = 2; }
impl TypeLevelUsize for P4 { const VALUE: usize = 4; }
impl TypeLevelUsize for P8 { const VALUE: usize = 8; }
impl TypeLevelUsize for P16 { const VALUE: usize = 16; }
impl TypeLevelUsize for P32 { const VALUE: usize = 32; }

/// Holds generic associated types for Packed.
pub trait PackedGats<'a> {
    /// Stands in for `&self`. Usually `&'a T` or a VList thereof.
    type Ref;
    /// Stands in for `&mut self`. Usually `&'a mut T` or a VList thereof.
    type RefMut;
    /// Usually `&'a [T]` or a VList thereof.
    type ScalarSlice;
    /// Usually `&'a mut [T]` or a VList thereof.
    type ScalarSliceMut;

    fn as_packed_ref(&'a self) -> Self::Ref;
    fn as_packed_mut(&'a mut self) -> Self::RefMut;
}

pub trait Packed: for<'a> PackedGats<'a> {
    type Scalar;
    type Width: TypeLevelUsize;
    const WIDTH: usize = Self::Width::VALUE;

    fn load_unaligned<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self;

    unsafe fn load_unaligned_unchecked<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self;

    fn store_unaligned<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut);

    unsafe fn store_unaligned_unchecked<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut);

    fn coalesce(self) -> Self::Scalar;

    #[inline(always)]
    fn extract(&self, idx: usize) -> Self::Scalar {
        Self::_proxy_extract(self.as_packed_ref(), idx)
    }

    #[inline(always)]
    unsafe fn extract_unchecked(&self, idx: usize) -> Self::Scalar {
        Self::_proxy_extract_unchecked(self.as_packed_ref(), idx)
    }

    // (these are the methods you actually implement for extract.
    //  FIXME:  This could be made less horrifyingly ugly by instead having a trait implemented on
    //          the Ref types that has `fn extract(self, usize)` and etc.)
    fn _proxy_extract<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar;

    unsafe fn _proxy_extract_unchecked<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar;

    fn replace(self, idx: usize, data: Self::Scalar) -> Self;

    unsafe fn replace_unchecked(self, idx: usize, data: Self::Scalar) -> Self;

    fn splat(data: Self::Scalar) -> Self;

    fn default() -> Self;

    #[inline(always)]
    fn scalar_reduce<T, F>(&self, mut acc: T, mut func: F) -> T
    where F: FnMut(T, Self::Scalar) -> T
    {
        for i in 0..Self::WIDTH {
            acc = func(acc, self.extract(i))
        }
        acc
    }
}

/// Complementary methods to Packed that are only implemented on true SIMD vector types
/// (the ones in std)
pub trait PrimitivePacked: Packed {
    // (this is here rather than on Packed because it would be a disaster to have e.g.
    //  a `Complex::product` that does independent products of the reals and imaginaries)
    fn product(&self) -> Self::Scalar;
}

impl<'a> PackedGats<'a> for f64x2 {
    type Ref = &'a Self;
    type RefMut = &'a mut Self;
    type ScalarSlice = &'a [f64];
    type ScalarSliceMut = &'a mut [f64];

    #[inline(always)]
    fn as_packed_ref(&'a self) -> Self::Ref { self }
    #[inline(always)]
    fn as_packed_mut(&'a mut self) -> Self::RefMut { self }
}

impl Packed for f64x2 {
    type Scalar = f64;
    type Width = P2;

    #[inline(always)]
    fn load_unaligned<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        <f64x2>::load_unaligned(data)
    }

    #[inline(always)]
    unsafe fn load_unaligned_unchecked<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        debug_assert!(data.len() >= Self::WIDTH);
        <f64x2>::load_unaligned_unchecked(data)
    }

    #[inline(always)]
    fn store_unaligned<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        <f64x2>::store_unaligned(self, data);
    }

    #[inline(always)]
    unsafe fn store_unaligned_unchecked<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        debug_assert!(data.len() >= Self::WIDTH);
        <f64x2>::store_unaligned_unchecked(self, data);
    }

    #[inline(always)]
    fn coalesce(self) -> Self::Scalar {
        for i in 1..Self::WIDTH {
            debug_assert_eq!(self.extract(i - 1), self.extract(i));
        }
        self.extract(0)
    }

    #[inline(always)]
    fn _proxy_extract<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        <f64x2>::extract(*refs, idx)
    }

    #[inline(always)]
    unsafe fn _proxy_extract_unchecked<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        debug_assert!(idx < Self::WIDTH);
        <f64x2>::extract_unchecked(*refs, idx)
    }

    #[inline(always)]
    fn replace(self, idx: usize, data: Self::Scalar) -> Self {
        <f64x2>::replace(self, idx, data)
    }

    #[inline(always)]
    unsafe fn replace_unchecked(self, idx: usize, data: Self::Scalar) -> Self {
        debug_assert!(idx < Self::WIDTH);
        <f64x2>::replace_unchecked(self, idx, data)
    }

    #[inline(always)]
    fn splat(data: Self::Scalar) -> Self {
        <f64x2>::splat(data)
    }

    #[inline(always)]
    fn default() -> Self {
        <f64x2>::splat(Self::Scalar::default())
    }
}

impl PrimitivePacked for f64x2 {
    #[inline(always)]
    fn product(&self) -> Self::Scalar {
        let mut acc = 1 as Self::Scalar;
        for i in 0..Self::WIDTH {
            acc *= self.extract(i)
        }
        acc
    }
}

impl<'a, V> PackedGats<'a> for Cons<V, Nil>
where
    V: PackedGats<'a>,
{
    type Ref = Cons<V::Ref, Nil>;
    type RefMut = Cons<V::RefMut, Nil>;
    type ScalarSlice = Cons<V::ScalarSlice, Nil>;
    type ScalarSliceMut = Cons<V::ScalarSliceMut, Nil>;

    #[inline(always)]
    fn as_packed_ref(&'a self) -> Self::Ref {
        let vlist_pat![v] = self;
        vlist![v.as_packed_ref()]
    }

    #[inline(always)]
    fn as_packed_mut(&'a mut self) -> Self::RefMut {
        let vlist_pat![v] = self;
        vlist![v.as_packed_mut()]
    }
}

impl<'a, V, Rest> PackedGats<'a> for Cons<V, Rest>
where
    V: PackedGats<'a>,
    Rest: PackedGats<'a>,
{
    type Ref = Cons<V::Ref, Rest::Ref>;
    type RefMut = Cons<V::RefMut, Rest::RefMut>;
    type ScalarSlice = Cons<V::ScalarSlice, Rest::ScalarSlice>;
    type ScalarSliceMut = Cons<V::ScalarSliceMut, Rest::ScalarSliceMut>;

    #[inline(always)]
    fn as_packed_ref(&'a self) -> Self::Ref {
        let vlist_pat![v, ...rest] = self;
        vlist![v.as_packed_ref(), ...rest.as_packed_ref()]
    }

    #[inline(always)]
    fn as_packed_mut(&'a mut self) -> Self::RefMut {
        let vlist_pat![v, ...rest] = self;
        vlist![v.as_packed_mut(), ...rest.as_packed_mut()]
    }
}

// The base case is VList![V] rather than Nil as Width would be degenerate otherwise.
impl<V> Packed for Cons<V, Nil>
where
    V: Packed,
{
    type Scalar = Cons<V::Scalar, Nil>;
    type Width = V::Width;

    #[inline(always)]
    fn load_unaligned<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        let vlist_pat![data] = data;
        vlist![V::load_unaligned(data)]
    }

    #[inline(always)]
    unsafe fn load_unaligned_unchecked<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        let vlist_pat![data] = data;
        vlist![V::load_unaligned_unchecked(data)]
    }

    #[inline(always)]
    fn store_unaligned<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        let vlist_pat![v] = self;
        let vlist_pat![data] = data;
        v.store_unaligned(data);
    }

    #[inline(always)]
    unsafe fn store_unaligned_unchecked<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        let vlist_pat![v] = self;
        let vlist_pat![data] = data;
        v.store_unaligned_unchecked(data);
    }

    #[inline(always)]
    fn coalesce(self) -> Self::Scalar {
        let vlist_pat![v] = self;
        vlist![v.coalesce()]
    }

    #[inline(always)]
    fn _proxy_extract<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        let vlist_pat![v] = refs;
        vlist![V::_proxy_extract(v, idx)]
    }

    #[inline(always)]
    unsafe fn _proxy_extract_unchecked<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        let vlist_pat![v] = refs;
        vlist![V::_proxy_extract_unchecked(v, idx)]
    }

    #[inline(always)]
    fn replace(self, idx: usize, data: Self::Scalar) -> Self {
        let vlist_pat![v] = self;
        let vlist_pat![data] = data;
        vlist![v.replace(idx, data)]
    }

    #[inline(always)]
    unsafe fn replace_unchecked(self, idx: usize, data: Self::Scalar) -> Self {
        let vlist_pat![v] = self;
        let vlist_pat![data] = data;
        vlist![v.replace_unchecked(idx, data)]
    }

    #[inline(always)]
    fn splat(data: Self::Scalar) -> Self {
        let vlist_pat![data] = data;
        vlist![V::splat(data)]
    }

    #[inline(always)]
    fn default() -> Self {
        vlist![<V as Packed>::default()]
    }
}

impl<V, Rest> Packed for Cons<V, Rest>
where
    V: Packed,
    Rest: Packed<Width = V::Width>,
{
    type Scalar = Cons<V::Scalar, Rest::Scalar>;
    type Width = V::Width;

    #[inline(always)]
    fn load_unaligned<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        let vlist_pat![v, ...rest] = data;
        vlist![
            V::load_unaligned(v),
            ...Rest::load_unaligned(rest)
        ]
    }

    #[inline(always)]
    unsafe fn load_unaligned_unchecked<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        let vlist_pat![v, ...rest] = data;
        vlist![
            V::load_unaligned_unchecked(v),
            ...Rest::load_unaligned_unchecked(rest)
        ]
    }

    #[inline(always)]
    fn store_unaligned<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        let vlist_pat![v, ...rest] = self;
        let vlist_pat![data_v, ...data_rest] = data;
        v.store_unaligned(data_v);
        rest.store_unaligned(data_rest);
    }

    #[inline(always)]
    unsafe fn store_unaligned_unchecked<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        let vlist_pat![v, ...rest] = self;
        let vlist_pat![data_v, ...data_rest] = data;
        v.store_unaligned_unchecked(data_v);
        rest.store_unaligned_unchecked(data_rest);
    }

    #[inline(always)]
    fn coalesce(self) -> Self::Scalar {
        let vlist_pat![v, ...rest] = self;
        vlist![v.coalesce(), ...rest.coalesce()]
    }

    #[inline(always)]
    fn _proxy_extract<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        let vlist_pat![v, ...rest] = refs;
        vlist![
            V::_proxy_extract(v, idx),
            ...Rest::_proxy_extract(rest, idx)
        ]
    }

    #[inline(always)]
    unsafe fn _proxy_extract_unchecked<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        let vlist_pat![v, ...rest] = refs;
        vlist![
            V::_proxy_extract_unchecked(v, idx),
            ...Rest::_proxy_extract_unchecked(rest, idx)
        ]
    }

    #[inline(always)]
    fn replace(self, idx: usize, data: Self::Scalar) -> Self {
        let vlist_pat![v, ...rest] = self;
        let vlist_pat![v_data, ...rest_data] = data;
        vlist![v.replace(idx, v_data), ...rest.replace(idx, rest_data)]
    }

    #[inline(always)]
    unsafe fn replace_unchecked(self, idx: usize, data: Self::Scalar) -> Self {
        let vlist_pat![v, ...rest] = self;
        let vlist_pat![v_data, ...rest_data] = data;
        vlist![v.replace_unchecked(idx, v_data), ...rest.replace_unchecked(idx, rest_data)]
    }

    #[inline(always)]
    fn splat(data: Self::Scalar) -> Self {
        let vlist_pat![v, ...rest] = data;
        vlist![V::splat(v), ...Rest::splat(rest)]
    }

    #[inline(always)]
    fn default() -> Self {
        vlist![<V as Packed>::default(), ...<Rest as Packed>::default()]
    }
}

// Implemented on:
// * primitive scalars (i8, f32, ...)
// * vlists thereof
//
///  Trait that helps automatically look up the largest supported vector type for a scalar.
pub trait Packable: Sized {
    type Vector: Packed<Scalar = Self> + Clone;
}

/// Complementary methods to Packable that are only implemented on true primitive scalar types.
pub trait PrimitivePackable: Packable {
    const SIZE: usize;
}

impl Packable for f64 {
    type Vector = ::faster::f64s;
}

impl PrimitivePackable for f64 {
    const SIZE: usize = 64;
}

impl<X> Packable for Cons<X, Nil>
where
    X: Packable,
{
    type Vector = Cons<X::Vector, Nil>;
}

// TODO: Less restrictive Packable impls that would allow types where
//       `<A as Packable>::Vector::Width != <B as Packable>::Vector::Width`.
//       I think it should take the minimum of the two widths, and then construct
//       vectors entirely of that width. (typenum can help there)
//
//       ...it's going to be a lot of type-level programming.
//              - ML

// FIXME: This should be for Cons<V, Rest>, like the Packed impl
impl<A, B, Rest> Packable for Cons<A, Cons<B, Rest>>
where
    A: Packable,
    B: Packable,
    Rest: Packable,
    B::Vector: Packed<Width=<A::Vector as Packed>::Width>,
    Rest::Vector: Packed<Width=<A::Vector as Packed>::Width>,
{
    type Vector = Cons<A::Vector, Cons<B::Vector, Rest::Vector>>;
}

//--------------------------------------------------------------------------------

// TODO: impl CustomPacked for tuples

/// Can be implemented to create a user-defined `Packed` type.
///
/// `Packed` has a blanket impl for types which implement this trait.
pub trait CustomPacked: Sized {
    // FIXME: Guh!! Users need to write 3 associated types and 6 methods? This is awful!

    /// The desired scalar type.
    type CustomScalar;
    /// The scalar type for BaseVector, which should be isomorphic to CustomScalar.
    type BaseScalar;
    /// Another type isomorphic to Self that implements `Packed`.
    type BaseVector: Packed<Scalar = Self::BaseScalar>;

    fn vector_into_base(vector: Self) -> Self::BaseVector;
    fn vector_from_base(vector: Self::BaseVector) -> Self;
    fn scalar_into_base(scalar: Self::CustomScalar) -> Self::BaseScalar;
    fn scalar_from_base(scalar: Self::BaseScalar) -> Self::CustomScalar;

    // TODO: Yuck!!! In order to write these signatures, users are forced to know about PackedGats!
    //       Is there anything we can do?
    fn vector_as_base<'a>(vector: &'a Self) -> <Self::BaseVector as PackedGats<'a>>::Ref;
    fn vector_as_base_mut<'a>(vector: &'a mut Self) -> <Self::BaseVector as PackedGats<'a>>::RefMut;
}

impl<'a, T> PackedGats<'a> for T
where
    T: CustomPacked
{
    type Ref = <T::BaseVector as PackedGats<'a>>::Ref;
    type RefMut = <T::BaseVector as PackedGats<'a>>::RefMut;
    type ScalarSlice = <T::BaseVector as PackedGats<'a>>::ScalarSlice;
    type ScalarSliceMut = <T::BaseVector as PackedGats<'a>>::ScalarSliceMut;

    #[inline(always)]
    fn as_packed_ref(&'a self) -> Self::Ref {
        T::vector_as_base(self)
    }

    #[inline(always)]
    fn as_packed_mut(&'a mut self) -> Self::RefMut {
        T::vector_as_base_mut(self)
    }
}

impl<T> Packed for T
where
    T: CustomPacked,
{
    type Scalar = T::CustomScalar;
    type Width = <T::BaseVector as Packed>::Width;

    #[inline(always)]
    fn load_unaligned<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        T::vector_from_base(T::BaseVector::load_unaligned(data))
    }

    #[inline(always)]
    unsafe fn load_unaligned_unchecked<'a>(data: <Self as PackedGats<'a>>::ScalarSlice) -> Self {
        T::vector_from_base(T::BaseVector::load_unaligned_unchecked(data))
    }

    #[inline(always)]
    fn store_unaligned<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        T::vector_into_base(self).store_unaligned(data)
    }

    #[inline(always)]
    unsafe fn store_unaligned_unchecked<'a>(self, data: <Self as PackedGats<'a>>::ScalarSliceMut) {
        T::vector_into_base(self).store_unaligned_unchecked(data)
    }

    #[inline(always)]
    fn coalesce(self) -> Self::Scalar {
        T::scalar_from_base(T::vector_into_base(self).coalesce())
    }

    #[inline(always)]
    fn _proxy_extract<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        T::scalar_from_base(T::BaseVector::_proxy_extract(refs, idx))
    }

    #[inline(always)]
    unsafe fn _proxy_extract_unchecked<'a>(refs: <Self as PackedGats<'a>>::Ref, idx: usize) -> Self::Scalar {
        T::scalar_from_base(T::BaseVector::_proxy_extract_unchecked(refs, idx))
    }

    #[inline(always)]
    fn replace(self, idx: usize, data: Self::Scalar) -> Self {
        let base = T::vector_into_base(self);
        let base = base.replace(idx, T::scalar_into_base(data));
        T::vector_from_base(base)
    }

    #[inline(always)]
    unsafe fn replace_unchecked(self, idx: usize, data: Self::Scalar) -> Self {
        let base = T::vector_into_base(self);
        let base = base.replace_unchecked(idx, T::scalar_into_base(data));
        T::vector_from_base(base)
    }

    #[inline(always)]
    fn splat(data: Self::Scalar) -> Self {
        T::vector_from_base(T::BaseVector::splat(T::scalar_into_base(data)))
    }

    #[inline(always)]
    fn default() -> Self {
        T::vector_from_base(T::BaseVector::default())
    }
}

//--------------------------------------------------------------------------------

mod test {
    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct Complex<V> { real: V, imag: V }

    // Did somebody say ***BOILERPLATE?***
    impl<V: Packed> CustomPacked for Complex<V> {
        type CustomScalar = Complex<V::Scalar>;
        type BaseScalar = Cons<V::Scalar, Cons<V::Scalar, Nil>>;
        type BaseVector = Cons<V, Cons<V, Nil>>;

        fn vector_into_base(Complex { real, imag }: Self) -> Self::BaseVector {
            vlist![real, imag]
        }
        fn vector_from_base(vlist_pat![real, imag]: Self::BaseVector) -> Self {
            Complex { real, imag }
        }
        fn scalar_into_base(Complex { real, imag }: Self::CustomScalar) -> Self::BaseScalar {
            vlist![real, imag]
        }
        fn scalar_from_base(vlist_pat![real, imag]: Self::BaseScalar) -> Self::CustomScalar {
            Complex { real, imag }
        }
        fn vector_as_base<'a>(Complex { real, imag }: &'a Self) -> <Self::BaseVector as PackedGats<'a>>::Ref {
            vlist![real.as_packed_ref(), imag.as_packed_ref()]
        }
        fn vector_as_base_mut<'a>(Complex { real, imag }: &'a mut Self) -> <Self::BaseVector as PackedGats<'a>>::RefMut {
            vlist![real.as_packed_mut(), imag.as_packed_mut()]
        }
    }

    #[test]
    fn custom_packed() {
        use ::faster::f64s;
        let cs = Complex::<f64s>::splat(Complex { real: 1.0, imag: 0.0 });
        assert_eq!(cs, Complex { real: f64s(1.0), imag: f64s(0.0) });
    }
}
