/// Trait for properly deallocating Unions that are not Copy.
///
/// Unlike the other Indexed* traits, this one is exported because there is no
/// way to avoid mentioning it. For example to require IndexedClone it suffices
/// to write `Coproduct<T>: Clone`.
pub trait IndexedDrop {
    /// # Safety
    /// The argument `i` must be the index of the active variant
    /// of the Union.
    unsafe fn idrop(&mut self, i: u32);
}

/// This trait is implemented for Unions where variant I has type X.
pub trait At<I, X> {
    /// Create a union that contains the given value.
    fn inject(x: X) -> Self;

    /// Convert a union to the contained type.
    /// # Safety
    /// If the active variant of the coproduct is not at index I,
    /// calling this method is undefined behaviour.
    unsafe fn take(self) -> X;
}

pub trait Without<I> {
    /// The coproduct minus its Ith variant
    type Pruned;
}
