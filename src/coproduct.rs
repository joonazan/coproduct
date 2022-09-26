use crate::{
    count::Count,
    public_traits::*,
    union::{union_transmute, IndexedClone, IndexedDebug, IndexedEq},
    EmptyUnion, Union,
};
use core::mem::ManuallyDrop;
use frunk::{HCons, HNil};

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
        T: Without<I> + At<I, X>,
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

impl<Res> Embed<LeakingCoproduct<EmptyUnion>, HNil> for Res {
    fn embed(src: LeakingCoproduct<EmptyUnion>) -> Self {
        match src.union {}
    }
}

impl<Res, IH, IT, H, T> Embed<LeakingCoproduct<Union<H, T>>, HCons<IH, IT>>
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
            pub fn inject<I, X>(x: X) -> Self
            where
                I: Count,
                T: At<I, X>,
            {
                Self(LeakingCoproduct::inject(x))
            }

            pub fn uninject<I, X>(self) -> Result<X, $type<T::Pruned>>
            where
                T: Without<I> + At<I, X>,
                I: Count,
                T::Pruned: $trait,
            {
                self.unwrap().uninject().map_err($type)
            }

            pub fn embed<U: $trait, I>(self) -> $type<U>
            where
                $type<U>: Embed<$type<T>, I>,
            {
                <$type<U> as Embed<$type<T>, I>>::embed(self)
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

/// Use this whenever possible. It has strictly less code than Coproduct
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

/// This one supports types are not Copy. You should use CopyableCoproduct
/// if possible.
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
}
