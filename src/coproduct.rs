use crate::{
    public_traits::*,
    union::{union_transmute, IndexedClone, IndexedDebug, IndexedEq},
    EmptyUnion, Union,
};
use core::any::TypeId;
use core::mem::ManuallyDrop;

#[cfg(feature = "type_inequality_hack")]
use crate::Merge;

/// Leaks memory if the contents are not Copy.
///
/// Do not use directly. Its only purpose is to avoid duplicating methods
/// for Copy and non-Copy coproducts.
struct LeakingCoproduct<T> {
    tag: TypeId,
    union: T,
}

impl<X: IndexedDebug> core::fmt::Debug for LeakingCoproduct<X> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { self.union.ifmt(f, self.tag) }
    }
}

impl<T: IndexedEq> PartialEq for LeakingCoproduct<T> {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && unsafe { self.union.ieq(&other.union, self.tag) }
    }
}

pub trait At<I, X> {
    fn inject(x: X) -> Self;

    fn uninject(self) -> Result<X, Self::Pruned>;

    type Pruned;
}

impl<I, X: 'static, U> At<I, X> for LeakingCoproduct<U>
where
    U: UnionAt<I, X>,
{
    fn inject(x: X) -> Self {
        Self {
            tag: TypeId::of::<X>(),
            union: U::inject(x),
        }
    }

    fn uninject(self) -> Result<X, Self::Pruned> {
        if self.tag == TypeId::of::<X>() {
            Ok(unsafe { self.union.take() })
        } else {
            Err(LeakingCoproduct {
                tag: self.tag,
                union: unsafe { union_transmute(self.union) },
            })
        }
    }

    type Pruned = LeakingCoproduct<U::Pruned>;
}

/// Implemented on Coproducts that Source can be embedded into.
pub trait Embed<Target, Indices> {
    fn embed(self) -> Target;
}

impl<Res> Embed<Res, EmptyUnion> for LeakingCoproduct<EmptyUnion> {
    fn embed(self) -> Res {
        match self.union {}
    }
}

impl<Res, IH, IT, H, T> Embed<LeakingCoproduct<Res>, Union<IH, IT>>
    for LeakingCoproduct<Union<H, T>>
where
    Res: UnionAt<IH, H>,
    LeakingCoproduct<T>: Embed<LeakingCoproduct<Res>, IT>,
{
    fn embed(self) -> LeakingCoproduct<Res> {
        LeakingCoproduct {
            tag: self.tag,
            union: unsafe { union_transmute(self.union) },
        }
    }
}

pub trait Split<Selection, Indices> {
    type Remainder;

    /// Extract a subset of the possible types in a coproduct (or get the remaining possibilities)
    fn split(self) -> Result<Selection, Self::Remainder>;
}

impl<ToSplit, THead: 'static, TTail, NHead, NTail, Rem>
    Split<LeakingCoproduct<Union<THead, TTail>>, Union<NHead, NTail>> for ToSplit
where
    ToSplit: At<NHead, THead, Pruned = Rem>,
    Rem: Split<LeakingCoproduct<TTail>, NTail>,
{
    type Remainder = Rem::Remainder;

    fn split(self) -> Result<LeakingCoproduct<Union<THead, TTail>>, Self::Remainder> {
        match self.uninject() {
            Ok(found) => Ok(LeakingCoproduct::inject(found)),
            Err(rest) => rest.split().map(|subset| LeakingCoproduct {
                tag: subset.tag,
                union: Union {
                    tail: ManuallyDrop::new(subset.union),
                },
            }),
        }
    }
}

impl<ToSplit> Split<LeakingCoproduct<EmptyUnion>, EmptyUnion> for ToSplit {
    type Remainder = Self;

    #[inline(always)]
    fn split(self) -> Result<LeakingCoproduct<EmptyUnion>, Self::Remainder> {
        Err(self)
    }
}

impl<H: 'static, T> LeakingCoproduct<Union<H, T>> {
    fn take_head(self) -> Result<H, LeakingCoproduct<T>> {
        if self.tag == TypeId::of::<H>() {
            Ok(ManuallyDrop::into_inner(unsafe { self.union.head }))
        } else {
            Err(LeakingCoproduct {
                tag: self.tag,
                union: ManuallyDrop::into_inner(unsafe { self.union.tail }),
            })
        }
    }
}

