use core::mem::ManuallyDrop;

use crate::count::{Here, There};

#[repr(C)]
pub union Union<A, B> {
    head: ManuallyDrop<A>,
    tail: ManuallyDrop<B>,
}

#[derive(Copy, Clone, Debug)]
pub enum EmptyUnion {}

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

/// Trait for properly deallocating Unions that are not Copy.
pub trait IndexedDrop {
    /// # Safety
    /// The argument `i` must be the index of the active variant
    /// of the Union.
    unsafe fn idrop(&mut self, i: u32);
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

pub trait Inject<X, I> {
    /// Create a union that contains the given value.
    fn inject(x: X) -> Self;
}

impl<X, Rest> Inject<X, Here> for Union<X, Rest> {
    fn inject(x: X) -> Self {
        Union {
            head: ManuallyDrop::new(x),
        }
    }
}

impl<X, I, H, T> Inject<X, There<I>> for Union<H, T>
where
    T: Inject<X, I>,
{
    fn inject(x: X) -> Self {
        Union {
            tail: ManuallyDrop::new(T::inject(x)),
        }
    }
}

/// Convert a union to the contained type.
pub trait UnsafeTake<X, I> {
    /// # Safety
    /// If the active variant of the coproduct is not at index I,
    /// calling this method is undefined behaviour.
    unsafe fn take(self) -> X;
}

impl<H, T> UnsafeTake<H, Here> for Union<H, T> {
    unsafe fn take(self) -> H {
        ManuallyDrop::into_inner(self.head)
    }
}

impl<H, T, X, I> UnsafeTake<X, There<I>> for Union<H, T>
where
    T: UnsafeTake<X, I>,
{
    unsafe fn take(self) -> X {
        ManuallyDrop::into_inner(self.tail).take()
    }
}

/// Removes one variant from a Union.
/// # Safety
/// This function is only safe to call if the variant removed
/// is not active. Otherwise it will produce a Union in an invalid state.
pub unsafe fn prune<T, I>(cp: T) -> T::Pruned
where
    T: Without<I>,
{
    #[repr(C)]
    union Transmuter<T, Index>
    where
        T: Without<Index>,
    {
        before: ManuallyDrop<T>,
        after: ManuallyDrop<T::Pruned>,
    }
    ManuallyDrop::into_inner(
        Transmuter {
            before: ManuallyDrop::new(cp),
        }
        .after,
    )
}

pub trait Without<I> {
    type Pruned;
}

impl<H, T> Without<Here> for Union<H, T> {
    type Pruned = T;
}

impl<I, H, T> Without<There<I>> for Union<H, T>
where
    T: Without<I>,
{
    type Pruned = Union<H, T::Pruned>;
}
