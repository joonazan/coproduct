use crate::At;
use crate::{Embed, EmptyUnion, Here, Split, Union, UnionAt};
use core::any::Any;
use core::any::TypeId;
use core::fmt::{Debug, Formatter};
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::mem::transmute;
use core::ops::Deref;

pub type TypedAny<T> = Typed<T, dyn Any>;

pub struct Typed<Dom, T: ?Sized> {
    types: PhantomData<Dom>,
    type_id: TypeId,
    data: T,
}

impl<Dom, T: 'static> Typed<Dom, T> {
    pub fn new<I>(x: T) -> Self
    where
        Dom: UnionAt<I, T>,
    {
        Self {
            types: PhantomData,
            type_id: TypeId::of::<T>(),
            data: x,
        }
    }
}

unsafe fn change_variants<T, U>(x: &TypedAny<T>) -> &TypedAny<U> {
    &*(x as *const TypedAny<T> as *const TypedAny<U>)
}

unsafe fn change_variants_box<T, U>(x: Box<TypedAny<T>>) -> Box<TypedAny<U>> {
    transmute(x)
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
        unsafe { change_variants(self) }
    }
}

impl<Res, IH, IT, H, T> Embed<Box<TypedAny<Res>>, Union<IH, IT>> for Box<TypedAny<Union<H, T>>>
where
    Res: UnionAt<IH, H>,
    Box<TypedAny<T>>: Embed<Box<TypedAny<Res>>, IT>,
{
    fn embed(self) -> Box<TypedAny<Res>> {
        unsafe { change_variants_box(self) }
    }
}

impl<Res, T> Embed<Res, EmptyUnion> for T
where
    T: Deref<Target = TypedAny<EmptyUnion>>,
{
    fn embed(self) -> Res {
        self.ex_falso()
    }
}

pub trait Splittable<Types, Indices> {
    type Remains;
}

impl<H, T, IH, IT, ToSplit, Rem> Splittable<Union<H, T>, Union<IH, IT>> for ToSplit
where
    ToSplit: UnionAt<IH, H, Pruned = Rem>,
    Rem: Splittable<T, IT>,
{
    type Remains = Rem::Remains;
}

impl<ToSplit> Splittable<EmptyUnion, EmptyUnion> for ToSplit {
    type Remains = Self;
}

impl<'a, ToSplit, Types, Indices, Rem> Split<&'a TypedAny<Types>, Indices> for &'a TypedAny<ToSplit>
where
    Types: TypeIn,
    ToSplit: Splittable<Types, Indices, Remains = Rem>,
    Rem: 'a,
{
    type Remainder = &'a TypedAny<Rem>;

    fn split(self) -> Result<&'a TypedAny<Types>, Self::Remainder> {
        if Types::contain_typeid(self.type_id) {
            Ok(unsafe { change_variants(self) })
        } else {
            Err(unsafe { change_variants(self) })
        }
    }
}

impl<ToSplit, Types, Indices, Rem> Split<Box<TypedAny<Types>>, Indices> for Box<TypedAny<ToSplit>>
where
    Types: TypeIn,
    ToSplit: Splittable<Types, Indices, Remains = Rem>,
{
    type Remainder = Box<TypedAny<Rem>>;

    fn split(self) -> Result<Box<TypedAny<Types>>, Self::Remainder> {
        if Types::contain_typeid(self.type_id) {
            Ok(unsafe { change_variants_box(self) })
        } else {
            Err(unsafe { change_variants_box(self) })
        }
    }
}

trait TypeIn {
    fn contain_typeid(id: TypeId) -> bool;
}

impl<H: 'static, T> TypeIn for Union<H, T>
where
    T: TypeIn,
{
    fn contain_typeid(id: TypeId) -> bool {
        id == TypeId::of::<H>() || T::contain_typeid(id)
    }
}

impl TypeIn for EmptyUnion {
    #[inline]
    fn contain_typeid(_: TypeId) -> bool {
        false
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
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        self.ex_falso()
    }
}

impl<H, T> Debug for TypedAny<Union<H, T>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("TypedAny").field(&()).finish()
    }
}

impl<T> TypedAny<T> {
    /// If the coproduct contains an X, returns that value.
    /// Otherwise, returns the same coproduct, refined to indicate
    /// that it cannot contain X.
    pub fn uninject<'a, I, X: 'static>(&'a self) -> Result<&'a X, &'a TypedAny<T::Pruned>>
    where
        T: UnionAt<I, X>,
    {
        if self.type_id == TypeId::of::<X>() {
            Ok(unsafe { self.data.downcast_ref_unchecked() })
        } else {
            Err(unsafe { change_variants(self) })
        }
    }

    /// Convert a coproduct into another with more variants.
    pub fn embed<'a, U, I>(&'a self) -> U
    where
        &'a Self: Embed<U, I>,
    {
        <&'a Self as Embed<U, I>>::embed(self)
    }

    /// Split a coproduct into two disjoint sets. Returns the active one.
    pub fn split<'a, U, I>(&'a self) -> Result<U, <&'a Self as Split<U, I>>::Remainder>
    where
        &'a Self: Split<U, I>,
    {
        <&'a Self as Split<U, I>>::split(self)
    }
}

impl<I, X: 'static, T> At<I, X> for Box<TypedAny<T>>
where
    T: UnionAt<I, X>,
{
    fn inject(x: X) -> Self {
        Box::new(Typed::new(x))
    }

    fn uninject(mut self) -> Result<X, Self::Pruned> {
        if self.type_id == TypeId::of::<X>() {
            Ok(unsafe { core::ptr::read(self.data.downcast_mut_unchecked()) })
        } else {
            Err(unsafe { change_variants_box(self) })
        }
    }

    type Pruned = Box<TypedAny<T::Pruned>>;
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
    use crate::{At, Embed, Split, Typed, TypedAny};

    #[test]
    fn inject_uninject() {
        let storage = Typed::new(47);
        let c: &TypedAny!(u8) = &storage;
        assert_eq!(c.uninject(), Ok(&47));
    }

    #[test]
    fn embed_split() {
        let storage = Typed::new(42u16);
        let c: &TypedAny!(u8, u16) = &storage;
        let widened: &TypedAny!(u8, u16, u32, u64) = c.embed();
        assert_eq!(Ok(c), widened.split())
    }

    #[test]
    fn inject_uninject_box() {
        let storage = Typed::new(47);
        let c: Box<TypedAny!(u8)> = Box::new(storage);
        assert_eq!(c.uninject(), Ok(47));
    }

    #[test]
    fn embed_split_box() {
        let make = || {
            let storage = Typed::new(42u16);
            let c: Box<TypedAny!(u8, u16)> = Box::new(storage);
            c
        };
        let widened: Box<TypedAny!(u8, u16, u32, u64)> = make().embed();
        assert_eq!(Ok(make()), widened.split())
    }
}
