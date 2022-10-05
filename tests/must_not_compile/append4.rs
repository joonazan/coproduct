use coproduct::{
    merge::{Append, Present},
    Coproduct, Here, MkUnion, There,
};

struct A;
struct B;

type C = <MkUnion!(A, B) as Append<A, Present<There<Here>>>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
