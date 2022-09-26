use crate::{
    count::Count,
    public_traits::*,
    union::{union_transmute, IndexedClone, IndexedDebug, IndexedEq},
};

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

impl<T: IndexedDebug + Copy> core::fmt::Debug for CopyableCoproduct<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CopyableCoproduct").field(&self.0).finish()
    }
}

impl<T: IndexedDebug + IndexedDrop> core::fmt::Debug for Coproduct<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Coproduct").field(&self.0).finish()
    }
}

impl<T: IndexedEq> PartialEq for LeakingCoproduct<T> {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && unsafe { self.union.ieq(&other.union, self.tag) }
    }
}

impl<T> PartialEq for CopyableCoproduct<T>
where
    T: IndexedEq + Copy,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PartialEq for Coproduct<T>
where
    T: IndexedEq + IndexedDrop,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Implement traits on types implementing this trait to avoid writing
/// everything for CopyableCoproduct and Coproduct separately
trait CoproductWrapper {
    type T;
    fn wrap(inner: LeakingCoproduct<Self::T>) -> Self;
    fn unwrap(self) -> LeakingCoproduct<Self::T>;
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
}

impl<T: Copy> CopyableCoproduct<T> {
    pub fn inject<I, X>(x: X) -> Self
    where
        I: Count,
        T: At<I, X>,
    {
        Self::wrap(LeakingCoproduct::inject(x))
    }

    pub fn uninject<I, X>(self) -> Result<X, CopyableCoproduct<T::Pruned>>
    where
        T: Without<I> + At<I, X>,
        I: Count,
        T::Pruned: Copy,
    {
        self.unwrap().uninject().map_err(CoproductWrapper::wrap)
    }
}

impl<T: IndexedDrop> Coproduct<T> {
    pub fn inject<I, X>(x: X) -> Self
    where
        I: Count,
        T: At<I, X>,
    {
        Self::wrap(LeakingCoproduct::inject(x))
    }

    pub fn uninject<I, X>(self) -> Result<X, Coproduct<T::Pruned>>
    where
        T: Without<I> + At<I, X>,
        I: Count,
        T::Pruned: IndexedDrop,
    {
        self.unwrap().uninject().map_err(CoproductWrapper::wrap)
    }
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

impl<T: Copy> CoproductWrapper for CopyableCoproduct<T> {
    type T = T;

    fn wrap(inner: LeakingCoproduct<Self::T>) -> Self {
        Self(inner)
    }

    fn unwrap(self) -> LeakingCoproduct<Self::T> {
        self.0
    }
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

impl<T: IndexedDrop> CoproductWrapper for Coproduct<T> {
    type T = T;

    fn wrap(inner: LeakingCoproduct<Self::T>) -> Self {
        Self(inner)
    }

    fn unwrap(self) -> LeakingCoproduct<Self::T> {
        let me = core::mem::ManuallyDrop::new(self);
        unsafe { core::ptr::read(&me.0) }
    }
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
