pub trait NotEqual<Other> {}

impl<T: IdType, U: IdType> NotEqual<U> for T where T: Compare<U, EQUAL = false> {}

/// Type-level id used to test type inequality.
///
/// Because type equality does not need to rely on these,
/// id collisions never cause wrong behaviour, just
/// compilation failure. Two different types with the same
/// id will fail to be equal and fail to be not equal.
/// See `tests/must_not_compile/type_id_collision.rs` for an example.
pub trait IdType {
    const ID: u64;
}

trait Compare<T> {
    const EQUAL: bool;
}

impl<T: IdType, U: IdType> Compare<T> for U {
    const EQUAL: bool = T::ID == U::ID;
}

#[cfg(test)]
mod tests {
    use super::*;
    struct A;
    impl IdType for A {
        const ID: u64 = 0;
    }

    struct B;
    impl IdType for B {
        const ID: u64 = 1;
    }

    fn require_not_equal<A, B>()
    where
        A: NotEqual<B>,
    {
    }

    #[test]
    fn testy() {
        require_not_equal::<A, B>();
    }
}
