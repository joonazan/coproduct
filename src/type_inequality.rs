use core::any::TypeId;

pub trait NotEqual<Other> {}

impl<T: IdType, U: IdType> NotEqual<U> for T where T: Compare<U, EQUAL = false> {}

/// Type-level id used to test type inequality.
///
/// Because type equality does not need to rely on these,
/// id collisions never cause wrong behaviour, just
/// compilation failure. Two different types with the same
/// id will fail to be equal and fail to be not equal.
trait IdType {
    const ID: u64;
}

impl<T: 'static> IdType for T {
    const ID: u64 = typeid_hack::<T>();
}

/// This is not how TypeId is supposed to be used and will probably
/// fail to compile in the future. But for now it is the best way to
/// get a compile-time type id.
const fn typeid_hack<T: 'static>() -> u64 {
    unsafe { core::mem::transmute(TypeId::of::<T>()) }
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
    struct B;

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
