use coproduct::{Coproduct, MkUnion, Union};

#[derive(Debug)]
struct A;
#[derive(Debug)]
struct B;
#[derive(Debug)]
struct C;
#[derive(Debug)]
struct D;

type ABC = MkUnion!(A, B, C);

fn main() {
    let abc: Coproduct<ABC> = Coproduct::inject(A);
    let abcd: Coproduct<Union<D, ABC>> = abc.embed();
    println!("{:?}", abcd);
}
