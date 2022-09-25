use crate::{
    count::Count,
    union::{IndexedClone, IndexedDrop, Inject},
};

/// Leaks memory if the contents are not Copy.
///
/// Do not use directly. Its only purpose is to avoid duplicating methods
/// for Copy and non-Copy coproducts.
struct LeakingCoproduct<T> {
    tag: u32,
    union: T,
}

/// Implement traits on types implementing this trait to avoid writing
/// everything for CopyableCoproduct and Coproduct separately
trait CoproductWrapper {
    type T;
    fn wrap(inner: LeakingCoproduct<Self::T>) -> Self;
}

impl<T, X, I> Inject<X, I> for LeakingCoproduct<T>
where
    I: Count,
    T: Inject<X, I>,
{
    fn inject(x: X) -> Self {
        Self {
            tag: I::count(),
            union: T::inject(x),
        }
    }
}

impl<X, I, T, C> Inject<X, I> for C
where
    I: Count,
    T: Inject<X, I>,
    C: CoproductWrapper<T = T>,
{
    fn inject(x: X) -> Self {
        C::wrap(LeakingCoproduct::inject(x))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::union::*;

    #[test]
    fn inject() {
        Coproduct::<Union<u8, EmptyUnion>>::inject(1);
    }
}
