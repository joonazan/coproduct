use coproduct::{
    merge::{Append, NotPresent},
    Coproduct, MkUnion,
};

struct A;

type C = <MkUnion!(A) as Append<A, NotPresent>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
