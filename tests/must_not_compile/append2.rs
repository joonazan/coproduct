use coproduct::{
    merge::{Append, Present},
    type_inequality::IdType,
    Coproduct, Here, MkUnion,
};

struct A;
impl IdType for A {
    const ID: u64 = 0;
}

struct B;
impl IdType for B {
    const ID: u64 = 1;
}

type C = <MkUnion!(A) as Append<B, Present<Here>>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
