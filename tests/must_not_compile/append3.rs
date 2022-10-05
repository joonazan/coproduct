use coproduct::{
    merge::{Append, NotPresent},
    type_inequality::IdType,
    Coproduct, MkUnion,
};

struct A;
impl IdType for A {
    const ID: u64 = 0;
}

struct B;
impl IdType for B {
    const ID: u64 = 1;
}

type C = <MkUnion!(A, B) as Append<B, NotPresent>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
