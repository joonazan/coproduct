use crate::{EmptyUnion, Here, There, Union, UnionAt};

pub trait Merge<Other, Ds> {
    type Merged;
}

impl<Other, H, T, Hd, Td> Merge<Other, Union<Hd, Td>> for Union<H, T>
where
    Other: Append<H, Hd>,
    T: Merge<Other::Extended, Td>,
{
    type Merged = T::Merged;
}

impl<Other> Merge<Other, EmptyUnion> for EmptyUnion {
    type Merged = Other;
}

pub trait Append<X, D> {
    type Extended;
}

impl<X, I, T> Append<X, Present<I>> for T
where
    Self: UnionAt<I, X>,
{
    type Extended = Self;
}

impl<X, T> Append<X, NotPresent> for T
where
    Self: DoesNotContain<X>,
{
    type Extended = Union<X, T>;
}

// Present and NotPresent are used in tests/must_not_compile
// but otherwise they are just implementation details

#[cfg(trybuild)]
pub struct Present<T>(T);
#[cfg(not(trybuild))]
struct Present<T>(T);

#[cfg(trybuild)]
pub struct NotPresent;
#[cfg(not(trybuild))]
struct NotPresent;

trait DoesNotContain<X> {}

impl<X> DoesNotContain<X> for EmptyUnion {}

impl<X, H, T> DoesNotContain<X> for Union<H, T>
where
    H: NotEqual<X>,
    T: DoesNotContain<X>,
{
}

trait NotEqual<Other> {}

impl<Other: TypeId, T: TypeId> NotEqual<Other> for T where T::Id: NotEqual<Other::Id> {}

impl<X> NotEqual<Here> for There<X> {}
impl<X> NotEqual<There<X>> for Here {}
impl<A, B> NotEqual<There<A>> for There<B> where B: NotEqual<A> {}

/// Used to test type inequality.
///
/// Because type equality does not need to rely on these,
/// id collisions never cause wrong behaviour, just
/// compilation failure. Two different types with the same
/// id will fail to be equal and fail to be not equal.
/// See `tests/must_not_compile/type_id_collision.rs` for an example.
pub trait TypeId {
    type Id;
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{inject, Coproduct, Embed, IndexedDrop, MkUnion};

    #[test]
    fn compile_failures() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/must_not_compile/*.rs");
    }

    struct A;
    impl TypeId for A {
        type Id = Here;
    }

    struct B;
    impl TypeId for B {
        type Id = There<Here>;
    }

    #[test]
    fn append() {
        type C = <Union<A, EmptyUnion> as Append<B, NotPresent>>::Extended;
        let _c: Coproduct<C> = inject(A);

        type C2 = <MkUnion!(A, B) as Append<B, Present<There<Here>>>>::Extended;
        let _c: Coproduct<C2> = inject(A);
    }

    trait MergeTest<Other, Ds, Is1, Is2> {
        type Lcm;
        fn to_lcm(self, other: Other) -> (Self::Lcm, Self::Lcm);
    }

    impl<Other: IndexedDrop, Ds, C: IndexedDrop, Is1, Is2> MergeTest<Coproduct<Other>, Ds, Is1, Is2>
        for Coproduct<C>
    where
        C: Merge<Other, Ds>,
        Coproduct<C>: Embed<Coproduct<C::Merged>, Is1>,
        Coproduct<Other>: Embed<Coproduct<C::Merged>, Is2>,
        C::Merged: IndexedDrop,
    {
        type Lcm = Coproduct<C::Merged>;

        fn to_lcm(self, other: Coproduct<Other>) -> (Self::Lcm, Self::Lcm) {
            (self.embed(), other.embed())
        }
    }

    #[test]
    fn mergetest() {
        let a: Coproduct!(A) = inject(A);
        let b: Coproduct!(B) = inject(B);
        let _ab_ab = a.to_lcm(b);
    }

    #[test]
    fn mergetest2() {
        let a: Coproduct!(A, B) = inject(A);
        let b: Coproduct!(B) = inject(B);
        let _ab_ab = a.to_lcm(b);
    }
}
