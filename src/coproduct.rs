use crate::{
    count::Count,
    public_traits::*,
    union::{union_transmute, IndexedClone, IndexedDebug, IndexedEq},
    EmptyUnion, Union,
};
use core::mem::ManuallyDrop;

/// Leaks memory if the contents are not Copy.
///
/// Do not use directly. Its only purpose is to avoid duplicating methods
/// for Copy and non-Copy coproducts.
struct LeakingCoproduct<T> {
    tag: u32,
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

impl<T> LeakingCoproduct<T> {
    fn inject<I, X>(x: X) -> Self
    where
        I: Count,
        T: At<I, X>,
    {
        Self {
            tag: I::count(),
            union: T::inject(x),
        }
    }

    fn uninject<I, X>(self) -> Result<X, LeakingCoproduct<T::Pruned>>
    where
        T: At<I, X>,
        I: Count,
    {
        if self.tag == I::count() {
            Ok(unsafe { self.union.take() })
        } else {
            let tag = if self.tag < I::count() {
                self.tag
            } else {
                self.tag - 1
            };
            Err(LeakingCoproduct {
                tag,
                union: unsafe { union_transmute(self.union) },
            })
        }
    }

    fn embed<U, I>(self) -> LeakingCoproduct<U>
    where
        LeakingCoproduct<U>: Embed<LeakingCoproduct<T>, I>,
    {
        Embed::embed(self)
    }
}

/// Implemented on Coproducts that Source can be embedded into.
pub trait Embed<Source, Indices> {
    fn embed(src: Source) -> Self;
}

impl<Res> Embed<LeakingCoproduct<EmptyUnion>, EmptyUnion> for Res {
    fn embed(src: LeakingCoproduct<EmptyUnion>) -> Self {
        match src.union {}
    }
}

impl<Res, IH, IT, H, T> Embed<LeakingCoproduct<Union<H, T>>, Union<IH, IT>>
    for LeakingCoproduct<Res>
where
    Res: At<IH, H>,
    IH: Count,
    LeakingCoproduct<Res>: Embed<LeakingCoproduct<T>, IT>,
{
    fn embed(src: LeakingCoproduct<Union<H, T>>) -> Self {
        match src.take_head() {
            Ok(x) => LeakingCoproduct::inject(x),
            Err(x) => LeakingCoproduct::embed(x),
        }
    }
}

pub trait Split<Selection, Indices>: Sized {
    type Remainder;

    /// Extract a subset of the possible types in a coproduct (or get the remaining possibilities)
    fn split(self) -> Result<Selection, Self::Remainder>;
}

impl<H, T, THead, TTail, NHead: Count, NTail, Rem>
    Split<LeakingCoproduct<Union<THead, TTail>>, Union<NHead, NTail>>
    for LeakingCoproduct<Union<H, T>>
where
    Union<H, T>: At<NHead, THead, Pruned = Rem>,
    LeakingCoproduct<Rem>: Split<LeakingCoproduct<TTail>, NTail>,
{
    type Remainder = <LeakingCoproduct<Rem> as Split<LeakingCoproduct<TTail>, NTail>>::Remainder;

    fn split(self) -> Result<LeakingCoproduct<Union<THead, TTail>>, Self::Remainder> {
        match self.uninject::<NHead, THead>() {
            Ok(found) => Ok(LeakingCoproduct::inject(found)),
            Err(rest) => rest.split().map(|subset| LeakingCoproduct {
                tag: subset.tag + 1,
                union: Union {
                    tail: ManuallyDrop::new(subset.union),
                },
            }),
        }
    }
}

impl<Choices> Split<LeakingCoproduct<EmptyUnion>, EmptyUnion> for Choices {
    type Remainder = Self;

    #[inline(always)]
    fn split(self) -> Result<LeakingCoproduct<EmptyUnion>, Self::Remainder> {
        Err(self)
    }
}

impl<H, T> LeakingCoproduct<Union<H, T>> {
    fn take_head(self) -> Result<H, LeakingCoproduct<T>> {
        if self.tag == 0 {
            Ok(ManuallyDrop::into_inner(unsafe { self.union.head }))
        } else {
            Err(LeakingCoproduct {
                tag: self.tag - 1,
                union: ManuallyDrop::into_inner(unsafe { self.union.tail }),
            })
        }
    }
}

/// Unwrapping is a bit more difficult for Coproduct than for CopyableCoproduct,
/// so unwrap needs to be statically dispatched.
trait CoproductWrapper<T> {
    fn unwrap(self) -> LeakingCoproduct<T>;
}

macro_rules! define_methods {
    ($type: ident, $trait: ident) => {
        impl<T: $trait> $type<T> {
            /// Create a new coproduct that holds the given value.
            pub fn inject<I, X>(x: X) -> Self
            where
                I: Count,
                T: At<I, X>,
            {
                Self(LeakingCoproduct::inject(x))
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
            pub fn uninject<I, X>(self) -> Result<X, $type<T::Pruned>>
            where
                T: At<I, X>,
                I: Count,
                T::Pruned: $trait,
            {
                self.unwrap().uninject().map_err($type)
            }

            /// Convert a coproduct into another with more variants.
            pub fn embed<U, I>(self) -> U
            where
                U: Embed<Self, I>,
            {
                <U as Embed<Self, I>>::embed(self)
            }

            /// Split a coproduct into two disjoint sets. Returns the active one.
            pub fn split<U, I>(self) -> Result<U, <Self as Split<U, I>>::Remainder>
            where
                Self: Split<U, I>,
            {
                <Self as Split<U, I>>::split(self)
            }
        }

        impl<H, T> $type<Union<H, T>>
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
            fn embed(src: $type<T>) -> Self {
                Self(src.unwrap().embed())
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
                f.debug_tuple("Coproduct").field(&self.0).finish()
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
        let me = core::mem::ManuallyDrop::new(self);
        unsafe { core::ptr::read(&me.0) }
    }
}

define_methods!(Coproduct, IndexedDrop);

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
}
