use coproduct::{
    type_inequality::{self, IdType},
    Coproduct, Merge, MkUnion,
};

struct A;
impl IdType for A {
    type Id = type_inequality::Zero<type_inequality::End>;
}

struct B;
impl IdType for B {
    // same Id as A!
    type Id = type_inequality::Zero<type_inequality::End>;
}

type C<Ds> = <MkUnion!(A) as Merge<MkUnion!(B), Ds>>::Merged;

fn main() {
    let _: Coproduct<C<_>> = coproduct::inject(A);
}
