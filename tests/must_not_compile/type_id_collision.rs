use coproduct::{
    merge::{Merge, TypeId},
    Coproduct, Here, MkUnion,
};
struct A;
impl TypeId for A {
    type Id = Here;
}

struct B;
impl TypeId for B {
    // Same as the type id for A!
    type Id = Here;
}

type C<Ds> = <MkUnion!(A) as Merge<MkUnion!(B), Ds>>::Merged;

fn main() {
    let _: Coproduct<C<_>> = coproduct::inject(A);
}
