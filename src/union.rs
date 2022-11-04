use core::any::TypeId;
use core::mem::ManuallyDrop;

use crate::{public_traits::*, Here, There};

#[repr(C)]
pub union Union<A, B> {
    pub(crate) head: ManuallyDrop<A>,
    pub(crate) tail: ManuallyDrop<B>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EmptyUnion {}

#[macro_export]
macro_rules! MkUnion {
    ($t: ty) => ($crate::Union<$t, $crate::EmptyUnion>);
    ($h:ty, $($t:ty),+) => ($crate::Union<$h, $crate::MkUnion!($($t),+)>);
}

// Clone can only be implemented for unions where every variant is Copy.
// It isn't possible to call the correct clone function without knowing
// which variant is stored in the Union.
impl<A: Copy, B: Copy> Clone for Union<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Copy, B: Copy> Copy for Union<A, B> {}

/// Trait for properly cloning Unions that are not Copy.
pub trait IndexedClone {
    /// # Safety
    /// The argument `i` must be the type id of the type stored in the Union.
    unsafe fn iclone(&self, i: TypeId) -> Self;
}

impl<H: Clone + 'static, T: IndexedClone> IndexedClone for Union<H, T> {
    unsafe fn iclone(&self, i: TypeId) -> Self {
        if i == TypeId::of::<H>() {
            Union {
                head: self.head.clone(),
            }
        } else {
            Union {
                tail: ManuallyDrop::new(self.tail.iclone(i)),
            }
        }
    }
}

impl IndexedClone for EmptyUnion {
    #[inline]
    unsafe fn iclone(&self, _: TypeId) -> Self {
        match *self {}
    }
}

pub trait IndexedDebug {
    /// # Safety
    /// The argument `i` must be the type id of the type stored in the Union.
    unsafe fn ifmt(&self, f: &mut core::fmt::Formatter<'_>, i: TypeId) -> core::fmt::Result;
}

impl<H: core::fmt::Debug + 'static, T: IndexedDebug> IndexedDebug for Union<H, T> {
    unsafe fn ifmt(&self, f: &mut core::fmt::Formatter<'_>, i: TypeId) -> core::fmt::Result {
        if i == TypeId::of::<H>() {
            self.head.fmt(f)
        } else {
            self.tail.ifmt(f, i)
        }
    }
}

impl IndexedDebug for EmptyUnion {
    #[inline]
    unsafe fn ifmt(&self, _: &mut core::fmt::Formatter<'_>, _: TypeId) -> core::fmt::Result {
        match *self {}
    }
}

impl<H: 'static, T: IndexedDrop> IndexedDrop for Union<H, T> {
    unsafe fn idrop(&mut self, i: TypeId) {
        if i == TypeId::of::<H>() {
            ManuallyDrop::drop(&mut self.head)
        } else {
            self.tail.idrop(i)
        }
    }
}

impl IndexedDrop for EmptyUnion {
    #[inline]
    unsafe fn idrop(&mut self, _: TypeId) {
        match *self {}
    }
}

/// PartialEq cannot be implemented for Union, since it can contain
/// bytes that are full of garbage and shouldn't be compared.
pub trait IndexedEq {
    /// # Safety
    /// The argument `i` must be the type id of the type stored in the Union.
    unsafe fn ieq(&self, other: &Self, i: TypeId) -> bool;
}

impl<H: PartialEq + 'static, T: IndexedEq> IndexedEq for Union<H, T> {
    unsafe fn ieq(&self, other: &Self, i: TypeId) -> bool {
        if i == TypeId::of::<H>() {
            self.head == other.head
        } else {
            self.tail.ieq(&other.tail, i)
        }
    }
}

impl IndexedEq for EmptyUnion {
    #[inline]
    unsafe fn ieq(&self, _: &Self, _: TypeId) -> bool {
        match *self {}
    }
}

impl<X, Rest> UnionAt<Here, X> for Union<X, Rest> {
    fn inject(x: X) -> Self {
        Union {
            head: ManuallyDrop::new(x),
        }
    }
    unsafe fn take(self) -> X {
        ManuallyDrop::into_inner(self.head)
    }

    type Pruned = Rest;
}

impl<I, X, H, T> UnionAt<There<I>, X> for Union<H, T>
where
    T: UnionAt<I, X>,
{
    fn inject(x: X) -> Self {
        Union {
            tail: ManuallyDrop::new(T::inject(x)),
        }
    }
    unsafe fn take(self) -> X {
        ManuallyDrop::into_inner(self.tail).take()
    }

    type Pruned = Union<H, T::Pruned>;
}

/// Changes type to ANYTHING.
/// # Safety
/// Only use this on repr(C) unions. The output union must be able to contain
/// the active variant of the input union.
pub unsafe fn union_transmute<X, Y>(x: X) -> Y {
    #[repr(C)]
    union Transmuter<A, B> {
        before: ManuallyDrop<A>,
        after: ManuallyDrop<B>,
    }
    ManuallyDrop::into_inner(
        Transmuter {
            before: ManuallyDrop::new(x),
        }
        .after,
    )
}
