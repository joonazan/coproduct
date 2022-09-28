use core::mem::ManuallyDrop;

use crate::{public_traits::*, Here, There};

#[repr(C)]
pub union Union<A, B> {
    pub(crate) head: ManuallyDrop<A>,
    pub(crate) tail: ManuallyDrop<B>,
}

#[derive(Copy, Clone, Debug)]
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
    /// The argument `i` must be the index of the active variant
    /// of the Union.
    unsafe fn iclone(&self, i: u32) -> Self;
}

impl<H: Clone, T: IndexedClone> IndexedClone for Union<H, T> {
    unsafe fn iclone(&self, i: u32) -> Self {
        if i == 0 {
            Union {
                head: self.head.clone(),
            }
        } else {
            Union {
                tail: ManuallyDrop::new(self.tail.iclone(i - 1)),
            }
        }
    }
}

impl IndexedClone for EmptyUnion {
    unsafe fn iclone(&self, _: u32) -> Self {
        match *self {}
    }
}

pub trait IndexedDebug {
    /// # Safety
    /// The argument `i` must be the index of the active variant
    /// of the Union.
    unsafe fn ifmt(&self, f: &mut core::fmt::Formatter<'_>, i: u32) -> core::fmt::Result;
}

impl<H: core::fmt::Debug, T: IndexedDebug> IndexedDebug for Union<H, T> {
    unsafe fn ifmt(&self, f: &mut core::fmt::Formatter<'_>, i: u32) -> core::fmt::Result {
        if i == 0 {
            self.head.fmt(f)
        } else {
            self.tail.ifmt(f, i - 1)
        }
    }
}

impl IndexedDebug for EmptyUnion {
    unsafe fn ifmt(&self, _: &mut core::fmt::Formatter<'_>, _: u32) -> core::fmt::Result {
        match *self {}
    }
}

impl<H, T: IndexedDrop> IndexedDrop for Union<H, T> {
    unsafe fn idrop(&mut self, i: u32) {
        if i == 0 {
            ManuallyDrop::drop(&mut self.head)
        } else {
            self.tail.idrop(i - 1)
        }
    }
}

impl IndexedDrop for EmptyUnion {
    unsafe fn idrop(&mut self, _: u32) {
        match *self {}
    }
}

/// PartialEq cannot be implemented for Union, since it can contain
/// bytes that are full of garbage and shouldn't be compared.
pub trait IndexedEq {
    /// # Safety
    /// The argument `i` must be the index of the active variant
    /// of the Union.
    unsafe fn ieq(&self, other: &Self, i: u32) -> bool;
}

impl<H: PartialEq, T: IndexedEq> IndexedEq for Union<H, T> {
    unsafe fn ieq(&self, other: &Self, i: u32) -> bool {
        if i == 0 {
            self.head == other.head
        } else {
            self.tail.ieq(&other.tail, i - 1)
        }
    }
}

impl IndexedEq for EmptyUnion {
    unsafe fn ieq(&self, _: &Self, _: u32) -> bool {
        match *self {}
    }
}

impl<X, Rest> At<Here, X> for Union<X, Rest> {
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

impl<I, X, H, T> At<There<I>, X> for Union<H, T>
where
    T: At<I, X>,
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
/// Only use this on repr(C) unions. The output union must contain the active
/// variant of the input union.
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
