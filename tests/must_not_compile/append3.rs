use coproduct::{
    merge::{Append, NotPresent},
    Coproduct, MkUnion,
};

struct A;
struct B;

type C = <MkUnion!(A, B) as Append<B, NotPresent>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
