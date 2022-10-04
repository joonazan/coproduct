use crate::{type_inequality::NotEqual, EmptyUnion, Union, UnionAt};

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

pub struct Present<T>(T);
pub struct NotPresent;

trait DoesNotContain<X> {}

impl<X> DoesNotContain<X> for EmptyUnion {}

impl<X, H, T> DoesNotContain<X> for Union<H, T>
where
    H: NotEqual<X>,
    T: DoesNotContain<X>,
{
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        inject,
        type_inequality::{self, IdType},
        Coproduct, Embed, Here, IndexedDrop, MkUnion, There,
    };

    #[test]
    #[cfg_attr(miri, ignore)]
    fn compile_failures() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/must_not_compile/*.rs");
    }

    struct A;
    impl IdType for A {
        type Id = type_inequality::Zero<type_inequality::End>;
    }

    struct B;
    impl IdType for B {
        type Id = type_inequality::One<type_inequality::End>;
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
