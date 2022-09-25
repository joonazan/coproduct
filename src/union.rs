use core::mem::ManuallyDrop;

use crate::count::{Here, There};

pub union Union<A, B> {
    head: ManuallyDrop<A>,
    tail: ManuallyDrop<B>,
}

#[derive(Copy, Clone)]
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

pub trait Inject<X, I> {
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
