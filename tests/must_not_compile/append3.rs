use coproduct::{
    merge::{Append, NotPresent, TypeId},
    Coproduct, Here, MkUnion, There,
};
struct A;
impl TypeId for A {
    type Id = Here;
}

struct B;
impl TypeId for B {
    type Id = There<Here>;
}

type C = <MkUnion!(A, B) as Append<B, NotPresent>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
