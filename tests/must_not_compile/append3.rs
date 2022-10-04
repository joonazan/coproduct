use coproduct::{
    merge::{Append, NotPresent},
    type_inequality::{self, IdType},
    Coproduct, MkUnion,
};

struct A;
impl IdType for A {
    type Id = type_inequality::Zero<type_inequality::End>;
}

struct B;
impl IdType for B {
    type Id = type_inequality::One<type_inequality::End>;
}

type C = <MkUnion!(A, B) as Append<B, NotPresent>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
