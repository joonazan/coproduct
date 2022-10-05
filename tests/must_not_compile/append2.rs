use coproduct::{
    merge::{Append, Present},
    Coproduct, Here, MkUnion,
};

struct A;
struct B;

type C = <MkUnion!(A) as Append<B, Present<Here>>>::Extended;

fn main() {
    let _: Coproduct<C> = coproduct::inject(A);
}
