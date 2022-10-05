use coproduct::{
    merge::{Append, Present},
    type_inequality::IdType,
    Coproduct, Here, MkUnion, There,
};

struct A;
impl IdType for A {
    const ID: u64 = 0;
}

struct B;
impl IdType for B {
    const ID: u64 = 1;
}

type C = <MkUnion!(A, B) as Append<A, Present<There<Here>>>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
