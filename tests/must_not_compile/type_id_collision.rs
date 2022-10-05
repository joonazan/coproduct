use coproduct::{type_inequality::IdType, Coproduct, Merge, MkUnion};

struct A;
impl IdType for A {
    const ID: u64 = 0;
}

struct B;
impl IdType for B {
    // same Id as A!
    const ID: u64 = 0;
}

type C<Ds> = <MkUnion!(A) as Merge<MkUnion!(B), Ds>>::Merged;

fn main() {
    let _: Coproduct<C<_>> = coproduct::inject(A);
}