/// Unwrapping is a bit more difficult for Coproduct than for CopyableCoproduct,
/// so unwrap needs to be statically dispatched.
trait CoproductWrapper<T> {
    // Returning a LeakingCoproduct doesn't cause leaks as it is private,
    // which guarantees that library users won't get their hands on it.
    // It will either be wrapped again or destroyed by the take method.
    fn unwrap(self) -> LeakingCoproduct<T>;
}

macro_rules! define_methods {
    ($type: ident, $trait: ident) => {
        impl<I, X: 'static, U: $trait> At<I, X> for $type<U>
        where
            U: UnionAt<I, X>,
            U::Pruned: $trait,
        {
            fn inject(x: X) -> Self {
                $type(LeakingCoproduct::inject(x))
            }

            fn uninject(self) -> Result<X, Self::Pruned> {
                self.unwrap().uninject().map_err($type)
            }

            type Pruned = $type<U::Pruned>;
        }

        impl<T: $trait> $type<T> {
            /// Create a new coproduct that holds the given value.
            pub fn inject<I, X>(x: X) -> Self
            where
                Self: At<I, X>,
            {
                <Self as At<I, X>>::inject(x)
            }

            /// If the coproduct contains an X, returns that value.
            /// Otherwise, returns the same coproduct, refined to indicate
            /// that it cannot contain X.
            ///
            /// This method can be used to do exhaustive case analysis on a
            /// coproduct:
            ///  ```
            /// # use coproduct::Coproduct;
            /// # struct Cat;
            /// # #[derive(Debug, PartialEq)]
            /// # struct Dog(&'static str);
            /// let animal: Coproduct!(Cat, Dog) = Coproduct::inject(Dog("Sparky"));
            ///
            /// let non_cat = match animal.uninject::<_, Cat>() {
            ///     Ok(_) => unreachable!(),
            ///     Err(non_cat) => non_cat,
            /// };
            /// match non_cat.uninject::<_, Dog>() {
            ///     Ok(dog) => assert_eq!(dog, Dog("Sparky")),
            ///     Err(non_animal) => {
            ///         // There are animals other than cats and dogs? Absurd!
            ///         non_animal.ex_falso()
            ///     }
            /// }
            ///  ```
            pub fn uninject<I, X>(self) -> Result<X, <Self as At<I, X>>::Pruned>
            where
                Self: At<I, X>,
            {
                <Self as At<I, X>>::uninject(self)
            }

            /// Convert a coproduct into another with more variants.
            pub fn embed<U, I>(self) -> U
            where
                Self: Embed<U, I>,
            {
                <Self as Embed<U, I>>::embed(self)
            }

            /// Split a coproduct into two disjoint sets. Returns the active one.
            pub fn split<U, I>(self) -> Result<U, <Self as Split<U, I>>::Remainder>
            where
                Self: Split<U, I>,
            {
                <Self as Split<U, I>>::split(self)
            }
        }

        impl<H: 'static, T> $type<Union<H, T>>
        where
            Union<H, T>: $trait,
            T: $trait,
        {
            /// Try to take the first variant out. On failure, return the
            /// remaining variants.
            pub fn take_head(self) -> Result<H, $type<T>> {
                self.unwrap().take_head().map_err($type)
            }
        }

        impl $type<EmptyUnion> {
            /// From falsehood, anything follows.
            ///
            /// Given a coproduct that cannot contain anything,
            /// just call this method.
            pub fn ex_falso<T>(&self) -> T {
                match self.0.union {}
            }
        }

        impl<T: $trait, I, U: $trait> Embed<$type<T>, I> for $type<U>
        where
            LeakingCoproduct<U>: Embed<LeakingCoproduct<T>, I>,
        {
            fn embed(self) -> $type<T> {
                $type(self.unwrap().embed())
            }
        }

        impl<T: $trait, I, U: $trait, Rem> Split<$type<T>, I> for $type<U>
        where
            LeakingCoproduct<U>: Split<LeakingCoproduct<T>, I, Remainder = LeakingCoproduct<Rem>>,
            Rem: $trait,
        {
            type Remainder = $type<Rem>;

            fn split(self) -> Result<$type<T>, $type<Rem>> {
                self.unwrap().split().map($type).map_err($type)
            }
        }

        #[cfg(feature = "type_inequality_hack")]
        impl<T: $trait, U: $trait, Ds> Merge<$type<T>, Ds> for $type<U>
        where
            U: Merge<T, Ds>,
            U::Merged: $trait,
        {
            type Merged = $type<U::Merged>;
        }

        impl<T> PartialEq for $type<T>
        where
            T: IndexedEq + $trait,
        {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl<T: IndexedDebug + $trait> core::fmt::Debug for $type<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($type)).field(&self.0).finish()
            }
        }
    };
}

