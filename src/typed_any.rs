use crate::{At, Embed, EmptyUnion, Here, Split, Union, UnionAt};
use core::fmt::{Debug, Formatter};
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::{any::Any, mem::transmute};

#[repr(transparent)]
pub struct TypedAny<T> {
    types: PhantomData<T>,
    data: dyn Any,
}

unsafe fn from_any_ref<T>(x: &dyn Any) -> &TypedAny<T> {
    transmute(x)
}

impl<'a, I, X: 'static, T> At<I, &'a X> for &'a TypedAny<T>
where
    T: UnionAt<I, X>,
    T::Pruned: 'a,
{
    fn inject(x: &'a X) -> Self {
        unsafe { from_any_ref(x) }
    }

    fn uninject(self) -> Result<&'a X, Self::Pruned> {
        self.data.downcast_ref().ok_or(unsafe { transmute(self) })
    }

    type Pruned = &'a TypedAny<T::Pruned>;
}

impl TypedAny<EmptyUnion> {
    pub fn ex_falso<T>(&self) -> T {
        unsafe { unreachable_unchecked() }
    }
}

impl<'a, Res, IH, IT, H, T> Embed<&'a TypedAny<Res>, Union<IH, IT>> for &'a TypedAny<Union<H, T>>
where
    Res: UnionAt<IH, H>,
    &'a TypedAny<T>: Embed<&'a TypedAny<Res>, IT>,
{
    fn embed(self) -> &'a TypedAny<Res> {
        unsafe { transmute(self) }
    }
}

impl<'a, Res> Embed<&'a TypedAny<Res>, EmptyUnion> for &'a TypedAny<EmptyUnion> {
    fn embed(self) -> &'a TypedAny<Res> {
        self.ex_falso()
    }
}

impl<'a, ToSplit, THead: 'static, TTail, NHead, NTail, Rem>
    Split<&'a TypedAny<Union<THead, TTail>>, Union<NHead, NTail>> for ToSplit
where
    ToSplit: At<NHead, &'a THead, Pruned = Rem>,
    Rem: Split<&'a TypedAny<TTail>, NTail>,
{
    type Remainder = Rem::Remainder;

    fn split(self) -> Result<&'a TypedAny<Union<THead, TTail>>, Self::Remainder> {
        match self.uninject() {
            Ok(found) => Ok(crate::inject(found)),
            Err(rest) => rest.split().map(|x| unsafe { transmute(x) }),
        }
    }
}

impl<'a, ToSplit> Split<&'a TypedAny<EmptyUnion>, EmptyUnion> for ToSplit {
    type Remainder = Self;

    #[inline]
    fn split(self) -> Result<&'a TypedAny<EmptyUnion>, Self::Remainder> {
        Err(self)
    }
}

impl<H: PartialEq + 'static, T> PartialEq for TypedAny<Union<H, T>>
where
    TypedAny<T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.uninject::<Here, H>(), other.uninject::<Here, H>()) {
            (Ok(a), Ok(b)) => a == b,
            (Err(a), Err(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialEq for TypedAny<EmptyUnion> {
    fn eq(&self, _: &Self) -> bool {
        self.ex_falso()
    }
}

impl Debug for TypedAny<EmptyUnion> {
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.ex_falso()
    }
}

impl<H, T> Debug for TypedAny<Union<H, T>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypedAny").field(&()).finish()
    }
}

impl<T> TypedAny<T> {
    /// If the coproduct contains an X, returns that value.
    /// Otherwise, returns the same coproduct, refined to indicate
    /// that it cannot contain X.
    pub fn uninject<'a, I, X>(&'a self) -> Result<&'a X, <&'a Self as At<I, &'a X>>::Pruned>
    where
        &'a Self: At<I, &'a X>,
    {
        <&'a Self as At<I, &X>>::uninject(self)
    }

    /// Convert a coproduct into another with more variants.
    pub fn embed<'a, U, I>(&'a self) -> U
    where
        &'a Self: Embed<U, I>,
    {
        <&'a Self as Embed<U, I>>::embed(self)
    }

    /// Split a coproduct into two disjoint sets. Returns the active one.
    pub fn split<'a, U: ?Sized, I>(
        &'a self,
    ) -> Result<&'a U, <&'a Self as Split<&'a U, I>>::Remainder>
    where
        &'a Self: Split<&'a U, I>,
    {
        <&'a Self as Split<&'a U, I>>::split(self)
    }
}

/// Builds a [TypedAny] that can hold the types given as arguments.
#[macro_export]
macro_rules! TypedAny {
    ( $( $t:ty ),+ ) => (
        $crate::TypedAny<$crate::MkUnion!( $( $t ),+ )>
    );
}

#[cfg(test)]
mod tests {

    #[test]
    fn inject_uninject() {
        let c: &TypedAny!(u8) = crate::inject(&47);
        assert_eq!(c.uninject(), Ok(&47));
    }

    #[test]
    fn embed_split() {
        let c: &TypedAny!(u8, u16) = crate::inject(&42u16);
        let widened: &TypedAny!(u8, u16, u32, u64) = c.embed();
        assert_eq!(Ok(c), widened.split())
    }
}
