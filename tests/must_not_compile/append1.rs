use coproduct::{
    merge::{Append, NotPresent},
    type_inequality::IdType,
    Coproduct, MkUnion,
};

struct A;
impl IdType for A {
    const ID: u64 = 0;
}

type C = <MkUnion!(A) as Append<A, NotPresent>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
