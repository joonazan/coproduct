pub trait NotEqual<Other> {}

impl<Other: IdType, T: IdType> NotEqual<Other> for T where T::Id: IdNotEqual<Other::Id> {}

/// Type-level id used to test type inequality.
///
/// Because type equality does not need to rely on these,
/// id collisions never cause wrong behaviour, just
/// compilation failure. Two different types with the same
/// id will fail to be equal and fail to be not equal.
/// See `tests/must_not_compile/type_id_collision.rs` for an example.
pub trait IdType {
    type Id;
}

pub struct End;
pub struct One<T>(T);
pub struct Zero<T>(T);

trait IdNotEqual<Other> {}

impl<X> IdNotEqual<End> for One<X> {}
impl<X> IdNotEqual<End> for Zero<X> {}
impl<X> IdNotEqual<One<X>> for End {}
impl<X> IdNotEqual<Zero<X>> for End {}
impl<A, B> IdNotEqual<One<A>> for Zero<B> {}
impl<A, B> IdNotEqual<Zero<A>> for One<B> {}
impl<A, B> IdNotEqual<One<A>> for One<B> where B: IdNotEqual<A> {}
impl<A, B> IdNotEqual<Zero<A>> for Zero<B> where B: IdNotEqual<A> {}