/// A coproduct that can only hold copyable types.
#[derive(Copy, Clone)]
pub struct CopyableCoproduct<T>(LeakingCoproduct<T>)
where
    T: Copy;

impl<T: Copy> Clone for LeakingCoproduct<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Copy> Copy for LeakingCoproduct<T> {}

impl<T: Copy> CoproductWrapper<T> for CopyableCoproduct<T> {
    fn unwrap(self) -> LeakingCoproduct<T> {
        self.0
    }
}

define_methods!(CopyableCoproduct, Copy);

/// Builds a [CopyableCoproduct] that can hold the types given as arguments.
#[macro_export]
macro_rules! CopyableCoproduct {
    ( $( $t:ty ),+ ) => (
        $crate::CopyableCoproduct<$crate::MkUnion!( $( $t ),+ )>
    );
}

/// Can hold any type. You should use [CopyableCoproduct]
/// if your types are copyable.
pub struct Coproduct<T: IndexedDrop>(LeakingCoproduct<T>);

impl<T: IndexedClone + IndexedDrop> Clone for Coproduct<T> {
    fn clone(&self) -> Self {
        Self(LeakingCoproduct {
            tag: self.0.tag,
            union: unsafe { self.0.union.iclone(self.0.tag) },
        })
    }
}

impl<T: IndexedDrop> Drop for Coproduct<T> {
    fn drop(&mut self) {
        unsafe { self.0.union.idrop(self.0.tag) }
    }
}

impl<T: IndexedDrop> CoproductWrapper<T> for Coproduct<T> {
    fn unwrap(self) -> LeakingCoproduct<T> {
        // As Coproduct is Drop, moving out of it isn't possible,
        // which necessitates ptr::read.
        // self needs to be wrapped in ManuallyDrop because otherwise it would
        // be dropped here!
        let me = core::mem::ManuallyDrop::new(self);
        unsafe { core::ptr::read(&me.0) }
    }
}

define_methods!(Coproduct, IndexedDrop);

/// Create a coproduct containing X.
/// This standalone function more convenient than the method or trait when
/// writing very abstracted code.
pub fn inject<I, X, C>(x: X) -> C
where
    C: At<I, X>,
{
    C::inject(x)
}

/// Builds a [Coproduct] that can hold the types given as arguments.
#[macro_export]
macro_rules! Coproduct {
    ( $( $t:ty ),+ ) => (
        $crate::Coproduct<$crate::MkUnion!( $( $t ),+ )>
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::union::*;

    #[test]
    fn inject_uninject() {
        let c = Coproduct::<Union<u8, EmptyUnion>>::inject(47);
        assert_eq!(c.uninject(), Ok(47));
    }

    #[test]
    fn leak_check() {
        // This produces a memory leak detected by Miri if Drop doesn't work
        let _: Coproduct!(String) = Coproduct::inject("hello".into());
    }

    #[test]
    fn embed_split() {
        let c: Coproduct!(u8, u16) = Coproduct::inject(42u16);
        let widened: Coproduct!(u8, u16, u32, u64) = c.clone().embed();
        assert_eq!(Ok(c), widened.split())
    }

    #[test]
    fn double_free() {
        let c: Coproduct!(u8, Box<u16>) = Coproduct::inject(Box::new(42u16));
        let _d = match c.uninject::<_, u8>() {
            Ok(_) => unreachable!(),
            Err(d) => d,
        };
    }
}
