use coproduct::{
    merge::{Append, Present},
    type_inequality::{self, IdType},
    Coproduct, Here, MkUnion,
};

struct A;
impl IdType for A {
    type Id = type_inequality::Zero<type_inequality::End>;
}

struct B;
impl IdType for B {
    type Id = type_inequality::One<type_inequality::End>;
}

type C = <MkUnion!(A) as Append<B, Present<Here>>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
